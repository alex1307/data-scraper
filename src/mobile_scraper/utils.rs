use std::fs;
use std::io::Result;

pub fn read_file_from(dirname: &str, filename: &str) -> Result<String> {
    let path = format!("{}/{}", dirname, filename);
    fs::read_to_string(path)
}
