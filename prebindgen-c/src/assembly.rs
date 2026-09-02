//! What the generated C-facing Rust file is assembled from.
//!
//! Each final artifact is planned when the C generation plan is frozen and
//! rendered at the final writing boundary, so nothing it needs is looked up in
//! a live registry while the file is assembled. A wrapper — the exported
//! `extern "C"` function one declared `#[prebindgen]` function becomes — reads
//! its own frozen state plus the boundary sites of its own function, which the
//! same frozen plan holds.

use std::rc::Rc;

use prebindgen_registry::{
    generation::GenerationPlan,
    write::{ArtifactKey, RustArtifact},
};

use super::*;

/// One final artifact of the generated Rust file.
pub(crate) enum CFinalArtifact {
    /// A private converter, carrying one value across the boundary.
    Converter(Box<crate::chain::CFunction>),
    /// The exported wrapper for one declared `#[prebindgen]` function.
    Wrapper(Box<CWrapper>),
    /// One captured constant, re-stated as an alias to its source.
    Const(Box<CConst>),
    /// The memory helpers the generated layer hands `char*` and array blocks
    /// to C through.
    Memory(Box<CMemory>),
    /// The array builder, which copies a `Vec<T>` into a C block.
    ArrayBuilder,
    /// One `#[repr(C)]` mirror of a declared data struct.
    DataStruct(Box<CDataStruct>),
    /// One `#[repr(C)]` mirror of a declared fieldless enum.
    Enum(Box<CEnum>),
    /// One reserved representation value a generated sum-type ABI uses.
    DomainConstant(Box<CDomainConstant>),
    /// One artifact the registry generation plan already holds: an opaque
    /// handle, a value-opaque, a tagged union, or a callback.
    Planned(Box<CPlanned>),
}

impl RustArtifact for CFinalArtifact {
    fn key(&self) -> ArtifactKey {
        match self {
            Self::Converter(converter) => converter.key(),
            Self::Wrapper(wrapper) => wrapper.key(),
            Self::Const(constant) => constant.key(),
            Self::Memory(memory) => memory.key(),
            Self::ArrayBuilder => array_builder_key(),
            Self::DataStruct(item) => item.key(),
            Self::Enum(item) => item.key(),
            Self::DomainConstant(item) => item.key(),
            Self::Planned(item) => item.key(),
        }
    }

    fn provides(&self) -> Vec<ArtifactKey> {
        match self {
            Self::Planned(item) => item.provides(),
            _ => vec![self.key()],
        }
    }

    fn reachable(&self) -> bool {
        // Every C artifact reaches the file. Unreached converters are pruned
        // from the generation plan before the assembly sees them, and every
        // other kind is planned only when something needs it — the memory
        // helpers because an artifact says it calls them, a mirror because its
        // type crosses. Stated rather than inherited: an artifact that needs a
        // different answer should be a deliberate change here.
        true
    }

    fn calls(&self) -> Vec<ArtifactKey> {
        match self {
            Self::Converter(converter) => converter.calls(),
            Self::Wrapper(wrapper) => wrapper.calls(),
            Self::Const(constant) => constant.calls(),
            Self::Memory(memory) => memory.calls(),
            // The block it fills is malloc'd, and freed by the universal freer.
            Self::ArrayBuilder => vec![memory_key()],
            Self::DataStruct(item) => item.calls(),
            Self::Enum(item) => item.calls(),
            Self::DomainConstant(item) => item.calls(),
            Self::Planned(item) => item.calls(),
        }
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        match self {
            Self::Converter(converter) => converter.render(emit),
            Self::Wrapper(wrapper) => wrapper.render(emit),
            Self::Const(constant) => constant.render(emit),
            Self::Memory(memory) => memory.render(emit),
            Self::ArrayBuilder => CMemory::render_array_builder(),
            Self::DataStruct(item) => item.render(emit),
            Self::Enum(item) => item.render(emit),
            Self::DomainConstant(item) => item.render(emit),
            Self::Planned(item) => item.render(emit),
        }
    }
}

