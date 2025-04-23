use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::{
    account_layout, conversion_graph_layout, entry_layout, system_layout, transaction_layout,
    BinaryStorage, TombstoneReader, TombstoneWriter
};
use crate::util::uuid::generate_deterministic_uuid;
use crate::index::BTreeIndex;
use crate::model::{Transaction, Entry, Account, System, ConversionGraph};

static ACCOUNT_BIN_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/accounts.bin"));
static TRANSACTION_BIN_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/transactions.bin"));
static ENTRY_BIN_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/entries.bin"));
static SYSTEM_BIN_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/systems.bin"));
static CONVERSION_GRAPH_BIN_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/conversion_graphs.bin"));

static ACCOUNT_IDX_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/accounts.idx"));
static TRANSACTION_IDX_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/transactions.idx"));
static ENTRY_IDX_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/entries.idx"));
static SYSTEM_IDX_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/systems.idx"));
static CONVERSION_GRAPH_IDX_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("data/conversion_graphs.idx"));

#[derive(Debug)]
pub struct Ledger {
    pub storage: BinaryStorage,

    pub accounts: HashMap<Uuid, Account>,
    pub transactions: HashMap<Uuid, Transaction>,
    pub entries: Vec<Entry>,
    pub systems: HashMap<Uuid, System>,
    pub conversion_graphs: HashMap<Uuid, ConversionGraph>,

    pub account_index: BTreeIndex,
    pub transaction_index: BTreeIndex,
    pub entry_index: BTreeIndex,
    pub system_index: BTreeIndex,
    pub conversion_graph_index: BTreeIndex,
}

impl Ledger {
    pub fn load_from_disk() -> std::io::Result<Self> {
        let start = std::time::Instant::now();

        // ---------------------------------------------------------------------------------

        let mut readers = HashMap::new();
        let mut writers = HashMap::new();
        let mut layouts = HashMap::new();

        readers.insert("accounts".to_string(), BufReader::new(File::open(*ACCOUNT_BIN_PATH).unwrap()));
        readers.insert("transactions".to_string(), BufReader::new(File::open(*TRANSACTION_BIN_PATH).unwrap()));
        readers.insert("entries".to_string(), BufReader::new(File::open(*ENTRY_BIN_PATH).unwrap()));
        readers.insert("systems".to_string(), BufReader::new(File::open(*SYSTEM_BIN_PATH).unwrap()));
        readers.insert("conversion_graphs".to_string(), BufReader::new(File::open(*CONVERSION_GRAPH_BIN_PATH).unwrap()));

        writers.insert("accounts".to_string(), BufWriter::new(OpenOptions::new().append(true).create(true).open(*ACCOUNT_BIN_PATH)?));
        writers.insert("transactions".to_string(), BufWriter::new(OpenOptions::new().append(true).create(true).open(*TRANSACTION_BIN_PATH)?));
        writers.insert("entries".to_string(), BufWriter::new(OpenOptions::new().append(true).create(true).open(*ENTRY_BIN_PATH)?));
        writers.insert("systems".to_string(), BufWriter::new(OpenOptions::new().append(true).create(true).open(*SYSTEM_BIN_PATH)?));
        writers.insert("conversion_graphs".to_string(), BufWriter::new(OpenOptions::new().append(true).create(true).open(*CONVERSION_GRAPH_BIN_PATH)?));

        layouts.insert("accounts".to_string(), account_layout());
        layouts.insert("transactions".to_string(), transaction_layout());
        layouts.insert("entries".to_string(), entry_layout());
        layouts.insert("systems".to_string(), system_layout());
        layouts.insert("conversion_graphs".to_string(), conversion_graph_layout());

        let storage = BinaryStorage::new(readers, writers, layouts);

        let accounts_list = storage.read::<Account>()?;
        let transactions_list = storage.read::<Transaction>()?;
        let entries_list = storage.read::<Entry>()?;
        let systems_list = storage.read::<System>()?;
        let conversion_graphs_list = storage.read::<ConversionGraph>()?;

        let accounts: HashMap<Uuid, Account> = accounts_list.into_iter().map(|account| (account.id, account)).collect();
        let transactions: HashMap<Uuid, Transaction> = transactions_list.into_iter().map(|transaction| (transaction.id, transaction)).collect();
        let systems: HashMap<Uuid, System> = systems_list.into_iter().map(|system| {
            let uuid = generate_deterministic_uuid(&system.id);
            (uuid, system)
        }).collect();
        let conversion_graphs: HashMap<Uuid, ConversionGraph> = conversion_graphs_list.into_iter().map(|graph| {
            let uuid = generate_deterministic_uuid(&graph.graph);
            (uuid, graph)
        }).collect();

        // ---------------------------------------------------------------------------------


        let duration = start.elapsed();
        println!("Completed loading ledger data. Took: {:?}", duration);

        Ok(Self {
            storage: storage,

            accounts,
            transactions,
            systems,
            conversion_graphs,
            entries: entries_list,

            account_index: BTreeIndex::load(*ACCOUNT_IDX_PATH)?,
            transaction_index: BTreeIndex::load(*TRANSACTION_IDX_PATH)?,
            entry_index: BTreeIndex::load(*ENTRY_IDX_PATH)?,
            system_index: BTreeIndex::load(*SYSTEM_IDX_PATH)?,
            conversion_graph_index: BTreeIndex::load(*CONVERSION_GRAPH_IDX_PATH)?,
        })
    }

