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
#   ## escapes: types   `ty.as_syn()`         — MUST reach zero
#   ## escapes: items   `f.origin.as_syn()`   — expected to persist
#
# A type escape is a source type the model should have been able to answer for;
# the classification sites above are the subset that visibly does classify. An
# item escape is a captured item's own node, which an emitter re-stating a whole
# item legitimately needs until items grow modelled accessors.
#
# The bucket is read off the RECEIVER: `origin` means the item's node, anything
# else means a type. Two over-counts land in the type bucket on purpose, both in
# the safe direction — adapter declarations reuse `Origin` for a placeless
# location (`decl.rust_type`), and carrying a spelling into an adapter-owned
# `syn::Type` field is not classification either. Both are follow-ups the count
# names rather than hides.
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
    let mut out = Vec::new();
    for run in use_runs(stream) {
        // `use syn::...` only; `use crate::Type` binds something else entirely.
        if run.first().map(String::as_str) != Some("syn") {
            continue;
        }
        let mut i = 1;
        while i < run.len() {
            if WATCHED.contains(&run[i].as_str()) {
                let renamed = run.get(i + 1).map(String::as_str) == Some("as");
                match (renamed, run.get(i + 2)) {
                    (true, Some(alias)) => {
                        out.push(alias.clone());
                        i += 3;
                        continue;
                    }
                    _ => out.push(run[i].clone()),
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

/// Which bucket an `as_syn()` call belongs to, read off the **receiver**.
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
fn escape_bucket(receiver: Option<&str>) -> fn(&mut Counts) -> &mut usize {
    match receiver {
        Some("origin") => |c: &mut Counts| &mut c.escape_item,
        _ => |c: &mut Counts| &mut c.escape_type,
    }
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
        // `<receiver> . as_syn ( )` — the escape, bucketed by receiver.
        if matches!(&toks[i], TokenTree::Ident(id) if *id == "as_syn")
            && i > 0
            && is_punct(&toks[i - 1], '.')
            && matches!(toks.get(i + 1), Some(TokenTree::Group(g))
                if g.delimiter() == Delimiter::Parenthesis && g.stream().is_empty())
        {
            let receiver = match toks.get(i.wrapping_sub(2)) {
                Some(TokenTree::Ident(id)) if i >= 2 => Some(id.to_string()),
                _ => None,
            };
            *escape_bucket(receiver.as_deref())(n) += 1;
            i += 2;
            continue;
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
    // Nor is a call that takes arguments — the escape is nullary.
    assert_eq!(c("fn f() { let t = ty.as_syn(x); }").escape_type, 0);

    // The two populations are independent: classifying through an escape counts
    // in both, which is the intended double entry — one says a node was taken,
    // the other says what was done with it.
    let both = c("fn f() { matches!(ty.as_syn(), syn::Type::Slice(_)) }");
    assert_eq!((both.escape_type, both.classify), (1, 1));
}
