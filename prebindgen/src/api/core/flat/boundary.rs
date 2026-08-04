//! The mechanical boundary check: a committed ledger of every place outside
//! this module that can still take captured Rust syntax apart.
//!
//! Issue #211's sixth completion criterion is that a mechanical check must
//! prevent new source-syntax classifiers from appearing outside the frontend.
//! This is that check, and it counts **two populations**:
//!
//! * **classification sites** — variant mentions of a [`WATCHED`] syn enum. What
//!   a consumer visibly *does* with a node.
//! * **escapes** — calls to [`Origin::as_syn`](super::Origin::as_syn), the one
//!   route from the model to a `syn` node at all. What a consumer *can* do.
//!
//! The second exists because the first can only measure. Since the model's
//! syntax is sealed — `spell()` yields tokens, `as_syn()` yields the node — a
//! file with no escapes cannot classify source syntax however it is written, and
//! the ledger's own blind spots (an ident compared by name, a helper handed the
//! node, a syn enum outside `WATCHED`) are gated behind the same call. Counting
//! them turns "we grep for what we thought of" into "we count the door".
//!
//! Both are read the same way and fail the same way: a count that moved is a
//! ledger edit, and the diff is the decision. It is test-only; it ships no
//! production code.
//!
//! ## What a site is
//!
//! A place that looks at captured Rust syntax and asks *what shape is this*:
//!
//! ```ignore
//! syn::Type::Reference(r) => vec![(*r.elem).clone()],          // core/types_util.rs
//! if !matches!(arg_ty, syn::Type::Reference(_)) => { .. }      // jnigen emit/wrapper.rs
//! let syn::Type::Slice(s) = &*r.elem else { .. };              // cbindgen builder.rs
//! ```
//!
//! Each is an independent decision about what the source Rust *means* — the
//! decisions #211 says belong to this module alone. #210 was two such places,
//! in one file, disagreeing about `[u8; <Holder>::N]`.
//!
//! So [`scan_tree`] counts, per file, how many times a variant of a [`WATCHED`]
//! syn syntax enum is named, and [`boundary_ledger`] fails if any count moved.
//! Up means a new classifier landed outside the language. Down means one was
//! migrated — the goal, but it still edits the ledger, so the progress of the
//! adapter migrations shows as a diff instead of being invisible.
//!
//! The seed count is therefore high: nothing consumes elements yet, so this
//! freezes the population as it stands and every later PR pays it down.
//!
//! The number's job is to not move, not to be a precise census: a few counted
//! occurrences *build* syntax rather than classify it (`cbindgen/builder.rs`
//! returns a `syn::Type::Path`, which is emission). Separating build from match
//! mechanically costs real code, and those sites are few and stable.
//!
//! ## Why the count is read off disk
//!
//! The crate has no default features and `unstable-cbindgen` gates the whole
//! cbindgen suite, so CI runs both `cargo test` and `cargo test --all
//! --all-features`. A check that inspected the *compiled* crate would count a
//! different population in each, i.e. give two answers in one CI run — the
//! failure mode of #219. Reading the source files makes the count
//! feature-independent, so both invocations agree.
//!
//! ## Why tokens, not a grep and not an AST visit
//!
//! A `grep -c` counts lines rather than occurrences, counts the inline
//! `#[cfg(test)] mod` blocks (so *adding a test* would fail the check, which is
//! how these ledgers die), and is defeated by one line: `use syn::Type;` then
//! `Type::Reference(_)` greps as zero. A `syn::visit::Visit` walk fixes those
//! and opens a worse hole — it cannot see inside macro invocations, and a large
//! share of the sites live in `matches!(..)`. A token walk sees patterns,
//! expressions, types and macro bodies alike, and is immune to line wrapping.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// The syn syntax enums whose variants count as a classification site.
///
/// Deliberately minimal. It is the population #211 and `docs/source-frontend.md`
/// already track, and the low-noise one: emitters construct `syn::Item::Fn`
/// constantly but rarely construct a `syn::Type` or `syn::Expr`, so adding
/// `Item` here would churn the ledger on every emission change.
///
/// Extending this list is one line, plus a regenerated ledger whose diff is the
/// classification decision.
const WATCHED: &[&str] = &["Type", "Expr"];

/// Ledger location, relative to `src/`.
const LEDGER: &str = "api/core/flat/boundary.ledger";

