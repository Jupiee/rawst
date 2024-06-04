use crate::core::errors::RawstErr;
use crate::core::task::{HttpTask, Getter};
use crate::core::config::Config;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use chrono::prelude::Local;
use toml;
use base64::{Engine, prelude::BASE64_STANDARD};

#[derive(Deserialize, Serialize)]
struct Downloads {

    record: Vec<Record>

}

#[derive(Deserialize, Serialize)]
struct Record {

    pub id: String,
    pub url: String,
    pub file_name: String,
    pub file_size: u64,
    pub total_downloaded: u64,
    pub file_location: String,
    pub threads_used: usize,
    pub timestamp: String

}

impl Record {

    pub fn new(url: String, file_name: String, file_size: u64, total_downloaded: u64, file_location: String, threads_used: usize) -> Record {

        let current_time= Local::now();

        let encoded_timestamp= BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());

        Record {

            id: encoded_timestamp,
            url,
            file_name,
            file_size,
            total_downloaded,
            file_location,
            threads_used,
            timestamp: current_time.to_string()

        }

    }

}

pub struct HistoryManager {

    pub history_file_path: String

}

impl HistoryManager {

    pub fn new(local_dir_path: String) -> Self {

        HistoryManager {

            history_file_path: local_dir_path

        }

    }

    pub fn add_record(&self, task: &HttpTask, config: &Config) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.toml");

        let mut history_file= File::options()
            .append(true)
            .open(file_path)
            .map_err(|e| RawstErr::FileError(e))?;

        let record= Record::new(
            task.url.clone(),
            task.filename.to_string(),
            task.content_length(),
            task.get_downloaded(),
            config.download_path.clone(),
            config.threads
        );

        let download= Downloads {

            record: vec![record]

        };

        let toml= toml::to_string(&download).unwrap();

        let content= format!("{}\n", toml);

        history_file.write(&content.as_bytes()).map_err(|e| RawstErr::FileError(e))?;

        Ok(())

    }

    pub async fn get_history(&self) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.toml");

        let mut history_file= File::open(file_path)
            .map_err(|e| RawstErr::FileError(e))?;

        let mut content= String::new();

        history_file.read_to_string(&mut content).map_err(|e| RawstErr::FileError(e))?;

        let value: toml::Value = toml::from_str(&content).unwrap();

        if let Some(table) = value.as_table() {

            println!("{}", table);

        }

        else {

            println!("No records found");

        }

        Ok(())

    }

}