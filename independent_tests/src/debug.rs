use prebindgen::{prebindgen, prebindgen_path};

#[prebindgen]
pub struct DebugStruct {
    pub name: String,
}

prebindgen_path!(DEBUG_PATH);

fn main() {
    println!("Debug path: {}", DEBUG_PATH);
    
    let file_path = format!("{}/prebindgen.rs", DEBUG_PATH);
    println!("Looking for file at: {}", file_path);
    
    if std::path::Path::new(&file_path).exists() {
        println!("✅ File exists!");
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            println!("Content:\n{}", content);
        }
    } else {
        println!("❌ File does not exist");
        
        // Check what's in the directory
        if let Ok(entries) = std::fs::read_dir(DEBUG_PATH) {
            println!("Directory contents:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {:?}", entry.file_name());
                }
            }
        }
    }
}