/// Regenerated verbatim on every write, so the contract cannot drift from the
/// numbers underneath it.
const HEADER: &str = "\
# prebindgen source-syntax boundary ledger — issue #211.
#
# One line per file OUTSIDE `api/core/flat/`, counting how many times a
# variant of a watched syn syntax enum is named in production code:
#
#     syn::Type::Reference(r) => ...        <- one site
#     matches!(ty, syn::Expr::Lit(_))       <- one site
#
# Watched enums: syn::Type, syn::Expr. Test code is excluded.
#
# Every one of these is a place that independently decides what captured Rust
# MEANS. #211 says only `core::flat` may do that, so this file freezes the
# population: any change fails `cargo test -p prebindgen boundary_ledger`.
#
# This is the CLASSIFY half of the rule, and only that half. Spelling the source
# — handing an element's `syntax` slice to `quote!` — names no variant and is
# not counted, because re-emitting what the source wrote is what generated Rust
# is for. Deciding what the source MEANT from that syntax is what it is not for.
#
# To change it deliberately:
#
#     UPDATE_BOUNDARY_LEDGER=1 cargo test -p prebindgen boundary_ledger
#     git diff prebindgen/src/api/core/flat/boundary.ledger
#
# A count going DOWN is the goal (a classifier reads elements instead) and is
# still a ledger edit, so the win shows up in the diff.
#
# ── THE ESCAPES ───────────────────────────────────────────────────────────
#
# The model's syntax is sealed: `Origin::syntax` is private, `spell()` hands out
# tokens, and `as_syn()` is the ONE way to a `syn` node. So the two sections
# below count what the first section can only measure — every place that still
# has a node to take apart.
#
#   ## escapes: types   `as_syn()`, `stripped_syntax()`, `to_syn()`,
#                       `type_from_ident()`   — ZERO, and must STAY zero: a new
#                       row is a consumer taking a node the model can answer
#                       for, and the fix is to ask the model
#   ## escapes: items   `f.origin.as_syn()`, `enum_item()`
#                                             — persists, and is ACCOUNTED:
#                       an emitter re-stating a whole item verbatim, nothing
#                       else
#
# A type escape is a source type the model should have been able to answer for.
# There are none: every consumer outside `core::flat` reads `kind`, spells
# `spell()`, or looks up a `TypeKey`.
#
# An item escape is a captured item's own node, which an emitter re-stating a
# whole item verbatim legitimately needs — and that is the ONLY reason it is
# legitimate. Taking an item to read a FACT off it (a name, a type, a signature)
# is a missing accessor. `ITEM_FLOOR` says per file which of the two it is, and
# `the_item_floor_is_accounted_for` fails on an unaccounted file or a stale
# entry, exactly as `FLOOR` does below. Two `registry/scan.rs` sites sat in this
# count for forty-five stages reading a signature and a const's type: the header
# called the whole population `expected to persist` and nothing checked which
# sites had earned it (S46).
#
# So the classification sites above are a DIFFERENT POPULATION — one this
# transition never had to remove, because it was never the model's to answer
# for. That claim is CHECKED rather than asserted: `FLOOR` in `boundary.rs`
# names, per file, which of four things its sites read — a WIRE the adapter
# composed, a build-script DECLARATION, a GENERATED SIGNATURE, or the
# array-length whitelist — and `the_classification_floor_is_accounted_for` fails
# if a file classifies with no entry, or holds an entry and no longer
# classifies. A new classifier has to say which population it joins, in the
# commit that adds it; if it is none of them, it is reading source syntax.
#
# (S8 claimed `api/core` held zero type escapes and it held seven, for
# twenty-five stages, because nothing checked it. An unchecked claim drifts —
# which is also why THIS text lives in `boundary.rs`'s `HEADER`, the file the
# ledger is generated from. Editing the generated `.ledger` instead is undone by
# the next `UPDATE_BOUNDARY_LEDGER=1`.)
#
# The scan counts FIVE doors — `as_syn`, `stripped_syntax`, `to_syn`,
# `enum_item`, `type_from_ident` — each by NAME rather than by call shape, so UFCS
# (`TypeRef::as_syn(&ty)`) and a function item (`let read = TypeRef::as_syn;`)
# are counted like a method call. A shape-matched rule missed both, which is a
# ratchet that can be stepped around. `escape_surface_is_closed` keeps the list
# honest: it reads the model's own surface and fails if a public method hands
# out a non-leaf syn node under a name the scan does not count — which is how
# four of the five doors were found. It asks the question the safe way round:
# every syn return is a door unless the type is on a small LEAF allowlist, and
# unless the function is one of three NAMED transformers that take a node and
# nothing else. Both allowlists are the exception side, where being wrong is a
# false alarm rather than a silent hole.
#
# It reads more than functions, because more than a function can hand out a
# node: a public field, a trait method and its impl, and an alias that hides the
# name (`use syn::Type as SynType` is resolved; a crate rename, a local
# `type Node = syn::Type` and an associated `type Target = syn::Type` are
# reported, since flat names syn types in full). An alias is reported whatever
# its own visibility is: an alias is transparent, so a private one still lets a
# public signature hand out the node.
#
# The bucket is read off the NAME, then the RECEIVER: `enum_item` hands out an
# item; for `as_syn`, a receiver of `origin` (or an `Origin::` qualifier) means
# the item's node, anything else means a type. Two over-counts land in the type bucket on purpose, both in
# the safe direction — adapter declarations reuse `Origin` for a placeless
# location (`decl.rust_type`), and carrying a spelling into an adapter-owned
# `syn::Type` field is not classification either. Both are follow-ups the count
# names rather than hides.
#
# A NODE CACHE is why the type count can go UP for a good change. A field typed
# `syn::Type` holds a node permanently: it costs ONE escape where it is filled
# and none at all where it is read, so N consumers read a node for free and the
# census sees 1. Replacing that field with a `TypeRef` removes the one and
# reveals the N — the reads were always there. A rising count after a field is
# de-cached is the measure becoming true, not the boundary getting worse; a
# rising count with no field removed is the thing this ledger exists to catch.
#
# KNOWN BLIND SPOTS — classification neither half sees:
#
#   * token-string classification, e.g. `core/domain.rs` matching
#     `ty.to_token_stream().to_string()` against \"i8\" / \"f32\". The seal does
#     NOT close this and cannot: what `domain.rs` reads is a `syn::Type` a build
#     script supplied, which never was the model's. `spell().to_string()` reaches
#     a string too — one call further, and greppable, which is the whole gain;
#   * ident-name classification, e.g. `seg.ident == \"Option\"` — now gated, in
#     that a path has to come from an escape first;
#   * helper delegation — `jnigen/jni/classify.rs` is a whole classifier with
#     zero watched sites; gated the same way;
#   * syn enums outside WATCHED: Item, Fields, FnArg, ReturnType,
#     GenericArgument, Pat, ... — likewise reachable only through an escape.
#
# One listed gap is closed rather than still open: `types_util::match_pattern`
# unified against `parse_quote!(_)` patterns, adding shape rules with no watched
# site. It is deleted — the wildcard tables it served held a single entry,
# `Result<_, _>`, which the model already names `TypeKind::Fallible`.
#
# A check that silently under-reports is worse than no check, which is why the
# gaps are listed here rather than implied away.
";

/// Every classification site in the crate's own sources, keyed by path relative
/// to `src/` with `/` separators so the ledger is identical on every platform.
///
/// Excluded: this module's own directory (the language is where classification
/// is *supposed* to live), and test code — `tests.rs`, anything under a `tests/`
/// directory, and any item carrying `#[cfg(test)]`.
fn scan_tree(src_root: &Path) -> BTreeMap<String, Counts> {
    let mut out = BTreeMap::new();
    let mut stack = vec![src_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).expect("src/ is readable");
        for entry in entries {
            let path = entry.expect("readable dir entry").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "tests") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs")
                || path.file_name().is_some_and(|n| n == "tests.rs")
            {
                continue;
            }
            let rel = rel_key(src_root, &path);
            if rel.starts_with("api/core/flat/") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("source file is UTF-8");
            let n = scan_file(&text);
            if !n.is_empty() {
                out.insert(rel, n);
            }
        }
    }
    out
}

fn rel_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("path came from walking root")
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Sites in one file's text.
fn scan_file(text: &str) -> Counts {
    let stream = TokenStream::from_str(text).expect("source file parses as tokens");
    let aliases = collect_aliases(stream.clone());
    let mut n = Counts::default();
    count(stream, &aliases, &mut n);
    n
}

/// Idents bound to a watched enum by a `use` in this file — `use syn::Type` or
/// `use syn::Type as T`.
///
/// Without this the check is defeated by a one-line import, and CI's
/// `imports_granularity=Crate` makes such an import more likely over time, not
/// less. No file does this today; the point is that none can start.
fn collect_aliases(stream: TokenStream) -> Vec<String> {
    syn_imports(stream)
        .into_iter()
        .filter(|(_, original)| WATCHED.contains(&original.as_str()))
        .map(|(bound, _)| bound)
        .collect()
}

/// Every syn **type** a file imports, as `(bound name, original name)` —
/// `use syn::{Expr, Type as T}` yields `[("Expr", "Expr"), ("T", "Type")]`.
///
/// One walk for both halves of the check. The classification scan wants the
/// names bound to a [`WATCHED`] enum; the surface guard wants the names bound to
/// anything that is not a [`LEAF`], because `use syn::Type as SynType` makes
/// `-> &SynType` a door that a literal `syn::` match cannot see (#313 review).
///
/// A run is flattened, so `use syn::{punctuated::Punctuated, Type}` reads as
/// `["syn", "punctuated", "Punctuated", "Type"]` and a module segment is
/// indistinguishable from an item — except by case, which in Rust is not a
/// coincidence: modules are `snake_case`, types are `CamelCase`. An import that
/// defeats that convention defeats this, and would be the only one in the tree.
fn syn_imports(stream: TokenStream) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for run in use_runs(stream) {
        // `use syn::...` only; `use crate::Type` binds something else entirely.
        if run.first().map(String::as_str) != Some("syn") {
            continue;
        }
        let mut i = 1;
        while i < run.len() {
            // `use syn as syntax;` renames the crate, so `syntax::Type` is a
            // node under a name no uppercase check finds. Reported rather than
            // resolved — see `RENAMED_SYN`.
            if run[i] == "as" {
                if let Some(alias) = run.get(i + 1) {
                    out.push((alias.clone(), RENAMED_SYN.to_string()));
                }
                i += 2;
                continue;
            }
            let is_type = run[i].starts_with(char::is_uppercase);
            if is_type {
                let renamed = run.get(i + 1).map(String::as_str) == Some("as");
                match (renamed, run.get(i + 2)) {
                    (true, Some(alias)) => {
                        out.push((alias.clone(), run[i].clone()));
                        i += 3;
                        continue;
                    }
                    _ => out.push((run[i].clone(), run[i].clone())),
                }
            }
            i += 1;
        }
    }
    out
}

