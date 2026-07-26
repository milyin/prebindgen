use super::*;
use crate::{api::core::registry::Registry, SourceLocation};

/// Build a registry from `syn` item sources, then intern one root type.
fn graph_of(items: &[&str], root: &str) -> (ShapeGraph, SemanticUse) {
    let loc = SourceLocation::default();
    let parsed: Vec<(syn::Item, SourceLocation)> = items
        .iter()
        .map(|s| {
            (
                syn::parse_str::<syn::Item>(s).expect("test item"),
                loc.clone(),
            )
        })
        .collect();
    let registry = Registry::<()>::from_items(parsed).expect("index items");
    let ty: syn::Type = syn::parse_str(root).expect("test type");
    let mut graph = ShapeGraph::new();
    let root_use = graph.intern_root(&ty, &registry);
    (graph, root_use)
}

fn key(s: &str) -> TypeKey {
    TypeKey::parse(s).expect("test key")
}

// ── structure ───────────────────────────────────────────────────────────

/// A struct is a `Product` whose fields keep their `syn::Member` addressing —
/// named stays named, tuple stays indexed — and each field edge records how the
/// struct holds it.
#[test]
fn struct_is_a_product_keeping_member_addressing() {
    let (g, root) = graph_of(
        &["pub struct Rec { pub id: u64, pub label: String }"],
        "Rec",
    );
    assert_eq!(root.source, SourceUse::Value);
    let SemanticShape::Product { key: k, fields } = g.get(root.shape) else {
        panic!("expected a product, got {:?}", g.get(root.shape));
    };
    assert_eq!(k, &key("Rec"));
    assert_eq!(fields.len(), 2);
    assert!(matches!(&fields[0].member, syn::Member::Named(i) if i == "id"));
    assert!(matches!(&fields[1].member, syn::Member::Named(i) if i == "label"));
    assert_eq!(fields[0].diagnostic_name, "id");

    let (g2, root2) = graph_of(&["pub struct Pair(pub u64, pub u64);"], "Pair");
    let SemanticShape::Product { fields, .. } = g2.get(root2.shape) else {
        panic!("expected a product");
    };
    assert!(matches!(&fields[0].member, syn::Member::Unnamed(i) if i.index == 0));
    assert!(matches!(&fields[1].member, syn::Member::Unnamed(i) if i.index == 1));
}

