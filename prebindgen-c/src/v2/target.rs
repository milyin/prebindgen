//! The C target, as the v2 engine asks it questions.
//!
//! This is the whole of what C contributes to a v2 run: which relation a value
//! crosses through, what carries it, how an exported function is shaped, and
//! what public Rust its declarations need. There is no type walk here and no
//! control flow — the registry owns both — which is why the adapter is a
//! handful of local answers rather than a pipeline of its own.
//!
//! It also *holds* what the binding declared. The registry stores no
//! configuration and knows no precedence: which C name a type gets, which
//! declarator produced a declaration, which destructor frees a handle are all
//! recorded here by [`super`] and looked up here when the registry asks
//! (#766). The names themselves are the frontend's manglers' business, settled
//! when the binding is built, so the target never spells one of its own.

use std::collections::BTreeMap;

use prebindgen_registry::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_registry_v2::{
    mirrored_enum, AbiSpec, Access, Artifact, BoundarySpec, ChildValue, Declaration, Direction,
    EnumArm, FailureCategory, FailureRoute, Layout, OperandSpec, Operation, OperationType,
    OutputPlacement, ParamRole, PlanningError, PrimitiveFailure, PrimitiveSpec, Protocol, Relation,
    ReprSpec, Requirement, ResolvedShape, ResolvedValues, Selection, SelectionQuery,
    SiteDescriptor, SourceItem, StandardOp, SurfaceRequest, SurfaceSpec, Target, TargetAttempt,
    TargetSupport, Terminal, Unsupported, WireType, WrapperParam,
};
use quote::{format_ident, quote};

/// What the C frontend recorded for one value or one exported function.
///
/// Also this target's [`Target::ConversionKey`]: it is plain data, so two
/// values the binding declared the same way convert the same way — which is
/// what the key has to mean for the registry to reuse one conversion for both.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CChoice {
    /// A scalar crossing unchanged. The default for every value nothing more
    /// specific covers.
    Scalar,
    /// A `repr(C)` aggregate passed by value, under this C name.
    DataStruct { c_name: String },
    /// An opaque handle: C holds a `<c_name> *` to a Rust-owned value and
    /// frees it through the `release` symbol.
    OpaquePtr { c_name: String, release: String },
    /// A fieldless enum: C declares an enum of the same values under this
    /// name, and a value of the type crosses as one of them.
    Enum { c_name: String },
    /// A source function to expose: the wrapper around it carries this symbol.
    Function { symbol: String },
    /// A declaration v1 lowers and v2 does not yet: an enum, a value-opaque
    /// type, a tagged union. Carries the declarator's name so the refusal says
    /// which capability is missing, and the C name it would have had so the
    /// report can say where it was going.
    Unimplemented {
        declarator: &'static str,
        c_name: String,
    },
}

/// C contributes no operation of its own: reading an aggregate member, and
/// going between a source enum and the C one declared for it, are standard
/// operations the registry renders — it is the registry that can spell a
/// source path. This type has no values, which is that fact stated so the
/// compiler keeps it true.
#[derive(Clone, Debug)]
pub enum CPayload {}

/// The C target: what the binding declared, and the answers the registry gets
/// out of it.
#[derive(Default)]
pub struct CTarget {
    /// How every value of this source type crosses, wherever it appears, by
    /// the type's canonical key.
    types: BTreeMap<String, CChoice>,
}

impl CTarget {
    /// Record how a declared type's values cross.
    ///
    /// What each output *is* travels with it: the engine holds the binding's
    /// declarations as the pair of what was declared and what this target
    /// recorded for it, and hands the choice back with every question about
    /// that output. This table answers the other question — how a value of
    /// the type crosses wherever else it turns up.
    pub(crate) fn declare(&mut self, declaration: &Declaration, choice: &CChoice) {
        if let Declaration::Type(key) = declaration {
            self.types.insert(key.as_str().to_string(), choice.clone());
        }
    }

