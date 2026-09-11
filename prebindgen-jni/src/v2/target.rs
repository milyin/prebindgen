//! The JNI target, as the v2 engine asks it questions.
//!
//! Everything JNI contributes to a v2 run: which relation a value crosses
//! through, what JVM carrier holds it and how its properties are read, how a
//! native method is shaped and how a failure inside it reaches the JVM, and
//! what Kotlin each declaration needs — stated as a payload the Kotlin writer
//! in [`super::kotlin`] renders. There is no type walk and no control flow
//! here; the registry owns both.
//!
//! Names arrive in the policies. Which Kotlin class a type is declared as and
//! where a function lands are the frontend's settings applied, settled when
//! the requests are built in [`super`], so the target never spells a name of
//! its own.

use prebindgen_registry::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_registry_v2::{
    AbiSpec, Access, Artifact, BoundarySpec, ChildValue, Direction, ElementId, ElementKind,
    FailureCategory, FailureRoute, Layout, NativeParam, OperandSpec, Operation, OperationType,
    OutputPlacement, ParamRole, PlanningError, PrimitiveFailure, PrimitiveSpec, Protocol, Relation,
    RelationId, ReprSpec, ResolvedShape, ResolvedValues, SelectionQuery, SiteDescriptor,
    SourceItem, SurfaceRequest, SurfaceSpec, Target, TargetAttempt, TargetSupport, Terminal,
    Unsupported, WireType,
};
use quote::{format_ident, quote};

/// What the JNI frontend recorded for one value or one exported function.
#[derive(Clone, Debug)]
pub enum JniPolicy {
    /// A scalar crossing as its JNI carrier. The default for every value
    /// nothing more specific covers.
    Scalar,
    /// A JVM object whose properties are read, declared as this Kotlin class
    /// (fully qualified).
    DataClass { class: String },
    /// An exported function: a native method on the harness object, and the
    /// Kotlin function that calls it.
    Function {
        /// The package the Kotlin function is declared in.
        package: String,
        /// The Kotlin name of the function a caller uses.
        method: String,
        /// The name of its native method on the harness object.
        native: String,
        /// The `Java_…` symbol the JVM looks that native method up by.
        symbol: String,
    },
    /// A declarator v1 lowers and v2 does not yet — a handle class, an enum
    /// class, a sealed class, a constant, a class member. Carries the
    /// declarator's name so the refusal says which capability is missing.
    Unimplemented { declarator: &'static str },
}

/// What the JNI target renders itself: one operation, or one Kotlin
/// declaration.
#[derive(Clone, Debug)]
pub enum JniPayload {
    /// A property read through the JVM: the getter's name and its descriptor.
    Getter { name: String, descriptor: String },
    /// The error-reporting helper: throw what the JVM did not already throw.
    ReportError,
    /// A Kotlin data class and its properties, `(name, Kotlin type)`.
    Class {
        package: String,
        class: String,
        properties: Vec<(String, String)>,
    },
    /// A native method on the harness object, and the Kotlin function over it.
    Method {
        package: String,
        /// The Kotlin function's name.
        method: String,
        /// The native method's name on the harness.
        native: String,
        params: Vec<(String, String)>,
        ret: String,
    },
}

/// The JNI target.
///
/// It carries the Kotlin class declared for each Rust type, because a Kotlin
/// signature has to name the class the frontend chose — not the Rust type it
/// was built from — and a value's own policy is not in view where a signature
/// is written. Renaming a class in the declarations therefore moves it in the
/// declaration and in every signature mentioning it.
pub struct JniTarget {
    /// Rust type key → fully qualified Kotlin class.
    classes: std::collections::BTreeMap<String, String>,
}

impl JniTarget {
    pub(crate) fn new(classes: std::collections::BTreeMap<String, String>) -> Self {
        JniTarget { classes }
    }

