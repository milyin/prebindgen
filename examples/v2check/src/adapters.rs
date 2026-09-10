//! Two language adapters against the v2 target interface: one C, one JNI.
//!
//! They are what `docs/v2` claims an adapter is — a handful of local answers,
//! with no type walk, no name allocation and no control flow of their own. Both
//! are compiled by `build.rs` (through `#[path]`, so there is one copy) and are
//! deliberately small: what they demonstrate is the size of a target's first
//! version, not the completeness of the real `prebindgen-c` and
//! `prebindgen-jni`.
//!
//! Neither adapter spells a source type. Its carriers are its own — a `repr(C)`
//! aggregate it declares, `JObject`, `jlong` — and every one of them is chosen
//! from a `ScalarKind` or from the layout of an already-planned child, never
//! from the text of a signature.

use prebindgen_flat::flat::{ScalarKind, TypeKind};
use prebindgen_registry_v2::{
    Access, Artifact, BoundarySpec, ChildValue, Direction, ElementId, ElementKind, FailureCategory,
    FailureRoute, Generation, Layout, NativeParam, OperandSpec, Operation, OperationType,
    OutputPlacement, ParamRole, PlanningError, PrimitiveFailure, PrimitiveSpec, Protocol, Relation,
    RelationId, ReprSpec, ResolvedShape, ResolvedValues, SelectionQuery, SiteDescriptor,
    SourceItem, StandardOp, SurfaceRequest, SurfaceSpec, Target, TargetAttempt, TargetSupport,
    Terminal, Unsupported, WireType,
};
use quote::{format_ident, quote};

/// The C carrier for a scalar: what `cbindgen` will write as `int64_t`.
fn c_scalar(kind: ScalarKind) -> Option<syn::Type> {
    Some(match kind {
        ScalarKind::I8 => syn::parse_quote!(i8),
        ScalarKind::I16 => syn::parse_quote!(i16),
        ScalarKind::I32 => syn::parse_quote!(i32),
        ScalarKind::I64 => syn::parse_quote!(i64),
        ScalarKind::U8 => syn::parse_quote!(u8),
        ScalarKind::U16 => syn::parse_quote!(u16),
        ScalarKind::U32 => syn::parse_quote!(u32),
        ScalarKind::U64 => syn::parse_quote!(u64),
        ScalarKind::F32 => syn::parse_quote!(f32),
        ScalarKind::F64 => syn::parse_quote!(f64),
        // `bool`, `isize` and `usize` cross C deliberately or not at all; the
        // increment says so rather than guessing a width.
        _ => return None,
    })
}

/// The scalar kind of a type, when it is one.
fn scalar_of(ty: &prebindgen_flat::flat::TypeRef) -> Option<ScalarKind> {
    match ty.kind() {
        TypeKind::Scalar(kind) => Some(*kind),
        _ => None,
    }
}

