use crate::core::errors::RawstErr;
use crate::core::task::{HttpTask, Getter};
use crate::core::config::Config;

use std::path::Path;

use serde::{Deserialize, Serialize};
use chrono::prelude::Local;
use tokio::io::AsyncReadExt;
use tokio::{fs::File, io::AsyncWriteExt};
use toml;

#[derive(Deserialize, Serialize)]
struct Downloads {

    record: Vec<Record>

}

#[derive(Deserialize, Serialize)]
struct Record {

    pub url: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_location: String,
    pub threads_used: usize,
    pub timestamp: String

}

impl Record {

    pub fn new(url: String, file_name: String, file_size: u64, file_location: String, threads_used: usize) -> Record {

        Record {

            url,
            file_name,
            file_size,
            file_location,
            threads_used,
            timestamp: Local::now().to_string()

        }

    }

}

pub struct History {

    pub history_file_path: String

}

impl History {

    pub fn new(local_dir_path: String) -> Self {

        History {

            history_file_path: local_dir_path

        }

    }

    pub async fn add_record(&self, task: &HttpTask, config: &Config) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.toml");

        let mut history_file= File::options()
            .append(true)
            .open(file_path)
            .await
            .map_err(|e| RawstErr::FileError(e))?;

        let record= Record::new(
            task.url.clone(),
            task.filename.to_string(),
            task.get_downloaded(),
            config.download_path.clone(),
            config.threads
        );

        let download= Downloads {

            record: vec![record]

        };

        let toml= toml::to_string(&download).unwrap();

        let content= format!("{}\n", toml);

        history_file.write_all(&content.as_bytes()).await.map_err(|e| RawstErr::FileError(e))?;

        Ok(())

    }

    pub async fn get_history(&self) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.toml");

        let mut history_file= File::open(file_path)
            .await
            .map_err(|e| RawstErr::FileError(e))?;

        let mut content= String::new();

        history_file.read_to_string(&mut content).await.map_err(|e| RawstErr::FileError(e))?;

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