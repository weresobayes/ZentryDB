use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::Path;

use regex::Regex;
use uuid::Uuid;
use chrono::{TimeZone, Utc};
use serde_json::{from_slice, to_vec, Value};

use crate::model::{Account, AccountType, Entry, Transaction, System, ConversionGraph};
use crate::storage::layout::{BinaryLayout, BinaryField, LengthType, account_layout, system_layout, conversion_graph_layout};

fn account_type_to_byte(account_type: &AccountType) -> u8 {
    match account_type {
        AccountType::Asset => 0,
        AccountType::Liability => 1,
        AccountType::Equity => 2,
        AccountType::Revenue => 3,
        AccountType::Expense => 4,
    }
}

fn classify_graph_key(graph: &ConversionGraph) -> &'static str {
    let active_conversion_key_pattern = Regex::new(r"^\s*\w+\s*(->|<-|<->)\s*\w+\s*$").unwrap();
    let historical_conversion_key_pattern = Regex::new(
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+\+\d{2}:\d{2}\[\s*\w+\s*(->|<-|<->)\s*\w+\s*\]\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+\+\d{2}:\d{2}$"
    ).unwrap();

    if historical_conversion_key_pattern.is_match(&graph.graph) {
        return "historical";
    } else if active_conversion_key_pattern.is_match(&graph.graph) {
        return "active";
    }

    "unknown"
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

fn write_length_prefixed_field(writer: &mut BufWriter<File>, bytes: &[u8], name: &str, length_type: LengthType) -> std::io::Result<()> {
    let len = bytes.len();
    
    match length_type {
        LengthType::U8 if len <= u8::MAX as usize => {
            writer.write_all(&[len as u8])?;
        }
        LengthType::U16 if len <= u16::MAX as usize => {
            writer.write_all(&(len as u16).to_le_bytes())?;
        }
        LengthType::U32 => {
            writer.write_all(&(len as u32).to_le_bytes())?;
        }
        _ => {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("{} is too long to encode", name),
            ));
        }
    }

    writer.write_all(bytes)
}

pub fn write_account_bin(account: &Account, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    let mut writer = BufWriter::new(file);
    let layout = account_layout();

    for field in layout.fields {
        match field {
            BinaryField::Uuid("id") => {
                writer.write_all(account.id.as_bytes())?;
            }
            BinaryField::U8("account_type") => {
                writer.write_all(&[account_type_to_byte(&account.account_type)])?;
            }
            BinaryField::I64("created_at") => {
                let ts = account.created_at.timestamp();
                writer.write_all(&ts.to_le_bytes())?;
            }
            BinaryField::LengthPrefixed { name, length_type } => {
                let bytes = match name {
                    "name" => account.name.as_bytes(),
                    "system_id" => account.system_id.as_bytes(),
                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            format!("Unknown field in layout: {}", name),
                        ));
                    }
                };

                write_length_prefixed_field(&mut writer, bytes, name, length_type)?;
            }
            other => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Unexpected binary field in Account layout: {:?}", other),
                ));
            }
        }
    }

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
    let layout = system_layout();

    for field in layout.fields {
        match field {
            BinaryField::LengthPrefixed { name, length_type } => {
                let bytes = match name {
                    "system_id" => system.id.as_bytes(),
                    "description" => system.description.as_bytes(),
                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            format!("Unknown field in layout: {}", name),
                        ));
                    }
                };

                write_length_prefixed_field(&mut writer, bytes, name, length_type)?;
            }
            other => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Unexpected binary field in System layout: {:?}", other),
                ));
            }
        }
    }

    Ok(())
}

pub fn write_conversion_graph_bin(graph: &ConversionGraph, path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    
    let mut writer = BufWriter::new(file);
    let layout = conversion_graph_layout();

    for field in layout.fields {
        match field {
            BinaryField::LengthPrefixed { name, length_type } => {
                let bytes = match name {
                    "graph" => {
                        let key_class = classify_graph_key(graph);

                        match key_class {
                            "active" => {
                                format!("C[{}]", graph.graph).into_bytes()
                            }
                            "historical" => {
                                format!("H[{}]", graph.graph).into_bytes()
                            }
                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidInput,
                                    format!("Unknown graph key class: {}", key_class),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            format!("Unknown field in layout: {}", name),
                        ));
                    }
                };

                write_length_prefixed_field(&mut writer, &bytes, name, length_type)?;
            }
            BinaryField::F64("rate") => {
                writer.write_all(&graph.rate.to_le_bytes())?;
            }
            BinaryField::I64("rate_since") => {
                let timestamp = graph.rate_since.timestamp();
                writer.write_all(&timestamp.to_le_bytes())?;
            }
            other => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Unexpected binary field in ConversionGraph layout: {:?}", other),
                ));
            }
        }
    }

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

        let buf = file.fill_buf()?;
        if buf.len() < 1 {
            break;
        }

        // if it's not a current/active conversion graph, skip
        let tag = buf[0];
        if tag != b'C' {
            let skip_by = graph_len + 8 + 8;
            file.consume(skip_by);
            continue;
        }

        let mut graph_buf = vec![0u8; graph_len];
        file.read_exact(&mut graph_buf)?;
        let graph_with_key = String::from_utf8(graph_buf).unwrap_or_default();

        let re = Regex::new(r"C\[(.*?)\]").unwrap();

        let mut graph = String::new();

        if let Some(caps) = re.captures(&graph_with_key) {
            if let Some(g) = caps.get(1) {
                graph = g.as_str().to_string();
            }
        }

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

pub fn compute_object_size(layout: &BinaryLayout, data: &[u8], offset: usize) -> std::io::Result<usize> {
    let mut cursor = offset;
    let mut total_size = 0;

    for field in &layout.fields {
        match field {
            BinaryField::Uuid(_) => {
                total_size += 16;
                cursor += 16;
            }
            BinaryField::U8(_) => {
                total_size += 1;
                cursor += 1;
            }
            BinaryField::U32(_) => {
                total_size += 4;
                cursor += 4;
            }
            BinaryField::I64(_) | BinaryField::F64(_) => {
                total_size += 8;
                cursor += 8;
            }
            BinaryField::LengthPrefixed { name: _, length_type } => {
                let len_size = length_type.byte_len();

                if cursor + len_size > data.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Not enough data for length prefix",
                    ));
                }

                let length = match length_type {
                    LengthType::U8 => data[cursor] as usize,
                    LengthType::U16 => {
                        let mut buf = [0u8; 2];
                        buf.copy_from_slice(&data[cursor..cursor + 2]);
                        u16::from_le_bytes(buf) as usize
                    }
                    LengthType::U32 => {
                        let mut buf = [0u8; 4];
                        buf.copy_from_slice(&data[cursor..cursor + 4]);
                        u32::from_le_bytes(buf) as usize
                    }
                };

                total_size += len_size + length;
                cursor += len_size + length;
            }
        }
    }

    Ok(total_size)
}
