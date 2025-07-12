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

    let address = "http://127.0.0.1:50051".to_owned();
    
    if let Some(cmd) = &args.command {
        match cmd {
            Command::Server(server_options) => {
                server::start_server(server_options).await?;

            }, 
            _ => {
                let converted = args.to_primitive_types();
                client::run_client(converted, address).await?;

            }
        }
    }

    Ok(())
}