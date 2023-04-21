use std::fs;
use std::io::Result;

pub fn read_file_from(dirname: &str, filename: &str) -> Result<String> {
    let path = format!("{}/{}", dirname, filename);
    fs::read_to_string(path)
}

pub fn extract_integers(s: &str) -> Vec<u32> {
    s.split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}
