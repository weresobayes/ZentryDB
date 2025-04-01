use crate::model::Transaction;
use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Write}};
use serde_json::{from_str, to_string};

pub fn write_transaction(tx: &Transaction) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/transactions.jsonl")?;
    
    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", to_string(tx)?)?;
    Ok(())
}

pub fn load_transactions() -> std::io::Result<Vec<Transaction>> {
    let file = OpenOptions::new()
        .read(true)
        .open("data/transactions.jsonl")?;
    
    let reader = BufReader::new(file);

    reader.lines().map(|line| {
        let line = line?;
        let transaction: Transaction = from_str(&line)?;

        Ok(transaction)
    }).collect()
}
    