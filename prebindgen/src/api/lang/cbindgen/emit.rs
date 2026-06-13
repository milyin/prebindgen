use super::*;

impl Cbindgen {
    /// Whether the generated layer hands `char*` data memory to C — a `String`
    /// return value, or a declared data struct that is produced as output and has
    /// a `String` field. When true, a `free_memory_function` must be declared.
    pub(super) fn needs_free(&self, registry: &Registry<()>) -> bool {
        let string_ty: syn::Type = syn::parse_quote!(String);
        if registry.output_entry(&string_ty).is_some() {
            return true;
        }
        // Opaque error types are marshalled to a malloc'd `char*` message.
        if self
            .opaque_errors
            .keys()
            .any(|key| registry.output_entry(&key.to_type()).is_some())
        {
            return true;
        }
        self.data.keys().any(|key| {
            let ty = key.to_type();
            registry.output_entry(&ty).is_some()
                && self
                    .struct_fields(registry, &ty)
                    .map(|fields| fields.iter().any(|(_, fty)| is_string(fty)))
                    .unwrap_or(false)
        })
    }

    /// Whether any declared function returns a `Vec<_>` (possibly nested under
    /// `Result`/`Option`), so the array builder/freer prelude must be emitted.
    pub(super) fn produces_array(&self, registry: &Registry<()>) -> bool {
        self.functions.keys().any(|orig| {
            registry
                .functions
                .get(orig)
                .map(|(f, _)| match &f.sig.output {
                    syn::ReturnType::Type(_, ty) => type_contains_vec(ty),
                    syn::ReturnType::Default => false,
                })
                .unwrap_or(false)
        })
    }

