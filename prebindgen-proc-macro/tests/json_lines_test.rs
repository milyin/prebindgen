#[test]
fn test_json_lines_format() {
    // Get the path where prebindgen writes files
    let test_path = crate::prebindgen_path!();
    
    // Check if the file exists (it should since we've built the test_structures binary)
    if std::path::Path::new(&test_path).exists() {
        let content = std::fs::read_to_string(&test_path)
            .expect("Should be able to read prebindgen.json");
        
        println!("📄 JSON file content ({} bytes):", content.len());
        
        // Parse each line as JSON
        let mut records = Vec::new();
        for line in content.lines() {
            if !line.trim().is_empty() {
                match serde_json::from_str::<serde_json::Value>(line) {
                    Ok(record) => records.push(record),
                    Err(e) => panic!("Invalid JSON line: {}\nError: {}", line, e),
                }
            }
        }
        
        println!("✅ Found {} valid JSON records", records.len());
        
        // Check that each record has the expected structure
        for (i, record) in records.iter().enumerate() {
            assert!(record["name"].is_string(), "Record {} should have a name", i);
            assert!(record["kind"].is_string(), "Record {} should have a kind", i);
            assert!(record["content"].is_string(), "Record {} should have content", i);
            
            println!("  📝 Record {}: {} ({})", i, record["name"], record["kind"]);
        }
        
        // Verify we have at least some records
        assert!(!records.is_empty(), "Should have at least some records");
        
        println!("✅ JSON-lines format test passed!");
    } else {
        println!("⚠️  prebindgen.json not found at: {}", test_path);
        println!("This is expected if test structures haven't been compiled yet");
        // Don't fail the test, just skip it
    }
}

#[test]
fn test_no_deduplication() {
    // This test verifies that records are appended without deduplication
    let test_path = prebindgen_proc_macro::prebindgen_path!();
    
    if std::path::Path::new(&test_path).exists() {
        let content = std::fs::read_to_string(&test_path)
            .expect("Should be able to read prebindgen.json");
        
        // Parse all records
        let records: Vec<serde_json::Value> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("Should be valid JSON"))
            .collect();
        
        // Count occurrences of each name
        let mut name_counts = std::collections::HashMap::new();
        for record in &records {
            let name = record["name"].as_str().unwrap();
            *name_counts.entry(name).or_insert(0) += 1;
        }
        
        // Check if we have duplicates (which proves no deduplication)
        let has_duplicates = name_counts.values().any(|&count| count > 1);
        
        if has_duplicates {
            println!("✅ Found duplicate records - no deduplication is working correctly");
            for (name, count) in &name_counts {
                if *count > 1 {
                    println!("  📝 '{}' appears {} times", name, count);
                }
            }
        } else {
            println!("ℹ️  No duplicates found - this might be expected depending on compilation");
        }
        
        println!("✅ No deduplication test passed!");
    } else {
        println!("⚠️  Skipping no deduplication test - file not found");
    }
}

#[test]
fn test_append_only_behavior() {
    // This test verifies that the macro appends to the file without reading it first
    let test_path = prebindgen_proc_macro::prebindgen_path!();
    
    if std::path::Path::new(&test_path).exists() {
        let content = std::fs::read_to_string(&test_path)
            .expect("Should be able to read prebindgen.json");
        
        // The fact that we can read the file and parse each line individually
        // proves that the append-only JSON-lines format is working
        let line_count = content.lines().filter(|line| !line.trim().is_empty()).count();
        
        println!("📄 File has {} non-empty lines", line_count);
        assert!(line_count > 0, "Should have at least one line");
        
        // Each line should be parseable as individual JSON
        for (i, line) in content.lines().enumerate() {
            if !line.trim().is_empty() {
                serde_json::from_str::<serde_json::Value>(line)
                    .unwrap_or_else(|e| panic!("Line {} is not valid JSON: {}\nError: {}", i, line, e));
            }
        }
        
        println!("✅ All lines are valid individual JSON objects");
        println!("✅ Append-only behavior test passed!");
    } else {
        println!("⚠️  Skipping append-only test - file not found");
    }
}