    /// The Kotlin spelling of a value: the class its type was declared as, or
    /// the scalar's Kotlin type.
    fn kotlin_type(&self, ty: &TypeRef) -> Option<String> {
        match named(ty) {
            Some(name) => self.classes.get(&name).cloned(),
            None => scalar_of(ty)
                .and_then(jvm_scalar)
                .map(|(_, kotlin, _)| kotlin.to_string()),
        }
    }
}

/// The Rust name of the error-reporting helper the wrappers call.
pub(crate) const REPORT_ERROR: &str = "report_jni_error";

/// The JVM carrier, Kotlin type and descriptor of a scalar, when this adapter
/// has one.
///
/// One scalar today, as the specification's element paths need: every getter
/// this adapter describes is read with `JValueOwned::j`, which extracts a
/// long. The rest of `ScalarKind` arrives with its own increment.
fn jvm_scalar(kind: ScalarKind) -> Option<(syn::Type, &'static str, &'static str)> {
    match kind {
        ScalarKind::I64 => Some((syn::parse_quote!(jni::sys::jlong), "Long", "J")),
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

/// A source identifier as Kotlin spells it: the raw-identifier prefix Rust
/// needs for `r#type` dropped, and a Kotlin keyword back-ticked. What a Kotlin
/// declaration and the expression calling it both use, so the two agree.
fn kotlin_ident(ident: &syn::Ident) -> String {
    kotlin_codegen::escape_kotlin_ident(&plain(ident))
}

/// A source identifier's spelling without Rust's raw-identifier prefix.
fn plain(ident: &syn::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}

/// The JVM getter a Kotlin property compiles to.
///
/// `val secs` becomes `getSecs()`. A property whose name starts with `is` and
/// continues with anything but a lowercase letter — `isReady`, `is_ready`,
/// `is2` — keeps its name as the getter, for every type and not only
/// `Boolean`: that is Kotlin's JVM interop rule, and `island` is not an
/// instance of it.
fn getter(name: &str) -> String {
    let rest = &name[name.len().min(2)..];
    let is_prefixed = name.starts_with("is") && !rest.starts_with(|c: char| c.is_lowercase());
    if is_prefixed {
        return name.to_string();
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => format!("get{}{}", first.to_uppercase(), chars.as_str()),
        None => "get".to_string(),
    }
}

/// A name for a wrapper parameter the target adds — the environment, the
/// receiver — that no source parameter already uses.
fn free_name(preferred: &str, function: &prebindgen_registry::flat::Function) -> syn::Ident {
    let taken = |name: &str| {
        function
            .params
            .iter()
            .any(|param| plain(&param.name) == name)
    };
    let mut candidate = preferred.to_string();
    while taken(&candidate) {
        candidate.push('_');
    }
    format_ident!("{candidate}")
}

impl Target for JniTarget {
    type Policy = JniPolicy;
    type Payload = JniPayload;

    fn select(&self, query: &SelectionQuery<'_, JniPolicy>) -> TargetSupport<RelationId> {
        let want_record = match query.policy {
            JniPolicy::DataClass { .. } => true,
            JniPolicy::Scalar | JniPolicy::Function { .. } => false,
            JniPolicy::Unimplemented { declarator } => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.jni.{declarator}"),
                    format!(
                        "`{}` is declared with `{declarator}`, which the v2 JNI target does \
                         not lower yet",
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
                let Some((carrier, _, _)) = scalar_of(&shape.crossing.ty).and_then(jvm_scalar)
                else {
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
                if item.fields.is_empty() {
                    // A Kotlin data class needs at least one property, so
                    // there is no declaration to read this object's properties
                    // from.
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.empty_class",
                        format!(
                            "`{}` has no fields, and a Kotlin data class needs at least one \
                             property",
                            record.record
                        ),
                    )));
                }
                let object = WireType::abi(syn::parse_quote!(jni::objects::JObject<'_>));
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
                                    jni::JNIEnv<'_>
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
                            name: getter(&plain(name)),
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
        let symbol = match policy {
            JniPolicy::Function { symbol, .. } => symbol,
            // A class member reaches here when every value it takes has a
            // carrier; the member itself is still a declarator v2 does not
            // lower, and says so where the report can group it.
            JniPolicy::Unimplemented { declarator } => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.jni.{declarator}"),
                    format!(
                        "`{}` is declared as a `{declarator}`, which the v2 JNI target does \
                         not lower yet",
                        site.element.rust_origin
                    ),
                )));
            }
            JniPolicy::Scalar | JniPolicy::DataClass { .. } => {
                return Err(PlanningError::InvalidInput(format!(
                    "`{}` is exported under a policy that is not a function policy",
                    site.element.rust_origin
                )));
            }
        };
        // The two parameters the JVM adds are named around the source's: a
        // source parameter called `env` keeps its name, and the environment
        // steps aside.
        let mut params = vec![
            NativeParam {
                name: free_name("env", site.function),
                ty: WireType::abi(syn::parse_quote!(jni::JNIEnv<'_>)),
                role: ParamRole::Context("jni.env".to_string()),
                mutable: true,
            },
            // The native method is an instance method of the harness `object`,
            // so what the JVM passes here is the singleton, not a class.
            NativeParam {
                name: free_name("_this", site.function),
                ty: WireType::abi(syn::parse_quote!(jni::objects::JObject<'_>)),
                role: ParamRole::Unused,
                mutable: false,
            },
        ];
        // A native parameter keeps the source parameter's name, as v1's do.
        params.extend(
            values
                .inputs
                .iter()
                .zip(&site.function.params)
                .enumerate()
                .map(|(index, (value, param))| NativeParam {
                    name: param.name.clone(),
                    ty: value.repr.layout.wire().clone(),
                    role: ParamRole::Input(index),
                    mutable: false,
                }),
        );
        let jni_error =
            || OperationType::Carrier(WireType::internal(syn::parse_quote!(jni::errors::Error)));
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: AbiSpec {
                abi: "system".to_string(),
                symbol: symbol.clone(),
                params,
                ret: values.output.map(|value| value.repr.layout.wire().clone()),
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            failures: vec![FailureRoute {
                category: FailureCategory::Runtime,
                report: Some(PrimitiveSpec {
                    operands: vec![
                        OperandSpec::context(
                            "jni.env",
                            OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                jni::JNIEnv<'_>
                            ))),
                            Access::Exclusive,
                        ),
                        OperandSpec::error(jni_error()),
                    ],
                    result: None,
                    failure: PrimitiveFailure::fallible(jni_error(), FailureCategory::Runtime),
                    dependencies: vec![Artifact::new(REPORT_ERROR, report_jni_error())],
                    implementation: Operation::Target(JniPayload::ReportError),
                }),
                on_report_failure: Terminal::Abort,
                // Zero is not a result: it is what a native method must return
                // while an exception is pending, and Kotlin observes the
                // exception. A wrapper that returns nothing terminates with
                // nothing.
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
                let JniPolicy::Function {
                    package,
                    method,
                    native,
                    ..
                } = request.policy
                else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exported under a policy that is not a function policy",
                        function.name
                    )));
                };
                let mut params = Vec::new();
                for (param, value) in function.params.iter().zip(&values.inputs) {
                    let Some(kotlin) = self.kotlin_type(&value.crossing.ty) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.kotlin_type",
                            format!("`{}` has no Kotlin spelling yet", param.name),
                        )));
                    };
                    params.push((kotlin_ident(&param.name), kotlin));
                }
                let ret = match values.output {
                    None => "Unit".to_string(),
                    Some(value) => match self.kotlin_type(&value.crossing.ty) {
                        Some(kotlin) => kotlin,
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
                    // A method taking a declared class is unusable unless the
                    // class it names is emitted too.
                    requires: values
                        .inputs
                        .iter()
                        .filter_map(|value| named(&value.crossing.ty))
                        .map(|name| ElementId::new(ElementKind::Type, name))
                        .collect(),
                    // What crosses is a JVM object: the Rust side holds a
                    // reference to it and declares no type of its own.
                    rust: Vec::new(),
                    payload: Some(JniPayload::Method {
                        package: package.clone(),
                        method: method.clone(),
                        native: native.clone(),
                        params,
                        ret,
                    }),
                }))
            }
            SourceItem::Record(record) => {
                let JniPolicy::DataClass { class } = request.policy else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as a data class under a policy that is not one",
                        record.name
                    )));
                };
                if record.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.empty_class",
                        format!(
                            "`{}` has no fields, and a Kotlin data class needs at least one \
                             property",
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
                    let Some((_, kotlin, _)) = scalar_of(&field.ty).and_then(jvm_scalar) else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.carrier",
                            format!("property `{name}` has no Kotlin spelling yet"),
                        )));
                    };
                    properties.push((kotlin_ident(name), kotlin.to_string()));
                }
                // The class was declared under a fully qualified name; the
                // writer files it by package.
                let (package, class) = match class.rsplit_once('.') {
                    Some((package, class)) => (package.to_string(), class.to_string()),
                    None => (String::new(), class.clone()),
                };
                Ok(TargetAttempt::Ready(SurfaceSpec {
                    element: request.element.id.clone(),
                    requires: Vec::new(),
                    rust: Vec::new(),
                    payload: Some(JniPayload::Class {
                        package,
                        class,
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
                let report = format_ident!("{REPORT_ERROR}");
                quote!(#report(&mut #env, #error))
            }
            JniPayload::Class { .. } | JniPayload::Method { .. } => {
                unreachable!("a Kotlin declaration is not an operation")
            }
        }
    }
}

/// The error-reporting helper every wrapper's failure route calls.
///
/// A `jni` error that came out of a JVM call already left an exception
/// pending, and throwing a second one over it is itself an error; anything
/// else becomes a `RuntimeException`.
fn report_jni_error() -> proc_macro2::TokenStream {
    let report = format_ident!("{REPORT_ERROR}");
    quote! {
        pub fn #report(
            env: &mut jni::JNIEnv<'_>,
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
