use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::Path};

use serde_json::{from_str, to_string};

use crate::{index::BTreeIndex, Account, Entry, System, Transaction};
use crate::model::ConversionGraph;
use crate::storage::binary::write_conversion_graph_bin;
use crate::util::uuid::generate_deterministic_uuid;

use super::{read_accounts_bin, read_conversion_graphs_bin, read_entries_bin, read_systems_bin, read_transactions_bin};

pub fn write_conversion_graph_bin_and_index(
    graph: &ConversionGraph,
    bin_path: &Path,
    index: &mut BTreeIndex,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(bin_path)?;

    write_conversion_graph(graph)?;

    let offset = file.seek(SeekFrom::End(0))?;
    write_conversion_graph_bin(graph, bin_path)?;
    
    let uuid = generate_deterministic_uuid(&graph.graph);
    index.insert(uuid, offset);

    Ok(())
}

pub fn write_conversion_graph(graph: &ConversionGraph) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/conversion_graphs.jsonl")?;
    
    let mut writer = BufWriter::new(file);

    let json = to_string(graph)?;
    writeln!(writer, "{}", json)?;

    Ok(())
}

pub fn zero_conversion_graph_at_offset(bin_path: &Path, offset: u64) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(bin_path)?;
    
    file.seek(SeekFrom::Start(offset))?;
    let mut len_buf = [0u8; 1];
    file.read_exact(&mut len_buf)?;
    let graph_len = len_buf[0] as usize;
    
    file.seek(SeekFrom::Start(offset))?;
    let zeros = vec![0u8; 1 + graph_len + 8 + 8];
    file.write_all(&zeros)?;

    Ok(())
}

pub fn load_conversion_graphs() -> std::io::Result<Vec<ConversionGraph>> {
    let file = OpenOptions::new()
        .read(true)
        .open("data/conversion_graphs.jsonl")?;
    
    let reader = BufReader::new(file);

    reader.lines().map(|line| {
        let line = line?;
        let graph: ConversionGraph = from_str(&line)?;

        Ok(graph)
    }).collect()
}

pub fn load_conversion_graphs_from_bin(bin_path: &Path) -> std::io::Result<Vec<ConversionGraph>> {
    let start = std::time::Instant::now();
    let result = read_conversion_graphs_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading conversion graphs took: {:?}", duration);
    result
}

pub fn load_systems_from_bin(bin_path: &Path) -> std::io::Result<Vec<System>> {
    let start = std::time::Instant::now();
    let result = read_systems_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading systems took: {:?}", duration);
    result
}

pub fn load_accounts_from_bin(bin_path: &Path) -> std::io::Result<Vec<Account>> {
    let start = std::time::Instant::now();
    let result = read_accounts_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading accounts took: {:?}", duration);
    result
}

pub fn load_entries_from_bin(bin_path: &Path) -> std::io::Result<Vec<Entry>> {
    let start = std::time::Instant::now();
    let result = read_entries_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading entries took: {:?}", duration);
    result
}

pub fn load_transactions_from_bin(bin_path: &Path) -> std::io::Result<Vec<Transaction>> {
    let start = std::time::Instant::now();
    let result = read_transactions_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading transactions took: {:?}", duration);
    result
}
    