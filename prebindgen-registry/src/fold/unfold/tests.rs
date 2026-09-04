//! What a row says a value comes apart into.
//!
//! Each test declares a deconstructing row, unfolds it, and states the leaves
//! it expects by name, in order, with what each carries — the same shape the
//! constructing tests beside it take, because the two answer one question in
//! opposite directions.

use prebindgen_flat::flat::{ScalarKind, TypeRef};

use super::*;
use crate::recipe::{Arm, ArmKey, Deconstructing, Recipes};

/// The JVM's answers, which `prebindgen-jni` states for itself. Repeated here
/// so this crate's tests need no adapter.
struct Jni;

fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, proc_macro2::Span::call_site())
}

impl UnfoldPolicy for Jni {
    fn selector(&self, source: &TypeRef) -> UnfoldLeaf {
        UnfoldLeaf {
            name: "tag".to_string(),
            path: Vec::new(),
            // The sum, not the integer it crosses as: what the emitter needs is
            // which sum it is choosing between.
            out_ty: source.clone(),
            identity: false,
            nullable: false,
            source: LeafSource::SumTag,
            groups: Vec::new(),
        }
    }

    fn presence(&self, name: &str) -> UnfoldLeaf {
        UnfoldLeaf {
            name: format!("{name}__present"),
            path: Vec::new(),
            out_ty: TypeRef::scalar(ScalarKind::Bool),
            identity: false,
            nullable: false,
            source: LeafSource::Presence,
            groups: Vec::new(),
        }
    }

    fn part_name(&self, reach: &Reach, index: usize, field: Option<&syn::Ident>) -> String {
        match reach {
            Reach::Accessor(func) => func.to_string(),
            // A field's own name where it has one, which is what a struct's
            // slots are called on the far side.
            _ => field.map_or_else(|| format!("f{index}"), |name| name.to_string()),
        }
    }

    fn arm_part_name(&self, variant: &syn::Ident, member: &syn::Member, _index: usize) -> String {
        let member = match member {
            syn::Member::Named(name) => name.to_string(),
            syn::Member::Unnamed(index) => format!("v{}", index.index),
        };
        format!("{variant}__{member}")
    }

    fn identity_name(&self) -> String {
        "handle".to_string()
    }

    fn nest(&self, outer: &str, inner: &str) -> String {
        format!("{outer}__{inner}")
    }
}

/// One leaf per line, flat enough for a test to state what it expects.
fn render(leaves: &[UnfoldLeaf], hoists: &[Hoist]) -> Vec<String> {
    let path_of = |steps: &[PathStep]| {
        steps
            .iter()
            .map(|step| step.ident().to_string())
            .collect::<Vec<_>>()
            .join(".")
    };
    let mut out: Vec<String> = leaves
        .iter()
        .map(|leaf| {
            format!(
                "{} : {} path=[{}] identity={} nullable={} groups={:?}",
                leaf.name,
                leaf.out_ty.key(),
                path_of(&leaf.path),
                leaf.identity,
                leaf.nullable,
                leaf.groups
            )
        })
        .collect();
    for hoist in hoists {
        out.push(format!(
            "hoist [{}] consuming={}",
            path_of(&hoist.prefix),
            hoist.consuming
        ));
    }
    out
}

/// One part binding a fixture writes: the row it sits in, the arm, the part's
/// position, and the row that part is taken to.
struct Bind {
    owner: &'static str,
    owner_row: &'static str,
    arm: Option<ArmKey>,
    index: usize,
    part: &'static str,
    part_row: &'static str,
}

