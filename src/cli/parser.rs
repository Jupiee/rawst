use crate::core::config::Config;
use crate::core::task::DownloadTask;
use crate::core::http_handler::Downloader;
use crate::core::io::{read_links, build_config, load_config, config_exist};
use crate::core::errors::RawstErr;
use crate::core::utils::{cache_headers, extract_filename_from_header, extract_filename_from_url};

use clap::{value_parser, crate_authors, crate_name, crate_description, crate_version};
use clap::{Arg, ArgMatches, Command};
use reqwest::Client;

fn build_command() -> ArgMatches {

    return Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::new("Url")
            .short('u')
            .long("url")
            .value_parser(value_parser!(String))
            .help("Url to download")
        )
        .arg(
            Arg::new("InputFile")
            .short('f')
            .long("file")
            .value_parser(value_parser!(String))
            .help("Filepath to the file with links")
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
            .default_value("1")
            .help("Maximum number of concurrent downloads")
        )
        .get_matches()

}

async fn url_download(args: ArgMatches, mut config: Config) -> Result<(), RawstErr> {

    let threads= args.get_one::<usize>("Threads").unwrap().to_owned();

    // 8 threads are maximum
    // more than 8 threads could cause rate limiting
    if !(1..9).contains(&threads) {

        return Err(RawstErr::InvalidThreadCount)
        
    }

    config.threads= threads;
    
    let client= Client::new();

    let url= args.get_one::<String>("Url").unwrap();

    let save_as= args.get_one::<String>("Saveas");
    
    let cached_headers= cache_headers(&client, url).await?;
    
    let mut filename= match extract_filename_from_header(&cached_headers) {

        Some(result) => result,
        None => extract_filename_from_url(url)

    };
    
    if save_as.is_some() {

        filename.stem= save_as.unwrap().to_owned();
        
    }
    
    let task= DownloadTask::new(
        url.to_owned(),
        filename,
        cached_headers
    );

    let downloader= Downloader::new(client, config)?;

    downloader.download(task).await?;

    Ok(())

}

async fn list_download(args: ArgMatches, config: Config) -> Result<(), RawstErr> {

    let client= Client::new();

    let file_path= args.get_one::<String>("InputFile").unwrap();

    let link_string= read_links(file_path).await?;

    let url_list= link_string.split("\n").collect::<Vec<&str>>();

    let mut download_tasks: Vec<DownloadTask> = Vec::new();

    for url in url_list {

        let url= url.to_string();

        let cached_headers= cache_headers(&client, &url).await?;

        let filename= match extract_filename_from_header(&cached_headers) {

            Some(result) => result,
            None => extract_filename_from_url(&url)

        };

        let task= DownloadTask::new(
            url,
            filename,
            cached_headers
        );

        download_tasks.push(task);

    }

    let downloader= Downloader::new(client, config)?;

    downloader.multi_download(download_tasks).await?;

    Ok(())

}

pub async fn init() -> Result<(), RawstErr> {

    let args= build_command();
    let config= match config_exist() {

        true => load_config().await?,
        false => build_config().await?

    };

    if args.contains_id("Url") {

        url_download(args, config).await?;

        return Ok(())

    }

    else if args.contains_id("InputFile") {

        list_download(args, config).await?;

        return Ok(())

    }

    else {

        return Err(RawstErr::InvalidArgs);

    }

}