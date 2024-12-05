use crate::cli::args::DownloadArgs;
use crate::cli::args::ResumeArgs;
use crate::core::config::Config;
use crate::core::engine::Engine;
use crate::core::errors::RawstErr;

pub async fn download(args: DownloadArgs, config: Config) -> Result<(), RawstErr> {
    // TODO: Fuse url_download and list_download
    // TODO: Support downloading many elements from each source
    log::trace!("Downloading files ({args:?}, {config:?})");
    let engine= Engine::new(config);

    if args.input_file.is_some() {
        engine.process_list_download(args).await
    } else {
        engine.process_url_download(args).await
    }
}

pub async fn resume_download(args: ResumeArgs, config: Config) -> Result<(),RawstErr> {
    let ids= args.download_ids;
    let mut engine= Engine::new(config);

    if ids.len() > 1 {
        for id in ids {
            engine.process_resume_request(id).await?

        }

        Ok(())
    }
    else {
        let id= ids.iter().next().unwrap().to_string();
        engine.process_resume_request(id).await

    }

}