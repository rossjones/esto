use esto_core::{record::Record, storage::Storage};

use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use store::storer_server::{Storer, StorerServer};
use store::{StoreReply, StoreRequest};

pub mod store {
    tonic::include_proto!("store");
}

#[derive(Debug)]
pub struct LocalStorer {
    storage: Storage,
}

#[tonic::async_trait]
impl Storer for LocalStorer {
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
}

pub async fn run(storage: Storage) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = LocalStorer { storage };

    Server::builder()
        .add_service(StorerServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
