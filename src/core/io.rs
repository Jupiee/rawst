use crate::core::errors::RawstErr;
use crate::core::utils::FileName;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::{future::join_all, stream::StreamExt};
use reqwest::Response;
use tokio::fs::{File, remove_file};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use indicatif::ProgressBar;

pub async fn merge_files(filename: &FileName, chunks: u64) -> Result<(), RawstErr> {

    let output_file= File::create(filename.to_string()).await
        .map_err(|e| RawstErr::FileError(e))?;

    let mut output_file= BufWriter::new(output_file);

    let mut io_tasks= Vec::new();

    // Creates a closure for each temporary file read operation
    (0..chunks).into_iter().for_each(|i| {

        let temp_filename= format!("{}-{}.tmp", filename.stem, i);

        let io_task= tokio::spawn(async move {

            let temp_file= File::open(&temp_filename).await.map_err(|e| RawstErr::FileError(e))?;
            let mut temp_file= BufReader::new(temp_file);
            let mut buffer= Vec::new();

            temp_file.read_to_end(&mut buffer).await.map_err(|e| RawstErr::FileError(e))?;

            remove_file(temp_filename).await.map_err(|e| RawstErr::FileError(e))?;

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

    return Ok(())

}

pub async fn create_file(filepath: String, response: Response, pb: ProgressBar, downloaded: Arc<AtomicU64>) -> Result<(), RawstErr> {

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

pub async fn read_links(filepath: &String) -> Result<String, RawstErr> {

    let mut file= File::open(filepath).await.map_err(|e| RawstErr::FileError(e))?;

    let mut file_content= String::new();

    file.read_to_string(&mut file_content).await.map_err(|e| RawstErr::FileError(e))?;

    return Ok(file_content)

}