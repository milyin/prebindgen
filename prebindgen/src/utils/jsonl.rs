//! Serialization utilities for reading and writing records.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::api::record::Record;

/// Write a collection of records to a file in JSON-lines format
#[allow(dead_code)]
pub fn write_jsonl_file<P: AsRef<Path>>(
    file_path: P,
    records: &[Record],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(&file_path)?;
    for record in records {
        let json_line = record.to_jsonl_string()?;
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
mod tests {
    use crate::api::record::RecordKind;

    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_jsonl_round_trip() {
        // Create some test records
        let records = vec![
            Record {
                kind: RecordKind::Struct,
                name: "TestStruct".to_string(),
                content: "pub struct TestStruct { }".to_string(),
                source_location: Default::default(),
            },
            Record {
                kind: RecordKind::Function,
                name: "test_func".to_string(),
                content: "pub fn test_func() { }".to_string(),
                source_location: Default::default(),
            },
            Record {
                kind: RecordKind::Enum,
                name: "TestEnum".to_string(),
                content: "pub enum TestEnum { A, B }".to_string(),
                source_location: Default::default(),
            },
        ];

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Write records to JSONL file
        write_jsonl_file(temp_path, &records).unwrap();

        // Read records back
        let loaded_records = read_jsonl_file(temp_path).unwrap();

        // Verify they match
        assert_eq!(records.len(), loaded_records.len());
        for (original, loaded) in records.iter().zip(loaded_records.iter()) {
            assert_eq!(original.kind, loaded.kind);
            assert_eq!(original.name, loaded.name);
            assert_eq!(original.content, loaded.content);
        }
    }

    #[test]
    fn test_jsonl_file_format() {
        // Create a test record
        let record = Record {
            kind: RecordKind::Struct,
            name: "Test".to_string(),
            content: "pub struct Test { }".to_string(),
            source_location: Default::default(),
        };

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Write record to JSONL file
        write_jsonl_file(temp_path, &[record]).unwrap();

        // Read raw content and verify format
        let content = fs::read_to_string(temp_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Should have exactly one line
        assert_eq!(lines.len(), 1);

        // Line should be valid JSON
        let parsed: Record = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.name, "Test");
        assert_eq!(parsed.kind, RecordKind::Struct);
    }
}
