//! Shared helpers for the crate's unit tests. Compiled only under
//! `cfg(test)`; not part of any public or crate API.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::api::core::registry::{Registry, TypeCell, TypeEntry, TypeSubject};

/// A type-table cell for a fixture.
///
/// The subject is always [`TypeSubject::Adapter`]: a hand-built table has no
/// `Flat` behind it, so no key in one has a source reading. A test that cares
/// about the `Source` side builds its registry from items instead.
///
/// Takes no key: `Adapter` carries nothing, since nothing ever read it back.
pub(crate) fn cell<M>(root: bool, entry: Option<TypeEntry<M>>) -> TypeCell<M> {
    TypeCell {
        subject: TypeSubject::Adapter,
        root,
        entry,
    }
}

/// Index a `Registry` from a list of Rust item sources.
///
/// Accepts any item, not just a fn, so a fixture can declare the types it names.
/// Whatever it does *not* declare is supplied by [`declare_referenced`], because
/// these fixtures exist to exercise plan shapes and a handle declaration is noise
/// in them.
pub(crate) fn reg_with(sources: &[&str]) -> Registry<()> {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, crate::SourceLocation::default())
        })
        .collect::<Vec<_>>();
    reg_from_items(declare_referenced(items)).expect("index")
}

/// Build a `Registry` from an item stream, the way `Registry::from_items` used
/// to before reading captured output became `FlatBuilder`'s job alone.
///
/// Test-only sugar: the two steps are one line each in a build script, but they
/// appear in dozens of fixtures here.
pub(crate) fn reg_from_items<M, I>(items: I) -> Result<Registry<M>, crate::core::ScanError>
where
    I: IntoIterator<Item = (syn::Item, crate::SourceLocation)>,
{
    let flat = crate::core::Flat::builder().items(items).build()?;
    Registry::new(flat)
}

/// Append a marked type alias for every nominal type the stream names but never
/// declares, so a fixture satisfies the flat API's self-sufficiency rule.
///
/// A fixture that is *about* a handle's treatment already declares it; this covers
/// the ones where the handle is incidental — `reg_with(&["fn get(s: &Storage) -> Payload"])`
/// is testing an unfold plan, not what `Storage` is. Declaring them as
/// [`Extern`](crate::core::flat::Extern)s is exactly what a real source crate does
/// for a foreign handle, and it is inert for the registry either way: a type alias
/// lands in no registry map.
///
/// Runs to a fixed point, since a declaration can only ever resolve more references.
/// It cannot help a **path-qualified** name (`std::time::Duration`), which no
/// declaration can name — such a fixture has to spell the type bare and declare it.
pub(crate) fn declare_referenced<I>(items: I) -> Vec<(syn::Item, crate::SourceLocation)>
where
    I: IntoIterator<Item = (syn::Item, crate::SourceLocation)>,
{
    use crate::api::core::flat::{Flat, ItemError};

    let mut items: Vec<(syn::Item, crate::SourceLocation)> = items.into_iter().collect();

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
            items.push((alias, crate::SourceLocation::default()));
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
