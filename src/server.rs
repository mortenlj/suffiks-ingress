mod protogen;

use tonic::{Request, Response, Status, transport::Server};
use tokio_stream::wrappers::ReceiverStream;
use crate::protogen::extension::extension_server::{Extension, ExtensionServer};
use crate::protogen::extension::{DefaultResponse, DocumentationRequest, DocumentationResponse, SyncRequest, ValidationRequest, ValidationResponse, Response as ExtensionResponse};


#[derive(Default)]
pub struct Ingress {}

#[tonic::async_trait]
impl Extension for Ingress {
    type SyncStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn sync(&self, request: Request<SyncRequest>) -> Result<Response<Self::SyncStream>, Status> {
        todo!()
    }

    type DeleteStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn delete(&self, request: Request<SyncRequest>) -> Result<Response<Self::DeleteStream>, Status> {
        todo!()
    }

    async fn default(&self, request: Request<SyncRequest>) -> Result<Response<DefaultResponse>, Status> {
        todo!()
    }

    async fn validate(&self, request: Request<ValidationRequest>) -> Result<Response<ValidationResponse>, Status> {
        todo!()
    }

    async fn documentation(&self, request: Request<DocumentationRequest>) -> Result<Response<DocumentationResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();
    // creating a service
    let ingress = <Ingress as Default>::default();
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(ExtensionServer::new(ingress))
        .serve(addr)
        .await?;
    Ok(())
}
