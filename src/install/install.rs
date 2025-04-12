fn create_data_files() -> std::io::Result<()> {
    std::fs::create_dir_all("data")?;
    
    let files = [
        "data/accounts.bin",
        "data/accounts.idx",
        "data/accounts.jsonl",
        "data/entries.bin",
        "data/entries.idx",
        "data/entries.jsonl",
        "data/transactions.bin",
        "data/transactions.idx",
        "data/transactions.jsonl",
        "data/conversion_graphs.bin",
        "data/conversion_graphs.idx",
        "data/conversion_graphs.jsonl",
        "data/systems.bin",
        "data/systems.idx",
        "data/systems.jsonl",
    ];

    for file in &files {
        if std::path::Path::new(file).exists() {
            return Ok(());
        }
    }

    for file in &files {
        std::fs::File::create(file)?;
    }

    Ok(())
}

pub fn install() -> std::io::Result<()> {
    create_data_files()?;
    Ok(())
}
