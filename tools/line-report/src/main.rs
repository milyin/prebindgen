//! Rust source lines per crate, split so adapter growth cannot hide in tests.
//!
//! #613's size gate is "prebindgen-c and prebindgen-jni production Rust line
//! counts both decrease", which is only checkable if everyone counts the same
//! way. This is that way:
//!
//! ```text
//! cargo run --manifest-path tools/line-report/Cargo.toml
//! cargo run --manifest-path tools/line-report/Cargo.toml -- prebindgen-c
//! cargo run --manifest-path tools/line-report/Cargo.toml -- --self-test
//! ```
//!
//! Three numbers per crate, and every line of every `src/**/*.rs` file is in
//! exactly one of them:
//!
//! - **test files** — a file under a `tests/` directory, or named `tests.rs`
//!   or `test_util.rs`. Wholly test support.
//! - **test items** — `#[cfg(test)]` items and statements inside the remaining
//!   files, from the attribute to the end of what it applies to. Test support
//!   that would otherwise read as production, which is the number that stops a
//!   move into an inline test module from looking like a deletion.
//! - **production** — what is left, and what the gate is about.
//!
//! Only a bare `#[cfg(test)]` counts. An item behind
//! `#[cfg(any(test, feature = "testing"))]` ships to other crates under that
//! feature, so it is production by this rule.
//!
//! **The boundary is syntactic.** `syn` parses the file and the walk asks each
//! attributed node for its own span; nothing here matches braces, indentation
//! or line shapes. Three rounds of #614 review each found another valid
//! construct a delimiter heuristic could not delimit — a gated multiline call
//! ending in `);`, a `}` inside a raw string or nested block comment, and a
//! gated `let x = if c { .. } else { .. };` whose first brace pair is not its
//! body — which is why the question is put to a parser instead.

use std::{collections::BTreeSet, path::Path};

use syn::spanned::Spanned;

const CRATES: [&str; 8] = [
    "prebindgen",
    "prebindgen-flat",
    "prebindgen-registry",
    "prebindgen-c",
    "prebindgen-jni",
    "prebindgen-c-runtime",
    "prebindgen-jni-runtime",
    "prebindgen-proc-macro",
];

fn main() {
    // `tools/line-report` -> the repository root.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository root is two levels above this crate")
        .to_path_buf();
    let requested: Vec<String> = std::env::args().skip(1).collect();
    if requested == ["--self-test"] {
        self_test();
        return;
    }
    let crates: Vec<&str> = if requested.is_empty() {
        CRATES.to_vec()
    } else {
        requested.iter().map(String::as_str).collect()
    };

    println!(
        "{:<24}{:>12}{:>12}{:>12}{:>12}",
        "crate", "production", "test items", "test files", "total"
    );
    let (mut all_production, mut all_items, mut all_files) = (0usize, 0usize, 0usize);
    for name in crates {
        let src = root.join(name).join("src");
        if !src.is_dir() {
            continue;
        }
        let (mut production, mut items, mut test_files) = (0usize, 0usize, 0usize);
        for path in sources(&src) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            if is_test_support(path.strip_prefix(&src).unwrap_or(&path)) {
                test_files += text.lines().count();
                continue;
            }
            let (file_production, file_items) = count(&text, &path.display().to_string());
            production += file_production;
            items += file_items;
        }
        let total = production + items + test_files;
        println!("{name:<24}{production:>12}{items:>12}{test_files:>12}{total:>12}");
        all_production += production;
        all_items += items;
        all_files += test_files;
    }
    println!(
        "{:<24}{:>12}{:>12}{:>12}{:>12}",
        "TOTAL",
        all_production,
        all_items,
        all_files,
        all_production + all_items + all_files
    );
}

