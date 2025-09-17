use crate::model::Search::Search;
use crate::services::SearchBuilder::{CRAWLER_KEY, ID_KEY};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

use crate::utils::ConfigLoader::load_config; // add to Cargo.toml: walkdir = "2"

fn find_autouncle_configs(root: &str) -> Vec<String> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension() == Some(OsStr::new("yml")))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

pub fn scrape_all_autouncle() -> anyhow::Result<()> {
    let config_files = find_autouncle_configs("config/autouncle");
    if config_files.is_empty() {
        log::warn!("No autouncle configs found under config/autouncle");
        return Ok(());
    }

    for cfg_path in config_files {
        log::info!("==> Scraping with config: {}", cfg_path);
        let cfg = load_config(&cfg_path)?; // твоят ConfigLoader

        // 1) Построй базовия URL (+ optional brand сегмент)
        let mut base = cfg.base_url.clone();
        if let Some(brand) = &cfg.brand {
            if !brand.trim().is_empty() {
                // гарантирано добави /<brand> след .../cars_search
                if base.ends_with("/cars_search") {
                    base.push('/');
                    base.push_str(brand);
                } else if base.ends_with("/cars_search/") {
                    base.push_str(brand);
                }
            }
        }

        // 2) Paging loop
        let start = cfg.paging.start.max(1);
        let end = start + cfg.paging.max.saturating_sub(1);
        for page in start..=end {
            // сглоби query
            let mut pairs: Vec<(String, String)> = cfg
                .filters
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            if let Some(ref p) = cfg.paging.param {
                pairs.push((p.clone(), page.to_string()));
            }

            // encode
            let query = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs.iter().map(|(k, v)| (&k[..], &v[..])))
                .finish();

            let url = format!("{base}?{query}");
            log::info!("GET {}", url);

            // 3) Тук извикай твоя Autouncle scraping flow:
            // fetch_page(&url) -> parse -> sink
            // handle next/stop условия ако има няма резултати и т.н.
        }
    }

    Ok(())
}

pub fn build_searches(cfg_root: &str) -> Vec<Search> {
    let mut searches: Vec<Search> = Vec::new();

    // Determine market from the last path segment (e.g., config/autouncle/ch -> ch)
    let market = Path::new(cfg_root)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let source = format!("autouncle.{}", market);

    let config_files = find_autouncle_configs(cfg_root);
    if config_files.is_empty() {
        log::warn!("No autouncle configs found under {}", cfg_root);
        return searches;
    }

    for (file_idx, cfg_path) in config_files.iter().enumerate() {
        match load_config(cfg_path) {
            Ok(cfg) => {
                // base URL + optional /brand
                let mut base = cfg.base_url.clone();
                if let Some(brand) = &cfg.brand {
                    if !brand.trim().is_empty() {
                        if base.ends_with("/cars_search") {
                            base.push('/');
                            base.push_str(brand);
                        } else if base.ends_with("/cars_search/") {
                            base.push_str(brand);
                        }
                    }
                }
                // Ensure a trailing '?' so our Search::from appends pairs correctly
                if !base.ends_with('?') {
                    base.push('?');
                }

                // Single Search per YAML; pagination happens inside the scraper runtime
                let mut params: HashMap<String, String> = cfg.filters.clone();
                // DO NOT insert page here
                params.insert("url".to_string(), base.clone());
                params.insert(CRAWLER_KEY.to_string(), source.clone());
                // Stable ID based on market and file index
                let id_val = format!("{}-{}", market, file_idx + 1);
                params.insert(ID_KEY.to_string(), id_val);

                searches.push(Search::from(params));
            }
            Err(e) => {
                log::error!("Failed to load {}: {:?}", cfg_path, e);
            }
        }
    }

    searches
}
