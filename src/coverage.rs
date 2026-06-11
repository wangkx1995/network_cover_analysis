use std::collections::HashMap;
use anyhow::Result;
use crate::types::MergedCoverage;
use crate::data_source::DataSource;

pub fn load_coverage_map(source: &dyn DataSource) -> Result<HashMap<String, MergedCoverage>> {
    let records = source.load_records()?;

    let mut dedup: HashMap<(String, String), HashMap<String, String>> = HashMap::new();
    for record in records {
        let pn = record.fields.get("poi_number").cloned().unwrap_or_default();
        let tech = record.fields.get("technology").cloned().unwrap_or_default();
        dedup.insert((pn, tech), record.fields);
    }

    let mut lte_rows: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut nr_rows: HashMap<String, HashMap<String, String>> = HashMap::new();

    for ((pn, tech), row) in dedup {
        match tech.as_str() {
            "LTE" => { lte_rows.insert(pn, row); }
            "NR" => { nr_rows.insert(pn, row); }
            _ => {}
        }
    }

    let parse_f64 = |key: &str, source: &HashMap<String, String>| -> f64 {
        source.get(key).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
    };

    let mut result = HashMap::new();
    for (pn, lte_row) in &lte_rows {
        let coverage_m = parse_f64("coverage_m", lte_row);
        let coverage_tu = parse_f64("coverage_tu", lte_row);
        let item_num_m = parse_f64("item_num_m", lte_row);
        let item_num_tu = parse_f64("item_num_tu", lte_row);

        let (coverage_m_2, coverage_tu_2, item_num_m_2, item_num_tu_2) =
            if let Some(nr_row) = nr_rows.get(pn) {
                (
                    parse_f64("coverage_m_2", nr_row),
                    parse_f64("coverage_tu_2", nr_row),
                    parse_f64("item_num_m", nr_row),
                    parse_f64("item_num_tu", nr_row),
                )
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

        result.insert(pn.clone(), MergedCoverage {
            coverage_m,
            coverage_tu,
            item_num_m,
            item_num_tu,
            coverage_m_2,
            coverage_tu_2,
            item_num_m_2,
            item_num_tu_2,
        });
    }

    Ok(result)
}
