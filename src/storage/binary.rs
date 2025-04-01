use std::fs::{OpenOptions, File};
use std::io::{BufReader, Read, BufWriter, Write, ErrorKind};
use std::path::Path;

use uuid::Uuid;
use chrono::{TimeZone, Utc};
use serde_json::{from_slice, to_vec, Value};

use crate::model::{Account, AccountType, Entry, Transaction};

fn account_type_to_byte(account_type: &AccountType) -> u8 {
    match account_type {
        AccountType::Asset => 0,
        AccountType::Liability => 1,
        AccountType::Equity => 2,
        AccountType::Revenue => 3,
        AccountType::Expense => 4,
    }
}

fn byte_to_account_type(byte: u8) -> std::io::Result<AccountType> {
    match byte {
        0 => Ok(AccountType::Asset),
        1 => Ok(AccountType::Liability),
        2 => Ok(AccountType::Equity),
        3 => Ok(AccountType::Revenue),
        4 => Ok(AccountType::Expense),
        _ => Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Invalid account type byte",
        )),
    }
}

pub fn write_account_bin(account: &Account, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);

    writer.write_all(account.id.as_bytes())?;

    let name_bytes = account.name.as_bytes();
    let name_len = name_bytes.len();

    if name_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Account name is too long",
        ));
    }
    writer.write_all(&[name_len as u8])?;
    writer.write_all(name_bytes)?;

    writer.write_all(&[account_type_to_byte(&account.account_type)])?;

    let timestamp = account.created_at.timestamp();
    writer.write_all(&timestamp.to_le_bytes())?;
    
    Ok(())
}

pub fn write_transaction_bin(transaction: &Transaction, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);

    let timestamp = transaction.timestamp.timestamp();
    let description_bytes = transaction.description.as_bytes();
    let description_len = description_bytes.len();
    if description_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Transaction description is too long",
        ));
    }

    writer.write_all(transaction.id.as_bytes())?;
    writer.write_all(&[description_len as u8])?;
    writer.write_all(description_bytes)?;

    match &transaction.metadata {
        Some(value) => {
            writer.write_all(&[1])?;

            let serialized_value = to_vec(value)?;
            let len = serialized_value.len() as u32;

            writer.write_all(&len.to_le_bytes())?;
            writer.write_all(&serialized_value)?;
        }
        None => {}
    }

    writer.write_all(&timestamp.to_le_bytes())?;
    
    Ok(())
}

pub fn write_entry_bin(entry: &Entry, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);

    writer.write_all(entry.id.as_bytes())?;
    writer.write_all(entry.transaction_id.as_bytes())?;
    writer.write_all(entry.account_id.as_bytes())?;
    writer.write_all(&entry.amount.to_le_bytes())?;
    
    Ok(())
}

pub fn read_accounts_bin(path: &Path) -> std::io::Result<Vec<Account>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut accounts = Vec::new();

    loop {
        let mut id_bytes = [0u8; 16];
        if file.read_exact(&mut id_bytes).is_err() {
            break;
        }

        let mut len_buf = [0u8; 1];
        file.read_exact(&mut len_buf)?;
        let name_len = len_buf[0] as usize;

        let mut name_buf = vec![0u8; name_len];
        file.read_exact(&mut name_buf)?;
        let name = String::from_utf8(name_buf).unwrap_or_default();

        let mut type_buf = [0u8; 1];
        file.read_exact(&mut type_buf)?;
        let account_type = byte_to_account_type(type_buf[0])?;

        let mut timestamp_buf = [0u8; 8];
        file.read_exact(&mut timestamp_buf)?;
        let timestamp = i64::from_le_bytes(timestamp_buf);
        let created_at = Utc.timestamp_opt(timestamp, 0).unwrap();

        accounts.push(Account {
            id: Uuid::from_bytes(id_bytes),
            name,
            account_type,
            created_at,
        });
    }

    Ok(accounts)
}

fn read_metadata<R: Read>(reader: &mut R) -> std::io::Result<Option<Value>> {
    let mut tag = [0u8; 1];
    reader.read_exact(&mut tag)?;

    if tag[0] == 0 {
        return Ok(None)
    } else {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf);
        
        let mut bytes = vec![0u8; len as usize];
        reader.read_exact(&mut bytes)?;

        let value: Value = from_slice(&bytes)?;
        Ok(Some(value))
    }
}

pub fn read_transactions_bin(path: &Path) -> std::io::Result<Vec<Transaction>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut transactions = Vec::new();

    loop {
        let mut id_bytes = [0u8; 16];
        if (file.read_exact(&mut id_bytes)).is_err() {
            break;
        }

        let mut len_buf = [0u8; 1];
        if (file.read_exact(&mut len_buf)).is_err() {
            break;
        }
        let description_len = len_buf[0] as usize;

        let mut description_buf = vec![0u8; description_len];
        if (file.read_exact(&mut description_buf)).is_err() {
            break;
        }
        let description = String::from_utf8(description_buf).unwrap_or_default();

        let metadata = read_metadata(&mut file)?;

        let mut timestamp_buf = [0u8; 8];
        if (file.read_exact(&mut timestamp_buf)).is_err() {
            break;
        }
        let _timestamp = i64::from_le_bytes(timestamp_buf);
        let timestamp = Utc.timestamp_opt(_timestamp, 0).unwrap();

        transactions.push(Transaction {
            id: Uuid::from_bytes(id_bytes),
            description,
            metadata,
            timestamp,
        });
    }

    Ok(transactions)
}