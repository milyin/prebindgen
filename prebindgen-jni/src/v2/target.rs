//! The JNI target, as the v2 engine asks it questions.
//!
//! Everything JNI contributes to a v2 run: which relation a value crosses
//! through, what JVM carrier holds it and how its properties are read, how a
//! native method is shaped and how a failure inside it reaches the JVM, and
//! what Kotlin each declaration needs — stated as a payload the Kotlin writer
//! in [`super::kotlin`] renders. There is no type walk and no control flow
//! here; the registry owns both.
//!
//! It also *holds* what the binding declared. The registry stores no
//! configuration and knows no precedence: which Kotlin class a type is
//! declared as, where a function lands, which native method and `Java_…`
//! symbol it gets, and which setting on it v2 does not honour yet are all
//! recorded here by [`super`] and looked up here when the registry asks
//! (#766). The names themselves are the frontend's settings applied, settled
//! when the binding is built, so the target never spells one of its own.

use std::collections::BTreeMap;

use prebindgen_registry::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_registry_v2::{
    AbiSpec, Access, Artifact, BoundarySpec, ChildValue, Declaration, Direction, FailureCategory,
    FailureRoute, Layout, OperandSpec, Operation, OperationType, OutputPlacement, ParamRole,
    PlanningError, PrimitiveFailure, PrimitiveSpec, Protocol, Relation, ReprSpec, Requirement,
    ResolvedShape, ResolvedValues, Selection, SelectionQuery, SiteDescriptor, SourceItem,
    SurfaceRequest, SurfaceSpec, Target, TargetAttempt, TargetSupport, Terminal, Unsupported,
    WireType, WrapperParam,
};
use quote::{format_ident, quote};

/// What the JNI frontend recorded for one value or one exported function.
///
/// Also this target's [`Target::ConversionKey`]: it is plain data, so two
/// values the binding declared the same way convert the same way — which is
/// what the key has to mean for the registry to reuse one conversion for both.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JniChoice {
    /// A scalar crossing as its JNI carrier. The default for every value
    /// nothing more specific covers.
    Scalar,
    /// A JVM object whose properties are read, declared as this Kotlin class
    /// (fully qualified).
    DataClass { class: String },
    /// An opaque handle: the JVM holds the address as a `Long` inside this
    /// Kotlin class (fully qualified), and frees it through the `native`
    /// method on the harness, which the JVM looks up by `symbol`.
    PtrClass {
        class: String,
        native: String,
        symbol: String,
    },
    /// A source function to expose: as a native method on the harness object,
    /// and the Kotlin function that calls it.
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
    /// class, a sealed class, a constant, a class member — or a declarator v2
    /// lowers under a setting it does not honour yet. Carries the declarator's
    /// name for the report, the capability the refusal names, and the Kotlin
    /// placement the declaration would have had.
    Unimplemented {
        declarator: &'static str,
        /// The missing capability's code stem, `unsupported.jni.<capability>`.
        /// The declarator itself, unless a setting on it is what v2 lacks.
        capability: &'static str,
        placement: String,
    },
}

impl JniChoice {
    /// The native method on the harness this choice names, if it names one.
    /// Two choices naming the same one are two definitions of one symbol.
    pub(crate) fn native(&self) -> Option<&str> {
        match self {
            JniChoice::PtrClass { native, .. } | JniChoice::Function { native, .. } => Some(native),
            _ => None,
        }
    }

    /// A declarator v2 does not lower: the capability missing is the
    /// declarator itself.
    pub(crate) fn unimplemented(declarator: &'static str, placement: String) -> Self {
        JniChoice::Unimplemented {
            declarator,
            capability: declarator,
            placement,
        }
    }
}

