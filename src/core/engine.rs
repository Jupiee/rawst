use crate::core::config::Config;
use crate::core::task::HttpTask;
use crate::core::http_handler::HttpHandler;
use crate::core::history::HistoryManager;
use crate::core::errors::RawstErr;
use crate::core::utils::{extract_filename_from_header, extract_filename_from_url, cache_headers};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use futures::stream::{self, StreamExt};
use reqwest::Client;

pub struct Engine {

    config: Config,
    client: Client,
    http_handler: HttpHandler,
    history_manager: HistoryManager,
    multi_bar: MultiProgress

}

impl Engine {

    pub fn new(config: Config) -> Self {

        let client= Client::new();
        let http_handler= HttpHandler::new(client.clone());
        let history_manager= HistoryManager::new(config.config_path.clone());

        Engine {

            config,
            client,
            http_handler,
            history_manager,
            multi_bar: MultiProgress::new()

        }

    }

    pub async fn http_download(&self, task: HttpTask) -> Result<(), RawstErr> {

        let progressbar= self.multi_bar.add(ProgressBar::new(task.content_length()).with_message(task.filename.to_string()));
        
        progressbar.set_style(ProgressStyle::with_template("{msg} | {bytes}/{total_bytes} | [{wide_bar:.green/white}] | {eta} | [{decimal_bytes_per_sec}]")
        .unwrap()
        .progress_chars("=>_"));
    
        match self.config.threads {

            1 => self.http_handler.sequential_download(&task, &progressbar, &self.config).await?,
            _ => self.http_handler.concurrent_download(&task, &progressbar, &self.config).await?

        }

        progressbar.finish();

        self.history_manager.add_record(&task, &self.config)?;

        Ok(())

    }

    pub async fn list_http_download(&self, tasks: Vec<HttpTask>) -> Result<(), RawstErr> {

        let http_download_tasks= stream::iter((0..tasks.len()).map(|i| {

            let threaded_task= tasks[i].clone();
    
            async move {
    
                self.http_download(threaded_task).await?;
    
                Ok::<_, RawstErr>(())
    
            }
    
        }));
    
        http_download_tasks.buffer_unordered(tasks.len()).collect::<Vec<_>>().await;
    
        Ok(())

    }

    pub async fn create_http_task(&mut self, url: String, save_as: Option<&String>, threads: Option<&usize>) -> Result<HttpTask, RawstErr> {

        // overrides the default count in config
        if threads.is_some() {

            self.config.threads= threads.unwrap().to_owned()

        }

        // 8 threads are maximum
        // more than 8 threads could cause rate limiting
        if !(1..9).contains(&self.config.threads) {

            return Err(RawstErr::InvalidThreadCount)

        }

        let cached_headers= cache_headers(&self.client, &url).await?;

        let mut filename= match extract_filename_from_header(&cached_headers) {

            Some(result) => result,
            None => extract_filename_from_url(&url)
    
        };

        if save_as.is_some() {

            filename.stem= save_as.unwrap().to_owned();
            
        }

        let mut task= HttpTask::new(url, filename, cached_headers, self.config.threads);

        // checks if the server allows to receive byte ranges for concurrent download
        // otherwise uses single thread
        if self.config.threads > 1 && !task.allows_partial_content() {

            println!("Warning!: Server doesn't allow partial content, sequentially downloading..");
            self.config.threads= 1
    
        }

        if self.config.threads > 1 {

            task.into_chunks(self.config.threads as u64)

        }

        return Ok(task)

    }

}