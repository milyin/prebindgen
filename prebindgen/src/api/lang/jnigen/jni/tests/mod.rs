use quote::ToTokens;

use super::*;
use crate::api::{
    core::{
        niches::{NicheSlot, Niches},
        registry::{Registry, TypeEntry, TypeKey},
    },
    test_util::unique_test_dir,
};

mod callbacks;
mod config;
mod consts;
mod flatten;
mod niches;
mod snapshots;
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
        required: false,
        niches,
        metadata: KotlinMeta::default(),
    }
}

fn install_input(
    reg: &mut Registry<KotlinMeta>,
    ty_str: &str,
    _rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.input_types.insert(TypeKey::parse(ty_str), Some(e));
}

fn install_output(
    reg: &mut Registry<KotlinMeta>,
    ty_str: &str,
    _rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.output_types.insert(TypeKey::parse(ty_str), Some(e));
}
