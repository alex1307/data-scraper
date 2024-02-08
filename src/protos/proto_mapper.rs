use crate::model::VehicleDataModel::{self, LinkId};

use super::vehicle_model::{
    BaseVehicleInfo, Consumption, DetailedVehicleInfo, Id, Price, VehicleChangeLogInfo,
};

impl From<VehicleDataModel::BaseVehicleInfo> for BaseVehicleInfo {
    fn from(base_vehicle_info: VehicleDataModel::BaseVehicleInfo) -> Self {
        Self {
            id: base_vehicle_info.id,
            source: base_vehicle_info.source,
            make: base_vehicle_info.make,
            model: base_vehicle_info.model,
            title: base_vehicle_info.title,
            currency: base_vehicle_info.currency.to_string(),
            price: base_vehicle_info.price.unwrap_or(0),
            millage: base_vehicle_info.millage.unwrap_or(0),
            month: base_vehicle_info.month.unwrap_or(0) as u32,
            year: base_vehicle_info.year as u32,
            engine: base_vehicle_info.engine.to_string(),
            gearbox: base_vehicle_info.gearbox.to_string(),
            cc: base_vehicle_info.cc,
            power_ps: base_vehicle_info.power_ps,
            power_kw: base_vehicle_info.power_kw,
        }
    }
}

impl From<VehicleDataModel::DetailedVehicleInfo> for DetailedVehicleInfo {
    fn from(detailed_vehicle_info: VehicleDataModel::DetailedVehicleInfo) -> Self {
        Self {
            id: detailed_vehicle_info.id,
            source: detailed_vehicle_info.source,
            phone: detailed_vehicle_info.phone,
            location: detailed_vehicle_info.location,
            view_count: detailed_vehicle_info.view_count,
            cc: detailed_vehicle_info.cc,
            fuel_consumption: detailed_vehicle_info.fuel_consumption,
            electric_drive_range: detailed_vehicle_info.electric_drive_range,
            equipment: detailed_vehicle_info.equipment,
            is_dealer: detailed_vehicle_info.is_dealer,
            seller_name: detailed_vehicle_info.seller_name,
        }
    }
}

impl From<VehicleDataModel::VehicleChangeLogInfo> for VehicleChangeLogInfo {
    fn from(source: VehicleDataModel::VehicleChangeLogInfo) -> Self {
        Self {
            id: source.id,
            source: source.source,
            published_on: source.published_on,
            last_modified_on: source.last_modified_on,
            last_modified_message: source.last_modified_message,
            days_in_sale: source.days_in_sale.unwrap_or(0),
            sold: source.sold,
            promoted: source.promoted,
        }
    }
}

impl From<VehicleDataModel::Price> for Price {
    fn from(source: VehicleDataModel::Price) -> Self {
        Self {
            id: source.id,
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
impl From<VehicleDataModel::Consumption> for Consumption {
    fn from(source: VehicleDataModel::Consumption) -> Self {
        Self {
            id: source.id,
            source: source.source,
            fuel_consumption: source.fuel_consumption.unwrap_or(0.0),
            make: source.make,
            model: source.model,
            year: source.year as u32,
            co2_emission: source.co2_emission,
            kw_consuption: source.kw_consuption.unwrap_or(0.0),
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