    pub fn persist_indexes(&self) -> std::io::Result<()> {
        self.account_index.persist(*ACCOUNT_IDX_PATH)?;
        self.transaction_index.persist(*TRANSACTION_IDX_PATH)?;
        self.entry_index.persist(*ENTRY_IDX_PATH)?;
        self.system_index.persist(*SYSTEM_IDX_PATH)?;
        self.conversion_graph_index.persist(*CONVERSION_GRAPH_IDX_PATH)?;
        Ok(())
    }

    pub fn create_account(&mut self, account: Account) -> std::io::Result<()> {
        let (offset, account) = self.storage.write(account)?;

        let uuid = generate_deterministic_uuid(&account.id);
        self.accounts.insert(uuid, account);
        self.account_index.insert(uuid, offset);

        Ok(())
    }

    pub fn create_system(&mut self, system: System) -> std::io::Result<()> {
        let (offset, system) = self.storage.write(system)?;

        let uuid = generate_deterministic_uuid(&system.id);
        self.systems.insert(uuid, system);
        self.system_index.insert(uuid, offset);

        Ok(())
    }

    /// Archives an existing conversion graph by appending it to history with a time range key
    fn archive_conversion_graph(&mut self, graph: &ConversionGraph, expired_at: DateTime<Utc>) -> std::io::Result<()> {
        // Create historical version of the old graph
        let historical_graph = ConversionGraph {
            graph: format!("{}[{}]{}", 
                graph.rate_since.to_rfc3339(),
                graph.graph,
                expired_at.to_rfc3339()
            ),
            rate: graph.rate,
            rate_since: graph.rate_since,
        };

        println!("Archiving conversion graph: {:?}", historical_graph);

        // Get the old graph's offset from index and zero out old record if it exists
        let old_uuid = generate_deterministic_uuid(&graph.graph);
        if let Some(offset) = self.conversion_graph_index.get(&old_uuid) {
            self.storage.tombstone(graph.clone(), offset)?;
        }

        // Append historical version to storage
        let (offset, historical_graph) = self.storage.write(historical_graph)?;
        
        // Update in-memory map - insert or update
        let historical_uuid = generate_deterministic_uuid(&historical_graph.graph);
        self.conversion_graph_index.insert(historical_uuid, offset);
        self.conversion_graphs.insert(historical_uuid, historical_graph);
        
        Ok(())
    }

