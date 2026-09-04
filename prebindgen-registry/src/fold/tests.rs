//! What a row says a parameter comes apart into.
//!
//! Each test declares a row, folds it, and states the leaves it expects by
//! name, in order, with their types — because the names and the order are the
//! thing under test. They are the ones the decomposition produced before the
//! row replaced it, checked against it while both existed (#701 step 2).

use prebindgen_flat::flat::{ScalarKind, TypeRef};

use super::*;
use crate::recipe::{Arm, Constructing, RecipeName};

/// One way of obtaining a value, as a declaration writes it.
#[derive(Clone)]
enum Variant {
    /// Call this constructor.
    Ctor(syn::Ident),
    /// Take the value already built.
    Identity,
}

/// The JVM's answers, which `prebindgen-jni` states for itself in
/// `jni::param_rows`. Repeated here so this crate's tests need no adapter.
struct Jni;

fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, proc_macro2::Span::call_site())
}

impl FoldPolicy for Jni {
    fn selector(&self, prefix: &str) -> FoldLeaf {
        FoldLeaf {
            name: ident(&format!("{prefix}_sel")),
            ty: TypeRef::scalar(ScalarKind::I32),
        }
    }

    fn presence(&self, prefix: &str) -> FoldLeaf {
        FoldLeaf {
            name: ident(&format!("{prefix}_present")),
            ty: TypeRef::scalar(ScalarKind::Bool),
        }
    }

    fn sole(&self, prefix: &str) -> syn::Ident {
        ident(prefix)
    }

    fn part(&self, prefix: &str, name: &str) -> syn::Ident {
        ident(&format!("{prefix}_{name}"))
    }

    fn arm_sole(&self, prefix: &str, arm: usize) -> syn::Ident {
        ident(&format!("{prefix}_{arm}"))
    }

    fn arm_part(&self, prefix: &str, arm: usize, index: usize) -> syn::Ident {
        ident(&format!("{prefix}_{arm}_{index}"))
    }

    fn presence_leaf(&self, parts: usize) -> bool {
        // One part carries absence itself. Past one it cannot, and a flag in
        // front is cheaper than boxing a nullable primitive per part — an
        // `Option<i32>` argument would arrive as an `Integer?`.
        parts > 1
    }

    fn identity_leaf_ty(&self, ty: &TypeRef, borrowed: bool) -> TypeRef {
        // Optional because the selector decides whether this arm is live; a
        // borrowed crossing lends the value, so the leaf carries the borrow and
        // the arm clones out of it.
        if borrowed {
            ty.borrowed().optional()
        } else {
            ty.optional()
        }
    }

    fn arm_leaf_ty(&self, ty: &TypeRef) -> (TypeRef, bool) {
        // An argument already optional passes through: the wire cannot carry a
        // second absence, and `None` is a legitimate value for the arm the
        // selector picked.
        if ty.optional_inner().is_some() {
            (ty.clone(), true)
        } else {
            (ty.optional(), false)
        }
    }
}

/// One plan, rendered flat enough to compare by equality.
fn render(plan: &FoldPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "target={} by_ref={} optional={} selector={:?} present={:?}",
        plan.target.key(),
        plan.by_ref,
        plan.produces_option(),
        plan.selector,
        plan.present
    );
    for (index, leaf) in plan.leaves.iter().enumerate() {
        let _ = writeln!(out, "  leaf {index}: {} : {}", leaf.name, leaf.ty.key());
    }
    for variant in &plan.variants {
        let _ = writeln!(
            out,
            "  variant ctor={:?} fallible={} clone={}",
            variant.ctor.as_ref().map(|c| c.to_string()),
            variant.fallible,
            variant.clone
        );
        for arg in &variant.inputs {
            let _ = writeln!(out, "    {}", render_arg(arg));
        }
    }
    out
}

