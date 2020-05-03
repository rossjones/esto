use esto_core::{record::Record, storage::Storage};

use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use esto_rpc::esto_server::{Esto, EstoServer};
use esto_rpc::{ReadReply, ReadRequest};
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
        println!("Got a request: {:?}", request);

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
    ) -> Result<Response<ReadReply>, Status> {
        println!("Got a request: {:?}", request);
        let r = request.into_inner();

        let id = Uuid::parse_str(&r.entity_id).unwrap();

        let record_bytes = self.storage.read(id).unwrap();

        // decode record_bytes and then jsonify

        let reply = ReadReply {
            entity_data: format!("{:?}", record_bytes),
        };
        Ok(Response::new(reply))
    }
}

pub async fn run(storage: Storage) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let storage = LocalStorage { storage };

    Server::builder()
        .add_service(EstoServer::new(storage))
        .serve(addr)
        .await?;

    Ok(())
}
