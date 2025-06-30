# Prebindgen Independent Tests

This directory contains comprehensive independent tests for the `prebindgen` proc-macro crate.

## Test Structure

### 1. `copy_structs_enums.rs`
Tests the core functionality of copying struct and enum definitions:
- ✅ Struct copying with various field types
- ✅ Enum copying with all variant types (unit, tuple, struct)
- ✅ Generic struct copying
- ✅ Preservation of derive attributes
- ✅ No duplicate definitions
- ✅ Nested enum handling

### 2. `path_access.rs`
Tests the `prebindgen_path!` macro functionality:
- ✅ Path constant generation (custom names and default)
- ✅ Access to generated content via path constants
- ✅ Path consistency across multiple invocations
- ✅ Directory existence and permissions
- ✅ Path matching with OUT_DIR when available

### 3. `no_out_dir.rs`
Tests fallback behavior when `OUT_DIR` is not available:
- ✅ Functionality without OUT_DIR environment variable
- ✅ Temp directory fallback mechanism
- ✅ Unique path generation for isolation
- ✅ Content accessibility in fallback scenarios
- ✅ Permission handling in temp directories
- ✅ Complete workflow validation

## Running Tests

### All Tests
```bash
cd independent_tests
cargo test
```

### Specific Test Suite
```bash
# Test struct/enum copying
cargo test --test copy_structs_enums

# Test path access functionality
cargo test --test path_access

# Test OUT_DIR fallback behavior
cargo test --test no_out_dir
```

### Verbose Output
```bash
cargo test -- --nocapture
```

## Test Features

- **Isolation**: Each test file is independent and can be run separately
- **Comprehensive**: Covers all major functionality and edge cases
- **Real-world scenarios**: Tests both build-time (OUT_DIR) and development-time scenarios
- **Error handling**: Tests fallback mechanisms and error conditions
- **Performance**: Validates that global path management works efficiently

## Expected Behavior

1. **With OUT_DIR**: Tests should use the provided OUT_DIR path
2. **Without OUT_DIR**: Tests should automatically fall back to temp directory
3. **Content verification**: All generated content should be accessible and correct
4. **Path consistency**: All path constants should point to the same location
5. **No duplicates**: Each definition should appear exactly once in the generated file

## Troubleshooting

If tests fail:

1. **Check environment**: Ensure `OUT_DIR` is set during build
2. **Permissions**: Verify write permissions to temp directories
3. **Dependencies**: Ensure `prebindgen` dependency is correctly resolved
4. **Clean build**: Try `cargo clean && cargo test`

## Integration

These tests are designed to be:
- Run as part of CI/CD pipelines
- Used for development verification
- Referenced for usage examples
- Extended for additional functionality
