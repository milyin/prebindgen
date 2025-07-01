# Integration Tests for prebindgen

This crate contains integration tests that depend on `prebindgen-proc-macro` functionality. The tests have been moved here to allow proper testing of proc-macro features, since proc-macros cannot be used within the same crate where they are defined.

## Structure

- `src/test_structures.rs` - Binary that uses `#[prebindgen]` to generate test data
- `tests/json_lines_test.rs` - Tests for JSON-lines output format
- `tests/path_access.rs` - Tests for the `prebindgen_path!()` macro

## Running Tests

To run these tests:

```bash
cargo test -p tests-integration
```

The tests require the test structures binary to be built first to generate the `prebindgen.json` file:

```bash
cargo run -p tests-integration --bin test_structures
```

This separation ensures that:

1. The proc-macro crate can be tested independently
2. Integration tests can properly use the proc-macro functionality
3. Test organization is cleaner and more maintainable
