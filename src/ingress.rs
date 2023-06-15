use k8s_openapi::api::networking::v1::{HTTPIngressPath, HTTPIngressRuleValue, IngressBackend, IngressRule, IngressServiceBackend, ServiceBackendPort};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::{Api, Client, CustomResource};
use kube::api::PostParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use tracing::{info, warn};

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
        let sync_request = request.into_inner();
        let owner = sync_request.owner.unwrap();
        info!("owner: {:?}", owner);
        let spec: Ingress = serde_json::from_slice(sync_request.spec.as_slice())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        info!("spec: {:?}", spec);
        use k8s_openapi::api::networking::v1::{Ingress as K8sIngress, IngressSpec as K8sIngressSpec};
        let rules = spec.routes.iter().map(|route| {
            let paths = vec![
                HTTPIngressPath {
                    backend: IngressBackend {
                        resource: None,
                        service: Some(IngressServiceBackend {
                            name: owner.name.clone(),
                            port: Some(ServiceBackendPort {
                                name: None,
                                number: Some(route.port.into()),
                            }),
                        }),
                    },
                    path: Some(route.path.clone()),
                    path_type: "Prefix".to_string(),
                }
            ];
            IngressRule {
                host: Some(route.host.clone()),
                http: Some(HTTPIngressRuleValue {paths}),
            }
        }).collect();
        let ingress_client: Api<K8sIngress> = Api::default_namespaced(self.client.clone());
        let ingress = K8sIngress {
            metadata: ObjectMeta {
                name: Some(owner.name.clone()),
                namespace: Some(owner.namespace.clone()),
                owner_references: Some(vec![OwnerReference {
                    api_version: owner.api_version.clone(),
                    kind: owner.kind.clone(),
                    name: owner.name.clone(),
                    uid: owner.uid.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            spec: Some(K8sIngressSpec {
                rules: Some(rules),
                ..Default::default()
            }),
            ..Default::default()
        };
        ingress_client.replace(owner.name.as_str(), &PostParams::default(), &ingress).await
            .map_err(|e| Status::aborted(e.to_string()))?;
        let (_, rx) = mpsc::channel(1);
        Ok(Response::new(ReceiverStream::from(rx)))
    }

    type DeleteStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn delete(&self, _request: Request<SyncRequest>) -> Result<Response<Self::DeleteStream>, Status> {
        warn!("delete called, not implemented");
        Err(Status::new(Code::Ok, "Delete not implemented"))
    }

    async fn default(&self, _request: Request<SyncRequest>) -> Result<Response<DefaultResponse>, Status> {
        // TODO: Return some defaults
        let resp = DefaultResponse {
            spec: vec![],
        };
        Ok(Response::new(resp))
    }

    async fn validate(&self, _request: Request<ValidationRequest>) -> Result<Response<ValidationResponse>, Status> {
        // TODO: Do proper validation
        Ok(Response::new(ValidationResponse::default()))
    }

    async fn documentation(&self, _request: Request<DocumentationRequest>) -> Result<Response<DocumentationResponse>, Status> {
        let page = "TODO: Documentation is still pending".as_bytes().to_vec();
        let pages = vec![page];
        let resp = DocumentationResponse {
            pages,
        };
        Ok(Response::new(resp))
    }
}
