use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileDeResults {
    #[serde(rename = "search")]
    pub search: Search,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Search {
    pub srp: SRP,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SRP {
    #[serde(rename = "data")]
    pub data: SRPData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SRPData {
    #[serde(rename = "metaData")]
    meta_data: MetaData,

    #[serde(rename = "aggregations")]
    aggregations: Aggregations,

    #[serde(rename = "searchResults")]
    search_result: SearchResult,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaData {
    title: String,
    headline: String,
    description: String,
    keywords: String,
    breadcrumbs: Vec<Breadcrumb>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Aggregations {
    st: Vec<KeyCount>,
    clim: Vec<KeyCount>,
    fe: Vec<KeyCount>,
    // Assuming `dm` is similar in structure to `st`, `clim`, etc.
    dm: Vec<KeyCount>,
    sr: Vec<KeyCount>,
    // Add other fields if needed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct KeyCount {
    key: String,
    count: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Breadcrumb {
    label: String,
    href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchResult {
    #[serde(rename = "searchId")]
    search_id: String,

    #[serde(rename = "numResultsTotal")]
    total_results: u32,

    #[serde(rename = "page")]
    page: u32,

    #[serde(rename = "numPages")]
    number_of_pages: u32,

    #[serde(rename = "obsSearchResultsCount")]
    obs_search_results_count: u32,

    #[serde(rename = "hasNextPage")]
    has_next_page: bool,

    #[serde(rename = "items")]
    items: Vec<SearchItem>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchItem {
    isEyeCatcher: Option<bool>,
    searchId: Option<String>,
    hasElectricEngine: Option<bool>,
    //financePlans: Vec<FinancePlan>,
    // sellerId: u64,
    priceRating: Option<PriceRating>,
    segment: Option<String>,
    title: Option<String>,
    vc: Option<String>,
    category: Option<String>,
    id: Option<u64>,
    // customDimensions: std::collections::HashMap<String, String>,
    // obsUrl: String,
    // relativeUrl: String,
    attributes: Option<Vec<String>>,
    contactInfo: Option<ContactInfo>,
    // //    previewImage: Image,
    // //    previewThumbnails: Vec<Image>,
    price: Option<Price>,
    // isFinancingAvailable: bool,
    make: Option<String>,
    model: Option<String>,
    #[serde(rename = "type")]
    modelType: Option<String>,
    // emailLink: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinancePlan {
    #[serde(rename = "type")]
    plan_type: String,
    url: String,
    showInGallery: bool,
    offer: FinanceOffer,
    budgetStatus: String,
    fallback: bool,
    downPayment: u32,
    loanDuration: u32,
    localized: FinanceLocalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinanceOffer {
    bankName: String,
    loanBroker: String,
    minMonthlyInstallment: u32,
    minInterestRateEffective: f64,
    minInterestRateNominal: f64,
    maxMonthlyInstallment: u32,
    maxInterestRateEffective: f64,
    maxInterestRateNominal: f64,
    localized: FinanceLocalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinanceLocalized {
    minMonthlyInstallment: String,
    minInterestRateEffective: String,
    minInterestRateNominal: String,
    maxMonthlyInstallment: String,
    maxInterestRateEffective: String,
    maxInterestRateNominal: String,
    disclaimer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PriceRating {
    rating: String,
    ratingLabel: String,
    thresholdLabels: Option<Vec<String>>,
    vehiclePriceOffset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Kba {
    hsn: String,
    tsn: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ContactInfo {
    typeLocalized: String,
    name: String,
    location: String,
    rating: Rating,
    hasContactPhones: bool,
    contactPhone: String,
    country: String,
    sellerType: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Rating {
    score: f64,
    count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Image {
    src: String,
    srcSet: String,
    alt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Price {
    gross: String,
    grossAmount: f64,
    grossCurrency: String,
    net: Option<String>,
    netAmount: Option<f64>,
    vat: Option<String>,
}
