use std::fmt;
use std::path::Path;

use reqwest::{header::HeaderMap, Url};

#[derive(Debug, Clone)]
pub struct FileName {
    pub stem: String,
    pub extension: String,
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.stem, self.extension)
    }
}

pub fn extract_filename_from_url(url: &str) -> FileName {
    let parsed_url = Url::parse(url).expect("Invalid Url");

    let filename = parsed_url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .unwrap();

    let path = Path::new(filename.last().unwrap());

    FileName {
        stem: path.file_stem().unwrap().to_str().unwrap().to_string(),
        extension: path.extension().unwrap().to_str().unwrap().to_string(),
    }
}

pub fn extract_filename_from_header(headers: &HeaderMap) -> Option<FileName> {
    let header_value = headers.get("Content-Disposition");

    match header_value {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap().split(';').collect();

            for part in parts {
                if let Some(filename) = part.trim().strip_prefix("filename=") {
                    let filename = filename.trim_matches('"');

                    let path = Path::new(filename);

                    let file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();

                    let extension = path.extension().unwrap().to_str().unwrap().to_string();

                    let structed_filename = FileName {
                        stem: file_stem,
                        extension,
                    };

                    return Some(structed_filename);
                }
            }

            None
        }
        None => None,
    }
}
