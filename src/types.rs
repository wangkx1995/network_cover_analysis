use std::collections::HashMap;
use geo_types::Geometry;

#[derive(Debug, Clone)]
pub struct RawRecord {
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SpatialFeature {
    pub row_id: usize,
    pub geometry: Geometry<f64>,
    pub raw: RawRecord,
}

#[derive(Debug, Clone)]
pub struct MergedCoverage {
    pub coverage_m: f64,
    pub coverage_tu: f64,
    pub item_num_m: f64,
    pub item_num_tu: f64,
    pub coverage_m_2: f64,
    pub coverage_tu_2: f64,
    pub item_num_m_2: f64,
    pub item_num_tu_2: f64,
}
