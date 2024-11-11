use rawst::cli::args;
use rawst::cli::args::Command;
use rawst::core::config::Config;
use rawst::core::downloader;
use rawst::core::errors::RawstErr;
use rawst::core::history;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    if let Some(cmd) = args::get_command() {
        run(cmd).await?
    }

    Ok(())
}

pub async fn run(cmd: Command) -> Result<(), RawstErr> {
    let config = match Config::load().await {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.initialise_files().await?;
            config
        }
    };

    match cmd {
        Command::Download(args) => downloader::download(args, config).await,
        Command::Resume(args) => downloader::resume_download(args, config).await,
        Command::History(args) => history::show_history(args, config).await,
    }
}
