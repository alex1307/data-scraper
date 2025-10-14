use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaDataKafkaRecord {
    /// Уникален идентификатор на филтъра (стабилен във времето)
    pub filter_id: String,

    /// Източник – mobile.de, autouncle.dk и т.н.
    pub source: String,

    /// Flow файлът, който е стартирал скрейпъра
    pub flow: String,

    /// Тип страница (srp, detail, и т.н.)
    pub page_type: Option<String>,

    /// Пълният URL, от който са извлечени резултатите
    pub url: String,

    /// JSON обект с филтри (make, model, price_min, ...)
    pub filters: Option<serde_json::Value>,

    /// JSON масив с екстри (["4x4", "hud", ...])
    pub equipment: Vec<String>,
}

#[derive(Debug, FromRow)]
pub struct MetaDataDBRecord {
    /// Уникален идентификатор на филтъра (стабилен във времето)
    pub filter_id: String,

    /// Източник – mobile.de, autouncle.dk и т.н.
    pub source: String,

    /// Flow файлът, който е стартирал скрейпъра
    pub flow: String,

    /// Тип страница (srp, detail, и т.н.)
    pub page_type: Option<String>,

    /// Пълният URL, от който са извлечени резултатите
    pub url: String,

    /// JSON обект с филтри (make, model, price_min, ...)
    pub filters: Option<serde_json::Value>,

    /// JSON масив с екстри (["4x4", "hud", ...])
    pub equipment: String,

    pub last_run_on: chrono::NaiveDate,
}

impl From<MetaDataKafkaRecord> for MetaDataDBRecord {
    #[inline]
    fn from(record: MetaDataKafkaRecord) -> Self {
        MetaDataDBRecord {
            filter_id: record.filter_id,
            source: record.source,
            flow: record.flow,
            page_type: record.page_type,
            url: record.url,
            filters: record.filters,
            equipment: record.equipment.join(","),
            last_run_on: chrono::Utc::now().naive_utc().date(),
        }
    }
}
