use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use iri_string::types::IriString;

use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::http_handler::HttpHandler;
use crate::core::task::HttpTask;
use crate::core::utils::{extract_filename_from_header, extract_filename_from_url};

pub struct Engine {
    config: Config,
    http_handler: HttpHandler,
    multi_bar: MultiProgress,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Engine {
            config,
            http_handler: HttpHandler::new(),
            multi_bar: MultiProgress::new(),
        }
    }

    pub async fn http_download(&self, task: HttpTask) -> Result<(), RawstErr> {
        let progressbar = self
            .multi_bar
            .add(ProgressBar::new(task.content_length()).with_message(task.filename.to_string()));

        progressbar.set_style(ProgressStyle::with_template("{msg} | {bytes}/{total_bytes} | [{wide_bar:.green/white}] | {eta} | [{decimal_bytes_per_sec}]")
        .unwrap()
        .progress_chars("=>_"));

        match self.config.threads {
            1 => {
                self.http_handler
                    .sequential_download(&task, &progressbar, &self.config)
                    .await?
            }
            _ => {
                self.http_handler
                    .concurrent_download(&task, &progressbar, &self.config)
                    .await?
            }
        }

        progressbar.finish();

        Ok(())
    }

    pub async fn list_http_download(&self, tasks: Vec<HttpTask>) -> Result<(), RawstErr> {
        let http_download_tasks = stream::iter((0..tasks.len()).map(|i| {
            let threaded_task = tasks[i].clone();

            async move {
                self.http_download(threaded_task).await?;

                Ok::<_, RawstErr>(())
            }
        }));

        http_download_tasks
            .buffer_unordered(tasks.len())
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    pub async fn create_http_task(
        &mut self,
        iri: IriString,
        save_as: Option<&String>,
    ) -> Result<HttpTask, RawstErr> {
        let cached_headers = self.http_handler.cache_headers(&iri).await?;

        let mut filename = match extract_filename_from_header(&cached_headers) {
            Some(result) => result,
            None => extract_filename_from_url(&iri),
        };

        if save_as.is_some() {
            filename.stem = save_as.unwrap().to_owned();
        }

        let mut task = HttpTask::new(iri, filename, cached_headers, self.config.threads);

        // checks if the server allows to receive byte ranges for concurrent download
        // otherwise uses single thread
        if self.config.threads > 1 && !task.allows_partial_content() {
            println!("Warning!: Server doesn't allow partial content, sequentially downloading..");
            self.config.threads = 1

        // This here is for building only single chunk for single thread downloads
        // if the server allows for partial content. Useful for resuming downloads
        // with one thread.
        } else if self.config.threads == 1 && task.allows_partial_content() {
            task.create_single_chunk();
        }

        task.calculate_chunks(self.config.threads as u64);

        Ok(task)
    }
}
