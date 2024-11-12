use rawst::cli::args;
use rawst::cli::args::Arguments;
use rawst::cli::args::Command;
use rawst::core::config::Config;
use rawst::core::downloader;
use rawst::core::errors::RawstErr;
use rawst::core::history;
use rawst::core::logger;

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

    logger::init(&config, &args).map_err(|_| RawstErr::InitilisationError)?;

    log::trace!("Arguments: {args:?}");
    log::trace!("Config: {config:?}");

    if args.command.is_some() {
        run(config, args).await?
    }

    Ok(())
}

async fn run(config: Config, args: Arguments) -> Result<(), RawstErr> {
    if let Some(cmd) = args.command {
        match cmd {
            Command::Download(args) => downloader::download(args, config).await?,
            Command::Resume(args) => downloader::resume_download(args, config).await?,
            Command::History(args) => history::show_history(args, config).await?,
        }
    }

    Ok(())
}