/// Unfold one value from its row, with the part bindings stated explicitly.
fn unfolds_bound(
    sources: &[&str],
    rows: &[(&str, &str, Deconstructing)],
    binds: &[Bind],
    target: &str,
    target_row: &str,
) -> Result<Vec<String>, String> {
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
    let ty = |spelling: &str| {
        model
            .classify(&syn::parse_str(spelling).expect("test type"))
            .expect("a type the model accepts")
    };
    let mut recipes = Recipes::builder();
    let mut defaulted = std::collections::HashSet::new();
    for (name, row, shape) in rows {
        if defaulted.insert(*name) {
            recipes.declare_derived_default(ty(name), crate::recipe::Direction::Deconstruct);
        }
        recipes.declare(ty(name), RecipeName::new(*row), shape.clone());
    }
    let recipes = recipes.build(&model).expect("the rows build");
    let mut bound = crate::recipe::Bindings::builder();
    for bind in binds {
        let owner =
            crate::recipe::Crossing::new(ty(bind.owner), crate::recipe::Direction::Deconstruct);
        bound.bind(
            crate::recipe::Site::arm_part(
                &owner.row(RecipeName::new(bind.owner_row)),
                bind.arm,
                bind.index,
            ),
            crate::recipe::Crossing::new(ty(bind.part), crate::recipe::Direction::Deconstruct),
            crate::recipe::Ask::Recipe(RecipeName::new(bind.part_row)),
            crate::recipe::Origin::Adapter,
        );
    }
    let bindings = bound.build(&recipes).expect("bindings");
    Folding::new(&recipes, &model)
        .unfold(&Jni, &bindings, &ty(target), &RecipeName::new(target_row))
        .map(|(leaves, hoists)| render(&leaves, &hoists))
        .map_err(|e| e.to_string())
}

/// Unfold one value from its row.
fn unfolds(sources: &[&str], rows: &[(&str, Deconstructing)], target: &str) -> Vec<String> {
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
    let ty = |spelling: &str| {
        model
            .classify(&syn::parse_str(spelling).expect("test type"))
            .expect("a type the model accepts")
    };
    let mut recipes = Recipes::builder();
    for (name, row) in rows {
        // The value still crosses whole where nothing asks for its parts, so
        // the derived row stays the default beside the one being tested.
        recipes.declare_derived_default(ty(name), crate::recipe::Direction::Deconstruct);
        recipes.declare(ty(name), RecipeName::new("parts"), row.clone());
    }
    let recipes = recipes.build(&model).expect("the rows build");
    // A part is taken apart further only where a binding says so, and these
    // fixtures splice every part that has a row — so each row's parts are bound
    // to the `parts` row of their own type.
    let mut bound = crate::recipe::Bindings::builder();
    for (name, row) in rows {
        let owner = crate::recipe::Crossing::new(ty(name), crate::recipe::Direction::Deconstruct);
        let key = owner.row(RecipeName::new("parts"));
        let reaches = match row {
            Deconstructing::Product(Deconstruct::Fields(r))
            | Deconstructing::Product(Deconstruct::ValueForm { parts: r, .. }) => r.clone(),
            _ => Vec::new(),
        };
        for (index, reach) in reaches.iter().enumerate() {
            let Reach::Accessor(func) = reach else {
                continue;
            };
            let Some(ret) = model.function(func).map(|f| f.ret.clone()) else {
                continue;
            };
            let core = ret.optional_inner().unwrap_or(&ret);
            let core = core.borrow_target().unwrap_or(core);
            if recipes
                .key_of(
                    &crate::recipe::Crossing::new(
                        core.clone(),
                        crate::recipe::Direction::Deconstruct,
                    )
                    .key(),
                    &RecipeName::new("parts"),
                )
                .is_none()
            {
                continue;
            }
            bound.bind(
                crate::recipe::Site::arm_part(&key, None, index),
                crate::recipe::Crossing::new(core.clone(), crate::recipe::Direction::Deconstruct),
                crate::recipe::Ask::Recipe(RecipeName::new("parts")),
                crate::recipe::Origin::Adapter,
            );
        }
    }
    let bindings = bound.build(&recipes).expect("bindings");
    let (leaves, hoists) = Folding::new(&recipes, &model)
        .unfold(&Jni, &bindings, &ty(target), &RecipeName::new("parts"))
        .unwrap_or_else(|e| panic!("the row does not unfold: {e}"));
    render(&leaves, &hoists)
}

const SAMPLE: &str = "pub struct Sample { pub key: u32, pub payload: u64 }";

