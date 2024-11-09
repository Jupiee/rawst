use crate::cli::args::DownloadArgs;
use crate::cli::args::ResumeArgs;
use crate::core::config::Config;
use crate::core::engine::Engine;
use crate::core::errors::RawstErr;
use crate::core::history::HistoryManager;
use crate::core::io::{get_cache_sizes, read_links};
use crate::core::task::HttpTask;

use std::sync::atomic::Ordering;

use base64::{prelude::BASE64_STANDARD, Engine as Base64Engine};
use chrono::prelude::Local;

pub async fn download(args: DownloadArgs, config: Config) -> Result<(), RawstErr> {
    // TODO: Fuse url_download and list_download
    // TODO: Support downloading many elements from each source
    if args.input_file.is_some() {
        list_download(args, config).await
    } else {
        url_download(args, config).await
    }
}

pub async fn url_download(args: DownloadArgs, mut config: Config) -> Result<(), RawstErr> {
    let url = args.files.into_iter().next().ok_or(RawstErr::InvalidArgs)?;
    let save_as = args.output_file_path.into_iter().next();
    // override the default count in config
    config.threads = args.threads.into();

    let mut engine = Engine::new(config.clone());

    let http_task = engine.create_http_task(url, (&save_as).into()).await?;

    let history_manager = HistoryManager::new(config.config_path.clone());

    let current_time = Local::now();

    let encoded_timestamp_as_id = BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());

    history_manager.add_record(&http_task, &config, encoded_timestamp_as_id.clone())?;

    engine.http_download(http_task).await?;

    history_manager.update_record(encoded_timestamp_as_id)?;

    Ok(())
}

pub async fn list_download(args: DownloadArgs, mut config: Config) -> Result<(), RawstErr> {
    config.threads = 1;

    let mut engine = Engine::new(config.clone());

    let history_manager = HistoryManager::new(config.config_path.clone());

    let file_path = args.input_file.ok_or(RawstErr::InvalidArgs)?;

    let link_string = read_links(&file_path).await?;

    let mut task_ids: Vec<String> = Vec::new();
    let mut http_tasks: Vec<HttpTask> = Vec::new();

    let url_list = link_string.split("\n").collect::<Vec<&str>>();
    for (index, url) in url_list.iter().enumerate() {
        let url = url.trim().to_string();

        let http_task = engine.create_http_task(url, None).await?;

        let current_time = Local::now();

        // Adding index number to distinguish between each id of each task
        let encoded_timestamp_as_id =
            BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes()) + &index.to_string();

        history_manager.add_record(&http_task, &config, encoded_timestamp_as_id.clone())?;

        http_tasks.push(http_task);

        task_ids.push(encoded_timestamp_as_id);
    }

    engine.list_http_download(http_tasks).await?;

    for id in task_ids.iter() {
        history_manager.update_record(id.clone())?;
    }

    Ok(())
}

pub async fn resume_download(args: ResumeArgs, mut config: Config) -> Result<(), RawstErr> {
    let id = args
        .download_ids
        .into_iter()
        .next()
        .ok_or(RawstErr::InvalidArgs)?;

    let history_manager = HistoryManager::new(config.config_path.clone());

    let record = if id == "auto" {
        history_manager.get_recent_pending()?
    } else {
        history_manager.get_record(&id)?
    };

    match record {
        Some(data) => {
            // notice: I can also get total file size by getting content length through http_task object
            if data.status == "Pending" {
                config.threads = data.threads_used;

                let (file_stem, _) = data.file_name.rsplit_once(".").unwrap();

                let mut engine = Engine::new(config.clone());

                let mut http_task = engine
                    .create_http_task(data.url, Some(&file_stem.trim().to_owned()))
                    .await?;

                let cache_sizes =
                    get_cache_sizes(data.file_name, data.threads_used, config).unwrap();

                http_task.calculate_x_offsets(&cache_sizes);

                http_task
                    .total_downloaded
                    .fetch_add(cache_sizes.iter().sum::<u64>(), Ordering::SeqCst);

                engine.http_download(http_task).await?;

                history_manager.update_record(data.id)?
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
