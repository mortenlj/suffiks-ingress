use kube::{Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use tracing::warn;

use crate::protogen::extension::{DefaultResponse, DocumentationRequest, DocumentationResponse, Response as ExtensionResponse, SyncRequest, ValidationRequest, ValidationResponse};
use crate::protogen::extension::extension_server::Extension;

/// Fake spec to help generating openapi schema for extension
#[derive(Serialize, Deserialize, CustomResource, JsonSchema, Debug, Clone)]
#[kube(group = "suffiks.com", version = "v1", kind = "XIngress", namespaced)]
#[serde(rename_all = "camelCase")]
pub struct XIngressSpec {
    ingress: Ingress,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Ingress {
    /// List of routes this application will handle
    routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Route {
    host: String,
    path: String,
    port: u16,
    #[serde(rename = "type")]
    route_type: RouteType,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RouteType {
    Http,
    Grpc,
}

pub struct IngressHandler {
    client: Client,
}

impl IngressHandler {
    pub fn new(client: Client) -> Self {
        Self {
            client
        }
    }
}

#[tonic::async_trait]
impl Extension for IngressHandler {
    type SyncStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn sync(&self, request: Request<SyncRequest>) -> Result<Response<Self::SyncStream>, Status> {
        warn!("sync called, not implemented");
        Err(Status::new(Code::Ok, "Sync not implemented"))
    }

    type DeleteStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn delete(&self, request: Request<SyncRequest>) -> Result<Response<Self::DeleteStream>, Status> {
        warn!("delete called, not implemented");
        Err(Status::new(Code::Ok, "Delete not implemented"))
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