    /// How a value of this type crosses: what was declared for its type, else
    /// a scalar.
    ///
    /// The position goes unread. C has no per-site declarator — `data_type!`
    /// and `ptr_type!` are stated about a type, not about one parameter of one
    /// function — so every value of a type crosses the same way, and the key
    /// this yields depends on the type alone. Looked up by the type's key,
    /// which is what the declaration was recorded by: `String` is a kind of
    /// its own to the model, not a named type, and `ptr_type!(String)` has to
    /// find it all the same.
    fn conversion(&self, ty: &TypeRef) -> CChoice {
        self.types
            .get(ty.key().as_str())
            .cloned()
            .unwrap_or(CChoice::Scalar)
    }
}

/// The C carrier for a scalar, when this adapter has one.
///
/// One scalar today, as the specification's declaration paths need. The rest of
/// `ScalarKind` arrives with its own increment; until then a value of another
/// kind is a reported skip, never a guess.
fn c_scalar(kind: ScalarKind) -> Option<syn::Type> {
    match kind {
        ScalarKind::I64 => Some(syn::parse_quote!(i64)),
        _ => None,
    }
}

/// The scalar kind of a type, when it is one.
fn scalar_of(ty: &TypeRef) -> Option<ScalarKind> {
    match ty.kind() {
        TypeKind::Scalar(kind) => Some(*kind),
        _ => None,
    }
}

impl Target for CTarget {
    const NAME: &'static str = "c";

    type ConversionKey = CChoice;
    type Payload = CPayload;

