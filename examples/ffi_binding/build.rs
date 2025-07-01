use prebindgen::Record;
use std::fs;
use std::collections::HashSet;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let json_path = ffi_common::GENERATED_PATH;
    let dest_path = format!("{}/ffi_common.rs", out_dir);
    
    // Read JSON and convert to Rust code
    if let Ok(json_content) = fs::read_to_string(json_path) {
        // Parse JSON-lines format (each line is a separate JSON object)
        let records: Vec<Record> = json_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        
        let mut rust_code = String::new();
        let mut seen_names = HashSet::new();
        
        for record in records {
            // Only add if we haven't seen this name before (deduplicate)
            if !seen_names.contains(&record.name) {
                seen_names.insert(record.name.clone());
                rust_code.push_str(&record.content);
                rust_code.push('\n');
            }
        }
        
        fs::write(&dest_path, rust_code).unwrap_or_else(|e| {
            panic!("Failed to write Rust code to '{}': {}", dest_path, e)
        });
    } else {
        panic!("Failed to read JSON file from '{}'", json_path);
    }
}
