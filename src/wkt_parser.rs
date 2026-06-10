use anyhow::Result;
use wkt::TryFromWkt;
use geo_types::Geometry;
use crate::types::{RawRecord, SpatialFeature};

pub fn parse_all_wkt(records: Vec<RawRecord>) -> Result<Vec<SpatialFeature>> {
    let mut features = Vec::with_capacity(records.len());

    for (row_id, record) in records.into_iter().enumerate() {
        let wkt_str = record
            .fields
            .get("polygon_geom")
            .ok_or_else(|| anyhow::anyhow!("missing polygon_geom at row {}", row_id))?;

        let wkt_str = wkt_str.trim();
        if wkt_str.is_empty() || wkt_str.eq_ignore_ascii_case("null") {
            continue;
        }
        let geometry: Geometry<f64> = Geometry::try_from_wkt_str(wkt_str)
            .map_err(|e| anyhow::anyhow!("WKT parse error at row {}: {}", row_id, e))?;

        features.push(SpatialFeature {
            row_id,
            geometry,
            raw: record,
        });
    }

    Ok(features)
}

pub fn split_by_data_type(
    features: Vec<SpatialFeature>,
) -> Result<(
    Vec<SpatialFeature>,
    Vec<SpatialFeature>,
    Vec<SpatialFeature>,
)> {
    let mut pois = Vec::new();
    let mut micros = Vec::new();
    let mut markets = Vec::new();

    for f in features {
        match f.raw.fields.get("data_type").map(|s| s.as_str()) {
            Some("POI") => pois.push(f),
            Some("微网格") => micros.push(f),
            Some("市场网格") => markets.push(f),
            Some(other) => anyhow::bail!("unknown data_type: {}", other),
            None => anyhow::bail!("missing data_type field"),
        }
    }

    Ok((pois, micros, markets))
}
