use rawst_dl::cli::args;
use rawst_dl::cli::args::Command;
use rawst_dl::core::errors::RawstErr;
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
                let proto_args = args.to_primitive_types();
                client::run_client(proto_args, address).await?;

            }
        }
    }

    Ok(())
}