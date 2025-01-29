use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use iri_string::types::IriString;
use reqwest::header::HeaderMap;
use chrono::prelude::{Local, DateTime};
use sha2::{Sha256, Digest};

#[derive(Clone, Debug)]
pub struct Chunk {
    pub x_offset: u64, // x offset is starting byte
    pub y_offset: u64, // y offset is end byte

    pub downloaded: Arc<AtomicU64>, // downloaded bytes of a chunk
}

impl Chunk {
    pub fn new(x_offset: u64, y_offset: u64) -> Self {
        Chunk {
            x_offset,
            y_offset,
            downloaded: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn is_downloaded(&self) -> bool {
        // Each chunk file size is usually the difference of x and y offset added by 1
        // (y_offset - x_offset) + 1
        self.x_offset == self.y_offset + 1
    }
}

#[derive(Clone, Debug)]
pub enum ChunkType {
    Single(Chunk),
    Multiple(Vec<Chunk>),
    None,
}

#[derive(Clone)]
pub struct HttpTask {
    pub iri: IriString,
    pub filename: PathBuf,
    pub total_downloaded: Arc<AtomicU64>,
    pub chunk_data: ChunkType,
    pub additional_headers: HashMap<String, String>,
    pub timestamp: DateTime<Local>,

    // Cached headermap from Head request
    // Efficient for header values retrieval
    headers: HeaderMap,
}

impl std::fmt::Debug for HttpTask {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "HttpTask{{iri:{}, filename:{:?}, ...}}",
            self.iri, self.filename
        )
    }
}

impl HttpTask {
    pub fn new(
        iri: IriString,
        filename: PathBuf,
        cached_headers: HeaderMap,
        additional_headers: HashMap<String, String>
    ) -> Self {
        assert!(filename.is_relative());

        let chunk_data = ChunkType::None;

        HttpTask {
            iri,
            filename,
            headers: cached_headers,
            total_downloaded: Arc::new(AtomicU64::new(0)),
            chunk_data,
            additional_headers,
            timestamp: Local::now(),
        }
    }

    pub fn hashed_file_name(&self) -> String {

        let formatted_string = format!("{}{}", self.iri, self.timestamp);

        let mut hasher = Sha256::new();

        hasher.update(formatted_string.as_bytes());

        format!("{:x}", hasher.finalize())

    }

    pub fn allocate_chunks(&mut self, number_of_chunks: usize) {

        // Allocates chunk space depending on number of threads 
        if number_of_chunks == 1 {
            self.chunk_data = ChunkType::Single(Chunk::new(0, 0));
        }
        else {
            self.chunk_data = ChunkType::Multiple(Vec::with_capacity(number_of_chunks));
        }

    }

    pub fn calculate_chunks(&mut self, number_of_chunks: u64) {
        let total_size = self.content_length();

        self.allocate_chunks(number_of_chunks as usize);

        match &mut self.chunk_data {
            ChunkType::Single(chunk) => {
                chunk.x_offset = 0;
                chunk.y_offset = total_size;
            }
            ChunkType::Multiple(chunks) => {
                let chunk_size = total_size / number_of_chunks;

                (0..number_of_chunks).for_each(|i| match i {
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
            ChunkType::None => (),
        }
    }

    pub fn calculate_x_offsets(&mut self, offsets: &[u64]) {
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

                    chunks[index].x_offset += *value;
                }
            }
            ChunkType::None => (),
        }
    }

    pub fn content_length(&self) -> u64 {
        match self.headers.get("content-length") {
            Some(length) => length
                .to_str()
                .unwrap()
                .parse()
                .expect("Invalid size format"),
            None => 0,
        }
    }

    pub fn allows_partial_content(&self) -> bool {
        match self.headers.get("accept-ranges") {
            Some(value) => value != "none",
            None => false,
        }
    }
}
