use kube::Client;
use tonic::transport::Server;

use ingress::IngressHandler;

use crate::protogen::extension::extension_server::ExtensionServer;

mod protogen;
mod ingress;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "0.0.0.0:8080".parse().unwrap();
    // create kube client
    let client = Client::try_default().await?;
    // creating a service
    let ingress = IngressHandler::new(client);
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(ExtensionServer::new(ingress))
        .serve(addr)
        .await?;
    Ok(())
}
