use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct PagingConfig {
    pub param: String,
    pub start: u32,
    pub max: u32,
}

#[derive(Debug, Deserialize)]
pub struct ScraperConfig {
    pub base_url: String,
    pub brand: Option<String>,
    pub filters: HashMap<String, String>,
    pub paging: PagingConfig,
}

pub fn load_config(path: &str) -> anyhow::Result<ScraperConfig> {
    let f = std::fs::File::open(path)?;
    let cfg: ScraperConfig = serde_yaml::from_reader(f)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{File, remove_file};
    use std::io::Write;

    #[test]
    fn test_load_valid_config() {
        let file_path = env::temp_dir().join("test_config.yml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "base_url: 'https://example.com'").unwrap();
        writeln!(file, "brand: 'Mercedes'").unwrap();
        writeln!(file, "filters:").unwrap();
        writeln!(file, "  s[has_4wd]: 'true'").unwrap();
        writeln!(file, "paging:").unwrap();
        writeln!(file, "  param: 'page'").unwrap();
        writeln!(file, "  start: 1").unwrap();
        writeln!(file, "  max: 2").unwrap();

        let cfg = load_config(file_path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.base_url, "https://example.com");
        assert_eq!(cfg.brand.unwrap(), "Mercedes");
        assert_eq!(cfg.filters.get("s[has_4wd]").unwrap(), "true");
        assert_eq!(cfg.paging.param, "page");
        assert_eq!(cfg.paging.start, 1);
        assert_eq!(cfg.paging.max, 2);

        let _ = remove_file(file_path);
    }

    #[test]
    fn test_load_invalid_config() {
        let file_path = env::temp_dir().join("invalid_config.yml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "not: valid: yaml").unwrap();

        let result = load_config(file_path.to_str().unwrap());
        assert!(result.is_err());

        let _ = remove_file(file_path);
    }
}
