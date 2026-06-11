use std::collections::HashMap;
use crate::spatial_join::JoinResult;

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
