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

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut fields = HashMap::new();
            for (i, name) in col_names.iter().enumerate() {
                let val = column_to_string(&row, i);
                fields.insert(name.clone(), val);
            }
            records.push(RawRecord { fields });
        }

        Ok(records)
    }
}

fn column_to_string(row: &postgres::Row, idx: usize) -> String {
    let ty = row.columns()[idx].type_();
    match ty.name() {
        "varchar" | "text" | "bpchar" | "name" => {
            row.try_get::<_, Option<String>>(idx).ok().flatten().unwrap_or_default()
        }
        "int2" => {
            row.try_get::<_, Option<i16>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "int4" => {
            row.try_get::<_, Option<i32>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "int8" => {
            row.try_get::<_, Option<i64>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "float4" => {
            row.try_get::<_, Option<f32>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "float8" => {
            row.try_get::<_, Option<f64>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "bool" => {
            row.try_get::<_, Option<bool>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "numeric" => {
            row.try_get::<_, Option<f64>>(idx).ok().flatten().map(|v| v.to_string()).unwrap_or_default()
        }
        "timestamptz" | "timestamp" => {
            row.try_get::<_, Option<String>>(idx).ok().flatten().unwrap_or_default()
        }
        _ => {
            row.try_get::<_, Option<String>>(idx).ok().flatten().unwrap_or_default()
        }
    }
}