/// One declared function's exported wrapper, frozen at the end of resolution.
pub(crate) struct CWrapper {
    /// This function's own boundary sites, as the registry planned them. All a
    /// wrapper reads of the plan, and taken by value so a wrapper can be stated
    /// before the plan is built.
    sites: Vec<prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation>>,
    /// The `#[prebindgen]` function this wrapper exports.
    function: prebindgen_registry::flat::Function,
    /// The `#[no_mangle]` symbol the wrapper is exported under.
    symbol: syn::Ident,
    /// Path the wrapper calls the source function by.
    call_path: syn::Path,
    /// Whether `.panic()` was declared, which is what allows a fallible
    /// conversion in a function that returns no `Result`.
    panic: bool,
    /// Per parameter, the resource its argument names and how the call uses
    /// it — `None` for a parameter that names no single owned resource. The
    /// alias preflight compares these; declaring which types are opaque is a
    /// binding question, so it is answered while planning.
    alias_slots: Vec<Option<(TypeKey, AliasAccess)>>,
}

impl CWrapper {
    /// Plan the wrapper for one declared function.
    /// `sites` are the boundary sites the registry planned for this function,
    /// which is everything of the plan a wrapper reads. Taking them rather than
    /// the plan is what lets a wrapper be stated as an `ArtifactPlan` before the
    /// plan is built, since the sites exist first.
    pub(crate) fn new(
        decls: &CbindgenBuilder,
        sites: &[prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation>],
        function: &prebindgen_registry::flat::Function,
    ) -> Self {
        let name = &function.name;
        if let Some((_, err)) = function.ret.fallible_parts() {
            let err_key = err.key();
            assert!(
                decls.error.contains(&err_key),
                "Cbindgen: function `{}` returns `Result<_, {}>` but `{}` is not a \
                 declared error type — add `.data_struct({}).error()`",
                name,
                err_key,
                err_key,
                err_key,
            );
        }
        Self {
            sites: sites
                .iter()
                .filter(|site| site.id().site().owner == *name)
                .cloned()
                .collect(),
            function: function.clone(),
            symbol: decls.fn_symbol(name),
            call_path: decls.src_fn(name),
            panic: decls.functions.get(name).is_some_and(|cfg| cfg.panic),
            alias_slots: function
                .params
                .iter()
                .map(|param| decls.alias_slot_of(&param.ty))
                .collect(),
        }
    }

    /// One boundary site of this wrapper's own function, when the function has
    /// one in that role. A void return and an infallible function have no
    /// return or error site.
    fn site_if_planned(
        &self,
        role: prebindgen_registry::recipe::Role,
    ) -> Option<&prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation>> {
        self.sites.iter().find(|site| site.id().site().role == role)
    }

    /// One boundary site of this wrapper's own function.
    fn site(
        &self,
        role: prebindgen_registry::recipe::Role,
    ) -> &prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation> {
        let name = self.function.name.clone();
        let described = role.to_string();
        let site = self
            .site_if_planned(role)
            .unwrap_or_else(|| panic!("C generation plan has no {described} of `{name}`"));
        assert!(
            matches!(
                site.cleanup(),
                prebindgen_registry::generation::Cleanup::None
            ),
            "ordinary C sites cannot carry a deferred cleanup operation"
        );
        site
    }

