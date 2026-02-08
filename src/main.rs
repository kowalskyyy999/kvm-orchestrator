pub mod vsc {
    tonic::include_proto!("libvirt_service");
}

use tonic::{
    transport::{Channel, Server},
    Request, Response, Status,
};
use vsc::libvirt_service_server::{LibvirtService, LibvirtServiceServer};
use vsc::{
    ControllerDomainRequest, CreateDomainRequest, Instructions, MonitoringDomainRequest,
    UniversalResponse,
};

use libvirtrs::connection::Connection;

#[derive(Clone)]
struct KVMManager {
    conn: Connection,
}

mod client;

#[tonic::async_trait]
impl LibvirtService for KVMManager {
    async fn create_domain_service(
        &self,
        request: Request<CreateDomainRequest>,
    ) -> Result<Response<UniversalResponse>, Status> {
        let req = request.into_inner();
        let xml_request = req.xml;

        let conn = &self.conn;

        let domain = &conn.define_domain(xml_request.as_str()).unwrap();

        Ok(Response::new(UniversalResponse {
            message: "Success".to_string(),
        }))
    }

    async fn controller_domain_service(
        &self,
        request: Request<ControllerDomainRequest>,
    ) -> Result<Response<UniversalResponse>, Status> {
        let req = request.into_inner();
        let ins = Instructions::from_i32(req.instruction).unwrap_or(Instructions::Start);

        let conn = &self.conn;
        let domains = &conn.list_all_domains().unwrap();
        let req_domain_name = req.name;

        for domain in domains {
            let domain_name = domain.get_name();
            if domain_name.eq(&req_domain_name) {
                let _ = match ins {
                    Instructions::Start => domain.start(),
                    Instructions::Shutdown => domain.shutdown(),
                    Instructions::Reboot => domain.reboot(),
                };
                break;
            };
        }

        Ok(Response::new(UniversalResponse {
            message: "Success".to_string(),
        }))
    }

    async fn monitoring_domain_service(
        &self,
        request: Request<MonitoringDomainRequest>,
    ) -> Result<Response<UniversalResponse>, Status> {
        Ok(Response::new(UniversalResponse {
            message: "Success".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    if let Ok(_) = std::fs::File::open(".env") {
        dotenv::dotenv().ok();
    };

    let addr = "[::]:50052".parse()?;

    let conn = Connection::open("qemu:///system").unwrap();

    let kvm_manager = KVMManager { conn };

    tracing::info!(message = "auth service listening on ", %addr);
    Server::builder()
        .trace_fn(|_| tracing::info_span!("Libvirt Service"))
        .add_service(LibvirtServiceServer::new(kvm_manager))
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}
