use rawst_dl::cli::args;
use rawst_dl::core::config::Config;
use rawst_dl::cli::args::Command;
use rawst_dl::core::errors::RawstErr;
use rawst_dl::server::server;
use rawst_dl::client::client;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    let args = args::get();
    let config = match Config::load().await {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.initialise_files().await.map_err(|_| RawstErr::InitilisationError).unwrap();
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
                client::run_client(download_args, config, address).await?;

            },
            _ => ()
        }
    }

    Ok(())
}