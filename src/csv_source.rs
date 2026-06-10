use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use polars::prelude::*;
use crate::types::RawRecord;
use crate::data_source::DataSource;

pub struct CsvSource {
    path: String,
}

impl CsvSource {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }
}

impl DataSource for CsvSource {
    fn load_records(&self) -> Result<Vec<RawRecord>> {
        let df = CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(PathBuf::from(&self.path)))?
            .finish()?;

        let col_names: Vec<&str> = df.get_column_names().iter().map(|s| s.as_str()).collect();
        let height = df.height();

        let mut records = Vec::with_capacity(height);
        for i in 0..height {
            let mut fields = HashMap::new();
            for &name in &col_names {
                let series = df.column(name)?;
                let val = series
                    .get(i)
                    .map(|av| av.to_string())
                    .map(|s| s.trim_matches('"').to_string())
                    .unwrap_or_default();
                fields.insert(name.to_string(), val);
            }
            records.push(RawRecord { fields });
        }

        Ok(records)
    }
}