/// A product read off the value where it stands: one leaf per part, reached by
/// a field step.
#[test]
fn fields_read_where_the_value_stands() {
    let rows = vec![(
        "Sample",
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
    )];
    assert_eq!(
        unfolds(&[SAMPLE], &rows, "Sample"),
        [
            "key : u32 path=[key] identity=false nullable=false groups=[]",
            "payload : u64 path=[payload] identity=false nullable=false groups=[]",
        ]
    );
}

/// The value itself may be one of the parts, and it keeps the position the row
/// puts it in.
///
/// What goes last is its EMISSION — after every borrow taken off the value has
/// ended — and that is the emitter's ordering rather than the leaf list's. The
/// two were conflated here until the decomposition disagreed.
#[test]
fn the_value_itself_keeps_its_declared_position() {
    let rows = vec![(
        "Sample",
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity, Reach::Field(0)])),
    )];
    assert_eq!(
        unfolds(&[SAMPLE], &rows, "Sample"),
        [
            "handle : Sample path=[] identity=true nullable=false groups=[]",
            "key : u32 path=[key] identity=false nullable=false groups=[]",
        ]
    );
}

/// An accessor is called and its return is the leaf.
#[test]
fn an_accessor_is_one_leaf() {
    let rows = vec![(
        "Sample",
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
            "sample_key",
        ))])),
    )];
    assert_eq!(
        unfolds(
            &[SAMPLE, "pub fn sample_key(s: &Sample) -> u32 { todo!() }"],
            &rows,
            "Sample"
        ),
        ["sample_key : u32 path=[sample_key] identity=false nullable=false groups=[]"]
    );
}

/// An accessor whose return states a row of its own is spliced: the child's
/// leaves arrive under the parent's name.
#[test]
fn an_accessor_with_a_row_is_spliced() {
    let rows = vec![
        (
            "Holder",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
                "holder_sample",
            ))])),
        ),
        (
            "Sample",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        ),
    ];
    assert_eq!(
        unfolds(
            &[
                SAMPLE,
                "pub struct Holder { pub inner: u8 }",
                "pub fn holder_sample(h: &Holder) -> &Sample { todo!() }",
            ],
            &rows,
            "Holder"
        ),
        [
            "holder_sample__key : u32 path=[holder_sample.key] identity=false nullable=false \
             groups=[]",
            "holder_sample__payload : u64 path=[holder_sample.payload] identity=false \
             nullable=false groups=[]",
        ]
    );
}

/// A value form is called once and every part hangs off that one call, which is
/// what the hoist records.
#[test]
fn a_value_form_is_bound_once() {
    let rows = vec![(
        "Sample",
        Deconstructing::Product(Deconstruct::ValueForm {
            func: ident("sample_parts"),
            parts: vec![Reach::Field(0), Reach::Field(1)],
        }),
    )];
    assert_eq!(
        unfolds(
            &[
                SAMPLE,
                "pub struct SampleParts { pub a: u32, pub b: u64 }",
                "pub fn sample_parts(s: Sample) -> SampleParts { todo!() }",
            ],
            &rows,
            "Sample"
        ),
        [
            "a : u32 path=[sample_parts.a] identity=false nullable=false groups=[]",
            "b : u64 path=[sample_parts.b] identity=false nullable=false groups=[]",
            "hoist [sample_parts] consuming=true",
        ]
    );
}

