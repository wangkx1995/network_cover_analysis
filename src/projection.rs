use geo_types::{Coord, Geometry, LineString, MultiPolygon, Polygon};
use crate::types::SpatialFeature;

const A: f64 = 6378137.0;

fn lonlat_to_mercator(lon: f64, lat: f64) -> (f64, f64) {
    let x = lon * A * std::f64::consts::PI / 180.0;
    let lat_rad = lat.to_radians();
    let y = (std::f64::consts::FRAC_PI_4 + lat_rad / 2.0).tan().ln() * A;
    (x, y)
}

fn project_coords_iter<'a>(
    iter: impl Iterator<Item = &'a Coord<f64>>,
) -> Vec<Coord<f64>> {
    iter.map(|c| {
        let (x, y) = lonlat_to_mercator(c.x, c.y);
        Coord { x, y }
    })
    .collect()
}

fn project_polygon(poly: &Polygon<f64>) -> Polygon<f64> {
    let exterior = LineString::new(project_coords_iter(poly.exterior().coords()));
    let interiors: Vec<LineString<f64>> = poly
        .interiors()
        .iter()
        .map(|ring| LineString::new(project_coords_iter(ring.coords())))
        .collect();
    Polygon::new(exterior, interiors)
}

fn project_multipolygon(mp: &MultiPolygon<f64>) -> MultiPolygon<f64> {
    MultiPolygon::new(mp.iter().map(project_polygon).collect())
}

fn project_geometry(geom: &Geometry<f64>) -> Geometry<f64> {
    match geom {
        Geometry::Polygon(poly) => Geometry::Polygon(project_polygon(poly)),
        Geometry::MultiPolygon(mp) => Geometry::MultiPolygon(project_multipolygon(mp)),
        other => other.clone(),
    }
}

pub fn project_features(features: &[SpatialFeature]) -> Vec<SpatialFeature> {
    features
        .iter()
        .map(|f| SpatialFeature {
            geometry: project_geometry(&f.geometry),
            ..f.clone()
        })
        .collect()
}
