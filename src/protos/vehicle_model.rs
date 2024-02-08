#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseVehicleInfo {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub make: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub model: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub title: ::prost::alloc::string::String,
    /// Assuming Currency is a defined enum or message
    #[prost(string, tag = "6")]
    pub currency: ::prost::alloc::string::String,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(uint32, tag = "7")]
    pub price: u32,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(uint32, tag = "8")]
    pub millage: u32,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(uint32, tag = "9")]
    pub month: u32,
    #[prost(uint32, tag = "10")]
    pub year: u32,
    /// Assuming Engine is a defined message
    #[prost(string, tag = "11")]
    pub engine: ::prost::alloc::string::String,
    /// Assuming Gearbox is a defined message
    #[prost(string, tag = "12")]
    pub gearbox: ::prost::alloc::string::String,
    #[prost(uint32, tag = "13")]
    pub cc: u32,
    #[prost(uint32, tag = "14")]
    pub power_ps: u32,
    #[prost(uint32, tag = "15")]
    pub power_kw: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VehicleChangeLogInfo {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub published_on: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub last_modified_on: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub last_modified_message: ::prost::alloc::string::String,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(uint32, tag = "6")]
    pub days_in_sale: u32,
    #[prost(bool, tag = "7")]
    pub sold: bool,
    #[prost(bool, tag = "8")]
    pub promoted: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetailedVehicleInfo {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub phone: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub location: ::prost::alloc::string::String,
    #[prost(uint32, tag = "5")]
    pub view_count: u32,
    #[prost(uint32, tag = "6")]
    pub cc: u32,
    #[prost(double, tag = "7")]
    pub fuel_consumption: f64,
    #[prost(double, tag = "8")]
    pub electric_drive_range: f64,
    #[prost(uint64, tag = "9")]
    pub equipment: u64,
    #[prost(bool, tag = "10")]
    pub is_dealer: bool,
    #[prost(string, tag = "11")]
    pub seller_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Price {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(uint32, tag = "3")]
    pub estimated_price: u32,
    #[prost(uint32, tag = "4")]
    pub price: u32,
    /// Assuming Currency is defined in enums.proto
    #[prost(string, tag = "5")]
    pub currency: ::prost::alloc::string::String,
    #[prost(uint32, tag = "6")]
    pub save_difference: u32,
    #[prost(uint32, tag = "7")]
    pub overpriced_difference: u32,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(string, tag = "8")]
    pub ranges: ::prost::alloc::string::String,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(string, tag = "9")]
    pub rating: ::prost::alloc::string::String,
    /// For 'thresholds', protobuf doesn't support Vec directly. You might consider using repeated field.
    #[prost(uint32, repeated, tag = "10")]
    pub thresholds: ::prost::alloc::vec::Vec<u32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Consumption {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub make: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub model: ::prost::alloc::string::String,
    #[prost(uint32, tag = "5")]
    pub year: u32,
    #[prost(uint32, tag = "6")]
    pub co2_emission: u32,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(float, tag = "7")]
    pub fuel_consumption: f32,
    /// Optional in Rust, so no 'optional' keyword needed in proto3
    #[prost(float, tag = "8")]
    pub kw_consuption: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Id {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub source: ::prost::alloc::string::String,
}