fn render_arg(arg: &FoldArg) -> String {
    match arg {
        FoldArg::Leaf(index, passthrough) => format!("leaf {index} passthrough={passthrough}"),
        FoldArg::Build(build) => {
            let mut out = format!(
                "build {} by_ref={} selector={:?}",
                build.target.key(),
                build.by_ref,
                build.selector
            );
            for variant in &build.variants {
                // Every field, not just the constructor. `fallible` routes the
                // error and `clone` decides whether a borrowed value survives
                // the call, so a difference in either is a difference in
                // behaviour.
                out.push_str(&format!(
                    "\n      variant ctor={:?} fallible={} clone={}",
                    variant.ctor.as_ref().map(|c| c.to_string()),
                    variant.fallible,
                    variant.clone
                ));
                for arg in &variant.inputs {
                    out.push_str(&format!("\n        {}", render_arg(arg)));
                }
            }
            out
        }
    }
}

/// A row for `variants`: one constructor is a product, several are a choice.
fn row_of(variants: &[Variant]) -> Constructing {
    if let [Variant::Ctor(func)] = variants {
        return Shape::Product(Construct::Call(func.clone()));
    }
    Shape::Choice {
        arms: variants
            .iter()
            .map(|v| Arm {
                alternative: None,
                op: match v {
                    Variant::Ctor(func) => Construct::Call(func.clone()),
                    Variant::Identity => Construct::Identity,
                },
            })
            .collect(),
    }
}

/// Fold one parameter from its row and render the plan.
fn plan_of(
    sources: &[&str],
    rows: &[(&str, Vec<Variant>)],
    func: &str,
    param: &str,
) -> Result<String, String> {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, prebindgen::SourceLocation::default())
        })
        .collect::<Vec<_>>();
    let model = prebindgen_flat::flat::Flat::builder()
        .items(crate::test_util::declare_referenced(items))
        .build()
        .expect("parse");
    let mut recipes = Recipes::builder();
    for (target, variants) in rows {
        let ty = model
            .classify(&syn::parse_str(target).expect("test type"))
            .expect("a type the model accepts");
        // DECLARED, not defaulted: the crossing's default row stays its own
        // conversion, which is what an identity arm's part takes.
        recipes.declare_derived_default(ty.clone(), Direction::Construct);
        recipes.declare(ty, RecipeName::new("parts"), row_of(variants));
    }
    let recipes = recipes.build(&model).expect("the rows build");
    let reading = model
        .function(&ident(func))
        .expect("the function")
        .params
        .iter()
        .find(|p| p.name == ident(param))
        .expect("the parameter")
        .ty
        .clone();
    Folding::new(&recipes, &model)
        .fold(
            &Jni,
            param,
            &reading,
            &RecipeName::new("parts"),
            &RecipeName::new("parts"),
        )
        .map(|plan| render(&plan))
        .map_err(|e| e.to_string())
}

/// [`plan_of`], for a row that must fold.
fn folds(sources: &[&str], rows: &[(&str, Vec<Variant>)], func: &str, param: &str) -> String {
    plan_of(sources, rows, func, param).unwrap_or_else(|e| panic!("the row does not fold: {e}"))
}

/// The rendered plan as lines, so a test states its leaves one per line.
fn lines(plan: &str) -> Vec<&str> {
    plan.lines().collect()
}

/// A declaration the row form refuses, and the reason it gives.
fn rejects(
    sources: &[&str],
    rows: &[(&str, Vec<Variant>)],
    func: &str,
    param: &str,
    because: &str,
) {
    let refusal = plan_of(sources, rows, func, param)
        .err()
        .unwrap_or_else(|| panic!("{because} was accepted"));
    assert!(
        refusal.contains("built from leaves of its own"),
        "refused as unsupported nesting: {refusal}"
    );
}

