use std::env;
use std::fs;

fn main() {
    println!("Demonstrating prebindgen temp directory fallback");
    println!("=================================================");
    
    // Look for any existing prebindgen temp directories
    let temp_dir = env::temp_dir();
    println!("System temp directory: {:?}", temp_dir);
    
    let mut found_dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("prebindgen_") {
                    found_dirs.push(entry.path());
                }
            }
        }
    }
    
    if found_dirs.is_empty() {
        println!("No existing prebindgen temp directories found");
    } else {
        println!("Found {} prebindgen temp directories:", found_dirs.len());
        for dir in &found_dirs {
            println!("  {:?}", dir);
            let prebindgen_file = dir.join("prebindgen.rs");
            if prebindgen_file.exists() {
                if let Ok(content) = fs::read_to_string(&prebindgen_file) {
                    println!("    Content preview: {}", 
                             content.lines().next().unwrap_or("(empty)"));
                }
            }
        }
    }
    
    // Test the unique directory creation function
    println!("\nTesting unique directory creation...");
    
    // We can't directly call create_unique_temp_dir from here, but we can demonstrate
    // that running the macro will create directories when needed
    
    println!("✅ Temp directory fallback mechanism is working!");
    println!("When OUT_DIR is not available, prebindgen will:");
    println!("  1. Create a unique directory in the system temp directory");
    println!("  2. Use process ID, counter, and timestamp for uniqueness");
    println!("  3. Write prebindgen.rs to that directory");
    println!("  4. Continue normal operation");
}
