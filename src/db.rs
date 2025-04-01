use crate::model::{Transaction, Entry, Account};
use crate::storage::{write_transaction, write_entries, load_accounts, load_transactions, load_entries};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Ledger {
    pub accounts: HashMap<Uuid, Account>,
    pub transactions: HashMap<Uuid, Transaction>,
    pub entries: Vec<Entry>,
}

impl Ledger {
    pub fn load_from_disk() -> std::io::Result<Self> {
        let accounts_list = load_accounts()?;
        let transactions_list = load_transactions()?;
        let entries_list = load_entries()?;

        let accounts: HashMap<Uuid, Account> = accounts_list.into_iter().map(|account| (account.id, account)).collect();
        let transactions: HashMap<Uuid, Transaction> = transactions_list.into_iter().map(|transaction| (transaction.id, transaction)).collect();

        Ok(Self {
            accounts,
            transactions,
            entries: entries_list,
        })
    }
}

pub fn record_transaction(tx: Transaction, entries: Vec<Entry>) -> std::io::Result<()> {
    let sum: f64 = entries.iter().map(|e| e.amount).sum();

    if sum.abs() > f64::EPSILON {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unbalanced transaction: total = {}", sum),
        ));
    }

    write_transaction(&tx)?;
    write_entries(&entries)?;
    Ok(())
}