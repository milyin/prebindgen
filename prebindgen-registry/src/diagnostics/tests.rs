use std::collections::HashSet;

use prebindgen_flat::flat::Flat;

use super::*;

fn flat_with(sources: &[&str]) -> Flat {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, prebindgen::SourceLocation::default())
        })
        .collect::<Vec<_>>();
    Flat::builder().items(items).build().expect("index")
}

fn ident(n: &str) -> syn::Ident {
    syn::parse_str(n).unwrap()
}

/// The default report: everything captured and nothing claimed is a skip, one
/// line per item, functions before struct/enums.
#[test]
fn every_unclaimed_item_is_reported_once() {
    let flat = flat_with(&[
        "pub fn a(x: u64) -> u64 { x }",
        "pub fn b(x: u64) -> u64 { x }",
        "pub struct S { pub v: u64 }",
        "pub enum E { X = 1 }",
    ]);
    let lines = unclaimed_report(&flat, &Claimed::default());
    assert_eq!(
        lines,
        vec![
            "prebindgen: skipping undeclared #[prebindgen] fn `a`",
            "prebindgen: skipping undeclared #[prebindgen] fn `b`",
            "prebindgen: skipping undeclared #[prebindgen] struct/enum `E`",
            "prebindgen: skipping undeclared #[prebindgen] struct/enum `S`",
        ]
    );
}

/// A claim silences the skip — whether the binding emits the item or only
/// references it (helpers and boundary-only types are folded into these sets
/// by the generator, precisely so both count as claimed).
#[test]
fn a_claimed_item_is_not_reported() {
    let flat = flat_with(&[
        "pub fn a(x: u64) -> u64 { x }",
        "pub struct S { pub v: u64 }",
    ]);
    let claimed = Claimed {
        functions: HashSet::from([ident("a")]),
        types: HashSet::from([TypeKey::parse("S").unwrap()]),
        ..Claimed::default()
    };
    assert!(unclaimed_report(&flat, &claimed).is_empty());
}

/// An ignore silences the skip exactly like a declaration does — of every kind.
/// The two differ only in what the registry does with them, which here is
/// nothing.
#[test]
fn an_ignored_item_is_not_reported_as_skipped() {
    let flat = flat_with(&[
        "pub fn a(x: u64) -> u64 { x }",
        "pub struct S { pub v: u64 }",
        "pub const K: u64 = 7;",
    ]);
    let claimed = Claimed {
        consts: Some(HashSet::new()),
        ignored_functions: HashSet::from([ident("a")]),
        ignored_types: HashSet::from([TypeKey::parse("S").unwrap()]),
        ignored_consts: HashSet::from([ident("K")]),
        ..Claimed::default()
    };
    assert!(
        unclaimed_report(&flat, &claimed).is_empty(),
        "an ignore must suppress the skip line, not just the stale-ignore line"
    );
}

/// A stale ignore — one naming nothing — is itself worth a line, because it
/// means build.rs has drifted from the source crate. It is only ever a warning:
/// a *declaration* that matches nothing is a hard error the registry raises.
#[test]
fn a_stale_ignore_is_reported() {
    let flat = flat_with(&["pub fn a(x: u64) -> u64 { x }"]);
    let claimed = Claimed {
        functions: HashSet::from([ident("a")]),
        ignored_functions: HashSet::from([ident("gone_fn")]),
        ignored_types: HashSet::from([TypeKey::parse("Gone").unwrap()]),
        ..Claimed::default()
    };
    assert_eq!(
        unclaimed_report(&flat, &claimed),
        vec![
            "prebindgen: ignored function `gone_fn` not found among #[prebindgen] items",
            "prebindgen: ignored type `Gone` not found among #[prebindgen] items",
        ]
    );
}

/// An **alias** is a captured item, so ignoring one by name is not stale —
/// `declared_type` counts aliases, unlike the struct/enum skip population.
#[test]
fn ignoring_an_alias_is_not_stale() {
    let flat = flat_with(&[
        "pub type Handle = other::Inner;",
        "pub fn f(x: u64) -> u64 { x }",
    ]);
    let claimed = Claimed {
        functions: HashSet::from([ident("f")]),
        ignored_types: HashSet::from([TypeKey::parse("Handle").unwrap()]),
        ..Claimed::default()
    };
    assert!(unclaimed_report(&flat, &claimed).is_empty());
    // …and the alias is never itself a "skipping undeclared struct/enum",
    // because it is neither.
    assert!(unclaimed_report(&flat, &Claimed::default())
        .iter()
        .all(|l| !l.contains("Handle")));
}

/// An ignore predicate acknowledges matching undeclared items of EVERY kind —
/// fn, struct/enum, const (one flat namespace, so a name filter needs no kind)
/// — and a predicate matching nothing is silent: it is a filter, not a claim.
#[test]
fn an_ignore_predicate_covers_every_kind_and_is_silent_when_unmatched() {
    let flat = flat_with(&[
        "pub fn helper_a(x: u64) -> u64 { x }",
        "pub fn helper_b(x: u64) -> u64 { x }",
        "pub struct HelperThing { pub v: u64 }",
        "pub const HELPER_MAX: u64 = 1;",
    ]);
    let claimed = Claimed {
        // Some(..) = this binding HAS a const mechanism, so consts are reported.
        consts: Some(HashSet::new()),
        ignored_name_predicates: vec![
            std::sync::Arc::new(|n: &str| n.to_lowercase().starts_with("helper")),
            // A second, zero-match predicate is fine and says nothing.
            std::sync::Arc::new(|n: &str| n.starts_with("nothing_")),
        ],
        ..Claimed::default()
    };
    assert!(unclaimed_report(&flat, &claimed).is_empty());
}

/// `consts: None` means the binding has no const mechanism at all: every const
/// is re-emitted verbatim, so none was skipped and reporting one would be a lie.
/// This is the one asymmetry with functions and types.
#[test]
fn no_const_mechanism_reports_no_consts() {
    let flat = flat_with(&["pub const K: u64 = 7;"]);

    assert!(unclaimed_report(&flat, &Claimed::default()).is_empty());

    let declares_consts = Claimed {
        consts: Some(HashSet::new()),
        ..Claimed::default()
    };
    assert_eq!(
        unclaimed_report(&flat, &declares_consts),
        vec!["prebindgen: skipping undeclared #[prebindgen] const `K`"]
    );
}
