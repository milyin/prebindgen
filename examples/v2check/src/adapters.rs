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

/// The C carrier for a scalar.
///
/// Every scalar but one crosses C as itself: the Rust spelling *is* the
/// carrier, and `cbindgen` writes the C name for it — `int64_t` for `i64`,
/// `uintptr_t` for `usize`. So this is derived from the kind's own spelling
/// rather than from a table that would have to be kept in step with one.
///
/// `bool` is the exception. A C caller writes a `_Bool` through whatever it
/// likes, and a Rust `bool` holding anything but 0 or 1 is undefined
/// behaviour — so the byte cannot be *received* as a `bool` at all. It
/// crosses as storage of the same size and alignment, which cbindgen still
/// writes as `bool`, and is converted at each end: see
/// [`CPayload::NormalizeBool`] and [`CPayload::StoreBool`]. The carrier is
/// the same in both directions, so a struct member has one C type regardless
/// of which way its aggregate travels.
fn c_scalar(kind: ScalarKind) -> syn::Type {
    if kind == ScalarKind::Bool {
        return syn::parse_quote!(::core::mem::MaybeUninit<bool>);
    }
    rust_scalar(kind)
}

/// The Rust spelling of a scalar kind.
fn rust_scalar(kind: ScalarKind) -> syn::Type {
    let ident = format_ident!("{}", kind.as_str());
    syn::parse_quote!(#ident)
}

/// The scalar kind of a type, when it is one.
fn scalar_of(ty: &prebindgen_flat::flat::TypeRef) -> Option<ScalarKind> {
    match ty.kind() {
        TypeKind::Scalar(kind) => Some(*kind),
        _ => None,
    }
}

/// A one-operand, infallible conversion the adapter renders itself.
///
/// Every scalar whose carrier is not the Rust value has exactly this shape:
/// one owned operand in, one value out, nothing that can fail. Which side is
/// the carrier and which the source is the direction's business, so both are
/// arguments.
fn convert<P>(from: OperationType, to: OperationType, implementation: P) -> PrimitiveSpec<P> {
    PrimitiveSpec {
        operands: vec![OperandSpec::value(from, Access::Owned)],
        result: Some(to),
        failure: PrimitiveFailure::Infallible,
        dependencies: Vec::new(),
        implementation: Operation::Target(implementation),
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

/// What C renders itself.
///
/// Reading an aggregate member is a standard operation the registry renders, so
/// for a long time this type had no values at all. Both of the values it has
/// now are the two ends of one exception: a `bool` crosses C as storage rather
/// than as itself, which is neither an identity nor something the registry can
/// express.
#[derive(Clone, Debug)]
pub enum CPayload {
    /// `MaybeUninit<bool>` → `bool`, by reading the byte and comparing it to
    /// zero. Any bit pattern is a legal input; every one of them produces a
    /// valid `bool`, which is why this normalizes rather than failing.
    NormalizeBool,
    /// `bool` → `MaybeUninit<bool>`. A `bool` Rust built is already 0 or 1, so
    /// this only puts it into the carrier the C side reads.
    StoreBool,
}

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
                let Some(kind) = scalar_of(&shape.crossing.ty) else {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.carrier",
                        format!("`{}` has no C carrier yet", shape.crossing.ty.key()),
                    )));
                };

                let carrier = WireType::abi(c_scalar(kind));

                // The C carrier of an `i64` *is* the `i64`, so the conversion
                // renders nothing at all. `bool` is the one scalar that
                // travels as storage rather than as itself, so each end gets
                // the operation that end needs.
                let wire = OperationType::Carrier(carrier.clone());
                let source = OperationType::Source(shape.crossing.ty.clone());
                let codec = match (kind, shape.crossing.direction) {
                    (ScalarKind::Bool, Direction::IntoRust) => {
                        convert(wire, source, CPayload::NormalizeBool)
                    }
                    (ScalarKind::Bool, _) => convert(source, wire, CPayload::StoreBool),
                    _ => PrimitiveSpec::identity(wire),
                };

                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier),
                    protocol: Protocol::terminal(codec),
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
                            "`{c_name}` has no fields, and an empty aggregate has no \
                                 portable C form"
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
                if record.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.c.empty_aggregate",
                        format!(
                            "`{c_name}` has no fields, and an empty aggregate has no \
                                 portable C form"
                        ),
                    )));
                }
                let ident = format_ident!("{c_name}");
                let mut fields = Vec::new();
                for field in &record.fields {
                    let Some(ty) = scalar_of(&field.ty).map(c_scalar) else {
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
        operands: &[syn::Ident],
    ) -> proc_macro2::TokenStream {
        match payload {
            CPayload::NormalizeBool => {
                let value = &operands[0];
                // `read` rather than `assume_init`: the byte may be neither 0
                // nor 1, and reading it as `u8` is defined for every one.
                quote! {
                    unsafe { ::core::ptr::read(#value.as_ptr() as *const u8) != 0 }
                }
            }
            CPayload::StoreBool => {
                let value = &operands[0];
                quote!(::core::mem::MaybeUninit::new(#value))
            }
        }
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
    /// `jboolean` → `bool`: a byte a Kotlin caller filled in, where only zero
    /// is false.
    BoolFromJvm,
    /// A width-preserving cast between a Rust scalar and its JVM carrier —
    /// `u64` and `jlong`, `bool` and `jboolean`.
    Cast { ty: syn::Type },
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

/// How one scalar crosses the JVM boundary.
struct JvmScalar {
    /// The `jni::sys` type it travels in.
    carrier: syn::Type,
    /// How that carrier is spelled in Kotlin.
    kotlin: &'static str,
    /// The JVM type descriptor letter — also the name of the `JValueOwned`
    /// accessor that reads it, lowercased.
    descriptor: &'static str,
    /// Whether the carrier already *is* the Rust value, so no conversion has
    /// to be rendered around it.
    identical: bool,
}

/// The JVM carrier of a scalar, when it has one.
///
/// The JVM has no unsigned integers, so an unsigned Rust type rides in the
/// signed carrier of the same width and is cast back at the Rust end — the
/// bits are preserved, and Kotlin sees the signed reading of them. `bool` has
/// no carrier of its own either: `jboolean` is a `u8`.
///
/// `usize`/`isize` are deliberately absent: their width is platform
/// dependent, so there is no stable JVM type to pick. This is the same line
/// `prebindgen-jni` draws for primitive arrays.
fn jvm_scalar(kind: ScalarKind) -> Option<JvmScalar> {
    let (carrier, kotlin, descriptor): (syn::Type, _, _) = match kind {
        ScalarKind::Bool => (syn::parse_quote!(jboolean), "Boolean", "Z"),
        ScalarKind::I8 | ScalarKind::U8 => (syn::parse_quote!(jbyte), "Byte", "B"),
        ScalarKind::I16 | ScalarKind::U16 => (syn::parse_quote!(jshort), "Short", "S"),
        ScalarKind::I32 | ScalarKind::U32 => (syn::parse_quote!(jint), "Int", "I"),
        ScalarKind::I64 | ScalarKind::U64 => (syn::parse_quote!(jlong), "Long", "J"),
        ScalarKind::F32 => (syn::parse_quote!(jfloat), "Float", "F"),
        ScalarKind::F64 => (syn::parse_quote!(jdouble), "Double", "D"),
        ScalarKind::Isize | ScalarKind::Usize => return None,
    };
    Some(JvmScalar {
        carrier,
        kotlin,
        descriptor,
        identical: matches!(
            kind,
            ScalarKind::I8
                | ScalarKind::I16
                | ScalarKind::I32
                | ScalarKind::I64
                | ScalarKind::F32
                | ScalarKind::F64
        ),
    })
}

/// The zero a native method returns while an exception is pending.
fn zero(kind: Option<ScalarKind>) -> syn::Expr {
    match kind {
        Some(ScalarKind::F32 | ScalarKind::F64) => syn::parse_quote!(0.0),
        _ => syn::parse_quote!(0),
    }
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
                let kind = scalar_of(&shape.crossing.ty);
                let Some((kind, scalar)) = kind.zip(kind.and_then(jvm_scalar)) else {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.carrier",
                        format!("`{}` has no JNI carrier yet", shape.crossing.ty.key()),
                    )));
                };
                let carrier = WireType::abi(scalar.carrier.clone());
                let wire = OperationType::Carrier(carrier.clone());
                let source = OperationType::Source(shape.crossing.ty.clone());
                let entering = shape.crossing.direction == Direction::IntoRust;
                let codec = match (kind, entering) {
                    // A source `i64` and a `jlong` are the same Rust value.
                    _ if scalar.identical => PrimitiveSpec::identity(wire),
                    // A `jboolean` is a `u8` a Kotlin caller filled in; only 0
                    // is false, and every other byte is true.
                    (ScalarKind::Bool, true) => convert(wire, source, JniPayload::BoolFromJvm),
                    // Every other conversion is a width-preserving cast:
                    // `bool` to `jboolean`, and each unsigned type to and from
                    // the signed carrier it rides in.
                    (_, true) => convert(
                        wire,
                        source,
                        JniPayload::Cast {
                            ty: rust_scalar(kind),
                        },
                    ),
                    (_, false) => convert(
                        source,
                        wire,
                        JniPayload::Cast {
                            ty: scalar.carrier.clone(),
                        },
                    ),
                };
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier),
                    protocol: Protocol::terminal(codec),
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
                if item.fields.is_empty() {
                    // A Kotlin data class needs at least one property, so
                    // there is no declaration to read this object's properties
                    // from.
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.empty_class",
                        format!(
                            "`{}` has no fields, and a Kotlin data class needs at least \
                                 one property",
                            record.record
                        ),
                    )));
                }
                let object = WireType::abi(syn::parse_quote!(JObject<'_>));
                let mut projections = Vec::new();
                for (field, child) in item.fields.iter().zip(children) {
                    let Some(name) = field.name.as_ref() else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.positional_field",
                            "a positional field has no Kotlin property to read".to_string(),
                        )));
                    };
                    let Some(scalar) = scalar_of(&field.ty).and_then(jvm_scalar) else {
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
                            descriptor: format!("(){}", scalar.descriptor),
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
                // while an exception is pending. Which zero depends on what
                // the carrier is — a `jdouble` has no integer literal — so the
                // source scalar picks the spelling. A wrapper that returns
                // nothing terminates with nothing.
                terminate: match values.output {
                    Some(value) => Terminal::Return(zero(scalar_of(&value.crossing.ty))),
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
                            Some(scalar) => scalar.kotlin.to_string(),
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
                        Some(scalar) => scalar.kotlin.to_string(),
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
                if record.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.empty_class",
                        format!(
                            "`{}` has no fields, and a Kotlin data class needs at least \
                                 one property",
                            record.name
                        ),
                    )));
                }
                let mut properties = Vec::new();
                for field in &record.fields {
                    let Some(name) = field.name.as_ref() else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.positional_field",
                            "a positional field has no Kotlin property name".to_string(),
                        )));
                    };
                    let Some(scalar) = scalar_of(&field.ty).and_then(jvm_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.carrier",
                            format!("property `{name}` has no Kotlin spelling yet"),
                        )));
                    };
                    properties.push((name.to_string(), scalar.kotlin.to_string()));
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
                // `JValueOwned`'s accessors are named after the descriptor
                // letters they read: `J` is read by `j`, `Z` by `z`.
                let accessor = format_ident!(
                    "{}",
                    descriptor
                        .chars()
                        .next_back()
                        .expect("a descriptor names a return type")
                        .to_ascii_lowercase()
                );
                // `z` is the one accessor that does not produce the carrier
                // it names: the JVM's boolean is a byte, and `JValueOwned`
                // hands back the `bool` it read out of it.
                let read = if descriptor.ends_with('Z') {
                    quote!(value.z().map(|flag| flag as jboolean))
                } else {
                    quote!(value.#accessor())
                };
                quote! {
                    #env.call_method(&#object, #name, #descriptor, &[])
                        .and_then(|value| #read)
                }
            }
            JniPayload::BoolFromJvm => {
                let value = &operands[0];
                quote!(#value != 0)
            }
            JniPayload::Cast { ty } => {
                let value = &operands[0];
                quote!(#value as #ty)
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
        #[allow(unused_imports)]
        use jni::{
            objects::{JClass, JObject},
            sys::{jboolean, jbyte, jdouble, jfloat, jint, jlong, jshort},
            JNIEnv,
        };
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
