use chrono::{DateTime, Utc};
use tabled::Tabled;

#[derive(Tabled)]
pub struct ConversionGraphRow {
    pub graph: String,
    pub rate: f64,
    pub rate_since: DateTime<Utc>,
}