/// **Every enum is a `Choice`, unit-only included.** A unit enum is the
/// degenerate sum whose variant groups are all empty — that is exactly the
/// tag-only lowering, and collapsing it here would be an adapter's decision
/// taken one tier too early.
#[test]
fn every_enum_is_a_choice_including_unit_only() {
    let (g, root) = graph_of(&["pub enum Op { Add, Sub, Mul }"], "Op");
    let SemanticShape::Choice { key: k, variants } = g.get(root.shape) else {
        panic!(
            "a unit enum must still be a Choice, got {:?}",
            g.get(root.shape)
        );
    };
    assert_eq!(k, &key("Op"));
    assert_eq!(variants.len(), 3);
    assert!(variants.iter().all(|v| v.fields.is_empty()));
    assert_eq!(
        variants.iter().map(|v| v.tag).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    // The classifier rides ON the choice rather than replacing it.
    assert_eq!(g.get(root.shape).enum_shape(), Some(EnumShape::Unit));
}

/// A payload enum is the same node kind with non-empty groups, and the
/// classifier is the only thing that changes.
#[test]
fn payload_enum_is_the_same_node_kind() {
    let (g, root) = graph_of(
        &["pub enum Shape { Empty, Circle(f64), Rect { w: f64, h: f64 } }"],
        "Shape",
    );
    let SemanticShape::Choice { variants, .. } = g.get(root.shape) else {
        panic!("expected a choice");
    };
    assert_eq!(g.get(root.shape).enum_shape(), Some(EnumShape::Sum));
    assert!(variants[0].fields.is_empty());
    assert!(matches!(&variants[1].fields[0].member, syn::Member::Unnamed(i) if i.index == 0));
    assert!(matches!(&variants[2].fields[0].member, syn::Member::Named(i) if i == "w"));
    // The variant ident stays the SOURCE spelling, independent of any
    // destination-language rename.
    assert_eq!(variants[2].ident, "Rect");
    // …and the nested-prefix leaf name comes from `SumSpec`, this tier's
    // Choice constructor, rather than being derived a second time here.
    assert_eq!(variants[2].fields[0].diagnostic_name, "rect_w");
}

// ── use-qualified edges ─────────────────────────────────────────────────

/// The reason edges are use-qualified at all: `Vec<&T>` is a container held by
/// value whose *elements* are shared-borrowed. A root-only crossing context
/// cannot say this — it has exactly one slot for a qualifier and two things to
/// qualify.
#[test]
fn vec_of_shared_refs_qualifies_container_and_element_separately() {
    let (g, root) = graph_of(&["pub struct Rec { pub id: u64 }"], "Vec<&Rec>");
    assert_eq!(root.source, SourceUse::Value);
    let SemanticShape::Sequence { kind, elem } = g.get(root.shape) else {
        panic!("expected a sequence");
    };
    assert_eq!(*kind, SequenceKind::Vec);
    assert_eq!(elem.source, SourceUse::SharedRef);
    assert!(matches!(g.get(elem.shape), SemanticShape::Product { .. }));
}

/// The three source use qualifiers, at the root.
#[test]
fn root_use_records_the_source_qualifier() {
    for (spelling, want) in [
        ("Rec", SourceUse::Value),
        ("&Rec", SourceUse::SharedRef),
        ("&mut Rec", SourceUse::ExclusiveRef),
    ] {
        let (g, root) = graph_of(&["pub struct Rec { pub id: u64 }"], spelling);
        assert_eq!(root.source, want, "for `{spelling}`");
        // All three name the SAME shape — the qualifier is on the edge, not a
        // distinct node, so `&Rec` and `Rec` share one interned product.
        assert_eq!(g.key(root.shape), &key("Rec"));
    }
}

/// `&[T]` peels its borrow onto the edge and becomes an ordinary sequence node,
/// so `&[T]`, `Vec<T>` and `Cow<'_, [T]>` share one traversal and differ only
/// by `SequenceKind`.
#[test]
fn sequence_kinds_are_distinguished_but_share_a_node_kind() {
    for (spelling, want_kind, want_use) in [
        ("Vec<u8>", SequenceKind::Vec, SourceUse::Value),
        ("&[u8]", SequenceKind::Slice, SourceUse::SharedRef),
        ("Cow<'a, [u8]>", SequenceKind::CowSlice, SourceUse::Value),
    ] {
        let (g, root) = graph_of(&[], spelling);
        assert_eq!(root.source, want_use, "for `{spelling}`");
        let SemanticShape::Sequence { kind, elem } = g.get(root.shape) else {
            panic!(
                "`{spelling}` must be a sequence, got {:?}",
                g.get(root.shape)
            );
        };
        assert_eq!(*kind, want_kind, "for `{spelling}`");
        assert_eq!(elem.source, SourceUse::Value);
    }
}

/// `Option<T>` is a layer, not a property of the thing it wraps.
#[test]
fn option_is_a_layer() {
    let (g, root) = graph_of(&["pub struct Rec { pub id: u64 }"], "Option<&Rec>");
    let SemanticShape::Optional(inner) = g.get(root.shape) else {
        panic!("expected an optional");
    };
    assert_eq!(inner.source, SourceUse::SharedRef);
    assert!(matches!(g.get(inner.shape), SemanticShape::Product { .. }));
}

/// `Box<T>` is heap indirection, which is a representation fact — so it is
/// transparent here and `Box<Rec>` interns to the same node `Rec` does.
#[test]
fn box_is_transparent() {
    let (g, root) = graph_of(&["pub struct Rec { pub id: u64 }"], "Option<Box<Rec>>");
    let SemanticShape::Optional(inner) = g.get(root.shape) else {
        panic!("expected an optional");
    };
    assert_eq!(g.key(inner.shape), &key("Rec"));
}

/// `Result<T, E>` is a two-alternative sum at the source. Both adapters route
/// it to an error channel, but that is a Tier 1 decision — modelling it as an
/// opaque leaf would hide `T` and `E` from the tier whose job is structure.
#[test]
fn result_is_an_ordinary_choice() {
    let (g, root) = graph_of(
        &[
            "pub struct Rec { pub id: u64 }",
            "pub struct Error { pub m: String }",
        ],
        "Result<Rec, Error>",
    );
    let SemanticShape::Choice { variants, .. } = g.get(root.shape) else {
        panic!("expected a choice, got {:?}", g.get(root.shape));
    };
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].ident, "Ok");
    assert_eq!(variants[1].ident, "Err");
    assert_eq!(g.key(variants[0].fields[0].uses.shape), &key("Rec"));
    assert_eq!(g.key(variants[1].fields[0].uses.shape), &key("Error"));

    // `Result<(), E>` has a unit Ok arm: a group that contributes nothing.
    let (g2, root2) = graph_of(&["pub struct Error { pub m: String }"], "Result<(), Error>");
    let SemanticShape::Choice { variants, .. } = g2.get(root2.shape) else {
        panic!("expected a choice");
    };
    assert!(variants[0].fields.is_empty());
    assert_eq!(variants[1].fields.len(), 1);
}

