use std::collections::HashMap;
use crate::spatial_join::JoinResult;

pub fn dedup_best_match(results: Vec<JoinResult>) -> Vec<JoinResult> {
    let mut best: HashMap<usize, JoinResult> = HashMap::new();

    for r in results {
        best.entry(r.left_row_id)
            .and_modify(|existing| {
                if r.intersection_ratio > existing.intersection_ratio {
                    *existing = r.clone();
                }
            })
            .or_insert(r);
    }

    best.into_values().collect()
}

pub fn build_micro_market_map(
    results: &[JoinResult],
) -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();
    for r in results {
        let micro_id = r
            .left_raw
            .fields
            .get("poi_number")
            .cloned()
            .unwrap_or_default();
        if let Some(ref right_raw) = r.right_raw {
            let market_id = right_raw
                .fields
                .get("poi_number")
                .cloned()
                .unwrap_or_default();
            let market_name = right_raw
                .fields
                .get("poi_name")
                .cloned()
                .unwrap_or_default();
            map.insert(micro_id, (market_id, market_name));
        }
    }
    map
}