    /// Creates a conversion relationship between systems based on the graph string format.
    /// Accepts formats:
    /// - One-way: "USD -> IDR" or "USD <- IDR"
    /// - Two-way: "USD <-> SGD"
    pub fn create_conversion_graph(&mut self, mut graph: ConversionGraph) -> std::io::Result<()> {
        // Parse the graph string to get systems and direction
        let parts: Vec<&str> = graph.graph.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid graph format: {}", graph.graph),
            ));
        }

        let (from_system, direction, to_system) = (parts[0], parts[1], parts[2]);
        
        // Validate both systems exist
        let from_uuid = generate_deterministic_uuid(&from_system);
        let to_uuid = generate_deterministic_uuid(&to_system);

        if !self.systems.contains_key(&from_uuid) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("source system not found: {}", from_system),
            ));
        }
        if !self.systems.contains_key(&to_uuid) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("target system not found: {}", to_system),
            ));
        }

        let now = Utc::now();

        match direction {
            "->" => {
                // Check if conversion already exists
                let graph_key = format!("{} -> {}", from_system, to_system);
                let existing_uuid = generate_deterministic_uuid(&graph_key);
                
                // Clone existing graph if it exists, then archive it
                if let Some(existing) = self.conversion_graphs.get(&existing_uuid).cloned() {
                    self.archive_conversion_graph(&existing, now)?;
                }
                
                // Create new conversion
                graph.graph = graph_key;
                graph.rate_since = now;
                let uuid = generate_deterministic_uuid(&graph.graph);
                let (offset, graph) = self.storage.write(graph)?;
                self.conversion_graphs.entry(uuid).and_modify(|e| *e = graph.clone()).or_insert(graph);
                self.conversion_graph_index.insert(uuid, offset);
            }
            "<-" => {
                // Check if conversion already exists
                let graph_key = format!("{} -> {}", to_system, from_system);
                let existing_uuid = generate_deterministic_uuid(&graph_key);
                
                // Clone existing graph if it exists, then archive it
                if let Some(existing) = self.conversion_graphs.get(&existing_uuid).cloned() {
                    self.archive_conversion_graph(&existing, now)?;
                }
                
                // Create new conversion
                graph.graph = graph_key;
                graph.rate_since = now;
                let uuid = generate_deterministic_uuid(&graph.graph);
                let (offset, graph) = self.storage.write(graph)?;
                self.conversion_graphs.entry(uuid).and_modify(|e| *e = graph.clone()).or_insert(graph);
                self.conversion_graph_index.insert(uuid, offset);
            }
            "<->" => {
                // Check and archive both directions if they exist
                let forward_key = format!("{} -> {}", from_system, to_system);
                let reverse_key = format!("{} -> {}", to_system, from_system);
                
                let forward_uuid = generate_deterministic_uuid(&forward_key);
                let reverse_uuid = generate_deterministic_uuid(&reverse_key);
                
                // Clone and archive existing conversions
                if let Some(existing) = self.conversion_graphs.get(&forward_uuid).cloned() {
                    self.archive_conversion_graph(&existing, now)?;
                }
                if let Some(existing) = self.conversion_graphs.get(&reverse_uuid).cloned() {
                    self.archive_conversion_graph(&existing, now)?;
                }

                // Create new bidirectional conversions
                let forward = ConversionGraph {
                    graph: forward_key,
                    rate: graph.rate,
                    rate_since: graph.rate_since,
                };
                let uuid = generate_deterministic_uuid(&forward.graph);
                let (offset, forward) = self.storage.write(forward)?;
                self.conversion_graphs.entry(uuid).and_modify(|e| *e = forward.clone()).or_insert(forward);
                self.conversion_graph_index.insert(uuid, offset);

                let reverse = ConversionGraph {
                    graph: reverse_key,
                    rate: 1.0 / graph.rate,
                    rate_since: graph.rate_since,
                };
                let uuid = generate_deterministic_uuid(&reverse.graph);
                let (offset, reverse) = self.storage.write(reverse)?;
                self.conversion_graphs.entry(uuid).and_modify(|e| *e = reverse.clone()).or_insert(reverse);
                self.conversion_graph_index.insert(uuid, offset);
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid direction: {}. must be ->, <-, or <->", direction),
                ));
            }
        }

        Ok(())
    }

    pub fn record_transaction(&mut self, tx: Transaction, entries: Vec<Entry>) -> std::io::Result<()> {
        let sum: f64 = entries.iter().map(|e| e.amount).sum();
        if sum.abs() > f64::EPSILON {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unbalanced transaction: total = {}", sum),
            ));
        }

        let mut system_entries: HashMap<Uuid, Vec<&Entry>> = HashMap::new();
        for entry in entries.iter() {
            let acc_uuid = generate_deterministic_uuid(&entry.account_id);

            let account = self.accounts.get(&acc_uuid).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("account not found: {}", entry.account_id),
                )
            })?;

            let uuid = generate_deterministic_uuid(&account.id);
            
            system_entries
                .entry(uuid)
                .or_default()
                .push(entry);
        }

        for (system_id, entries) in system_entries.iter() {
            if !self.systems.contains_key(&system_id) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("system not found: {}", system_id),
                ));
            }

            let system_sum: f64 = entries.iter().map(|e| e.amount).sum();
            if system_sum.abs() > f64::EPSILON {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unbalanced entries in system {}: sum = {}", system_id, system_sum),
                ));
            }
        }

        for entry in entries.iter() {
            let (offset, entry) = self.storage.write(entry.clone())?;

            self.entry_index.insert(generate_deterministic_uuid(&entry.id), offset);
            self.entries.push(entry);
        }

        let (offset, tx) = self.storage.write(tx)?;
        self.transaction_index.insert(generate_deterministic_uuid(&tx.id), offset);
        self.transactions.insert(tx.id, tx);
        Ok(())
    }
}