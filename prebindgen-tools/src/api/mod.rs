//! Implementation of the API selected in `lib.rs`.

pub(super) mod convert;
pub(super) mod file;
pub(super) mod names;
pub(super) mod place;
pub(super) mod qualify;
pub(super) mod record;
pub(super) mod shape;
pub(super) mod wire;

/// Refuse a model the frontend could not read in full: every
/// [`Unsupported`](prebindgen_flat::flat::Element::Unsupported) element, reported at
/// once. An adapter calls this before looking at any declaration, so a
/// binding is built against the whole source or not at all.
pub fn check_supported(flat: &prebindgen_flat::Flat) -> Result<(), String> {
    let bad: Vec<String> = flat.unsupported().map(|u| u.error.to_string()).collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the source declares items the flat model cannot express:\n  {}",
            bad.join("\n  ")
        ))
    }
}
