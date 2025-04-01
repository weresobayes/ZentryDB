use crate::model::Entry;
use std::{fs::OpenOptions, io::{BufWriter, Write, BufReader, BufRead}};
use serde_json::{from_str, to_string};

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
