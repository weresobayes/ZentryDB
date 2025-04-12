use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: String,
    pub description: String,
}

/// Represents a system conversion relationship between two systems.
/// 
/// # Format
/// The graph is represented as a string in the format "A -> B", where A and B are system ids.
/// 
/// # Examples
/// Users can define the graph in several ways:
/// - One-way conversion: `"USD -> IDR"` or `"USD <- IDR"`
/// - Two-way conversion: `"USD <-> SGD"`
/// 
/// # Internal Representation
/// - All conversions are stored internally in the "A -> B" format
/// - Bidirectional graphs are stored as two separate conversion records
/// - The conversion rate always follows the formula: `A * rate = B`
/// - For bidirectional conversions:
///   - "A -> B" is stored with the original rate
///   - "B -> A" is stored with rate = 1 / (original rate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionGraph {
    pub graph: String,
    pub rate: f64,
    pub rate_since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>,
    pub system_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: f64, // positive for debit, negative for credit
}