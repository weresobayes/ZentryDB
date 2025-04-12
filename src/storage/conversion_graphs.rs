use std::{fs::OpenOptions, io::{BufRead, BufReader, Seek, BufWriter, SeekFrom, Write}};
use std::path::Path;

use serde_json::{from_str, to_string};

use crate::index::BTreeIndex;
use crate::model::ConversionGraph;
use crate::storage::binary::write_conversion_graph_bin;
use crate::util::uuid::generate_deterministic_uuid;

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
