use tonic::{transport::Server, Request, Response, Status};

use crate::cli::args::{ServerOptions, Arguments, Command};
use crate::core::errors::RawstErr;
use crate::core::config::Config;
use crate::core::engine::{download, resume_download};
use crate::core::history;
use crate::core::config::edit_config;

use crate::cli::args::rawstcli_proto::rawst_download_server::{RawstDownload, RawstDownloadServer};
use crate::cli::args::rawstcli_proto::{Request as ProtoRequest, Response as ProtoResponse};

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

            let config = match Config::load().await {
                Ok(config) => config,
                Err(_) => {
                    let config = Config::default();
                    config.initialise_files().await.map_err(|_| RawstErr::InitilisationError).unwrap();
                    config
                }
            };

            if let Some(cmd) = converted.command {
                match cmd {
                    Command::Download(args) => download(args, config).await.map_err(|_| RawstErr::InitilisationError).unwrap(),
                    Command::Resume(args) => resume_download(args, config).await.map_err(|_| RawstErr::InitilisationError).unwrap(),
                    Command::History(args) => history::check_history_args(args, config).await.map_err(|_| RawstErr::InitilisationError).unwrap(),
                    Command::Config => edit_config(config).await.unwrap(),
                    _ => ()
                }
            }

            let display = format!("Downloaded");

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