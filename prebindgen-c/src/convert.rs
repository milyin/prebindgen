use prebindgen_registry::Conversions;

use super::*;

impl CbindgenBuilder {
    pub(crate) fn prereq_domain_constants(&self, registry: &Registry) -> Vec<syn::Item> {
        let mut items = Vec::new();
        for decl in &self.convert_decls {
            let Some(domain) = decl.domain() else {
                continue;
            };
            let demand = [Direction::Construct, Direction::Deconstruct]
                .into_iter()
                .flat_map(|direction| registry.readings(direction))
                .map(|subject| option_depth(subject, decl.key()))
                .max()
                .unwrap_or(0);
            let ty = domain.ty();
            let base = self
                .convert_bases
                .get(decl.key())
                .cloned()
                .unwrap_or_else(|| {
                    let short = type_short(&decl.rust_type().key().clone());
                    self.mangle_rust_type
                        .as_ref()
                        .map(|m| m(&short))
                        .unwrap_or_else(|| snake_case(&short))
                })
                .to_ascii_uppercase();
            for (index, value) in domain
                .niche_values(demand.saturating_add(8))
                .into_iter()
                .filter_map(prebindgen_registry::ScalarValue::portable_expr)
                .take(demand)
                .enumerate()
            {
                let name = format_ident!("{}_NICHE_{}", base, index);
                items.push(syn::parse_quote!(
                    #[doc = "Reserved representation value used by generated sum-type ABIs."]
                    pub const #name: #ty = #value;
                ));
                if index == 0 {
                    let none = format_ident!("{}_NONE", base);
                    items.push(syn::parse_quote!(
                        #[doc = "Representation of None for the first optional layer."]
                        pub const #none: #ty = #value;
                    ));
                }
            }
        }
        items
    }

    pub(crate) fn custom_plan(
        &self,
        ty: &TypeRef,
        registry: &impl Conversions,
        direction: Direction,
    ) -> Option<(crate::chain::CustomPlan, Niches)> {
        let key = ty.key();
        let decl = self.convert_decls.iter().find(|d| *d.key() == key)?;
        let spec = match direction {
            Direction::Construct => decl.input_spec().as_ref()?,
            Direction::Deconstruct => decl.output_spec().as_ref()?,
        };
        let (repr, operation) = self.custom_operation(decl, spec, registry, direction);
        assert!(
            is_scalar(&repr),
            "Cbindgen custom representations must be C scalar types"
        );
        if let Some(domain) = decl.domain() {
            assert_eq!(
                TypeKey::from_type(domain.ty()),
                TypeKey::from_type(&repr),
                "Cbindgen conversion domain type does not match its {} representation",
                match direction {
                    Direction::Construct => "input",
                    Direction::Deconstruct => "output",
                }
            );
        }
        let ident = match direction {
            Direction::Construct => Self::in_name_of(&key),
            Direction::Deconstruct => Self::out_name_of(&key),
        };
        let valid = decl
            .domain()
            .as_ref()
            .map(|domain| match direction {
                Direction::Construct => domain.contains_expr(quote!(v)),
                Direction::Deconstruct => domain.contains_expr(quote!(__repr)),
            })
            .map(|tokens| {
                syn::parse2(tokens).expect("a representation-domain predicate is an expression")
            });
        let invalid_message = format!("{} representation is outside its declared domain", key);
        let niches = self.c_domain_niches(decl, registry, direction);
        Some((
            crate::chain::CustomPlan {
                ident,
                source: ty.clone(),
                source_module: self.source_module.clone(),
                wire: repr,
                direction,
                operation,
                valid,
                invalid_message,
            },
            niches,
        ))
    }

