use std::{fs::OpenOptions, io::{BufRead, BufReader, Seek, BufWriter, SeekFrom, Write}};
use std::path::Path;

use serde_json::{from_str, to_string};

use crate::index::btree::BTreeIndex;
use crate::model::Account;
use crate::storage::binary::write_account_bin;

pub fn write_account_bin_and_index(
    account: &Account,
    bin_path: &Path,
    index: &mut BTreeIndex,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(bin_path)?;

    write_account(account)?;

    let offset = file.seek(SeekFrom::End(0))?;
    write_account_bin(account, bin_path)?;
    index.insert(account.id, offset);

    Ok(())
}

pub fn write_account(account: &Account) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/accounts.jsonl")?;
    
    let mut writer = BufWriter::new(file);

    let json = to_string(account)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

pub fn load_accounts() -> std::io::Result<Vec<Account>> {
    let file = OpenOptions::new()
        .read(true)
        .open("data/accounts.jsonl")?;
    
    let reader = BufReader::new(file);

    reader.lines().map(|line| {
        let line = line?;
        let account: Account = from_str(&line)?;

        Ok(account)
    }).collect()
}