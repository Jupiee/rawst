use std::fs;
use std::path::Path;
use std::path::PathBuf;

use chrono::prelude::Local;
use iri_string::types::IriString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::args::HistoryArgs;
use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::task::HttpTask;

pub async fn show_history(_args: HistoryArgs, config: Config) -> Result<(), RawstErr> {
    HistoryManager::new(config.config_path).get_history()
}

#[derive(Deserialize, Serialize)]
struct Downloads {
    record: Vec<Record>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Record {
    pub id: String,
    pub iri: IriString,
    pub file_name: PathBuf,
    pub file_size: u64,
    pub file_location: PathBuf,
    pub threads_used: usize,
    pub timestamp: String,
    pub status: String,
}

impl Record {
    pub fn new(
        id: String,
        iri: IriString,
        file_name: PathBuf,
        file_size: u64,
        file_location: PathBuf,
        threads_used: usize,
    ) -> Record {
        let current_time = Local::now();

        Record {
            id,
            iri,
            file_name,
            file_size,
            file_location,
            threads_used,
            timestamp: current_time.to_string(),
            status: "Pending".to_string(),
        }
    }
}

pub struct HistoryManager {
    pub history_file_path: PathBuf,
}

impl HistoryManager {
    pub fn new(local_dir_path: PathBuf) -> Self {
        HistoryManager {
            history_file_path: local_dir_path,
        }
    }

    pub fn add_record(&self, task: &HttpTask, config: &Config, id: String) -> Result<(), RawstErr> {
        let file_path = Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str: String = fs::read_to_string(&file_path).unwrap_or_else(|_| {
            panic!(
                "Couldn't read history database at '{}'.",
                file_path.display()
            )
        });

        // NOTE: This breaks on changes to Record.
        let mut records: Vec<Record> = serde_json::from_str(&json_str).unwrap_or_else(|_| {
            panic!(
                "Couldn't parse history database at '{}'.",
                file_path.display()
            )
        });

        let new_record = Record::new(
            id,
            task.iri.clone(),
            task.filename.clone(),
            task.content_length(),
            config.download_path.clone(),
            config.threads,
        );

        records.push(new_record);

        let new_json_str = serde_json::to_string_pretty(&records).unwrap();

        fs::write(file_path, new_json_str).map_err(RawstErr::FileError)?;

        Ok(())
    }

    pub fn update_record(&self, id: String) -> Result<(), RawstErr> {
        let file_path = Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let mut records: Vec<Record> =
            serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter_mut() {
            if record.id == id {
                record.status = "Completed".to_string();
            }
        }

        let new_json_str = serde_json::to_string_pretty(&records).unwrap();

        fs::write(file_path, new_json_str).map_err(RawstErr::FileError)?;

        Ok(())
    }

    pub fn get_history(&self) -> Result<(), RawstErr> {
        let file_path = Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(file_path).unwrap();

        let value: Value = serde_json::from_str(&json_str).expect("There are no downloads");

        let mut result = Vec::new();

        match value {
            Value::Array(arr) => {
                for item in arr.iter() {
                    let record = serde_json::from_value::<Record>(item.clone());
                    if let Ok(r) = record {
                        result.push(r);
                    }
                }
            }
            _ => {
                println!("Expected an array");
                return Ok(());
            }
        }

        for record in result.iter() {
            println!("Record\nid= {:?}\niri= {:?}\nfile name= {:?}\nfile size= {:?}\nfile location= {:?}\nthreads used= {:?}\ntimestamp= {:?}\nstatus= {:?}\n",
            record.id, record.iri, record.file_name, record.file_size, record.file_location, record.threads_used, record.timestamp, record.status);
        }

        Ok(())
    }

    pub fn get_recent_pending(&self) -> Result<Option<Record>, RawstErr> {
        let file_path = Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter().rev() {
            if record.status == "Pending" {
                return Ok(Some(record.to_owned()));
            }
        }

        Ok(None)
    }

    pub fn get_record(&self, id: &String) -> Result<Option<Record>, RawstErr> {
        let file_path = Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records {
            if record.id == *id {
                return Ok(Some(record));
            }
        }

        Ok(None)
    }
}
