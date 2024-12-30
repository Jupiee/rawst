use rawst::cli::args;
use rawst::cli::args::Arguments;
use rawst::cli::args::Command;
use rawst::core::config::Config;
use rawst::core::engine::{download, resume_download};
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
    
    if args.verbosity.is_some() || args.log_verbosity.is_some() {
        logger::init(&config, &args).map_err(|_| RawstErr::InitilisationError)?;
    }

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
            Command::Download(args) => download(args, config).await?,
            Command::Resume(args) => resume_download(args, config).await?,
            Command::History => history::show_history(config).await?,
        }
    }

    Ok(())
}
