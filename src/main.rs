mod config;
mod types;
mod data_source;
mod csv_source;
mod pg_source;
mod wkt_parser;
mod projection;
mod spatial_join;
mod result_processor;
mod coverage;
mod output;

use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use anyhow::Result;
use config::load_config;
use data_source::DataSource;
use csv_source::CsvSource;
use pg_source::PgSource;
use wkt_parser::{parse_all_wkt, split_by_data_type};
use projection::project_features;
use spatial_join::spatial_join;
use result_processor::{dedup_best_match, build_micro_market_map};
use coverage::load_coverage_map;
use types::MergedCoverage;

fn main() -> Result<()> {
    let start = Instant::now();
    let config = load_config("config/app.toml")?;
    fs::create_dir_all(&config.output.dir)?;

    let source: Box<dyn DataSource> = match config.source.source_type.as_str() {
        "csv" => {
            let csv_cfg = config.source.csv.expect("csv config required");
            Box::new(CsvSource::new(&csv_cfg.path))
        }
        "postgres" => {
            let pg_cfg = config.source.postgres.expect("postgres config required");
            Box::new(PgSource::new(&pg_cfg.conn_str, &pg_cfg.query))
        }
        _ => anyhow::bail!("unsupported source type: {}", config.source.source_type),
    };

    let t0 = Instant::now();
    let coverage_map: HashMap<String, MergedCoverage> = config
        .coverage
        .as_ref()
        .map(|c| load_coverage_map(&c.path))
        .transpose()?
        .unwrap_or_default();
    println!("  coverage load: {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let records = source.load_records()?;
    let features = parse_all_wkt(records)?;
    let (pois, micros, markets) = split_by_data_type(features)?;
    println!("  load+parse+wkt: {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let pois_proj = project_features(&pois);
    let micros_proj = project_features(&micros);
    let markets_proj = project_features(&markets);
    println!("  projection:     {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    output::save_market_direct(
        &markets,
        &coverage_map,
        &format!("{}/{}", config.output.dir, config.output.market_direct),
    )?;
    println!("  market_direct:  {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let micro_market_results = spatial_join(&micros_proj, &markets_proj);
    let micro_market_deduped = dedup_best_match(micro_market_results);
    output::save_micro_market(
        &micro_market_deduped,
        &coverage_map,
        &format!("{}/{}", config.output.dir, config.output.micro_market),
    )?;
    println!("  micro->market:  {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let micro_market_map = build_micro_market_map(&micro_market_deduped);
    let poi_micro_results = spatial_join(&pois_proj, &micros_proj);
    let poi_micro_deduped = dedup_best_match(poi_micro_results);
    output::save_poi_micro(
        &poi_micro_deduped,
        &micro_market_map,
        &coverage_map,
        &format!("{}/{}", config.output.dir, config.output.poi_micro),
    )?;
    println!("  poi->micro:     {:6.1}s", t0.elapsed().as_secs_f64());

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    if secs >= 60.0 {
        println!("Total time: {:.0}m {:.1}s", secs / 60.0, secs % 60.0);
    } else {
        println!("Total time: {:.1}s", secs);
    }

    Ok(())
}
