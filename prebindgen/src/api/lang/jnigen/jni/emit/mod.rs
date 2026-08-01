//! JNI `extern "C"` wrapper and converter-body emission (free fns).
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

// ──────────────────────────────────────────────────────────────────────
// Function-wrapper emission (JNI extern "C")
// ──────────────────────────────────────────────────────────────────────

use super::*;

mod callback;
mod convert;
mod delivery;
mod flat_input;
mod names;
mod struct_out;
mod sum_out;
mod vec_build;
mod wrapper;

pub(crate) use callback::*;
pub(crate) use convert::*;
pub(crate) use delivery::*;
pub(crate) use flat_input::*;
pub(crate) use names::*;
pub(crate) use struct_out::*;
pub(crate) use sum_out::*;
pub(crate) use vec_build::*;
pub(crate) use wrapper::*;

#[cfg(test)]
mod destructure_ledger {
    //! A committed census of every place this adapter **destructures an
    //! `Option` in emitted Rust**.
    //!
    //! `kind` says a position is optional. It deliberately does not say whether
    //! Rust spells that `Option<T>`, `Box<Option<T>>`, `Cow<'_, Option<T>>` or
    //! anything else — the flat model states the destination-language
    //! invariant, and the side reading it is the side that must accept any
    //! representation. So an emitter may classify off `kind`, but it must not
    //! *spell* off it, and matching a reached place against `Option`'s patterns
    //! does exactly that.
    //!
    //! That is how #268 happened: `match &place { Some(..) => .. }` is `E0308`
    //! the moment the source spells the field `Box<Option<T>>`, and no test
    //! could see it because this suite asserts on generated text and never
    //! compiles it. The fix is [`bind_as_option`] — a type-ascribed `let` is a
    //! coercion site, deref coercion is transitive and a no-op when the types
    //! already match, so one shape serves every representation.
    //!
    //! ## What this counts, and what it cannot
    //!
    //! Only **patterns** — `Option::Some(..) =>` — never constructions. Building
    //! an `Option` the emitter itself owns says nothing about the source's
    //! representation and is always safe, which is why `flat_input.rs` and
    //! `wrapper.rs` sit at zero here despite emitting `Option::Some` freely.
    //!
    //! It cannot tell a *coerced* destructure from a raw one, so a count that
    //! does not move is not proof of correctness. Its job is to make a NEW
    //! destructure impossible to add silently: the number moves, and review has
    //! to say which of the three it is —
    //!
    //!   * destructuring a coerced binding (fine — that is the fix),
    //!   * destructuring a value the emitter itself bound (fine — the emitter
    //!     owns its type),
    //!   * destructuring a place read from the source (**the bug**; route it
    //!     through [`bind_as_option`]).
    //!
    //! Owned positions are the standing exception: deref coercion applies to
    //! references, and moving a payload out of a wrapper is something only some
    //! of them permit (`Box` can, `Rc` cannot), so a site that must MOVE keeps
    //! its direct match and is correct only for representations that allow it.
    //!
    //! To change it deliberately: update the table below in the same commit,
    //! and say in review which category the new site is.

    /// `(file, destructuring-pattern count)`.
    const LEDGER: &[(&str, usize)] = &[
        // 2 fn-return matches (owned), 2 leaf reaches (one coerced, one owned
        // accessor return), 1 owned identity move, 1 emitter-bound local.
        ("delivery.rs", 6),
        // The `Option<sum>` present-flag split — coerced.
        ("struct_out.rs", 1),
        // Constructions only.
        ("flat_input.rs", 0),
        ("wrapper.rs", 0),
    ];

    #[test]
    fn option_destructuring_sites_are_accounted_for() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/api/lang/jnigen/jni/emit");
        let mut drift: Vec<String> = Vec::new();
        for (file, expected) in LEDGER {
            let src = std::fs::read_to_string(std::path::Path::new(dir).join(file))
                .unwrap_or_else(|e| panic!("read {file}: {e}"));
            // A pattern arm is `Option::Some(<binding>) =>`; a construction has
            // no `=>` after its closing paren. Whitespace is stripped so line
            // wrapping cannot hide a site.
            let bare: String = src.chars().filter(|c| !c.is_whitespace()).collect();
            let found = bare
                .match_indices("option::Option::Some(")
                .filter(|(i, _)| {
                    bare[*i..]
                        .find(')')
                        .is_some_and(|j| bare[*i + j..].starts_with(")=>"))
                })
                .count();
            if found != *expected {
                drift.push(format!("  {file}: {expected} -> {found}"));
            }
        }
        assert!(
            drift.is_empty(),
            "OPTION-DESTRUCTURING LEDGER DRIFT:\n{}\n\n\
             An emitter must not assume how Rust spells an optional — see this \
             module's docs. If the new site destructures a place read from the \
             source, route it through `bind_as_option`; if it is a coerced \
             binding, an emitter-owned value, or an owned move, update the \
             table and say which in review.",
            drift.join("\n"),
        );
    }
}
