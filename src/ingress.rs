use std::collections::BTreeMap;
use k8s_openapi::api::networking::v1::{HTTPIngressPath, HTTPIngressRuleValue, IngressBackend, IngressRule, IngressServiceBackend, ServiceBackendPort};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::{Api, Client, CustomResource};
use kube::api::PostParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};
use tracing::{debug, error, info, warn};

use crate::protogen::extension::{DefaultResponse, DocumentationRequest, DocumentationResponse, Owner, Response as ExtensionResponse, SyncRequest, ValidationRequest, ValidationResponse};
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
    /// IngressClass to use
    ingress_class: Option<String>,
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

    fn build_rules(owner: &Owner, spec: &Ingress) -> Option<Vec<IngressRule>> {
        Some(spec.routes.iter().map(|route| {
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
                http: Some(HTTPIngressRuleValue { paths }),
            }
        }).collect())
    }
}

#[tonic::async_trait]
impl Extension for IngressHandler {
    type SyncStream = ReceiverStream<Result<ExtensionResponse, Status>>;

    async fn sync(&self, request: Request<SyncRequest>) -> Result<Response<Self::SyncStream>, Status> {
        let sync_request = request.into_inner();
        let owner = sync_request.owner.unwrap();
        debug!("owner: {:?}", owner);
        let spec: Ingress = serde_json::from_slice::<XIngressSpec>(sync_request.spec.as_slice())
            .map_err(|e| {
                let string_spec = std::str::from_utf8(sync_request.spec.as_slice()).unwrap();
                error!(spec = string_spec, "failed to parse spec: {}", e);
                Status::invalid_argument(e.to_string())
            })
            .map(|xspec| xspec.ingress)?;
        debug!("spec: {:?}", spec);

        use k8s_openapi::api::networking::v1::{Ingress as K8sIngress, IngressSpec as K8sIngressSpec, IngressTLS as K8sIngressTLS};

        let rules = Self::build_rules(&owner, &spec);
        let owner_references = Some(vec![OwnerReference {
            api_version: owner.api_version.clone(),
            kind: owner.kind.clone(),
            name: owner.name.clone(),
            uid: owner.uid.clone(),
            ..Default::default()
        }]);

        let tls_hosts = spec.routes.iter().map(|route| route.host.clone()).collect();
        let tls: Option<Vec<K8sIngressTLS>> = Some(K8sIngressTLS {
            hosts: Some(tls_hosts),
            secret_name: Some(format!("{}-ingress-cert", owner.name)),
        }).map(|x| vec![x]);

        let ingress_client: Api<K8sIngress> = Api::namespaced(self.client.clone(), owner.namespace.as_str());

        match ingress_client.get_opt(owner.name.as_str()).await {
            Ok(None) => {
                info!("Creating ingress for {}", owner.name);
                let ingress = K8sIngress {
                    metadata: ObjectMeta {
                        name: Some(owner.name.clone()),
                        namespace: Some(owner.namespace.clone()),
                        annotations: Some(BTreeMap::from([
                            ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-production".to_string())
                        ])),
                        labels: Some(BTreeMap::from([
                            ("app.kubernetes.io/name".to_string(), owner.name.clone()),
                            ("app.kubernetes.io/instance".to_string(), owner.name.clone()),
                            ("app.kubernetes.io/managed-by".to_string(), "suffiks-ingress".to_string()),
                        ])),
                        owner_references,
                        ..Default::default()
                    },
                    spec: Some(K8sIngressSpec {
                        rules,
                        tls,
                        ingress_class_name: spec.ingress_class,
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                ingress_client.create( &PostParams::default(), &ingress).await
                    .map_err(|e| {
                        error!("failed to create ingress: {}", e);
                        Status::aborted(e.to_string())
                    })?;

            }
            Ok(Some(mut ingress)) => {
                info!("Updating ingress for {}", owner.name);
                ingress.metadata.owner_references = owner_references;
                match ingress.metadata.annotations {
                    Some(ref mut annotations) => {
                        annotations.insert("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-production".to_string());
                    }
                    None => {
                        ingress.metadata.annotations = Some(BTreeMap::from([
                            ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-production".to_string())
                        ]));
                    }
                }
                match ingress.metadata.labels {
                    Some(ref mut labels) => {
                        labels.insert("app.kubernetes.io/name".to_string(), owner.name.clone());
                        labels.insert("app.kubernetes.io/instance".to_string(), owner.name.clone());
                        labels.insert("app.kubernetes.io/managed-by".to_string(), "suffiks-ingress".to_string());
                    }
                    None => {
                        ingress.metadata.labels = Some(BTreeMap::from([
                            ("app.kubernetes.io/name".to_string(), owner.name.clone()),
                            ("app.kubernetes.io/instance".to_string(), owner.name.clone()),
                            ("app.kubernetes.io/managed-by".to_string(), "suffiks-ingress".to_string()),
                        ]))
                    }
                }
                match ingress.spec {
                    Some(mut ingress_spec) => {
                        ingress_spec.rules = rules;
                        ingress_spec.tls = tls;
                        ingress_spec.ingress_class_name = spec.ingress_class;
                        ingress.spec = Some(ingress_spec);
                    }
                    None => {
                        ingress.spec = Some(K8sIngressSpec {
                            rules,
                            tls,
                            ingress_class_name: spec.ingress_class,
                            ..Default::default()
                        });
                    }
                }

                ingress_client.replace(owner.name.as_str(), &PostParams::default(), &ingress).await
                    .map_err(|e| {
                        error!("failed to replace ingress: {}", e);
                        Status::aborted(e.to_string())
                    })?;
            }
            Err(e) => {
                error!("failed to get ingress: {}", e);
                return Err(Status::aborted(e.to_string()));
            }
        };

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