/// A constructor argument that builds from leaves of its own, standing inside
/// an arm. Both paths refuse it: the arm's leaves are live only when the
/// selector picks that arm, and a nested build's leaves would have to be live
/// on that condition too.
#[test]
fn a_nested_build_inside_an_arm_is_refused() {
    rejects(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn sel_new(key: KeyExpr, n: i32) -> Selector { todo!() }",
            "fn query(selector: Selector) {}",
        ],
        &[
            ("KeyExpr", vec![Variant::Ctor(ident("key_new"))]),
            (
                "Selector",
                vec![Variant::Ctor(ident("sel_new")), Variant::Identity],
            ),
        ],
        "query",
        "selector",
        "a nested build inside an arm",
    );
}

/// The same argument, optional. Both paths refuse it: the value's own absence
/// and the argument's would need two answers on one wire.
#[test]
fn an_optional_nested_build_is_refused() {
    rejects(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn sel_new(key: Option<KeyExpr>, n: i32) -> Selector { todo!() }",
            "fn query(selector: Selector) {}",
        ],
        &[
            ("KeyExpr", vec![Variant::Ctor(ident("key_new"))]),
            ("Selector", vec![Variant::Ctor(ident("sel_new"))]),
        ],
        "query",
        "selector",
        "an optional nested build",
    );
}

/// One constructor taking one argument: the parameter keeps its own name.
#[test]
fn a_single_argument_constructor() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: KeyExpr) {}",
        ],
        &[("KeyExpr", vec![Variant::Ctor(ident("key_new"))])],
        "publish",
        "key",
    );
    assert_eq!(
        lines(&row),
        [
            "target=KeyExpr by_ref=false optional=false selector=None present=None",
            "  leaf 0: key : & str",
            "  variant ctor=Some(\"key_new\") fallible=false clone=false",
            "    leaf 0 passthrough=false",
        ]
    );
}

/// One constructor taking several: each leaf is named after its parameter.
#[test]
fn a_multi_argument_constructor() {
    let row = folds(
        &[
            "fn enc_new(id: i32, schema: u64) -> Encoding { todo!() }",
            "fn put(encoding: Encoding) {}",
        ],
        &[("Encoding", vec![Variant::Ctor(ident("enc_new"))])],
        "put",
        "encoding",
    );
    assert_eq!(
        lines(&row),
        [
            "target=Encoding by_ref=false optional=false selector=None present=None",
            "  leaf 0: encoding_id : i32",
            "  leaf 1: encoding_schema : u64",
            "  variant ctor=Some(\"enc_new\") fallible=false clone=false",
            "    leaf 0 passthrough=false",
            "    leaf 1 passthrough=false",
        ]
    );
}

/// A constructor and the value itself: a selector, then one leaf per arm.
#[test]
fn a_choice_of_constructor_and_identity() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: KeyExpr) {}",
        ],
        &[(
            "KeyExpr",
            vec![Variant::Ctor(ident("key_new")), Variant::Identity],
        )],
        "publish",
        "key",
    );
    assert_eq!(
        lines(&row),
        [
            "target=KeyExpr by_ref=false optional=false selector=Some(0) present=None",
            "  leaf 0: key_sel : i32",
            "  leaf 1: key_0 : Option < & str >",
            "  leaf 2: key_1 : Option < KeyExpr >",
            "  variant ctor=Some(\"key_new\") fallible=false clone=false",
            "    leaf 1 passthrough=false",
            "  variant ctor=None fallible=false clone=false",
            "    leaf 2 passthrough=false",
        ]
    );
}

/// A borrowed parameter: the identity arm clones, and its leaf is a borrow.
#[test]
fn a_borrowed_choice_clones_its_identity_arm() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: &KeyExpr) {}",
        ],
        &[(
            "KeyExpr",
            vec![Variant::Ctor(ident("key_new")), Variant::Identity],
        )],
        "publish",
        "key",
    );
    assert_eq!(
        lines(&row),
        [
            "target=KeyExpr by_ref=true optional=false selector=Some(0) present=None",
            "  leaf 0: key_sel : i32",
            "  leaf 1: key_0 : Option < & str >",
            "  leaf 2: key_1 : Option < & KeyExpr >",
            "  variant ctor=Some(\"key_new\") fallible=false clone=false",
            "    leaf 1 passthrough=false",
            "  variant ctor=None fallible=false clone=true",
            "    leaf 2 passthrough=false",
        ]
    );
}