    fn select(&self, query: &SelectionQuery<'_, CChoice>) -> TargetSupport<Selection<CChoice>> {
        // A declared type's own crossing is planned as that declaration says,
        // not as the per-type default for values of it: the two agree for a
        // type declared once, and differ by design for one declared twice.
        let conversion = match query.position.is_declared_type() {
            true => query.declared.clone(),
            false => self.conversion(&query.crossing.ty),
        };
        // An aggregate carries its members, so it wants the struct's fields; a
        // scalar is carried whole. A declarator v2 has no lowering for is
        // refused here, before anything under it is planned — never quietly
        // crossed as the scalar default.
        let want_struct = match &conversion {
            CChoice::DataStruct { .. } => true,
            CChoice::Scalar
            | CChoice::Enum { .. }
            | CChoice::OpaquePtr { .. }
            | CChoice::Function { .. } => false,
            CChoice::Unimplemented { declarator, .. } => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.c.{declarator}"),
                    format!(
                        "`{}` is declared with `{declarator}`, which the v2 C target does not \
                         lower yet",
                        query.crossing.ty.key()
                    ),
                )));
            }
        };
        for (id, relation) in query.candidates {
            match (relation, want_struct) {
                (Relation::Struct(_), true) | (Relation::Atomic, false) => {
                    return Ok(TargetAttempt::Ready(Selection {
                        relation: *id,
                        conversion,
                    }))
                }
                _ => {}
            }
        }
        // A data struct is read through fields, and the model offers none for
        // this type: it is an extern, captured or the binding's own, and there
        // is nothing to see into.
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.c.not_a_struct",
            format!(
                "`{}` is declared as a data struct, and the model has no fields for it",
                query.crossing.ty.key()
            ),
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        conversion: &CChoice,
    ) -> TargetSupport<ReprSpec<CPayload>> {
        match (shape.relation, conversion) {
            // The address of a boxed source value, cast to a pointer to the
            // incomplete C type this adapter declares. Both directions and the
            // release are the registry's standard operations; the adapter
            // states only the carrier.
            (Relation::Atomic, CChoice::OpaquePtr { c_name, .. }) => {
                let ident = format_ident!("{c_name}");
                let carrier = WireType::abi(syn::parse_quote!(*mut #ident));
                let ty = shape.crossing.ty.clone();
                Ok(TargetAttempt::Ready(match shape.crossing.direction {
                    Direction::IntoRust => ReprSpec {
                        layout: Layout::Scalar(carrier.clone()),
                        protocol: Protocol::terminal(PrimitiveSpec::from_raw(
                            carrier.clone(),
                            ty.clone(),
                        )),
                        release: Some(PrimitiveSpec::release(carrier, ty)),
                    },
                    Direction::OutOfRust => ReprSpec {
                        layout: Layout::Scalar(carrier.clone()),
                        protocol: Protocol::terminal(PrimitiveSpec::into_raw(ty, carrier)),
                        release: None,
                    },
                }))
            }
            // A fieldless enum crosses as the C enum this target declares for
            // it: the same values, so the match either way is exhaustive and
            // cannot fail.
            (Relation::Atomic, CChoice::Enum { c_name }) => {
                let unit = match mirrored_enum(shape.unit, c_name, Self::NAME) {
                    Ok(unit) => unit,
                    Err(reason) => return Ok(TargetAttempt::Unsupported(reason)),
                };
                let ident = format_ident!("{c_name}");
                let carrier = WireType::abi(syn::parse_quote!(#ident));
                let ty = shape.crossing.ty.clone();
                // Each value is carried as the same value of the C enum, so
                // both directions name every value and neither can fail.
                let values: Vec<EnumArm> = unit
                    .iter()
                    .map(|value| {
                        let name = &value.name;
                        EnumArm {
                            name: name.clone(),
                            shape: value.shape,
                            carried: syn::parse_quote!(#ident::#name),
                        }
                    })
                    .collect();
                let codec = match shape.crossing.direction {
                    Direction::OutOfRust => PrimitiveSpec {
                        operands: vec![OperandSpec::value(
                            OperationType::Source(ty.clone()),
                            Access::Owned,
                        )],
                        result: Some(OperationType::Carrier(carrier.clone())),
                        failure: PrimitiveFailure::Infallible,
                        dependencies: Vec::new(),
                        implementation: Operation::Standard(StandardOp::EnumOut {
                            source: Box::new(ty),
                            values,
                        }),
                    },
                    Direction::IntoRust => PrimitiveSpec {
                        operands: vec![OperandSpec::value(
                            OperationType::Carrier(carrier.clone()),
                            Access::Owned,
                        )],
                        result: Some(OperationType::Source(ty.clone())),
                        failure: PrimitiveFailure::Infallible,
                        dependencies: Vec::new(),
                        implementation: Operation::Standard(StandardOp::EnumIn {
                            source: Box::new(ty),
                            values,
                            invalid: None,
                        }),
                    },
                };
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier),
                    protocol: Protocol::terminal(codec),
                    release: None,
                }))
            }
            (Relation::Atomic, _) => {
                let Some(carrier) = scalar_of(&shape.crossing.ty).and_then(c_scalar) else {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.carrier",
                        format!("`{}` has no C carrier yet", shape.crossing.ty.key()),
                    )));
                };
                let carrier = WireType::abi(carrier);
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier.clone()),
                    // The C carrier of an `i64` *is* the `i64`, so the
                    // conversion renders nothing at all.
                    protocol: Protocol::terminal(PrimitiveSpec::identity(OperationType::Carrier(
                        carrier,
                    ))),
                    release: None,
                }))
            }
            (Relation::Struct(strukt), _) => {
                let CChoice::DataStruct { c_name } = conversion else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is planned through its fields, and is declared to be carried whole",
                        strukt.name
                    )));
                };
                let Some(item) = shape.strukt else {
                    return Err(PlanningError::InternalInvariant(
                        "a struct relation without its strukt".to_string(),
                    ));
                };
                if item.fields.is_empty() {
                    // A `repr(C)` struct with no members has no portable C
                    // representation, and rustc's FFI lint says so about
                    // passing one across an `extern "C"` boundary.
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.empty_aggregate",
                        format!(
                            "`{c_name}` has no fields, and an empty aggregate has no portable \
                             C form"
                        ),
                    )));
                }
                let aggregate = WireType::abi({
                    let ident = format_ident!("{c_name}");
                    syn::parse_quote!(#ident)
                });
                let mut members = Vec::new();
                let mut projections = Vec::new();
                for (field, child) in item.fields.iter().zip(children) {
                    if !matches!(child.layout, Layout::Scalar(_)) {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.c.nested_member",
                            format!(
                                "member `{}` of `{c_name}` needs a nested aggregate member",
                                child.part.label()
                            ),
                        )));
                    }
                    let member = field.member();
                    members.push(member.clone());
                    projections.push(PrimitiveSpec {
                        operands: vec![OperandSpec::value(
                            OperationType::Carrier(aggregate.clone()),
                            Access::Shared,
                        )],
                        result: Some(OperationType::Carrier(child.layout.wire().clone())),
                        // Reading a member of a by-value struct cannot fail,
                        // and the copied value owes nothing to the aggregate.
                        failure: PrimitiveFailure::Infallible,
                        dependencies: Vec::new(),
                        implementation: Operation::Standard(StandardOp::ReadMember { member }),
                    });
                }
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Aggregate {
                        ty: aggregate,
                        members,
                    },
                    protocol: Protocol::Product { projections },
                    release: None,
                }))
            }
        }
    }

    fn boundary(
        &self,
        site: &SiteDescriptor<'_, CChoice>,
        values: &ResolvedValues<'_, CPayload>,
    ) -> TargetSupport<BoundarySpec<CPayload>> {
        // A handle's release is a site with no source function, exported under
        // the destructor symbol its declaration named.
        let symbol = match (site.declared, site.function) {
            (CChoice::Function { symbol }, Some(_)) => symbol,
            (CChoice::OpaquePtr { release, .. }, None) => release,
            (CChoice::Unimplemented { declarator, .. }, _) => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.c.{declarator}"),
                    format!(
                        "`{}` is declared as a `{declarator}`, which the v2 C target does not \
                         lower yet",
                        site.declaration
                    ),
                )));
            }
            _ => {
                return Err(PlanningError::InvalidInput(format!(
                    "`{}` is declared in a way that does not fit this site",
                    site.declaration
                )));
            }
        };
        // A C parameter keeps the source parameter's name: that is what the
        // header shows, and what v1 shows. A release has no source parameter
        // to take a name from, and takes v1's.
        let params = values
            .inputs
            .iter()
            .enumerate()
            .map(|(index, value)| WrapperParam {
                name: match site.function {
                    Some(function) => function.params[index].name.clone(),
                    None => format_ident!("this_"),
                },
                ty: value.repr.layout.wire().clone(),
                role: ParamRole::Input(index),
                mutable: false,
            })
            .collect();
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: AbiSpec {
                abi: "C".to_string(),
                symbol: symbol.clone(),
                params,
                ret: values.output.map(|value| value.repr.layout.wire().clone()),
                attrs: Vec::new(),
                unsafety: false,
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            // A member read is infallible and a scalar crosses unchanged; the
            // one thing that can fail is a handle arriving null, and C has no
            // exception to raise. The process stops, as it does under v1's
            // `.panic()`. A runtime failure has no route, and a function that
            // could raise one is skipped until it does.
            failures: vec![FailureRoute {
                category: FailureCategory::Binding,
                report: None,
                on_report_failure: Terminal::Abort,
                terminate: Terminal::Abort,
            }],
        }))
    }

    fn surface(
        &self,
        request: &SurfaceRequest<'_, CChoice>,
        values: &ResolvedValues<'_, CPayload>,
    ) -> TargetSupport<SurfaceSpec<CPayload>> {
        let declared = request.declared;
        // An opaque handle is declared the same way whatever the item behind
        // it: an alias, or a struct whose fields C never sees.
        if let CChoice::OpaquePtr { c_name, .. } = declared {
            let ident = format_ident!("{c_name}");
            return Ok(TargetAttempt::Ready(SurfaceSpec {
                declaration: request.declaration.clone(),
                requires: Vec::new(),
                // A struct whose only member is a zero-length array is what
                // `cbindgen` renders as an incomplete type: a C caller can
                // hold a pointer to one and nothing else. The same declaration
                // v1 emits, case lint included.
                rust: vec![Artifact::new(
                    c_name.clone(),
                    quote! {
                        #[repr(C)]
                        #[allow(non_camel_case_types)]
                        pub struct #ident {
                            _private: [u8; 0],
                        }
                    },
                )],
                payload: None,
            }));
        }
        match request.item {
            SourceItem::Extern(opaque) => Err(PlanningError::InvalidInput(format!(
                "`{}` has no fields, and is declared as something that reads them",
                opaque.name
            ))),
            SourceItem::Function(_) => Ok(TargetAttempt::Ready(SurfaceSpec {
                declaration: request.declaration.clone(),
                // A wrapper taking or returning a declared type is unusable
                // unless the public type it names is emitted too.
                requires: values
                    .inputs
                    .iter()
                    .chain(values.output.iter())
                    .filter_map(|value| Requirement::of(value))
                    .collect(),
                rust: Vec::new(),
                payload: None,
            })),
            // The C enum itself: the same values under the same names, and
            // the numbers Rust assigns, so a C caller reading the header sees
            // what a Rust caller sees.
            SourceItem::Enum(unit) => {
                let CChoice::Enum { c_name } = declared else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as an enum, and is declared as something else",
                        unit.name
                    )));
                };
                if let Err(reason) = mirrored_enum(Some(unit), c_name, Self::NAME) {
                    return Ok(TargetAttempt::Unsupported(reason));
                }
                let ident = format_ident!("{c_name}");
                let values = unit
                    .discriminant_values()
                    .expect("the values were checked before the mirror was built")
                    .into_iter()
                    .map(|(name, number)| {
                        let number = proc_macro2::Literal::i64_unsuffixed(number);
                        quote!(#name = #number)
                    });
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    declaration: request.declaration.clone(),
                    requires: Vec::new(),
                    rust: vec![Artifact::new(
                        c_name.clone(),
                        quote! {
                            #[repr(C)]
                            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
                            #[allow(non_camel_case_types)]
                            pub enum #ident {
                                #(#values),*
                            }
                        },
                    )],
                    payload: None,
                }))
            }
            SourceItem::Struct(strukt) => {
                let CChoice::DataStruct { c_name } = declared else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as a data type, and is declared as something else",
                        strukt.name
                    )));
                };
                if strukt.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.empty_aggregate",
                        format!(
                            "`{c_name}` has no fields, and an empty aggregate has no portable \
                             C form"
                        ),
                    )));
                }
                let ident = format_ident!("{c_name}");
                // A member mirrors a field one for one, its condition included:
                // a field the source crate may not have must not become a
                // member the header always declares.
                let conditions = request.field_conditions();
                let mut fields = Vec::new();
                for (index, field) in strukt.fields.iter().enumerate() {
                    let Some(ty) = scalar_of(&field.ty).and_then(c_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.c.carrier",
                            format!("field `{}` of `{c_name}` has no C carrier yet", field.index),
                        )));
                    };
                    let Some(name) = &field.name else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.c.positional_field",
                            format!(
                                "field {} of `{c_name}` has no name, and a C member needs one",
                                field.index
                            ),
                        )));
                    };
                    let condition = &conditions[index];
                    // cbindgen guards a member only for a condition its
                    // `[defines]` table names; with no entry it writes the
                    // member unguarded and says nothing, because its warning
                    // goes through `log` and a build script driving its library
                    // API installs no logger. The header then declares a member
                    // the library may not have, which no compiler or linker
                    // catches — the caller and the library simply disagree
                    // about the struct's size. This line is the only output
                    // such a build produces, so it names the fix; it cannot
                    // tell whether the fix is already in place, since reading
                    // the consumer's cbindgen configuration would cost a
                    // dependency for a warning.
                    for under in condition {
                        // Tokens print with a space between each pair, which
                        // `close_up` removes where it separates no two words —
                        // the same treatment a type key gets in the report.
                        let under = prebindgen_registry::close_up(&under.to_string());
                        println!(
                            "cargo:warning=prebindgen: `{c_name}.{name}` is emitted under \
                             {under}; unless your cbindgen configuration already maps that \
                             condition in [defines], the header declares the member \
                             unconditionally and a C caller disagrees with the library \
                             about the layout of `{c_name}`"
                        );
                    }
                    fields.push(quote!(#(#condition)* pub #name: #ty));
                }
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    declaration: request.declaration.clone(),
                    requires: Vec::new(),
                    // `repr(C)` is required: without it the layout the header
                    // promises is not the layout the wrapper reads. The C name
                    // is the mangler's — `foo_t` — so the case lint is silenced
                    // as v1 silences it.
                    rust: vec![Artifact::new(
                        c_name.clone(),
                        quote! {
                            #[repr(C)]
                            #[allow(non_camel_case_types)]
                            pub struct #ident { #(#fields),* }
                        },
                    )],
                    payload: None,
                }))
            }
        }
    }

    fn render_operation(&self, payload: &CPayload, _: &[syn::Ident]) -> proc_macro2::TokenStream {
        match *payload {}
    }
}
