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
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) else {
        return Err("Failed to open file".into());
    };
    // Check if file is empty (just created or was deleted)
    for record in records {
        let json_line = record.borrow().to_jsonl_string()?;
        writeln!(file, "{json_line}")?;
    }
    file.flush()?;
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
