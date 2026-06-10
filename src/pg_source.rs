use std::collections::HashMap;
use anyhow::Result;
use postgres::{Client, NoTls};
use crate::types::RawRecord;
use crate::data_source::DataSource;

pub struct PgSource {
    conn_str: String,
    query: String,
}

impl PgSource {
    pub fn new(conn_str: &str, query: &str) -> Self {
        Self {
            conn_str: conn_str.to_string(),
            query: query.to_string(),
        }
    }
}

impl DataSource for PgSource {
    fn load_records(&self) -> Result<Vec<RawRecord>> {
        let mut client = Client::connect(&self.conn_str, NoTls)?;
        let rows = client.query(&self.query, &[])?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let col_names: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let num_cols = col_names.len();

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut fields = HashMap::new();
            for i in 0..num_cols {
                let val: Option<String> = row.try_get(i)?;
                fields.insert(col_names[i].clone(), val.unwrap_or_default());
            }
            records.push(RawRecord { fields });
        }

        Ok(records)
    }
}
