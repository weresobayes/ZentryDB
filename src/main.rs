use std::path::Path;
use uuid::Uuid;
use chrono::Utc;

use ZentryDB::model::*;
use ZentryDB::storage::binary::{read_accounts_bin, read_transactions_bin, read_entries_bin};
use ZentryDB::storage::accounts::write_account_bin_and_index;
use ZentryDB::storage::transactions::write_transaction_bin_and_index;
use ZentryDB::storage::entries::write_entry_bin_and_index;
use ZentryDB::index::btree::BTreeIndex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut account_index = BTreeIndex::load(Path::new("data/accounts.idx"))?;
    let mut transaction_index = BTreeIndex::load(Path::new("data/transactions.idx"))?;
    let mut entry_index = BTreeIndex::load(Path::new("data/entries.idx"))?;

    let account_bin_path = Path::new("data/accounts.bin");
    let transaction_bin_path = Path::new("data/transactions.bin");
    let entry_bin_path = Path::new("data/entries.bin");

    let first_account = Account {
        id: Uuid::new_v4(),
        name: "Dwik cashes".to_string(),
        account_type: AccountType::Asset,
        created_at: Utc::now(),
    };

    let second_account = Account {
        id: Uuid::new_v4(),
        name: "Dwik bank account".to_string(),
        account_type: AccountType::Asset,
        created_at: Utc::now(),
    };

    let transaction = Transaction {
        id: Uuid::new_v4(),
        description: "Transfer from Dwik bank to Dwik cashes".to_string(),
        metadata: None,
        timestamp: Utc::now(),
    };

    let debit = Entry {
        id: Uuid::new_v4(),
        transaction_id: transaction.id,
        account_id: second_account.id,
        amount: 1000000000.0,
    };

    let credit = Entry {
        id: Uuid::new_v4(),
        transaction_id: transaction.id,
        account_id: first_account.id,
        amount: -1000000000.0,
    };

    write_account_bin_and_index(&first_account, account_bin_path, &mut account_index)?;
    write_account_bin_and_index(&second_account, account_bin_path, &mut account_index)?;
    println!("✅ Wrote account to binary file.");

    write_transaction_bin_and_index(&transaction, transaction_bin_path, &mut transaction_index)?;
    println!("✅ Wrote transaction to binary file.");

    write_entry_bin_and_index(&debit, entry_bin_path, &mut entry_index)?;
    write_entry_bin_and_index(&credit, entry_bin_path, &mut entry_index)?;
    println!("✅ Wrote entry to binary file.");

    account_index.persist(Path::new("data/accounts.idx"))?;
    transaction_index.persist(Path::new("data/transactions.idx"))?;
    entry_index.persist(Path::new("data/entries.idx"))?;

    let accounts = read_accounts_bin(account_bin_path)?;
    for a in accounts {
        println!("📘 Account: {:?}:{:?} - {}", a.account_type, a.id, a.name);
    }

    let transactions = read_transactions_bin(transaction_bin_path)?;
    for t in transactions {
        println!("📅 Transaction: {:?} - {}", t.id, t.description);
    }

    let entries: Vec<Entry> = read_entries_bin(entry_bin_path)?;
    for e in entries {
        println!("📝 Entry: {:?} - {}", e.id, e.transaction_id);
    }

    Ok(())
}
