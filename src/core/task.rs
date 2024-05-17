use crate::core::utils::FileName;

use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

use reqwest::header::HeaderMap;

// Abstract trait for getting download bytes
pub trait Getter {

    fn get_bytes_left(&self) -> u64;

    fn get_downloaded(&self) -> u64;

}

#[derive(Clone)]
pub struct Chunk {

    pub x_offset: u64,  // x offset is starting byte
    pub y_offset: u64,  // y offset is end byte

    pub downloaded: Arc<AtomicU64>  // downloaded bytes of a chunk

}

#[allow(dead_code)]
impl Chunk {

    pub fn new(x_offset: u64, y_offset: u64) -> Self {

        return Chunk {

            x_offset,
            y_offset,
            downloaded: Arc::new(AtomicU64::new(0))

        }

    }

}

impl Getter for Chunk {

    fn get_bytes_left(&self) -> u64 {

        let downloaded= self.downloaded.load(Ordering::SeqCst);

        return self.y_offset - downloaded

    }

    fn get_downloaded(&self) -> u64 {
        
        return self.downloaded.load(Ordering::SeqCst)

    }

}

#[derive(Clone)]
pub struct HttpTask {

    pub url: String,
    pub filename: FileName,
    pub total_downloaded: Arc<AtomicU64>,
    pub chunks: Vec<Chunk>,

    // Cached headermap from Head request
    // Efficient for header values retrieval
    headers: HeaderMap

}

impl Getter for HttpTask {

    fn get_bytes_left(&self) -> u64 {
        
        let downloaded= self.total_downloaded.load(Ordering::SeqCst);

        return self.content_length() - downloaded

    }

    fn get_downloaded(&self) -> u64 {
        
        return self.total_downloaded.load(Ordering::SeqCst)

    }

}

impl HttpTask {

    pub fn new(url: String, filename: FileName, cached_headers: HeaderMap, number_of_chunks: usize) -> Self {

        return HttpTask {

            url,
            filename,
            headers: cached_headers,
            total_downloaded: Arc::new(AtomicU64::new(0)),
            chunks: Vec::with_capacity(number_of_chunks)

        };

    }

    pub fn into_chunks(&mut self, number_of_chunks: u64) {

        let total_size= self.content_length();

        let chunk_size= total_size / number_of_chunks;

        (0..number_of_chunks).into_iter().for_each(|i| {

            // Calculates start and end byte offset for each chunk
            match i {

                0 => {

                    self.chunks.push(Chunk::new(0, chunk_size));

                },
                last_chunk if last_chunk == number_of_chunks - 1 => {

                    let start= self.chunks[(i - 1) as usize].y_offset + 1;
                    let end= total_size;

                    self.chunks.push(Chunk::new(start, end));

                },
                _ => {

                    let start= self.chunks[(i - 1) as usize].y_offset + 1;
                    let end= start + chunk_size;

                    self.chunks.push(Chunk::new(start, end));

                }

            }

        });

    }

    pub fn content_length(&self) -> u64 {

        match self.headers.get("content-length") {

            Some(length) => return length.to_str().unwrap().parse().expect("Invalid size format"),
            None => return 0

        };

    }

    pub fn allows_partial_content(&self) -> bool {

        match self.headers.get("accept-ranges") {

            Some(value) => {

                if value != "none" {

                    return true

                }

                return false

            },
            None => {

                return false

            }

        }

    }

}