/// An optional parameter built by a one-argument constructor: one nullable
/// leaf carries both the value and its absence.
#[test]
fn an_optional_single_argument_constructor() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: Option<KeyExpr>) {}",
        ],
        &[("KeyExpr", vec![Variant::Ctor(ident("key_new"))])],
        "publish",
        "key",
    );
    assert_eq!(
        lines(&row),
        [
            "target=KeyExpr by_ref=false optional=true selector=None present=None",
            "  leaf 0: key : Option < & str >",
            "  variant ctor=Some(\"key_new\") fallible=false clone=false",
            "    leaf 0 passthrough=false",
        ]
    );
}

/// An optional parameter built by a multi-argument constructor: a presence
/// flag in front, and the arguments stay plain.
#[test]
fn an_optional_multi_argument_constructor() {
    let row = folds(
        &[
            "fn enc_new(id: i32, schema: u64) -> Encoding { todo!() }",
            "fn put(encoding: Option<Encoding>) {}",
        ],
        &[("Encoding", vec![Variant::Ctor(ident("enc_new"))])],
        "put",
        "encoding",
    );
    assert_eq!(
        lines(&row),
        [
            "target=Encoding by_ref=false optional=true selector=None present=Some(0)",
            "  leaf 0: encoding_present : bool",
            "  leaf 1: encoding_id : i32",
            "  leaf 2: encoding_schema : u64",
            "  variant ctor=Some(\"enc_new\") fallible=false clone=false",
            "    leaf 1 passthrough=false",
            "    leaf 2 passthrough=false",
        ]
    );
}

/// An optional parameter with several arms: the selector carries absence, and
/// no presence flag joins it.
#[test]
fn an_optional_choice() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: Option<KeyExpr>) {}",
        ],
        &[(
            "KeyExpr",
            vec![Variant::Ctor(ident("key_new")), Variant::Identity],
        )],
        "publish",
        "key",
    );
    assert_eq!(
        lines(&row),
        [
            "target=KeyExpr by_ref=false optional=true selector=Some(0) present=None",
            "  leaf 0: key_sel : i32",
            "  leaf 1: key_0 : Option < & str >",
            "  leaf 2: key_1 : Option < KeyExpr >",
            "  variant ctor=Some(\"key_new\") fallible=false clone=false",
            "    leaf 1 passthrough=false",
            "  variant ctor=None fallible=false clone=false",
            "    leaf 2 passthrough=false",
        ]
    );
}

/// A nested build whose own constructor is fallible carries that through: the
/// inner call's error is routed, and the flag saying so is part of the plan.
#[test]
fn a_nested_build_keeps_its_constructors_fallibility() {
    let row = folds(
        &[
            "fn key_try(s: &str) -> Result<KeyExpr, Error> { todo!() }",
            "fn sel_new(key: KeyExpr, n: i32) -> Selector { todo!() }",
            "fn query(selector: Selector) {}",
        ],
        &[
            ("KeyExpr", vec![Variant::Ctor(ident("key_try"))]),
            ("Selector", vec![Variant::Ctor(ident("sel_new"))]),
        ],
        "query",
        "selector",
    );
    assert!(
        row.contains("fallible=true"),
        "the fixture must actually nest a fallible constructor: {row}"
    );
    assert_eq!(
        lines(&row),
        [
            "target=Selector by_ref=false optional=false selector=None present=None",
            "  leaf 0: selector_key : & str",
            "  leaf 1: selector_n : i32",
            "  variant ctor=Some(\"sel_new\") fallible=false clone=false",
            "    build KeyExpr by_ref=false selector=None",
            "      variant ctor=Some(\"key_try\") fallible=true clone=false",
            "        leaf 0 passthrough=false",
            "    leaf 1 passthrough=false",
        ]
    );
}

