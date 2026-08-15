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

fn a_record(name: &str, content: &str) -> Record {
    Record::new(
        RecordKind::Struct,
        name.to_string(),
        content.to_string(),
        Default::default(),
        None,
    )
}

fn line_of(record: &Record) -> String {
    record.to_jsonl_string().unwrap()
}

#[test]
fn write_record_file_creates_the_group_directory_and_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("structs")
        .join("Test_0123456789abcdef.jsonl");
    let record = a_record("Test", "pub struct Test;");

    write_record_file(&path, &line_of(&record)).unwrap();

    let read = read_jsonl_file(&path).unwrap();
    assert_eq!(read, vec![record]);
    assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 1);
}

#[test]
fn write_record_file_leaves_no_temporaries_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("group").join("Test_0123456789abcdef.jsonl");

    write_record_file(&path, &line_of(&a_record("Test", "pub struct Test;"))).unwrap();

    let leftovers = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "Test_0123456789abcdef.jsonl")
        .collect::<Vec<_>>();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn concurrent_identical_writes_leave_one_complete_file() {
    // The file name is derived from the contents, so every writer of a given
    // path writes these same bytes: the losers of the rename race cost nothing,
    // and no reader can observe a partial file.
    const WRITERS: usize = 32;

    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("default")
        .join("Test_0123456789abcdef.jsonl");
    let record = a_record("Test", "pub struct Test;");
    let line = line_of(&record);

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(WRITERS));
    let handles = (0..WRITERS)
        .map(|_| {
            let barrier = std::sync::Arc::clone(&barrier);
            let path = path.clone();
            let line = line.clone();
            std::thread::spawn(move || {
                barrier.wait();
                write_record_file(&path, &line).unwrap();
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }

    let files = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        vec![path.clone()],
        "exactly one file, no temporaries"
    );
    assert_eq!(read_jsonl_file(&path).unwrap(), vec![record]);
}

#[test]
fn write_record_file_rewrites_an_empty_file() {
    // A zero-length file is not a record this layout could have published, so
    // it must not suppress the real one.
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("default")
        .join("Test_0123456789abcdef.jsonl");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "").unwrap();

    let record = a_record("Test", "pub struct Test;");
    write_record_file(&path, &line_of(&record)).unwrap();

    assert_eq!(read_jsonl_file(&path).unwrap(), vec![record]);
}
