use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::collections::HashMap;

use regex::Regex;
use uuid::Uuid;
use chrono::{TimeZone, Utc};

use crate::model::{Account, AccountType, Entry, Transaction, System, ConversionGraph};
use crate::storage::layout::{BinaryLayout, BinaryField, LengthType};

use bimap::BiMap;
use once_cell::sync::Lazy;

static ACCOUNT_TYPE_BIMAP: Lazy<BiMap<u8, AccountType>> = Lazy::new(|| {
    let mut map = BiMap::new();

    map.insert(0u8, AccountType::Asset);
    map.insert(1u8, AccountType::Liability);
    map.insert(2u8, AccountType::Equity);
    map.insert(3u8, AccountType::Revenue);
    map.insert(4u8, AccountType::Expense);

    map
});

fn account_type_from_u8(byte: u8) -> Option<AccountType> {
    ACCOUNT_TYPE_BIMAP.get_by_left(&byte).cloned()
}

fn account_type_to_u8(account_type: &AccountType) -> Option<u8> {
    ACCOUNT_TYPE_BIMAP.get_by_right(account_type).cloned()
}

enum BinaryReadError {
    DeadRecord,
}

enum BinaryWriteError {
    TryingToTombstoneWrongRecord,
}

impl From<BinaryReadError> for std::io::Error {
    fn from(error: BinaryReadError) -> Self {
        match error {
            BinaryReadError::DeadRecord => std::io::Error::new(ErrorKind::Other, "dead record"),
        }
    }
}

impl From<BinaryWriteError> for std::io::Error {
    fn from(error: BinaryWriteError) -> Self {
        match error {
            BinaryWriteError::TryingToTombstoneWrongRecord => std::io::Error::new(ErrorKind::Other, "trying to tombstone wrong record"),
        }
    }
}

pub trait TombstoneReader {
    fn is_ignorable_error(&self, e: &std::io::Error) -> bool;

    fn is_tombstone_byte(&self, byte: u8) -> bool {
        byte == 0x00
    }

    fn read_or_skip<T>(&self) -> std::io::Result<T>
    where
        T: FromBinary;

    fn read<T>(&self) -> std::io::Result<Vec<T>>
    where
        T: FromBinary;
}

pub trait TombstoneWriter {
    fn tombstone<T>(&self, item: T, offset: u64) -> std::io::Result<()>
    where
        T: FromBinary + PartialEq;

    fn write<T>(&self, item: T) -> std::io::Result<(u64, T)>
    where
        T: ToBinary;
}

pub trait ToBinary {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()>;
}

pub trait FromBinary {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self>
    where
        Self: Sized;

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct BinaryStorage {
    readers: RefCell<HashMap<String, BufReader<File>>>,
    writers: RefCell<HashMap<String, BufWriter<File>>>,
    layouts: HashMap<String, BinaryLayout>,
}

impl BinaryStorage {
    pub fn new(readers: HashMap<String, BufReader<File>>, writers: HashMap<String, BufWriter<File>>, layouts: HashMap<String, BinaryLayout>) -> Self {
        Self {
            readers: RefCell::new(readers),
            writers: RefCell::new(writers),
            layouts,
        }
    }

    pub fn read_single<T>(&self, offset: u64) -> std::io::Result<T>
    where
        T: FromBinary,
    {
        let type_key = match std::any::type_name::<T>() {
            t if t.contains("Account") => "accounts",
            t if t.contains("Transaction") => "transactions",
            t if t.contains("Entry") => "entries",
            t if t.contains("System") => "systems",
            t if t.contains("ConversionGraph") => "conversion_graphs",
            _ => return Err(std::io::Error::new(ErrorKind::Other, "unsupported type"))
        };

        let mut readers = self.readers.borrow_mut();
        let reader = readers.get_mut(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no reader found for type"))?;

        let layout = self.layouts.get(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no layout found for type"))?;

        reader.seek(SeekFrom::Start(offset))?;

        let mut tombstone_buf = [0u8; 1];
        reader.read_exact(&mut tombstone_buf)?;

        if self.is_tombstone_byte(tombstone_buf[0]) {
            return Err(BinaryReadError::DeadRecord.into())
        }

        T::from_binary(reader, layout)
    }
}

impl FromBinary for Account {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self> {
        let mut id = Uuid::nil();
        let mut name = String::new();
        let mut account_type = AccountType::Asset;
        let mut created_at = Utc.timestamp_opt(0, 0).unwrap();
        let mut system_id = String::new();

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    let mut buf = [0u8; 16];
                    reader.read_exact(&mut buf)?;
                    id = Uuid::from_bytes(buf);
                }
                BinaryField::LengthPrefixed { length_type, name: "name" } => {
                    name = read_length_prefixed_string(reader, length_type)?;
                }
                BinaryField::U8("account_type") => {
                    let mut buf = [0u8; 1];
                    reader.read_exact(&mut buf)?;
                    account_type = account_type_from_u8(buf[0])
                        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "unknown account type"))?;
                }
                BinaryField::I64("created_at") => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    let ts = i64::from_le_bytes(buf);
                    created_at = Utc.timestamp_opt(ts, 0).unwrap();
                }
                BinaryField::LengthPrefixed { length_type, name: "system_id" } => {
                    system_id = read_length_prefixed_string(reader, length_type)?;
                }
                _ => {}
            }
        }
        Ok(Account { id, name, account_type, created_at, system_id })
    }

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        for field in &layout.fields {
            match field {
                BinaryField::Uuid(_) => { let mut buf = [0u8; 16]; reader.read_exact(&mut buf)?; }
                BinaryField::U8(_) => { let mut buf = [0u8; 1]; reader.read_exact(&mut buf)?; }
                BinaryField::I64(_) => { let mut buf = [0u8; 8]; reader.read_exact(&mut buf)?; }
                BinaryField::LengthPrefixed { length_type, .. } => { let _ = read_length_prefixed_string(reader, length_type)?; }
                _ => {}
            }
        }
        Ok(())
    }
}

