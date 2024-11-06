mod cli;
mod core;

use cli::parser::init;
use core::errors::RawstErr;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    init().await?;

    Ok(())
}
