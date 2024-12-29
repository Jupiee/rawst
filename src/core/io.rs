use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use futures::{future::join_all, stream::StreamExt};
use indicatif::ProgressBar;
use reqwest::Response;
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::task::{ChunkType, HttpTask};
use crate::core::utils::chunk_file_name;

pub async fn merge_files(filename: &PathBuf, config: &Config) -> Result<(), RawstErr> {
    let output_path = config.download_dir.join(filename);

    let output_file = File::create(output_path)
        .await
        .map_err(RawstErr::FileError)?;

    let mut output_file = BufWriter::new(output_file);

    let mut io_tasks = Vec::new();

    // Creates a closure for each temporary file read operation
    (0..config.threads).for_each(|i| {
        let chunk_filename = chunk_file_name(filename, i);
        assert!(chunk_filename.is_relative());
        let chunk_path = config.cache_dir.join(chunk_filename);

        let io_task = tokio::spawn(async move {
            let temp_file = File::open(&chunk_path).await.map_err(RawstErr::FileError)?;
            let mut temp_file = BufReader::new(temp_file);
            let mut buffer = Vec::new();

            temp_file
                .read_to_end(&mut buffer)
                .await
                .map_err(RawstErr::FileError)?;

            remove_file(chunk_path).await.map_err(RawstErr::FileError)?;

            Ok::<_, RawstErr>(buffer)
        });

        io_tasks.push(io_task);
    });

    let results = join_all(io_tasks).await;

    for task in results {
        let data = task.map_err(|err| RawstErr::FileError(err.into()))??;

        output_file
            .write_all(&data)
            .await
            .map_err(RawstErr::FileError)?;
    }

    output_file.flush().await.map_err(RawstErr::FileError)?;

    Ok(())
}

pub async fn create_file(
    task: &HttpTask,
    response: Response,
    pb: &ProgressBar,
    base_path: &Path,
) -> Result<(), RawstErr> {
    let file_path = base_path.join(task.filename.clone());

    let mut file = File::options()
        .append(true)
        .create(true)
        .open(file_path)
        .await
        .map_err(RawstErr::FileError)?;

    let mut stream = response.bytes_stream();

    // Recieves bytes as stream and write them into the a file
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(RawstErr::HttpError)?;

        file.write_all(&chunk).await.map_err(RawstErr::FileError)?;

        // Updates total download bytes and the progressbar
        let chunk_size = chunk.len() as u64;
        task.total_downloaded
            .fetch_add(chunk_size, Ordering::SeqCst);
        pb.set_position(task.total_downloaded.load(Ordering::SeqCst));
    }

    Ok(())
}

pub async fn create_cache(
    chunk_number: usize,
    task: &HttpTask,
    response: Response,
    pb: &ProgressBar,
    base_path: &Path,
) -> Result<(), RawstErr> {
    if let ChunkType::Multiple(chunks) = &task.chunk_data {

        let chunk_file_name = chunk_file_name(&task.filename, chunk_number);
        assert!(chunk_file_name.is_relative());
        assert!(base_path.is_dir());

        let filepath = base_path.join(chunk_file_name);

        let mut file = File::options()
            .append(true)
            .create(true)
            .open(filepath)
            .await
            .map_err(RawstErr::FileError)?;

        let mut stream = response.bytes_stream();

        // Recieves bytes as stream and write them into the a file
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(RawstErr::HttpError)?;

            file.write_all(&chunk).await.map_err(RawstErr::FileError)?;

            file.flush().await.map_err(RawstErr::FileError)?;

            // Updates total download bytes and the progressbar
            let chunk_size = chunk.len() as u64;
            task.total_downloaded
                .fetch_add(chunk_size, Ordering::SeqCst);
            pb.set_position(task.total_downloaded.load(Ordering::SeqCst));

            // Updates downloaded bytes for each chunk
            chunks[chunk_number]
                .downloaded
                .fetch_add(chunk_size, Ordering::SeqCst);
        }
    }

    Ok(())
}

pub fn get_cache_sizes(
    filename: &PathBuf,
    threads: usize,
    config: Config,
) -> Result<Vec<u64>, RawstErr> {
    let mut cache_sizes: Vec<u64> = vec![];

    match threads > 1 {
        false => {
            let path = config.download_dir.join(filename);

            let meta_data = std::fs::metadata(path).map_err(|err| RawstErr::FileError(err))?;

            cache_sizes.push(meta_data.len());
        }
        true => {
            (0..threads).try_for_each(|i| {
                let chunk_filename = chunk_file_name(filename, i);

                let path = config.cache_dir.join(chunk_filename);

                let meta_data = std::fs::metadata(path).map_err(|err| RawstErr::FileError(err))?;

                cache_sizes.push(meta_data.len());
                Ok::<_, RawstErr>(())
            })?;
        }
    }

    Ok(cache_sizes)
}

pub async fn read_links(filepath: &PathBuf) -> Result<String, RawstErr> {
    let mut file = File::open(filepath).await.map_err(RawstErr::FileError)?;

    let mut file_content = String::new();

    file.read_to_string(&mut file_content)
        .await
        .map_err(RawstErr::FileError)?;

    Ok(file_content)
}
