//! What a row says a value comes apart into.
//!
//! Each test declares a deconstructing row, unfolds it, and states the leaves
//! it expects by name, in order, with what each carries — the same shape the
//! constructing tests beside it take, because the two answer one question in
//! opposite directions.

use prebindgen_flat::flat::{ScalarKind, TypeRef};

use super::*;
use crate::recipe::{Arm, Deconstructing, Recipes};

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

    fn part_name(&self, reach: &Reach, index: usize) -> String {
        match reach {
            Reach::Accessor(func) => func.to_string(),
            _ => format!("f{index}"),
        }
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
    let (leaves, hoists) = Folding::new(&recipes, &model)
        .unfold(&Jni, &ty(target), &RecipeName::new("parts"))
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
            "f0 : u32 path=[key] identity=false nullable=false groups=[]",
            "f1 : u64 path=[payload] identity=false nullable=false groups=[]",
        ]
    );
}

/// The value itself is one of the parts, and it goes last — after every borrow
/// taken off it has ended.
#[test]
fn the_value_itself_is_the_last_part() {
    let rows = vec![(
        "Sample",
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity, Reach::Field(0)])),
    )];
    assert_eq!(
        unfolds(&[SAMPLE], &rows, "Sample"),
        [
            "f1 : u32 path=[key] identity=false nullable=false groups=[]",
            "handle : Sample path=[] identity=true nullable=false groups=[]",
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
            "holder_sample__f0 : u32 path=[holder_sample.key] identity=false nullable=false \
             groups=[]",
            "holder_sample__f1 : u64 path=[holder_sample.payload] identity=false nullable=false \
             groups=[]",
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
            "f0 : u32 path=[sample_parts.a] identity=false nullable=false groups=[]",
            "f1 : u64 path=[sample_parts.b] identity=false nullable=false groups=[]",
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
            "Ok__f0 : u32 path=[] identity=false nullable=false groups=[0]",
            "Err__f0 : u64 path=[] identity=false nullable=false groups=[1]",
        ]
    );
}
