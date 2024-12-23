use std::path::Path;
use std::path::PathBuf;

use iri_string::types::IriString;
use reqwest::header::HeaderMap;

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
                    println!("path: {:?}", path);
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