/// The declared name of a nominal type.
fn named(ty: &prebindgen_flat::flat::TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// C
// ---------------------------------------------------------------------------

/// What the C frontend recorded for one value or one function.
#[derive(Clone, Debug)]
pub enum CPolicy {
    /// A scalar crossing unchanged.
    Scalar,
    /// A `repr(C)` aggregate passed by value, under this name.
    DataStruct { c_name: String },
    /// An exported function, under this symbol.
    Function { symbol: String },
}

/// C contributes no operation of its own: reading an aggregate member is a
/// standard operation the registry renders. This type has no values, which is
/// that fact stated so the compiler keeps it true.
#[derive(Clone, Debug)]
pub enum CPayload {}

pub struct CTarget;

impl Target for CTarget {
    type Policy = CPolicy;
    type Payload = CPayload;

    fn select(&self, query: &SelectionQuery<'_, CPolicy>) -> TargetSupport<RelationId> {
        // An aggregate carries its members, so it wants the record's fields; a
        // scalar is carried whole.
        let want_record = matches!(query.policy, CPolicy::DataStruct { .. });
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
                let carrier = scalar_of(&shape.crossing.ty).and_then(c_scalar);
                let Some(carrier) = carrier else {
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
        let CPolicy::Function { symbol } = policy else {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` is exported under a policy that is not a function policy",
                site.element.rust_origin
            )));
        };
        let params = values
            .inputs
            .iter()
            .enumerate()
            .map(|(index, value)| NativeParam {
                name: format_ident!("arg{index}"),
                ty: value.repr.layout.wire().clone(),
                role: ParamRole::Input(index),
                mutable: false,
            })
            .collect();
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: prebindgen_registry_v2::AbiSpec {
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
                let ident = format_ident!("{c_name}");
                let mut fields = Vec::new();
                for field in &record.fields {
                    let Some(ty) = scalar_of(&field.ty).and_then(c_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.c.carrier",
                            format!("field `{}` of `{c_name}` has no C carrier yet", field.index),
                        )));
                    };
                    let name = field
                        .name
                        .clone()
                        .unwrap_or_else(|| format_ident!("field{}", field.index));
                    fields.push(quote!(pub #name: #ty));
                }
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    element: request.element.id.clone(),
                    requires: Vec::new(),
                    // `repr(C)` is required: without it the layout the header
                    // promises is not the layout the wrapper reads.
                    rust: vec![Artifact::new(
                        c_name.clone(),
                        quote! {
                            #[repr(C)]
                            pub struct #ident { #(#fields),* }
                        },
                    )],
                    payload: None,
                }))
            }
        }
    }

    fn render_operation(
        &self,
        payload: &CPayload,
        _operands: &[syn::Ident],
    ) -> proc_macro2::TokenStream {
        // C ships no operation renderer, and this is what that means.
        match *payload {}
    }
}

// ---------------------------------------------------------------------------
// Kotlin, through JNI
// ---------------------------------------------------------------------------

/// Which Kotlin class each Rust type is declared as.
///
/// One table, read by the declaration and by every signature naming it, because
/// a class named in two places is a class that can be renamed in one.
#[derive(Clone, Debug, Default)]
pub struct JniClasses {
    package: String,
    classes: std::collections::HashMap<String, String>,
}

impl JniClasses {
    pub fn in_package(package: impl Into<String>) -> Self {
        JniClasses {
            package: package.into(),
            classes: std::collections::HashMap::new(),
        }
    }

    /// Declare that this Rust type crosses as that Kotlin class.
    pub fn with(mut self, rust: impl Into<String>, kotlin: impl Into<String>) -> Self {
        self.classes.insert(rust.into(), kotlin.into());
        self
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn kotlin(&self, rust: &str) -> Option<&str> {
        self.classes.get(rust).map(String::as_str)
    }
}

/// What the JNI frontend recorded for one value or one function.
#[derive(Clone, Debug)]
pub enum JniPolicy {
    /// A scalar crossing as its JNI carrier.
    Scalar,
    /// A JVM object whose properties are read, declared as the Kotlin class
    /// this Rust type is registered under.
    DataClass { rust: String },
    /// An exported function at this Kotlin placement, `example.Bindings.sum`.
    Function { placement: String },
}

impl JniPolicy {
    fn placement(placement: &str) -> Option<(String, String, String)> {
        let mut parts: Vec<&str> = placement.rsplitn(3, '.').collect();
        parts.reverse();
        match parts.as_slice() {
            [package, object, method] => {
                Some((package.to_string(), object.to_string(), method.to_string()))
            }
            _ => None,
        }
    }
}

/// What the JNI adapter renders from: one operation, or one Kotlin
/// declaration.
#[derive(Clone, Debug)]
pub enum JniPayload {
    /// A property read through the JVM.
    Getter { name: String, descriptor: String },
    /// The error-reporting helper.
    ReportError,
    /// A Kotlin data class and its properties.
    Class {
        package: String,
        class: String,
        properties: Vec<(String, String)>,
    },
    /// A Kotlin object with one external method.
    Method {
        package: String,
        object: String,
        method: String,
        params: Vec<(String, String)>,
        ret: String,
    },
}

/// The JNI adapter.
///
/// It carries the Kotlin class name declared for each Rust type, because a
/// Kotlin signature has to name the class the frontend chose — not the Rust
/// type it was built from. Renaming a class in the configuration therefore
/// moves it in the declaration and in every signature mentioning it.
pub struct JniTarget {
    classes: JniClasses,
}

impl JniTarget {
    pub fn new(classes: JniClasses) -> Self {
        JniTarget { classes }
    }
}

/// The Kotlin type and JVM descriptor of a scalar.
///
/// One scalar, because one accessor: every getter this adapter describes is
/// read with `JValueOwned::j`, which extracts a long. A second width needs its
/// own accessor and its own carrier import, and until it has them, saying so is
/// the honest answer.
fn jvm_scalar(kind: ScalarKind) -> Option<(syn::Type, &'static str, &'static str)> {
    Some(match kind {
        ScalarKind::I64 => (syn::parse_quote!(jlong), "Long", "J"),
        _ => return None,
    })
}

/// The JVM getter a Kotlin `val secs` compiles to.
fn getter(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => format!("get{}{}", first.to_uppercase(), chars.as_str()),
        None => "get".to_string(),
    }
}

impl Target for JniTarget {
    type Policy = JniPolicy;
    type Payload = JniPayload;

    fn select(&self, query: &SelectionQuery<'_, JniPolicy>) -> TargetSupport<RelationId> {
        let want_record = matches!(query.policy, JniPolicy::DataClass { .. });
        for (id, relation) in query.candidates {
            match (relation, want_record) {
                (Relation::Record(_), true) => return Ok(TargetAttempt::Ready(*id)),
                (Relation::Atomic, false) => return Ok(TargetAttempt::Ready(*id)),
                _ => {}
            }
        }
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.jni.no_relation",
            format!(
                "no relation available for `{}` under this JNI policy",
                query.crossing.ty.key()
            ),
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        _policy: &JniPolicy,
    ) -> TargetSupport<ReprSpec<JniPayload>> {
        match shape.relation {
            Relation::Atomic => {
                let carrier = scalar_of(&shape.crossing.ty).and_then(jvm_scalar);
                let Some((carrier, _, _)) = carrier else {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.carrier",
                        format!("`{}` has no JNI carrier yet", shape.crossing.ty.key()),
                    )));
                };
                let carrier = WireType::abi(carrier);
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier.clone()),
                    // A source `i64` and a `jlong` are the same Rust value.
                    protocol: Protocol::terminal(PrimitiveSpec::identity(OperationType::Carrier(
                        carrier,
                    ))),
                }))
            }
            Relation::Record(record) => {
                if shape.crossing.direction != Direction::IntoRust {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.object_output",
                        format!(
                            "`{}` leaving Rust as a JVM object is not implemented",
                            record.record
                        ),
                    )));
                }
                let Some(item) = shape.record else {
                    return Err(PlanningError::InternalInvariant(
                        "a record relation without its record".to_string(),
                    ));
                };
                let object = WireType::abi(syn::parse_quote!(JObject<'_>));
                let mut projections = Vec::new();
                for (field, child) in item.fields.iter().zip(children) {
                    let Some(name) = field.name.as_ref() else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.positional_field",
                            "a positional field has no Kotlin property to read".to_string(),
                        )));
                    };
                    let Some((_, _, descriptor)) = scalar_of(&field.ty).and_then(jvm_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.carrier",
                            format!("property `{name}` has no JNI carrier yet"),
                        )));
                    };
                    projections.push(PrimitiveSpec {
                        operands: vec![
                            // The environment is an operand, not an ambient
                            // variable.
                            OperandSpec::context(
                                "jni.env",
                                OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                    JNIEnv<'_>
                                ))),
                                Access::Exclusive,
                            ),
                            OperandSpec::value(
                                OperationType::Carrier(object.clone()),
                                Access::Shared,
                            ),
                        ],
                        result: Some(OperationType::Carrier(child.layout.wire().clone())),
                        // A JVM call can fail, and the error is the jni crate's.
                        failure: PrimitiveFailure::fallible(
                            OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                jni::errors::Error
                            ))),
                            FailureCategory::Runtime,
                        ),
                        dependencies: Vec::new(),
                        implementation: Operation::Target(JniPayload::Getter {
                            name: getter(&name.to_string()),
                            descriptor: format!("(){descriptor}"),
                        }),
                    });
                }
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(object),
                    protocol: Protocol::Product { projections },
                }))
            }
        }
    }

    fn boundary(
        &self,
        site: &SiteDescriptor<'_>,
        values: &ResolvedValues<'_, JniPayload>,
        policy: &JniPolicy,
    ) -> TargetSupport<BoundarySpec<JniPayload>> {
        let JniPolicy::Function { placement } = policy else {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` is exported under a policy that is not a function policy",
                site.element.rust_origin
            )));
        };
        let Some((package, object, method)) = JniPolicy::placement(placement) else {
            return Err(PlanningError::InvalidInput(format!(
                "`{placement}` is not a package.Object.method placement"
            )));
        };
        // Placement and symbol are one choice: renaming the Kotlin method
        // renames the symbol the wrapper exports.
        let symbol = format!("Java_{}_{}_{}", package.replace('.', "_"), object, method);
        let mut params = vec![
            NativeParam {
                name: format_ident!("env"),
                ty: WireType::abi(syn::parse_quote!(JNIEnv<'_>)),
                role: ParamRole::Context("jni.env".to_string()),
                mutable: true,
            },
            NativeParam {
                name: format_ident!("_class"),
                ty: WireType::abi(syn::parse_quote!(JClass<'_>)),
                role: ParamRole::Unused,
                mutable: false,
            },
        ];
        params.extend(
            values
                .inputs
                .iter()
                .enumerate()
                .map(|(index, value)| NativeParam {
                    name: format_ident!("arg{index}"),
                    ty: value.repr.layout.wire().clone(),
                    role: ParamRole::Input(index),
                    mutable: false,
                }),
        );
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: prebindgen_registry_v2::AbiSpec {
                abi: "system".to_string(),
                symbol,
                params,
                ret: values.output.map(|value| value.repr.layout.wire().clone()),
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            // Zero is not a result: it is what a native method must return
            // while an exception is pending, and Kotlin observes the exception.
            failures: vec![FailureRoute {
                category: FailureCategory::Runtime,
                report: Some(PrimitiveSpec {
                    operands: vec![
                        OperandSpec::context(
                            "jni.env",
                            OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                JNIEnv<'_>
                            ))),
                            Access::Exclusive,
                        ),
                        OperandSpec::error(OperationType::Carrier(WireType::internal(
                            syn::parse_quote!(jni::errors::Error),
                        ))),
                    ],
                    result: None,
                    failure: PrimitiveFailure::fallible(
                        OperationType::Carrier(WireType::internal(syn::parse_quote!(
                            jni::errors::Error
                        ))),
                        FailureCategory::Runtime,
                    ),
                    dependencies: vec![Artifact::new("report_jni_error", report_jni_error())],
                    implementation: Operation::Target(JniPayload::ReportError),
                }),
                on_report_failure: Terminal::Abort,
                // Zero is not a result: it is what a native method must return
                // while an exception is pending. A wrapper that returns
                // nothing has to terminate with nothing.
                terminate: match values.output {
                    Some(_) => Terminal::Return(syn::parse_quote!(0)),
                    None => Terminal::Return(syn::parse_quote!(())),
                },
            }],
        }))
    }

    fn surface(
        &self,
        request: &SurfaceRequest<'_, JniPolicy>,
        values: &ResolvedValues<'_, JniPayload>,
    ) -> TargetSupport<SurfaceSpec<JniPayload>> {
        match request.item {
            SourceItem::Function(function) => {
                let JniPolicy::Function { placement } = request.policy else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exported under a policy that is not a function policy",
                        function.name
                    )));
                };
                let Some((package, object, method)) = JniPolicy::placement(placement) else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{placement}` is not a package.Object.method placement"
                    )));
                };
                let mut params = Vec::new();
                for (param, value) in function.params.iter().zip(&values.inputs) {
                    let kotlin = match named(&value.crossing.ty) {
                        Some(name) => match self.classes.kotlin(&name) {
                            Some(class) => class.to_string(),
                            None => {
                                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                                    "unsupported.jni.undeclared_class",
                                    format!("`{name}` has no declared Kotlin class"),
                                )))
                            }
                        },
                        None => match scalar_of(&value.crossing.ty).and_then(jvm_scalar) {
                            Some((_, kotlin, _)) => kotlin.to_string(),
                            None => {
                                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                                    "unsupported.jni.kotlin_type",
                                    format!("`{}` has no Kotlin spelling yet", param.name),
                                )))
                            }
                        },
                    };
                    params.push((param.name.to_string(), kotlin));
                }
                let ret = match values.output {
                    None => "Unit".to_string(),
                    Some(value) => match scalar_of(&value.crossing.ty).and_then(jvm_scalar) {
                        Some((_, kotlin, _)) => kotlin.to_string(),
                        None => {
                            return Ok(TargetAttempt::Unsupported(Unsupported::new(
                                "unsupported.jni.kotlin_type",
                                "this result has no Kotlin spelling yet".to_string(),
                            )))
                        }
                    },
                };
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    element: request.element.id.clone(),
                    requires: values
                        .inputs
                        .iter()
                        .filter_map(|value| named(&value.crossing.ty))
                        .map(|name| ElementId::new(ElementKind::Type, name))
                        .collect(),
                    // What crosses is a JVM object: the Rust side holds a
                    // reference to it and declares no type of its own. The one
                    // Rust this declaration contributes is the import its
                    // carriers are spelled with.
                    rust: vec![Artifact::new("jni_imports", jni_imports())],
                    payload: Some(JniPayload::Method {
                        package,
                        object,
                        method,
                        params,
                        ret,
                    }),
                }))
            }
            SourceItem::Record(record) => {
                let JniPolicy::DataClass { rust } = request.policy else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as a data type under a policy that is not one",
                        record.name
                    )));
                };
                let package = self.classes.package().to_string();
                let Some(class) = self.classes.kotlin(rust) else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{rust}` is exposed as a data class and registered as no Kotlin class"
                    )));
                };
                let mut properties = Vec::new();
                for field in &record.fields {
                    let Some(name) = field.name.as_ref() else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.positional_field",
                            "a positional field has no Kotlin property name".to_string(),
                        )));
                    };
                    let Some((_, kotlin, _)) = scalar_of(&field.ty).and_then(jvm_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.carrier",
                            format!("property `{name}` has no Kotlin spelling yet"),
                        )));
                    };
                    properties.push((name.to_string(), kotlin.to_string()));
                }
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    element: request.element.id.clone(),
                    requires: Vec::new(),
                    rust: Vec::new(),
                    payload: Some(JniPayload::Class {
                        package,
                        class: class.to_string(),
                        properties,
                    }),
                }))
            }
        }
    }

    fn render_operation(
        &self,
        payload: &JniPayload,
        operands: &[syn::Ident],
    ) -> proc_macro2::TokenStream {
        match payload {
            JniPayload::Getter { name, descriptor } => {
                let (env, object) = (&operands[0], &operands[1]);
                // One expression: it evaluates to a `Result` and stops there.
                // The `match` on it, and what a failure does, are the wrapper's.
                quote! {
                    #env.call_method(&#object, #name, #descriptor, &[])
                        .and_then(|value| value.j())
                }
            }
            JniPayload::ReportError => {
                let (env, error) = (&operands[0], &operands[1]);
                quote!(report_jni_error(&mut #env, #error))
            }
            JniPayload::Class { .. } | JniPayload::Method { .. } => {
                unreachable!("a Kotlin declaration is not an operation")
            }
        }
    }
}

/// The imports the JNI carriers are spelled with.
fn jni_imports() -> proc_macro2::TokenStream {
    quote! {
        use jni::{objects::{JClass, JObject}, sys::jlong, JNIEnv};
    }
}

/// The helper the failure route calls: a generated artifact, not a fragment.
fn report_jni_error() -> proc_macro2::TokenStream {
    quote! {
        pub fn report_jni_error(
            env: &mut JNIEnv<'_>,
            error: jni::errors::Error,
        ) -> jni::errors::Result<()> {
            if env.exception_check()? {
                Ok(())
            } else {
                env.throw_new("java/lang/RuntimeException", error.to_string())
            }
        }
    }
}

/// The JNI adapter's foreign writer: the Kotlin the JVM compiles against.
///
/// It reads the frozen surfaces and the metadata its own operations were built
/// from, so a renamed property moves in the getter and in the class or in
/// neither.
pub fn write_kotlin(generation: &Generation<JniPayload>) -> String {
    use std::collections::BTreeMap;

    // One file per package would be the real shape; this adapter writes one
    // file and refuses to guess when two packages are configured, rather than
    // emitting declarations under a package their native symbols do not match.
    let mut classes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut objects: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for surface in generation.surfaces() {
        match &surface.payload {
            Some(JniPayload::Class {
                package,
                class,
                properties,
            }) => {
                let properties: Vec<String> = properties
                    .iter()
                    .map(|(name, ty)| format!("val {name}: {ty}"))
                    .collect();
                classes
                    .entry(package.clone())
                    .or_default()
                    .push(format!("data class {class}({})", properties.join(", ")));
            }
            Some(JniPayload::Method {
                package,
                object,
                method,
                params,
                ret,
            }) => {
                let params: Vec<String> = params
                    .iter()
                    .map(|(name, ty)| format!("{name}: {ty}"))
                    .collect();
                objects
                    .entry(package.clone())
                    .or_default()
                    .entry(object.clone())
                    .or_default()
                    .push(format!(
                        "    @JvmStatic\n    external fun {method}({}): {ret}",
                        params.join(", ")
                    ));
            }
            _ => {}
        }
    }

    let packages: Vec<&String> = classes.keys().chain(objects.keys()).collect();
    let package = match packages.first() {
        Some(first) => (*first).clone(),
        None => return String::new(),
    };
    assert!(
        packages.iter().all(|name| **name == package),
        "this reference adapter writes one Kotlin file, so it takes one package; \
         configured: {packages:?}"
    );

    let mut out = format!("package {package}\n");
    for declaration in classes.get(&package).into_iter().flatten() {
        out.push_str(&format!("\n{declaration}\n"));
    }
    // Every method of one object goes in that object, once.
    for (object, methods) in objects.get(&package).into_iter().flatten() {
        out.push_str(&format!(
            "\nobject {object} {{\n{}\n}}\n",
            methods.join("\n\n")
        ));
    }
    out
}
