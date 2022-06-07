use esto_core::{record::Record, storage::Storage};

use futures_util::FutureExt;
use tokio::sync::oneshot;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use esto_rpc::esto_server::{Esto, EstoServer};
use esto_rpc::{Event, ReadEventList, ReadRequest};
use esto_rpc::{StoreReply, StoreRequest};

pub mod esto_rpc {
    tonic::include_proto!("esto_rpc");
}

#[derive(Debug)]
pub struct LocalStorage {
    storage: Storage,
}

#[tonic::async_trait]
impl Esto for LocalStorage {
    async fn store_record(
        &self,
        request: Request<StoreRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<StoreReply>, Status> {
        let r = request.into_inner();

        // Create record
        let record = Record::new(
            Uuid::parse_str(&r.entity_id).unwrap(),
            &r.entity_type,
            &r.event_name,
            &r.event_data,
        );
        // Store record
        let response = match self.storage.write(&record) {
            Ok(()) => {
                let idx = self.storage.get_index(record.entity_id).unwrap();
                format!("len: {}", idx.records.len())
            }
            Err(x) => x.to_string(),
        };

        let reply = StoreReply { message: response };
        Ok(Response::new(reply))
    }

    async fn read_record(
        &self,
        request: Request<ReadRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<ReadEventList>, Status> {
        let r = request.into_inner();

        let id = Uuid::parse_str(&r.entity_id).unwrap();

        let records_vec = self.storage.read(id).unwrap();
        let records: Vec<_> = records_vec.iter().map(|rec| Record::decode(rec)).collect();

        let events = records
            .iter()
            .map(|rec| Event {
                entity_id: rec.entity_id.to_string(),
                entity_type: rec.entity_type.to_string(),
                event_name: rec.event_name.to_string(),
                event_data: rec.event_data.to_string(),
                timestamp: rec.timestamp.as_secs(),
            })
            .collect();

        Ok(Response::new(ReadEventList { events }))
    }
}

pub async fn run(
    storage: Storage,
    rx: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let storage = LocalStorage { storage };

    Server::builder()
        .add_service(EstoServer::new(storage))
        .serve_with_shutdown(addr, rx.map(drop))
        .await?;

    Ok(())
}