/// A nested build reached through a borrowed argument clones on its identity
/// arm, the same as a borrowed parameter does at the top level.
#[test]
fn a_nested_build_clones_a_borrowed_identity_arm() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn sel_new(key: &KeyExpr, n: i32) -> Selector { todo!() }",
            "fn query(selector: Selector) {}",
        ],
        &[
            (
                "KeyExpr",
                vec![Variant::Ctor(ident("key_new")), Variant::Identity],
            ),
            ("Selector", vec![Variant::Ctor(ident("sel_new"))]),
        ],
        "query",
        "selector",
    );
    assert!(
        row.contains("clone=true"),
        "the fixture must actually clone a borrowed identity arm: {row}"
    );
    assert_eq!(
        lines(&row),
        [
            "target=Selector by_ref=false optional=false selector=None present=None",
            "  leaf 0: selector_key_sel : i32",
            "  leaf 1: selector_key_0 : Option < & str >",
            "  leaf 2: selector_key_1 : Option < & KeyExpr >",
            "  leaf 3: selector_n : i32",
            "  variant ctor=Some(\"sel_new\") fallible=false clone=false",
            "    build KeyExpr by_ref=true selector=Some(0)",
            "      variant ctor=Some(\"key_new\") fallible=false clone=false",
            "        leaf 1 passthrough=false",
            "      variant ctor=None fallible=false clone=true",
            "        leaf 2 passthrough=false",
            "    leaf 3 passthrough=false",
        ]
    );
}

/// A constructor argument whose own type states a constructor row is built the
/// same way, and its leaves are named under the argument's name.
#[test]
fn a_nested_build() {
    let row = folds(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn sel_new(key: KeyExpr, n: i32) -> Selector { todo!() }",
            "fn query(selector: Selector) {}",
        ],
        &[
            ("KeyExpr", vec![Variant::Ctor(ident("key_new"))]),
            ("Selector", vec![Variant::Ctor(ident("sel_new"))]),
        ],
        "query",
        "selector",
    );
    assert_eq!(
        lines(&row),
        [
            "target=Selector by_ref=false optional=false selector=None present=None",
            "  leaf 0: selector_key : & str",
            "  leaf 1: selector_n : i32",
            "  variant ctor=Some(\"sel_new\") fallible=false clone=false",
            "    build KeyExpr by_ref=false selector=None",
            "      variant ctor=Some(\"key_new\") fallible=false clone=false",
            "        leaf 0 passthrough=false",
            "    leaf 1 passthrough=false",
        ]
    );
}

/// An argument that is itself optional passes through an arm unwrapped: the
/// wire has no second absence to spend, and the selector already said which
/// arm is live.
#[test]
fn an_optional_argument_inside_an_arm() {
    let row = folds(
        &[
            "fn enc_new(id: i32, schema: Option<u64>) -> Encoding { todo!() }",
            "fn put(encoding: Encoding) {}",
        ],
        &[(
            "Encoding",
            vec![Variant::Ctor(ident("enc_new")), Variant::Identity],
        )],
        "put",
        "encoding",
    );
    assert_eq!(
        lines(&row),
        [
            "target=Encoding by_ref=false optional=false selector=Some(0) present=None",
            "  leaf 0: encoding_sel : i32",
            "  leaf 1: encoding_0_0 : Option < i32 >",
            "  leaf 2: encoding_0_1 : Option < u64 >",
            "  leaf 3: encoding_1 : Option < Encoding >",
            "  variant ctor=Some(\"enc_new\") fallible=false clone=false",
            "    leaf 1 passthrough=false",
            "    leaf 2 passthrough=true",
            "  variant ctor=None fallible=false clone=false",
            "    leaf 3 passthrough=false",
        ]
    );
}