// ── interning and cycles ────────────────────────────────────────────────

/// Interning is keyed by the **full `TypeKey`**, never a bare ident — the
/// structural fix for #136.
///
/// This is a unit test on the graph, and deliberately not described as
/// end-to-end coverage of #136's collision: the source namespace is flat and
/// `check_no_duplicate` rejects duplicate idents across kinds, so two distinct
/// declared types sharing a short name cannot reach the capture pipeline at
/// all. What is asserted here is that the graph tells apart types that share a
/// tail ident — generic instantiations are the reachable form of that.
#[test]
fn interning_keys_on_the_full_type_key_not_a_bare_ident() {
    let (g, _) = graph_of(&[], "Vec<Vec<u8>>");
    // `Vec<Vec<u8>>`, `Vec<u8>` and `u8` are three nodes: keying on the tail
    // ident `Vec` would have collapsed the outer two into one and produced a
    // self-loop instead of a two-level sequence.
    assert!(g.id_of(&key("Vec<Vec<u8>>")).is_some());
    assert!(g.id_of(&key("Vec<u8>")).is_some());
    assert!(g.id_of(&key("u8")).is_some());
    assert_ne!(
        g.id_of(&key("Vec<Vec<u8>>")).unwrap(),
        g.id_of(&key("Vec<u8>")).unwrap()
    );
    assert_eq!(g.len(), 3);

    // Every id's key round-trips: nothing was interned under a shortened name.
    for id in g.ids() {
        assert_eq!(g.id_of(g.key(id)), Some(id));
    }
}

/// One type interned twice is one node, whatever route reached it.
#[test]
fn a_repeated_type_interns_once() {
    let (g, root) = graph_of(
        &[
            "pub struct Rec { pub a: Leaf, pub b: Leaf, pub c: Vec<Leaf> }",
            "pub struct Leaf { pub v: u64 }",
        ],
        "Rec",
    );
    let SemanticShape::Product { fields, .. } = g.get(root.shape) else {
        panic!("expected a product");
    };
    assert_eq!(fields[0].uses.shape, fields[1].uses.shape);
    let SemanticShape::Sequence { elem, .. } = g.get(fields[2].uses.shape) else {
        panic!("expected a sequence");
    };
    assert_eq!(elem.shape, fields[0].uses.shape);
}

/// Self-recursion interns **finitely**, with a `ShapeId` back-edge to the
/// `Node` node itself — asserted structurally, not inferred from "it
/// terminated".
#[test]
fn self_recursion_interns_finitely_with_a_back_edge() {
    let (g, root) = graph_of(
        &["pub struct Node { pub id: u64, pub children: Vec<Node> }"],
        "Node",
    );
    let node_id = root.shape;
    let SemanticShape::Product { fields, .. } = g.get(node_id) else {
        panic!("expected a product");
    };
    let SemanticShape::Sequence { elem, .. } = g.get(fields[1].uses.shape) else {
        panic!("expected a sequence");
    };
    // The back-edge is exactly the `Node` id, not a fresh copy of it.
    assert_eq!(elem.shape, node_id);
    assert_eq!(elem.source, SourceUse::Value);
    // Node, u64, Vec<Node> — and nothing else.
    assert_eq!(g.len(), 3);
    assert!(g.is_recursive(node_id));
    assert!(!g.is_recursive(g.id_of(&key("u64")).unwrap()));
}

