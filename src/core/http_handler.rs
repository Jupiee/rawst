use crate::core::io::{merge_files, create_file};
use crate::core::task::DownloadTask;
use crate::core::errors::RawstErr;

use std::sync::{atomic::AtomicU64, Arc};

use reqwest::{Client, header::RANGE};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

pub struct Downloader {

    pub client: Client,
    pub connections: u64,

    multi_bar: Arc<MultiProgress>

}

impl Downloader {

    pub fn new(client: Client, threads: u64) -> Result<Self, RawstErr> {

        // 8 threads are maximum
        // more than 8 threads could cause rate limiting
        if !(1..9).contains(&threads) {

            return Err(RawstErr::InvalidThreadCount)
            
        }

        return Ok(Downloader {

            client,
            connections: threads,
            multi_bar: Arc::new(MultiProgress::new())

        })
        
    }

    pub async fn multi_download(&self, tasks: Vec<DownloadTask>) -> Result<(), RawstErr> {

        let download_tasks= stream::iter((0..tasks.len()).map(|i| {

            let threaded_task= tasks[i].clone();

            async move {

                self.download(threaded_task).await?;

                Ok::<_, RawstErr>(())

            }

        }));

        download_tasks.buffer_unordered(tasks.len()).collect::<Vec<_>>().await;

        Ok(())

    }

    pub async fn download(&self, task: DownloadTask) -> Result<(), RawstErr> {

        let progressbar= self.multi_bar.add(ProgressBar::new(task.content_length().await).with_message(task.filename.to_string()));

        progressbar.set_style(ProgressStyle::with_template("{msg} | {bytes}/{total_bytes} | [{wide_bar:.green/white}] | {eta} | [{decimal_bytes_per_sec}]")
        .unwrap()
        .progress_chars("=>_"));

        let downloaded= Arc::new(AtomicU64::new(0));

        match self.connections {

            1 => {

                let response= self.client.get(task.url)
                    .send()
                    .await
                    .map_err(|_| RawstErr::Unreachable)?;

                if response.status().is_success() {

                    create_file(task.filename.to_string(), response, progressbar.clone(), downloaded).await?;
    
                }

            },
            _ => {

                let chunks= task.into_chunks(self.connections).await;
                
                // Creates a stream iter for downloading each chunk separately
                let download_tasks= stream::iter((0..self.connections).map(|i| {

                    let i= i as usize;
                    
                    let file_chunk= &chunks[i];
                    
                    let url= &task.url;
                    let client= &self.client;
                    let file_name_without_ext= &task.filename.stem;
                    let progressbar= progressbar.clone();
                    let downloaded= downloaded.clone();

                    // Creates closure for each request and IO operation
                    // Each closure has separate IO operation
                    async move {
                        
                        let response= client.get(url)
                            .header(RANGE, format!("bytes={}-{}", file_chunk.x_offset, file_chunk.y_offset))
                            .send()
                            .await
                            .map_err(|e| RawstErr::HttpError(e))?;

                        if response.status().is_success() {

                            let temp_filepath= format!("{}-{}.tmp", file_name_without_ext, i);

                            create_file(temp_filepath, response, progressbar, downloaded).await?;

                        }

                        Ok::<_, RawstErr>(())
                    
                    }

                }));

                download_tasks.buffer_unordered(self.connections as usize).collect::<Vec<_>>().await;

                merge_files(&task.filename, self.connections).await?;

            }

        }
        
        progressbar.finish();

        return Ok(())

    }

}