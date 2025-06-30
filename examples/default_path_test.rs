use prebindgen::{prebindgen, prebindgen_path};

#[prebindgen]
pub struct DefaultPathTest {
    pub value: i32,
}

// Use the default constant name
prebindgen_path!();

fn main() {
    println!("Testing prebindgen_path! with default name");
    println!("==========================================");
    
    println!("PREBINDGEN_PATH: {}", PREBINDGEN_PATH);
    
    // Show that we can construct the full file path
    let full_path = format!("{}/prebindgen.rs", PREBINDGEN_PATH);
    println!("Full prebindgen.rs path: {}", full_path);
    
    if std::path::Path::new(&full_path).exists() {
        println!("✅ File exists!");
    } else {
        println!("❌ File does not exist");
    }
}
