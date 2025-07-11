use tonic::{transport::Server, Request, Response, Status};

use crate::cli::args::{ServerOptions, Arguments};
use crate::core::errors::RawstErr;

use crate::cli::args::rawstcli_proto::rawst_download_server::{RawstDownload, RawstDownloadServer};
use crate::cli::args::rawstcli_proto::{Request as ProtoRequest, Response as ProtoResponse, DownloadArgs as ProtoDownloadArgs};

#[derive(Default)]
pub struct RawstService {}

#[tonic::async_trait]
impl RawstDownload for RawstService {
    async fn send(
            &self,
            request: Request<ProtoRequest>,
        ) -> Result<Response<ProtoResponse>, Status> {
            let arguments = request.into_inner();

            let converted = Arguments::from_primitive_types(arguments);

            //println!("{:?}", arguments);
            let display = format!("{:?}", converted);

            Ok(Response::new(
                ProtoResponse {
                    output: display
                }
            ))

        }
}

pub async fn start_server(opts: &ServerOptions) -> Result<(), RawstErr> {
    let address = opts.server_address.parse().unwrap();
    let service = RawstService::default();

    println!("Server running at {}", address);

    Server::builder()
        .add_service(RawstDownloadServer::new(service))
        .serve(address)
        .await
        .map_err(|_| RawstErr::Unreachable)?;

    Ok(())

}