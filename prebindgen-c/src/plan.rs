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
//! `lower_shape`, `encode_value` and `output_is_fallible` are three readings of
//! one [`CValuePlan`] now, so the walks they used to be are gone. They were
//! switched over only after a differential check showed them agreeing on every
//! call the whole C suite makes — layout, fallibility, and encoding
//! token-for-token.

use prebindgen_registry::{
    transform::{Lowered, TransformLowerer},
    unfold::{
        ordinary_with, select, OrdinaryLayer, OutChoice, OutLeaf, OutNode, OutOfRust, OutProduct,
        OutRun,
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
    // Shape only: this reads layers off the type and never consults the
    // converter table. Peeling a value that in fact crosses whole is harmless
    // because `ordinary_with` leaves the wrapped type on the layer node, so
    // `select` claims it there and the layers below it are discarded — the
    // "declared conversion beats the shape" rule lives in that claim, not here.
    if let Some(inner) = ty.optional_inner() {
        return Some((OrdinaryLayer::Optional, inner.clone()));
    }
    if let Some((elem, borrowed)) = c_run(ty) {
        return Some((OrdinaryLayer::Sequence { borrowed }, elem.clone()));
    }
    None
}

/// The element of a run C carries as a pointer-and-length pair, and whether it
/// is read **through a borrow** — a shared slice is copied out of, an owned
/// collection is consumed.
///
/// Asked once, where the layer policy decides a run is a layer at all. What it
/// answers travels on the node as [`OutRun::borrowed`], so the lowering reads a
/// plan fact rather than classifying the type a second time.
fn c_run(ty: &TypeRef) -> Option<(&TypeRef, bool)> {
    if let Some(elem) = r_cow_slice_elem(ty).or_else(|| r_scalar_slice_elem(ty)) {
        return Some((elem, true));
    }
    match ty.kind() {
        TypeKind::Vec(elem) => Some((elem, false)),
        _ => None,
    }
}

/// One C crossing, resolved: what it looks like on the wire, whether encoding
/// it can fail, and how to encode it.
///
/// The three used to be three walks over the same `TypeRef` that had to agree —
/// and `encode_value` called `lower_shape` mid-walk to find out where its
/// targets were, so they were not even independent. Produced together from one
/// pass over the semantic plan, they agree by construction.
pub(crate) struct CValuePlan {
    pub(crate) shape: ValueShape,
    pub(crate) fallible: bool,
    /// Whether this plan's encoder calls `__cbg_alloc_array`, so the crate must
    /// carry that helper's definition.
    ///
    /// A helper requirement is a fact of the resolved plan, not of the source
    /// syntax: the encoder that calls it was chosen here, so what it needs is
    /// answered here too. Read off the same plans the emitters consume, a
    /// helper cannot be called by generated code the prelude does not define.
    pub(crate) needs_array_alloc: bool,
    encode: Encoder,
}

/// Given the source expression and the lvalues its components are written into,
/// the statements that do it. A function rather than a token stream because the
/// value and the targets are chosen by whatever encloses a node — a run binds
/// each element, an option binds its payload — and only the enclosing node
/// knows them.
type Encoder = std::rc::Rc<dyn Fn(&TokenStream, &[TokenStream], &ErrRoute<'_>) -> TokenStream>;

impl CValuePlan {
    /// Emit the statements writing `val` into `targets`.
    pub(crate) fn encode(
        &self,
        val: &TokenStream,
        targets: &[TokenStream],
        route: &ErrRoute<'_>,
    ) -> TokenStream {
        (self.encode)(val, targets, route)
    }
}

impl CbindgenBuilder {
    /// Resolve one C crossing from its semantic plan.
    pub(crate) fn c_value_plan(&self, ty: &TypeRef, registry: &impl Conversions<()>) -> CValuePlan {
        self.value_plan(ty, registry)
            .lower(&mut PlanFromTree { registry })
            .expect("resolving a C value plan cannot fail")
    }
}

/// Lowers a semantic plan into the resolved C plan.
struct PlanFromTree<'a, R: Conversions<()>> {
    registry: &'a R,
}

impl<R: Conversions<()>> TransformLowerer<OutOfRust> for PlanFromTree<'_, R> {
    type Value = CValuePlan;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<CValuePlan, Self::Error> {
        if matches!(node.ty.kind(), TypeKind::Unit) {
            return Ok(CValuePlan {
                shape: ValueShape {
                    fields: vec![],
                    niches: Niches::empty(),
                },
                fallible: false,
                needs_array_alloc: false,
                encode: std::rc::Rc::new(|_, _, _| quote!()),
            });
        }
        let entry = self.registry.output_entry(&node.ty).unwrap_or_else(|| {
            panic!("Cbindgen: type `{}` has no output converter", node.ty.key())
        });
        let wire = entry.destination.clone();
        let niches = if entry.niches.is_empty() && matches!(wire, syn::Type::Ptr(_)) {
            let null = null_for(&wire);
            Niches::one(syn::parse_quote!(#null), syn::parse_quote!(v.is_null()))
        } else {
            entry.niches.clone()
        };
        let conv = entry.function.sig.ident.clone();
        let fallible = returns_result(&entry.function.sig.output);
        Ok(CValuePlan {
            shape: ValueShape {
                fields: vec![WireField { suffix: "", wire }],
                niches,
            },
            fallible,
            needs_array_alloc: false,
            encode: std::rc::Rc::new(move |val, targets, route| {
                let t0 = &targets[0];
                if fallible {
                    let converted = route_result(quote!(#conv(#val)), route);
                    quote!( #t0 = #converted; )
                } else {
                    quote!( #t0 = #conv(#val); )
                }
            }),
        })
    }

    /// A run is a malloc'd copy plus its length, its elements converted one by
    /// one. The element's own plan is not used: an array holds one C value per
    /// element, so the element converter is called directly.
    fn sequence(
        &mut self,
        _node: &OutNode,
        op: &OutRun,
        inner: &OutNode,
        _value: CValuePlan,
    ) -> Result<CValuePlan, Self::Error> {
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
        let elem_conv = entry.function.sig.ident.clone();
        let elem_map = map_arg(&elem_conv, entry.function.sig.unsafety.is_some());
        let fallible = returns_result(&entry.function.sig.output);
        // A plan fact, not a type test: the layer policy decided this when it
        // read the run off the type, and the node says so.
        let borrowed = op.borrowed;
        Ok(CValuePlan {
            shape: ValueShape {
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
            },
            fallible,
            // The run's own encoder is what calls the helper.
            needs_array_alloc: true,
            encode: std::rc::Rc::new(move |val, targets, route| {
                let t_ptr = &targets[0];
                let t_len = &targets[1];
                let source = if borrowed {
                    quote!(#val.iter().copied())
                } else {
                    quote!(#val)
                };
                if fallible {
                    let converted = route_result(quote!(#elem_conv(__value)), route);
                    quote!(
                        let mut __arr: ::std::vec::Vec<#elem_wire> = ::std::vec::Vec::new();
                        for __value in #source {
                            __arr.push(#converted);
                        }
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    )
                } else {
                    let mapped = if borrowed {
                        quote!(#val.iter().copied().map(#elem_map))
                    } else {
                        quote!(#val.into_iter().map(#elem_map))
                    };
                    quote!(
                        let __arr: ::std::vec::Vec<#elem_wire> = #mapped.collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    )
                }
            }),
        })
    }

    /// An option spends one of the inner value's free niches, or prepends a
    /// `present` flag. Which one it does decides where the inner value's
    /// targets start — the reason the two used to have to agree across walks.
    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: CValuePlan,
    ) -> Result<CValuePlan, Self::Error> {
        let inner_encode = value.encode.clone();
        let fallible = value.fallible;
        // A layer encodes through its inner value, so it needs whatever that
        // encoder needs.
        let needs_array_alloc = value.needs_array_alloc;
        if let Some((slot, rest)) = value.shape.niches.clone().carve() {
            let null = slot.value.clone();
            return Ok(CValuePlan {
                shape: ValueShape {
                    fields: value.shape.fields,
                    niches: rest,
                },
                fallible,
                needs_array_alloc,
                encode: std::rc::Rc::new(move |val, targets, route| {
                    // `None` reuses the next inner niche; `Some` encodes inline.
                    let inner_enc = inner_encode(&quote!(__x), targets, route);
                    let t0 = &targets[0];
                    quote!(
                        match #val {
                            ::core::option::Option::Some(__x) => { #inner_enc }
                            ::core::option::Option::None => { #t0 = #null; }
                        }
                    )
                }),
            });
        }
        let mut fields = vec![WireField {
            suffix: "_present",
            wire: syn::parse_quote!(bool),
        }];
        fields.extend(value.shape.fields);
        Ok(CValuePlan {
            shape: ValueShape {
                fields,
                niches: Niches::empty(),
            },
            fallible,
            needs_array_alloc,
            encode: std::rc::Rc::new(move |val, targets, route| {
                // Explicit `present` flag first; the inner value follows it.
                let present = &targets[0];
                let inner_enc = inner_encode(&quote!(__x), &targets[1..], route);
                quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => { #present = true; #inner_enc }
                        ::core::option::Option::None => { #present = false; }
                    }
                )
            }),
        })
    }

    fn product(
        &mut self,
        node: &OutNode,
        _op: &OutProduct,
        _children: Lowered<'_, OutOfRust, CValuePlan>,
    ) -> Result<CValuePlan, Self::Error> {
        unreachable!(
            "a C value plan has no products: `{}` reached one",
            node.ty.key()
        )
    }

    fn choice(
        &mut self,
        node: &OutNode,
        _op: &OutChoice,
        _variants: Lowered<'_, OutOfRust, CValuePlan>,
    ) -> Result<CValuePlan, Self::Error> {
        unreachable!(
            "a C value plan has no choices: `{}` reached one",
            node.ty.key()
        )
    }
}
