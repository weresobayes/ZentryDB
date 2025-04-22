use std::{fs::OpenOptions, io::{BufRead, BufReader, Seek, BufWriter, SeekFrom, Write}};
use std::path::Path;

use serde_json::{from_str, to_string};

use crate::index::BTreeIndex;
use crate::model::System;
use crate::storage::binary::write_system_bin;
use crate::util::uuid::generate_deterministic_uuid;

pub fn write_system_bin_and_index(
    system: &System,
    bin_path: &Path,
    index: &mut BTreeIndex,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(bin_path)?;

    write_system(system)?;

    let offset = file.seek(SeekFrom::End(0))?;
    write_system_bin(system, bin_path)?;
    
    let uuid = generate_deterministic_uuid(&system.id);
    index.insert(uuid, offset);

    Ok(())
}

pub fn write_system(system: &System) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/systems.jsonl")?;
    
    let mut writer = BufWriter::new(file);

    let json = to_string(system)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

pub fn load_systems() -> std::io::Result<Vec<System>> {
    let file = OpenOptions::new()
        .read(true)
        .open("data/systems.jsonl")?;
    
    let reader = BufReader::new(file);

    reader.lines().map(|line| {
        let line = line?;
        let system: System = from_str(&line)?;

        Ok(system)
    }).collect()
}
