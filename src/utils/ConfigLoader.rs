use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct PagingConfig {
    pub start: u32,
    pub max: u32,
    // query-based paging: e.g., param = "page"
    pub param: Option<String>,
    // path-based paging: e.g., type = "path", pattern = "p-{page}"
    pub r#type: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScraperConfig {
    pub base_url: String,
    pub brand: Option<String>,
    pub filters: HashMap<String, String>,
    pub paging: PagingConfig,
    pub variables: Option<HashMap<String, String>>, // e.g., year_from, year_to
    pub path_segments: Option<Vec<String>>,         // e.g., ["ot-{year_from}", "do-{year_to}"]
    pub query_defaults: Option<HashMap<String, String>>, // e.g., nup, pictonly, privonly
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
        writeln!(file, "  start: 1").unwrap();
        writeln!(file, "  max: 2").unwrap();
        writeln!(file, "  param: 'page'").unwrap();

        let cfg = load_config(file_path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.base_url, "https://example.com");
        assert_eq!(cfg.brand.as_deref(), Some("Mercedes"));
        assert_eq!(cfg.filters.get("s[has_4wd]").unwrap(), "true");
        assert_eq!(cfg.paging.start, 1);
        assert_eq!(cfg.paging.max, 2);
        assert_eq!(cfg.paging.param.as_deref(), Some("page"));
        assert!(cfg.paging.r#type.is_none());
        assert!(cfg.paging.pattern.is_none());
        assert!(cfg.variables.is_none());
        assert!(cfg.path_segments.is_none());
        assert!(cfg.query_defaults.is_none());

        let _ = remove_file(file_path);
    }

    #[test]
    fn test_load_config_with_path_paging_and_vars() {
        let file_path = env::temp_dir().join("test_mobilebg_config.yml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "base_url: 'https://www.mobile.bg/obiavi/avtomobili-dzhipove'"
        )
        .unwrap();
        writeln!(file, "filters:").unwrap();
        writeln!(file, "  extri: '66~60~69~85~3~49'").unwrap();
        writeln!(file, "paging:").unwrap();
        writeln!(file, "  start: 1").unwrap();
        writeln!(file, "  max: 3").unwrap();
        writeln!(file, "  type: 'path'").unwrap();
        writeln!(file, "  pattern: 'p-{{page}}'").unwrap();
        writeln!(file, "variables:").unwrap();
        writeln!(file, "  year_from: '2024'").unwrap();
        writeln!(file, "  year_to: '2025'").unwrap();
        writeln!(file, "path_segments:").unwrap();
        writeln!(file, "  - 'ot-{{year_from}}'").unwrap();
        writeln!(file, "  - 'do-{{year_to}}'").unwrap();
        writeln!(file, "query_defaults:").unwrap();
        writeln!(file, "  pictonly: '1'").unwrap();
        writeln!(file, "  privonly: '2'").unwrap();
        writeln!(file, "  nup: '014'").unwrap();

        let cfg = load_config(file_path.to_str().unwrap()).unwrap();
        assert_eq!(
            cfg.base_url,
            "https://www.mobile.bg/obiavi/avtomobili-dzhipove"
        );
        assert_eq!(cfg.paging.start, 1);
        assert_eq!(cfg.paging.max, 3);
        assert_eq!(cfg.paging.r#type.as_deref(), Some("path"));
        assert_eq!(cfg.paging.pattern.as_deref(), Some("p-{page}"));
        assert_eq!(cfg.paging.param.as_deref(), None);
        assert_eq!(
            cfg.variables
                .as_ref()
                .unwrap()
                .get("year_from")
                .map(String::as_str),
            Some("2024")
        );
        assert_eq!(
            cfg.variables
                .as_ref()
                .unwrap()
                .get("year_to")
                .map(String::as_str),
            Some("2025")
        );
        let segs = cfg.path_segments.as_ref().unwrap();
        assert_eq!(
            segs,
            &vec!["ot-{year_from}".to_string(), "do-{year_to}".to_string()]
        );
        let qd = cfg.query_defaults.as_ref().unwrap();
        assert_eq!(qd.get("pictonly").map(String::as_str), Some("1"));
        assert_eq!(qd.get("privonly").map(String::as_str), Some("2"));
        assert_eq!(qd.get("nup").map(String::as_str), Some("014"));

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
