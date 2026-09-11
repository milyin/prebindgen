//! The C target, as the v2 engine asks it questions.
//!
//! This is the whole of what C contributes to a v2 run: which relation a value
//! crosses through, what carries it, how an exported function is shaped, and
//! what public Rust its declarations need. There is no type walk here and no
//! control flow — the registry owns both — which is why the adapter is a
//! handful of local answers rather than a pipeline of its own.
//!
//! Names arrive in the policies. Which C name a type or a symbol gets is the
//! frontend's manglers' business, settled when the requests are built in
//! [`super`], so the target never spells a name of its own.

use prebindgen_registry::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_registry_v2::{
    AbiSpec, Access, Artifact, BoundarySpec, ChildValue, ElementId, ElementKind, Layout,
    NativeParam, OperandSpec, Operation, OperationType, OutputPlacement, ParamRole, PlanningError,
    PrimitiveFailure, PrimitiveSpec, Protocol, Relation, RelationId, ReprSpec, ResolvedShape,
    ResolvedValues, SelectionQuery, SiteDescriptor, SourceItem, StandardOp, SurfaceRequest,
    SurfaceSpec, Target, TargetAttempt, TargetSupport, Unsupported, WireType,
};
use quote::{format_ident, quote};

/// What the C frontend recorded for one value or one exported function.
#[derive(Clone, Debug)]
pub enum CPolicy {
    /// A scalar crossing unchanged. The default for every value nothing more
    /// specific covers.
    Scalar,
    /// A `repr(C)` aggregate passed by value, under this C name.
    DataStruct { c_name: String },
    /// An exported function, under this symbol.
    Function { symbol: String },
    /// A declaration v1 lowers and v2 does not yet: an opaque handle, an enum,
    /// a value-opaque type, a tagged union. Carries the declarator's name so
    /// the refusal says which capability is missing.
    Unimplemented { declarator: &'static str },
}

/// C contributes no operation of its own: reading an aggregate member is a
/// standard operation the registry renders, and a scalar crosses as itself.
/// This type has no values, which is that fact stated so the compiler keeps
/// it true.
#[derive(Clone, Debug)]
pub enum CPayload {}

/// The C target. Stateless: everything it needs arrives in a policy.
pub struct CTarget;

/// The C carrier for a scalar, when this adapter has one.
///
/// One scalar today, as the specification's element paths need. The rest of
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

/// The declared name of a nominal type.
fn named(ty: &TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        _ => None,
    }
}

impl Target for CTarget {
    type Policy = CPolicy;
    type Payload = CPayload;

    fn select(&self, query: &SelectionQuery<'_, CPolicy>) -> TargetSupport<RelationId> {
        // An aggregate carries its members, so it wants the record's fields; a
        // scalar is carried whole. A declarator v2 has no lowering for is
        // refused here, before anything under it is planned.
        let want_record = match query.policy {
            CPolicy::DataStruct { .. } => true,
            CPolicy::Scalar | CPolicy::Function { .. } => false,
            CPolicy::Unimplemented { declarator } => {
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
            match (relation, want_record) {
                (Relation::Record(_), true) => return Ok(TargetAttempt::Ready(*id)),
                (Relation::Atomic, false) => return Ok(TargetAttempt::Ready(*id)),
                _ => {}
            }
        }
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.c.no_relation",
            format!(
                "no relation available for `{}` under this C policy",
                query.crossing.ty.key()
            ),
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        policy: &CPolicy,
    ) -> TargetSupport<ReprSpec<CPayload>> {
        match shape.relation {
            Relation::Atomic => {
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
                }))
            }
            Relation::Record(record) => {
                let CPolicy::DataStruct { c_name } = policy else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is planned through its fields under a policy that carries it whole",
                        record.record
                    )));
                };
                let Some(item) = shape.record else {
                    return Err(PlanningError::InternalInvariant(
                        "a record relation without its record".to_string(),
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
                }))
            }
        }
    }

    fn boundary(
        &self,
        site: &SiteDescriptor<'_>,
        values: &ResolvedValues<'_, CPayload>,
        policy: &CPolicy,
    ) -> TargetSupport<BoundarySpec<CPayload>> {
        let symbol = match policy {
            CPolicy::Function { symbol } => symbol,
            CPolicy::Unimplemented { declarator } => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.c.{declarator}"),
                    format!(
                        "`{}` is declared as a `{declarator}`, which the v2 C target does not \
                         lower yet",
                        site.element.rust_origin
                    ),
                )));
            }
            CPolicy::Scalar | CPolicy::DataStruct { .. } => {
                return Err(PlanningError::InvalidInput(format!(
                    "`{}` is exported under a policy that is not a function policy",
                    site.element.rust_origin
                )));
            }
        };
        // A C parameter keeps the source parameter's name: that is what the
        // header shows, and what v1 shows.
        let params = values
            .inputs
            .iter()
            .zip(&site.function.params)
            .enumerate()
            .map(|(index, (value, param))| NativeParam {
                name: param.name.clone(),
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
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            // Nothing here can fail: a member read is infallible and a scalar
            // crosses unchanged. A fallible conversion would need a route, and
            // the function would be skipped until this list has one.
            failures: Vec::new(),
        }))
    }

    fn surface(
        &self,
        request: &SurfaceRequest<'_, CPolicy>,
        values: &ResolvedValues<'_, CPayload>,
    ) -> TargetSupport<SurfaceSpec<CPayload>> {
        match request.item {
            SourceItem::Function(_) => Ok(TargetAttempt::Ready(SurfaceSpec {
                element: request.element.id.clone(),
                // A wrapper taking an aggregate is unusable unless the public
                // type it names is emitted too.
                requires: values
                    .inputs
                    .iter()
                    .filter_map(|value| named(&value.crossing.ty))
                    .map(|name| ElementId::new(ElementKind::Type, name))
                    .collect(),
                rust: Vec::new(),
                payload: None,
            })),
            SourceItem::Record(record) => {
                let CPolicy::DataStruct { c_name } = request.policy else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as a data type under a policy that is not one",
                        record.name
                    )));
                };
                if record.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.empty_aggregate",
                        format!(
                            "`{c_name}` has no fields, and an empty aggregate has no portable \
                             C form"
                        ),
                    )));
                }
                let ident = format_ident!("{c_name}");
                let mut fields = Vec::new();
                for field in &record.fields {
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
                    fields.push(quote!(pub #name: #ty));
                }
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    element: request.element.id.clone(),
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
