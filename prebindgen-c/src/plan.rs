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

// ──────────────────────────────────────────────────────────────────────
// The callback plan (#447 §3)
// ──────────────────────────────────────────────────────────────────────

/// One declared callback's resolved boundary: how each argument crosses, what
/// `extern "C" fn` parameters that costs, and what the trampoline does with it.
///
/// The C closure struct's `call` pointer and the Rust trampoline that fires it
/// are two renderings of one boundary. They used to classify every argument
/// separately — the declaration to list its wire parameters, the dispatch to
/// fill them — agreeing because both walked the same declaration deterministically
/// rather than because they read one resolution. This is that resolution, built
/// once per callback and handed to both.
pub(crate) struct CCallbackPlan {
    /// One entry per declared argument, in signature order.
    pub(crate) args: Vec<CCallbackArg>,
}

impl CCallbackPlan {
    /// The `extern "C" fn` parameter types, in order — the `call` pointer's
    /// signature minus its trailing context pointer.
    pub(crate) fn wires(&self) -> impl Iterator<Item = &syn::Type> {
        self.args.iter().flat_map(|a| a.wires.iter())
    }

    /// Whether any argument's encoder calls `__cbg_alloc_array`, so the crate
    /// must carry that helper — a requirement of the plan, like every other
    /// consequence of the encoders it selected.
    pub(crate) fn needs_array_alloc(&self) -> bool {
        self.args.iter().any(|a| match &a.kind {
            CCallbackArgKind::Composite(plan) => plan.needs_array_alloc,
            _ => false,
        })
    }
}

/// One callback argument, resolved.
pub(crate) struct CCallbackArg {
    /// The Rust type the closure parameter is written with.
    pub(crate) src: syn::Type,
    /// The `extern "C" fn` parameters this argument contributes. A slice costs
    /// two, a decomposed composite one per wire field, everything else one.
    pub(crate) wires: Vec<syn::Type>,
    /// How it crosses, and what the trampoline needs to do it.
    pub(crate) kind: CCallbackArgKind,
}

/// How one callback argument reaches C.
pub(crate) enum CCallbackArgKind {
    /// A shared slice, delivered by reference as `(*const elem_wire, usize)` —
    /// zero-copy, so there is nothing to encode and nothing to drop.
    Slice { elem_wire: syn::Type },
    /// An owned handle the C side may take, delivered as `*mut wire`. Dropped
    /// after the call, which is a no-op if C took it.
    Takeable {
        conv: syn::Ident,
        opaque: syn::Type,
        fallible: bool,
    },
    /// A value with no wire of its own, decomposed into its shape's fields.
    Composite(CValuePlan),
    /// One value through its own converter.
    Single { conv: syn::Ident, fallible: bool },
}

impl CbindgenBuilder {
    /// The resolved plan for one declared callback, built once and shared.
    ///
    /// Stored rather than recomputed, because "both sides call the same
    /// function" only makes them agree while the function stays deterministic —
    /// which is a property of today's implementation, not of the boundary. One
    /// stored plan makes the declaration and the trampoline the same resolution
    /// by construction (#447 §3).
    ///
    /// Built from the declaration and the registry alone, so it does not depend
    /// on which emitter asks first. Entries only ever gain in the registry, so
    /// a plan built once its arguments resolved stays the answer.
    pub(crate) fn callback_plan(
        &self,
        key: &CallbackKey,
        registry: &impl Conversions<()>,
    ) -> std::rc::Rc<CCallbackPlan> {
        if let Some(hit) = self.callback_plans.borrow().get(key) {
            return hit.clone();
        }
        let plan = std::rc::Rc::new(self.build_callback_plan(key, registry));
        self.callback_plans
            .borrow_mut()
            .insert(key.clone(), plan.clone());
        plan
    }

