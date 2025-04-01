use std::path::Path;
use uuid::Uuid;
use chrono::Utc;

use ledger_db::model::*;
use ledger_db::storage::binary::read_accounts_bin;
use ledger_db::storage::accounts::write_account_bin_and_index;
use ledger_db::index::btree::BTreeIndex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut account_index = BTreeIndex::load(Path::new("data/accounts.idx"))?;

    let account = Account {
        id: Uuid::new_v4(),
        name: "Dwik cashes".to_string(),
        account_type: AccountType::Asset,
        created_at: Utc::now(),
    };

    let path = Path::new("data/accounts.bin");

    write_account_bin_and_index(&account, path, &mut account_index)?;
    println!("✅ Wrote account to binary file.");

    account_index.persist(Path::new("data/accounts.idx"))?;

    let accounts = read_accounts_bin(path)?;
    for a in accounts {
        println!("📘 Account: {:?}:{:?} - {}", a.account_type, a.id, a.name);
    }

    Ok(())
}