/// Mutual recursion through legal indirection yields **two nodes and two
/// back-edges**, not divergence.
///
/// `A { b: B }` / `B { a: A }` is infinitely sized and is not Rust, so the
/// fixture goes through `Vec`, which is the indirection a captured source crate
/// can actually declare.
#[test]
fn mutual_recursion_yields_two_nodes_and_two_back_edges() {
    let (g, root) = graph_of(
        &[
            "pub struct A { pub bs: Vec<B> }",
            "pub struct B { pub as_: Vec<A> }",
        ],
        "A",
    );
    let a_id = root.shape;
    let b_id = g.id_of(&key("B")).expect("B interned");
    assert_ne!(a_id, b_id);

    let elem_of = |id: ShapeId| {
        let SemanticShape::Product { fields, .. } = g.get(id) else {
            panic!("expected a product");
        };
        let SemanticShape::Sequence { elem, .. } = g.get(fields[0].uses.shape) else {
            panic!("expected a sequence");
        };
        *elem
    };
    // A → Vec<B> → B, and B → Vec<A> → A: both back-edges land on the exact
    // expected ids.
    assert_eq!(elem_of(a_id).shape, b_id);
    assert_eq!(elem_of(b_id).shape, a_id);

    // A, Vec<B>, B, Vec<A> — four nodes, finite.
    assert_eq!(g.len(), 4);
    assert!(g.is_recursive(a_id));
    assert!(g.is_recursive(b_id));
}

/// Recursion reached through a borrow keeps the borrow on the edge, so a cycle
/// does not silently become a by-value one.
#[test]
fn a_back_edge_keeps_its_use_qualifier() {
    let (g, root) = graph_of(
        &["pub struct Node { pub kids: Vec<&'static Node> }"],
        "Node",
    );
    let SemanticShape::Product { fields, .. } = g.get(root.shape) else {
        panic!("expected a product");
    };
    let SemanticShape::Sequence { elem, .. } = g.get(fields[0].uses.shape) else {
        panic!("expected a sequence");
    };
    assert_eq!(elem.shape, root.shape);
    assert_eq!(elem.source, SourceUse::SharedRef);
}

/// An enum can recur too, and the cycle runs through a variant payload.
#[test]
fn recursive_choice_interns_finitely() {
    let (g, root) = graph_of(&["pub enum Tree { Leaf(u64), Branch(Vec<Tree>) }"], "Tree");
    let SemanticShape::Choice { variants, .. } = g.get(root.shape) else {
        panic!("expected a choice");
    };
    let SemanticShape::Sequence { elem, .. } = g.get(variants[1].fields[0].uses.shape) else {
        panic!("expected a sequence");
    };
    assert_eq!(elem.shape, root.shape);
    assert!(g.is_recursive(root.shape));
}

/// An unindexed type is a `Leaf` — a statement about this graph's knowledge,
/// not about the type. What separates an opaque handle from an `i32` is a
/// Tier 1 declaration.
#[test]
fn unindexed_types_and_scalars_are_leaves() {
    let (g, root) = graph_of(&[], "Undeclared");
    assert!(matches!(g.get(root.shape), SemanticShape::Leaf(k) if k == &key("Undeclared")));
    let (g2, root2) = graph_of(&[], "i32");
    assert!(matches!(g2.get(root2.shape), SemanticShape::Leaf(_)));
}

