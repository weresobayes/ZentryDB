use uuid::Uuid;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct BTreeIndex {
    tree: BTreeMap<Uuid, u64>,
}

impl BTreeIndex {
    pub fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, id: Uuid, offset: u64) {
        self.tree.insert(id, offset);
    }

    pub fn get(&self, id: &Uuid) -> Option<u64> {
        self.tree.get(id).copied()
    }

    pub fn range(&self, start: &Uuid, end: &Uuid) -> Vec<(Uuid, u64)> {
        self.tree.range(start.clone()..end.clone()).map(|(k, v)| (*k, *v)).collect()
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn persist(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        for (id, offset) in &self.tree {
            file.write_all(id.as_bytes())?;
            file.write_all(&offset.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        use std::io::{Read, BufReader};
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut tree = BTreeMap::new();
        let mut buf = [0u8; 24];

        while reader.read_exact(&mut buf).is_ok() {
            let id = Uuid::from_bytes(buf[0..16].try_into().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID bytes")
            })?);
            let offset = u64::from_le_bytes(buf[16..24].try_into().unwrap());
            tree.insert(id, offset);
        }

        Ok(Self { tree })
    }

}