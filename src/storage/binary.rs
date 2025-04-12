use std::fs::{OpenOptions, File};
use std::io::{BufReader, Read, BufWriter, Write, ErrorKind};
use std::path::Path;

use uuid::Uuid;
use chrono::{TimeZone, Utc};
use serde_json::{from_slice, to_vec, Value};

use crate::model::{Account, AccountType, Entry, Transaction, System, ConversionGraph};

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

    let system_id_bytes = account.system_id.as_bytes();
    let system_id_len = system_id_bytes.len();
    if system_id_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "System ID is too long",
        ));
    }
    writer.write_all(&[system_id_len as u8])?;
    writer.write_all(system_id_bytes)?;
    
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

pub fn write_system_bin(system: &System, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);

    // Write system ID
    let id_bytes = system.id.as_bytes();
    let id_len = id_bytes.len();
    if id_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "System ID is too long",
        ));
    }
    writer.write_all(&[id_len as u8])?;
    writer.write_all(id_bytes)?;

    // Write description
    let desc_bytes = system.description.as_bytes();
    let desc_len = desc_bytes.len();
    if desc_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "System description is too long",
        ));
    }
    writer.write_all(&[desc_len as u8])?;
    writer.write_all(desc_bytes)?;
    
    Ok(())
}

pub fn write_conversion_graph_bin(graph: &ConversionGraph, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);

    // Write graph string
    let graph_bytes = graph.graph.as_bytes();
    let graph_len = graph_bytes.len();
    if graph_len > 255 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Graph string is too long",
        ));
    }
    writer.write_all(&[graph_len as u8])?;
    writer.write_all(graph_bytes)?;

    // Write rate
    writer.write_all(&graph.rate.to_le_bytes())?;

    // Write timestamp
    let timestamp = graph.rate_since.timestamp();
    writer.write_all(&timestamp.to_le_bytes())?;
    
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

        let mut system_id_len_buf = [0u8; 1];
        file.read_exact(&mut system_id_len_buf)?;
        let system_id_len = system_id_len_buf[0] as usize;

        let mut system_id_buf = vec![0u8; system_id_len];
        file.read_exact(&mut system_id_buf)?;
        let system_id = String::from_utf8(system_id_buf).unwrap_or_default();

        accounts.push(Account {
            id: Uuid::from_bytes(id_bytes),
            name,
            account_type,
            created_at,
            system_id,
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

pub fn read_entries_bin(path: &Path) -> std::io::Result<Vec<Entry>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut entries = Vec::new();

    loop {
        let mut id_bytes = [0u8; 16];
        if (file.read_exact(&mut id_bytes)).is_err() {
            break;
        }

        let mut transaction_id_bytes = [0u8; 16];
        if (file.read_exact(&mut transaction_id_bytes)).is_err() {
            break;
        }

        let mut account_id_bytes = [0u8; 16];
        if (file.read_exact(&mut account_id_bytes)).is_err() {
            break;
        }

        let mut amount_bytes = [0u8; 8];
        if (file.read_exact(&mut amount_bytes)).is_err() {
            break;
        }
        let amount = f64::from_le_bytes(amount_bytes);

        entries.push(Entry {
            id: Uuid::from_bytes(id_bytes),
            transaction_id: Uuid::from_bytes(transaction_id_bytes),
            account_id: Uuid::from_bytes(account_id_bytes),
            amount,
        });
    }

    Ok(entries)
}

pub fn read_systems_bin(path: &Path) -> std::io::Result<Vec<System>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut systems = Vec::new();

    loop {
        // Read ID
        let mut id_len_buf = [0u8; 1];
        if file.read_exact(&mut id_len_buf).is_err() {
            break;
        }
        let id_len = id_len_buf[0] as usize;

        let mut id_buf = vec![0u8; id_len];
        file.read_exact(&mut id_buf)?;
        let id = String::from_utf8(id_buf).unwrap_or_default();

        // Read description
        let mut desc_len_buf = [0u8; 1];
        file.read_exact(&mut desc_len_buf)?;
        let desc_len = desc_len_buf[0] as usize;

        let mut desc_buf = vec![0u8; desc_len];
        file.read_exact(&mut desc_buf)?;
        let description = String::from_utf8(desc_buf).unwrap_or_default();

        systems.push(System {
            id,
            description,
        });
    }

    Ok(systems)
}

pub fn read_conversion_graphs_bin(path: &Path) -> std::io::Result<Vec<ConversionGraph>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut graphs = Vec::new();

    loop {
        // Read graph string
        let mut graph_len_buf = [0u8; 1];
        if file.read_exact(&mut graph_len_buf).is_err() {
            break;
        }
        let graph_len = graph_len_buf[0] as usize;

        let mut graph_buf = vec![0u8; graph_len];
        file.read_exact(&mut graph_buf)?;
        let graph = String::from_utf8(graph_buf).unwrap_or_default();

        // Read rate
        let mut rate_buf = [0u8; 8];
        file.read_exact(&mut rate_buf)?;
        let rate = f64::from_le_bytes(rate_buf);

        // Read timestamp
        let mut timestamp_buf = [0u8; 8];
        file.read_exact(&mut timestamp_buf)?;
        let timestamp = i64::from_le_bytes(timestamp_buf);
        let rate_since = Utc.timestamp_opt(timestamp, 0).unwrap();

        graphs.push(ConversionGraph {
            graph,
            rate,
            rate_since,
        });
    }

    Ok(graphs)
}