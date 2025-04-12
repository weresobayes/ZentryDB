use chrono::Utc;
use uuid::Uuid;
use zentry_db::{
    db::Ledger, install, model::{Account, AccountType, Entry, System, Transaction}
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database installation
    install::install()?;

    // Initialize ledger
    let mut ledger = Ledger::load_from_disk()?;

    // Create currency system
    let idr_system = System {
        id: "IDR".to_string(),
        description: "Indonesian Rupiah".to_string(),
    };
    ledger.create_system(idr_system)?;

    // Create accounts
    let first_account = Account {
        id: Uuid::new_v4(),
        name: "Dwik cashes".to_string(),
        account_type: AccountType::Asset,
        created_at: Utc::now(),
        system_id: "IDR".to_string(),
    };
    ledger.create_account(first_account.clone())?;

    let second_account = Account {
        id: Uuid::new_v4(),
        name: "Dwik bank account".to_string(),
        account_type: AccountType::Asset,
        created_at: Utc::now(),
        system_id: "IDR".to_string(),
    };
    ledger.create_account(second_account.clone())?;
    println!("✅ Created accounts");

    // Create transaction with entries
    let transaction = Transaction {
        id: Uuid::new_v4(),
        description: "Transfer from Dwik bank to Dwik cashes".to_string(),
        metadata: None,
        timestamp: Utc::now(),
    };

    let entries = vec![
        Entry {
            id: Uuid::new_v4(),
            transaction_id: transaction.id,
            account_id: second_account.id,
            amount: 1000000000.0,
        },
        Entry {
            id: Uuid::new_v4(),
            transaction_id: transaction.id,
            account_id: first_account.id,
            amount: -1000000000.0,
        },
    ];

    // Record transaction (this will validate and write both transaction and entries)
    ledger.record_transaction(transaction, entries)?;
    println!("✅ Recorded transaction");

    // Print current state
    for (_, account) in ledger.accounts.iter() {
        println!(
            "📘 Account: {:?}:{:?} - {} ({})",
            account.account_type, account.id, account.name, account.system_id
        );
    }

    for (_, tx) in ledger.transactions.iter() {
        println!("📅 Transaction: {:?} - {}", tx.id, tx.description);
    }

    for entry in ledger.entries.iter() {
        println!("📝 Entry: {:?} - {}", entry.id, entry.transaction_id);
    }

    Ok(())
}
