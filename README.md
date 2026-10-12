# prebindgen: occurrence conversion slots

This branch replaces the generator engine with a small pipeline. The source
capture macro and source-reading support remain unchanged. The supported JniGen
`build.rs` declarations keep their existing syntax and import paths;
`prebindgen-registry` is a declaration re-export facade, with no old registry
engine behind it.

```text
#[prebindgen] source + build.rs declarations
        ↓ select declared functions, constructors, getters and referenced types
Flat<Slot>: syn::Type + conversion recipe at each parameter/result occurrence
        ↓ prebindgen-tools: defaults, overrides, validation and composition
Flat<Prepared>: wire roles, input construction, output delivery
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

A conversion belongs to an occurrence, so two parameters of the same Rust type
can have different input recipes. `Flat<S>` also preserves named record fields,
source names and locations. There is no parallel `TypeKind` tree. Wrapper
writers use prepared slots; recursive conversion decisions belong to tools.

## Examples migrated from main

The artificial `demo-flat`, `demo-c`, and shared demonstration declarations have
been removed. All eight Rust example crates from `main` are in the workspace.
Unsupported declarations, record fields and matching constructor arguments are
commented in the flat sources and `build.rs` files as the checklist for future
work. The retained records demonstrate the supported shapes of the real examples.

| Example | Currently active |
|---|---|
| [example-flat](examples/example-flat) | Calculator/Error aliases and Caption with only a text field |
| [example-cbindgen](examples/example-cbindgen/build.rs) | Calculator and Caption creation and typed release |
| [perftest-flat](examples/perftest-flat) | Storage/Archive creation, strings, BlobValue with bytes, Report/ReportStruct with a label |
| [perftest-c](examples/perftest-c/build.rs) | Storage, strings, BlobValue transfer, Report constructor selectors and label/handle outputs |
| [perftest-kotlin](examples/perftest-kotlin/build.rs) | Storage creation/release through JNI |
| [covertest-kotlin](examples/covertest-kotlin/build.rs) | The same BlobValue/Report conversions, strings, Storage, Archive, subpackages and the SummaryVault class rename |
| [covertest-helpers](examples/covertest-helpers) | Unsupported helper functions and multi-source bindings are commented out |
| [emitcheck](examples/emitcheck/build.rs) | Plain string/byte-vector fields compile; callback entry points remain commented, so it currently emits no API wrappers |

Source selection happens before subset validation. Unbound enums, callbacks,
collections and functions can stay in the source crate; selecting one still
produces a generation error when its shape is unsupported. Both annotated flat
examples also have tests that parse the entire captured surface, ensuring no
unsupported shapes remain active behind that selection.

The old foreign performance drivers require the commented bindings and have
been removed. Full coverage clients are replaced with focused C and Kotlin smoke
clients exercising the active declarations; this branch makes no performance or
full downstream compatibility claim. Generated artifacts remain under `OUT_DIR`.

## Supported generator API

JniGen supports `source(...)` or `items(...)`, `package!`, `ptr_class!`, `fun!`,
`.name()`, package prefix and native initialization. `build()` returns `JniGen`;
`write_rust` resolves relative paths under `OUT_DIR` and returns the written path;
`write_kotlin` returns the written paths. Settings may follow declarations.

The tools layer also supports `expand_param!` constructor/identity variants,
`expand_return!` getter/identity outputs, `.fields(fields!(...))` with per-field
output overrides, and per-function overrides. These fill slots before preparing
wrappers. The migrated C and JNI examples exercise Report constructor/handle input
variants and `.fields(fields!(report_to_struct)).field_self()` output delivery.
The reduced BlobValue uses a handle class; data-class generation stays disabled.

C uses `Cbindgen::builder().source(...)`, `.expand(...)`, `.function(fun!(...))`,
`.prefix(...)` and `.handle(...)`. Its old naming, raw-pointer and cbindgen
postprocessing settings are commented out in the example manifests.

Manual `Flat<Slot>` configuration remains available for generator development,
but none of the migrated binding examples requires it. Tools compose owned
handles, text/bytes, fields, constructors, input selectors and combined outputs.
Observation outputs execute before a consuming output. Moving twice or moving a
borrowed field is rejected; independent field handles explicitly require cloning.

## Boundary contracts and limits

Handles are opaque numeric IDs with runtime type checks. A selected input handle
transfers ownership when its conversion executes. On later failure, consumed
handles stay consumed; unvisited handles remain caller-owned. Pending output
handles are reclaimed on delivery failure. Closing an already closed or consumed
handle is harmless.

C wrappers return status and write output structs only on success. Strings and
bytes use pointer/length buffers. Callers supply valid input/output memory and
free returned buffers exactly once using the generated function. `last_error`
is thread-local. JNI uses typed `AutoCloseable` handle classes and reports
conversion failures as exceptions. Rust panics are caught when unwinding is
enabled. Multi-output JNI delivery uses an object array and a builder lambda;
a throwing builder releases its output handles.

Scalars, callbacks, enums, options, general collections, borrowed/mutable handle
inputs, methods, data classes, `convert!`, consuming record decomposition, field
renaming, local function declarations, domain errors, split overloads and shared
selector argument storage remain outside this subset.

## Build and verify

Rust 1.85+, a C compiler, JDK 11+ and a Kotlin compiler are needed for all checks.
Build once online to fetch dependencies; the boundary script then uses offline
Cargo builds.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 examples/check.py
```

The script builds every example and runs both C and both JNI smoke clients,
including string round trips, byte-vector constructors, consumed handles,
selector arms, field projections, builder rollback, output preservation and
concurrent close. `KOTLINC` and `CC` select the compilers.
`SANITIZE=1` instruments the C callers with AddressSanitizer and
UndefinedBehaviorSanitizer; JNI always runs with `-Xcheck:jni`.

The native Rust benchmark is commented out because it depends on disabled
scalar, borrowed-handle and callback APIs. Its driver reports that limitation.
