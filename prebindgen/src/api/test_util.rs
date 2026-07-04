//! Shared helpers for the crate's unit tests. Compiled only under
//! `cfg(test)`; not part of any public or crate API.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::api::core::registry::Registry;

/// Index a `Registry` from a list of Rust function sources.
pub(crate) fn reg_with(fns: &[&str]) -> Registry<()> {
    let items = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), crate::SourceLocation::default())
        })
        .collect::<Vec<_>>();
    Registry::from_items(items).expect("index")
}

/// A process-unique temp directory for a test that writes files. Keyed by
/// pid + a monotonic counter so tests that share a helper and run on
/// separate threads never clobber each other's output dir.
pub(crate) fn unique_test_dir(prefix: &str) -> PathBuf {
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), seq))
}
