use std::collections::HashMap;
use anyhow::Result;
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
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(&self.path)?;

        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let mut fields = HashMap::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    fields.insert(headers[i].clone(), field.to_string());
                }
            }
            records.push(RawRecord { fields });
        }

        Ok(records)
    }
}
