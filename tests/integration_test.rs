#[cfg(test)]
mod integration_tests {    
    #[test]
    fn test_out_dir_available() {
        // This test verifies that OUT_DIR is available during testing
        // (which it should be when build.rs is present)
        let out_dir = std::env::var("OUT_DIR")
            .expect("OUT_DIR should be set during cargo test when build.rs is present");
        
        assert!(!out_dir.is_empty(), "OUT_DIR should not be empty");
        assert!(std::path::Path::new(&out_dir).exists(), "OUT_DIR path should exist");
        
        println!("✅ OUT_DIR is available: {}", out_dir);
    }
}

// This is how you would use the macro in an actual project:
/*
use prebindgen::prebindgen;

#[prebindgen]
pub struct MyStruct {
    pub field1: String,
    pub field2: i32,
}

#[prebindgen]
pub enum MyEnum {
    Variant1,
    Variant2(String),
}

// Then in your build.rs or main.rs, you can include the generated file:
// include!(concat!(env!("OUT_DIR"), "/prebindgen.rs"));
*/
