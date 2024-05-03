use crate::core::errors::RawstErr;
use crate::core::utils::FileName;
use crate::core::config::Config;

use std::sync::Arc;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::{future::join_all, stream::StreamExt};
use reqwest::Response;
use tokio::fs::{File, remove_file, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use toml::from_str;
use directories::BaseDirs;
use indicatif::ProgressBar;

pub async fn merge_files(filename: &FileName, config: &Config) -> Result<(), RawstErr> {

    let output_path= Path::new(&config.download_path).join(filename.to_string());

    let output_file= File::create(output_path).await
        .map_err(|e| RawstErr::FileError(e))?;

    let mut output_file= BufWriter::new(output_file);

    let mut io_tasks= Vec::new();

    // Creates a closure for each temporary file read operation
    (0..config.threads).into_iter().for_each(|i| {

        let formatted_temp_filename= format!("{}-{}.tmp", filename.stem, i);

        let temp_file_path= Path::new(&config.cache_path).join(formatted_temp_filename);

        let io_task= tokio::spawn(async move {

            let temp_file= File::open(&temp_file_path).await.map_err(|e| RawstErr::FileError(e))?;
            let mut temp_file= BufReader::new(temp_file);
            let mut buffer= Vec::new();

            temp_file.read_to_end(&mut buffer).await.map_err(|e| RawstErr::FileError(e))?;

            remove_file(temp_file_path).await.map_err(|e| RawstErr::FileError(e))?;

            Ok::<_, RawstErr>(buffer)
        
        });
        
        io_tasks.push(io_task);

    }
    );

    let results= join_all(io_tasks).await;

    for task in results {

        let data= task.map_err(|err| RawstErr::FileError(err.into()))??;

        output_file.write_all(&data).await.map_err(|e| RawstErr::FileError(e))?;

    }

    output_file.flush().await.map_err(|e| RawstErr::FileError(e))?;

    Ok(())

}

pub async fn create_file(filename: String, response: Response, pb: ProgressBar, downloaded: Arc<AtomicU64>, base_path: &String) -> Result<(), RawstErr> {

    let filepath= Path::new(base_path).join(filename);

    let mut file= File::create(filepath).await.map_err(|e| RawstErr::FileError(e))?;

    let mut stream= response.bytes_stream();

    // Recieves bytes as stream and write them into the a file
    while let Some(chunk) = stream.next().await {

        let chunk= chunk.map_err(|e| RawstErr::HttpError(e))?;

        file.write_all(&chunk).await.map_err(|e| RawstErr::FileError(e))?;

        // Updates the progressbar
        let chunk_size= chunk.len() as u64;
        downloaded.fetch_add(chunk_size, Ordering::SeqCst);
        pb.set_position(downloaded.load(Ordering::SeqCst));
    
    }

    Ok(())

}

pub async fn build_config() -> Result<Config, RawstErr> {

    let config= Config::default();

    let content= format!(
        "download_path = {:?}\ncache_path= {:?}\nconfig_path= {:?}\nthreads= {:?}",
        config.download_path, config.cache_path, config.config_path, config.threads);

    let root_path= Path::new(&config.config_path).join("rawst");
    let config_file_path= &root_path.join("config.toml");

    create_dir_all(root_path).await.expect("Failed to create config directory");
    create_dir_all(&config.cache_path).await.expect("Failed to create cache directory");

    let mut config_file= File::create(config_file_path).await.map_err(|e| RawstErr::FileError(e))?;

    config_file.write_all(&content.as_bytes()).await.map_err(|e| RawstErr::FileError(e))?;

    Ok(config)

}

pub async fn load_config() -> Result<Config, RawstErr> {

    let config_dir= BaseDirs::new().unwrap()
        .data_local_dir()
        .join("rawst")
        .join("config.toml");

    let mut file_content= String::new();

    let mut file= File::open(config_dir).await.map_err(|e| RawstErr::FileError(e))?;

    file.read_to_string(&mut file_content).await.map_err(|e| RawstErr::FileError(e))?;

    let config: Config = from_str(&file_content).unwrap();

    Ok(config)

}

pub fn config_exist() -> bool {

    let config_file_path= BaseDirs::new().unwrap()
        .data_local_dir()
        .join("rawst")
        .join("config.toml");

    return config_file_path.exists()

}

pub async fn read_links(filepath: &String) -> Result<String, RawstErr> {

    let mut file= File::open(filepath).await.map_err(|e| RawstErr::FileError(e))?;

    let mut file_content= String::new();

    file.read_to_string(&mut file_content).await.map_err(|e| RawstErr::FileError(e))?;

    Ok(file_content)

}