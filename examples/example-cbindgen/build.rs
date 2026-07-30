//! Build script generating C bindings for `example-flat` using prebindgen + cbindgen.
//!
//! This is a language-specific binding crate. It reads the `#[prebindgen]` items
//! captured by `example-flat`, runs them through the `prebindgen::lang::Cbindgen`
//! adapter to produce a Rust file of `extern "C"` wrappers, then runs cbindgen on
//! that file to produce a C header.
//!
//! Both generated artifacts are also published into this crate's tree so they can
//! be committed and inspected, under a file name naming **every input that
//! changes the output** — the target architecture and the enabled features,
//! because `#[prebindgen]` `cfg` handling makes the generated code differ by both
//! (see `Foo` / `InsideFoo` in `example-flat`):
//!   - `generated/example_flat_<arch>[_<feature>…].rs` — the Rust FFI layer (`include!`d by `lib.rs`)
//!   - `include/example_flat_<arch>[_<feature>…].h`    — the C header
//!
//! One name per variant is what keeps them independent: a `--all-features` build
//! writes `…_internal_unstable.rs` and leaves the default-feature file alone,
//! instead of overwriting it with a variant that then gets committed by accident.
//!
//! Build for several targets (e.g. via the bundled `CMakeLists.txt`, or
//! `cargo build -p example-cbindgen --target x86_64-... ` then `--target aarch64-...`)
//! to produce both and see that they differ.

use std::path::{Path, PathBuf};

use syn::parse_quote as pq;

/// The target architecture this build targets (`x86_64`, `aarch64`, …), used to
/// name the per-target generated artifacts. `CARGO_CFG_TARGET_ARCH` is set by
/// cargo for the *target* being built, so a `--target` cross-build names its own
/// file.
fn target_arch() -> String {
    std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string())
}

/// The features of `example-flat` this build enables, in declaration order, as a
/// file-name suffix (`""`, `"_unstable"`, `"_internal_unstable"`, …).
///
/// The features gate `#[prebindgen]` items, so they change the generated output
/// exactly the way the target arch does — and so they belong in the name for the
/// same reason. Without this, `cargo test --all-features` (which CI runs) silently
/// rewrites the default-feature artifact with the all-features one.
fn feature_suffix() -> String {
    ["internal", "unstable"]
        .iter()
        .filter(|f| std::env::var(format!("CARGO_FEATURE_{}", f.to_uppercase())).is_ok())
        .map(|f| format!("_{f}"))
        .collect()
}

/// The committed artifact's stem: every input that changes the output, and
/// nothing else.
fn variant() -> String {
    format!("example_flat_{}{}", target_arch(), feature_suffix())
}

fn main() {
    // prebindgen part: build the Rust file of extern "C" wrappers.
    let bindings_file = generate_ffi_bindings();
    // cbindgen part: build the C header from that Rust file.
    generate_c_headers(&bindings_file);
}

