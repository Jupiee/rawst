use crate::core::errors::RawstErr;

use std::path::Path;
use std::path::PathBuf;

use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub download_path: PathBuf,
    pub cache_path: PathBuf,
    pub config_path: PathBuf,
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        let user_dirs = UserDirs::new().unwrap();
        let base_dirs = BaseDirs::new().unwrap();

        let local_dir = base_dirs.data_local_dir();

        Config {
            download_path: user_dirs.download_dir().unwrap().to_path_buf(),
            cache_path: local_dir.join("rawst").join("cache").to_path_buf(),
            config_path: local_dir.to_path_buf(),
            threads: 1,
        }
    }
}

impl Config {
    pub async fn build() -> Result<Config, RawstErr> {
        let default = Config::default();

        let content = toml::to_string(&default).unwrap();

        let root_path = Path::new(&default.config_path).join("rawst");
        let config_file_path = &root_path.join("config.toml");
        let history_file_path = &root_path.join("history.json");

        create_dir_all(root_path)
            .await
            .expect("Failed to create config directory");
        create_dir_all(&default.cache_path)
            .await
            .expect("Failed to create cache directory");

        let mut config_file = File::create(config_file_path)
            .await
            .map_err(RawstErr::FileError)?;
        let mut history_file = File::create(history_file_path)
            .await
            .map_err(RawstErr::FileError)?;

        config_file
            .write_all(content.as_bytes())
            .await
            .map_err(RawstErr::FileError)?;
        history_file
            .write_all("[\n\n]".as_bytes())
            .await
            .map_err(RawstErr::FileError)?;

        Ok(default)
    }

    pub async fn load() -> Result<Config, RawstErr> {
        let config_dir = BaseDirs::new()
            .unwrap()
            .data_local_dir()
            .join("rawst")
            .join("config.toml");

        let mut file_content = String::new();

        let mut file = File::open(config_dir).await.map_err(RawstErr::FileError)?;

        file.read_to_string(&mut file_content)
            .await
            .map_err(RawstErr::FileError)?;

        let config: Config = toml::from_str(&file_content).unwrap();

        Ok(config)
    }
}
