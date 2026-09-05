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
#[cfg(feature = "v2")]
mod v2;

fn write(cbindgen: CbindgenBuilder, registry: RegistryBuilder, tag: &str) -> String {
    let dir = unique_test_dir(&format!("cbindgen_{tag}"));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{tag}.rs"));
    let gen = cbindgen.build_over(registry).expect("resolve");
    let path = gen.write_rust(&out).expect("write_rust");
    std::fs::read_to_string(&path).unwrap()
}

/// Whether a final registry-owned operation whose name starts with `stem`
/// is called with `argument` in whitespace-compacted generated Rust.
///
/// Private converter names intentionally end in a stable identity hash. Tests
/// should pin the readable semantic stem and the call shape, not that suffix.
fn operation_call(compact: &str, stem: &str, argument: &str) -> bool {
    compact.match_indices(stem).any(|(start, _)| {
        let rest = &compact[start + stem.len()..];
        rest.find('(')
            .is_some_and(|open| rest[open + 1..].starts_with(argument))
    })
}

fn operation_name<'a>(compact: &'a str, stem: &str) -> Option<&'a str> {
    let start = compact.find(stem)?;
    let end = compact[start..].find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))? + start;
    Some(&compact[start..end])
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
