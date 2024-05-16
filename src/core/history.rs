use crate::core::errors::RawstErr;
use crate::core::task::HttpTask;
use crate::core::config::Config;

use std::path::Path;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use chrono::prelude::Local;
use tokio::{fs::File, io::AsyncWriteExt};
use toml;

#[derive(Debug, Clone, Deserialize, Serialize)]
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

struct History {

    pub history_file: File

}

impl History {

    pub async fn new(local_dir_path: String) -> Result<History, RawstErr> {

        let file_path= Path::new(&local_dir_path)
            .join("rawst")
            .join("history.toml")
            .display()
            .to_string();

        let history_file= File::open(file_path).await.map_err(|e| RawstErr::FileError(e))?;

        Ok(History {

            history_file

        })

    }

    pub async fn add_record(&mut self, task: HttpTask, config: Config) -> Result<(), RawstErr> {

        let record= Record::new(
            task.url,
            task.filename.to_string(),
            task.total_downloaded.load(Ordering::SeqCst),
            config.download_path,
            config.threads
        );

        let toml= toml::to_string(&record).unwrap();

        self.history_file.write_all(&toml.as_bytes()).await.map_err(|e| RawstErr::FileError(e))?;

        Ok(())

    }

}