use std::collections::HashMap;
use anyhow::Result;
use crate::types::MergedCoverage;

pub fn load_coverage_map(path: &str) -> Result<HashMap<String, MergedCoverage>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

    let mut dedup: HashMap<(String, String), Vec<HashMap<String, String>>> = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        let mut map = HashMap::new();
        for (i, field) in record.iter().enumerate() {
            if i < headers.len() {
                map.insert(headers[i].clone(), field.to_string());
            }
        }
        let pn = map.get("poi_number").cloned().unwrap_or_default();
        let tech = map.get("technology").cloned().unwrap_or_default();
        dedup.entry((pn, tech)).or_default().push(map);
    }

    let deduped: Vec<HashMap<String, String>> = dedup.into_values().filter_map(|mut v| v.pop()).collect();

    let mut lte_rows: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut nr_rows: HashMap<String, HashMap<String, String>> = HashMap::new();

    for row in deduped {
        let pn = row.get("poi_number").cloned().unwrap_or_default();
        let tech = row.get("technology").cloned().unwrap_or_default();
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
