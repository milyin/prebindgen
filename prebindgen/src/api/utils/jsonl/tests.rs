use std::fs;

use tempfile::NamedTempFile;

use super::*;
use crate::api::record::RecordKind;

#[test]
fn test_jsonl_round_trip() {
    // Create some test records
    let records = vec![
        Record::new(
            RecordKind::Struct,
            "TestStruct".to_string(),
            "pub struct TestStruct { }".to_string(),
            Default::default(),
            None,
        ),
        Record::new(
            RecordKind::Function,
            "test_func".to_string(),
            "pub fn test_func() { }".to_string(),
            Default::default(),
            None,
        ),
        Record::new(
            RecordKind::Enum,
            "TestEnum".to_string(),
            "pub enum TestEnum { A, B }".to_string(),
            Default::default(),
            None,
        ),
    ];

    // Create a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    // Write records to JSONL file
    write_to_jsonl_file(temp_path, &records).unwrap();

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
    let record = Record::new(
        RecordKind::Struct,
        "Test".to_string(),
        "pub struct Test { }".to_string(),
        Default::default(),
        None,
    );

    // Create a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    // Write record to JSONL file
    write_to_jsonl_file(temp_path, &[record]).unwrap();

    // Read raw content and verify format
    let content = fs::read_to_string(temp_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Should have exactly one line
    assert_eq!(lines.len(), 1);

    // Line should be valid JSON
    let parsed: Record = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(parsed.name, "Test");
    assert_eq!(parsed.kind, RecordKind::Struct);

    // Test record with cfg feature field
    let record_with_cfg = Record::new(
        RecordKind::Function,
        "test_func".to_string(),
        "pub fn test_func() { }".to_string(),
        Default::default(),
        Some("feature = \"unstable\"".to_string()),
    );

    write_to_jsonl_file(temp_path, &[record_with_cfg]).unwrap();
    let content = fs::read_to_string(temp_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    dbg!(&lines);
    assert_eq!(lines.len(), 2);

    let parsed: Record = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(parsed.cfg, Some("feature = \"unstable\"".to_string()));
}
