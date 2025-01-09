use std::path::PathBuf;

use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::core::errors::RawstErr;

pub async fn edit_config(mut config: Config) -> Result<(), RawstErr> {

    let stdin_handle = std::io::stdin();

    let mut cache_dir = String::new();
    println!("Enter cache directory path: (default: {}) leave blank to keep the default", config.cache_dir.display());
    stdin_handle.read_line(&mut cache_dir).unwrap();
    cache_dir = cache_dir.trim().to_string();

    if !cache_dir.is_empty() {

        let cache_dir_pathbuf = PathBuf::from(&cache_dir);
        config.cache_dir = cache_dir_pathbuf;

    }

    let mut log_dir = String::new();
    println!("Enter log directory path: (default: {}) leave blank to keep the default", config.log_dir.display());
    stdin_handle.read_line(&mut log_dir).unwrap();
    log_dir = log_dir.trim().to_string();

    if !log_dir.is_empty() {

        let log_dir_pathbuf = PathBuf::from(&log_dir);
        config.log_dir = log_dir_pathbuf;

    }

    let mut download_dir = String::new();
    println!("Enter download directory path: (default: {}) leave blank to keep the default", config.download_dir.display());
    stdin_handle.read_line(&mut download_dir).unwrap();
    download_dir = download_dir.trim().to_string();

    if !download_dir.is_empty() {

        let download_dir_pathbuf = PathBuf::from(&download_dir);
        config.download_dir = download_dir_pathbuf;

    }

    let mut threads = String::new();
    println!("Enter number of threads: (default: {}) leave blank to keep the default", config.threads);
    stdin_handle.read_line(&mut threads).unwrap();
    threads = threads.trim().to_string();

    if !threads.is_empty() {

        let threads_usize = threads.parse::<usize>().unwrap();
        config.threads = threads_usize

    }

    let config_toml = toml::to_string(&config).unwrap();

    let mut config_file = fs::File::options()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&config.config_file_path)
        .await
        .map_err(RawstErr::FileError)?;

    config_file
        .write_all(config_toml.trim().as_bytes())
        .await
        .map_err(RawstErr::FileError)?;

    config_file.flush().await.map_err(RawstErr::FileError)?;

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // XDG directories
    // ---------------
    // - Spec: https://specifications.freedesktop.org/basedir-spec/latest/
    // - Defaults summary: https://gist.github.com/roalcantara/107ba66dfa3b9d023ac9329e639bc58c
    // - directories lib: https://docs.rs/directories/latest/directories/
    /// The config directory ($XDG_CONFIG_HOME/rawst/: ~/.config/rawst/)
    pub config_dir: PathBuf,
    /// The main config file path ($XDG_CONFIG_HOME/rawst/config.toml: ~/.config/rawst/config.toml)
    pub config_file_path: PathBuf,

    /// The cache directory ($XDG_CACHE_HOME/rawst/: ~/.cache/rawst/)
    pub cache_dir: PathBuf,
    /// The history file path ($XDG_CONFIG_HOME/rawst/history.json: ~/.config/rawst/history.json)
    pub history_file_path: PathBuf,
    /// The history file path ($XDG_CONFIG_HOME/rawst/logs/: ~/.config/rawst/logs/)
    pub log_dir: PathBuf,

    /// The default download path ($XDG_DOWNLOAD_DIR: ~/Downloads/)
    pub download_dir: PathBuf,

    // Download parameters
    // -------------------
    pub threads: usize,
}

impl Config {
    pub fn log_file_path(&self) -> PathBuf {
        let td = format_timedate(chrono::Local::now());
        let thread_id = std::thread::current().id().as_u64();
        let run_id = format!("{}-{}", td, thread_id);

        // ~/.cache/rawst/logs/2024-12-31_23:59:59_-07:00-1.log
        self.log_dir.join(format!("{}.log", run_id))
    }
}

fn format_timedate(dt: chrono::DateTime<chrono::Local>) -> String {
    // "2024-12-31_23:59:59"
    format!("{}", dt.format("%Y-%m-%d_%Hh-%Mm-%Ss"))
}

impl Default for Config {
    fn default() -> Self {
        let user_dirs = UserDirs::new().unwrap();
        let base_dirs = BaseDirs::new().unwrap();

        // ~/.config/rawst/
        let config_dir = base_dirs.config_dir().join("rawst").to_path_buf();
        // ~/.config/rawst/config.toml
        let config_file_path = config_dir.join("config.toml");

        // ~/.cache/rawst/
        let cache_dir = base_dirs.cache_dir().join("rawst").to_path_buf();
        // ~/.cache/rawst/history.json
        let history_file_path = cache_dir.join("history.json");
        // ~/.cache/rawst/logs/
        let log_dir = cache_dir.join("logs").to_path_buf();

        Config {
            config_dir,
            config_file_path,
            cache_dir,
            history_file_path,
            log_dir,
            download_dir: user_dirs.download_dir().unwrap().to_path_buf(),

            threads: 1,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Config, RawstErr> {
        let base_dirs = BaseDirs::new().unwrap();
        let config_dir = base_dirs.config_dir().join("rawst").to_path_buf();
        let config_file_path = config_dir.join("config.toml");

        log::debug!("Loading config from '{config_file_path:?}'");
        let config_str = fs::read_to_string(config_file_path)
            .await
            .map_err(RawstErr::FileError)?;

        let config: Config = toml::from_str(&config_str).expect("Failed to read config file");

        Ok(config)
    }

    pub async fn initialise_files(&self) -> Result<(), RawstErr> {
        log::debug!("Creating new configuration");
        println!("Creating new configuration");
        // Configuration

        log::trace!("  Creating configuration files");
        println!("  Creating configuration files");
        {
            log::trace!("Creating directory {:?}", &self.config_dir);
            fs::create_dir_all(&self.config_dir)
                .await
                .expect("Failed to create config directory");

            log::trace!("Creating file '{:?}'", &self.config_file_path);
            let mut config_file = fs::File::create(&self.config_file_path)
                .await
                .map_err(RawstErr::FileError)?;

            let config_toml = toml::to_string(&self).unwrap();
            log::trace!("Writing file {:?}", &self.config_file_path);
            config_file
                .write_all(config_toml.as_bytes())
                .await
                .map_err(RawstErr::FileError)?;
        }

        log::trace!("  Creating cache files");
        println!("  Creating cache files");
        {
            log::trace!("Creating directory '{:?}'", &self.cache_dir);
            fs::create_dir_all(&self.cache_dir)
                .await
                .expect("Failed to create cache directory");
            log::trace!("Creating file {:?}", &self.history_file_path);
            let mut history_file = fs::File::create(&self.history_file_path)
                .await
                .map_err(RawstErr::FileError)?;
            log::trace!("Writing empty list to {:?}", &self.history_file_path);
            println!("Writing empty list to {:?}", &self.history_file_path);
            history_file
                .write_all("[\n\n]".as_bytes())
                .await
                .map_err(RawstErr::FileError)?;

            println!("  Creating logs directory");
            {
                log::trace!("Creating directory '{:?}'", &self.log_dir);
                fs::create_dir_all(&self.log_dir)
                    .await
                    .expect("Failed to create log directory");
            }
        }

        Ok(())
    }
}
