//! Serialization utilities for reading and writing records.

use std::{
    borrow::Borrow,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use crate::api::record::Record;

/// Write a collection of records to a file in JSON-lines format
pub fn write_to_jsonl_file<P: AsRef<Path>, R: Borrow<Record>>(
    file_path: P,
    records: &[R],
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize the complete batch before touching the output, then minimize
    // the number of append operations used for it.
    let mut buf = String::new();
    for record in records {
        buf.push_str(&record.borrow().to_jsonl_string()?);
        buf.push('\n');
    }
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) else {
        return Err("Failed to open file".into());
    };
    file.write_all(buf.as_bytes())?;
    Ok(())
}

/// Read records from a JSON-lines file
pub fn read_jsonl_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&file_path)?;
    let mut records = Vec::new();

    // Parse JSON-lines format: each line is a separate JSON object
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue; // Skip empty lines
        }

        let record: Record = serde_json::from_str(line)
            .map_err(|e| format!("{}:{}: {}", file_path.as_ref().display(), line_num + 1, e))?;

        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests;