/// The idents of every `use` statement, flattened through brace groups so
/// `use syn::{Type, Expr as E}` reads as one run.
fn use_runs(stream: TokenStream) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let toks: Vec<TokenTree> = stream.into_iter().collect();
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            TokenTree::Ident(id) if *id == "use" => {
                let mut run = Vec::new();
                i += 1;
                while i < toks.len() && !is_punct(&toks[i], ';') {
                    flatten_idents(&toks[i], &mut run);
                    i += 1;
                }
                out.push(run);
            }
            // A `use` can be nested in a module body or a function.
            TokenTree::Group(g) => {
                out.extend(use_runs(g.stream()));
                i += 1;
            }
            _ => i += 1,
        }
    }
    out
}

fn flatten_idents(tt: &TokenTree, out: &mut Vec<String>) {
    match tt {
        TokenTree::Ident(id) => out.push(id.to_string()),
        TokenTree::Group(g) => {
            for inner in g.stream() {
                flatten_idents(&inner, out);
            }
        }
        _ => {}
    }
}

/// What one file owes, in the three populations this ledger tracks.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct Counts {
    /// Variant mentions of a [`WATCHED`] syn enum — the original measure.
    classify: usize,
    /// `as_syn()` on a type spelling.
    escape_type: usize,
    /// `as_syn()` on a captured item's origin.
    escape_item: usize,
}

impl Counts {
    fn is_empty(self) -> bool {
        self == Self::default()
    }
}

/// Every route from the model to a structural `syn` node.
///
/// `as_syn` is the door the seal opened deliberately; the other three predate it
/// and hand out a node just as completely, so the census covers them or it is
/// not a census. [`escape_surface_is_closed`] is what stops a fifth from
/// appearing quietly.
///
/// | | hands out | why it exists |
/// |---|---|---|
/// | `as_syn` | the node itself | the escape |
/// | `stripped_syntax` | `syn::Type` | the spelling under a transparent wrapper |
/// | `to_syn` | `syn::Type` | the round-trip that checks the lowering |
/// | `enum_item` | `&syn::ItemEnum` | a declared enum's own item |
/// | `type_from_ident` | `syn::Type` | a spelling built from a name |
const ESCAPES: &[&str] = &[
    "as_syn",
    "stripped_syntax",
    "to_syn",
    "enum_item",
    "type_from_ident",
];

/// The syn types that are **leaves**: a name, a lifetime, a field address.
/// There is no shape to match on, so handing one out is not a door.
///
/// An allowlist, and deliberately the small side of the question. It began as
/// the opposite — a list of the structural types that *are* doors — and that is
/// a list of what someone thought of: a method returning `&[syn::Attribute]`
/// was invisible because `Attribute` was not on it (#313 review). Inverted, a
/// syn type nobody considered is a door until someone argues it is a leaf, and
/// that argument is a diff on this line.
const LEAF: &[&str] = &["Ident", "Lifetime", "Member", "Index"];

/// The functions that hand back a node they were **given**, so the caller could
/// have reached it without them.
///
/// An allowlist because a structural test cannot prove provenance: a function
/// can take a node *and* the model, and return the model's. Three names, each
/// still checked for the shape that makes the claim true — see
/// [`collect_doors`]. Adding a fourth is a deliberate line in this file.
const TRANSFORMERS: &[&str] = &[
    "canonical_type",
    "extract_fn_trait_args",
    "peel_transparent",
];

/// The "original" recorded for `use syn as <name>` — a rename of the crate
/// itself, which puts every node behind a path no uppercase check finds.
///
/// **Rejected rather than resolved**, and so is a local `type Node = syn::Type`.
/// Resolving either means following aliases to a fixed point, for a spelling
/// `flat` has no reason to write: the model names `syn::` types in full,
/// everywhere, today. If that ever has to change, this is the line that says so.
const RENAMED_SYN: &str = "<syn itself>";

/// Which bucket an escape belongs to — by name, then by **receiver**.
///
/// Outside `flat/`, a receiver named `origin` is always a captured item's node —
/// an `ItemFn`, an `ItemStruct`, a `Field`, a `Variant` — because `TypeRef`'s own
/// origin is private and a type spelling is therefore reached as `ty.as_syn()`.
/// Everything else is a type.
///
/// **A known over-count, in the safe direction.** Adapter *declarations* reuse
/// `Origin` to carry a placeless location (`decl.rust_type`, `declared_ty`), and
/// those escapes land in the type bucket though they read what a build script
/// wrote rather than what a source crate did. The bucket that must reach zero is
/// the one that over-counts; the real fix is that a declaration is not an
/// `Origin`, which is a refactor and not a rule.
fn escape_bucket(name: &str, qualifier: Option<&str>) -> fn(&mut Counts) -> &mut usize {
    let item: fn(&mut Counts) -> &mut usize = |c| &mut c.escape_item;
    let ty: fn(&mut Counts) -> &mut usize = |c| &mut c.escape_type;
    match (name, qualifier) {
        // Hands out a whole captured item, whatever the receiver is called.
        ("enum_item", _) => item,
        // `f.origin.as_syn()` — a method call on an item's origin — and
        // `Origin::as_syn(&f.origin)`, the same thing spelled through UFCS.
        ("as_syn", Some("origin" | "Origin")) => item,
        _ => ty,
    }
}

/// What stands before the escape's name: the receiver of `recv.as_syn()`, or the
/// path qualifier of `TypeRef::as_syn(..)`. `None` for a bare mention.
fn escape_qualifier(toks: &[TokenTree], i: usize) -> Option<String> {
    // `recv . as_syn`
    if i >= 2 && is_punct(&toks[i - 1], '.') {
        if let Some(TokenTree::Ident(id)) = toks.get(i - 2) {
            return Some(id.to_string());
        }
    }
    // `Qualifier :: as_syn` — a `::` is two `Punct` tokens.
    if i >= 3 && is_sep(toks, i - 2) {
        if let Some(TokenTree::Ident(id)) = toks.get(i - 3) {
            return Some(id.to_string());
        }
    }
    None
}

fn count(stream: TokenStream, aliases: &[String], n: &mut Counts) {
    let toks: Vec<TokenTree> = stream.into_iter().collect();
    let mut i = 0;
    while i < toks.len() {
        // `#[cfg(test)] <item>` — skip the item wholesale. Five inline test
        // modules exist and one is named `replace_ident_tests`, so a rule keyed
        // on the module name would miss it.
        if is_punct(&toks[i], '#') {
            if let Some(TokenTree::Group(g)) = toks.get(i + 1) {
                if g.delimiter() == Delimiter::Bracket && is_cfg_test(g.stream()) {
                    i += 2;
                    skip_item(&toks, &mut i);
                    continue;
                }
            }
            i += 1;
            continue;
        }
        // An import is not a classifier, and counting one would churn the ledger
        // whenever rustfmt regroups imports.
        if matches!(&toks[i], TokenTree::Ident(id) if *id == "use") {
            while i < toks.len() && !is_punct(&toks[i], ';') {
                i += 1;
            }
            i += 1;
            continue;
        }
        // `syn :: <watched> :: <variant>`
        if matches!(&toks[i], TokenTree::Ident(id) if *id == "syn")
            && is_sep(&toks, i + 1)
            && matches!(toks.get(i + 3), Some(TokenTree::Ident(id)) if WATCHED.contains(&id.to_string().as_str()))
            && is_sep(&toks, i + 4)
            && matches!(toks.get(i + 6), Some(TokenTree::Ident(_)))
        {
            n.classify += 1;
            i += 7;
            continue;
        }
        // `<alias> :: <variant>`
        if matches!(&toks[i], TokenTree::Ident(id) if aliases.contains(&id.to_string()))
            && is_sep(&toks, i + 1)
            && matches!(toks.get(i + 3), Some(TokenTree::Ident(_)))
        {
            n.classify += 1;
            i += 4;
            continue;
        }
        // The escape — **every mention of the name**, not every call.
        //
        // A shape-matched rule (`<recv> . as_syn ( )`) is the census that gets
        // bypassed: `TypeRef::as_syn(&ty)` reaches the same node through UFCS,
        // and `let read = TypeRef::as_syn;` reaches it through a function item
        // that is never *called* at the escape's own name at all. Both would
        // have added a source-syntax escape with no ledger drift. Counting the
        // ident covers call, UFCS and reference alike, because none of them can
        // reach the node without writing the name.
        if let TokenTree::Ident(id) = &toks[i] {
            let name = id.to_string();
            // ...except a definition, which is what is being counted.
            let is_def =
                i > 0 && matches!(toks.get(i - 1), Some(TokenTree::Ident(kw)) if *kw == "fn");
            if ESCAPES.contains(&name.as_str()) && !is_def {
                *escape_bucket(&name, escape_qualifier(&toks, i).as_deref())(n) += 1;
                i += 1;
                continue;
            }
        }
        if let TokenTree::Group(g) = &toks[i] {
            count(g.stream(), aliases, n);
        }
        i += 1;
    }
}

