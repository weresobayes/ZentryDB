use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Write, Seek, SeekFrom}};
use std::path::Path;

use serde_json::{from_str, to_string};

use crate::{index::BTreeIndex, read_transactions_bin};
use crate::model::Transaction;
use crate::storage::binary::write_transaction_bin;

pub fn write_transaction_bin_and_index(
    tx: &Transaction,
    path: &Path,
    index: &mut BTreeIndex
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    write_transaction(tx)?;

    let offset = file.seek(SeekFrom::End(0))?;
    write_transaction_bin(tx, path)?;
    index.insert(tx.id, offset);
    
    Ok(())
}

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

pub fn load_transactions_from_bin(bin_path: &Path) -> std::io::Result<Vec<Transaction>> {
    let start = std::time::Instant::now();
    let result = read_transactions_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading transactions took: {:?}", duration);
    result
}
    