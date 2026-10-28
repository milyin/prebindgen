# Reviewing generated bindings

The binding examples commit their generated output under `generated/`:

- `example-cbindgen` and `perftest-c`: Rust wrappers and C headers.
- `covertest-kotlin` and `perftest-kotlin`: Rust JNI wrappers and Kotlin files.
- `emitcheck`: its generated Rust compile fixture.

Regenerate these review artifacts from the repository root:

```sh
python3 examples/record-generated.py
```

Commit the generated changes alongside changes to the generator or binding
declarations. Do not edit the generated files by hand. The command owns these
five `generated/` trees and removes obsolete files. Add new binding examples to
the script's `FILES` list when they should publish review artifacts.

The script builds the workspace and obtains the exact `OUT_DIR` paths from
Cargo's JSON messages. Kotlin build scripts replace their generator-owned output
trees before writing, so cached obsolete files reach neither tests nor snapshots.
The generated files contain no build directory paths.

These are byte-for-byte snapshots of the output compiled and exercised by the
examples. Builds and boundary tests continue to use `OUT_DIR`; snapshots do not
introduce another source set or duplicate Kotlin classes.

Internal JNI names include a readable Rust function or type name and a stable
hash suffix, for example `call_read_<hash>`, `drop_Handle_<hash>` and
`__pb_jni_error_Error_typed_<hash>`. Sequence sink interfaces follow the same
rule. Function identities include the qualified source path and binding name;
drop identities use the canonical Rust type; error helpers use the error type
and mapped/typed mode. Adding or reordering unrelated declarations therefore
does not renumber existing bindings. Hashing happens during generation, with no
runtime work. Name collisions are generation errors rather than counter suffixes.
Unnamed and generic error types such as `()` and `Vec<i64>` receive stable readable handler names,
such as `ErrorHandler_Unit_typed_<hash>`, instead of names derived from their
first caller. Concrete named domain-error handler names retain their existing spelling.

CI runs the read-only check before the boundary suite:

```sh
python3 examples/record-generated.py --check
```

It fails for missing, changed or obsolete snapshots. Both commands build with
Cargo's offline mode, so dependencies must already be available in the cache.
