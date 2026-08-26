use prebindgen::SourceLocation;
use prebindgen_registry::RegistryBuilder;

use super::*;
pub(crate) use crate::test_util::declare_referenced;
use crate::test_util::unique_test_dir;

mod aliasing;
mod boundary_invariants;
mod builder;
mod callbacks;
mod errors;
mod inputs;
mod lowering;
mod recipes;
mod returns;
mod structs;
mod tagged_unions;

fn write(cbindgen: CbindgenBuilder, registry: RegistryBuilder, tag: &str) -> String {
    let dir = unique_test_dir(&format!("cbindgen_{tag}"));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{tag}.rs"));
    let gen = cbindgen.build_with(registry).expect("resolve");
    assert_converter_identity_matches_compatibility_names(&gen);
    let path = gen.write_rust(&out).expect("write_rust");
    std::fs::read_to_string(&path).unwrap()
}

fn assert_converter_identity_matches_compatibility_names(gen: &Cbindgen) {
    let mut names_by_operation = std::collections::HashMap::new();
    let mut operations_by_name = std::collections::HashMap::new();
    for function in gen.gen.converter_functions() {
        let (operation, name) = function.operation_and_compatibility_name();
        let name = name.to_string();
        if let Some(previous) = names_by_operation.insert(operation.clone(), name.clone()) {
            assert_eq!(
                previous, name,
                "one semantic C operation must not select two compatibility names"
            );
        }
        if let Some(previous) = operations_by_name.insert(name.clone(), operation.clone()) {
            assert_eq!(
                previous, *operation,
                "one compatibility name must not hide two semantic C operations: {name}"
            );
        }
    }
}

fn error_struct() -> syn::ItemStruct {
    syn::parse_quote!(
        pub struct Error {
            pub message: String,
        }
    )
}

fn catch<F: FnOnce()>(f: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
}

/// Like [`catch`], but returns the panic MESSAGE — for asserting that a
/// rejection names the reason that actually applies, not merely that it
/// rejected.
fn catch_msg<F: FnOnce()>(f: F) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .err()
        .and_then(|e| {
            e.downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
        })
        .expect("expected a panic carrying a message")
}
