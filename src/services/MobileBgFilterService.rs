use std::collections::HashMap;

use crate::utils::ConfigLoader::{ScraperConfig, load_config};

/// Apply `{var}` placeholders in a string using provided variables
fn apply_vars(s: &str, vars: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{}}}", k), v);
    }
    out
}

/// Build a single mobile.bg URL for the given page using the YAML config
pub fn make_url(cfg: &ScraperConfig, page: u32) -> String {
    // base path (no trailing slash per our YAML)
    let mut base = cfg.base_url.clone();

    // variables (e.g. year_from, year_to)
    let vars: HashMap<String, String> = cfg.variables.clone().unwrap_or_else(|| HashMap::new());

    // optional path segments with placeholders (e.g. ot-{year_from}/do-{year_to})
    if let Some(segs) = &cfg.path_segments {
        for seg in segs {
            let filled = apply_vars(seg, &vars);
            if !filled.is_empty() {
                base.push('/');
                base.push_str(&filled);
            }
        }
    }

    // path-based paging (e.g. p-{page})
    if let Some(t) = &cfg.paging.r#type {
        if t == "path" {
            let pattern = cfg.paging.pattern.as_deref().unwrap_or("p-{page}");
            let filled = pattern.replace("{page}", &page.to_string());
            base.push('/');
            base.push_str(&filled);
        }
    }

    // query params = query_defaults + filters
    let mut pairs: Vec<(String, String)> = Vec::new();

    if let Some(defs) = &cfg.query_defaults {
        for (k, v) in defs {
            pairs.push((k.clone(), v.clone()));
        }
    }

    for (k, v) in &cfg.filters {
        pairs.push((k.clone(), v.clone()));
    }

    // Build query string: encode all params EXCEPT `extri` (mobile.bg expects raw '~')
    let mut query_parts: Vec<String> = Vec::new();
    for (k, v) in &pairs {
        if k == "extri" {
            // Append as-is
            query_parts.push(format!("{}={}", k, v));
        } else {
            // Standard percent-encode
            let encoded = url::form_urlencoded::Serializer::new(String::new())
                .append_pair(k, v)
                .finish();
            query_parts.push(encoded);
        }
    }
    let query = query_parts.join("&");

    let url = format!("{}?{}", base, query);
    log::info!("MobileBgFilterService URL: {}", url);

    url
}

/// Utility: generate all page URLs from the config file path
pub fn build_urls_from_config(cfg_path: &str) -> anyhow::Result<Vec<String>> {
    let cfg = load_config(cfg_path)?;
    let start = cfg.paging.start.max(1);
    let end = start + cfg.paging.max.saturating_sub(1);
    let mut out = Vec::new();
    for p in start..=end {
        out.push(make_url(&cfg, p));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_vars() {
        let mut vars = HashMap::new();
        vars.insert("year_from".to_string(), "2024".to_string());
        vars.insert("year_to".to_string(), "2025".to_string());
        let s = "ot-{year_from}/do-{year_to}";
        assert_eq!(apply_vars(s, &vars), "ot-2024/do-2025");
    }

    #[test]
    fn test_make_url_basic() {
        let mut filters = HashMap::new();
        filters.insert("extri".to_string(), "66~60~69~85~3~49".to_string());

        let cfg = ScraperConfig {
            base_url: "https://www.mobile.bg/obiavi/avtomobili-dzhipove".to_string(),
            brand: None,
            filters,
            paging: crate::utils::ConfigLoader::PagingConfig {
                start: 1,
                max: 2,
                param: None,
                r#type: Some("path".to_string()),
                pattern: Some("p-{page}".to_string()),
            },
            variables: Some(HashMap::from([
                ("year_from".to_string(), "2024".to_string()),
                ("year_to".to_string(), "2025".to_string()),
            ])),
            path_segments: Some(vec![
                "ot-{year_from}".to_string(),
                "do-{year_to}".to_string(),
            ]),
            query_defaults: Some(HashMap::from([
                ("nup".to_string(), "014".to_string()),
                ("pictonly".to_string(), "1".to_string()),
                ("privonly".to_string(), "2".to_string()),
            ])),
        };

        let url = make_url(&cfg, 2);
        assert!(
            url.starts_with(
                "https://www.mobile.bg/obiavi/avtomobili-dzhipove/ot-2024/do-2025/p-2?"
            )
        );
        assert!(
            url.contains("extri=66~60~69~85~3~49")
                || url.contains("extri=66%7E60%7E69%7E85%7E3%7E49")
        );
        assert!(url.contains("pictonly=1"));
        assert!(url.contains("privonly=2"));
        assert!(url.contains("nup=014"));
    }
}