/// What the JNI target renders itself: one operation, or one Kotlin
/// declaration.
#[derive(Clone, Debug)]
pub enum JniPayload {
    /// A property read through the JVM: the getter's name and its descriptor.
    Getter { name: String, descriptor: String },
    /// The error-reporting helper: throw what the JVM did not already throw.
    ReportError,
    /// Throw a binding failure's message as an `IllegalStateException`.
    ThrowMessage,
    /// A Kotlin class wrapping a Rust address, and the native method on the
    /// harness that frees one — a [`JniPayload::Method`] taking the handle.
    Handle {
        package: String,
        class: String,
        release: Box<JniPayload>,
    },
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
        params: Vec<(String, KotlinType)>,
        ret: KotlinType,
        /// The conditions the source function was written under, as written.
        ///
        /// Kotlin has no conditional compilation, so the generated function
        /// exists whatever they say and a call to an absent symbol fails at
        /// run time. Naming them in the documentation is what the writer can
        /// do about that; empty in the ordinary case.
        conditions: Vec<String>,
    },
}

/// A Kotlin type as the public function spells it, and how a value of it
/// reaches the native method.
#[derive(Clone, Debug)]
pub enum KotlinType {
    /// Spelled the same on both sides and passed as it is: a scalar, or a data
    /// class the native method reads through the JVM.
    Value(String),
    /// A handle class. The native method takes and returns the address as a
    /// `Long`; the public function unwraps one and wraps the other.
    Handle(String),
}

impl KotlinType {
    /// What the public function declares.
    pub fn public(&self) -> &str {
        match self {
            KotlinType::Value(kotlin) | KotlinType::Handle(kotlin) => kotlin,
        }
    }

    /// What the native method declares.
    pub fn native(&self) -> &str {
        match self {
            KotlinType::Value(kotlin) => kotlin,
            KotlinType::Handle(_) => "Long",
        }
    }
}

/// The JNI target: what the binding declared, and the answers the registry
/// gets out of it.
///
/// [`Self::classes`] is separate from [`Self::types`] because a Kotlin
/// signature has to name the class the frontend chose for a type — not the
/// Rust type it was built from — wherever that type appears, including in a
/// function's signature where the *function's* declaration is what is in view.
/// Renaming a class in the declarations therefore moves it in the declaration
/// and in every signature mentioning it.
pub struct JniTarget {
    /// Rust type key → fully qualified Kotlin class, and whether it is a
    /// handle class rather than a data class. Spelling, for every signature
    /// that names the type.
    classes: BTreeMap<String, (String, bool)>,
    /// How every value of this source type crosses, wherever it appears, by
    /// the type's canonical key.
    types: BTreeMap<String, JniChoice>,
}

impl JniTarget {
    pub(crate) fn new(classes: BTreeMap<String, (String, bool)>) -> Self {
        JniTarget {
            classes,
            types: BTreeMap::new(),
        }
    }

    /// Record how a declared type's values cross.
    ///
    /// What each output *is* travels with it: the engine holds the binding's
    /// declarations as the pair of what was declared and what this target
    /// recorded for it, and hands the choice back with every question about
    /// that output. This table answers the other question — how a value of
    /// the type crosses wherever else it turns up.
    pub(crate) fn declare(&mut self, declaration: &Declaration, choice: &JniChoice) {
        if let Declaration::Type(key) = declaration {
            self.types.insert(key.as_str().to_string(), choice.clone());
        }
    }

    /// How a value of this type crosses: what was declared for its type, else
    /// a scalar.
    ///
    /// The position goes unread. The per-site declarators JNI has —
    /// `expand_param`, `expand_return`, `split_on_param` — are exactly the
    /// settings v2 does not lower yet, and a function carrying one is refused
    /// whole when the binding is built rather than having the setting dropped
    /// here. When they arrive, this is where they take effect: a different
    /// key for that one value, and the registry plans it as a second
    /// conversion.
    fn conversion(&self, ty: &TypeRef) -> JniChoice {
        // By the type's key, which is what the declaration was recorded by:
        // `String` is a kind of its own to the model, not a named type, and
        // `ptr_class!(String)` has to find it all the same.
        self.types
            .get(ty.key().as_str())
            .cloned()
            .unwrap_or(JniChoice::Scalar)
    }

