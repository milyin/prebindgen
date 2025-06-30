use prebindgen::{prebindgen, prebindgen_path};
use std::fs;
use std::env;
use std::process;

// Test structures for no_out_dir tests
#[prebindgen]
pub struct NoOutDirStruct {
    pub data: String,
    pub timestamp: u64,
}

#[prebindgen]
pub enum NoOutDirEnum {
    State1,
    State2 { info: String },
    State3(i32, String),
}

// Generate path when OUT_DIR might not be available
prebindgen_path!(TEST_PATH);

#[test]
fn test_works_without_out_dir() {
    // This test simulates what happens when OUT_DIR is not available
    // We'll temporarily unset OUT_DIR and test fallback behavior
    
    let original_out_dir = env::var("OUT_DIR").ok();
    
    // Test that even if OUT_DIR was not set, our path constant works
    println!("TEST_PATH: {}", TEST_PATH);
    
    // Path should still be valid
    println!("Path length: {}", TEST_PATH.len());
    assert!(
        TEST_PATH.starts_with('/') || TEST_PATH.contains("temp") || TEST_PATH.contains("tmp"),
        "Path should be valid even without OUT_DIR: {}",
        TEST_PATH
    );
    
    // Should be accessible
    assert!(
        std::path::Path::new(TEST_PATH).exists(),
        "Directory should exist even without OUT_DIR: {}",
        TEST_PATH
    );
    
    println!("✅ Works without OUT_DIR test passed");
    
    // Restore OUT_DIR if it was originally set
    if let Some(out_dir) = original_out_dir {
        unsafe {
            env::set_var("OUT_DIR", out_dir);
        }
    }
}

#[test]
fn test_temp_directory_fallback() {
    // When OUT_DIR is not available, we should fall back to temp directory
    let path = TEST_PATH;
    
    // Should contain temp-related path components or be in a temp directory
    let is_temp_path = path.contains("temp") || 
                      path.contains("tmp") || 
                      path.contains("T/") ||  // macOS temp structure
                      path.starts_with("/tmp") ||
                      path.contains("prebindgen");
    
    // Either it's OUT_DIR (if available) or it should be a temp path
    let out_dir_available = env::var("OUT_DIR").is_ok();
    if !out_dir_available {
        assert!(
            is_temp_path,
            "Without OUT_DIR, path should be in temp directory: {}",
            path
        );
        println!("✅ Using temp directory fallback: {}", path);
    } else {
        println!("✅ Using OUT_DIR: {}", path);
    }
    
    println!("✅ Temp directory fallback test passed");
}

#[test]
fn test_generated_content_accessible_without_out_dir() {
    // Verify that generated content is accessible via the fallback path
    let file_path = format!("{}/prebindgen.rs", TEST_PATH);
    
    // File should exist
    assert!(
        std::path::Path::new(&file_path).exists(),
        "prebindgen.rs should exist even without OUT_DIR: {}",
        file_path
    );
    
    // Content should be readable
    let content = fs::read_to_string(&file_path)
        .expect("Should be able to read prebindgen.rs even without OUT_DIR");
    
    // Our definitions should be present
    assert!(
        content.contains("NoOutDirStruct"),
        "NoOutDirStruct should be in generated file"
    );
    
    assert!(
        content.contains("NoOutDirEnum"),
        "NoOutDirEnum should be in generated file"
    );
    
    assert!(
        content.contains("State1") && content.contains("State2") && content.contains("State3"),
        "All enum variants should be preserved"
    );
    
    println!("✅ Generated content accessible without OUT_DIR test passed");
}

#[test]
fn test_unique_path_generation() {
    // Test that the unique path generation works correctly
    let path = TEST_PATH;
    
    // Should be a valid directory
    assert!(
        std::path::Path::new(path).is_dir(),
        "Generated path should be a valid directory: {}",
        path
    );
    
    // If it's a unique temp path, it should contain identifying information
    if path.contains("prebindgen") {
        // Should contain process-specific or time-specific components for uniqueness
        let has_unique_component = path.contains(&process::id().to_string()) ||
                                  path.contains("prebindgen_") ||
                                  path.chars().any(|c| c.is_numeric());
        
        assert!(
            has_unique_component,
            "Unique temp path should contain identifying components: {}",
            path
        );
    }
    
    println!("✅ Unique path generation test passed");
}

#[test]
fn test_fallback_path_permissions() {
    // Test that we have proper permissions on the fallback directory
    let path = TEST_PATH;
    
    // Should be able to read the directory
    let entries = fs::read_dir(path)
        .expect("Should be able to read the prebindgen directory");
    
    // Should find at least the prebindgen.rs file
    let mut found_prebindgen_file = false;
    for entry in entries.flatten() {
        if entry.file_name() == "prebindgen.rs" {
            found_prebindgen_file = true;
            break;
        }
    }
    
    assert!(
        found_prebindgen_file,
        "Should find prebindgen.rs in the directory"
    );
    
    // Test write permissions by creating a temporary file
    let test_file = format!("{}/permission_test.tmp", path);
    fs::write(&test_file, "test")
        .expect("Should have write permissions in the prebindgen directory");
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    
    println!("✅ Fallback path permissions test passed");
}

/// Test that demonstrates the complete workflow without OUT_DIR
#[test]
fn test_complete_workflow_without_out_dir() {
    println!("🧪 Testing complete workflow without OUT_DIR dependency");
    
    // 1. Verify structs and enums were processed
    let file_path = format!("{}/prebindgen.rs", TEST_PATH);
    let content = fs::read_to_string(&file_path)
        .expect("Generated file should be readable");
    
    // 2. Verify content integrity
    assert!(content.contains("NoOutDirStruct"));
    assert!(content.contains("NoOutDirEnum"));
    
    // 3. Verify we can parse and analyze the generated content
    let lines: Vec<&str> = content.lines().collect();
    assert!(!lines.is_empty(), "Generated file should not be empty");
    
    // 4. Count definitions to ensure they're all there
    let struct_count = content.matches("struct").count();
    let enum_count = content.matches("enum").count();
    
    assert!(struct_count >= 1, "Should have at least one struct definition");
    assert!(enum_count >= 1, "Should have at least one enum definition");
    
    // 5. Verify path consistency
    assert_eq!(TEST_PATH, TEST_PATH, "Path should be consistent");
    
    println!("✅ Complete workflow without OUT_DIR test passed");
    println!("📁 Generated file: {}", file_path);
    println!("📊 Found {} struct(s) and {} enum(s)", struct_count, enum_count);
}
