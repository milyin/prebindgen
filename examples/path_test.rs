use prebindgen::{prebindgen, prebindgen_path};

#[prebindgen]
#[derive(Debug)]
pub struct PathTestStruct {
    pub name: String,
}

// Generate a constant with the prebindgen directory path
prebindgen_path!(PREBINDGEN_DIR);

// Generate another constant with a custom name
prebindgen_path!(MY_CUSTOM_PATH);

fn main() {
    println!("Testing prebindgen_path! macro");
    println!("===============================");
    
    println!("PREBINDGEN_DIR: {}", PREBINDGEN_DIR);
    println!("MY_CUSTOM_PATH: {}", MY_CUSTOM_PATH);
    
    // Verify they are the same
    assert_eq!(PREBINDGEN_DIR, MY_CUSTOM_PATH);
    println!("✅ Both constants point to the same directory");
    
    // Create a test struct
    let test_struct = PathTestStruct {
        name: "test".to_string(),
    };
    
    println!("Created struct: {:?}", test_struct);
    
    // Check if the prebindgen.rs file exists at the specified path
    let prebindgen_file = format!("{}/prebindgen.rs", PREBINDGEN_DIR);
    if std::path::Path::new(&prebindgen_file).exists() {
        println!("✅ prebindgen.rs file exists at: {}", prebindgen_file);
        
        if let Ok(content) = std::fs::read_to_string(&prebindgen_file) {
            println!("File content preview:");
            for (i, line) in content.lines().take(3).enumerate() {
                println!("  {}: {}", i + 1, line);
            }
            if content.lines().count() > 3 {
                println!("  ... ({} more lines)", content.lines().count() - 3);
            }
        }
    } else {
        println!("❌ prebindgen.rs file not found at: {}", prebindgen_file);
    }
}
