//! The C-local resolved plan (#444 §5).
//!
//! Cbindgen decides how a value crosses by walking its `TypeRef` in
//! [`Cbindgen::lower_shape`](crate::Cbindgen::lower_shape) — and again,
//! separately, in `encode_value` and `output_is_fallible`. Three walks that
//! must agree about one structure.
//!
//! This module builds the same layout from the registry's semantic tree
//! instead: [`ordinary`] gives the plan of a crossing with no declared
//! decomposition, and [`select`] applies the rule Cbindgen already has — *a
//! declared conversion beats the shape, at every level* — as one pre-descent
//! decision rather than a test repeated at each arm.
//!
//! No production path consumes it yet, which is why it is allowed to be dead
//! code: it is compared against the existing walk on every call the test suite
//! makes ([`CbindgenBuilder::assert_plan_agrees`]), which is the differential
//! check #444 asks for before an adapter is switched over. The switch replaces
//! `lower_shape_walk` with `plan_shape` once the two agree everywhere — and the
//! exceptions below say exactly where they do not.
#![allow(dead_code)]

use prebindgen_registry::{
    transform::{Lowered, TransformLowerer},
    unfold::{
        ordinary_with, select, OrdinaryLayer, OutChoice, OutLeaf, OutNode, OutOfRust, OutProduct,
    },
    Conversions,
};

use crate::*;

impl CbindgenBuilder {
    /// The semantic plan of one C crossing: the value's arity layers, with
    /// every subtree Cbindgen converts directly already collapsed to a leaf.
    ///
    /// The claim is exactly `r_has_own_wire`: a type with a declared conversion
    /// crosses as that conversion says, whatever shape it would otherwise
    /// decompose into. Stating it once, before descending, is what the walk
    /// states at every arm.
    pub(crate) fn value_plan(&self, ty: &TypeRef, registry: &impl Conversions<()>) -> OutNode {
        let layers = ordinary_with(ty, &mut c_layer);
        select(&layers, &mut |node, _link| {
            r_has_own_wire(&node.ty, registry).then(|| node.ty.clone())
        })
    }

    /// Lower that plan to the C wire components, the way `lower_shape` does by
    /// walking the type.
    pub(crate) fn plan_shape(&self, ty: &TypeRef, registry: &impl Conversions<()>) -> ValueShape {
        let plan = self.value_plan(ty, registry);
        plan.lower(&mut ShapeFromPlan { registry })
            .expect("lowering a C value plan cannot fail")
    }
}

/// Lowers a semantic plan into the C ABI components of a present/ok value.
struct ShapeFromPlan<'a, R: Conversions<()>> {
    registry: &'a R,
}

impl<R: Conversions<()>> TransformLowerer<OutOfRust> for ShapeFromPlan<'_, R> {
    type Value = ValueShape;
    type Error = std::convert::Infallible;

    /// A value with its own wire, and the unit, are terminal: one component
    /// from the converter entry, or none at all.
    fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<ValueShape, Self::Error> {
        if matches!(node.ty.kind(), TypeKind::Unit) {
            return Ok(ValueShape {
                fields: vec![],
                niches: Niches::empty(),
            });
        }
        let entry = self.registry.output_entry(&node.ty).unwrap_or_else(|| {
            panic!("Cbindgen: type `{}` has no output converter", node.ty.key())
        });
        let wire = entry.destination.clone();
        // A pointer wire carries a free NULL niche unless the conversion
        // declared its own.
        let niches = if entry.niches.is_empty() && matches!(wire, syn::Type::Ptr(_)) {
            let null = null_for(&wire);
            Niches::one(syn::parse_quote!(#null), syn::parse_quote!(v.is_null()))
        } else {
            entry.niches.clone()
        };
        Ok(ValueShape {
            fields: vec![WireField { suffix: "", wire }],
            niches,
        })
    }

    /// A run is a malloc'd copy plus its length. The element must lower to ONE
    /// C value — a composite element has nothing for the array to hold.
    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &(),
        inner: &OutNode,
        _value: ValueShape,
    ) -> Result<ValueShape, Self::Error> {
        let entry = self.registry.output_entry(&inner.ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen: run element `{}` has no output converter",
                inner.ty.key()
            )
        });
        assert!(
            !marker_destination(&entry.destination),
            "Cbindgen: run element `{}` has no wire of its own, so there is nothing for the \
             array to hold — give it a `convert!` declaration or deliver its parts separately",
            inner.ty.key(),
        );
        let elem_wire = entry.destination.clone();
        Ok(ValueShape {
            fields: vec![
                WireField {
                    suffix: "",
                    wire: syn::parse_quote!(*mut #elem_wire),
                },
                WireField {
                    suffix: "_len",
                    wire: syn::parse_quote!(usize),
                },
            ],
            niches: Niches::empty(),
        })
    }

    /// An option spends one of the inner value's free niches; with none left it
    /// prepends an explicit `present` flag.
    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: ValueShape,
    ) -> Result<ValueShape, Self::Error> {
        if let Some((_slot, rest)) = value.niches.clone().carve() {
            return Ok(ValueShape {
                fields: value.fields,
                niches: rest,
            });
        }
        let mut fields = vec![WireField {
            suffix: "_present",
            wire: syn::parse_quote!(bool),
        }];
        fields.extend(value.fields);
        Ok(ValueShape {
            fields,
            niches: Niches::empty(),
        })
    }

    /// A C crossing with no declared decomposition has neither: `ordinary`
    /// builds layers over one leaf, and a claim replaces a subtree with a leaf.
    fn product(
        &mut self,
        node: &OutNode,
        _op: &OutProduct,
        _children: Lowered<'_, OutOfRust, ValueShape>,
    ) -> Result<ValueShape, Self::Error> {
        unreachable!(
            "a C value plan has no products: `{}` reached one",
            node.ty.key()
        )
    }

    fn choice(
        &mut self,
        node: &OutNode,
        _op: &OutChoice,
        _variants: Lowered<'_, OutOfRust, ValueShape>,
    ) -> Result<ValueShape, Self::Error> {
        unreachable!(
            "a C value plan has no choices: `{}` reached one",
            node.ty.key()
        )
    }
}