    /// Assemble this wrapper's `#[no_mangle] extern "C"` function.
    fn render_fn(&self, emit: &prebindgen_registry::RustWriter) -> syn::ItemFn {
        let f = &self.function;
        let orig = &f.name;
        let call_path = &self.call_path;
        let sym = &self.symbol;

        // The ELEMENT: a signature is a parameter list and a return, both
        // already classified. An elided return is `TypeKind::Unit`, which is
        // the `ReturnType::Default` arm this used to write.

        let has_fallible_input = f.params.iter().enumerate().any(|(index, _)| {
            self.site(prebindgen_registry::recipe::Role::Param { index })
                .abi()
                .payload()
                .failure()
                == prebindgen_registry::generation::Failure::Fallible
        });

        // Peel an outer `Result<_, E>`; `value_ty` is the success/return value.
        // Off `TypeKind::Fallible`, where `result_parts` found the `Result` in a
        // path first — and both sides come back as readings, so everything
        // downstream of here reads too.
        let (value_ty, err_reading) = match f.ret.fallible_parts() {
            Some((ok, e)) => (ok, Some(e)),
            None => (&f.ret, None),
        };
        let value_site = (!matches!(value_ty.kind(), TypeKind::Unit))
            .then(|| self.site(prebindgen_registry::recipe::Role::Return));
        let has_fallible_output = value_site.is_some_and(|site| {
            site.abi().payload().failure() == prebindgen_registry::generation::Failure::Fallible
        });

        // Error wiring: that the error type is declared via `.error()` was
        // checked when the wrapper was planned.
        let err_bits = err_reading.map(|err| {
            let site = self.site(prebindgen_registry::recipe::Role::Error);
            let crate::compile::CValue::Direct {
                wire, converter, ..
            } = site.abi().payload()
            else {
                panic!("C error site must have one direct wire");
            };
            (
                wire.clone(),
                converter.ident(emit),
                emit.emit_source_type(err),
            )
        });

        // No `Result` channel ⇒ a fallible input must be declared `.panic()`.
        if err_reading.is_none() {
            assert!(
                !(has_fallible_input || has_fallible_output) || self.panic,
                "Cbindgen: function `{}` has a fallible binding conversion but does not \
                 return `Result`; add \
                 `.panic()` after its `.function(...)` declaration to allow aborting \
                 on the internal error, or change its signature",
                orig,
            );
        }

        // Structural lowering of the (present/ok) value, then the null-niche rule:
        //   * Result + a free pointer niche  → NULL marks `Err` (value in-band);
        //   * Result without a free niche     → `bool` status, value to out-params;
        //   * no Result                       → field 0 is the C return, rest out.
        let shape = value_site.map_or(
            FrozenValueLayout {
                fields: Vec::new(),
                niches: Niches::empty(),
            },
            |site| FrozenValueLayout {
                fields: site.abi().payload().fields(),
                niches: site.abi().payload().effective_niches(),
            },
        );
        let result_slot = shape.niches.clone().carve().map(|(slot, _)| slot);
        let result_in_band = err_reading.is_some() && result_slot.is_some();
        let field0_is_return = result_in_band || err_reading.is_none();

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
            let slot = result_slot.as_ref().expect("in-band result has a niche");
            let value = &slot.value;
            quote!(#value)
        } else {
            quote!(false)
        };
        let mut planned_routes = f
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                self.site(prebindgen_registry::recipe::Role::Param { index })
                    .failure_route()
            })
            .chain(
                value_site
                    .into_iter()
                    .filter_map(|site| site.failure_route()),
            );
        let planned_route = planned_routes.next();
        assert!(
            planned_routes.all(|route| Some(route) == planned_route),
            "C function sites disagree on their frozen failure route"
        );
        match planned_route {
            Some(crate::compile::CFailureRoute::Panic) => {
                assert!(
                    err_bits.is_none(),
                    "panic route attached to a Result function"
                );
            }
            Some(crate::compile::CFailureRoute::Error(error_site)) => {
                assert!(
                    err_bits.is_some(),
                    "error route attached without a Result channel"
                );
                let expected = prebindgen_registry::generation::SiteId::new(
                    prebindgen_registry::recipe::Site {
                        owner: orig.clone(),
                        role: prebindgen_registry::recipe::Role::Error,
                    },
                );
                assert_eq!(
                    error_site, &expected,
                    "failure route names the wrong error site"
                );
            }
            None => {}
        }
        let input_route = match &err_bits {
            Some((_, e_conv, e_ty_src)) => ErrRoute::Result {
                e_conv,
                e_ty_src: e_ty_src.clone(),
                fail_return: fail_return.clone(),
            },
            None => ErrRoute::Panic,
        };
        let (in_params, decodes, call_args) = self.planned_inputs(&input_route, emit);
        let call = quote!(#call_path(#(#call_args),*));

        let e_param = err_bits
            .as_ref()
            .map(|(err_wire, _, _)| quote!(e: *mut #err_wire));
        let ret_arrow = c_return.as_ref().map(|w| quote!(-> #w));

        // Assemble the body per the three structural modes.
        let body = match (&err_bits, field0_is_return) {
            // No `Result`: straight-line. `void` when there are no fields.
            (None, _) => {
                if let Some(field0_wire) = field0_wire.as_ref() {
                    let enc = value_site.map_or_else(TokenStream::new, |site| {
                        site.abi()
                            .payload()
                            .encode(quote!(__v), &targets, &input_route, emit)
                    });
                    quote!(
                        #(#decodes)*
                        let __v = #call;
                        let __ret: #field0_wire;
                        #enc
                        __ret
                    )
                } else {
                    quote!( #(#decodes)* #call; )
                }
            }
            // `Result` with a free niche: value in-band, NULL marks `Err`.
            (Some((_, e_conv, _)), true) => {
                let field0_wire = field0_wire.as_ref().expect("in-band ⇒ pointer return");
                let null = &result_slot
                    .as_ref()
                    .expect("in-band result has a niche")
                    .value;
                let enc = value_site.map_or_else(TokenStream::new, |site| {
                    site.abi()
                        .payload()
                        .encode(quote!(__v), &targets, &input_route, emit)
                });
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
                let enc = value_site.map_or_else(TokenStream::new, |site| {
                    site.abi()
                        .payload()
                        .encode(quote!(__v), &targets, &input_route, emit)
                });
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

        syn::parse_quote! {
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

    /// The runtime **alias preflight**: reject a call whose arguments name the
    /// same resource in a combination that would reconstruct or invalidate it
    /// twice, *before* any conversion runs.
    ///
    /// `z_combine(primary: ZThing, fallback: ZThing)` is a supported
    /// declaration; called as `z_combine(x, x)` it reaches `Box::from_raw`
    /// twice on one allocation. So is `f(a: ZThing, b: &ZThing)` — the borrow
    /// dangles the moment the consume takes ownership — and `f(a: &mut T, b:
    /// &T)`, where the exclusive reference is not exclusive at all. Rejecting
    /// the *declaration* is not an option: these are shapes that ship today, so
    /// removing them would be a regression that a later stage would have to
    /// undo.
    ///
    /// Emitted whenever a call has **at least one `Consume` or
    /// `ExclusiveBorrow`** and **any other active access in the same resource
    /// domain**. Stated that way rather than as "two or more consumed
    /// parameters", which would skip exactly the two mixed cases above.
    ///
    /// A NULL pointer names no resource — two NULLs are not an alias — and is
    /// rejected by each converter's own null check, so the comparison skips it.
    ///
    /// Shared/shared is *not* rejected: two `&T` to one resource is legal Rust
    /// and legal C.
    fn alias_preflight(&self, route: &ErrRoute) -> Option<TokenStream> {
        let slots: Vec<(syn::Ident, TypeKey, AliasAccess)> = self
            .function
            .params
            .iter()
            .zip(&self.alias_slots)
            .filter_map(|(param, slot)| {
                slot.as_ref()
                    .map(|(key, access)| (param.name.clone(), key.clone(), *access))
            })
            .collect();

        let on_err = route_message(route);
        let mut checks: Vec<TokenStream> = Vec::new();
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                let (a, a_key, a_access) = &slots[i];
                let (b, b_key, b_access) = &slots[j];
                if a_key != b_key {
                    continue;
                }
                // At least one side must be exclusive about the resource.
                if matches!(a_access, AliasAccess::Shared)
                    && matches!(b_access, AliasAccess::Shared)
                {
                    continue;
                }
                let msg = format!(
                    "aliasing arguments: `{a}` ({}) and `{b}` ({}) are the same `{}` — a \
                     consumed or exclusively-borrowed resource may not be named twice in one call",
                    a_access.describe(),
                    b_access.describe(),
                    a_key.as_str(),
                );
                checks.push(quote!(
                    if !(#a as *const ()).is_null() && (#a as *const ()) == (#b as *const ()) {
                        let __msg = ::std::string::String::from(#msg);
                        #on_err
                    }
                ));
            }
        }
        (!checks.is_empty()).then(|| quote!(#(#checks)*))
    }

    /// Build the wire param list, per-input decode statements, and call-site
    /// argument expressions. Fallible inputs (converter returns `Result<_,
    /// String>`) route their `Err(msg)` per `route`; infallible inputs decode
    /// directly.
    fn planned_inputs(
        &self,
        route: &ErrRoute<'_>,
        emit: &prebindgen_registry::RustWriter,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let mut params = Vec::new();
        let mut decodes: Vec<TokenStream> = self.alias_preflight(route).into_iter().collect();
        let mut call_args = Vec::new();

        for (index, param) in self.function.params.iter().enumerate() {
            let ident = &param.name;
            let site = self.site(prebindgen_registry::recipe::Role::Param { index });
            match site.abi().payload() {
                crate::compile::CValue::BorrowedInput {
                    element,
                    wire,
                    reinterpret,
                } => {
                    let len_id = format_ident!("{}_len", ident);
                    let source = emit.emit_source_type(element);
                    params.push(quote!(#ident: #wire));
                    params.push(quote!(#len_id: usize));
                    let from_parts = if *reinterpret {
                        quote!(::core::slice::from_raw_parts(
                            #ident as *const #source,
                            #len_id,
                        ))
                    } else {
                        quote!(::core::slice::from_raw_parts(#ident, #len_id))
                    };
                    decodes.push(quote!(
                        let #ident: &[#source] = if #ident.is_null() {
                            &[]
                        } else {
                            #from_parts
                        };
                    ));
                }
                crate::compile::CValue::Direct {
                    wire, converter, ..
                } => {
                    let conv = converter.ident(emit);
                    params.push(quote!(#ident: #wire));
                    if converter.fallible() {
                        let on_err = route_message(route);
                        decodes.push(quote!(
                            let #ident = match #conv(#ident) {
                                ::core::result::Result::Ok(__v) => __v,
                                ::core::result::Result::Err(__msg) => { #on_err }
                            };
                        ));
                    } else {
                        decodes.push(quote!(let #ident = #conv(#ident);));
                    }
                }
                _ => panic!("C input site has an output-only ABI plan"),
            }
            call_args.push(quote!(#ident));
        }

        (params, decodes, call_args)
    }
}

impl RustArtifact for CWrapper {
    fn calls(&self) -> Vec<ArtifactKey> {
        // One site per parameter, plus the return and the error channel when
        // the function has them — the same sites the body decodes and encodes
        // through.
        let mut calls = Vec::new();
        let roles = (0..self.function.params.len())
            .map(|index| prebindgen_registry::recipe::Role::Param { index })
            .chain([
                prebindgen_registry::recipe::Role::Return,
                prebindgen_registry::recipe::Role::Error,
            ]);
        for role in roles {
            if let Some(site) = self.site_if_planned(role) {
                site.abi().payload().calls(&mut calls);
            }
        }
        calls
    }

    fn key(&self) -> ArtifactKey {
        ArtifactKey::Artifact(
            prebindgen_registry::generation::ArtifactId::new("c-wrapper", self.symbol.to_string())
                .expect("an exported symbol is a non-empty artifact name"),
        )
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        vec![syn::Item::Fn(self.render_fn(emit))]
    }
}

/// One captured `#[prebindgen]` constant, re-stated in the generated file as
/// an alias to the source item.
///
/// The initializer is never copied: it may name source-crate internals, and
/// an alias keeps a constant with a non-portable initializer valid here.
/// cbindgen cannot evaluate a path initializer, so an aliased constant does
/// not surface as a `#define` in the C header.
pub(crate) struct CConst {
    /// The constant as the model holds it.
    constant: prebindgen_registry::flat::Constant,
    /// Module the source constant is reached through. `None` when the binding
    /// declared no source module, which emits nothing.
    source_module: Option<syn::Path>,
}

impl CConst {
    /// Plan the alias for one captured constant.
    pub(crate) fn new(
        decls: &CbindgenBuilder,
        constant: &prebindgen_registry::flat::Constant,
    ) -> Self {
        Self {
            constant: constant.clone(),
            source_module: decls.source_module.clone(),
        }
    }
}

impl RustArtifact for CConst {
    fn calls(&self) -> Vec<ArtifactKey> {
        Vec::new()
    }

    fn key(&self) -> ArtifactKey {
        ArtifactKey::Artifact(
            prebindgen_registry::generation::ArtifactId::new(
                "c-const",
                self.constant.name.to_string(),
            )
            .expect("a constant name is a non-empty artifact name"),
        )
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        self.source_module
            .as_ref()
            .map(|module| vec![syn::Item::Const(emit.const_alias(&self.constant, module))])
            .unwrap_or_default()
    }
}

/// The memory helpers' identity, which every artifact that hands `char*`
/// memory to C — or frees any block — depends on.
pub(crate) fn memory_key() -> ArtifactKey {
    artifact_id("c-runtime", "memory")
}

/// The array builder's identity, which every artifact that hands C a block of
/// converted elements depends on.
pub(crate) fn array_builder_key() -> ArtifactKey {
    artifact_id("c-runtime", "array-builder")
}

/// An adapter-scoped artifact identity, for the artifacts this module names
/// itself rather than reading off the generation plan.
fn artifact_id(kind: &str, name: impl Into<String>) -> ArtifactKey {
    ArtifactKey::Artifact(
        prebindgen_registry::generation::ArtifactId::new(kind, name)
            .expect("a C artifact name is non-empty"),
    )
}

/// One artifact the registry generation plan already holds.
///
/// Handles, value-opaques, tagged unions and callbacks are planned into the
/// generation plan while their sites are compiled. This is the same artifact,
/// placed in the file: the payload is the plan's, and rendering it is a lookup
/// in a frozen plan.
pub(crate) struct CPlanned {
    generation: Rc<GenerationPlan<crate::compile::CRepresentation>>,
    id: prebindgen_registry::generation::ArtifactId,
}

impl CPlanned {
    /// Every planned artifact of one kind, in the plan's own order.
    pub(crate) fn of_kind(
        generation: &Rc<GenerationPlan<crate::compile::CRepresentation>>,
        kind: &str,
    ) -> Vec<Self> {
        generation
            .artifacts()
            .filter(|artifact| artifact.id().kind() == kind)
            .map(|artifact| Self {
                generation: Rc::clone(generation),
                id: artifact.id().clone(),
            })
            .collect()
    }
}

impl CPlanned {
    /// The plan's own artifact description.
    fn payload(&self) -> &crate::chain::CArtifact {
        self.generation
            .artifact(&self.id)
            .unwrap_or_else(|| panic!("the C generation plan lost artifact {}", self.id))
            .payload()
    }
}

impl RustArtifact for CPlanned {
    fn provides(&self) -> Vec<ArtifactKey> {
        let mut provided = vec![self.key()];
        provided.extend(self.payload().provides());
        provided
    }

    fn calls(&self) -> Vec<ArtifactKey> {
        self.payload().calls()
    }

    fn key(&self) -> ArtifactKey {
        ArtifactKey::Artifact(self.id.clone())
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        self.generation
            .artifact(&self.id)
            .unwrap_or_else(|| panic!("the C generation plan lost artifact {}", self.id))
            .payload()
            .render(emit)
    }
}

/// The memory helpers: the C allocator the generated layer calls, the raw
/// C-string block builder, the universal freer C calls back, and — when a
/// `Vec<T>` return hands out a block — the array builder.
///
/// Planned only when the layer actually hands memory to C, which is also when
/// a declared `.free_memory_function` becomes required.
pub(crate) struct CMemory {
    /// The declared freer's exported symbol.
    free_ident: syn::Ident,
}

impl CMemory {
    /// Plan the memory helpers for a binding that hands memory to C.
    ///
    /// Whether it does is not asked here: the artifacts that allocate say so
    /// through their own dependencies, and the caller plans this when one of
    /// them names it.
    pub(crate) fn new(decls: &CbindgenBuilder) -> Self {
        let Some(free_fn) = &decls.free_fn else {
            panic!(
                "Cbindgen: the generated layer hands C memory it must free — a \
                 `char*` block (a `String` return or a `String` data-struct \
                 field) or an array block (a `Vec` returned or delivered to a \
                 callback) — but no memory-freeing function is declared: add \
                 `.free_memory_function(\"z_free\")`"
            )
        };
        Self {
            free_ident: format_ident!("{}", free_fn),
        }
    }

    /// Copy a `Vec<W>` into a C-`malloc`'d block of `W` and return
    /// `(ptr, len)` (empty ⇒ `(NULL, 0)`). The block is freed C-side via the
    /// `z_free_array` macro (per-element drop + the universal freer).
    fn render_array_builder() -> Vec<syn::Item> {
        vec![syn::parse_quote!(
            #[allow(non_snake_case, dead_code)]
            pub(crate) unsafe fn __cbg_alloc_array<W>(v: ::std::vec::Vec<W>) -> (*mut W, usize) {
                let n = v.len();
                if n == 0 {
                    return (::core::ptr::null_mut(), 0);
                }
                let p = malloc(n.wrapping_mul(::core::mem::size_of::<W>())) as *mut W;
                if p.is_null() {
                    return (::core::ptr::null_mut(), 0);
                }
                for (i, e) in v.into_iter().enumerate() {
                    ::core::ptr::write(p.add(i), e);
                }
                (p, n)
            }
        )]
    }
}

impl RustArtifact for CMemory {
    fn calls(&self) -> Vec<ArtifactKey> {
        Vec::new()
    }

    fn key(&self) -> ArtifactKey {
        memory_key()
    }

    fn render(&self, _emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        let free_ident = &self.free_ident;
        // C allocator (linked from the C runtime; no crate dependency).
        let items: Vec<syn::Item> = vec![
            syn::parse_quote!(
                extern "C" {
                    fn malloc(size: usize) -> *mut ::core::ffi::c_void;
                    fn free(ptr: *mut ::core::ffi::c_void);
                }
            ),
            // Raw, destructor-free C-string block. `CString::new` drops interior
            // NULs so the terminator marks the true end for C consumers.
            syn::parse_quote!(
                #[allow(non_snake_case, dead_code)]
                pub(crate) fn __cbg_alloc_cstr(
                    s: ::std::string::String,
                ) -> *mut ::core::ffi::c_char {
                    let c = ::std::ffi::CString::new(s).unwrap_or_default();
                    let bytes = c.as_bytes_with_nul();
                    unsafe {
                        let p = malloc(bytes.len()) as *mut u8;
                        if p.is_null() {
                            return ::core::ptr::null_mut();
                        }
                        ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                        p as *mut ::core::ffi::c_char
                    }
                }
            ),
            // Universal raw memory freer: type-agnostic C `free`, no length, no
            // destructor (NULL-safe via C `free`).
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #free_ident(p: *mut ::core::ffi::c_void) {
                    free(p);
                }
            ),
        ];
        items
    }
}

/// One declared data struct's `#[repr(C)]` mirror.
///
/// The mirror is a layout fact: each field is stated in its wire form, which
/// is decided while planning, when the binding's declarations and the model
/// are both available.
pub(crate) struct CDataStruct {
    /// The C-facing struct name.
    c_struct: syn::Ident,
    /// Field name and wire type, in source order.
    fields: Vec<(syn::Ident, syn::Type)>,
}

impl CDataStruct {
    /// Plan the mirrors of every declared data struct that crosses.
    pub(crate) fn all(decls: &CbindgenBuilder, registry: &Registry) -> Vec<Self> {
        let mut mirrors = Vec::new();
        for (key, _cfg) in sorted_by_key(&decls.data) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if decls.in_frag(&reading).is_none() && decls.out_frag(&reading).is_none() {
                continue;
            }
            let Some(fields) = decls.struct_fields(registry, &reading.key()) else {
                continue;
            };
            mirrors.push(Self {
                c_struct: decls.c_type_ident(&reading.key()),
                fields: fields
                    .iter()
                    .map(|(name, ty)| {
                        let wire = decls.data_field_wire(ty).unwrap_or_else(|| {
                            panic!(
                                "Cbindgen: field `{}` of data struct `{}` has unsupported type `{}`",
                                name,
                                type_short(&reading.key()),
                                ty
                            )
                        });
                        (name.clone(), wire)
                    })
                    .collect(),
            });
        }
        mirrors
    }
}

impl RustArtifact for CDataStruct {
    fn calls(&self) -> Vec<ArtifactKey> {
        Vec::new()
    }

    fn key(&self) -> ArtifactKey {
        artifact_id("c-data-struct", self.c_struct.to_string())
    }

    fn render(&self, _emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        let c_struct = &self.c_struct;
        let fields = self
            .fields
            .iter()
            .map(|(name, wire)| quote!(pub #name: #wire));
        vec![syn::parse_quote!(
            #[repr(C)]
            #[allow(non_camel_case_types)]
            pub struct #c_struct {
                #(#fields,)*
            }
        )]
    }
}

/// One declared fieldless enum's `#[repr(C)]` mirror.
///
/// Each discriminant is re-stated **as written** — `= 0x07` stays `0x07` —
/// which is what keeps every value C already accepted: a `const` or
/// `cfg`-driven expression, and anything the source's own `repr` admits.
/// Resolving each to a number would narrow that to what `i64` and a literal
/// can express, for no gain, since cbindgen re-reads this as Rust source.
///
/// That is why the model's own values are retained here and spelled by the
/// writer, rather than the mirror being spelled while planning.
pub(crate) struct CEnum {
    /// The C-facing enum name.
    c_name: syn::Ident,
    /// The source enum's values, in declaration order.
    values: Vec<prebindgen_registry::flat::EnumValue>,
}

impl CEnum {
    /// Plan the mirrors of every declared fieldless enum that crosses.
    pub(crate) fn all(decls: &CbindgenBuilder, registry: &Registry) -> Vec<Self> {
        let mut mirrors = Vec::new();
        for (key, _cfg) in sorted_by_key(&decls.enums) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if decls.in_frag(&reading).is_none() && decls.out_frag(&reading).is_none() {
                continue;
            }
            let Some(item) = unit_enum(registry, &reading.key()) else {
                continue;
            };
            mirrors.push(Self {
                c_name: decls.c_type_ident(&reading.key()),
                values: item.values.clone(),
            });
        }
        mirrors
    }
}

impl RustArtifact for CEnum {
    fn calls(&self) -> Vec<ArtifactKey> {
        Vec::new()
    }

    fn key(&self) -> ArtifactKey {
        artifact_id("c-enum", self.c_name.to_string())
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        let c_name = &self.c_name;
        let variants = self.values.iter().map(|value| {
            let id = &value.name;
            match emit.discriminant(value) {
                Some(expr) => quote!(#id = #expr),
                None => quote!(#id),
            }
        });
        vec![syn::parse_quote!(
            #[repr(C)]
            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
            #[allow(non_camel_case_types)]
            pub enum #c_name {
                #(#variants),*
            }
        )]
    }
}

/// One reserved representation value a generated sum-type ABI uses.
pub(crate) struct CDomainConstant {
    /// The exported constant name.
    name: syn::Ident,
    /// The scalar type the value is stated in.
    ty: syn::Type,
    /// The value itself.
    value: syn::Expr,
    /// Documentation, which differs between a niche slot and the `None` of
    /// the first optional layer.
    doc: &'static str,
}

impl RustArtifact for CDomainConstant {
    fn calls(&self) -> Vec<ArtifactKey> {
        Vec::new()
    }

    fn key(&self) -> ArtifactKey {
        artifact_id("c-domain-constant", self.name.to_string())
    }

    fn render(&self, _emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        let (name, ty, value, doc) = (&self.name, &self.ty, &self.value, self.doc);
        vec![syn::parse_quote!(
            #[doc = #doc]
            pub const #name: #ty = #value;
        )]
    }
}

impl CDomainConstant {
    /// Plan every reserved value the declared conversions need.
    pub(crate) fn all(decls: &CbindgenBuilder, registry: &Registry) -> Vec<Self> {
        decls.domain_constants(registry)
    }

    /// Build one, named by its base and slot index.
    pub(crate) fn niche(base: &str, index: usize, ty: syn::Type, value: syn::Expr) -> Self {
        Self {
            name: format_ident!("{}_NICHE_{}", base, index),
            ty,
            value,
            doc: "Reserved representation value used by generated sum-type ABIs.",
        }
    }

    /// Build the `None` of the first optional layer, which shares the first
    /// niche's value.
    pub(crate) fn none(base: &str, ty: syn::Type, value: syn::Expr) -> Self {
        Self {
            name: format_ident!("{}_NONE", base),
            ty,
            value,
            doc: "Representation of None for the first optional layer.",
        }
    }
}
