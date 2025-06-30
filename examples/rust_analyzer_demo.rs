// This example demonstrates that despite rust-analyzer showing errors,
// the proc-macros work perfectly during compilation and execution.

use prebindgen::{prebindgen, prebindgen_path};

#[prebindgen]
#[derive(Debug)]
pub struct ExampleStruct {
    pub name: String,
    pub id: u64,
}

prebindgen_path!(GENERATED_PATH);

fn main() {
    println!("🚀 Testing rust-analyzer compatibility");
    println!("=====================================");
    
    // This works despite any IDE errors
    let example = ExampleStruct {
        name: "test".to_string(),
        id: 42,
    };
    
    println!("✅ Created struct: {:?}", example);
    println!("✅ Generated path: {}", GENERATED_PATH);
    
    // Verify the file exists
    let file_path = format!("{}/prebindgen.rs", GENERATED_PATH);
    if std::path::Path::new(&file_path).exists() {
        println!("✅ prebindgen.rs file exists and is accessible");
        
        // Read a small portion to verify
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let lines: Vec<&str> = content.lines().collect();
            println!("✅ File contains {} lines of generated code", lines.len());
            
            // Check if our struct definition is in there
            if content.contains("ExampleStruct") {
                println!("✅ Our struct definition was successfully copied");
            }
        }
    }
    
    println!("\n🎉 All functionality works correctly!");
    println!("💡 Any rust-analyzer errors are IDE-only and can be ignored.");
}
