use rawst::cli::parser::init;
use rawst::core::errors::RawstErr;

#[tokio::main]
async fn main() -> Result<(), RawstErr> {
    init().await?;

    Ok(())
}
