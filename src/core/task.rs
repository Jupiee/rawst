use crate::core::utils::FileName;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use reqwest::header::HeaderMap;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub x_offset: u64, // x offset is starting byte
    pub y_offset: u64, // y offset is end byte

    pub downloaded: Arc<AtomicU64>, // downloaded bytes of a chunk
}

impl Chunk {
    pub fn new(x_offset: u64, y_offset: u64) -> Self {
        return Chunk {
            x_offset,
            y_offset,
            downloaded: Arc::new(AtomicU64::new(0)),
        };
    }

    pub fn is_downloaded(&self) -> bool {
        if self.downloaded.load(Ordering::SeqCst) == self.y_offset + 1 {
            return true;
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChunkType {
    Single(Chunk),
    Multiple(Vec<Chunk>),
    None,
}

#[derive(Clone, Debug)]
pub struct HttpTask {
    pub url: String,
    pub filename: FileName,
    pub total_downloaded: Arc<AtomicU64>,
    pub chunk_data: ChunkType,

    // Cached headermap from Head request
    // Efficient for header values retrieval
    headers: HeaderMap,
}

impl HttpTask {
    pub fn new(
        url: String,
        filename: FileName,
        cached_headers: HeaderMap,
        number_of_chunks: usize,
    ) -> Self {
        let chunk_data = if number_of_chunks == 1 {
            ChunkType::None
        } else {
            ChunkType::Multiple(Vec::with_capacity(number_of_chunks))
        };

        return HttpTask {
            url,
            filename,
            headers: cached_headers,
            total_downloaded: Arc::new(AtomicU64::new(0)),
            chunk_data,
        };
    }

    pub fn calculate_chunks(&mut self, number_of_chunks: u64) {
        let total_size = self.content_length();

        match &mut self.chunk_data {
            ChunkType::Single(chunk) => {
                chunk.x_offset = 0;
                chunk.y_offset = total_size;
            }
            ChunkType::Multiple(chunks) => {
                let chunk_size = total_size / number_of_chunks;

                (0..number_of_chunks).into_iter().for_each(|i| match i {
                    0 => {
                        chunks.push(Chunk::new(0, chunk_size));
                    }
                    last_chunk if last_chunk == number_of_chunks - 1 => {
                        let start = chunks[(i - 1) as usize].y_offset + 1;
                        let end = total_size;

                        chunks.push(Chunk::new(start, end));
                    }
                    _ => {
                        let start = chunks[(i - 1) as usize].y_offset + 1;
                        let end = start + chunk_size;

                        chunks.push(Chunk::new(start, end));
                    }
                });
            }
            ChunkType::None => return,
        }
    }

    pub fn create_single_chunk(&mut self) {
        self.chunk_data = ChunkType::Single(Chunk::new(0, 0));
    }

    pub fn calculate_x_offsets(&mut self, offsets: &Vec<u64>) {
        match &mut self.chunk_data {
            ChunkType::Single(chunk) => {
                chunk.x_offset += offsets[0];
                chunk.downloaded.fetch_add(offsets[0], Ordering::SeqCst);
            }
            ChunkType::Multiple(chunks) => {
                for (index, value) in offsets.iter().enumerate() {
                    chunks[index]
                        .downloaded
                        .fetch_add(offsets[index], Ordering::SeqCst);

                    let difference = chunks[index].y_offset - chunks[index].x_offset;

                    if chunks[index].downloaded.load(Ordering::SeqCst) < difference {
                        chunks[index].x_offset += *value;
                    }
                }
            }
            ChunkType::None => return,
        }
    }

    pub fn content_length(&self) -> u64 {
        match self.headers.get("content-length") {
            Some(length) => {
                return length
                    .to_str()
                    .unwrap()
                    .parse()
                    .expect("Invalid size format")
            }
            None => return 0,
        };
    }

    pub fn allows_partial_content(&self) -> bool {
        match self.headers.get("accept-ranges") {
            Some(value) => {
                if value != "none" {
                    return true;
                }

                return false;
            }
            None => return false,
        }
    }
}
