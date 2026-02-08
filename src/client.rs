use vsc::libvirt_service_client::LibvirtServiceClient;
use vsc::{ControllerDomainRequest, Instructions};

pub mod vsc {
    tonic::include_proto!("libvirt_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LibvirtServiceClient::connect("http://192.168.1.8:50052").await?;

    let request = tonic::Request::new(ControllerDomainRequest {
        name: String::from("freebsd-guest4"),
        instruction: Instructions::Start.into(),
    });

    client.controller_domain_service(request).await?;

    Ok(())
}
