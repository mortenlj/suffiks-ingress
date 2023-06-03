use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status, transport::Server};

use crate::protogen::extension::{DefaultResponse, DocumentationRequest, DocumentationResponse, Response as ExtensionResponse, SyncRequest, ValidationRequest, ValidationResponse};
use crate::protogen::extension::extension_server::{Extension, ExtensionServer};

mod protogen;

#[derive(Default)]
pub struct IngressHandler {}

#[tonic::async_trait]
impl Extension for IngressHandler {
    type SyncStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn sync(&self, request: Request<SyncRequest>) -> Result<Response<Self::SyncStream>, Status> {
        todo!()
    }

    type DeleteStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn delete(&self, request: Request<SyncRequest>) -> Result<Response<Self::DeleteStream>, Status> {
        todo!()
    }

    async fn default(&self, _request: Request<SyncRequest>) -> Result<Response<DefaultResponse>, Status> {
        Err(Status::new(Code::Unimplemented, "Default not implemented"))
    }

    async fn validate(&self, _request: Request<ValidationRequest>) -> Result<Response<ValidationResponse>, Status> {
        Err(Status::new(Code::Unimplemented, "Validate not implemented"))
    }

    async fn documentation(&self, _request: Request<DocumentationRequest>) -> Result<Response<DocumentationResponse>, Status> {
        Err(Status::new(Code::Unimplemented, "Documentation not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "0.0.0.0:8080".parse().unwrap();
    // creating a service
    let ingress = <IngressHandler as Default>::default();
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(ExtensionServer::new(ingress))
        .serve(addr)
        .await?;
    Ok(())
}
