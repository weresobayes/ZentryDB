use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Seek, Write, SeekFrom}};
use std::path::Path;

use serde_json::{from_str, to_string};

use crate::{index::BTreeIndex, read_entries_bin};
use crate::model::Entry;
use crate::storage::binary::write_entry_bin;

pub fn write_entry_bin_and_index(entry: &Entry, path: &Path, index: &mut BTreeIndex) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    let entries = [entry.clone()];

    write_entries(&entries)?;

    let offset = file.seek(SeekFrom::End(0))?;
    write_entry_bin(entry, path)?;
    index.insert(entry.id, offset);
    
    Ok(())
}

pub fn write_entries(entries: &[Entry]) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/entries.jsonl")?;

    let mut writer = BufWriter::new(file);

    for entry in entries {
        writeln!(writer, "{}", to_string(entry)?)?;
    }

    Ok(())
}

pub fn load_entries() -> std::io::Result<Vec<Entry>> {
    let file = OpenOptions::new()
        .read(true)
        .open("data/entries.jsonl")?;
    
    let reader = BufReader::new(file);

    reader.lines().map(|line| {
        let line = line?;
        let entry: Entry = from_str(&line)?;

        Ok(entry)
    }).collect()
}

pub fn load_entries_from_bin(bin_path: &Path) -> std::io::Result<Vec<Entry>> {
    let start = std::time::Instant::now();
    let result = read_entries_bin(bin_path);
    let duration = start.elapsed();
    println!("Loading entries took: {:?}", duration);
    result
}