/// Generate the Rust FFI bindings from `example-flat`'s prebindgen output via the
/// `lang::Cbindgen` adapter, and publish the result to `generated/example_flat.rs`.
fn generate_ffi_bindings() -> PathBuf {
    let unstable = std::env::var("CARGO_FEATURE_UNSTABLE").is_ok();

    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name, so no per-item `.name(...)` is needed.
    let mut cbindgen = prebindgen::lang::Cbindgen::new()
        .source_module(pq!(example_flat))
        // Single universal freer for the `char*` data the layer hands out
        // (the `String` returns). Opaque handles keep their typed `*_drop`.
        .free_memory_function("example_free")
        // `Calculator` -> `calculator_t` / `calculator_drop`; the base defaults to
        // the snake_case of the Rust short name (no `mangle_rust_type` override).
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        // Keep the Rust function names verbatim as the exported C symbols.
        .mangle_function(|n| n.to_string());

    // The opaque `Calculator` handle (Box-owned; `calculator_drop` generated).
    cbindgen = cbindgen.opaque_ptr(pq!(Calculator));

    // `Error` is opaque (a boxed std error), marshalled to C as a `char*` message
    // via `error_get_message(&e)`; each fallible wrapper gains a `char **e` out-param.
    // `error_get_message` is consumed only as that message fn, so it is not also
    // declared as an exported `.function` — mark it ignored to silence the
    // "skipping undeclared" notice.
    cbindgen = cbindgen
        .opaque_error(pq!(Error), pq!(error_get_message))
        .ignore_function(pq!(error_get_message));

    // The primitive-repr `Operation` enum -> a C enum.
    cbindgen = cbindgen.enum_type(pq!(Operation));

    // The data-carrying `Shape` enum -> a `#[repr(C)]` enum with payload
    // variants, which cbindgen renders as a C tag + `union`. Its `Labeled` arm
    // owns a `char *`, so a typed `shape_drop` is generated to free the active
    // arm. `Drawing` carries one as a by-value field.
    cbindgen = cbindgen.tagged_union(pq!(Shape));
    cbindgen = cbindgen.data_struct(pq!(Drawing));

    // #158 part 2: payload wires come from the resolved CONVERTER DESTINATION,
    // not the layout-preserving mirror policy. `Note` exercises the two shapes
    // that admits — a nested `data_struct` by value (`Caption`, owning a
    // `char *` the union's drop must reach through and free) and a converted
    // leaf (`Millis` → `u64`).
    cbindgen = cbindgen.data_struct(pq!(Caption));
    cbindgen = cbindgen.convert(
        prebindgen::convert!(Millis)
            .input(prebindgen::fun!(millis_from_raw))
            .output(prebindgen::fun!(millis_to_raw)),
    );
    cbindgen = cbindgen
        .ignore_function(pq!(millis_from_raw))
        .ignore_function(pq!(millis_to_raw));
    cbindgen = cbindgen.tagged_union(pq!(Note));

    // Multi-target cfg demonstration: `InsideFoo` (a fieldless enum whose
    // discriminants vary by `target_arch`) and `Foo` (a by-value data struct whose
    // field set varies by `target_arch` + feature). Declared unconditionally —
    // exactly one `InsideFoo` and one field-shape of `Foo` survive the target's
    // cfg filtering, so the generated `inside_foo_t` / `foo_t` differ per target.
    cbindgen = cbindgen.enum_type(pq!(InsideFoo));
    cbindgen = cbindgen.data_struct(pq!(Foo));

    // The history-replay callback signature -> a `closure_value_t` closure struct
    // (`.base_name` gives the otherwise `f64`-derived struct a descriptive name).
    cbindgen = cbindgen
        .callback(pq!(impl Fn(f64) + Send + Sync + 'static))
        .base_name("value");

    // Constructors / `Result`-returning ops (fallible inputs route through the
    // error out-param), plus the infallible by-value `Foo` accessors and the
    // `InsideFoo` producer — none need `.panic()`. `calculator_apply` takes an
    // `Operation` by value, so its `char **e` also carries an invalid-discriminant
    // rejection (see `inside_foo_value` below).
    for function in [
        pq!(calculator_new),
        pq!(calculator_new_from_str),
        pq!(calculator_apply),
        // Two consumed handles of one type, and one consumed beside one
        // borrowed: the two shapes the alias preflight covers. Both return
        // `Result`, so the rejection reaches the caller through `char **e`
        // rather than aborting.
        pq!(calculator_merge),
        pq!(calculator_absorb),
        pq!(foo_new),
        pq!(foo_get_id),
        pq!(inside_foo_default),
        // The tagged union crossing OUT: constructed and returned. Nothing to
        // validate on this side — Rust always writes a live arm.
        pq!(shape_new_empty),
        pq!(shape_new_circle),
        pq!(shape_new_rect),
        // The union crossing IN on a fallible function: an out-of-range tag
        // reports through the same `char **e` as the domain error.
        pq!(shape_try_area),
        // `Note`'s constructors: the nested-struct, converted-leaf and `bool`
        // payloads crossing OUT.
        pq!(note_new_silent),
        pq!(note_new_after),
        pq!(note_new_flagged),
    ] {
        cbindgen = cbindgen.function(function);
    }
    if unstable {
        // `calculator_reset` mirrors an `#[unstable]` slice of the API; only present
        // in the captured source when the feature is enabled. Its `&mut` borrow is
        // fallible (null-checked) with no `Result`, so `.panic()`.
        cbindgen = cbindgen.function(pq!(calculator_reset)).panic();
    }

    // Borrow-only accessors / predicates / the callback driver: they have fallible
    // (null-checked) borrow inputs but no `Result` channel, so `.panic()` lets the
    // wrapper abort on a null handle. `inside_foo_value` joins them for the same
    // reason with a different fallible input: a C enum is an `int` at the ABI, so
    // its discriminant is validated on the way in (never materialised unchecked),
    // and with no `char **e` to report an out-of-range one it aborts. The three
    // union-consuming accessors are the same case one level up — the tag a C
    // caller supplies is validated before the Rust enum exists, here abortively.
    for function in [
        pq!(inside_foo_value),
        pq!(shape_area),
        pq!(shape_get_label),
        pq!(drawing_new),
        pq!(drawing_get_shape),
        // `Note` crossing IN (tag validated, so fallible without a `Result`),
        // and its `&str`-taking constructor.
        pq!(note_value),
        pq!(note_emphatic),
        pq!(note_new_titled),
        pq!(note_new_sketched),
        pq!(caption_new),
        pq!(calculator_new_clone),
        pq!(calculator_get_value),
        pq!(calculator_get_count),
        pq!(calculator_is),
        pq!(calculator_to_string),
        pq!(calculator_get_history),
        pq!(calculator_for_each),
        // `&str` label input is fallible (null-checked) with no `Result`.
        pq!(shape_new_labeled),
    ] {
        cbindgen = cbindgen.function(function).panic();
    }

    // Reads example-flat's `#[prebindgen]` output straight from its directory.
    let registry = prebindgen::core::Registry::builder()
        .source(example_flat::PREBINDGEN_OUT_DIR)
        .build()
        .expect("scan prebindgen items");
    // Always written to OUT_DIR under a stable name too, so the commented-out
    // `include!(OUT_DIR ...)` alternative in `lib.rs` works for any target.
    let out_file = registry
        .resolve(cbindgen)
        .expect("resolve prebindgen items")
        .write_rust("example_flat.rs")
        .expect("write generated bindings");

    // Publish the generated Rust into the crate tree under its per-variant name
    // (committed artifact). Write only on change so cargo doesn't rebuild-loop on
    // the `include!`d file.
    let in_tree = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("generated")
        .join(format!("{}.rs", variant()));
    write_if_changed(&in_tree, &std::fs::read_to_string(&out_file).unwrap());

    // `lib.rs` includes this path rather than matching on `cfg`: the build script
    // already knows which variant it wrote, and a `cfg` matrix in the source would
    // have to be re-taught every arch × feature combination.
    println!(
        "cargo:rustc-env=EXAMPLE_FLAT_BINDINGS={}",
        in_tree.display()
    );
    println!("cargo:warning=Generated bindings at: {}", in_tree.display());
    in_tree
}

/// Generate the C header from the prebindgen-generated Rust file via cbindgen, and
/// publish it to `include/example_flat_<arch>[_<feature>…].h` (per-variant, like
/// the Rust file).
fn generate_c_headers(bindings_file: &Path) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let header_path = PathBuf::from(&out_dir).join("example_flat.h");

    match cbindgen::Builder::new()
        .with_config(config)
        .with_crate(&crate_dir)
        .with_src(bindings_file) // the prebindgen-generated Rust file
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(&header_path);
            // Publish the header to the in-tree, committed `include/` dir, per-variant.
            let stable = PathBuf::from(&crate_dir)
                .join("include")
                .join(format!("{}.h", variant()));
            write_if_changed(&stable, &std::fs::read_to_string(&header_path).unwrap());
            println!("cargo:warning=Generated C header at: {}", stable.display());
        }
        Err(e) => {
            println!("cargo:warning=Failed to generate C header: {e:?}");
        }
    }
}

/// Overwrite `path` only when `contents` differs from what is already there
/// (a no-op otherwise), so re-running the build introduces no spurious changes.
fn write_if_changed(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::read_to_string(path).ok().as_deref() != Some(contents) {
        std::fs::write(path, contents).expect("write generated artifact");
    }
}