/// Consume the item an attribute was attached to: everything up to and including
/// its first brace-delimited body, or its terminating `;`, whichever comes first.
/// That covers `mod x { .. }`, `mod x;`, `fn f() -> T { .. }`, `use ..;` and
/// `impl T { .. }` alike, and steps over any further attributes on the way.
fn skip_item(toks: &[TokenTree], i: &mut usize) {
    while *i < toks.len() {
        let tt = &toks[*i];
        *i += 1;
        match tt {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => return,
            _ if is_punct(tt, ';') => return,
            _ => {}
        }
    }
}

/// True only for the exact predicate `cfg(test)`.
///
/// Deliberately conservative: this check's job is to stop a classifier hiding, so
/// anything it cannot *prove* is test-only gets counted. Looking for the ident
/// `test` anywhere in the predicate got that backwards — `cfg(not(test))` and
/// `cfg(any(test, feature = "x"))` both compile in a production build, and both
/// were treated as test-only, so a classifier under either evaded the ledger
/// entirely.
///
/// `cfg(all(test, …))` is genuinely test-only and is nonetheless counted. That is
/// the safe direction to err in, and nothing in the tree writes one; if that
/// changes, widening this is a deliberate edit with a ledger diff attached.
fn is_cfg_test(stream: TokenStream) -> bool {
    let mut idents = Vec::new();
    for tt in stream {
        flatten_idents(&tt, &mut idents);
    }
    idents == ["cfg", "test"]
}

/// A `::` is two `Punct` tokens, so a path separator spans `i` and `i + 1`.
fn is_sep(toks: &[TokenTree], i: usize) -> bool {
    toks.get(i).is_some_and(|t| is_punct(t, ':'))
        && toks.get(i + 1).is_some_and(|t| is_punct(t, ':'))
}

fn is_punct(tt: &TokenTree, c: char) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == c)
}

/// The three populations, in the order the ledger writes them. A `##` line
/// switches section; a `#` line is a comment, as before.
type Section = (&'static str, fn(&Counts) -> usize);

const SECTIONS: &[Section] = &[
    ("classification sites", |c| c.classify),
    ("escapes: types", |c| c.escape_type),
    ("escapes: items", |c| c.escape_item),
];

fn render(sites: &BTreeMap<String, Counts>) -> String {
    let mut s = String::from(HEADER);
    for (i, (name, get)) in SECTIONS.iter().enumerate() {
        s.push('\n');
        // The first section is the original ledger, and its lines are written
        // exactly as before so the two added populations read as an addition
        // rather than a rewrite.
        if i > 0 {
            s.push_str(&format!("## {name}\n"));
        }
        let mut total = 0;
        for (path, counts) in sites {
            let n = get(counts);
            if n > 0 {
                s.push_str(&format!("{n}\t{path}\n"));
                total += n;
            }
        }
        s.push_str(&format!("\n# total {name}: {total}\n"));
    }
    s
}

fn parse(text: &str) -> BTreeMap<String, Counts> {
    let mut out: BTreeMap<String, Counts> = BTreeMap::new();
    let mut section = 0usize;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            section = SECTIONS
                .iter()
                .position(|(n, _)| *n == name.trim())
                .expect("ledger section is one this scanner writes");
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let (n, path) = line
            .split_once('\t')
            .expect("ledger line is `<count>\\t<path>`");
        let n: usize = n.parse().expect("ledger count is a number");
        let counts = out.entry(path.to_string()).or_default();
        match section {
            0 => counts.classify = n,
            1 => counts.escape_type = n,
            _ => counts.escape_item = n,
        }
    }
    out
}

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Why each file still has classification sites, now that none of them read
/// **source** syntax.
///
/// `escapes: types` reached zero at S38, so nothing outside `core::flat` can
/// reach a source type node at all. What the first census still counts is
/// therefore something else, and this says which of four things it is per file.
/// All four are legitimate — none was ever the model's to answer for — but a
/// claim the ledger does not check is a claim that drifts, which is how S8's
/// "`api/core` is zero" survived twenty-five stages while being false.
///
/// So the floor is DATA, and [`the_classification_floor_is_accounted_for`]
/// fails when a file has sites and no entry here. Adding a classifier means
/// choosing which population it joins, in the same commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Floor {
    /// A **wire**: the type an adapter chose for the boundary — `jni::sys::jlong`,
    /// `*mut t_t`, `JObject`. It is composed by the adapter, is not a Rust type
    /// any source crate wrote, and the model never classified it. Asking
    /// `matches!(wire, syn::Type::Ptr(_))` is asking about the adapter's own
    /// output.
    Wire,
    /// A type a **build script** wrote in a declaration — `const_expr!(…)`'s
    /// value type, a `.callback(…)` argument, `LocalField::Local`'s signature.
    /// #280 seals minting to the model, so there is no reading to ask: the
    /// declaration is the only thing that can say.
    Declaration,
    /// A **generated converter's own signature** — `entry.function.sig.output`.
    /// This adapter built that `fn` a moment ago; reading its return is reading
    /// its own note to itself.
    GeneratedSignature,
    /// The **acceptance whitelist** for an array length, narrowed at S29 to the
    /// two `syn::Expr` forms `flat::lower_array_len` accepts. It guards a
    /// path-qualifying rewrite, so it must name expression forms.
    AcceptedLength,
}

/// The account. Every file with a nonzero classification count appears here.
const FLOOR: &[(&str, Floor)] = &[
    ("api/core/registry/scan.rs", Floor::Declaration),
    ("api/core/types_util.rs", Floor::Declaration),
    ("api/lang/cbindgen/builder.rs", Floor::Declaration),
    ("api/lang/cbindgen/emit.rs", Floor::Wire),
    ("api/lang/cbindgen/mod.rs", Floor::Wire),
    ("api/lang/cbindgen/trait_impl.rs", Floor::Wire),
    ("api/lang/jnigen/jni/builder.rs", Floor::Declaration),
    ("api/lang/jnigen/jni/emit/convert.rs", Floor::Wire),
    ("api/lang/jnigen/jni/emit/flat_input.rs", Floor::Wire),
    ("api/lang/jnigen/jni/emit/names.rs", Floor::AcceptedLength),
    ("api/lang/jnigen/jni/emit/wrapper.rs", Floor::Wire),
    ("api/lang/jnigen/jni/fold.rs", Floor::Wire),
    ("api/lang/jnigen/jni/prim.rs", Floor::Wire),
    ("api/lang/jnigen/jni/render.rs", Floor::Wire),
    ("api/lang/jnigen/jni/trait_impl.rs", Floor::Wire),
    ("api/lang/jnigen/jni/wire_access.rs", Floor::Wire),
    ("api/lang/jnigen/util.rs", Floor::GeneratedSignature),
];

