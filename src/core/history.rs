use crate::core::errors::RawstErr;
use crate::core::task::HttpTask;
use crate::core::config::Config;

use std::path::Path;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::prelude::Local;


#[derive(Deserialize, Serialize)]
struct Downloads {

    record: Vec<Record>

}

#[derive(Deserialize, Serialize, Debug)]
struct Record {

    pub id: String,
    pub url: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_location: String,
    pub threads_used: usize,
    pub timestamp: String,
    pub status: String

}

impl Record {

    pub fn new(id: String, url: String, file_name: String, file_size: u64,  file_location: String, threads_used: usize) -> Record {

        let current_time= Local::now();

        Record {

            id,
            url,
            file_name,
            file_size,
            file_location,
            threads_used,
            timestamp: current_time.to_string(),
            status: "Pending".to_string()

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

    pub fn add_record(&self, task: &HttpTask, config: &Config, id: String) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let mut records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        let new_record= Record::new(
            id,
            task.url.clone(),
            task.filename.to_string(),
            task.content_length(),
            config.download_path.clone(),
            config.threads
        );

        records.push(new_record);

        let new_json_str= serde_json::to_string_pretty(&records).unwrap();

        fs::write(file_path, new_json_str).map_err(|e| RawstErr::FileError(e))?;

        Ok(())

    }

    pub fn update_record(&self, id: String) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let mut records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter_mut() {

            if record.id == id {
                
                record.status= "Completed".to_string();

            }

        }

        let new_json_str= serde_json::to_string_pretty(&records).unwrap();

        fs::write(file_path, new_json_str).map_err(|e| RawstErr::FileError(e))?;

        Ok(())

    }

    pub fn get_history(&self) -> Result<(), RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(file_path).unwrap();

        let value: Value = serde_json::from_str(&json_str).expect("There are no downloads");
        
        let mut result = Vec::new();
        
        match value {

            Value::Array(arr) => {

                for item in arr.iter() {

                    let record= serde_json::from_value::<Record>(item.clone());

                    if record.is_ok() {

                        result.push(record.unwrap());

                    }

                }

            },
            _ => {

                println!("Expected an array");
                return Ok(())

            }
        }

        for record in result.iter() {

            println!("Record\nid= {:?}\nurl= {:?}\nfile name= {:?}\nfile size= {:?}\nfile location= {:?}\nthreads used= {:?}\ntimestamp= {:?}\nstatus= {:?}\n",
            record.id, record.url, record.file_name, record.file_size, record.file_location, record.threads_used, record.timestamp, record.status);

        }

        Ok(())

    }

    pub fn get_record(&self, id: &String) -> Result<Option<(String, usize, String, String)>, RawstErr> {

        let file_path= Path::new(&self.history_file_path)
            .join("rawst")
            .join("history.json");

        let json_str = fs::read_to_string(&file_path).unwrap();

        let mut records: Vec<Record> = serde_json::from_str(&json_str).expect("There are no downloads");

        for record in records.iter_mut() {

            if record.id == *id {
                
                let url= record.url.clone();
                let threads= record.threads_used;
                let filename= record.file_name.clone();
                let status= record.status.clone();

                return Ok(Some((url, threads, filename, status)))

            }

        }

        Ok(None)

    }

}