    fn custom_operation(
        &self,
        decl: &ConvertDecl,
        spec: &ConvertSpec,
        registry: &impl Conversions,
        direction: Direction,
    ) -> (syn::Type, crate::chain::CustomOperation) {
        match spec {
            ConvertSpec::PrebindgenFn(f) => {
                let item = registry
                    .flat()
                    .function(&f)
                    .unwrap_or_else(|| panic!("Cbindgen conversion function {} was not found", f));
                let (repr, by_ref, fallible) = match direction {
                    Direction::Construct => {
                        let (repr, by_ref) = one_param(item);
                        let (ok, fallible) = match item.ret.fallible_parts() {
                            Some((ok, _)) => (ok, true),
                            None => (&item.ret, false),
                        };
                        assert_eq!(ok.key(), *decl.key());
                        (repr, by_ref, fallible)
                    }
                    Direction::Deconstruct => {
                        let (param, by_ref) = one_param(item);
                        assert_eq!(param.key(), *decl.key());
                        let (repr, fallible) = match item.ret.fallible_parts() {
                            Some((ok, _)) => (ok, true),
                            None => (&item.ret, false),
                        };
                        (repr, by_ref, fallible)
                    }
                };
                let repr = scalar_ty(repr).unwrap_or_else(|| {
                    panic!("Cbindgen custom representations must be C scalar types")
                });
                let path = self.conversion_fn_path(registry, f);
                (
                    repr,
                    crate::chain::CustomOperation::Function {
                        path,
                        by_ref,
                        fallible,
                    },
                )
            }
            ConvertSpec::Trait { repr, fallible } => (
                repr.clone(),
                crate::chain::CustomOperation::Trait {
                    fallible: *fallible,
                },
            ),
        }
    }

    fn c_domain_niches(
        &self,
        decl: &ConvertDecl,
        registry: &impl Conversions,
        direction: Direction,
    ) -> Niches {
        let Some(domain) = decl.domain() else {
            return Niches::empty();
        };
        // A crossing with no reading contributes no demand, and that is an
        // answer rather than a gap being swallowed: the niche allocator is
        // reserving values no SIBLING CONVERSION can produce, and a crossing
        // the registry never entered has no conversion to produce one. Spelled
        // as an explicit `0` — the same answer the JNI adapter's twin gives — so the
        // reasoning is in the code instead of in a claim that a `filter_map`
        // silently relied on.
        let demand = registry
            .crossing_keys(direction)
            .iter()
            .map(|candidate| {
                registry
                    .reading(candidate)
                    .map_or(0, |reading| option_depth(&reading, decl.key()))
            })
            .max()
            .unwrap_or(0);
        Niches::from_slots(
            domain
                .niche_values(demand.saturating_add(8))
                .into_iter()
                .filter_map(|value| value.portable_expr().map(|literal| (value, literal)))
                .take(demand)
                .map(|(value, literal)| {
                    let matches = match value {
                        prebindgen_registry::ScalarValue::F32(bits) => {
                            syn::parse_quote!(v.to_bits() == #bits)
                        }
                        prebindgen_registry::ScalarValue::F64(bits) => {
                            syn::parse_quote!(v.to_bits() == #bits)
                        }
                        _ => syn::parse_quote!(v == #literal),
                    };
                    NicheSlot {
                        value: literal,
                        matches,
                    }
                }),
        )
    }

    fn conversion_fn_path(&self, registry: &impl Conversions, ident: &syn::Ident) -> syn::Path {
        let Some(mut module) = registry.origin_module(ident) else {
            return self.src_fn(ident);
        };
        module.segments.push(syn::PathSegment::from(ident.clone()));
        module
    }
}

/// The single parameter of a conversion fn, peeled of a leading `&`.
///
/// Off the ELEMENT: a signature is a parameter list, and the borrow is
/// `TypeKind::Ref` — where this filtered `syn::FnArg::Typed` and matched
/// `syn::Type::Reference` to reach the same two facts.
fn one_param(f: &prebindgen_registry::flat::Function) -> (&TypeRef, bool) {
    assert_eq!(
        f.params.len(),
        1,
        "conversion functions take exactly one parameter"
    );
    let ty = &f.params[0].ty;
    match ty.kind() {
        TypeKind::Ref { inner, .. } => (inner, true),
        _ => (ty, false),
    }
}

/// How many `Option<…>` layers `candidate` puts over `target`, or 0 if it is
/// not that type under any number of them.
///
/// Counted off the **reading**. The optional layers are already what the model
/// says this type is — `TypeKind::Optional` is produced for exactly `Option<T>`
/// — so peeling tokens to rediscover them was re-deriving the classification
/// the registry stored (#291).
fn option_depth(candidate: &prebindgen_registry::flat::TypeRef, target: &TypeKey) -> usize {
    let mut reading = candidate;
    let mut depth = 0;
    while let Some(inner) = reading.optional_inner() {
        reading = inner;
        depth += 1;
    }
    if reading.key() == *target {
        depth
    } else {
        0
    }
}
