pub(crate) use prebindgen::SourceLocation;
use quote::ToTokens;

use super::*;
pub(crate) use crate::test_util::{declare_referenced, unique_test_dir};

/// A test item's `SourceLocation` stamped with the tests' canonical source
/// crate `myflat` — the production path records origins from stream stamps
/// (`Source` fills them at parse time), so tests build their items the same
/// way instead of poking a registry-level override.
fn myflat_loc() -> prebindgen::SourceLocation {
    prebindgen::SourceLocation {
        crate_name: Some("myflat".to_string()),
        ..Default::default()
    }
}

/// Locate one generated Rust function by semantic evidence in its signature
/// and body. Private converter symbols are deliberately chosen only during
/// final rendering, so tests must not reverse-engineer meaning from them.
fn generated_function(
    rust: &str,
    signature_needles: &[&str],
    body_needles: &[&str],
) -> syn::ItemFn {
    let file = syn::parse_file(rust).expect("generated Rust parses");
    file.items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) => Some(function),
            _ => None,
        })
        .find(|function| {
            let signature = function.sig.to_token_stream().to_string();
            let body = function.block.to_token_stream().to_string();
            signature_needles.iter().all(|needle| signature.contains(needle))
                && body_needles.iter().all(|needle| body.contains(needle))
        })
        .unwrap_or_else(|| {
            panic!(
                "generated function not found; signature={signature_needles:?}, body={body_needles:?}\n{rust}"
            )
        })
}

mod aliasing;
mod callbacks;
mod config;
mod consts;
mod cross_artifact;
mod flatten;
mod sealed;
mod snapshots;
mod symbols;
#[cfg(feature = "v2")]
mod v2;
mod value_form;
mod values;