    fn build_callback_plan(
        &self,
        key: &CallbackKey,
        registry: &impl Conversions<()>,
    ) -> CCallbackPlan {
        let cfg = self
            .callbacks
            .get(key)
            .expect("a callback plan is asked for by one of its own emitters");
        let mut args = Vec::new();
        for (i, declared) in cfg.args.iter().enumerate() {
            // The slice test reads the DECLARED spelling: a `.callback(...)`
            // argument is written by the build script, and a slice one may
            // never have been interned, so there is no reading to ask.
            if let Some((src_elem, elem_wire)) = self.callback_slice_elem_wire(declared) {
                args.push(CCallbackArg {
                    src: syn::parse_quote!(&[#src_elem]),
                    wires: vec![
                        syn::parse_quote!(*const #elem_wire),
                        syn::parse_quote!(usize),
                    ],
                    kind: CCallbackArgKind::Slice { elem_wire },
                });
                continue;
            }
            let reading = registry.reading_of(declared).unwrap_or_else(|| {
                panic!(
                    "Cbindgen: callback arg `{}` was never classified",
                    declared.to_token_stream()
                )
            });
            let entry = registry.output_entry(&reading).unwrap_or_else(|| {
                panic!(
                    "Cbindgen: callback arg `{}` has no output converter (declare it \
                     as a opaque_ptr/data_struct/enum_type)",
                    declared.to_token_stream()
                )
            });
            let src = self.src_ty_deep_of(&reading);
            let wire = entry.destination.clone();
            let conv = entry.function.sig.ident.clone();
            let fallible = returns_result(&entry.function.sig.output);
            let is_takeable = cfg.takeable.contains(&i);

            if is_takeable {
                args.push(CCallbackArg {
                    src,
                    wires: vec![syn::parse_quote!(*mut #wire)],
                    kind: CCallbackArgKind::Takeable {
                        conv,
                        opaque: wire,
                        fallible,
                    },
                });
                continue;
            }
            if self.callback_arg_is_composite(&reading, is_takeable, registry) {
                // Decomposed: the C params are the fields its shape lowers to,
                // each `MaybeUninit` so an absent value can leave its slot
                // unwritten without the wrapper materialising something to fill
                // it — and without leaving it indeterminate for a callee that
                // reads it anyway.
                let plan = self.c_value_plan(&reading, registry);
                let wires = plan
                    .shape
                    .fields
                    .iter()
                    .map(|f| {
                        let w = &f.wire;
                        syn::parse_quote!(::core::mem::MaybeUninit<#w>)
                    })
                    .collect();
                args.push(CCallbackArg {
                    src,
                    wires,
                    kind: CCallbackArgKind::Composite(plan),
                });
                continue;
            }
            // A marker converter with no structural lowering has no C ABI at
            // all — the one shape neither branch above can carry.
            assert!(
                !marker_destination(&entry.destination),
                "Cbindgen: callback argument `{}` has no C ABI — it resolves to a marker \
                 converter and is not one of the shapes lowered structurally (`Option<T>`, \
                 `Vec<T>`, `Cow<'_, [T]>`). Deliver its parts as separate callback \
                 arguments instead.",
                declared.to_token_stream(),
            );
            args.push(CCallbackArg {
                src,
                wires: vec![wire],
                kind: CCallbackArgKind::Single { conv, fallible },
            });
        }
        CCallbackPlan { args }
    }
}

/// The statement binding one callback argument's converted wire value.
///
/// A firing callback has no error channel, so a fallible converter aborts —
/// stated once here because both the takeable and the single-value arms need
/// exactly this and differ only in whether the binding is mutable.
pub(crate) fn convert_or_abort(
    conv: &syn::Ident,
    arg: &syn::Ident,
    wire: &syn::Ident,
    fallible: bool,
    mutable: bool,
) -> TokenStream {
    let mut_kw = if mutable { quote!(mut) } else { quote!() };
    if fallible {
        quote!(
            let #mut_kw #wire = match #conv(#arg) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                }
            };
        )
    } else {
        quote!(let #mut_kw #wire = #conv(#arg);)
    }
}
