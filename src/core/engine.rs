use std::path::PathBuf;
use std::collections::HashMap;
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
use crate::cli::args::ResumeArgs;
use crate::core::io::{get_cache_sizes, read_links};

pub async fn download(args: DownloadArgs, config: Config) -> Result<(), RawstErr> {
    // TODO: Fuse url_download and list_download
    // TODO: Support downloading many elements from each source
    log::trace!("Downloading files ({args:?}, {config:?})");
    let engine= Engine::new(config);

    if args.input_file.is_some() {
        let file_path = args.input_file.ok_or(RawstErr::InvalidArgs)?;
        engine.process_list_download(file_path).await

    } else {

        let iri: IriString = args.iris.into_iter().next().ok_or(RawstErr::InvalidArgs)?;
        let save_as = args.output_file_path.into_iter().next();
        let threads = args.threads.into();

        engine.process_url_download(iri, save_as, threads).await
    }
}

pub async fn resume_download(args: ResumeArgs, config: Config) -> Result<(),RawstErr> {
    let ids= args.download_ids;
    let mut engine= Engine::new(config);

    if ids.len() > 1 {
        for id in ids {
            engine.process_resume_request(id).await?

        }

        Ok(())
    }
    else {
        let id= ids.iter().next().unwrap().to_string();
        engine.process_resume_request(id).await

    }

}

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

    pub async fn process_url_download(mut self, iri: IriString, save_as: Option<PathBuf>, threads: usize) -> Result<(), RawstErr> {
        // override the default count in config
        self.config.threads = threads;

        let http_task = self.create_http_task(iri, (&save_as).into()).await?;
    
        let current_time = Local::now();
    
        let encoded_timestamp_as_id = BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());
    
        self.history_manager.add_record(&http_task, &self.config, encoded_timestamp_as_id.clone())?;
    
        self.http_download(http_task).await?;
    
        self.history_manager.update_record(encoded_timestamp_as_id)?;
    
        Ok(())
    }

    pub async fn process_list_download(mut self, file_path: PathBuf) -> Result<(), RawstErr> {
        self.config.threads = 1;
        
        let link_string = read_links(&file_path).await?;
    
        let mut tasks: HashMap<String, HttpTask> = HashMap::new();
    
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

            tasks.insert(encoded_timestamp_as_id, http_task);
    
        }

        let val: Vec<HttpTask> = tasks.clone().into_values().collect();
    
        self.list_http_download(val).await?;
    
        for id in tasks.keys() {
            self.history_manager.update_record(id.to_owned())?;
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
                    self.config.download_dir = data.file_location;

                    let file_name = PathBuf::from(&data.file_name.file_stem().unwrap());

                    let mut http_task = self
                        .create_http_task(data.iri, Some(&file_name))
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

        progressbar.set_position(task.total_downloaded.load(Ordering::SeqCst));
        progressbar.reset_eta();

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
            let output_path = save_as.parent().unwrap();
            if output_path.exists() {
                self.config.download_dir = output_path.to_path_buf();

            }
            
            let mut new_filename = PathBuf::from(save_as.file_name().unwrap());
            new_filename.add_extension(extension);
            filename = new_filename;
            assert!(filename.is_relative());
        }

        let mut task = HttpTask::new(iri, filename, cached_headers);

        // checks if the server allows to receive byte ranges for concurrent download
        // otherwise uses single thread
        if self.config.threads > 1 && !task.allows_partial_content() {
            println!("Warning!: Server doesn't allow partial content, sequentially downloading..");
            self.config.threads = 1

        }

        task.calculate_chunks(self.config.threads as u64);

        Ok(task)
    }
}