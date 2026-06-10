use std::collections::HashMap;
use anyhow::Result;
use csv::Writer;
use serde::Serialize;
use crate::types::{SpatialFeature, MergedCoverage};
use crate::spatial_join::JoinResult;

fn fmt_rate(v: f64) -> String {
    format!("{:.2}", v)
}

fn calc_share(m: f64, tu: f64) -> String {
    let denom = m + tu;
    if denom > 0.0 {
        format!("{:.2}", m * 100.0 / denom)
    } else {
        "0.00".to_string()
    }
}

fn get_region_name(fields: &HashMap<String, String>) -> String {
    fields.get("region_name").cloned().unwrap_or_default()
}

fn lookup_coverage<'a>(
    coverage_map: &'a HashMap<String, MergedCoverage>,
    poi_number: &str,
) -> Option<&'a MergedCoverage> {
    coverage_map.get(poi_number)
}

// ─── Market Direct ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct MarketDirectRecord {
    #[serde(rename = "地市")]
    region_name: String,
    #[serde(rename = "市场网格编号")]
    market_id: String,
    #[serde(rename = "市场网格名称")]
    market_name: String,
    #[serde(rename = "4G移动覆盖率")]
    coverage_4g: String,
    #[serde(rename = "4G竞对覆盖率")]
    coverage_tu_4g: String,
    #[serde(rename = "5G移动覆盖率")]
    coverage_5g: String,
    #[serde(rename = "5G竞对覆盖率")]
    coverage_tu_5g: String,
    #[serde(rename = "4G市场份额（市占率）")]
    share_4g: String,
    #[serde(rename = "5G市场份额（市占率）")]
    share_5g: String,
}

pub fn save_market_direct(
    markets: &[SpatialFeature],
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    for m in markets {
        let pn = m.raw.fields.get("poi_number").cloned().unwrap_or_default();
        let cov = lookup_coverage(coverage_map, &pn);

        wtr.serialize(MarketDirectRecord {
            region_name: get_region_name(&m.raw.fields),
            market_id: pn,
            market_name: m.raw.fields.get("poi_name").cloned().unwrap_or_default(),
            coverage_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m)),
            coverage_tu_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu)),
            coverage_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m_2)),
            coverage_tu_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu_2)),
            share_4g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m, c.item_num_tu)),
            share_5g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m_2, c.item_num_tu_2)),
        })?;
    }
    wtr.flush()?;
    Ok(())
}

// ─── Micro Market ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct MicroMarketRecord {
    #[serde(rename = "地市")]
    region_name: String,
    #[serde(rename = "微网格编号")]
    micro_id: String,
    #[serde(rename = "微网格名称")]
    micro_name: String,
    #[serde(rename = "4G移动覆盖率")]
    coverage_4g: String,
    #[serde(rename = "4G竞对覆盖率")]
    coverage_tu_4g: String,
    #[serde(rename = "5G移动覆盖率")]
    coverage_5g: String,
    #[serde(rename = "5G竞对覆盖率")]
    coverage_tu_5g: String,
    #[serde(rename = "4G市场份额（市占率）")]
    share_4g: String,
    #[serde(rename = "5G市场份额（市占率）")]
    share_5g: String,
}

pub fn save_micro_market(
    results: &[JoinResult],
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    for r in results {
        let pn = r.left_raw.fields.get("poi_number").cloned().unwrap_or_default();
        let cov = lookup_coverage(coverage_map, &pn);

        wtr.serialize(MicroMarketRecord {
            region_name: get_region_name(&r.left_raw.fields),
            micro_id: pn,
            micro_name: r.left_raw.fields.get("poi_name").cloned().unwrap_or_default(),
            coverage_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m)),
            coverage_tu_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu)),
            coverage_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m_2)),
            coverage_tu_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu_2)),
            share_4g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m, c.item_num_tu)),
            share_5g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m_2, c.item_num_tu_2)),
        })?;
    }
    wtr.flush()?;
    Ok(())
}

// ─── POI Micro ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct PoiMicroRecord {
    #[serde(rename = "地市")]
    region_name: String,
    #[serde(rename = "POI编号")]
    poi_id: String,
    #[serde(rename = "归属的微网格编号")]
    micro_id: String,
    #[serde(rename = "归属的市场网格编号")]
    market_id: String,
    #[serde(rename = "4G移动覆盖率")]
    coverage_4g: String,
    #[serde(rename = "4G竞对覆盖率")]
    coverage_tu_4g: String,
    #[serde(rename = "5G移动覆盖率")]
    coverage_5g: String,
    #[serde(rename = "5G竞对覆盖率")]
    coverage_tu_5g: String,
    #[serde(rename = "4G市场份额（市占率）")]
    share_4g: String,
    #[serde(rename = "5G市场份额（市占率）")]
    share_5g: String,
}

pub fn save_poi_micro(
    results: &[JoinResult],
    micro_market_map: &HashMap<String, (String, String)>,
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    for r in results {
        let pn = r.left_raw.fields.get("poi_number").cloned().unwrap_or_default();
        let micro_id = r
            .right_raw
            .as_ref()
            .and_then(|raw| raw.fields.get("poi_number").cloned())
            .unwrap_or_default();
        let (market_id, _) = micro_market_map
            .get(&micro_id)
            .cloned()
            .unwrap_or_default();
        let cov = lookup_coverage(coverage_map, &pn);

        wtr.serialize(PoiMicroRecord {
            region_name: get_region_name(&r.left_raw.fields),
            poi_id: pn,
            micro_id,
            market_id,
            coverage_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m)),
            coverage_tu_4g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu)),
            coverage_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_m_2)),
            coverage_tu_5g: cov.map_or_else(String::new, |c| fmt_rate(c.coverage_tu_2)),
            share_4g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m, c.item_num_tu)),
            share_5g: cov.map_or_else(String::new, |c| calc_share(c.item_num_m_2, c.item_num_tu_2)),
        })?;
    }
    wtr.flush()?;
    Ok(())
}
