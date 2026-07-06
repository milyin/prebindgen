use quote::ToTokens;

use super::*;

fn rules(enabled: &[&str], disabled: &[&str]) -> CfgExprRules {
    CfgExprRules {
        enabled_features: enabled.iter().map(|s| s.to_string()).collect(),
        disabled_features: disabled.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

/// A `#[cfg(feature = "...")]` on a single function parameter is honored:
/// dropped when the feature is disabled, kept (cfg attr stripped) when
/// enabled — without needing two whole-function variants.
#[test]
fn cfg_on_fn_parameter_is_filtered() {
    let src = SourceLocation::default();
    let make = || -> syn::Item {
        syn::parse_quote! {
            pub fn f(a: i32, #[cfg(feature = "unstable")] b: i64) -> i32 { a }
        }
    };

    // Disabled → the guarded parameter is removed; the rest is intact.
    let mut item = make();
    assert!(process_item_features(
        &mut item,
        &rules(&[], &["unstable"]),
        &src
    ));
    let s = item.to_token_stream().to_string();
    assert!(s.contains("a : i32"), "{s}");
    assert!(!s.contains("b : i64"), "{s}");
    assert!(!s.contains("cfg"), "cfg attr should be gone: {s}");

    // Enabled → the parameter is kept and its cfg attribute is stripped.
    let mut item = make();
    assert!(process_item_features(
        &mut item,
        &rules(&["unstable"], &[]),
        &src
    ));
    let s = item.to_token_stream().to_string();
    assert!(s.contains("a : i32"), "{s}");
    assert!(s.contains("b : i64"), "{s}");
    assert!(
        !s.contains("cfg"),
        "cfg attr should be stripped on keep: {s}"
    );
}

/// Parameters without a cfg attribute are untouched.
#[test]
fn fn_parameters_without_cfg_are_preserved() {
    let src = SourceLocation::default();
    let mut item: syn::Item = syn::parse_quote! {
        pub fn g(x: u8, y: u16, z: u32) {}
    };
    assert!(process_item_features(
        &mut item,
        &rules(&[], &["unstable"]),
        &src
    ));
    let s = item.to_token_stream().to_string();
    assert!(
        s.contains("x : u8") && s.contains("y : u16") && s.contains("z : u32"),
        "{s}"
    );
}