/// Every `.rs` file under `dir`, in path order.
fn sources(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
            .map(|entry| entry.expect("read directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn is_test_support(relative: &Path) -> bool {
    relative
        .file_name()
        .is_some_and(|name| name == "tests.rs" || name == "test_util.rs")
        || relative
            .components()
            .any(|part| part.as_os_str() == "tests")
}

/// (production lines, test-item lines) for one production file.
fn count(text: &str, label: &str) -> (usize, usize) {
    let file: syn::File = syn::parse_file(text)
        .unwrap_or_else(|error| panic!("line report cannot parse {label}: {error}"));
    let mut gated = BTreeSet::new();
    let mut walk = Walk { gated: &mut gated };
    syn::visit::Visit::visit_file(&mut walk, &file);
    let lines = text.lines().count();
    let items = gated.len();
    (lines - items, items)
}

/// Collects the line numbers covered by `#[cfg(test)]` nodes.
///
/// A set of lines rather than a count: two gated nodes can share a line, and a
/// gated node inside a gated module must not be counted twice.
struct Walk<'a> {
    gated: &'a mut BTreeSet<usize>,
}

impl Walk<'_> {
    /// Record `node`'s lines when `attrs` gate it out of a normal build.
    fn record<T: Spanned>(&mut self, attrs: &[syn::Attribute], node: &T) {
        if !attrs.iter().any(is_cfg_test) {
            return;
        }
        // From the FIRST attribute rather than from the node: a node's own
        // span starts after its attributes, and the `#[cfg(test)]` line is
        // test support like the rest of what it gates.
        let start = attrs
            .iter()
            .map(|attr| attr.span().start().line)
            .min()
            .unwrap_or_else(|| node.span().start().line);
        for line in start..=node.span().end().line {
            self.gated.insert(line);
        }
    }
}

/// Whether this attribute is exactly `#[cfg(test)]`.
///
/// `#[cfg(any(test, feature = "testing"))]` is not: what it gates ships to
/// other crates under that feature, so it is production.
fn is_cfg_test(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("cfg")
        && attr
            .parse_args::<syn::Path>()
            .is_ok_and(|path| path.is_ident("test"))
}

impl<'ast> syn::visit::Visit<'ast> for Walk<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if let Some(attrs) = item_attrs(item) {
            self.record(attrs, item);
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        if let Some(attrs) = impl_item_attrs(item) {
            self.record(attrs, item);
        }
        syn::visit::visit_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        if let Some(attrs) = trait_item_attrs(item) {
            self.record(attrs, item);
        }
        syn::visit::visit_trait_item(self, item);
    }

    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        match stmt {
            syn::Stmt::Local(local) => self.record(&local.attrs, stmt),
            syn::Stmt::Macro(item) => self.record(&item.attrs, stmt),
            syn::Stmt::Expr(expression, _) => self.record(expression_attrs(expression), stmt),
            syn::Stmt::Item(_) => {}
        }
        syn::visit::visit_stmt(self, stmt);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.record(&field.attrs, field);
        syn::visit::visit_field(self, field);
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        self.record(&variant.attrs, variant);
        syn::visit::visit_variant(self, variant);
    }
}

fn item_attrs(item: &syn::Item) -> Option<&[syn::Attribute]> {
    Some(match item {
        syn::Item::Const(item) => &item.attrs,
        syn::Item::Enum(item) => &item.attrs,
        syn::Item::ExternCrate(item) => &item.attrs,
        syn::Item::Fn(item) => &item.attrs,
        syn::Item::ForeignMod(item) => &item.attrs,
        syn::Item::Impl(item) => &item.attrs,
        syn::Item::Macro(item) => &item.attrs,
        syn::Item::Mod(item) => &item.attrs,
        syn::Item::Static(item) => &item.attrs,
        syn::Item::Struct(item) => &item.attrs,
        syn::Item::Trait(item) => &item.attrs,
        syn::Item::TraitAlias(item) => &item.attrs,
        syn::Item::Type(item) => &item.attrs,
        syn::Item::Union(item) => &item.attrs,
        syn::Item::Use(item) => &item.attrs,
        _ => return None,
    })
}

fn impl_item_attrs(item: &syn::ImplItem) -> Option<&[syn::Attribute]> {
    Some(match item {
        syn::ImplItem::Const(item) => &item.attrs,
        syn::ImplItem::Fn(item) => &item.attrs,
        syn::ImplItem::Macro(item) => &item.attrs,
        syn::ImplItem::Type(item) => &item.attrs,
        _ => return None,
    })
}

fn trait_item_attrs(item: &syn::TraitItem) -> Option<&[syn::Attribute]> {
    Some(match item {
        syn::TraitItem::Const(item) => &item.attrs,
        syn::TraitItem::Fn(item) => &item.attrs,
        syn::TraitItem::Macro(item) => &item.attrs,
        syn::TraitItem::Type(item) => &item.attrs,
        _ => return None,
    })
}

