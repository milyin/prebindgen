#!/usr/bin/env zsh

echo "Testing prebindgen macro with and without OUT_DIR"
echo "================================================="

# Test 1: With OUT_DIR (normal build)
echo "\n1. Testing with OUT_DIR (cargo build):"
cd /Users/milyin/ZS/prebindgen/test_workspace
cargo build --quiet
if find target -name "prebindgen.rs" -type f | head -1 | xargs cat | grep -q "TestStruct"; then
    echo "✅ SUCCESS: prebindgen.rs generated in OUT_DIR"
else
    echo "❌ FAILED: prebindgen.rs not found in OUT_DIR"
fi

# Test 2: Check that temp directory creation works
echo "\n2. Testing unique temp directory creation:"
cd /Users/milyin/ZS/prebindgen
if cargo test test_unique_temp_dir_creation --quiet; then
    echo "✅ SUCCESS: Unique temp directory creation works"
else
    echo "❌ FAILED: Unique temp directory creation failed"
fi

# Test 3: Build documentation to ensure it compiles
echo "\n3. Testing documentation build:"
if cargo doc --no-deps --quiet; then
    echo "✅ SUCCESS: Documentation builds correctly"
else
    echo "❌ FAILED: Documentation build failed"
fi

echo "\n================================================="
echo "All tests completed!"
