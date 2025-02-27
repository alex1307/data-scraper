use crate::model::VehicleDataModel::{self, LinkId};

use super::vehicle_model::{BaseVehicleInfo, DetailedVehicleInfo, DownloadStatus, Id, Price};

impl From<VehicleDataModel::BaseVehicleInfo> for BaseVehicleInfo {
    fn from(source: VehicleDataModel::BaseVehicleInfo) -> Self {
        Self {
            id: format!("{}-{}", source.id, source.source),
            source: source.source,
            make: source.make,
            model: source.model,
            title: source.title,
            currency: source.currency.to_string(),
            price: source.price.unwrap_or(0),
            millage: source.mileage.unwrap_or(0),
            month: source.month.unwrap_or(0) as u32,
            year: source.year as u32,
            engine: source.engine.to_string(),
            gearbox: source.gearbox.to_string(),
            cc: source.cc,
            power_ps: source.power_ps,
            power_kw: source.power_kw,
            search_id: source.search_id,
            url: source.url,
        }
    }
}

impl From<VehicleDataModel::DetailedVehicleInfo> for DetailedVehicleInfo {
    fn from(source: VehicleDataModel::DetailedVehicleInfo) -> Self {
        Self {
            id: format!("{}-{}", source.id, source.source),
            source: source.source,
            location: source.location,
            equipment: source.equipment,
            seller_name: source.seller_name,
            seller_url: source.seller_url,
            consumption_fuel: source.consumption_fuel,
            consumption_kw: source.consumption_kw,
            co2: source.co2,
            range: source.range,
            days_in_sale: source.days_in_sale.unwrap_or(0),
        }
    }
}

impl From<VehicleDataModel::Price> for Price {
    fn from(source: VehicleDataModel::Price) -> Self {
        Self {
            id: format!("{}-{}", source.id, source.source),
            source: source.source,
            price: source.price,
            currency: source.currency.to_string(),
            estimated_price: source.estimated_price.unwrap_or(0),
            save_difference: source.save_difference,
            overpriced_difference: source.overpriced_difference,
            ranges: source.ranges.unwrap_or("[]".to_string()),
            rating: source.rating.unwrap_or("".to_string()),
            thresholds: source.thresholds,
        }
    }
}

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