impl FromBinary for Transaction {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self> {
        let mut id = Uuid::nil();
        let mut description = String::new();
        let mut metadata: Option<serde_json::Value> = None;
        let mut timestamp = Utc.timestamp_opt(0, 0).unwrap();

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    let mut buf = [0u8; 16];
                    reader.read_exact(&mut buf)?;
                    id = Uuid::from_bytes(buf);
                }
                BinaryField::LengthPrefixed { length_type, name: "description" } => {
                    description = read_length_prefixed_string(reader, length_type)?;
                }
                BinaryField::LengthPrefixed { length_type, name: "metadata" } => {
                    let json_str = read_length_prefixed_string(reader, length_type)?;
                    metadata = if json_str.is_empty() { None } else { serde_json::from_str(&json_str).ok() };
                }
                BinaryField::I64("timestamp") => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    let ts = i64::from_le_bytes(buf);
                    timestamp = Utc.timestamp_opt(ts, 0).unwrap();
                }
                _ => {}
            }
        }
        Ok(Transaction { id, description, timestamp, metadata })
    }

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        for field in &layout.fields {
            match field {
                BinaryField::Uuid(_) => { let mut buf = [0u8; 16]; reader.read_exact(&mut buf)?; }
                BinaryField::I64(_) => { let mut buf = [0u8; 8]; reader.read_exact(&mut buf)?; }
                BinaryField::LengthPrefixed { length_type, .. } => { let _ = read_length_prefixed_string(reader, length_type)?; }
                _ => {}
            }
        }
        Ok(())
    }
}

impl FromBinary for Entry {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self> {
        let mut id = Uuid::nil();
        let mut transaction_id = Uuid::nil();
        let mut account_id = Uuid::nil();
        let mut amount = 0.0;

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    let mut buf = [0u8; 16];
                    reader.read_exact(&mut buf)?;
                    id = Uuid::from_bytes(buf);
                }
                BinaryField::Uuid("transaction_id") => {
                    let mut buf = [0u8; 16];
                    reader.read_exact(&mut buf)?;
                    transaction_id = Uuid::from_bytes(buf);
                }
                BinaryField::Uuid("account_id") => {
                    let mut buf = [0u8; 16];
                    reader.read_exact(&mut buf)?;
                    account_id = Uuid::from_bytes(buf);
                }
                BinaryField::F64("amount") => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    amount = f64::from_le_bytes(buf);
                }
                _ => {}
            }
        }
        Ok(Entry { id, transaction_id, account_id, amount })
    }

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        for field in &layout.fields {
            match field {
                BinaryField::Uuid(_) => { let mut buf = [0u8; 16]; reader.read_exact(&mut buf)?; }
                BinaryField::F64(_) => { let mut buf = [0u8; 8]; reader.read_exact(&mut buf)?; }
                _ => {}
            }
        }
        Ok(())
    }
}

impl FromBinary for System {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self> {
        let mut id = String::new();
        let mut description = String::new();

        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { length_type, name: "system_id" } => {
                    id = read_length_prefixed_string(reader, length_type)?;
                }
                BinaryField::LengthPrefixed { length_type, name: "description" } => {
                    description = read_length_prefixed_string(reader, length_type)?;
                }
                _ => {}
            }
        }
        Ok(System { id, description })
    }

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { length_type, .. } => { let _ = read_length_prefixed_string(reader, length_type)?; }
                _ => {}
            }
        }
        Ok(())
    }
}

