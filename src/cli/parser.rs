use crate::core::config::Config;
use crate::core::task::HttpTask;
use crate::core::io::{read_links, build_config, load_config, config_exist};
use crate::core::errors::RawstErr;
use crate::core::engine::Engine;

use clap::{value_parser, crate_authors, crate_name, crate_description, crate_version};
use clap::{Arg, ArgMatches, Command};
use futures::stream::{self, StreamExt};

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
            .help("Maximum number of concurrent downloads")
        )
        .get_matches()

}

async fn url_download(args: ArgMatches, config: Config) -> Result<(), RawstErr> {

    let url= args.get_one::<String>("Url").unwrap().to_string();

    let save_as= args.get_one::<String>("Saveas");

    let threads= args.get_one::<usize>("Threads");

    let mut engine= Engine::new(config);

    let http_task= engine.create_http_task(url, save_as, threads).await?;

    engine.http_download(http_task).await?;

    Ok(())

}

async fn list_download(args: ArgMatches, mut config: Config) -> Result<(), RawstErr> {

    config.threads= 1;

    let mut engine= Engine::new(config);

    let file_path= args.get_one::<String>("InputFile").unwrap();

    let link_string= read_links(file_path).await?;

    let url_list= link_string.split("\n").collect::<Vec<&str>>();

    let mut http_tasks: Vec<HttpTask> = Vec::new();

    for url in url_list {

        let url= url.to_string();

        let http_task= engine.create_http_task(url, None, None).await?;

        http_tasks.push(http_task);

    }

    let http_download_tasks= stream::iter((0..http_tasks.len()).map(|i| {

        let threaded_task= http_tasks[i].clone();
        let engine_clone= engine.clone();

        async move {

            engine_clone.http_download(threaded_task).await?;

            Ok::<_, RawstErr>(())

        }

    }));

    http_download_tasks.buffer_unordered(http_tasks.len()).collect::<Vec<_>>().await;

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