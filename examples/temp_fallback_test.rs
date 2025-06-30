use prebindgen::prebindgen;
use std::env;

#[prebindgen]
#[derive(Debug)]
pub struct TempTestStruct {
    pub name: String,
    pub value: i32,
}

fn main() {
    // This test demonstrates the fallback behavior when OUT_DIR is not available
    
    // First, let's check if OUT_DIR exists
    if let Ok(out_dir) = env::var("OUT_DIR") {
        println!("OUT_DIR is available: {}", out_dir);
    } else {
        println!("OUT_DIR is not available - macro should fallback to temp directory");
    }
    
    let test_struct = TempTestStruct {
        name: "temp test".to_string(),
        value: 123,
    };
    
    println!("Created struct: {:?}", test_struct);
    
    // Look for prebindgen.rs files in temp directory
    if let Ok(temp_dir) = env::var("TMPDIR").or_else(|_| env::var("TMP")).or_else(|_| env::var("TEMP")) {
        println!("System temp directory: {}", temp_dir);
        
        // Try to find any prebindgen directories
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("prebindgen_") {
                        println!("Found prebindgen temp directory: {:?}", entry.path());
                        let prebindgen_file = entry.path().join("prebindgen.rs");
                        if prebindgen_file.exists() {
                            if let Ok(content) = std::fs::read_to_string(&prebindgen_file) {
                                println!("Content: {}", content);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Fallback to std::env::temp_dir()
        let temp_dir = std::env::temp_dir();
        println!("Using std::env::temp_dir(): {:?}", temp_dir);
    }
}
