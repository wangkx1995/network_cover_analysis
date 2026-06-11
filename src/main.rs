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
use config::{load_config, CsvConfig, PostgresConfig};
use data_source::DataSource;
use csv_source::CsvSource;
use pg_source::PgSource;
use wkt_parser::{parse_all_wkt, split_by_data_type};
use projection::project_features;
use spatial_join::spatial_join;
use result_processor::build_micro_market_map;
use coverage::load_coverage_map;
use types::MergedCoverage;

fn build_source(
    source_type: &str,
    csv: Option<&CsvConfig>,
    postgres: Option<&PostgresConfig>,
    conn_str: Option<&str>,
) -> Result<Box<dyn DataSource>> {
    match source_type {
        "csv" => {
            let csv_cfg = csv.ok_or_else(|| anyhow::anyhow!("csv config required"))?;
            Ok(Box::new(CsvSource::new(&csv_cfg.path)))
        }
        "postgres" => {
            let pg_cfg = postgres.ok_or_else(|| anyhow::anyhow!("postgres config required"))?;
            let conn = conn_str.ok_or_else(|| anyhow::anyhow!("database.conn_str required for postgres source"))?;
            Ok(Box::new(PgSource::new(conn, &pg_cfg.query)))
        }
        _ => anyhow::bail!("unsupported source type: {}", source_type),
    }
}

fn main() -> Result<()> {
    let start = Instant::now();
    let config = load_config("config/app.toml")?;
    fs::create_dir_all(&config.output.dir)?;

    let conn_str = config.database.as_ref().map(|d| d.conn_str.as_str());

    let source = build_source(
        &config.source.source_type,
        config.source.csv.as_ref(),
        config.source.postgres.as_ref(),
        conn_str,
    )?;

    let t0 = Instant::now();
    let coverage_map: HashMap<String, MergedCoverage> = if let Some(c) = &config.coverage {
        let cov_source = build_source(
            &c.source_type,
            c.csv.as_ref(),
            c.postgres.as_ref(),
            conn_str,
        )?;
        load_coverage_map(&*cov_source)?
    } else {
        eprintln!("[warn] coverage section not configured, all coverage fields will be empty");
        HashMap::new()
    };
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
    output::save_micro_market(
        &micro_market_results,
        &coverage_map,
        &format!("{}/{}", config.output.dir, config.output.micro_market),
    )?;
    println!("  micro->market:  {:6.1}s", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let micro_market_map = build_micro_market_map(&micro_market_results);
    let poi_micro_results = spatial_join(&pois_proj, &micros_proj);
    output::save_poi_micro(
        &poi_micro_results,
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
