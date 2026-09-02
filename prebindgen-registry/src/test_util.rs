//! Shared helpers for the crate's unit tests. Compiled only under
//! `cfg(test)`; not part of any public or crate API.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use prebindgen::SourceLocation;

use crate::registry::{Registry, RegistryBuilder};

/// Index a `Registry` from a list of Rust item sources.
///
/// Accepts any item, not just a fn, so a fixture can declare the types it names.
/// Whatever it does *not* declare is supplied by [`declare_referenced`], because
/// these fixtures exist to exercise plan shapes and a handle declaration is noise
/// in them.
pub(crate) fn reg_with(sources: &[&str]) -> RegistryBuilder {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, SourceLocation::default())
        })
        .collect::<Vec<_>>();
    reg_from_items(declare_referenced(items)).expect("index")
}

/// A **scanned** registry from item sources — for tests that drive `expand`
/// directly and need the type tables populated, without going through a
/// generator's conversion loop.
pub(crate) fn scanned_with(sources: &[&str]) -> Registry {
    reg_with(sources).scanned().expect("scan")
}

/// A type as a **build script** would declare it: real tokens, no source
/// position. What
/// [`RegistryBuilder::export_type`](crate::registry::RegistryBuilder::export_type)
/// takes.
pub(crate) fn declared_origin(ty: syn::Type) -> prebindgen_flat::flat::Origin<syn::Type> {
    prebindgen_flat::flat::Origin::new(ty, std::rc::Rc::new(SourceLocation::default()))
}

/// Build a `Registry` from an item stream, the way `Registry::from_items` used
/// to before reading captured output became `FlatBuilder`'s job alone.
///
/// Test-only sugar: the two steps are one line each in a build script, but they
/// appear in dozens of fixtures here.
pub(crate) fn reg_from_items<I>(items: I) -> Result<RegistryBuilder, crate::ScanError>
where
    I: IntoIterator<Item = (syn::Item, SourceLocation)>,
{
    let flat = prebindgen_flat::Flat::builder().items(items).build()?;
    Registry::builder(flat)
}

/// Append a marked type alias for every nominal type the stream names but never
/// declares, so a fixture satisfies the flat API's self-sufficiency rule.
///
/// A fixture that is *about* a handle's treatment already declares it; this covers
/// the ones where the handle is incidental — `reg_with(&["fn get(s: &Storage) -> Payload"])`
/// is testing a plan's shape, not what `Storage` is. Declaring them as
/// [`Extern`](prebindgen_flat::flat::Extern)s is exactly what a real source crate does
/// for a foreign handle, and it is inert for the registry either way: a type alias
/// lands in no registry map.
///
/// Runs to a fixed point, since a declaration can only ever resolve more references.
/// It cannot help a **path-qualified** name (`std::time::Duration`), which no
/// declaration can name — such a fixture has to spell the type bare and declare it.
pub(crate) fn declare_referenced<I>(items: I) -> Vec<(syn::Item, SourceLocation)>
where
    I: IntoIterator<Item = (syn::Item, SourceLocation)>,
{
    use prebindgen_flat::flat::{Flat, ItemError};

    let mut items: Vec<(syn::Item, SourceLocation)> = items.into_iter().collect();

    loop {
        let flat = Flat::builder()
            .items(items.iter().cloned())
            .build()
            .expect("fixture parses");
        // A set: the same name is reported once per referencing item.
        let missing: std::collections::BTreeSet<String> = flat
            .unsupported()
            .filter_map(|u| match &*u.error {
                // Skip a name the stream already holds. Refusal is transitive, so a
                // declared struct whose own field is undeclared reports as
                // unresolved too — declaring an alias for it would collide. Adding
                // the root name resolves it on the next round.
                ItemError::UnresolvedType { name }
                    if !name.contains("::") && flat.element(name).is_none() =>
                {
                    Some(name.clone())
                }
                _ => None,
            })
            .collect();
        if missing.is_empty() {
            return items;
        }
        for name in missing {
            let ident = quote::format_ident!("{name}");
            let alias: syn::Item = syn::parse_quote!(
                pub type #ident = __fixture::#ident;
            );
            items.push((alias, SourceLocation::default()));
        }
    }
}

/// A process-unique temp directory for a test that writes files. Keyed by
/// pid + a monotonic counter so tests that share a helper and run on
/// separate threads never clobber each other's output dir.
pub(crate) fn unique_test_dir(prefix: &str) -> PathBuf {
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), seq))
}

/// Test-only adapter exposing the same model-generated source tokens as a
/// production emission callback.
/// A model from item sources, for a test that needs a `Flat` and no registry
/// around it.
pub(crate) fn flat_with(sources: &[&str]) -> prebindgen_flat::flat::Flat {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, SourceLocation::default())
        })
        .collect::<Vec<_>>();
    crate::Flat::builder()
        .items(declare_referenced(items))
        .build()
        .expect("index")
}

pub(crate) trait EmitSourceForTest {
    fn emit_source(&self) -> proc_macro2::TokenStream;
}

impl EmitSourceForTest for prebindgen_flat::flat::TypeRef {
    fn emit_source(&self) -> proc_macro2::TokenStream {
        crate::RustWriter::for_test().emit_source_type(self)
    }
}

/// Write a capture directory holding `records`, the way a source crate's build
/// script and the `#[prebindgen]` macro write one: a `prebindgen_output.toml`
/// naming the crate, and the captures under the `default` group's directory.
///
/// The description is spelled out here rather than produced by prebindgen,
/// which writes it only from a build script. If its format number moves, this
/// fixture stops matching and the tests below say so — which is the point of
/// the number.
pub(crate) fn write_capture_dir(
    tag: &str,
    crate_name: &str,
    records: &[&prebindgen::Record],
) -> PathBuf {
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    let group_dir = dir.join(prebindgen::layout::group_dir_name(
        prebindgen::DEFAULT_GROUP_NAME,
    ));
    std::fs::create_dir_all(&group_dir).unwrap();
    std::fs::write(
        dir.join("prebindgen_output.toml"),
        format!("format = 1\n\n[package]\nname = \"{crate_name}\"\nfeatures = []\n"),
    )
    .unwrap();
    prebindgen::utils::write_to_jsonl_file(group_dir.join("captures.jsonl"), records).unwrap();
    dir
}

/// The recipe tests compile against a `Flat` they wrote, with no registry around
/// it, so the model answers for itself: `reading` classifies the spelling the
/// key came from, which is what a scanned cell would hold, and `crossing_keys`
/// is empty because no population was scanned.
impl crate::Conversions for prebindgen_flat::flat::Flat {
    fn flat(&self) -> &prebindgen_flat::flat::Flat {
        self
    }
    fn reading(&self, key: &prebindgen_flat::TypeKey) -> Option<prebindgen_flat::flat::TypeRef> {
        let ty: syn::Type = syn::parse_str(key.as_str()).ok()?;
        self.classify(&ty).ok()
    }
    fn crossing_keys(&self, _dir: crate::registry::Direction) -> Vec<prebindgen_flat::TypeKey> {
        Vec::new()
    }
}
