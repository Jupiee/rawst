use directories::{UserDirs, BaseDirs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {

    pub download_path: String,
    pub cache_path: String,
    pub config_path: String,
    pub threads: usize

}

impl Default for Config {

    fn default() -> Self {
        
        let user_dirs= UserDirs::new().unwrap();
        let base_dirs= BaseDirs::new().unwrap();

        let local_dir= base_dirs.data_local_dir();

        let cache_path= local_dir.join("rawst").join("cache").display().to_string();

        return Config {
            
            download_path: user_dirs.download_dir().unwrap().display().to_string(),
            cache_path,
            config_path: local_dir.display().to_string(),
            threads: 1
        
        }

    }

}