#[cfg(test)]
impl CbindgenBuilder {
    /// Assert the plan-built layout matches the walk's, on every call the test
    /// suite makes.
    ///
    /// The differential check #444 asks for while an adapter is migrated. It
    /// runs over the fixtures the C tests already have — real declarations with
    /// real converters — rather than a list of types chosen by whoever wrote
    /// the check, which is the list most likely to miss the case that differs.
    ///
    /// Compares the wire components and not the leftover niches, deliberately.
    /// A niche disagreement has exactly one visible consequence — whether an
    /// `Option` layer spends a niche or prepends a `_present` field — and that
    /// shows up as a field divergence here, because the check runs on the
    /// crossing type itself rather than on its core in isolation. What it would
    /// miss is two shapes that both run out of niches and emit `_present` while
    /// disagreeing about what a hypothetical outer layer would have had left;
    /// worth a `render_niches` side only once such a divergence is found.
    pub(crate) fn assert_plan_agrees(
        &self,
        ty: &TypeRef,
        registry: &impl Conversions<()>,
        walked: &ValueShape,
    ) {
        let render = |s: &ValueShape| -> Vec<String> {
            s.fields
                .iter()
                .map(|f| format!("{}:{}", f.suffix, quote::ToTokens::to_token_stream(&f.wire)))
                .collect()
        };
        let planned = render(&self.plan_shape(ty, registry));
        let walked = render(walked);

        assert_eq!(
            walked,
            planned,
            "#444: the plan-built layout of `{}` differs from the walk's",
            ty.key()
        );
    }
}

/// The arity layers **C's** boundary reads off a type, which is not the reading
/// the decomposition boundary uses (#444 §2).
///
/// Two differences, both because C spends a representation niche per layer
/// rather than having one way to say absent:
///
/// * `Option` peels all the way down, so `Option<Option<T>>` is two layers;
/// * a shared-slice borrow is a run, where the model stops at the reference.
fn c_layer(ty: &TypeRef) -> Option<(OrdinaryLayer, TypeRef)> {
    // A declared conversion beats the shape, so a type with its own wire is not
    // peeled at all — the same rule `select` applies one step later, asked here
    // because a layer peeled off a value that crosses whole would describe an
    // ABI the converter table does not hand out.
    if let Some(inner) = ty.optional_inner() {
        return Some((OrdinaryLayer::Optional, inner.clone()));
    }
    if let Some(elem) = r_cow_slice_elem(ty)
        .or_else(|| r_scalar_slice_elem(ty))
        .or_else(|| match ty.kind() {
            TypeKind::Vec(elem) => Some(elem),
            _ => None,
        })
    {
        return Some((OrdinaryLayer::Sequence, elem.clone()));
    }
    None
}
