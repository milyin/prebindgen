use prebindgen_proc_macro::prebindgen_path;
use std::fs;
use std::env;

// Generate path constants
const CUSTOM_PATH: &str = prebindgen_path!();
const PREBINDGEN_PATH: &str = prebindgen_path!();

#[test]
fn test_path_constant_generation() {
    // Verify that the constants are generated and accessible
    println!("CUSTOM_PATH: {}", CUSTOM_PATH);
    println!("PREBINDGEN_PATH: {}", PREBINDGEN_PATH);
    
    // Both should point to the same file
    assert_eq!(
        CUSTOM_PATH, PREBINDGEN_PATH,
        "Both path constants should point to the same file"
    );
    
    // Should be a valid path ending with prebindgen.json
    assert!(
        CUSTOM_PATH.ends_with("/prebindgen.json"),
        "Path should end with /prebindgen.json: {}",
        CUSTOM_PATH
    );
    
    // Should be an absolute path within OUT_DIR
    assert!(
        CUSTOM_PATH.starts_with('/'),
        "Path should be absolute: {}",
        CUSTOM_PATH
    );
    
    println!("✅ Path constant generation test passed");
}

#[test]
fn test_access_generated_content_via_path() {
    // Use the generated path constant to access the file directly
    let file_path = CUSTOM_PATH;
    
    // Verify the file exists
    assert!(
        std::path::Path::new(file_path).exists(),
        "prebindgen.json should exist at path: {}",
        file_path
    );
    
    // Read and verify content
    let content = fs::read_to_string(file_path)
        .expect("Should be able to read prebindgen.json using path constant");
    
    // Parse JSON - now each line is a separate JSON object
    let records: Vec<serde_json::Value> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("Should be valid JSON"))
        .collect();
    
    // Since we now append all records, we just check that we have some records
    // and that the structure is correct
    assert!(!records.is_empty(), "Should have at least some records");
    
    // Verify record structure
    for record in &records {
        assert!(record["name"].is_string(), "Each record should have a name");
        assert!(record["kind"].is_string(), "Each record should have a kind");
        assert!(record["content"].is_string(), "Each record should have content");
    }
    
    // If PathTestEnum is present, check that enum variants are preserved in content
    if let Some(enum_record) = records.iter().find(|r| r["name"].as_str() == Some("PathTestEnum")) {
        let enum_content = enum_record["content"].as_str().unwrap();
        assert!(enum_content.contains("Alpha") && enum_content.contains("Beta") && enum_content.contains("Gamma"));
    }
    
    println!("✅ Access generated content via path test passed");
}

#[test]
fn test_path_matches_out_dir() {
    // The path should always be in OUT_DIR when available
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let expected_path = format!("{}/prebindgen.json", out_dir);
        assert_eq!(
            CUSTOM_PATH, expected_path,
            "Path constant should be OUT_DIR/prebindgen.json"
        );
        println!("✅ Path matches OUT_DIR test passed");
    } else {
        panic!("OUT_DIR should be set during tests with build.rs");
    }
}

#[test]
fn test_multiple_path_constants_consistency() {
    // Test that all shared path constants are consistent
    // Since they're all generated from the same global state, they should be identical
    
    // All should be the same
    assert_eq!(CUSTOM_PATH, PREBINDGEN_PATH);
    
    println!("✅ Multiple path constants consistency test passed");
}

#[test]
fn test_path_file_can_be_created() {
    // The file specified by the path should be creatable/accessible
    let path = std::path::Path::new(CUSTOM_PATH);
    
    // The parent directory should exist
    if let Some(parent) = path.parent() {
        assert!(
            parent.exists(),
            "Parent directory of prebindgen file should exist: {}",
            parent.display()
        );
        
        assert!(
            parent.is_dir(),
            "Parent should be a directory: {}",
            parent.display()
        );
    }
    
    println!("✅ Path file can be created test passed");
}

#[test]
fn test_can_create_additional_files_in_path() {
    // Test that we can create additional files in the same directory as the prebindgen file
    let path = std::path::Path::new(CUSTOM_PATH);
    let parent_dir = path.parent().expect("Should have a parent directory");
    let test_file_path = parent_dir.join("test_file.txt");
    
    // Write a test file
    fs::write(&test_file_path, "test content")
        .expect("Should be able to write to the prebindgen directory");
    
    // Verify it exists
    assert!(
        test_file_path.exists(),
        "Should be able to create files in the prebindgen directory"
    );
    
    // Read it back
    let content = fs::read_to_string(&test_file_path)
        .expect("Should be able to read back the test file");
    
    assert_eq!(content, "test content");
    
    // Clean up
    let _ = fs::remove_file(&test_file_path);
    
    println!("✅ Can create additional files in path test passed");
}
