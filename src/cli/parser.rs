use crate::core::config::Config;
use crate::core::task::HttpTask;
use crate::core::errors::RawstErr;
use crate::core::engine::Engine;
use crate::core::history::HistoryManager;
use crate::core::io::{read_links, config_exist, get_cache_sizes};

use std::sync::atomic::Ordering;

use base64::{Engine as Base64Engine, prelude::BASE64_STANDARD};
use chrono::prelude::Local;

use clap::{value_parser, crate_authors, crate_name, crate_description, crate_version};
use clap::{Arg, ArgMatches, Command};

fn build_command() -> ArgMatches {

    return Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::new("Url")
            .short('d')
            .long("download")
            .value_parser(value_parser!(String))
            .help("Url to download")
        )
        .arg(
            Arg::new("Resume")
            .long("resume")
            .value_parser(value_parser!(String))
            .help("Resume download of the given record ID")
        )
        .arg(
            Arg::new("InputFile")
            .short('f')
            .long("file")
            .value_parser(value_parser!(String))
            .help("Filepath to the file with links")
        )
        .arg(
            Arg::new("History")
            .long("history")
            .action(clap::ArgAction::SetTrue)
            .help("Display download history")
        )
        .arg(
            Arg::new("Saveas")
            .short('s')
            .long("save-as")
            .help("Save file as custom name")
        )
        .arg(
            Arg::new("Threads")
            .short('m')
            .long("max-threads")
            .value_parser(value_parser!(usize))
            .help("Maximum number of concurrent downloads")
        )
        .get_matches()

}

async fn url_download(args: ArgMatches, mut config: Config) -> Result<(), RawstErr> {

    let url= args.get_one::<String>("Url").unwrap().to_string();

    let save_as= args.get_one::<String>("Saveas");

    let threads= args.get_one::<usize>("Threads");

    // 8 threads are maximum
    // more than 8 threads could cause rate limiting
    let threads_limit= 1..9;

    // if thread argument has a value and is in valid range then
    // override the default count in config
    if threads.is_some_and(|threads| threads_limit.contains(threads)) {

        config.threads= threads.unwrap().to_owned();

    }

    else if threads.is_some_and(|threads| !threads_limit.contains(threads)) {

        return Err(RawstErr::InvalidThreadCount)

    }

    let mut engine= Engine::new(config.clone());

    let http_task= engine.create_http_task(url, save_as).await?;

    let history_manager= HistoryManager::new(config.config_path.clone());

    let current_time= Local::now();

    let encoded_timestamp_as_id= BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());

    history_manager.add_record(&http_task, &config, encoded_timestamp_as_id.clone())?;

    engine.http_download(http_task).await?;

    history_manager.update_record(encoded_timestamp_as_id)?;

    Ok(())

}

async fn list_download(args: ArgMatches, mut config: Config) -> Result<(), RawstErr> {

    config.threads= 1;

    let mut engine= Engine::new(config.clone());

    let history_manager= HistoryManager::new(config.config_path.clone());

    let file_path= args.get_one::<String>("InputFile").unwrap();

    let link_string= read_links(file_path).await?;

    let url_list= link_string.split("\n").collect::<Vec<&str>>();

    let mut http_tasks: Vec<HttpTask> = Vec::new();

    let mut task_ids: Vec<String> = Vec::new();

    for (index, url) in url_list.iter().enumerate() {

        let url= url.trim().to_string();

        let http_task= engine.create_http_task(url, None).await?;

        let current_time= Local::now();

        // Adding index number to distinguish between each id of each task
        let encoded_timestamp_as_id= BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes()) + &index.to_string();

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

async fn display_history(config: Config) -> Result<(), RawstErr> {

    let history_manager= HistoryManager::new(config.config_path);

    history_manager.get_history()?;

    Ok(())

}

async fn resume_download(args: ArgMatches, mut config: Config) -> Result<(), RawstErr> {

    let id= args.get_one::<String>("Resume").unwrap().to_owned();

    let history_manager= HistoryManager::new(config.config_path.clone());

    let record= history_manager.get_record(&id)?;

    match record {

        Some(data) => {

            // notice: I can also get total file size by getting content length through http_task object
            let (url, threads, file_name, status)= data;

            if status == "Pending" {

                config.threads= threads;
            
                let (file_stem, _)= file_name.rsplit_once(".").unwrap();
                
                let mut engine= Engine::new(config.clone());
                
                let mut http_task= engine.create_http_task(url, Some(&file_stem.trim().to_owned())).await?;
                
                let cache_sizes= get_cache_sizes(file_name, threads, config).unwrap();

                http_task.calculate_x_offsets(&cache_sizes);
                
                http_task.total_downloaded.fetch_add(cache_sizes.iter().sum::<u64>(), Ordering::SeqCst);

                engine.http_download(http_task).await?

            }

            else {

                println!("The file is already downloaded");

                return Ok(());

            }

        },
        None => {

            println!("Record with id {:?} not found", id);

            return Ok(())

        }

    }

    history_manager.update_record(id)?;

    return Ok(());

}

pub async fn init() -> Result<(), RawstErr> {

    let args= build_command();
    let config= match config_exist() {

        true => Config::load().await?,
        false => Config::build().await?

    };

    if args.contains_id("Url") {

        url_download(args, config).await?;

        return Ok(())

    }

    else if args.contains_id("InputFile") {

        list_download(args, config).await?;

        return Ok(())

    }
    
    else if args.contains_id("Resume") {

        resume_download(args, config).await?;

        return Ok(());

    }

    else if args.contains_id("History") {

        display_history(config).await?;

        return Ok(());

    }


    else {

        return Err(RawstErr::InvalidArgs);

    }

}