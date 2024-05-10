use crate::core::utils::FileName;

use std::sync::{atomic::AtomicU64, Arc};

use reqwest::header::HeaderMap;

#[derive(Clone)]
pub struct Chunk {
    
    pub x_offset: u64,  // x offset is starting byte
    pub y_offset: u64   // y offset is end byte

}

impl Chunk {

    pub fn new(x_offset: u64, y_offset: u64) -> Self {

        return Chunk {

            x_offset,
            y_offset

        }

    }

}

#[derive(Debug, Clone)]
pub struct HttpTask {

    pub url: String,
    pub filename: FileName,
    pub downloaded: Arc<AtomicU64>,

    // Cached headermap from Head request
    // Efficient for header values retrieval
    headers: HeaderMap

}

impl HttpTask {

    pub fn new(url: String, filename: FileName, cached_headers: HeaderMap) -> Self {

        return HttpTask {

            url,
            filename,
            headers: cached_headers,
            downloaded: Arc::new(AtomicU64::new(0))

        };

    }

    pub fn into_chunks(&self, number_of_chunks: u64) -> Vec<Chunk> {

        let total_size= self.content_length();

        let chunk_size= total_size / number_of_chunks;

        let mut chunks : Vec<Chunk> = Vec::with_capacity(number_of_chunks as usize);

        (0..number_of_chunks).into_iter().for_each(|i| {

            // Calculates start and end byte offset for each chunk
            match i {

                0 => {

                    chunks.push(Chunk::new(0, chunk_size));

                },
                last_chunk if last_chunk == number_of_chunks - 1 => {

                    let start= chunks[(i - 1) as usize].y_offset + 1;
                    let end= total_size;

                    chunks.push(Chunk::new(start, end));

                },
                _ => {

                    let start= chunks[(i - 1) as usize].y_offset + 1;
                    let end= start + chunk_size;

                    chunks.push(Chunk::new(start, end));

                }

            }

        });

        return chunks;

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