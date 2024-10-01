use crate::core::errors::RawstErr;
use crate::core::utils::FileName;
use crate::core::config::Config;
use crate::core::task::{HttpTask, ChunkType};

use std::path::Path;
use std::fs;
use std::sync::atomic::Ordering;

use futures::{future::join_all, stream::StreamExt};
use reqwest::Response;
use tokio::fs::{File, remove_file};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
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

pub async fn create_file(task: &HttpTask, response: Response, pb: &ProgressBar, base_path: &String) -> Result<(), RawstErr> {

    let filepath= Path::new(base_path).join(&task.filename.to_string());

    let mut file= File::options().append(true).create(true).open(filepath).await.map_err(|e| RawstErr::FileError(e))?;

    let mut stream= response.bytes_stream();

    // Recieves bytes as stream and write them into the a file
    while let Some(chunk) = stream.next().await {

        let chunk= chunk.map_err(|e| RawstErr::HttpError(e))?;

        file.write_all(&chunk).await.map_err(|e| RawstErr::FileError(e))?;

        // Updates total download bytes and the progressbar
        let chunk_size= chunk.len() as u64;
        task.total_downloaded.fetch_add(chunk_size, Ordering::SeqCst);
        pb.set_position(task.total_downloaded.load(Ordering::SeqCst));
    
    }

    Ok(())

}

pub async fn create_cache(chunk_number: usize, task: &HttpTask, response: Response, pb: &ProgressBar, base_path: &String) -> Result<(), RawstErr> {

    if let ChunkType::Multiple(chunks) = &task.chunk_data {

        if chunks[chunk_number].is_downloaded() {

            return Ok(())
    
        }

        let temp_filepath= format!("{}-{}.tmp", task.filename.stem, chunk_number);

        let filepath= Path::new(base_path).join(temp_filepath);

        let mut file= File::options().append(true).create(true).open(filepath).await.map_err(|e| RawstErr::FileError(e))?;

        let mut stream= response.bytes_stream();

        // Recieves bytes as stream and write them into the a file
        while let Some(chunk) = stream.next().await {

            let chunk= chunk.map_err(|e| RawstErr::HttpError(e))?;

            file.write_all(&chunk).await.map_err(|e| RawstErr::FileError(e))?;

            file.flush().await.map_err(|e| RawstErr::FileError(e))?;

            // Updates total download bytes and the progressbar
            let chunk_size= chunk.len() as u64;
            task.total_downloaded.fetch_add(chunk_size, Ordering::SeqCst);
            pb.set_position(task.total_downloaded.load(Ordering::SeqCst));

            // Updates downloaded bytes for each chunk
            chunks[chunk_number].downloaded.fetch_add(chunk_size, Ordering::SeqCst);
        
        }

    }

    Ok(())

}

pub fn get_cache_sizes(filename: String, threads: usize, config: Config) -> Result<Vec<u64>, RawstErr> {

    let mut cache_sizes: Vec<u64>= vec![];

    match threads > 1 {

        false => {

            let path= Path::new(&config.download_path).join(filename);

            let meta_data= fs::metadata(path).unwrap();

            cache_sizes.push(meta_data.len());

        },
        true => {

            (0..threads).into_iter().for_each(|i| {

                let (filename, _extension)= filename.rsplit_once(".").unwrap();
        
                let formatted_temp_filename= format!("{}-{}.tmp", filename.trim(), i);
        
                let path= Path::new(&config.cache_path).join(formatted_temp_filename);
        
                let meta_data= fs::metadata(path).unwrap();
        
                cache_sizes.push(meta_data.len());
        
            });

        }

    }

    Ok(cache_sizes)

}

pub fn config_exist() -> bool {

    let base_path= BaseDirs::new().unwrap()
        .data_local_dir()
        .join("rawst");

    let config_file_path= &base_path.join("config.toml");

    let history_file_path= &base_path.join("history.json");

    return config_file_path.exists() && history_file_path.exists()

}

pub async fn read_links(filepath: &String) -> Result<String, RawstErr> {

    let mut file= File::open(filepath).await.map_err(|e| RawstErr::FileError(e))?;

    let mut file_content= String::new();

    file.read_to_string(&mut file_content).await.map_err(|e| RawstErr::FileError(e))?;

    Ok(file_content)

}