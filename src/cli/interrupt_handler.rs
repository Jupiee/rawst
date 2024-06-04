use crate::core::task::HttpTask;
use crate::core::config::Config;
use crate::core::history::HistoryManager;

use std::process::exit;
use std::sync::atomic::Ordering;

use ctrlc;

pub enum TaskType {

    List(Vec<HttpTask>),
    Single(HttpTask)

}

pub fn create_interrupt_handler(task_type: TaskType, config: Config) {

    ctrlc::set_handler(move || {

        let history_manager= HistoryManager::new(config.config_path.clone());

        match &task_type {

            TaskType::List(task_list) => {

                for task in task_list {

                    println!("\nReceived Interrupt for {}, downloaded bytes: {}", task.filename, task.total_downloaded.load(Ordering::SeqCst));

                }

            },
            TaskType::Single(task) => {

                history_manager.add_record(task, &config).unwrap();

                println!("\nReceived Interrupt for {}, downloaded bytes: {}", task.filename, task.total_downloaded.load(Ordering::SeqCst));

            }

        }

        exit(1);

    }).expect("Error setting Ctrl-C handler");

}