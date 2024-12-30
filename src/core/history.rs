use std::fs;
use std::path::PathBuf;

use chrono::prelude::Local;
use iri_string::types::IriString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::task::HttpTask;

pub async fn show_history(config: Config) -> Result<(), RawstErr> {
    HistoryManager::new(config.history_file_path).get_history()
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
    pub file_path: PathBuf,
}

impl HistoryManager {
    pub fn new(file_path: PathBuf) -> Self {
        HistoryManager { file_path }
    }

    pub fn add_record(&self, task: &HttpTask, config: &Config, id: String) -> Result<(), RawstErr> {
        // TODO: Using jsonl would mean we can simply append to the history file.
        let json_str: String = fs::read_to_string(&self.file_path).unwrap_or_else(|_| {
            panic!(
                "Couldn't read history database at '{}'.",
                self.file_path.display()
            )
        });

        let mut records: Vec<Record> = serde_json::from_str(&json_str).unwrap_or_else(|_| {
            panic!(
                "Couldn't parse history database at '{}'.",
                self.file_path.display()
            )
        });

        let new_record = Record::new(
            id,
            task.iri.clone(),
            task.filename.clone(),
            task.content_length(),
            config.download_dir.clone(),
            config.threads,
        );

        records.push(new_record);

        let new_json_str = serde_json::to_string_pretty(&records).unwrap();

        fs::write(&self.file_path, new_json_str).map_err(RawstErr::FileError)?;

        Ok(())
    }

    pub fn update_record(&self, id: String) -> Result<(), RawstErr> {
        let json_str = fs::read_to_string(&self.file_path).unwrap();

        let mut records: Vec<Record> =
            serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter_mut() {
            if record.id == id {
                record.status = "Completed".to_string();
            }
        }

        let new_json_str = serde_json::to_string_pretty(&records).unwrap();

        fs::write(&self.file_path, new_json_str).map_err(RawstErr::FileError)?;

        Ok(())
    }

    pub fn get_history(&self) -> Result<(), RawstErr> {
        let json_str = fs::read_to_string(&self.file_path).unwrap();

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
            record.id, record.iri.to_string(), record.file_name, record.file_size, record.file_location, record.threads_used, record.timestamp, record.status);
        }

        Ok(())
    }

    pub fn get_recent_pending(&self) -> Result<Option<Record>, RawstErr> {
        let json_str = fs::read_to_string(&self.file_path).unwrap();

        let records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter().rev() {
            if record.status == "Pending" {
                return Ok(Some(record.to_owned()));
            }
        }

        Ok(None)
    }

    pub fn get_record(&self, id: &String) -> Result<Option<Record>, RawstErr> {
        let json_str = fs::read_to_string(&self.file_path).unwrap();

        let records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records {
            if record.id == *id {
                return Ok(Some(record));
            }
        }

        Ok(None)
    }
}