    /// Fields (`name`, `type`) of a declared data struct, looked up from the
    /// registry's indexed structs. `None` if the type isn't an indexed named
    /// struct.
    pub(super) fn struct_fields(
        &self,
        registry: &Registry<()>,
        ty: &syn::Type,
    ) -> Option<Vec<(syn::Ident, syn::Type)>> {
        let ident = type_path_tail(ty)?;
        let (item, _) = registry.structs.get(&ident)?;
        if let syn::Fields::Named(named) = &item.fields {
            Some(
                named
                    .named
                    .iter()
                    .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Exported `#[no_mangle]` symbol for a declared function:
    /// [`Self::mangle_function`] over the base — a `.base_name(...)` override when
    /// set, else the Rust fn ident — or that base verbatim when no mangler is set.
    pub(super) fn fn_symbol(&self, orig: &syn::Ident) -> syn::Ident {
        let base = self
            .functions
            .get(orig)
            .and_then(|c| c.base.clone())
            .unwrap_or_else(|| orig.to_string());
        match &self.mangle_function {
            Some(f) => format_ident!("{}", f(&base)),
            None => format_ident!("{}", base),
        }
    }

    /// Assemble the `#[no_mangle] extern "C"` wrapper for one declared fn.
    pub(super) fn emit_function_wrapper(
        &self,
        f: &syn::ItemFn,
        registry: &Registry<()>,
    ) -> TokenStream {
        let orig = &f.sig.ident;
        let call_path = self.src_fn(orig);
        let sym = self.fn_symbol(orig);

        let return_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };

        let has_fallible_input = f.sig.inputs.iter().any(|input| {
            let syn::FnArg::Typed(pt) = input else {
                return false;
            };
            registry
                .input_entry(&pt.ty)
                .map(|e| returns_result(&e.function.sig.output))
                .unwrap_or(false)
        });

        // Peel an outer `Result<_, E>`; `value_ty` is the success/return value.
        let (value_ty, err_ty): (syn::Type, Option<syn::Type>) = match result_parts(&return_ty) {
            Some((ok, e)) => (ok, Some(e)),
            None => (return_ty.clone(), None),
        };

        // Error wiring: the error type must be declared via `.error()`.
        let err_bits = err_ty.as_ref().map(|err_ty| {
            assert!(
                self.error.contains(&TypeKey::from_type(err_ty)),
                "Cbindgen: function `{}` returns `Result<_, {}>` but `{}` is not a \
                 declared error type — add `.data_struct({}).error()`",
                orig,
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
            );
            let entry = registry.output_entry(err_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: error type `{}` of `{}` has no output converter",
                    TypeKey::from_type(err_ty),
                    orig
                )
            });
            (
                entry.destination.clone(),
                entry.function.sig.ident.clone(),
                self.src_ty(err_ty),
            )
        });

        // No `Result` channel ⇒ a fallible input must be declared `.panic()`.
        if err_ty.is_none() {
            let allows_panic = self.functions.get(orig).map(|c| c.panic).unwrap_or(false);
            assert!(
                !has_fallible_input || allows_panic,
                "Cbindgen: function `{}` has a fallible input (e.g. a `String` or \
                 opaque-by-value argument) but does not return `Result`; add \
                 `.panic()` after its `.function(...)` declaration to allow aborting \
                 on the internal error, or change its signature",
                orig,
            );
        }

        // Structural lowering of the (present/ok) value, then the null-niche rule:
        //   * Result + a free pointer niche  → NULL marks `Err` (value in-band);
        //   * Result without a free niche     → `bool` status, value to out-params;
        //   * no Result                       → field 0 is the C return, rest out.
        let shape = self.lower_shape(&value_ty, registry);
        let result_in_band = err_ty.is_some() && shape.has_niche; // value rides the return
        let field0_is_return = result_in_band || err_ty.is_none();

        // Partition fields into the (optional) C return value + out-parameters,
        // and pick C names for the out-params (see `out_param_name`).
        let mut targets: Vec<TokenStream> = Vec::new();
        let mut out_fields: Vec<&WireField> = Vec::new();
        // `field0_wire` is the wire of the value's primary field when that field
        // is carried by the C return slot (modes A/D); `None` for mode B and unit.
        let field0_wire: Option<syn::Type> = if field0_is_return {
            shape.fields.first().map(|f| f.wire.clone())
        } else {
            None
        };
        if field0_is_return {
            if !shape.fields.is_empty() {
                targets.push(quote!(__ret));
                out_fields.extend(shape.fields[1..].iter());
            }
        } else {
            out_fields.extend(shape.fields.iter());
        }
        let prefixed = out_fields.iter().any(|wf| wf.suffix.is_empty());
        let out_names: Vec<syn::Ident> = out_fields
            .iter()
            .map(|wf| out_param_name(wf.suffix, prefixed))
            .collect();
        for name in &out_names {
            targets.push(quote!(*#name));
        }
        let out_param_decls: Vec<TokenStream> = out_fields
            .iter()
            .zip(&out_names)
            .map(|(wf, name)| {
                let wire = &wf.wire;
                quote!(#name: *mut #wire)
            })
            .collect();

        // C wrapper return type: the payload's field 0 (modes A/D), `bool` status
        // (mode B), or `void` (a unit value with no `Result`).
        let c_return: Option<syn::Type> = if field0_is_return {
            field0_wire.clone()
        } else {
            Some(syn::parse_quote!(bool))
        };

        // Input decode: route a fallible-input failure to the error out-param
        // (with the wrapper's fail value) when there is a `Result`, else panic.
        let fail_return = if result_in_band {
            null_for(field0_wire.as_ref().expect("in-band ⇒ pointer return"))
        } else {
            quote!(false)
        };
        let input_route = match &err_bits {
            Some((_, e_conv, e_ty_src)) => ErrRoute::Result {
                e_conv,
                e_ty_src: e_ty_src.clone(),
                fail_return: fail_return.clone(),
            },
            None => ErrRoute::Panic,
        };
        let (in_params, decodes, call_args) = self.emit_inputs(orig, f, registry, &input_route);
        let call = quote!(#call_path(#(#call_args),*));

        let e_param = err_bits
            .as_ref()
            .map(|(err_wire, _, _)| quote!(e: *mut #err_wire));
        let ret_arrow = c_return.as_ref().map(|w| quote!(-> #w));

        // Assemble the body per the three structural modes.
        let body = match (&err_bits, field0_is_return) {
            // No `Result`: straight-line. `void` when there are no fields.
            (None, _) => {
                if field0_wire.is_none() {
                    quote!( #(#decodes)* #call; )
                } else {
                    let field0_wire = field0_wire.as_ref().unwrap();
                    let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                    quote!(
                        #(#decodes)*
                        let __v = #call;
                        let __ret: #field0_wire;
                        #enc
                        __ret
                    )
                }
            }
            // `Result` with a free niche: value in-band, NULL marks `Err`.
            (Some((_, e_conv, _)), true) => {
                let field0_wire = field0_wire.as_ref().expect("in-band ⇒ pointer return");
                let null = null_for(field0_wire);
                let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { let __ret: #field0_wire; #enc __ret }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            #null
                        }
                    }
                )
            }
            // `Result` without a free niche: `bool` status, value to out-params.
            (Some((_, e_conv, _)), false) => {
                let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { #enc true }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            false
                        }
                    }
                )
            }
        };

        quote! {
            #[no_mangle]
            #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
            pub unsafe extern "C" fn #sym(
                #(#in_params,)*
                #(#out_param_decls,)*
                #e_param
            ) #ret_arrow {
                #body
            }
        }
    }

    /// Lower how a *present / ok* value of `ty` is carried over the C ABI: an
    /// ordered list of wire components, plus whether `fields[0]` is a pointer
    /// whose NULL bit-pattern is still free for an enclosing `Option`/`Result`
    /// layer to claim. Mirrors the niche-stacking model in `core::niches`.
    pub(super) fn lower_shape(&self, ty: &syn::Type, registry: &Registry<()>) -> ValueShape {
        if is_unit(ty) {
            return ValueShape {
                fields: vec![],
                has_niche: false,
            };
        }
        // `Vec<T>` → `T_wire* + size_t`. The element must lower to a single C
        // value (one converter); a composite element is unsupported.
        if is_vec(ty) {
            let elem = first_type_arg(ty).expect("Vec<T> has a type argument");
            assert!(
                !is_option(&elem) && !is_vec(&elem) && !is_result(&elem),
                "Cbindgen: `Vec<{}>` element must be a single-value type \
                 (scalar, data struct, String, or handle), not a composite",
                TypeKey::from_type(&elem),
            );
            let entry = registry.output_entry(&elem).unwrap_or_else(|| {
                panic!(
                    "Cbindgen: `Vec` element `{}` has no output converter",
                    TypeKey::from_type(&elem)
                )
            });
            let elem_wire = entry.destination.clone();
            return ValueShape {
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
                has_niche: false,
            };
        }
        // `Option<T>` consumes one discriminant. If the inner value still has a
        // free pointer niche, reuse it (NULL = `None`); otherwise prepend an
        // explicit `present: bool`. Either way the result exposes no niche.
        if is_option(ty) {
            let inner_ty = first_type_arg(ty).expect("Option<T> has a type argument");
            let inner = self.lower_shape(&inner_ty, registry);
            if inner.has_niche {
                return ValueShape {
                    fields: inner.fields,
                    has_niche: false,
                };
            }
            let mut fields = vec![WireField {
                suffix: "_present",
                wire: syn::parse_quote!(bool),
            }];
            fields.extend(inner.fields);
            return ValueShape {
                fields,
                has_niche: false,
            };
        }
        // Base value: one wire component from its rank-0/1 converter. A pointer
        // wire (String, opaque handle, `&'static`) carries a free NULL niche.
        let entry = registry.output_entry(ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: type `{}` has no output converter",
                TypeKey::from_type(ty)
            )
        });
        let wire = entry.destination.clone();
        let has_niche = matches!(wire, syn::Type::Ptr(_));
        ValueShape {
            fields: vec![WireField { suffix: "", wire }],
            has_niche,
        }
    }

    /// Emit the statements that write a native value `val` of type `ty` into the
    /// `targets` lvalues (one per field of `lower_shape(ty)`, in order).
    pub(super) fn encode_value(
        &self,
        ty: &syn::Type,
        val: TokenStream,
        targets: &[TokenStream],
        registry: &Registry<()>,
    ) -> TokenStream {
        if is_unit(ty) {
            return quote!();
        }
        if is_vec(ty) {
            let elem = first_type_arg(ty).expect("Vec<T> has a type argument");
            let entry = registry.output_entry(&elem).expect("Vec element converter");
            let elem_conv = entry.function.sig.ident.clone();
            let elem_wire = entry.destination.clone();
            let t_ptr = &targets[0];
            let t_len = &targets[1];
            return quote!(
                let __arr: ::std::vec::Vec<#elem_wire> =
                    #val.into_iter().map(#elem_conv).collect();
                let (__p, __n) = __cbg_alloc_array(__arr);
                #t_ptr = __p;
                #t_len = __n;
            );
        }
        if is_option(ty) {
            let inner_ty = first_type_arg(ty).expect("Option<T> has a type argument");
            let inner = self.lower_shape(&inner_ty, registry);
            if inner.has_niche {
                // None reuses the inner pointer's NULL; Some encodes inline.
                let inner_enc = self.encode_value(&inner_ty, quote!(__x), targets, registry);
                let null = null_for(&inner.fields[0].wire);
                let t0 = &targets[0];
                return quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => { #inner_enc }
                        ::core::option::Option::None => { #t0 = #null; }
                    }
                );
            }
            // Explicit `present` flag in targets[0]; inner value follows.
            let present = &targets[0];
            let inner_enc = self.encode_value(&inner_ty, quote!(__x), &targets[1..], registry);
            return quote!(
                match #val {
                    ::core::option::Option::Some(__x) => { #present = true; #inner_enc }
                    ::core::option::Option::None => { #present = false; }
                }
            );
        }
        // Base value: run its output converter into the single target.
        let entry = registry.output_entry(ty).expect("base value converter");
        let conv = entry.function.sig.ident.clone();
        let t0 = &targets[0];
        quote!( #t0 = #conv(#val); )
    }

    /// Build the wire param list, per-input decode statements, and call-site
    /// argument expressions. Fallible inputs (converter returns `Result<_,
    /// String>`) route their `Err(msg)` per `route`; infallible inputs decode
    /// directly.
    pub(super) fn emit_inputs(
        &self,
        orig: &syn::Ident,
        f: &syn::ItemFn,
        registry: &Registry<()>,
        route: &ErrRoute,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let mut params = Vec::new();
        let mut decodes = Vec::new();
        let mut call_args = Vec::new();

        for input in &f.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let syn::Pat::Ident(pat_id) = &*pt.pat else {
                continue;
            };
            let ident = &pat_id.ident;
            let arg_ty = &*pt.ty;

            // `&[E]` slice (scalar `E`): two wire params (`*const E`, `usize`),
            // decoded zero-copy. NULL pointer ⇒ empty slice (not an error).
            if let Some(elem) = scalar_slice_elem(arg_ty) {
                let len_id = format_ident!("{}_len", ident);
                params.push(quote!(#ident: *const #elem));
                params.push(quote!(#len_id: usize));
                decodes.push(quote!(
                    let #ident: &[#elem] = if #ident.is_null() {
                        &[]
                    } else {
                        ::core::slice::from_raw_parts(#ident, #len_id)
                    };
                ));
                call_args.push(quote!(#ident));
                continue;
            }

            let entry = registry.input_entry(arg_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: input type `{}` of `{}` has no input converter",
                    TypeKey::from_type(arg_ty),
                    orig
                )
            });
            let wire = &entry.destination;
            let conv = &entry.function.sig.ident;

            params.push(quote!(#ident: #wire));

            if returns_result(&entry.function.sig.output) {
                let on_err = match route {
                    ErrRoute::Result {
                        e_conv,
                        e_ty_src,
                        fail_return,
                    } => quote!(
                        if !e.is_null() {
                            *e = #e_conv(<#e_ty_src as ::core::convert::From<::std::string::String>>::from(__msg));
                        }
                        return #fail_return;
                    ),
                    ErrRoute::Panic => quote!(panic!("{}", __msg);),
                };
                decodes.push(quote!(
                    let #ident = match #conv(#ident) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__msg) => { #on_err }
                    };
                ));
            } else {
                decodes.push(quote!(let #ident = #conv(#ident);));
            }

            // Each input converter produces exactly the source param type
            // (`String` by value, `&T` for borrows, owned `T` for consume), so
            // the decoded binding is passed straight through.
            call_args.push(quote!(#ident));
        }

        (params, decodes, call_args)
    }
}
