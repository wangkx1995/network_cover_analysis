use anyhow::Result;
use crate::types::RawRecord;

pub trait DataSource {
    fn load_records(&self) -> Result<Vec<RawRecord>>;
}
