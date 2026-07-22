use super::*;

impl Cbindgen {
    pub(crate) fn prereq_domain_constants(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items = Vec::new();
        for decl in &self.convert_decls {
            let Some(domain) = &decl.domain else { continue };
            let demand = [Direction::Input, Direction::Output]
                .into_iter()
                .flat_map(|direction| registry.type_table(direction).keys())
                .map(|candidate| option_depth(candidate, &decl.key))
                .max()
                .unwrap_or(0);
            let ty = domain.ty();
            let base = self
                .convert_bases
                .get(&decl.key)
                .cloned()
                .unwrap_or_else(|| {
                    let short = type_short(&decl.key.to_type());
                    self.mangle_rust_type
                        .as_ref()
                        .map(|m| m(&short))
                        .unwrap_or_else(|| snake_case(&short))
                })
                .to_ascii_uppercase();
            for (index, value) in domain
                .niche_values(demand.saturating_add(8))
                .into_iter()
                .filter_map(crate::core::ScalarValue::portable_expr)
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

    pub(crate) fn in_custom(
        &self,
        ty: &syn::Type,
        registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        let decl = self.convert_decls.iter().find(|d| d.key == key)?;
        let spec = decl.input.as_ref()?;
        let (repr, conversion, fallible) = self.input_conversion(decl, spec, registry);
        assert!(
            is_scalar(&repr),
            "Cbindgen custom representations must be C scalar types"
        );
        if let Some(domain) = &decl.domain {
            assert_eq!(
                TypeKey::from_type(domain.ty()),
                TypeKey::from_type(&repr),
                "Cbindgen conversion domain type does not match its input representation"
            );
        }
        let src = self.src_ty(ty);
        let wire = repr.clone();
        let name = Self::in_name(ty);
        let valid = decl
            .domain
            .as_ref()
            .map(|d| d.contains_expr(quote!(v)))
            .unwrap_or_else(|| quote!(true));
        let msg = format!("{} representation is outside its declared domain", key);
        let function: syn::ItemFn = if decl.domain.is_some() || fallible {
            let converted = if fallible {
                quote!((#conversion).map_err(|e| e.to_string()))
            } else {
                quote!(::core::result::Result::Ok(#conversion))
            };
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #wire)
                    -> ::core::result::Result<#src, ::std::string::String>
                {
                    if !(#valid) {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#msg)
                        );
                    }
                    #converted
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #wire) -> #src {
                    #conversion
                }
            )
        };
        let niches = self.c_domain_niches(decl, registry, Direction::Input);
        Some(ConverterImpl {
            subs: vec![repr],
            destination: wire,
            function,
            pre_stages: vec![],
            niches,
            metadata: (),
        })
    }

    pub(crate) fn out_custom(
        &self,
        ty: &syn::Type,
        registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        let decl = self.convert_decls.iter().find(|d| d.key == key)?;
        let spec = decl.output.as_ref()?;
        let (repr, conversion, fallible) = self.output_conversion(decl, spec, registry);
        assert!(
            is_scalar(&repr),
            "Cbindgen custom representations must be C scalar types"
        );
        if let Some(domain) = &decl.domain {
            assert_eq!(
                TypeKey::from_type(domain.ty()),
                TypeKey::from_type(&repr),
                "Cbindgen conversion domain type does not match its output representation"
            );
        }
        let src = self.src_ty(ty);
        let wire = repr.clone();
        let name = Self::out_name(ty);
        let valid = decl
            .domain
            .as_ref()
            .map(|d| d.contains_expr(quote!(__repr)))
            .unwrap_or_else(|| quote!(true));
        let msg = format!("{} representation is outside its declared domain", key);
        let function: syn::ItemFn = if decl.domain.is_some() || fallible {
            let repr_expr = if fallible {
                quote!((#conversion).map_err(|error| error.to_string())?)
            } else {
                quote!(#conversion)
            };
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src)
                    -> ::core::result::Result<#wire, ::std::string::String>
                {
                    let __repr: #repr = #repr_expr;
                    if !(#valid) {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#msg)
                        );
                    }
                    ::core::result::Result::Ok(__repr)
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #wire {
                    #conversion
                }
            )
        };
        let niches = self.c_domain_niches(decl, registry, Direction::Output);
        Some(ConverterImpl {
            subs: vec![repr],
            destination: wire,
            function,
            pre_stages: vec![],
            niches,
            metadata: (),
        })
    }

    fn input_conversion(
        &self,
        decl: &ConvertDecl,
        spec: &ConvertSpec,
        registry: &Registry<()>,
    ) -> (syn::Type, syn::Expr, bool) {
        let target = self.src_ty(&decl.key.to_type());
        match spec {
            ConvertSpec::PrebindgenFn(f) => {
                let item = &registry
                    .functions
                    .get(f)
                    .unwrap_or_else(|| panic!("Cbindgen conversion function {} was not found", f))
                    .0;
                let (repr, by_ref) = one_param(item);
                let ret = fn_ret(item);
                let (ok, fallible) = match result_parts(&ret) {
                    Some((ok, _)) => (ok, true),
                    None => (ret, false),
                };
                assert_eq!(TypeKey::from_type(&ok), decl.key);
                let path = self.conversion_fn_path(registry, f);
                let expr = if by_ref {
                    syn::parse_quote!(#path(&v))
                } else {
                    syn::parse_quote!(#path(v))
                };
                (repr, expr, fallible)
            }
            ConvertSpec::Trait { repr, fallible } => {
                let expr = if *fallible {
                    syn::parse_quote!(
                        <#repr as ::core::convert::TryInto<#target>>::try_into(v)
                    )
                } else {
                    syn::parse_quote!(
                        <#repr as ::core::convert::Into<#target>>::into(v)
                    )
                };
                (repr.clone(), expr, *fallible)
            }
        }
    }

    fn output_conversion(
        &self,
        decl: &ConvertDecl,
        spec: &ConvertSpec,
        registry: &Registry<()>,
    ) -> (syn::Type, syn::Expr, bool) {
        let target = self.src_ty(&decl.key.to_type());
        match spec {
            ConvertSpec::PrebindgenFn(f) => {
                let item = &registry
                    .functions
                    .get(f)
                    .unwrap_or_else(|| panic!("Cbindgen conversion function {} was not found", f))
                    .0;
                let (param, by_ref) = one_param(item);
                assert_eq!(TypeKey::from_type(&param), decl.key);
                let ret = fn_ret(item);
                let (repr, fallible) = match result_parts(&ret) {
                    Some((ok, _)) => (ok, true),
                    None => (ret, false),
                };
                let path = self.conversion_fn_path(registry, f);
                let expr = if by_ref {
                    syn::parse_quote!(#path(&v))
                } else {
                    syn::parse_quote!(#path(v))
                };
                (repr, expr, fallible)
            }
            ConvertSpec::Trait { repr, fallible } => {
                let expr = if *fallible {
                    syn::parse_quote!(
                        <#target as ::core::convert::TryInto<#repr>>::try_into(v)
                    )
                } else {
                    syn::parse_quote!(
                        <#target as ::core::convert::Into<#repr>>::into(v)
                    )
                };
                (repr.clone(), expr, *fallible)
            }
        }
    }

    fn c_domain_niches(
        &self,
        decl: &ConvertDecl,
        registry: &Registry<()>,
        direction: Direction,
    ) -> Niches {
        let Some(domain) = &decl.domain else {
            return Niches::empty();
        };
        let demand = registry
            .type_table(direction)
            .keys()
            .map(|candidate| option_depth(candidate, &decl.key))
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
                        crate::core::ScalarValue::F32(bits) => {
                            syn::parse_quote!(v.to_bits() == #bits)
                        }
                        crate::core::ScalarValue::F64(bits) => {
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

    fn conversion_fn_path(&self, registry: &Registry<()>, ident: &syn::Ident) -> syn::Path {
        let Some(mut module) = registry.origin_module(ident) else {
            return self.src_fn(ident);
        };
        module.segments.push(syn::PathSegment::from(ident.clone()));
        module
    }
}

fn one_param(item: &syn::ItemFn) -> (syn::Type, bool) {
    let params: Vec<_> = item
        .sig
        .inputs
        .iter()
        .filter_map(|p| {
            if let syn::FnArg::Typed(p) = p {
                Some(&*p.ty)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        params.len(),
        1,
        "conversion functions take exactly one parameter"
    );
    match params[0] {
        syn::Type::Reference(r) => ((*r.elem).clone(), true),
        ty => (ty.clone(), false),
    }
}

fn fn_ret(item: &syn::ItemFn) -> syn::Type {
    match &item.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

fn option_depth(candidate: &TypeKey, target: &TypeKey) -> usize {
    let mut ty = candidate.to_type();
    let mut depth = 0;
    while is_option(&ty) {
        let Some(inner) = first_type_arg(&ty) else {
            return 0;
        };
        ty = inner;
        depth += 1;
    }
    if TypeKey::from_type(&ty) == *target {
        depth
    } else {
        0
    }
}
