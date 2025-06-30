use prebindgen_tests::{CUSTOM_PATH, PREBINDGEN_PATH, PathTestStruct, PathTestEnum};
use std::fs;
use std::env;

#[test]
fn test_path_constant_generation() {
    // Verify that the constants are generated and accessible
    println!("CUSTOM_PATH: {}", CUSTOM_PATH);
    println!("PREBINDGEN_PATH: {}", PREBINDGEN_PATH);
    
    // Both should point to the same directory
    assert_eq!(
        CUSTOM_PATH, PREBINDGEN_PATH,
        "Both path constants should point to the same directory"
    );
    
    // Should be a valid path
    assert!(
        !CUSTOM_PATH.is_empty(),
        "Path constant should not be empty"
    );
    
    // Should be an absolute path or a recognizable temp path
    assert!(
        CUSTOM_PATH.starts_with('/') || CUSTOM_PATH.contains("temp") || CUSTOM_PATH.contains("tmp"),
        "Path should be absolute or in temp directory: {}",
        CUSTOM_PATH
    );
    
    println!("✅ Path constant generation test passed");
}

#[test]
fn test_access_generated_content_via_path() {
    // Use the generated path constant to access the file
    let file_path = format!("{}/prebindgen.rs", CUSTOM_PATH);
    
    // Verify the file exists
    assert!(
        std::path::Path::new(&file_path).exists(),
        "prebindgen.rs should exist at path constructed from constant: {}",
        file_path
    );
    
    // Read and verify content
    let content = fs::read_to_string(&file_path)
        .expect("Should be able to read prebindgen.rs using path constant");
    
    // Verify our test definitions are in the content
    assert!(
        content.contains("PathTestStruct"),
        "PathTestStruct should be found in generated file"
    );
    
    assert!(
        content.contains("PathTestEnum"),
        "PathTestEnum should be found in generated file"
    );
    
    assert!(
        content.contains("Alpha") && content.contains("Beta") && content.contains("Gamma"),
        "All enum variants should be preserved"
    );
    
    println!("✅ Access generated content via path test passed");
}

#[test]
fn test_path_matches_out_dir() {
    // When OUT_DIR is available, our path should match it
    if let Ok(out_dir) = env::var("OUT_DIR") {
        assert_eq!(
            CUSTOM_PATH, out_dir,
            "When OUT_DIR is available, path constant should match it"
        );
        println!("✅ Path matches OUT_DIR test passed");
    } else {
        // If OUT_DIR is not available, path should be in temp directory
        assert!(
            CUSTOM_PATH.contains("temp") || CUSTOM_PATH.contains("tmp") || CUSTOM_PATH.contains("prebindgen"),
            "When OUT_DIR is not available, path should be in temp directory: {}",
            CUSTOM_PATH
        );
        println!("✅ Path fallback to temp directory test passed");
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
fn test_path_directory_exists() {
    // The directory specified by the path should exist
    assert!(
        std::path::Path::new(CUSTOM_PATH).exists(),
        "Directory specified by path constant should exist: {}",
        CUSTOM_PATH
    );
    
    // Should be a directory, not a file
    assert!(
        std::path::Path::new(CUSTOM_PATH).is_dir(),
        "Path should point to a directory: {}",
        CUSTOM_PATH
    );
    
    println!("✅ Path directory exists test passed");
}

#[test]
fn test_can_create_additional_files_in_path() {
    // Test that we can create additional files in the same directory
    let test_file_path = format!("{}/test_file.txt", CUSTOM_PATH);
    
    // Write a test file
    fs::write(&test_file_path, "test content")
        .expect("Should be able to write to the prebindgen directory");
    
    // Verify it exists
    assert!(
        std::path::Path::new(&test_file_path).exists(),
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
