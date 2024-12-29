use std::path::PathBuf;
use std::sync::atomic::Ordering;

use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use iri_string::types::IriString;
use base64::{prelude::BASE64_STANDARD, Engine as Base64Engine};
use chrono::prelude::Local;

use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::http_handler::HttpHandler;
use crate::core::task::HttpTask;
use crate::core::utils::{extract_filename_from_header, extract_filename_from_url};
use crate::core::history::HistoryManager;
use crate::cli::args::DownloadArgs;
use crate::core::io::{get_cache_sizes, read_links};

pub struct Engine {
    config: Config,
    http_handler: HttpHandler,
    history_manager: HistoryManager,
    multi_bar: MultiProgress,
}

impl Engine {
    pub fn new(config: Config) -> Self {

        let history_manager= HistoryManager::new(config.history_file_path.clone());

        Engine {
            config,
            http_handler: HttpHandler::new(),
            history_manager,
            multi_bar: MultiProgress::new(),
        }
    }

    pub async fn process_url_download(mut self, args: DownloadArgs) -> Result<(), RawstErr> {
        let iri: IriString = args.iris.into_iter().next().ok_or(RawstErr::InvalidArgs)?;
    
        let save_as = args.output_file_path.into_iter().next();
        // override the default count in config
        self.config.threads = args.threads.into();

        let http_task = self.create_http_task(iri, (&save_as).into()).await?;
    
        let current_time = Local::now();
    
        let encoded_timestamp_as_id = BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());
    
        self.history_manager.add_record(&http_task, &self.config, encoded_timestamp_as_id.clone())?;
    
        self.http_download(http_task).await?;
    
        self.history_manager.update_record(encoded_timestamp_as_id)?;
    
        Ok(())
    }

    pub async fn process_list_download(mut self, args: DownloadArgs) -> Result<(), RawstErr> {
        self.config.threads = 1;
        
        // TODO: move url parsing outside of this function
        // TODO: this function will accept only vector of urls
        let file_path = args.input_file.ok_or(RawstErr::InvalidArgs)?;
    
        let link_string = read_links(&file_path).await?;
    
        let mut task_ids: Vec<String> = Vec::new();
        let mut http_tasks: Vec<HttpTask> = Vec::new();
    
        let url_list = link_string.split("\n").collect::<Vec<&str>>();
        for (index, line) in url_list.iter().enumerate() {
            let iri = line
                .trim()
                .parse::<IriString>()
                .map_err(|_| RawstErr::InvalidArgs)?;
    
            let http_task = self.create_http_task(iri, None).await?;
    
            let current_time = Local::now();
    
            // Adding index number to distinguish between each id of each task
            let encoded_timestamp_as_id =
                BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes()) + &index.to_string();
    
            self.history_manager.add_record(&http_task, &self.config, encoded_timestamp_as_id.clone())?;
    
            http_tasks.push(http_task);
    
            task_ids.push(encoded_timestamp_as_id);
        }
    
        self.list_http_download(http_tasks).await?;
    
        for id in task_ids.iter() {
            self.history_manager.update_record(id.to_string())?;
        }
    
        Ok(())
    }

    pub async fn process_resume_request(&mut self, id: String) -> Result<(), RawstErr> {
        log::trace!("Resuming download (id:{:?}, config:{:?})", id, self.config);
        let record = if id == "auto" {
            self.history_manager.get_recent_pending()?
        } else {
            self.history_manager.get_record(&id)?
        };
    
        match record {
            Some(data) => {
                // notice: I can also get total file size by getting content length through http_task object
                if data.status == "Pending" {
                    self.config.threads = data.threads_used;

                    let mut http_task = self
                        .create_http_task(data.iri, Some(&data.file_name))
                        .await?;
    
                    let cache_sizes =
                        get_cache_sizes(&data.file_name, data.threads_used, self.config.clone())?;
    
                    http_task.calculate_x_offsets(&cache_sizes);
    
                    http_task
                        .total_downloaded
                        .fetch_add(cache_sizes.iter().sum::<u64>(), Ordering::SeqCst);
    
                    self.http_download(http_task).await?;
    
                    self.history_manager.update_record(data.id)?
                } else {
                    println!("The file is already downloaded");
    
                    return Ok(());
                }
            }
            None => {
                println!("Record with id {:?} not found", id);
    
                return Ok(());
            }
        }
    
        Ok(())
    }

    pub async fn http_download(&self, task: HttpTask) -> Result<(), RawstErr> {
        log::trace!("Starting HTTP download (task:{task:?})");
        let file_name_str = task.filename.display().to_string();

        let progressbar = self
            .multi_bar
            .add(ProgressBar::new(task.content_length()).with_message(file_name_str));

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
        save_as: Option<&PathBuf>,
    ) -> Result<HttpTask, RawstErr> {
        log::trace!("Creating HTTP download task (iri:{iri:?}, save_as:{save_as:?})");
        let cached_headers = self.http_handler.cache_headers(&iri).await?;

        let mut filename = match extract_filename_from_header(&cached_headers) {
            Some(result) => result,
            None => extract_filename_from_url(&iri),
        };

        if let Some(save_as) = save_as {
            let extension = filename.extension().unwrap();
            let mut new_filename = PathBuf::from(save_as.file_name().unwrap());
            new_filename.add_extension(extension);
            filename = new_filename;
            assert!(filename.is_relative());
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