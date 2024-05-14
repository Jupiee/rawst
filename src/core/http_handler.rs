use crate::core::io::{merge_files, create_file};
use crate::core::task::HttpTask;
use crate::core::errors::RawstErr;
use crate::core::config::Config;

use reqwest::{Client, header::RANGE};
use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;

#[derive(Clone)]
pub struct HttpHandler {

    pub client: Client,

}

impl HttpHandler {

    pub fn new(client: Client) -> Self {

        HttpHandler {

            client

        }
        
    }

    pub async fn download_test(&self, task: HttpTask, progressbar: ProgressBar, config: &Config) -> Result<(), RawstErr> {

        match config.threads {

            1 => {

                let response= self.client.get(&task.url)
                    .send()
                    .await
                    .map_err(|_| RawstErr::Unreachable)?;

                if response.status().is_success() {

                    create_file(task.filename.to_string(), None, task, response, progressbar.clone(), &config.download_path).await?;
    
                }

            },
            _ => {
                
                // Creates a stream iter for downloading each chunk separately
                let download_tasks= stream::iter((0..config.threads).map(|i| {

                    let client= &self.client;
                    let progressbar= progressbar.clone();
                    let task= task.clone();

                    // Creates closure for each request and IO operation
                    // Each closure has separate IO operation
                    async move {
                    
                        let response= client.get(&task.url)
                            .header(RANGE, format!("bytes={}-{}", &task.chunks[i].x_offset, &task.chunks[i].y_offset))
                            .send()
                            .await
                            .map_err(|e| RawstErr::HttpError(e))?;

                        if response.status().is_success() {

                            let temp_filepath= format!("{}-{}.tmp", task.filename.stem, i);

                            create_file(temp_filepath, Some(i), task, response, progressbar, &config.cache_path).await?;

                        }

                        Ok::<_, RawstErr>(())
                    
                    }

                }));

                download_tasks.buffer_unordered(config.threads).collect::<Vec<_>>().await;

                merge_files(&task.filename, config).await?;

            }

        }
        
        progressbar.finish();

        Ok(())

    }

}