/// `children()` is the one uniform edge accessor, so a traversal cannot forget
/// a variant.
#[test]
fn children_covers_every_node_kind() {
    let (g, root) = graph_of(
        &[
            "pub struct Rec { pub a: u64, pub b: Option<Vec<Op>> }",
            "pub enum Op { Add, Pair(u64, u64) }",
        ],
        "Rec",
    );
    // Product: one edge per field.
    assert_eq!(g.get(root.shape).children().len(), 2);
    // Choice: one edge per payload slot, flattened across variants.
    let op = g.id_of(&key("Op")).expect("Op interned");
    assert_eq!(g.get(op).children().len(), 2);
    // Leaf: none. Layers: exactly one.
    let u64_id = g.id_of(&key("u64")).unwrap();
    assert!(g.get(u64_id).children().is_empty());
    assert_eq!(g.get(g.id_of(&key("Vec<Op>")).unwrap()).children().len(), 1);
    assert_eq!(
        g.get(g.id_of(&key("Option<Vec<Op>>")).unwrap())
            .children()
            .len(),
        1
    );
}

// ── the tier boundary ───────────────────────────────────────────────────

/// **Tier 0 may not name a JVM descriptor, a C ABI type, a Kotlin class, or a
/// delivery protocol.**
///
/// Rust's module system cannot express "no adapter concept is reachable from
/// here" — `semantic.rs` could import from `api::lang` and still compile. So
/// the boundary is checked against the module's own text, which is crude but is
/// the only form of this check that can actually fail.
#[test]
fn no_adapter_policy_is_reachable_from_tier_0() {
    let src = include_str!("../semantic.rs");
    // Strip comments: the module *documents* what it may not name, and those
    // sentences are the point rather than a violation.
    let code: String = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n");

    for forbidden in [
        // adapters and their modules
        "lang::",
        "jnigen",
        "cbindgen",
        "JniGen",
        "Cbindgen",
        // JVM / JNI vocabulary
        "jlong",
        "jint",
        "JObject",
        "JNIEnv",
        "descriptor",
        "Kotlin",
        "kt_",
        // C ABI vocabulary
        "c_char",
        "c_int",
        "repr(C)",
        "MaybeUninit",
        "extern \"C\"",
        // delivery / wire policy
        "Wire",
        "wire",
        "Delivery",
        "delivery",
    ] {
        assert!(
            !code.contains(forbidden),
            "Tier 0 must not name adapter policy, found `{forbidden}` in semantic.rs"
        );
    }

    // …and its imports stay inside core.
    for line in code.lines().filter(|l| l.trim_start().starts_with("use ")) {
        assert!(
            line.contains("std::")
                || line.contains("super::")
                || line.contains("syn")
                || line.contains("crate::api::core"),
            "Tier 0 import escapes core: {line}"
        );
    }
}

// ── the `Shape<N>` decision ─────────────────────────────────────────────

/// `Shape<N>` survives Tier 0 as a **derived view**, not a second model: the
/// layer stack is semantic and is owned here, while `N` — the
/// null-representation choice — is a wire fact that belongs to Tier 1.
///
/// So the engines that fold over layers keep their algebra and stop deriving
/// the stack themselves.
#[test]
fn wrapper_projection_derives_the_shape_stack() {
    use crate::api::core::shape::Shape;

    let (g, root) = graph_of(
        &["pub struct Rec { pub id: u64 }"],
        "Option<Vec<Option<Rec>>>",
    );
    let projected = g.wrapper_projection(root.shape);
    // Outside in: Optional, Iterable, Optional, then the base.
    let Shape::Optional((), l1) = &projected else {
        panic!("expected an optional, got {projected:?}");
    };
    let Shape::Iterable(l2) = &**l1 else {
        panic!("expected an iterable");
    };
    let Shape::Optional((), l3) = &**l2 else {
        panic!("expected an optional");
    };
    assert!(matches!(**l3, Shape::Base));
    assert!(projected.has_iterable_layer());

    // …and the base the stack bottoms out at is the product itself.
    assert_eq!(g.key(g.base_of(root.shape)), &key("Rec"));

    // A shape with no layers projects to a bare base.
    let (g2, root2) = graph_of(&["pub struct Rec { pub id: u64 }"], "&Rec");
    assert!(matches!(g2.wrapper_projection(root2.shape), Shape::Base));
    assert_eq!(g2.base_of(root2.shape), root2.shape);
}