impl FromBinary for ConversionGraph {
    fn from_binary(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<Self> {
        let mut graph = String::new();
        let mut rate = 0.0;
        let mut rate_since = Utc::now();

        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { length_type, name: "graph" } => {
                    let len = match length_type {
                        LengthType::U8 => {
                            let mut len_buf = [0u8; 1];
                            reader.read_exact(&mut len_buf)?;
                            len_buf[0] as usize
                        }
                        LengthType::U16 => {
                            let mut len_buf = [0u8; 2];
                            reader.read_exact(&mut len_buf)?;
                            u16::from_le_bytes(len_buf) as usize
                        }
                        LengthType::U32 => {
                            let mut len_buf = [0u8; 4];
                            reader.read_exact(&mut len_buf)?;
                            u32::from_le_bytes(len_buf) as usize
                        }
                    };

                    let mut tag_buf = [0u8; 1];
                    reader.read_exact(&mut tag_buf)?;
                    let tag = tag_buf[0];

                    if tag != b'C' {
                        let skip_bytes = len + 8 + 8 - 1;
                        let mut skip_buf = vec![0u8; skip_bytes];
                        reader.read_exact(&mut skip_buf)?;

                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "not converting historical conversion graph, will come in improvement later",
                        ));
                    }

                    // Read the rest of the graph string (already read 1 byte for tag)
                    let mut graph_buf = vec![0u8; len - 1];
                    reader.read_exact(&mut graph_buf)?;
                    let graph_with_key = String::from_utf8([vec![tag], graph_buf].concat()).unwrap_or_default();

                    // Extract the value inside C[...]
                    let re = Regex::new(r"C\[(.*?)\]").unwrap();
                    if let Some(caps) = re.captures(&graph_with_key) {
                        if let Some(g) = caps.get(1) {
                            graph = g.as_str().to_string();
                        }
                    }
                }
                BinaryField::F64("rate") => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    rate = f64::from_le_bytes(buf);
                }
                BinaryField::I64("rate_since") => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    let timestamp = i64::from_le_bytes(buf);
                    rate_since = chrono::Utc.timestamp_opt(timestamp, 0).unwrap();
                }
                _ => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "invalid field for `ConversionGraph`",
                    ));
                }
            }
        }

        Ok(ConversionGraph { graph, rate, rate_since })
    }

    fn skip_bytes(reader: &mut BufReader<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { length_type, .. } => {
                    let len = match length_type {
                        LengthType::U8 => {
                            let mut len_buf = [0u8; 1];
                            reader.read_exact(&mut len_buf)?;
                            len_buf[0] as usize
                        }
                        LengthType::U16 => {
                            let mut len_buf = [0u8; 2];
                            reader.read_exact(&mut len_buf)?;
                            u16::from_le_bytes(len_buf) as usize
                        }
                        LengthType::U32 => {
                            let mut len_buf = [0u8; 4];
                            reader.read_exact(&mut len_buf)?;
                            u32::from_le_bytes(len_buf) as usize
                        }
                    };
                    // Skip the actual field data
                    let mut skip_buf = vec![0u8; len];
                    reader.read_exact(&mut skip_buf)?;
                }
                BinaryField::F64 { .. } => {
                    let mut skip_buf = [0u8; 8];
                    reader.read_exact(&mut skip_buf)?;
                }
                BinaryField::I64 { .. } => {
                    let mut skip_buf = [0u8; 8];
                    reader.read_exact(&mut skip_buf)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl TombstoneReader for BinaryStorage {
    fn is_ignorable_error(&self, e: &std::io::Error) -> bool {
        let msg = e.to_string();

        matches!(
            msg.as_str(), 
            m if m.contains("dead record")
                || m.contains("not enough data")
                || m.contains("not converting historical conversion graph")
        )
    }

    fn read<T>(&self) -> std::io::Result<Vec<T>>
    where
        T: FromBinary,
    {
        let mut items = Vec::new();

        loop {
            match self.read_or_skip::<T>() {
                Ok(i) => items.push(i),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    if self.is_ignorable_error(&e) {
                        continue;
                    }

                    return Err(e);
                }
            }
        }

        Ok(items)
    }

    fn read_or_skip<T>(&self) -> std::io::Result<T>
    where
        T: FromBinary,
    {
        let type_key = match std::any::type_name::<T>() {
            t if t.contains("Account") => "accounts",
            t if t.contains("Transaction") => "transactions",
            t if t.contains("Entry") => "entries",
            t if t.contains("System") => "systems",
            t if t.contains("ConversionGraph") => "conversion_graphs",
            _ => return Err(std::io::Error::new(ErrorKind::Other, "unsupported type"))
        };

        let mut readers = self.readers.borrow_mut();
        let reader = readers.get_mut(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no reader found for type"))?;

        let layout = self.layouts.get(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no layout found for type"))?;

        let reader_current_offset = reader.stream_position()?;

        let mut tombstone_buf = [0u8; 1];
        reader.read_exact(&mut tombstone_buf)?;

        if self.is_tombstone_byte(tombstone_buf[0]) {
            T::skip_bytes(reader, layout)?;
            return Err(BinaryReadError::DeadRecord.into())
        }

        match T::from_binary(reader, layout) {
            Ok(item) => Ok(item),
            Err(e) if e.kind() == ErrorKind::InvalidData && self.is_ignorable_error(&e) => {
                reader.seek(SeekFrom::Start(reader_current_offset))?;
                T::skip_bytes(reader, layout)?;
                Err(e)
            }
            Err(e) => Err(e)
        }
    }
}

impl TombstoneWriter for BinaryStorage {
    fn tombstone<T>(&self, item: T, offset: u64) -> std::io::Result<()>
    where
        T: FromBinary + PartialEq
    {
        let type_key = match std::any::type_name::<T>() {
            t if t.contains("Account") => "accounts",
            t if t.contains("Transaction") => "transactions",
            t if t.contains("Entry") => "entries",
            t if t.contains("System") => "systems",
            t if t.contains("ConversionGraph") => "conversion_graphs",
            _ => return Err(std::io::Error::new(ErrorKind::Other, "unsupported type"))
        };

        let item_from_binary = self.read_single::<T>(offset)?;

        if item_from_binary != item {
            return Err(BinaryWriteError::TryingToTombstoneWrongRecord.into())
        }

        let mut tombstone_buf = [0u8; 1];
        tombstone_buf[0] = 0;

        let mut writers = self.writers.borrow_mut();
        let writer = writers.get_mut(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no writer found for type"))?;

        writer.seek(SeekFrom::Start(offset))?;
        writer.write(&tombstone_buf)?;
        writer.seek(SeekFrom::End(0))?;

        Ok(())
    }

    fn write<T>(&self, item: T) -> std::io::Result<(u64, T)>
    where
        T: ToBinary
    {
        let type_key = match std::any::type_name::<T>() {
            t if t.contains("Account") => "accounts",
            t if t.contains("Transaction") => "transactions",
            t if t.contains("Entry") => "entries",
            t if t.contains("System") => "systems",
            t if t.contains("ConversionGraph") => "conversion_graphs",
            _ => return Err(std::io::Error::new(ErrorKind::Other, "unsupported type"))
        };

        let layouts = self.layouts.borrow();
        let layout = layouts.get(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no layout found for type"))?;

        let mut writers = self.writers.borrow_mut();
        let writer = writers.get_mut(type_key)
            .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "no writer found for type"))?;

        let offset = writer.seek(SeekFrom::End(0))?;

        item.to_binary(writer, layout)?;
        Ok((offset, item))
    }
}

impl ToBinary for System {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        // write living record
        writer.write_all(&[1u8; 1])?;

        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { name, length_type } => {
                    let bytes = match *name {
                        "system_id" => self.id.as_bytes(),
                        "description" => self.description.as_bytes(),
                        _ => {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidInput,
                                format!("unknown field in layout: {}", name),
                            ));
                        }
                    };

                    write_length_prefixed_field(writer, bytes, name, length_type)?;
                }
                other => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("unexpected binary field in `System` layout: {:?}", other),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl ToBinary for ConversionGraph {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        // write living record
        writer.write_all(&[1u8; 1])?;

        for field in &layout.fields {
            match field {
                BinaryField::LengthPrefixed { name, length_type } => {
                    let bytes = match *name {
                        "graph" => {
                            let key_class = classify_graph_key(self);

                            match key_class {
                                "active" => {
                                    format!("C[{}]", self.graph).into_bytes()
                                }
                                "historical" => {
                                    format!("H[{}]", self.graph).into_bytes()
                                }
                                _ => {
                                    return Err(std::io::Error::new(
                                        ErrorKind::InvalidInput,
                                        format!("unknown graph key class: {}", key_class),
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidInput,
                                format!("unknown field in layout: {}", name),
                            ));
                        }
                    };

                    write_length_prefixed_field(writer, &bytes, name, &length_type)?;
                }
                BinaryField::F64("rate") => {
                    writer.write_all(&self.rate.to_le_bytes())?;
                }
                BinaryField::I64("rate_since") => {
                    let timestamp = self.rate_since.timestamp();
                    writer.write_all(&timestamp.to_le_bytes())?;
                }
                other => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("unexpected binary field in `ConversionGraph` layout: {:?}", other),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl ToBinary for Entry {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        // write living record
        writer.write_all(&[1u8; 1])?;

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    writer.write_all(self.id.as_bytes())?;
                }
                BinaryField::Uuid("transaction_id") => {
                    writer.write_all(self.transaction_id.as_bytes())?;
                }
                BinaryField::Uuid("account_id") => {
                    writer.write_all(self.account_id.as_bytes())?;
                }
                BinaryField::I64("amount") => {
                    writer.write_all(&self.amount.to_le_bytes())?;
                }
                other => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("unexpected binary field in `Entry` layout: {:?}", other),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl ToBinary for Transaction {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        // write living record
        writer.write_all(&[1u8; 1])?;

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    writer.write_all(self.id.as_bytes())?;
                }
                BinaryField::LengthPrefixed { name, length_type } if *name == "description" => {
                    let bytes = self.description.as_bytes();
                    write_length_prefixed_field(writer, bytes, name, length_type)?;
                }
                BinaryField::LengthPrefixed { name, length_type } if *name == "metadata" => {
                    let bytes = match &self.metadata {
                        Some(val) => serde_json::to_vec(val)?,
                        None => Vec::new(),
                    };
                    write_length_prefixed_field(writer, &bytes, name, length_type)?;
                }
                BinaryField::I64("timestamp") => {
                    let timestamp = self.timestamp.timestamp();
                    writer.write_all(&timestamp.to_le_bytes())?;
                }
                other => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("unexpected binary field in `Transaction` layout: {:?}", other),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl ToBinary for Account {
    fn to_binary(&self, writer: &mut BufWriter<File>, layout: &BinaryLayout) -> std::io::Result<()> {
        // write living record
        writer.write_all(&[1u8; 1])?;

        for field in &layout.fields {
            match field {
                BinaryField::Uuid("id") => {
                    writer.write_all(self.id.as_bytes())?;
                }
                BinaryField::U8("account_type") => {
                    writer.write_all(&[account_type_to_u8(&self.account_type).unwrap()])?;
                }
                BinaryField::I64("created_at") => {
                    let ts = self.created_at.timestamp();
                    writer.write_all(&ts.to_le_bytes())?;
                }
                BinaryField::LengthPrefixed { name, length_type } if *name == "name" => {
                    let bytes = self.name.as_bytes();
                    write_length_prefixed_field(writer, bytes, name, length_type)?;
                }
                BinaryField::LengthPrefixed { name, length_type } if *name == "system_id" => {
                    let bytes = self.system_id.as_bytes();
                    write_length_prefixed_field(writer, bytes, name, length_type)?;
                }
                other => {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("unexpected binary field in `Account` layout: {:?}", other),
                    ));
                }
            }
        }
        Ok(())
    }
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
                        "not enough data for length prefix",
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

fn classify_graph_key(graph: &ConversionGraph) -> &'static str {
    let active_conversion_key_pattern = Regex::new(r"^\s*\w+\s*(->|<-|<->)\s*\w+\s*$").unwrap();
    let historical_conversion_key_pattern = Regex::new(
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?[+-]\d{2}:\d{2}\[\s*\w+\s*(->|<-|<->)\s*\w+\s*\]\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?[+-]\d{2}:\d{2}$"
    ).unwrap();

    if historical_conversion_key_pattern.is_match(&graph.graph) {
        return "historical";
    } else if active_conversion_key_pattern.is_match(&graph.graph) {
        return "active";
    }

    "unknown"
}

fn write_length_prefixed_field(writer: &mut BufWriter<File>, bytes: &[u8], name: &str, length_type: &LengthType) -> std::io::Result<()> {
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
                format!("`{}` is too long to encode", name),
            ));
        }
    }

    writer.write_all(bytes)
}

fn read_length_prefixed_string<R: std::io::Read>(reader: &mut R, length_type: &LengthType) -> std::io::Result<String> {
    let len = match length_type {
        LengthType::U8 => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0] as usize
        }
        LengthType::U16 => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            u16::from_le_bytes(buf) as usize
        }
        LengthType::U32 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            u32::from_le_bytes(buf) as usize
        }
    };
    let mut str_buf = vec![0u8; len];
    reader.read_exact(&mut str_buf)?;
    Ok(String::from_utf8(str_buf).unwrap_or_default())
}
