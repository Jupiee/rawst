use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::cli::args::DownloadArgs;
use crate::core::engine::download;
use crate::core::task::{TaskType, ChunkType};
use crate::core::utils::headermap_to_hashmap;
use std::sync::atomic::Ordering;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::cli::args::rawstcli_proto::rawst_download_client::RawstDownloadClient;
use crate::cli::args::rawstcli_proto::{HttpTask as ProtoTask, http_task::ChunkType as ProtoChunkType, Chunk as ProtoChunk, VecOfChunks};

pub async fn run_client(args: DownloadArgs, config: Config, address: String) -> Result<(), RawstErr> {
    let mut client = RawstDownloadClient::connect(address).await.map_err(|_| RawstErr::Unreachable)?;

    let task_type = download(args, config).await?;

    let multi_bar = MultiProgress::new();
    let progressbar = match &task_type {
        TaskType::Single(task) => {

            let file_name_str = task.filename.display().to_string();

            let progressbar = multi_bar
                .add(ProgressBar::new(task.content_length()).with_message(file_name_str));

            progressbar.set_style(ProgressStyle::with_template("{msg} | {bytes}/{total_bytes} | [{wide_bar:.green/white}] | {eta} | [{decimal_bytes_per_sec}]")
            .unwrap()
            .progress_chars("=>_"));

            progressbar.set_position(task.total_downloaded.load(Ordering::SeqCst));
            progressbar.reset_eta();

            Some(progressbar)

        },
        TaskType::Multiple(_) => None

    };

    let prototask = match task_type {
        TaskType::Single(http_task) => {

            let chunk_type = match http_task.chunk_data {
                ChunkType::Single(chunk) => {
                    Some(ProtoChunkType::Chunk(
                        ProtoChunk {
                            x_offset: chunk.x_offset,
                            y_offset: chunk.y_offset,
                            downloaded: chunk.downloaded.load(Ordering::SeqCst)
                        }
                    ))
                },
                ChunkType::Multiple(vec_chunks) => {

                    let mut proto_vec_chunks = vec![];

                    for chunk in vec_chunks {
                        let proto_chunk = ProtoChunk {
                            x_offset: chunk.x_offset,
                            y_offset: chunk.y_offset,
                            downloaded: chunk.downloaded.load(Ordering::SeqCst)
                        };
                        proto_vec_chunks.push(proto_chunk);
                    }

                    Some(ProtoChunkType::MultipleChunks(VecOfChunks{
                        multiple_chunks: proto_vec_chunks
                    }))

                },
                _ => None

            };

            let cached_headers = headermap_to_hashmap(&http_task.headers);

            Some(ProtoTask {
                iri: http_task.iri.to_string(),
                file_name: http_task.filename.into_os_string().into_string().unwrap(),
                total_downloaded: http_task.total_downloaded.load(Ordering::SeqCst),
                additional_headers: http_task.additional_headers,
                timestamp: http_task.timestamp.to_rfc3339(),
                chunk_type,
                headers: cached_headers
            })
        },
        TaskType::Multiple(_) => None
    };

    match prototask {
        Some(http_task) =>{

            let progressbar = progressbar.unwrap();
            
            let request = tonic::Request::new(http_task);
        
            let mut stream = client.download(request).await.map_err(|_| RawstErr::BadRequest)?.into_inner();

            while let Some(progressdata) = stream.message().await.map_err(|_| RawstErr::BadRequest).unwrap() {
                //println!("RESPONSE= {:?}", progressdata);
                progressbar.inc(progressdata.chunk_size);

            }

            progressbar.finish();
        
            Ok(())
        
        },
        None => Err(RawstErr::InvalidArgs)

    }

}