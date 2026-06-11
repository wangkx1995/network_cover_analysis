use geo::algorithm::area::Area;
use geo::algorithm::{BoundingRect, Intersects};
use geo::BooleanOps;
use geo_types::{Geometry, MultiPolygon};
use rayon::prelude::*;
use rstar::{RTree, RTreeObject, AABB};
use std::panic::catch_unwind;
use crate::types::{RawRecord, SpatialFeature};

struct RectEntry {
    rect: geo_types::Rect<f64>,
    index: usize,
}

impl RTreeObject for RectEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.rect.min().x, self.rect.min().y],
            [self.rect.max().x, self.rect.max().y],
        )
    }
}

#[derive(Debug, Clone)]
pub struct JoinResult {
    pub left_raw: RawRecord,
    pub right_raw: Option<RawRecord>,
}

fn as_multipolygon(g: &Geometry<f64>) -> Option<MultiPolygon<f64>> {
    match g {
        Geometry::Polygon(p) => Some(MultiPolygon::new(vec![p.clone()])),
        Geometry::MultiPolygon(mp) => Some(mp.clone()),
        _ => None,
    }
}

fn bbox_overlap_area(r1: &geo_types::Rect<f64>, r2: &geo_types::Rect<f64>) -> f64 {
    let x_min = r1.min().x.max(r2.min().x);
    let y_min = r1.min().y.max(r2.min().y);
    let x_max = r1.max().x.min(r2.max().x);
    let y_max = r1.max().y.min(r2.max().y);
    if x_max > x_min && y_max > y_min {
        (x_max - x_min) * (y_max - y_min)
    } else {
        0.0
    }
}

fn bbox_area(r: &geo_types::Rect<f64>) -> f64 {
    (r.max().x - r.min().x) * (r.max().y - r.min().y)
}

pub fn spatial_join(left: &[SpatialFeature], right: &[SpatialFeature]) -> Vec<JoinResult> {
    let right_items: Vec<(MultiPolygon<f64>, geo_types::Rect<f64>, &SpatialFeature)> = right
        .iter()
        .filter_map(|feat| {
            let mp = as_multipolygon(&feat.geometry)?;
            let rect = mp.bounding_rect()?;
            Some((mp, rect, feat))
        })
        .collect();

    let rtree: RTree<RectEntry> = RTree::bulk_load(
        right_items.iter().enumerate().map(|(i, (_, rect, _))| RectEntry { rect: *rect, index: i }).collect(),
    );
    left.par_iter()
        .flat_map(|left_feat| {
            let left_mp = match as_multipolygon(&left_feat.geometry) {
                Some(mp) => mp,
                None => {
                    return vec![JoinResult {
                        left_raw: left_feat.raw.clone(),
                        right_raw: None,
                    }];
                }
            };
            let left_area: f64 = left_mp.iter().map(|p| p.unsigned_area()).sum();
            if left_area <= 0.0 {
                return vec![JoinResult {
                    left_raw: left_feat.raw.clone(),
                    right_raw: None,
                }];
            }

            let left_rect = match left_mp.bounding_rect() {
                Some(r) => r,
                None => {
                    return vec![JoinResult {
                        left_raw: left_feat.raw.clone(),
                        right_raw: None,
                    }];
                }
            };
            let left_bbox_a = bbox_area(&left_rect);

            let env = AABB::from_corners(
                [left_rect.min().x, left_rect.min().y],
                [left_rect.max().x, left_rect.max().y],
            );

            let mut best_ratio = 0.0;
            let mut best_right: Option<&SpatialFeature> = None;

            for entry in rtree.locate_in_envelope_intersecting(&env) {
                let (right_mp, right_rect, right_feat) = &right_items[entry.index];
                let bbox_upper = if left_area > 0.0 {
                    bbox_overlap_area(&left_rect, right_rect) / left_area
                } else {
                    0.0
                };
                if bbox_upper <= best_ratio {
                    continue;
                }

                if !left_mp.intersects(right_mp) {
                    continue;
                }

                let ratio = match catch_unwind(|| {
                    let inter_mp = left_mp.intersection(right_mp);
                    let inter_area: f64 = inter_mp.iter().map(|p| p.unsigned_area()).sum();
                    inter_area / left_area
                }) {
                    Ok(r) => r,
                    Err(_) => {
                        let overlap = bbox_overlap_area(&left_rect, right_rect);
                        if left_bbox_a > 0.0 {
                            overlap / left_bbox_a
                        } else {
                            0.0
                        }
                    }
                };

                let ratio = ratio.clamp(0.0, 1.0);
                if ratio <= best_ratio {
                    continue;
                }

                best_ratio = ratio;
                best_right = Some(right_feat);
                if best_ratio >= 1.0 {
                    break;
                }
            }

            let right_raw = best_right.map(|r| r.raw.clone());
            vec![JoinResult {
                left_raw: left_feat.raw.clone(),
                right_raw,
            }]
        })
        .collect()
}
