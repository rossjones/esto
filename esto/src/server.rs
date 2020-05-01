use tonic::{transport::Server, Request, Response, Status};

use store::storer_server::{Storer, StorerServer};
use store::{StoreReply, StoreRequest};

pub mod store {
    tonic::include_proto!("store");
}

#[derive(Debug, Default)]
pub struct LocalStorer {}

#[tonic::async_trait]
impl Storer for LocalStorer {
    async fn store_record(
        &self,
        request: Request<StoreRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<StoreReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = StoreReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = LocalStorer::default();

    Server::builder()
        .add_service(StorerServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
