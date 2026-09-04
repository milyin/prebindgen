//! The view against the decomposition it replaces.
//!
//! Every test here builds the same construction twice — once through
//! `expand::apply`, from the declarations, and once through [`Folding::fold`],
//! from a row — and asserts the two plans agree. That is the question #701 step
//! 2 turns on: whether a row tree can carry what the decomposition carried,
//! down to the leaf names and their order.

use prebindgen_flat::flat::{ScalarKind, TypeRef};

use super::*;
use crate::{
    expand::{ExpandDecl, ExpandSel, Expansions, Variant},
    recipe::{Arm, Constructing, RecipeName},
};

/// The JVM's answers, which are the ones `expand::apply` hard-codes today.
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
                out.push_str(&format!(
                    "\n      variant ctor={:?}",
                    variant.ctor.as_ref().map(|c| c.to_string())
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

/// Build both plans for one parameter and return them rendered.
///
/// `sources` is the model; `expansions` names the constructors for each type,
/// in the form the declarations take; `func`/`param` is the position.
fn both(
    sources: &[&str],
    constructors: &[(&str, Vec<Variant>)],
    func: &str,
    param: &str,
) -> (String, String) {
    let (decl, row) = run(sources, constructors, func, param);
    (
        decl.unwrap_or_else(|e| panic!("the declarations were rejected: {e}")),
        row.unwrap_or_else(|e| panic!("the row was rejected: {e}")),
    )
}

/// Both paths must refuse this declaration, and neither may quietly answer
/// with a different boundary instead.
fn rejects(
    sources: &[&str],
    constructors: &[(&str, Vec<Variant>)],
    func: &str,
    param: &str,
    because: &str,
) {
    let (decl, row) = run(sources, constructors, func, param);
    let decl = decl.err().unwrap_or_else(|| {
        panic!("the declarations accepted {because}, so there is no parity to hold")
    });
    let row = row
        .err()
        .unwrap_or_else(|| panic!("the row accepted {because}, which the declarations refuse"));
    assert!(
        decl.contains("recursive") || decl.contains("Recursive"),
        "the declarations refuse it as unsupported nesting: {decl}"
    );
    assert!(
        row.contains("built from leaves of its own"),
        "the row refuses it as unsupported nesting: {row}"
    );
}

/// Build both plans for one parameter, keeping whichever refusal came back.
fn run(
    sources: &[&str],
    constructors: &[(&str, Vec<Variant>)],
    func: &str,
    param: &str,
) -> (Result<String, String>, Result<String, String>) {
    // ── the decomposition ───────────────────────────────────────────────
    let decls = Expansions {
        constructors: constructors
            .iter()
            .map(|(target, variants)| crate::expand::ConstructorDecl {
                target: crate::TypeKey::parse(target).expect("test type"),
                variants: variants.clone(),
                default: true,
            })
            .collect(),
        expands: vec![ExpandDecl {
            func: ident(func),
            param: ident(param),
            declared_target: None,
            sel: ExpandSel::TopLevel,
        }],
        ..Default::default()
    };
    let mut builder = crate::test_util::reg_with(sources);
    for (_, variants) in constructors {
        for variant in variants {
            if let Variant::Ctor(name) = variant {
                builder = builder.export(name);
            }
        }
    }
    let mut builder = builder
        .export(&ident(func))
        .decompose(crate::Decompositions {
            expansions: Some(decls),
            ..Default::default()
        });
    // Derives the plans; the readings themselves are not what this reads back.
    let applied = builder
        .expansion_leaf_readings()
        .map(|readings| readings.count());
    let from_declarations = match applied {
        Err(e) => Err(e.to_string()),
        Ok(_) => {
            let plans = crate::Conversions::expansion_plans(&builder);
            match plans.get(&(ident(func), ident(param))) {
                Some(expanded) => Ok(render(expanded)),
                None => Err("no plan for the parameter".to_string()),
            }
        }
    };

    // ── the row ─────────────────────────────────────────────────────────
    let model = crate::Conversions::flat(&builder).clone();
    let mut recipes = Recipes::builder();
    for (target, variants) in constructors {
        let ty = model
            .classify(&syn::parse_str(target).expect("test type"))
            .expect("a type the model accepts");
        // DECLARED, not defaulted: the crossing's default row stays its own
        // conversion, which is what an identity arm's part takes.
        recipes.declare(ty, RecipeName::new("parts"), row_of(variants));
    }
    let recipes = recipes.build(&model).expect("the rows build");
    let bindings = Bindings::builder()
        .build(&recipes)
        .expect("no bindings to resolve");
    let reading = model
        .function(&ident(func))
        .expect("the function")
        .params
        .iter()
        .find(|p| p.name == ident(param))
        .expect("the parameter")
        .ty
        .clone();
    let from_row = Folding::new(&recipes, &bindings, &model)
        .fold(&Jni, param, &reading, &RecipeName::new("parts"))
        .map(|plan| render(&plan))
        .map_err(|e| e.to_string());
    (from_declarations, from_row)
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
    let (decl, row) = both(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: KeyExpr) {}",
        ],
        &[("KeyExpr", vec![Variant::Ctor(ident("key_new"))])],
        "publish",
        "key",
    );
    assert_eq!(decl, row);
}

