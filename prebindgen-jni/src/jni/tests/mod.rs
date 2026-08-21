// Only `entry` (below, gated off with the `niches` module) needs `TypeEntry`.
pub(crate) use prebindgen::SourceLocation;
use prebindgen_registry::TypeEntry;
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

mod aliasing;
mod callbacks;
mod config;
mod consts;
mod cross_artifact;
mod flatten;
// BLOCKED by the prebindgen-jni crate split: every test in this module calls
// `Registry::empty_for_test()` and/or `Emit::for_test()`, both `pub(crate)` in
// `prebindgen::core` — reachable when this module lived inside the
// `prebindgen` crate, not from the separate `prebindgen-jni` crate it moved
// to. Left in place, not deleted, pending a `prebindgen`-side test-support
// hook (see the carve-prebindgen-jni report). `install_input`/`install_output`
// below exist only to serve it and are gated off with it.
mod niches;
mod sealed;
mod snapshots;
mod symbols;
mod value_form;
mod values;

/// Build a `TypeEntry` for use in tests. The function body is not
/// inspected by `option_input` / `option_output`; only the ident,
/// destination, and niches matter, so we use a stub `ItemFn`.
fn entry(wire: syn::Type, conv_name: &str, niches: Niches) -> TypeEntry<KotlinMeta> {
    let ident = syn::Ident::new(conv_name, proc_macro2::Span::call_site());
    let func: syn::ItemFn = syn::parse_quote!(
        unsafe fn #ident<'env, 'v>(
            env: &mut jni::JNIEnv<'env>,
            v: &#wire,
        ) -> ::core::result::Result<(), __JniErr> {
            Ok(())
        }
    );
    TypeEntry {
        destination: wire,
        function: func,
        pre_stages: vec![],
        subs: vec![],
        niches,
        metadata: KotlinMeta::default(),
    }
}

// BLOCKED: `Registry::insert_crossing` is `pub(crate)` in `prebindgen::core` —
// see the `niches` module gate above.
/// Put one conversion where both a registry query and an adapter lookup can
/// find it: in the registry the test builds, and in `decls` as the fragment a
/// compiled binding would have filed. Helpers under test read the second.
fn install(
    reg: &mut Registry<KotlinMeta>,
    decls: &Declarations,
    direction: Direction,
    ty_str: &str,
    e: TypeEntry<KotlinMeta>,
) {
    let ty: syn::Type = syn::parse_str(ty_str).expect("test type");
    let key = TypeKey::from_type(&ty);
    let assembly = match direction {
        Direction::Input => prebindgen_registry::recipe::Assembly::Construct,
        Direction::Output => prebindgen_registry::recipe::Assembly::Deconstruct,
    };
    decls.compiled.borrow_mut().record(
        key.clone(),
        assembly,
        prebindgen_registry::recipe::RecipeId::new("whole"),
        crate::jni::compile::JFrag::by_hand(key, e.as_converter()),
    );
    reg.insert_crossing(direction, &ty, true, Some(e));
}

fn install_input(
    reg: &mut Registry<KotlinMeta>,
    decls: &Declarations,
    ty_str: &str,
    e: TypeEntry<KotlinMeta>,
) {
    install(reg, decls, Direction::Input, ty_str, e);
}

fn install_output(
    reg: &mut Registry<KotlinMeta>,
    decls: &Declarations,
    ty_str: &str,
    e: TypeEntry<KotlinMeta>,
) {
    install(reg, decls, Direction::Output, ty_str, e);
}