    /// The Kotlin spelling of a value: the class its type was declared as, or
    /// the scalar's Kotlin type.
    fn kotlin_type(&self, ty: &TypeRef) -> Option<KotlinType> {
        match named(ty) {
            Some(name) => {
                let (class, handle) = self.classes.get(&name).cloned()?;
                Some(match handle {
                    true => KotlinType::Handle(class),
                    false => KotlinType::Value(class),
                })
            }
            None => scalar_of(ty)
                .and_then(jvm_scalar)
                .map(|(_, kotlin, _)| KotlinType::Value(kotlin.to_string())),
        }
    }
}

/// The Rust name of the error-reporting helper the wrappers call.
pub(crate) const REPORT_ERROR: &str = "report_jni_error";

/// The JVM carrier, Kotlin type and descriptor of a scalar, when this adapter
/// has one.
///
/// One scalar today, as the specification's declaration paths need: every getter
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
/// continues with anything but an ASCII lowercase letter — `isReady`,
/// `is_ready`, `is2`, `isé` — keeps its name as the getter, for every type and
/// not only `Boolean`: that is Kotlin's JVM interop rule. `island` is not an
/// instance of it, and neither is a property called `is` — `kotlinc` gives
/// both a `get` prefix.
fn getter(name: &str) -> String {
    let keeps_its_name = name.strip_prefix("is").is_some_and(|rest| {
        !rest.is_empty() && !rest.starts_with(|c: char| c.is_ascii_lowercase())
    });
    if keeps_its_name {
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
    const NAME: &'static str = "jni";

    type ConversionKey = JniChoice;
    type Payload = JniPayload;

    fn select(&self, query: &SelectionQuery<'_, JniChoice>) -> TargetSupport<Selection<JniChoice>> {
        // A declared type's own crossing is planned as that declaration says,
        // not as the per-type default for values of it: the two agree for a
        // type declared once, and differ by design for one declared twice.
        let conversion = match query.position.is_declared_type() {
            true => query.declared.clone(),
            false => self.conversion(&query.crossing.ty),
        };
        // A declarator v2 has no lowering for is refused here, before anything
        // under it is planned — never quietly crossed as the scalar default.
        let want_struct = match &conversion {
            JniChoice::DataClass { .. } => true,
            JniChoice::Scalar | JniChoice::PtrClass { .. } | JniChoice::Function { .. } => false,
            JniChoice::Unimplemented {
                declarator,
                capability,
                ..
            } => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.jni.{capability}"),
                    format!(
                        "`{}` is declared with `{declarator}`, which the v2 JNI target does \
                         not lower yet",
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
        // A data class is read through properties, and the model offers no
        // fields for this type: it is an extern, captured or the binding's
        // own, and there is nothing to see into.
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.jni.not_a_struct",
            format!(
                "`{}` is declared as a data class, and the model has no fields for it",
                query.crossing.ty.key()
            ),
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        conversion: &JniChoice,
    ) -> TargetSupport<ReprSpec<JniPayload>> {
        match (shape.relation, conversion) {
            // The address of a boxed source value as a `jlong`: JNI's wire is
            // 64 bits whatever the platform's pointer is. Both directions and
            // the release are the registry's standard operations; the adapter
            // states only the carrier.
            (Relation::Atomic, JniChoice::PtrClass { .. }) => {
                let carrier = WireType::abi(syn::parse_quote!(jni::sys::jlong));
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
            (Relation::Atomic, _) => {
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
                    release: None,
                }))
            }
            (Relation::Struct(strukt), _) => {
                if shape.crossing.direction != Direction::IntoRust {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.object_output",
                        format!(
                            "`{}` leaving Rust as a JVM object is not implemented",
                            strukt.name
                        ),
                    )));
                }
                let Some(item) = shape.strukt else {
                    return Err(PlanningError::InternalInvariant(
                        "a struct relation without its strukt".to_string(),
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
                            strukt.name
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
                    if !child.part.conditions.is_empty() {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.conditional_field",
                            format!(
                                "field `{name}` is written under a condition this build \
                                 cannot evaluate, and Kotlin cannot state one: a property \
                                 for it would be filled in by every caller and read by the \
                                 wrapper only sometimes"
                            ),
                        )));
                    }
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
                    release: None,
                }))
            }
        }
    }

    fn boundary(
        &self,
        site: &SiteDescriptor<'_, JniChoice>,
        values: &ResolvedValues<'_, JniPayload>,
    ) -> TargetSupport<BoundarySpec<JniPayload>> {
        // A handle's release is a site with no source function, placed where
        // its declaration said.
        let symbol = match (site.declared, site.function) {
            (JniChoice::Function { symbol, .. }, Some(_)) => symbol,
            (JniChoice::PtrClass { symbol, .. }, None) => symbol,
            // A class member reaches here when every value it takes has a
            // carrier; the member itself is still a declarator v2 does not
            // lower, and says so where the report can group it.
            (
                JniChoice::Unimplemented {
                    declarator,
                    capability,
                    ..
                },
                _,
            ) => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    format!("unsupported.jni.{capability}"),
                    format!(
                        "`{}` is declared as a `{declarator}`, which the v2 JNI target does \
                         not lower yet",
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
        // The two parameters the JVM adds are named around the source's: a
        // source parameter called `env` keeps its name, and the environment
        // steps aside. A release calls nothing on the JVM, so the environment
        // it is handed goes unused — and is named so.
        let mut params = vec![
            WrapperParam {
                name: match site.function {
                    Some(function) => free_name("env", function),
                    None => format_ident!("_env"),
                },
                ty: WireType::abi(syn::parse_quote!(jni::JNIEnv<'_>)),
                role: ParamRole::Context("jni.env".to_string()),
                mutable: site.function.is_some(),
            },
            // The native method is an instance method of the harness `object`,
            // so what the JVM passes here is the singleton, not a class.
            WrapperParam {
                name: match site.function {
                    Some(function) => free_name("_this", function),
                    None => format_ident!("_this"),
                },
                ty: WireType::abi(syn::parse_quote!(jni::objects::JObject<'_>)),
                role: ParamRole::Unused,
                mutable: false,
            },
        ];
        // A wrapper parameter keeps the source parameter's name, as v1's do. A
        // release has no source parameter to take a name from, and takes v1's.
        params.extend(
            values
                .inputs
                .iter()
                .enumerate()
                .map(|(index, value)| WrapperParam {
                    name: match site.function {
                        Some(function) => function.params[index].name.clone(),
                        None => format_ident!("ptr"),
                    },
                    ty: value.repr.layout.wire().clone(),
                    role: ParamRole::Input(index),
                    mutable: false,
                }),
        );
        let env = || {
            OperandSpec::context(
                "jni.env",
                OperationType::Carrier(WireType::internal(syn::parse_quote!(jni::JNIEnv<'_>))),
                Access::Exclusive,
            )
        };
        let jni_error =
            || OperationType::Carrier(WireType::internal(syn::parse_quote!(jni::errors::Error)));
        // Zero is not a result: it is what a native method must return while an
        // exception is pending, and Kotlin observes the exception. A wrapper
        // that returns nothing terminates with nothing.
        let terminate = || match values.output {
            Some(_) => Terminal::Return(syn::parse_quote!(0)),
            None => Terminal::Return(syn::parse_quote!(())),
        };
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: AbiSpec {
                abi: "system".to_string(),
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
            failures: vec![
                // A runtime failure: a JVM call that failed.
                FailureRoute {
                    category: FailureCategory::Runtime,
                    report: Some(PrimitiveSpec {
                        operands: vec![env(), OperandSpec::error(jni_error())],
                        result: None,
                        failure: PrimitiveFailure::fallible(jni_error(), FailureCategory::Runtime),
                        dependencies: vec![Artifact::new(REPORT_ERROR, report_jni_error())],
                        implementation: Operation::Target(JniPayload::ReportError),
                    }),
                    on_report_failure: Terminal::Abort,
                    terminate: terminate(),
                },
                // A binding failure: the caller broke the contract — a null
                // handle — and the message says how.
                FailureRoute {
                    category: FailureCategory::Binding,
                    report: Some(PrimitiveSpec {
                        operands: vec![
                            env(),
                            OperandSpec::error(OperationType::Carrier(WireType::internal(
                                syn::parse_quote!(String),
                            ))),
                        ],
                        result: None,
                        failure: PrimitiveFailure::fallible(jni_error(), FailureCategory::Runtime),
                        dependencies: Vec::new(),
                        implementation: Operation::Target(JniPayload::ThrowMessage),
                    }),
                    on_report_failure: Terminal::Abort,
                    terminate: terminate(),
                },
            ],
        }))
    }

    fn surface(
        &self,
        request: &SurfaceRequest<'_, JniChoice>,
        values: &ResolvedValues<'_, JniPayload>,
    ) -> TargetSupport<SurfaceSpec<JniPayload>> {
        let declared = request.declared;
        // A handle class is declared the same way whatever the item behind it:
        // an alias, or a struct whose fields the JVM never sees.
        if let JniChoice::PtrClass { class, native, .. } = declared {
            let (package, class) = match class.rsplit_once('.') {
                Some((package, class)) => (package.to_string(), class.to_string()),
                None => (String::new(), class.clone()),
            };
            return Ok(TargetAttempt::Ready(SurfaceSpec {
                declaration: request.declaration.clone(),
                requires: Vec::new(),
                // What crosses is a `jlong`, and the release wrapper is Rust
                // the registry renders: nothing to contribute.
                rust: Vec::new(),
                payload: Some(JniPayload::Handle {
                    package: package.clone(),
                    class: class.clone(),
                    release: Box::new(JniPayload::Method {
                        package,
                        method: String::new(),
                        native: native.clone(),
                        params: vec![("ptr".to_string(), KotlinType::Handle(class))],
                        ret: KotlinType::Value("Unit".to_string()),
                        conditions: request.item_conditions(),
                    }),
                }),
            }));
        }
        match request.item {
            SourceItem::Extern(opaque) => Err(PlanningError::InvalidInput(format!(
                "`{}` has no fields, and is declared as something that reads them",
                opaque.name
            ))),
            SourceItem::Function(function) => {
                let JniChoice::Function {
                    package,
                    method,
                    native,
                    ..
                } = declared
                else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exported, and is declared as something that is not a function",
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
                    None => KotlinType::Value("Unit".to_string()),
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
                    declaration: request.declaration.clone(),
                    // A method taking or returning a declared class is
                    // unusable unless the class it names is emitted too.
                    requires: values
                        .inputs
                        .iter()
                        .chain(values.output.iter())
                        .filter_map(|value| Requirement::of(value))
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
                        conditions: request.item_conditions(),
                    }),
                }))
            }
            SourceItem::Struct(strukt) => {
                let JniChoice::DataClass { class } = declared else {
                    return Err(PlanningError::InvalidInput(format!(
                        "`{}` is exposed as a data class, and is declared as something else",
                        strukt.name
                    )));
                };
                if strukt.fields.is_empty() {
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.jni.empty_class",
                        format!(
                            "`{}` has no fields, and a Kotlin data class needs at least one \
                             property",
                            strukt.name
                        ),
                    )));
                }
                let conditions = request.field_conditions();
                let mut properties = Vec::new();
                for (index, field) in strukt.fields.iter().enumerate() {
                    let Some(name) = field.name.as_ref() else {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.positional_field",
                            "a positional field has no Kotlin property name".to_string(),
                        )));
                    };
                    if !conditions[index].is_empty() {
                        return Ok(TargetAttempt::Unsupported(Unsupported::new(
                            "unsupported.jni.conditional_field",
                            format!(
                                "field `{name}` is written under a condition this build \
                                 cannot evaluate, and Kotlin cannot state one: the class \
                                 would promise a property the library reads only \
                                 sometimes"
                            ),
                        )));
                    }
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
                    declaration: request.declaration.clone(),
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
            JniPayload::ThrowMessage => {
                let (env, error) = (&operands[0], &operands[1]);
                quote!(#env.throw_new("java/lang/IllegalStateException", #error))
            }
            JniPayload::Class { .. } | JniPayload::Method { .. } | JniPayload::Handle { .. } => {
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