/// One constructor taking several: each leaf is named after its parameter.
#[test]
fn a_multi_argument_constructor() {
    let (decl, row) = both(
        &[
            "fn enc_new(id: i32, schema: u64) -> Encoding { todo!() }",
            "fn put(encoding: Encoding) {}",
        ],
        &[("Encoding", vec![Variant::Ctor(ident("enc_new"))])],
        "put",
        "encoding",
    );
    assert_eq!(decl, row);
}

/// A constructor and the value itself: a selector, then one leaf per arm.
#[test]
fn a_choice_of_constructor_and_identity() {
    let (decl, row) = both(
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
    assert_eq!(decl, row);
}

/// A borrowed parameter: the identity arm clones, and its leaf is a borrow.
#[test]
fn a_borrowed_choice_clones_its_identity_arm() {
    let (decl, row) = both(
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
    assert_eq!(decl, row);
}

/// An optional parameter built by a one-argument constructor: one nullable
/// leaf carries both the value and its absence.
#[test]
fn an_optional_single_argument_constructor() {
    let (decl, row) = both(
        &[
            "fn key_new(s: &str) -> KeyExpr { todo!() }",
            "fn publish(key: Option<KeyExpr>) {}",
        ],
        &[("KeyExpr", vec![Variant::Ctor(ident("key_new"))])],
        "publish",
        "key",
    );
    assert_eq!(decl, row);
}

/// An optional parameter built by a multi-argument constructor: a presence
/// flag in front, and the arguments stay plain.
#[test]
fn an_optional_multi_argument_constructor() {
    let (decl, row) = both(
        &[
            "fn enc_new(id: i32, schema: u64) -> Encoding { todo!() }",
            "fn put(encoding: Option<Encoding>) {}",
        ],
        &[("Encoding", vec![Variant::Ctor(ident("enc_new"))])],
        "put",
        "encoding",
    );
    assert_eq!(decl, row);
}

/// An optional parameter with several arms: the selector carries absence, and
/// no presence flag joins it.
#[test]
fn an_optional_choice() {
    let (decl, row) = both(
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
    assert_eq!(decl, row);
}

/// A constructor argument whose own type states a constructor row is built the
/// same way, and its leaves are named under the argument's name.
#[test]
fn a_nested_build() {
    let (decl, row) = both(
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
    assert_eq!(decl, row);
}

/// An argument that is itself optional passes through an arm unwrapped: the
/// wire has no second absence to spend, and the selector already said which
/// arm is live.
#[test]
fn an_optional_argument_inside_an_arm() {
    let (decl, row) = both(
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
    assert_eq!(decl, row);
}
