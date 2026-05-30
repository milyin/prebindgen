//! `Cbindgen` — the C / cbindgen language adapter.
//!
//! A [`Prebindgen`] back-end that turns a "flat" `#[prebindgen]` library into a
//! Rust file suitable for [`cbindgen`](https://github.com/mozilla/cbindgen) to
//! parse into a C header plus a static / dynamic library.
//!
//! Like the JNI back-end, items are **opt-in**: nothing is converted unless it
//! is explicitly declared with [`Cbindgen::function`] / [`Cbindgen::struct_`] /
//! [`Cbindgen::enum_`]. With no declarations the adapter emits an empty library
//! — the scaffolding state this module currently provides.
//!
//! The actual conversion (emitting `#[no_mangle] extern "C"` wrappers and
//! mapping Rust types onto C-ABI wire types) is added incrementally; for now
//! the `on_*` hooks are stubs.

use std::collections::HashSet;

use proc_macro2::TokenStream;

use crate::api::core::prebindgen::{ConverterImpl, Prebindgen};
use crate::api::core::registry::{Registry, TypeKey};

/// C / cbindgen language adapter. Build it with [`Cbindgen::new`] and declare
/// the items to convert with the fluent methods, then drive it through
/// [`Registry::write_rust`](crate::core::Registry::write_rust).
#[derive(Default)]
pub struct Cbindgen {
    /// Optional module path the original `#[prebindgen]` items live under, used
    /// to qualify bare references in generated bodies (cf. `JniExt`'s
    /// `source_module`). Unused by the current scaffolding.
    source_module: Option<syn::Path>,
    /// Idents of `#[prebindgen]` functions explicitly declared for conversion.
    functions: HashSet<syn::Ident>,
    /// Keys of structs / enums explicitly declared for conversion.
    types: HashSet<TypeKey>,
}

impl Cbindgen {
    /// Create an adapter with no declarations (emits an empty library).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the module path the original `#[prebindgen]` items live under.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = Some(p);
        self
    }

    /// Declare a `#[prebindgen]` function to convert into the C layer.
    pub fn function(mut self, ident: syn::Ident) -> Self {
        self.functions.insert(ident);
        self
    }

    /// Declare a struct (by type) to convert.
    pub fn struct_(mut self, ty: syn::Type) -> Self {
        self.types.insert(TypeKey::from_type(&ty));
        self
    }

    /// Declare an enum (by type) to convert.
    pub fn enum_(mut self, ty: syn::Type) -> Self {
        self.types.insert(TypeKey::from_type(&ty));
        self
    }
}

impl Prebindgen for Cbindgen {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.clone()
    }

    fn declared_types(&self) -> HashSet<TypeKey> {
        self.types.clone()
    }

    // ── Item emission (scaffolding stubs) ──────────────────────────────

    fn on_function(&self, _f: &syn::ItemFn, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }

    fn on_struct(&self, _s: &syn::ItemStruct, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }

    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }

    // ── Input direction (scaffolding stubs) ────────────────────────────

    fn on_input_type_rank_0(&self, _ty: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_input_type_rank_1(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_input_type_rank_2(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_input_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    // ── Output direction (scaffolding stubs) ───────────────────────────

    fn on_output_type_rank_0(&self, _ty: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_output_type_rank_1(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_output_type_rank_2(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_output_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An adapter with no declarations writes an empty (whitespace-only) file.
    #[test]
    fn empty_adapter_writes_empty_file() {
        let dir = std::env::temp_dir().join(format!("cbindgen_scaffold_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("empty.rs");

        let cbindgen = Cbindgen::new();
        let mut registry: Registry<()> = Registry::default();
        let path = registry.write_rust(&cbindgen, &out).expect("write_rust");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.trim().is_empty(),
            "expected empty output, got:\n{contents}"
        );
    }
}