fn expression_attrs(expression: &syn::Expr) -> &[syn::Attribute] {
    match expression {
        syn::Expr::Array(node) => &node.attrs,
        syn::Expr::Assign(node) => &node.attrs,
        syn::Expr::Async(node) => &node.attrs,
        syn::Expr::Await(node) => &node.attrs,
        syn::Expr::Binary(node) => &node.attrs,
        syn::Expr::Block(node) => &node.attrs,
        syn::Expr::Break(node) => &node.attrs,
        syn::Expr::Call(node) => &node.attrs,
        syn::Expr::Cast(node) => &node.attrs,
        syn::Expr::Closure(node) => &node.attrs,
        syn::Expr::Const(node) => &node.attrs,
        syn::Expr::Continue(node) => &node.attrs,
        syn::Expr::Field(node) => &node.attrs,
        syn::Expr::ForLoop(node) => &node.attrs,
        syn::Expr::Group(node) => &node.attrs,
        syn::Expr::If(node) => &node.attrs,
        syn::Expr::Index(node) => &node.attrs,
        syn::Expr::Infer(node) => &node.attrs,
        syn::Expr::Let(node) => &node.attrs,
        syn::Expr::Lit(node) => &node.attrs,
        syn::Expr::Loop(node) => &node.attrs,
        syn::Expr::Macro(node) => &node.attrs,
        syn::Expr::Match(node) => &node.attrs,
        syn::Expr::MethodCall(node) => &node.attrs,
        syn::Expr::Paren(node) => &node.attrs,
        syn::Expr::Path(node) => &node.attrs,
        syn::Expr::Range(node) => &node.attrs,
        syn::Expr::Reference(node) => &node.attrs,
        syn::Expr::Repeat(node) => &node.attrs,
        syn::Expr::Return(node) => &node.attrs,
        syn::Expr::Struct(node) => &node.attrs,
        syn::Expr::Try(node) => &node.attrs,
        syn::Expr::TryBlock(node) => &node.attrs,
        syn::Expr::Tuple(node) => &node.attrs,
        syn::Expr::Unary(node) => &node.attrs,
        syn::Expr::Unsafe(node) => &node.attrs,
        syn::Expr::While(node) => &node.attrs,
        syn::Expr::Yield(node) => &node.attrs,
        _ => &[],
    }
}

/// The constructs each round of #614's review found, and what the counter says
/// about them. Run with
/// `cargo run -p prebindgen-registry --features line-report --example
/// line-report -- --self-test`.
fn self_test() {
    // Every case here defeated a delimiter heuristic: the gated call ends in
    // `);` rather than a brace; the raw string and the block comment contain a
    // `}` that is text; the `if`/`else` and the chained initializer close their
    // first brace pair in the middle of the statement.
    let fixture = r####"
fn production_one() {
    let brace = "{";
}

#[cfg(test)]
mod tests;

impl Thing {
    #[cfg(test)]
    fn gated_method(&self) -> bool {
        self.0
    }

    fn production_two(&self) {
        #[cfg(test)]
        check(
            self.first(),
            "jni",
        );
        self.second();

        #[cfg(test)]
        let chosen = if self.first() {
            1
        } else {
            2
        };

        #[cfg(test)]
        let built = Thing {
            field: 1,
        }
        .method();

        #[cfg(test)]
        fn gated_with_a_raw_string() {
            let source = r#"
}
"#;
            check(source);
        }

        #[cfg(test)]
        fn gated_with_a_block_comment() {
            /* closes a block:
            }
            and nests: /* } */ still inside */
            done();
        }
    }
}

#[cfg(any(test, feature = "testing"))]
pub fn shipped_under_a_feature() -> bool {
    true
}
"####;
    let (production, items) = count(fixture, "self-test fixture");
    let total = fixture.lines().count();
    assert_eq!(production + items, total, "every line is counted once");
    // `mod tests;` 2, `gated_method` 4, the call 5, the `if`/`else` 6, the
    // chained initializer 5, and the two nested functions 7 each — 36 lines.
    // The statements between them, `self.second();` among them, are production.
    assert_eq!(items, 36, "the gated constructs are 36 lines");
    assert!(
        production > 0 && production == total - 36,
        "nothing outside a gated construct is counted as one"
    );
    println!("self-test ok: {total} lines, {items} gated, {production} production");
}