/// Why each file still takes a captured item's own node.
///
/// The header has called this population "expected to persist" from the start,
/// and that was right about the shape and silent about which sites qualify —
/// which is how two `registry/scan.rs` sites sat here for forty-five stages
/// reading a *fact off* an item rather than re-stating one (S46).
///
/// So it is accounted the same way [`FLOOR`] accounts the other census, and
/// [`the_item_floor_is_accounted_for`] enforces it. There is exactly one
/// legitimate reason, which is why this is a list and not an enum: an emitter
/// re-states a **whole captured item verbatim**, tokens and all. Anything that
/// takes an item to read one field off it is a missing accessor, and belongs in
/// the model instead.
const ITEM_FLOOR: &[(&str, &str)] = &[
    (
        "api/core/prebindgen.rs",
        "`const_path_alias` re-emits a captured `const` as an alias — attrs,          vis, ident and type spliced verbatim",
    ),
    (
        "api/core/write.rs",
        "a `Guard`'s `const _` is spliced into the generated file as written",
    ),
    (
        "api/lang/cbindgen/trait_impl.rs",
        "the C mirror re-states an `EnumValue`'s discriminant AS WRITTEN          (`= 0x07` stays `0x07`) — the one consumer `EnumValue::origin` exists          for, and its own doc says so",
    ),
    (
        "api/lang/jnigen/jni/trait_impl.rs",
        "`const_path_alias` again, on the JNI side",
    ),
];

/// Every file that still takes an item node is accounted for in [`ITEM_FLOOR`],
/// and every entry there still takes one.
///
/// Same two directions as [`the_classification_floor_is_accounted_for`], for
/// the same reason: a stale entry would let a new item escape land in that file
/// and inherit an account written for something else.
#[test]
fn the_item_floor_is_accounted_for() {
    let found = scan_tree(&src_root());
    let accounted: std::collections::BTreeSet<&str> =
        ITEM_FLOOR.iter().map(|(path, _)| *path).collect();

    let unaccounted: Vec<&String> = found
        .iter()
        .filter(|(path, c)| c.escape_item > 0 && !accounted.contains(path.as_str()))
        .map(|(path, _)| path)
        .collect();
    let stale: Vec<&&str> = accounted
        .iter()
        .filter(|p| found.get(**p).is_none_or(|c| c.escape_item == 0))
        .collect();

    assert!(
        unaccounted.is_empty() && stale.is_empty(),
        "ITEM FLOOR DRIFT\n\
         \x20 unaccounted (takes an item node, no `ITEM_FLOOR` entry): {unaccounted:?}\n\
         \x20 stale (`ITEM_FLOOR` entry, no longer takes one): {stale:?}\n\n\
         An item escape is legitimate for ONE reason: an emitter re-stating a \
         whole captured item verbatim, tokens and all. Taking an item to read a \
         FACT off it — a name, a type, a signature — is a missing accessor, and \
         the model is where it belongs. If this is the verbatim case, say so in \
         `ITEM_FLOOR` in this commit.\n"
    );
}

/// Every file that still classifies is accounted for in [`FLOOR`], and every
/// entry there still classifies.
///
/// The second half matters as much as the first: a stale entry is a file whose
/// sites were removed, and leaving it would let a NEW classifier land in that
/// file silently inheriting an account written for something else.
#[test]
fn the_classification_floor_is_accounted_for() {
    let found = scan_tree(&src_root());
    let accounted: std::collections::BTreeMap<&str, Floor> = FLOOR.iter().copied().collect();

    let unaccounted: Vec<&String> = found
        .iter()
        .filter(|(path, c)| c.classify > 0 && !accounted.contains_key(path.as_str()))
        .map(|(path, _)| path)
        .collect();
    let stale: Vec<&&str> = accounted
        .keys()
        .filter(|p| found.get(**p).is_none_or(|c| c.classify == 0))
        .collect();

    assert!(
        unaccounted.is_empty() && stale.is_empty(),
        "CLASSIFICATION FLOOR DRIFT\n\
         \x20 unaccounted (classifies, no `FLOOR` entry): {unaccounted:?}\n\
         \x20 stale (`FLOOR` entry, no longer classifies): {stale:?}\n\n\
         `escapes: types` is zero, so a classification site outside `core::flat` \
         reads a WIRE, a build-script DECLARATION, a GENERATED SIGNATURE, or the \
         array-length whitelist — never source syntax. Say which in `FLOOR`, in \
         this commit; if it is none of them, it is reading source syntax and the \
         fix is to ask the model instead.\n"
    );
}

#[test]
fn boundary_ledger() {
    let root = src_root();
    let found = scan_tree(&root);
    let ledger_path = root.join(LEDGER);

    if std::env::var_os("UPDATE_BOUNDARY_LEDGER").is_some() {
        fs::write(&ledger_path, render(&found)).expect("ledger is writable");
        return;
    }

    let committed = parse(&fs::read_to_string(&ledger_path).expect("ledger is committed"));
    if committed == found {
        return;
    }

    let mut drift = String::new();
    let paths: std::collections::BTreeSet<_> = committed.keys().chain(found.keys()).collect();
    for path in paths {
        let (was, now) = (committed.get(path), found.get(path));
        if was == now {
            continue;
        }
        for (name, get) in SECTIONS {
            let fmt = |v: Option<&Counts>| v.map_or(0, get);
            let (was, now) = (fmt(was), fmt(now));
            if was != now {
                drift.push_str(&format!("  {path} [{name}]: {was} -> {now}\n"));
            }
        }
    }
    panic!(
        "BOUNDARY LEDGER DRIFT — source-syntax classification sites changed:\n\
         {drift}\n\
         A new classifier outside core::flat needs one of:\n\
         \x20 * move it into core::flat (see #211), or\n\
         \x20 * regenerate and justify the change in review:\n\
         \x20     UPDATE_BOUNDARY_LEDGER=1 cargo test -p prebindgen boundary_ledger\n\
         \x20     git diff prebindgen/src/{LEDGER}\n"
    );
}

#[test]
fn scanner_recognizes_the_shapes_that_matter() {
    // A match arm, and a `matches!` body — invisible to an AST visitor, which is
    // why this walks tokens.
    assert_eq!(
        scan_file("fn f(t: &syn::Type) { match t { syn::Type::Slice(s) => g(s), _ => {} } }")
            .classify,
        1
    );
    assert_eq!(
        scan_file("fn f() -> bool { matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()) }")
            .classify,
        1
    );
    assert_eq!(
        scan_file("fn f() { let syn::Expr::Lit(l) = e else { return; }; }").classify,
        1
    );
    // Two on one line: a line count would report one.
    assert_eq!(
        scan_file("fn f() { if let (syn::Type::Path(a), syn::Type::Path(b)) = p {} }").classify,
        2
    );

    // The one-line defeat the alias handling closes.
    assert_eq!(
        scan_file("use syn::Type;\nfn f() { if let Type::Reference(r) = t {} }").classify,
        1
    );
    assert_eq!(
        scan_file(
            "use syn::{Expr, Type as T};\nfn f() { if let T::Ptr(p) = t { h(Expr::Lit(l)) } }"
        )
        .classify,
        2
    );
    // An import is not a site, and neither is a non-syn `Type`.
    assert_eq!(scan_file("use syn::Type;\nfn f() {}").classify, 0);
    assert_eq!(
        scan_file("fn f() { if let Type::Reference(r) = t {} }").classify,
        0
    );

    // Outside WATCHED — emitters construct these constantly.
    assert_eq!(
        scan_file("fn f() { let i = syn::Item::Fn(f); }").classify,
        0
    );

    // Test code does not count, whatever the module is called.
    assert_eq!(
        scan_file(
            "fn f() { match t { syn::Type::Slice(s) => (), _ => () } }\n\
             #[cfg(test)]\n\
             mod replace_ident_tests { fn g() { let _ = syn::Type::Ptr(p); } }"
        )
        .classify,
        1
    );
    assert_eq!(scan_file("#[cfg(test)]\nmod tests;").classify, 0);

    // But ONLY code proven test-only. Both of these compile in a production
    // build, and a rule that looked for the ident `test` anywhere let a
    // classifier under either evade the count.
    assert_eq!(
        scan_file("#[cfg(not(test))]\nfn g() { let _ = syn::Type::Ptr(p); }").classify,
        1,
        "cfg(not(test)) is production code"
    );
    assert_eq!(
        scan_file("#[cfg(any(test, feature = \"x\"))]\nfn g() { let _ = syn::Type::Ptr(p); }")
            .classify,
        1,
        "cfg(any(test, ..)) compiles whenever the other arm holds"
    );
    // A feature literally named "test" is not the test predicate either.
    assert_eq!(
        scan_file("#[cfg(feature = \"test\")]\nfn g() { let _ = syn::Type::Ptr(p); }").classify,
        1
    );
}

