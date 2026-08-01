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
    //! ## Why this walks tokens
    //!
    //! The first version of this check matched the text `option::Option::Some(`
    //! and was wrong in the way that matters: `struct_out.rs` emitted a second
    //! destructure spelled bare `Some(#cbind) =>`, and the census could not see
    //! it. That site was a real unfixed instance of the very bug — an
    //! unqualified match on a source place — so the guard's blind spot was
    //! exactly the bug's hiding place. A guard that can be evaded by choosing a
    //! different spelling of the same pattern is not a guard.
    //!
    //! So the scan parses each file's tokens, descends only into `quote!`
    //! bodies — which is what "emitted Rust" means, and keeps the emitter's own
    //! `if let Some(x)` out of the count — and recognizes a `Some ( … ) =>`
    //! pattern whatever path prefix it carries.
    //!
    //! ## What it counts, and what it cannot
    //!
    //! Only **patterns**, never constructions. Building an `Option` the emitter
    //! itself owns says nothing about the source's representation and is always
    //! safe, which is why `flat_input.rs` and `wrapper.rs` sit at zero despite
    //! emitting `Option::Some` freely.
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
    //! ## Coverage
    //!
    //! The table is checked against the **directory listing**, not kept by
    //! hand. A hand-listed census had already let `convert.rs` sit outside the
    //! guard entirely with two uncounted destructures, and would have let any
    //! new emitter module do the same — "a new destructure cannot be added
    //! silently" is only true if a new *file* cannot be either.
    //!
    //! To change it deliberately: update the table below in the same commit,
    //! and say in review which category the new site is.

    use proc_macro2::{Delimiter, TokenStream, TokenTree};

    /// `(file, destructuring-pattern count)` — **every** `.rs` in this
    /// directory, checked against the directory listing so a new emitter module
    /// cannot sit outside the census.
    const LEDGER: &[(&str, usize)] = &[
        ("callback.rs", 0),
        // The `Option<T>` output converters' niche and boxed-primitive arms.
        // They destructure `v`, the converter's own parameter — whose Rust type
        // is the crossing's SPELLING. Correct today only because a wrapped
        // spelling gets no converter at all (#270); if that is fixed and
        // `Box<Option<T>>` becomes a crossing, these two need coercing.
        ("convert.rs", 2),
        // 2 fn-return matches (owned), 2 leaf reaches (one coerced, one owned
        // accessor return), 1 owned identity move, 1 emitter-bound local.
        ("delivery.rs", 6),
        ("flat_input.rs", 0),
        // This file. The spellings in `the_census_is_spelling_independent` are
        // string literals, which tokenize as one `Literal` and are never walked.
        ("mod.rs", 0),
        ("names.rs", 0),
        // Both coerced: the `Option<sum>` present-flag split, and the nested
        // plan's present-flag split.
        ("struct_out.rs", 2),
        ("sum_out.rs", 0),
        ("vec_build.rs", 0),
        ("wrapper.rs", 0),
    ];

    /// Count `Some ( … ) =>` patterns inside `quote!` / `parse_quote!` bodies.
    ///
    /// `in_quote` is what keeps the emitter's own `if let Some(..)` out of the
    /// count: only tokens below one of those macros are emitted Rust.
    fn count(ts: TokenStream, in_quote: bool, n: &mut usize) {
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        let mut i = 0;
        while i < toks.len() {
            if let TokenTree::Ident(id) = &toks[i] {
                let bang =
                    matches!(toks.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!');
                if (id == "quote" || id == "parse_quote") && bang {
                    if let Some(TokenTree::Group(g)) = toks.get(i + 2) {
                        count(g.stream(), true, n);
                        i += 3;
                        continue;
                    }
                }
                // A `Some( … ) =>` arm. The path in front is not inspected, so
                // `Some`, `Option::Some` and `::core::option::Option::Some` all
                // land here — the spelling is what the first version of this
                // check wrongly keyed on.
                if in_quote && id == "Some" {
                    let parens = matches!(
                        toks.get(i + 1),
                        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis
                    );
                    // `=>` is two Puncts: '=' joint, then '>'.
                    let fat_arrow = matches!(toks.get(i + 2), Some(TokenTree::Punct(p)) if p.as_char() == '=')
                        && matches!(toks.get(i + 3), Some(TokenTree::Punct(p)) if p.as_char() == '>');
                    if parens && fat_arrow {
                        *n += 1;
                    }
                }
            }
            if let TokenTree::Group(g) = &toks[i] {
                count(g.stream(), in_quote, n);
            }
            i += 1;
        }
    }

    #[test]
    fn option_destructuring_sites_are_accounted_for() {
        let dir = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/api/lang/jnigen/jni/emit"
        ));

        // The census is over the DIRECTORY, not a hand-kept list: a new emitter
        // module that hand-listing would have let slip past the guard entirely
        // shows up here as an unlisted file.
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .expect("emit dir")
            .map(|e| e.expect("dir entry").path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .map(|p| {
                p.file_name()
                    .expect("file name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        on_disk.sort();
        let mut listed: Vec<String> = LEDGER.iter().map(|(f, _)| (*f).to_string()).collect();
        listed.sort();
        assert_eq!(
            listed, on_disk,
            "the destructure census must cover every emitter file — add the new              module to LEDGER with its count (see this module's docs for what              counts)"
        );

        let mut drift: Vec<String> = Vec::new();
        for (file, expected) in LEDGER {
            let src = std::fs::read_to_string(dir.join(file))
                .unwrap_or_else(|e| panic!("read {file}: {e}"));
            let ts: TokenStream = src
                .parse()
                .unwrap_or_else(|e| panic!("tokenize {file}: {e}"));
            let mut found = 0usize;
            count(ts, false, &mut found);
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

    /// The guard catches a destructure **however it is spelled** — the failure
    /// the first version had, and the reason this one walks tokens.
    ///
    /// A ledger that has never been seen to fail is not evidence of anything,
    /// so this proves it on both spellings rather than asserting it in prose.
    #[test]
    fn the_census_is_spelling_independent() {
        let one = |body: &str| {
            let src = format!("fn f() {{ quote! {{ match x {{ {body} }} }}; }}");
            let mut n = 0usize;
            count(src.parse().expect("tokenize"), false, &mut n);
            n
        };
        for spelling in [
            "Some(v) => {}",
            "Option::Some(v) => {}",
            "::core::option::Option::Some(v) => {}",
        ] {
            assert_eq!(one(spelling), 1, "not counted: {spelling}");
        }
        // A construction is not a destructure, and the emitter's own control
        // flow is not emitted Rust.
        let mut n = 0usize;
        count(
            "fn f() { if let Some(x) = y { quote! { Some(#x) }; } }"
                .parse()
                .expect("tokenize"),
            false,
            &mut n,
        );
        assert_eq!(n, 0, "constructions and host-code matches must not count");
    }
}
