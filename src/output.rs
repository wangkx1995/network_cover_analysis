use std::collections::HashMap;
use anyhow::Result;
use csv::Writer;
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

fn coverage_fields(cov: Option<&MergedCoverage>) -> Vec<String> {
    match cov {
        Some(c) => vec![
            fmt_rate(c.coverage_m),
            fmt_rate(c.coverage_tu),
            fmt_rate(c.coverage_m_2),
            fmt_rate(c.coverage_tu_2),
            calc_share(c.item_num_m, c.item_num_tu),
            calc_share(c.item_num_m_2, c.item_num_tu_2),
        ],
        None => vec!["".to_string(); 6],
    }
}

fn get_str(fields: &HashMap<String, String>, key: &str) -> String {
    fields.get(key).cloned().unwrap_or_default()
}

pub fn save_market_direct(
    markets: &[SpatialFeature],
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(["地市", "市场网格编号", "市场网格名称",
        "4G移动覆盖率", "4G竞对覆盖率", "5G移动覆盖率", "5G竞对覆盖率",
        "4G市场份额（市占率）", "5G市场份额（市占率）"])?;
    for m in markets {
        let pn = get_str(&m.raw.fields, "poi_number");
        let cov = coverage_map.get(&pn);
        let mut row = vec![
            get_str(&m.raw.fields, "region_name"),
            pn,
            get_str(&m.raw.fields, "poi_name"),
        ];
        row.extend(coverage_fields(cov));
        wtr.write_record(&row)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn save_micro_market(
    results: &[JoinResult],
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(["地市", "微网格编号", "微网格名称",
        "4G移动覆盖率", "4G竞对覆盖率", "5G移动覆盖率", "5G竞对覆盖率",
        "4G市场份额（市占率）", "5G市场份额（市占率）"])?;
    for r in results {
        let pn = get_str(&r.left_raw.fields, "poi_number");
        let cov = coverage_map.get(&pn);
        let mut row = vec![
            get_str(&r.left_raw.fields, "region_name"),
            pn,
            get_str(&r.left_raw.fields, "poi_name"),
        ];
        row.extend(coverage_fields(cov));
        wtr.write_record(&row)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn save_poi_micro(
    results: &[JoinResult],
    micro_market_map: &HashMap<String, (String, String)>,
    coverage_map: &HashMap<String, MergedCoverage>,
    path: &str,
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(["地市", "POI编号", "归属的微网格编号", "归属的市场网格编号",
        "4G移动覆盖率", "4G竞对覆盖率", "5G移动覆盖率", "5G竞对覆盖率",
        "4G市场份额（市占率）", "5G市场份额（市占率）"])?;
    for r in results {
        let pn = get_str(&r.left_raw.fields, "poi_number");
        let micro_id = r.right_raw.as_ref()
            .map(|raw| get_str(&raw.fields, "poi_number"))
            .unwrap_or_default();
        let market_id = micro_market_map.get(&micro_id)
            .map(|(id, _)| id.clone())
            .unwrap_or_default();
        let cov = coverage_map.get(&pn);
        let mut row = vec![
            get_str(&r.left_raw.fields, "region_name"),
            pn,
            micro_id,
            market_id,
        ];
        row.extend(coverage_fields(cov));
        wtr.write_record(&row)?;
    }
    wtr.flush()?;
    Ok(())
}