/// A sum: the selector first, then one group per alternative. Every arm's
/// leaves are written and the selector says which of them are live.
#[test]
fn a_sum_writes_a_selector_and_every_arm() {
    let rows = vec![(
        "Reply",
        Deconstructing::Choice {
            arms: vec![
                Arm {
                    alternative: Some(0),
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
                Arm {
                    alternative: Some(1),
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
            ],
        },
    )];
    assert_eq!(
        unfolds(&["pub enum Reply { Ok(u32), Err(u64) }"], &rows, "Reply"),
        [
            "tag : Reply path=[] identity=false nullable=false groups=[]",
            "Ok__v0 : u32 path=[] identity=false nullable=false groups=[0]",
            "Err__v0 : u64 path=[] identity=false nullable=false groups=[1]",
        ]
    );
}

/// A part taken to a row, whose own part is taken to another: the second
/// binding is written against the row the first one landed in.
///
/// Carrying one root row name down the walk would look for the grandchild's
/// binding under the root's name and miss it, and `Inner` would cross whole.
/// `Compiler::part_of` keys the same part by the row it is actually at, so the
/// view has to as well.
#[test]
fn a_binding_is_keyed_by_the_row_its_part_landed_in() {
    let rows = &[
        (
            "Outer",
            "root",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
                "outer_middle",
            ))])),
        ),
        (
            "Middle",
            "middle_parts",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
                "middle_inner",
            ))])),
        ),
        (
            "Inner",
            "inner_parts",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        ),
    ];
    let sources = &[
        "pub struct Outer { pub a: u8 }",
        "pub struct Middle { pub b: u8 }",
        "pub struct Inner { pub deep: u32 }",
        "pub fn outer_middle(o: &Outer) -> &Middle { todo!() }",
        "pub fn middle_inner(m: &Middle) -> &Inner { todo!() }",
    ];
    let binds = &[
        Bind {
            owner: "Outer",
            owner_row: "root",
            arm: None,
            index: 0,
            part: "Middle",
            part_row: "middle_parts",
        },
        // Written against `middle_parts`, which is where the part above landed
        // — not against `root`.
        Bind {
            owner: "Middle",
            owner_row: "middle_parts",
            arm: None,
            index: 0,
            part: "Inner",
            part_row: "inner_parts",
        },
    ];
    assert_eq!(
        unfolds_bound(sources, rows, binds, "Outer", "root").expect("unfolds"),
        ["outer_middle__middle_inner__deep : u32 \
          path=[outer_middle.middle_inner.deep] identity=false nullable=false groups=[]"]
    );
}

/// A binding on one part of one ARM, which is the case `ArmKey` exists to tell
/// from a part of the product itself.
///
/// The payload's own leaves stay live only in that arm, exactly as the payload
/// would have been.
#[test]
fn a_binding_on_an_arms_part_is_followed() {
    let rows = &[
        (
            "Reply",
            "parts",
            Deconstructing::Choice {
                arms: vec![
                    Arm {
                        alternative: Some(0),
                        op: Deconstruct::Fields(vec![Reach::Field(0)]),
                    },
                    Arm {
                        alternative: Some(1),
                        op: Deconstruct::Fields(vec![Reach::Field(0)]),
                    },
                ],
            },
        ),
        (
            "Payload",
            "payload_parts",
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        ),
    ];
    let sources = &[
        "pub struct Payload { pub lo: u32, pub hi: u64 }",
        "pub enum Reply { Ok(Payload), Err(u64) }",
    ];
    let binds = &[Bind {
        owner: "Reply",
        owner_row: "parts",
        arm: Some(ArmKey::Alternative(0)),
        index: 0,
        part: "Payload",
        part_row: "payload_parts",
    }];
    assert_eq!(
        unfolds_bound(sources, rows, binds, "Reply", "parts").expect("unfolds"),
        [
            "tag : Reply path=[] identity=false nullable=false groups=[]",
            "Ok__v0__lo : u32 path=[lo] identity=false nullable=false groups=[0]",
            "Ok__v0__hi : u64 path=[hi] identity=false nullable=false groups=[0]",
            "Err__v0 : u64 path=[] identity=false nullable=false groups=[1]",
        ]
    );
}

/// A row that is WRONG is an error, not a shape the view has not reached: two
/// leaves of one name would be two foreign arguments the emitter cannot tell
/// apart, and a caller running this beside the older path must not skip it.
#[test]
fn a_duplicate_leaf_name_is_a_defect_not_a_deferral() {
    let rows = &[(
        "Sample",
        "parts",
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(0)])),
    )];
    let refusal =
        unfolds_bound(&[SAMPLE], rows, &[], "Sample", "parts").expect_err("two leaves of one name");
    assert!(refusal.contains("two of its leaves the name"), "{refusal}");
}
