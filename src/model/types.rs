use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: f64, // positive for debit, negative for credit
}