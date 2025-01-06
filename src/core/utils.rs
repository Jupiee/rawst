use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::fs;

use iri_string::types::IriString;
use serde_json::Value;
use reqwest::header::HeaderMap;

use super::errors::RawstErr;

pub fn headers_from_file(input: PathBuf) -> Result<HashMap<String, String>, RawstErr> {

    let file_content = fs::read_to_string(input).map_err(|err| RawstErr::FileError(err))?;
    let json: Value = serde_json::from_str(&file_content).map_err(|_| RawstErr::InvalidArgs)?;
    let mut header_map = HashMap::new();

    // Iterate over the key-value pairs in the JSON object
    if let Value::Object(map) = json {
        for (key, value) in map {
            if let Value::String(value_str) = value {
                // Convert key and value to HeaderName and HeaderValue, and insert them
                header_map.insert(key, value_str);
            }
        }
    } else {
        return Err(RawstErr::InvalidArgs);
    }

    Ok(header_map)
}

pub fn extract_filename_from_url(iri: &IriString) -> PathBuf {
    // "http://example.com/path/to/file.tar.gz?query#frag"
    // => "/path/to/file.tar.gz"
    let file_name = iri.path_str();
    if file_name == "/" || !file_name.contains('.') {
        let mut domain = iri.authority_str().unwrap().to_owned();
        domain.push_str(".html");
        return PathBuf::from(domain)
    
    }
    let full_path = PathBuf::from(file_name);
    // => "file.tar.gz"
    let path = PathBuf::from(full_path.file_name().unwrap());

    assert!(path.is_relative());

    path
}

pub fn extract_filename_from_header(headers: &HeaderMap) -> Option<PathBuf> {
    let header_value = headers.get("Content-Disposition");

    match header_value {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap().split(';').collect();

            for part in parts {
                if let Some(filename) = part.trim().strip_prefix("filename=") {
                    let path = PathBuf::from(filename.trim_matches('"'));
                    assert!(path.is_relative());
                    return Some(path);
                }
            }

            None
        }
        None => None,
    }
}

pub fn chunk_file_name(filename: &Path, part: usize) -> PathBuf {
    filename.with_added_extension(format!("part-{}.tmp", part))
}
