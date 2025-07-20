use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream};
use iri_string::types::IriString;
use chrono::prelude::DateTime;

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64},
    Arc,
};

use crate::core::utils::hashmap_to_headermap;
use crate::core::task::{HttpTask, ChunkType, Chunk};
use crate::cli::args::ServerOptions;
use crate::core::errors::RawstErr;
use crate::core::config::Config;
use crate::core::engine::Engine;
use crate::core::history;
use crate::core::config::edit_config;

use crate::cli::args::rawstcli_proto::rawst_download_server::{RawstDownload, RawstDownloadServer};
use crate::cli::args::rawstcli_proto::{HttpTask as ProtoTask, ProgressData, http_task::ChunkType as ProtoChunkType};

#[derive(Default)]
pub struct RawstService {}

#[tonic::async_trait]
impl RawstDownload for RawstService {
    type DownloadStream = ReceiverStream<Result<ProgressData, Status>>;
    async fn download(
            &self,
            request: Request<ProtoTask>,
        ) -> Result<Response<Self::DownloadStream>, Status> {
            let data = request.into_inner();

            let config = match Config::load().await {
                Ok(config) => config,
                Err(_) => {
                    let config = Config::default();
                    config.initialise_files().await.map_err(|_| RawstErr::InitilisationError).unwrap();
                    config
                }
            };

            let chunk_type = match data.chunk_type {
                Some(ProtoChunkType::Chunk(chunk_data)) => {
                    Some(ChunkType::Single(Chunk {
                        x_offset: chunk_data.x_offset,
                        y_offset: chunk_data.y_offset,
                        downloaded: Arc::new(AtomicU64::new(chunk_data.downloaded))
                    }))

                },
                Some(ProtoChunkType::MultipleChunks(vec_chunks)) => {
                    let vec_chunks = vec_chunks.multiple_chunks;
                    let mut new_vec = vec![];

                    for chunk in vec_chunks {
                        new_vec.push(Chunk {
                            x_offset: chunk.x_offset,
                            y_offset: chunk.y_offset,
                            downloaded: Arc::new(AtomicU64::new(chunk.downloaded))
                        });
                    }

                    Some(ChunkType::Multiple(new_vec))

                },
                _ => None
            };

            if chunk_type.is_none() {

                Err(Status::invalid_argument("Number of Chunks can not be empty")).unwrap()

            }

            let cached_headers = hashmap_to_headermap(&data.headers);

            let http_task = HttpTask {
                iri: IriString::try_from(data.iri).map_err(|e| e.to_string()).unwrap(),
                filename: PathBuf::from(data.file_name),
                total_downloaded: Arc::new(AtomicU64::new(data.total_downloaded)),
                chunk_data: chunk_type.unwrap(),
                additional_headers: data.additional_headers,
                timestamp: DateTime::parse_from_rfc3339(data.timestamp.as_str()).unwrap().into(),
                headers: cached_headers
            };

            let (tx, rx) = mpsc::channel(http_task.content_length() as usize);

            let engine = Engine::new(config);

            tokio::spawn(async move {
                engine.http_download(http_task, tx).await.map_err(|_| RawstErr::Unreachable)

            });

            Ok(Response::new(ReceiverStream::new(rx)))
            
        }
}

pub async fn start_server(opts: ServerOptions) -> Result<(), RawstErr> {
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