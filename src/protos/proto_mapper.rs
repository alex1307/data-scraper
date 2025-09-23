use crate::model::VehicleDataModel::{self, LinkId};

use super::vehicle_model::{DownloadStatus, Id, Vehicle};

impl From<VehicleDataModel::DownloadStatus> for DownloadStatus {
    fn from(source: VehicleDataModel::DownloadStatus) -> Self {
        Self {
            id: source.id,
            source: source.source,
            url: source.url,
            listed: source.listed,
            actual: source.actual,
            hash: source.hash,
        }
    }
}

impl From<LinkId> for Id {
    fn from(link_id: LinkId) -> Self {
        Self {
            id: link_id.id,
            source: link_id.source,
        }
    }
}

impl From<VehicleDataModel::Vehicle> for Vehicle {
    fn from(source: VehicleDataModel::Vehicle) -> Self {
        Self {
            id: source.id,
            source: source.source,
            make: source.make,
            model: source.model,
            title: source.title,
            currency: source.currency.to_string(),
            price: source.price,
            mileage: source.mileage,
            year: source.year as u32,
            engine: source.engine.to_string(),
            gearbox: source.gearbox.to_string(),
            cc: source.cc.unwrap_or_default(),
            power_ps: source.power_ps,
            power_kw: source.power_kw,
            equipment: source.equipment.unwrap_or_default(),
            location: source.location.unwrap_or_default(),
            seller_name: source.seller_name.unwrap_or_default(),
            seller_url: source.seller_url.unwrap_or_default(),
            consumption_fuel: source.consumption_fuel.unwrap_or_default(),
            consumption_kw: source.consumption_kw.unwrap_or_default(),
            co2: source.co2.unwrap_or_default(),
            range: source.range.unwrap_or_default(),
            days_in_sale: source.days_in_sale.unwrap_or_default(),
            estimated_price: source.estimated_price.unwrap_or_default(),
            url: source.url,
            ranges: source.ranges.unwrap_or_default(),
            rating: source.rating.unwrap_or_default(),
            thresholds: source.thresholds,
        }
    }
}
