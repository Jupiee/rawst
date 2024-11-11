use std::path::PathBuf;

use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::core::errors::RawstErr;

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

    /// The default download path ($XDG_DOWNLOAD_DIR: ~/Downloads/)
    pub download_dir: PathBuf,

    // Download parameters
    // -------------------
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        let user_dirs = UserDirs::new().unwrap();
        let base_dirs = BaseDirs::new().unwrap();

        let config_dir = base_dirs.config_dir().join("rawst").to_path_buf();
        let config_file_path = config_dir.join("config.toml");

        let cache_dir = base_dirs.cache_dir().join("rawst").to_path_buf();
        let history_file_path = cache_dir.join("history.json");

        Config {
            config_dir,
            config_file_path,
            cache_dir,
            history_file_path,
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

        let config_str = fs::read_to_string(config_file_path)
            .await
            .map_err(RawstErr::FileError)?;

        let config: Config = toml::from_str(&config_str).unwrap();

        Ok(config)
    }

    pub async fn initialise_files(&self) -> Result<(), RawstErr> {
        // Configuration
        {
            fs::create_dir_all(&self.config_dir)
                .await
                .expect("Failed to create config directory");

            let mut config_file = fs::File::create(&self.config_file_path)
                .await
                .map_err(RawstErr::FileError)?;

            let config_toml = toml::to_string(&self).unwrap();
            config_file
                .write_all(config_toml.as_bytes())
                .await
                .map_err(RawstErr::FileError)?;
        }

        // Cache
        {
            fs::create_dir_all(&self.cache_dir)
                .await
                .expect("Failed to create cache directory");
            let mut history_file = fs::File::create(&self.history_file_path)
                .await
                .map_err(RawstErr::FileError)?;
            history_file
                .write_all("[\n\n]".as_bytes())
                .await
                .map_err(RawstErr::FileError)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_are_indeed_directories() {
        let config = Config::default();

        assert!(config.config_dir.is_dir());
        assert!(config.config_file_path.is_file());

        assert!(config.cache_dir.is_dir());
        assert!(config.history_file_path.is_file());

        assert!(config.download_dir.is_dir());
    }
}
