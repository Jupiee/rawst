use crate::core::io::{merge_files, create_file};
use crate::core::task::DownloadTask;
use crate::core::errors::RawstErr;
use crate::core::config::Config;

use std::sync::Arc;

use reqwest::{Client, header::RANGE};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

pub struct Downloader {

    pub client: Client,
    pub config: Config,

    multi_bar: Arc<MultiProgress>

}

impl Downloader {

    pub fn new(client: Client, config: Config) -> Result<Self, RawstErr> {

        Ok(Downloader {

            client,
            config,
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

        let progressbar= self.multi_bar.add(ProgressBar::new(task.content_length()).with_message(task.filename.to_string()));

        progressbar.set_style(ProgressStyle::with_template("{msg} | {bytes}/{total_bytes} | [{wide_bar:.green/white}] | {eta} | [{decimal_bytes_per_sec}]")
        .unwrap()
        .progress_chars("=>_"));

        match self.config.threads {

            1 => {

                let response= self.client.get(task.url)
                    .send()
                    .await
                    .map_err(|_| RawstErr::Unreachable)?;

                if response.status().is_success() {

                    create_file(task.filename.to_string(), response, progressbar.clone(), task.downloaded, &self.config.download_path).await?;
    
                }

            },
            _ => {

                let chunks= task.into_chunks(self.config.threads as u64);
                
                // Creates a stream iter for downloading each chunk separately
                let download_tasks= stream::iter((0..self.config.threads).map(|i| {

                    let file_chunk= &chunks[i];
                    
                    let client= &self.client;
                    let progressbar= progressbar.clone();
                    let task= task.clone();

                    // Creates closure for each request and IO operation
                    // Each closure has separate IO operation
                    async move {
                        
                        let response= client.get(task.url)
                            .header(RANGE, format!("bytes={}-{}", file_chunk.x_offset, file_chunk.y_offset))
                            .send()
                            .await
                            .map_err(|e| RawstErr::HttpError(e))?;

                        if response.status().is_success() {

                            let temp_filepath= format!("{}-{}.tmp", task.filename.stem, i);

                            create_file(temp_filepath, response, progressbar, task.downloaded, &self.config.cache_path).await?;

                        }

                        Ok::<_, RawstErr>(())
                    
                    }

                }));

                download_tasks.buffer_unordered(self.config.threads).collect::<Vec<_>>().await;

                merge_files(&task.filename, &self.config).await?;

            }

        }
        
        progressbar.finish();

        Ok(())

    }

}