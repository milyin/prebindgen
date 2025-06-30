# Troubleshooting

## Rust-Analyzer Issues

If you see errors like `internal error: proc-macro map is missing error entry for crate` in your IDE, this is a rust-analyzer (language server) issue, not a compilation error. The code will compile and run correctly.

### Workaround:

1. **Restart rust-analyzer**: In VS Code, open the command palette (Cmd/Ctrl+Shift+P) and run "Rust Analyzer: Restart Server"

2. **Clean and rebuild**: 
   ```bash
   cargo clean
   cargo build
   ```

3. **Alternative approach**: If the IDE errors persist, you can use a simpler approach:
   ```rust
   // Instead of using prebindgen_path!(), use env! directly when possible
   const PREBINDGEN_DIR: &str = env!("OUT_DIR");
   ```

4. **IDE-specific configuration**: Add this to your VS Code settings.json:
   ```json
   {
     "rust-analyzer.procMacro.enable": true,
     "rust-analyzer.procMacro.attributes.enable": true
   }
   ```

### Why this happens:

Proc-macros that use global state or complex initialization can sometimes confuse rust-analyzer's internal crate mapping system. This is purely an IDE analysis issue - your code compiles and runs correctly.

The error typically occurs when:
- The proc-macro uses global state (like our singleton destination path)
- Multiple proc-macros interact with each other
- The IDE tries to analyze the macro before the build context is fully established

### Verification:

Always verify that your code works by running:
```bash
cargo build    # Should succeed
cargo test     # Should pass
cargo run      # Should work correctly
```

If these commands work, the rust-analyzer error is just an IDE analysis issue and can be safely ignored.
