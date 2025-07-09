use rawst_dl::cli::args;
use rawst_dl::cli::args::{Arguments, InputSource};
use rawst_dl::cli::args::Command;
use rawst_dl::core::config::{Config, edit_config};
use rawst_dl::core::engine::{download, resume_download};
use rawst_dl::core::errors::RawstErr;
use rawst_dl::core::history;
use rawst_dl::core::logger;
use rawst_dl::server::server;
use rawst_dl::client::client;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    let args = args::get();
    let config = match Config::load().await {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.initialise_files().await?;
            config
        }
    };

    let address = "http://127.0.0.1:50051".to_owned();
    
    if let Some(cmd) = args.command {
        match cmd {
            Command::Server(server_options) => {

                server::start_server(server_options).await?;

            }, 
            Command::Download(download_args) => {

                let threads = match download_args.threads {
                    Some(number) => Some(number as u32),
                    None => None
                };

                let string_url = match download_args.input {
                    Some(InputSource::File(file_pathbuf)) => Some(file_pathbuf.to_string_lossy().into_owned()) ,
                    Some(InputSource::Iris(vec_iri)) => Some(vec_iri.into_iter().next().unwrap().as_str().to_string()),
                    _ => None
                };

                let vec_output_pathbuf = download_args.output_file_path.into_iter().map(|i| i.to_string_lossy().into_owned()).collect::<Vec<String>>();
                let header_file_path = match download_args.headers_file_path {
                    Some(file_path) => Some(file_path.to_string_lossy().into_owned()),
                    None => None
                };
                let color = match args.color.color {

                    concolor_clap::ColorChoice::Auto => "Auto".to_owned(),
                    concolor_clap::ColorChoice::Always => "Always".to_owned(),
                    concolor_clap::ColorChoice::Never => "Never".to_owned(),

                };
                let verbosity = match args.verbosity {
                    Some(log::LevelFilter::Off) => Some("Off".to_owned()),
                    Some(log::LevelFilter::Debug) => Some("Debug".to_owned()),
                    Some(log::LevelFilter::Error) => Some("Error".to_owned()),
                    Some(log::LevelFilter::Info) => Some("Info".to_owned()),
                    Some(log::LevelFilter::Trace) => Some("Trace".to_owned()),
                    Some(log::LevelFilter::Warn) => Some("Warn".to_owned()),
                    None => None

                };

                let log_verbosity = match args.log_verbosity {
                    Some(log::LevelFilter::Off) => Some("Off".to_owned()),
                    Some(log::LevelFilter::Debug) => Some("Debug".to_owned()),
                    Some(log::LevelFilter::Error) => Some("Error".to_owned()),
                    Some(log::LevelFilter::Info) => Some("Info".to_owned()),
                    Some(log::LevelFilter::Trace) => Some("Trace".to_owned()),
                    Some(log::LevelFilter::Warn) => Some("Warn".to_owned()),
                    None => None

                };

                let converted = (threads, string_url, vec_output_pathbuf, header_file_path, color, verbosity, log_verbosity);
                client::run_client(converted, address).await?;

            },
            Command::Resume(args) => resume_download(args, config).await?,
            Command::History(args) => history::check_history_args(args, config).await?,
            Command::Config => edit_config(config).await?,
        }
    }

    Ok(())
}
