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

mod aliasing;
mod callbacks;
mod config;
mod consts;
mod cross_artifact;
mod flatten;
mod sealed;
mod snapshots;
mod symbols;
mod value_form;
mod values;
