use rawst::cli::args;
use rawst::cli::args::Command;
use rawst::cli::parser as not_the_parser;
use rawst::core::config::Config;
use rawst::core::errors::RawstErr;
use rawst::core::io::config_exist;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    if let Some(cmd) = args::get_command() {
        run(cmd).await?
    }

    Ok(())
}

pub async fn run(cmd: Command) -> Result<(), RawstErr> {
    let config = match config_exist() {
        true => Config::load().await?,
        false => Config::build().await?,
    };

    match cmd {
        Command::Download(args) => {
            if args.input_file.is_some() {
                not_the_parser::list_download(args, config).await?;
            } else {
                not_the_parser::url_download(args, config).await?;
            }
            Ok(())
        }
        Command::Resume(args) => {
            not_the_parser::resume_download(args, config).await?;
            Ok(())
        }
        Command::History(args) => {
            not_the_parser::display_history(args, config).await?;
            Ok(())
        }
    }
}