/// **The census is only a census if it covers every door.**
///
/// The escape scan counts [`ESCAPES`] by name. That is a list, and a list is a
/// thing someone forgets to add to — so this reads the model's own surface and
/// fails if anything externally visible in `core/flat` hands out a non-[`LEAF`]
/// `syn` node under a name the scan does not count.
///
/// **Externally visible is not just a function** (#313 review). A public field
/// reopens exactly the capability an accessor closes; a trait method is as
/// visible as its trait, and its impl carries no visibility of its own; and an
/// alias — `use syn as syntax`, `type Node = syn::Type`, or an associated
/// `type Target = syn::Type` reached through `Self::` — puts a node behind a
/// name no check of `syn::` paths can follow. All are doors here.
///
/// The alias forms are **reported rather than resolved**: `flat` names `syn::`
/// types in full, everywhere, so following aliases to a fixed point would be
/// machinery for a spelling the model does not use. And an alias is reported
/// whatever its own visibility is, because an alias is transparent — a private
/// `type Node = syn::Type` still lets a public signature hand out the node, and
/// the caller never has to name `Node` to use it.
///
/// Three of the four entries were found this way rather than by design:
/// `stripped_syntax`, `to_syn` and `enum_item` predate the seal and hand out a
/// node just as completely as `as_syn` does. The review that asked for UFCS
/// coverage is the same question one level up — a spelling the check does not
/// know about is not counted, whether it is a call syntax or a whole method.
#[test]
fn escape_surface_is_closed() {
    let mut doors = Vec::new();
    let mut stack = vec![src_root().join("api/core/flat")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("flat/ is readable") {
            let path = entry.expect("readable dir entry").path();
            if path.is_dir() {
                // Test code is not the model's surface, and the same exclusion
                // the ledger scan makes: a `pub type` inside a `parse_quote!`
                // fixture is test *data*, not a declaration.
                if path.file_name().is_some_and(|n| n == "tests") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs")
                || path.file_name().is_some_and(|n| n == "tests.rs")
            {
                continue;
            }
            let text = fs::read_to_string(&path).expect("source file is UTF-8");
            let file: syn::File = syn::parse_str(&text).expect("flat source parses");
            let aliases: Vec<String> = syn_imports(TokenStream::from_str(&text).expect("tokens"))
                .into_iter()
                .filter(|(_, original)| !LEAF.contains(&original.as_str()))
                .map(|(bound, _)| bound)
                .collect();
            collect_doors(
                &file.items,
                &rel_key(&src_root(), &path),
                &aliases,
                &mut doors,
            );
        }
    }
    let uncounted: Vec<_> = doors
        .iter()
        .filter(|(name, _)| !ESCAPES.contains(&name.as_str()))
        .collect();
    assert!(
        uncounted.is_empty(),
        "these hand a structural syn node out of the model under a name the \
         escape scan does not count, so a consumer can take the source apart \
         with no ledger drift:\n{}\n\nEither add the name to ESCAPES (and \
         regenerate the ledger, so the doors it opens are counted), or return \
         tokens instead.",
        uncounted
            .iter()
            .map(|(name, at)| format!("  {at}: {name}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Public functions that hand out a non-[`LEAF`] `syn` node.
///
/// **A door is the default; being a transformer is what earns the exemption.**
/// The rule was a `self` receiver once, on the reasoning that a function without
/// one holds no model state — and an associated function does not need a
/// receiver to be handed the model (#313 review):
///
/// ```ignore
/// impl TypeRef {
///     pub fn leak(this: &Self) -> &syn::Type { this.as_syn() }
/// }
/// ```
///
/// `TypeRef::leak(&ty)` hands out the held node, and a rule keyed on the
/// receiver waves it through. So the exemption is stated as what it actually
/// is: a function is exempt only when it **already receives** a non-leaf syn
/// node and takes no receiver — then everything it hands back, the caller could
/// reach without it. That is `canonical_type`, `extract_fn_trait_args` and
/// `peel_transparent`, and nothing else.
///
/// Visibility still applies: `pub(super)` cannot be called from outside the
/// model, so it is the model reading itself.
fn collect_doors(
    items: &[syn::Item],
    at: &str,
    aliases: &[String],
    out: &mut Vec<(String, String)>,
) {
    fn escapes_flat(vis: &syn::Visibility) -> bool {
        match vis {
            syn::Visibility::Public(_) => true,
            syn::Visibility::Restricted(r) => !r.path.is_ident("super") && !r.path.is_ident("self"),
            syn::Visibility::Inherited => false,
        }
    }
    fn hands_out_a_node(ret: &syn::ReturnType, aliases: &[String]) -> bool {
        let syn::ReturnType::Type(_, ty) = ret else {
            return false;
        };
        names_a_node(quote::ToTokens::to_token_stream(ty), aliases)
    }
    /// A non-[`LEAF`] `syn::` type named anywhere in a signature position.
    ///
    /// Qualified, because `-> Option<&Type>` is *flat's own* `Type` — the model,
    /// not a node. Through groups, because `Option<(&'static str, syn::Type)>`
    /// nests the mention inside a parenthesised token group.
    fn names_a_node(stream: TokenStream, aliases: &[String]) -> bool {
        let toks: Vec<TokenTree> = stream.into_iter().collect();
        (0..toks.len()).any(|i| {
            (matches!(&toks[i], TokenTree::Ident(id) if id == "syn")
                && is_sep(&toks, i + 1)
                && matches!(toks.get(i + 3), Some(TokenTree::Ident(id))
                    if !LEAF.contains(&id.to_string().as_str())))
                // A name the file imported from syn — `use syn::Type as SynType`
                // makes `-> &SynType` the same door under another spelling.
                || matches!(&toks[i], TokenTree::Ident(id) if aliases.contains(&id.to_string()))
                || matches!(&toks[i], TokenTree::Group(g) if names_a_node(g.stream(), aliases))
        })
    }
    /// A function the caller could have written itself. **Named**, and then
    /// checked, because the structural test alone does not establish where the
    /// return came from (#313 review):
    ///
    /// ```ignore
    /// pub fn leak<'a>(model: &'a TypeRef, _decoy: &syn::Type) -> &'a syn::Type {
    ///     model.as_syn()
    /// }
    /// ```
    ///
    /// A node among the inputs says only that *a* node was passed, not that the
    /// returned one came from it — and any other parameter may carry the model.
    /// So the exemption is an allowlist of three, each of which must still take
    /// **exactly one** parameter, and that parameter must be the node. The decoy
    /// above cannot be exempted even by name.
    fn is_transformer(name: &str, sig: &syn::Signature, aliases: &[String]) -> bool {
        TRANSFORMERS.contains(&name)
            && sig.inputs.len() == 1
            && match sig.inputs.first() {
                Some(syn::FnArg::Typed(pt)) => {
                    names_a_node(quote::ToTokens::to_token_stream(&pt.ty), aliases)
                }
                _ => false,
            }
    }
    let is_door = |vis: &syn::Visibility, sig: &syn::Signature| {
        escapes_flat(vis)
            && hands_out_a_node(&sig.output, aliases)
            && !is_transformer(&sig.ident.to_string(), sig, aliases)
    };
    // A field is sealed when its type IS an `Origin`, whatever that origin holds:
    // `pub origin: Origin<syn::ItemFn>` names a node and hands out none.
    // `always_public` is what an enum variant's fields are: they carry no
    // visibility of their own and are reachable wherever the enum is.
    let field_is_door = |f: &syn::Field, always_public: bool| {
        let toks: Vec<TokenTree> = quote::ToTokens::to_token_stream(&f.ty)
            .into_iter()
            .collect();
        let sealed = matches!(toks.first(), Some(TokenTree::Ident(id)) if id == "Origin");
        (always_public || escapes_flat(&f.vis))
            && !sealed
            && names_a_node(quote::ToTokens::to_token_stream(&f.ty), aliases)
    };
    let fields_of = |fields: &syn::Fields,
                     ty_name: &syn::Ident,
                     always_public: bool,
                     out: &mut Vec<(String, String)>| {
        for f in fields {
            if field_is_door(f, always_public) {
                let field = f
                    .ident
                    .as_ref()
                    .map_or_else(|| "0".to_string(), ToString::to_string);
                out.push((format!("{ty_name}::{field}"), at.to_string()));
            }
        }
    };
    for item in items {
        if is_cfg_test_item(&item_attrs(item)) {
            continue;
        }
        match item {
            syn::Item::Fn(f) if is_door(&f.vis, &f.sig) => {
                out.push((f.sig.ident.to_string(), at.to_string()));
            }
            // A crate rename or a local alias puts a node behind a name this
            // check cannot follow, so the name itself is reported.
            //
            // **Whatever the alias's own visibility is.** An alias is
            // transparent: a private `type Node = syn::Type` still makes
            // `pub fn leak(&self) -> &Node` hand out a `syn::Type`, and the
            // caller never has to name `Node` to use it (#313 review).
            syn::Item::Type(t)
                if names_a_node(quote::ToTokens::to_token_stream(&t.ty), aliases) =>
            {
                out.push((format!("type {}", t.ident), at.to_string()));
            }
            syn::Item::Impl(im) => {
                for it in &im.items {
                    // An associated type is an alias reached through `Self::`,
                    // so a signature that names it names no node — which is how
                    // `impl Deref for TypeRef { type Target = syn::Type; }`
                    // hands the node to `&*ty` with nothing to count.
                    if let syn::ImplItem::Type(t) = it {
                        if names_a_node(quote::ToTokens::to_token_stream(&t.ty), aliases) {
                            out.push((format!("type {}", t.ident), at.to_string()));
                        }
                    }
                    if let syn::ImplItem::Fn(f) = it {
                        // A trait impl's method carries no visibility of its
                        // own: it is reachable wherever the trait is, so the
                        // trait's own declaration cannot be the only thing
                        // checked. Treated as public here, and again on the
                        // trait below — the same door twice is fine, a door
                        // missed is not.
                        let vis = match im.trait_ {
                            Some(_) => &syn::Visibility::Public(syn::token::Pub::default()),
                            None => &f.vis,
                        };
                        if is_door(vis, &f.sig) {
                            out.push((f.sig.ident.to_string(), at.to_string()));
                        }
                    }
                }
            }
            syn::Item::Trait(tr) if escapes_flat(&tr.vis) => {
                for it in &tr.items {
                    // An associated type's default is the same alias, declared
                    // one level up.
                    if let syn::TraitItem::Type(t) = it {
                        if t.default.as_ref().is_some_and(|(_, ty)| {
                            names_a_node(quote::ToTokens::to_token_stream(ty), aliases)
                        }) {
                            out.push((format!("{}::{}", tr.ident, t.ident), at.to_string()));
                        }
                    }
                    if let syn::TraitItem::Fn(f) = it {
                        // A trait method is as visible as its trait.
                        if is_door(&tr.vis, &f.sig) {
                            out.push((format!("{}::{}", tr.ident, f.sig.ident), at.to_string()));
                        }
                    }
                }
            }
            // A public field reopens exactly the capability an accessor closes.
            syn::Item::Struct(st) if escapes_flat(&st.vis) => {
                fields_of(&st.fields, &st.ident, false, out);
            }
            syn::Item::Enum(en) if escapes_flat(&en.vis) => {
                for v in &en.variants {
                    fields_of(&v.fields, &en.ident, true, out);
                }
            }
            syn::Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    collect_doors(items, at, aliases, out);
                }
            }
            _ => {}
        }
    }
}

/// An item's attributes, for the `#[cfg(test)]` skip the ledger scan also makes.
fn item_attrs(item: &syn::Item) -> Vec<syn::Attribute> {
    match item {
        syn::Item::Fn(i) => i.attrs.clone(),
        syn::Item::Impl(i) => i.attrs.clone(),
        syn::Item::Trait(i) => i.attrs.clone(),
        syn::Item::Struct(i) => i.attrs.clone(),
        syn::Item::Enum(i) => i.attrs.clone(),
        syn::Item::Type(i) => i.attrs.clone(),
        syn::Item::Mod(i) => i.attrs.clone(),
        _ => Vec::new(),
    }
}

fn is_cfg_test_item(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && is_cfg_test(
                quote::ToTokens::to_token_stream(&a.meta)
                    .into_iter()
                    .collect(),
            )
    })
}

/// The **guard's own** rules, on shapes the tree does not currently contain —
/// which is the point: it is what stops one from being added quietly.
#[test]
fn a_door_is_the_default_and_a_transformer_is_the_exception() {
    let doors = |src: &str| {
        let file: syn::File = syn::parse_str(src).expect("test source parses");
        let aliases: Vec<String> = syn_imports(TokenStream::from_str(src).expect("tokens"))
            .into_iter()
            .filter(|(_, original)| !LEAF.contains(&original.as_str()))
            .map(|(bound, _)| bound)
            .collect();
        let mut out = Vec::new();
        collect_doors(&file.items, "test", &aliases, &mut out);
        out.into_iter().map(|(name, _)| name).collect::<Vec<_>>()
    };

    // The #313 review's case: an associated function needs no receiver to be
    // handed the model, so a rule keyed on the receiver waved this through.
    assert_eq!(
        doors("impl TypeRef { pub fn leak(this: &Self) -> &syn::Type { this.as_syn() } }"),
        ["leak"]
    );
    // Neither does a free function, given the model by value.
    assert_eq!(
        doors("pub fn leak(t: &TypeRef) -> &syn::Type { t.as_syn() }"),
        ["leak"]
    );
    // And a syn type nobody put on a list is a door, not an oversight.
    assert_eq!(
        doors(
            "impl S { pub fn attrs(&self) -> &[syn::Attribute] { &self.origin.as_syn().attrs } }"
        ),
        ["attrs"]
    );

    // A decoy input defeats an existential test: this takes a node AND the
    // model, and returns the model's. Only the allowlist keeps it out.
    assert_eq!(
        doors(
            "pub fn leak<'a>(model: &'a TypeRef, _decoy: &syn::Type) -> &'a syn::Type { \
             model.as_syn() }"
        ),
        ["leak"]
    );
    // An alias is the same door under another spelling.
    assert_eq!(
        doors(
            "use syn::Type as SynType;\n\
             impl S { pub fn leak(&self) -> &SynType { self.as_syn() } }"
        ),
        ["leak"]
    );

    // A named transformer is exempt: it was handed a node and nothing else, so
    // it can give back only what the caller could already reach.
    assert!(doors("pub fn canonical_type(ty: &syn::Type) -> syn::Type { ty.clone() }").is_empty());
    assert!(
        doors(
            "pub fn peel_transparent(ty: &syn::Type) -> Option<(&'static str, syn::Type)> { None }"
        )
        .is_empty(),
        "the mention nests inside a token group, and must still be seen"
    );
    // The allowlist is not a password: the shape still has to hold.
    assert_eq!(
        doors("pub fn canonical_type(t: &TypeRef, ty: &syn::Type) -> syn::Type { t.as_syn() }"),
        ["canonical_type"],
        "a second parameter can carry the model, so one input or nothing"
    );
    // A leaf is not a node: there is no shape to match on.
    assert!(doors("impl S { pub fn name(&self) -> &syn::Ident { &self.name } }").is_empty());
    // The model reading itself is not a door.
    assert!(doors("impl S { pub(super) fn syntax(&self) -> &syn::Type { &self.ty } }").is_empty());
    // Nor is flat's OWN `Type`, which is the model and not a node.
    assert!(doors("impl F { pub fn declared_type(&self) -> Option<&Type> { None } }").is_empty());

    // ── Shapes that are not a function at all (#313 review) ───────────────

    // A public field reopens exactly the capability an accessor closes.
    assert_eq!(doors("pub struct S { pub node: syn::Type }"), ["S::node"]);
    // ...but a field whose type IS an `Origin` is sealed by the origin itself,
    // whatever it holds. Most of the model's public fields are this shape.
    assert!(doors("pub struct S { pub origin: Origin<syn::ItemFn> }").is_empty());
    assert!(doors("pub struct S { pub name: syn::Ident }").is_empty());
    // An enum's payload is a field too.
    assert_eq!(doors("pub enum E { A(syn::Type) }"), ["E::0"]);

    // A trait method is as visible as its trait, and its impl carries no
    // visibility of its own — so both halves are counted.
    assert_eq!(
        doors("pub trait Leak { fn leak(&self) -> &syn::Type; }"),
        ["Leak::leak"]
    );
    assert_eq!(
        doors("impl Leak for TypeRef { fn leak(&self) -> &syn::Type { self.as_syn() } }"),
        ["leak"],
        "an impl of a trait declared elsewhere is still reachable"
    );

    // An alias hides the node behind a name this check cannot follow, so the
    // ALIAS is reported — fix that, and the door it hid becomes visible.
    assert_eq!(doors("pub type Node = syn::Type;"), ["type Node"]);
    assert_eq!(
        doors("use syn as syntax;\npub type Node = syntax::Type;"),
        ["type Node"],
        "a renamed crate is followed as far as the alias that uses it"
    );
    // A leaf alias is not a door: there is still no shape to match on.
    assert!(doors("pub type Name = syn::Ident;").is_empty());
    // An alias is TRANSPARENT, so its own visibility says nothing: a private
    // one still makes `pub fn leak(&self) -> &Node` hand out a `syn::Type`,
    // and the caller never has to name `Node` to use it.
    assert_eq!(
        doors("type Node = syn::Type;\nimpl S { pub fn leak(&self) -> &Node { self.as_syn() } }"),
        ["type Node"]
    );

    // An associated type is an alias reached through `Self::`, so the signature
    // that returns it names no node at all. `&*ty` then hands out the node with
    // nothing to count.
    assert_eq!(
        doors(
            "impl std::ops::Deref for TypeRef {\n\
             type Target = syn::Type;\n\
             fn deref(&self) -> &Self::Target { self.as_syn() }\n\
             }"
        ),
        ["type Target"]
    );
    assert_eq!(
        doors("pub trait Leak { type Node = syn::Type; }"),
        ["Leak::Node"],
        "an associated type's default is the same alias, one level up"
    );
    assert!(
        doors("impl Deref for S { type Target = syn::Ident; }").is_empty(),
        "a leaf is a leaf however it is reached"
    );

    // Test code is not the model's surface.
    assert!(doors("#[cfg(test)]\nmod t { pub struct S { pub node: syn::Type } }").is_empty());
}

/// The escape scan: what it counts, which bucket it lands in, and what it must
/// not count.
#[test]
fn escapes_are_counted_by_their_receiver() {
    let c = |src: &str| scan_file(src);

    // A type escape, and an item escape, told apart by the receiver alone.
    assert_eq!(c("fn f() { let t = ty.as_syn(); }").escape_type, 1);
    assert_eq!(c("fn f() { let t = ty.as_syn(); }").escape_item, 0);
    assert_eq!(c("fn f() { let i = func.origin.as_syn(); }").escape_item, 1);
    assert_eq!(c("fn f() { let i = func.origin.as_syn(); }").escape_type, 0);

    // Spelling is free — the whole point of the split.
    assert_eq!(c("fn f() { quote!(fn g(x: #ty)) }").escape_type, 0);
    assert_eq!(c("fn f() { let t = ty.spell(); }").escape_type, 0);

    // Inside a macro body, where an AST visit would miss it.
    assert_eq!(
        c("fn f() { assert_eq!(k, TypeKey::from_type(ty.as_syn())); }").escape_type,
        1
    );
    // Two on one line: a line count would report one.
    assert_eq!(c("fn f() { g(a.as_syn(), b.as_syn()); }").escape_type, 2);

    // Test code does not count here either.
    assert_eq!(
        c("#[cfg(test)]\nmod t { fn g() { let _ = ty.as_syn(); } }").escape_type,
        0
    );

    // A method that merely *starts* with the name is not the escape.
    assert_eq!(c("fn f() { let t = ty.as_syntax(); }").escape_type, 0);

    // ── The bypasses (#313 review) ────────────────────────────────────────
    //
    // The same node, reached without ever writing `.as_syn()`. A rule matched on
    // the CALL SHAPE counted none of these, so a new escape could land with no
    // ledger drift — which is the one thing a ratchet must not allow.
    assert_eq!(
        c("fn f() { let t = TypeRef::as_syn(&ty); }").escape_type,
        1,
        "UFCS reaches the node"
    );
    assert_eq!(
        c("fn f() { let i = Origin::as_syn(&func.origin); }").escape_item,
        1,
        "UFCS on an origin is an item escape, read off the qualifier"
    );
    assert_eq!(
        c("fn f() { let read = TypeRef::as_syn; read(&ty); }").escape_type,
        1,
        "a function item is an escape at the point it is NAMED, and the call \
         site does not name it at all"
    );
    assert_eq!(
        c("fn f() { let t = <TypeRef>::as_syn(&ty); }").escape_type,
        1,
        "a qualified path still writes the name"
    );
    // The definition is not a use of itself — the model may reach its own field.
    assert_eq!(
        c("impl T { pub fn as_syn(&self) -> &S { &self.syntax } }").escape_type,
        0
    );

    // The other three doors, which predate the seal and hand out a node just as
    // completely. `escape_surface_is_closed` is what found them.
    assert_eq!(
        c("fn f() { let t = reading.stripped_syntax(); }").escape_type,
        1
    );
    assert_eq!(c("fn f() { let t = ty.kind().to_syn(); }").escape_type, 1);
    assert_eq!(
        c("fn f() { let e = registry.flat().enum_item(&n); }").escape_item,
        1,
        "an item, whatever the receiver is called"
    );
    assert_eq!(c("fn f() { let t = type_from_ident(&n); }").escape_type, 1);

    // The two populations are independent: classifying through an escape counts
    // in both, which is the intended double entry — one says a node was taken,
    // the other says what was done with it.
    let both = c("fn f() { matches!(ty.as_syn(), syn::Type::Slice(_)) }");
    assert_eq!((both.escape_type, both.classify), (1, 1));
}
