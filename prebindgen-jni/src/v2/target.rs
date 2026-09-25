//! The JNI target, as the v2 engine calls it: the operations only the JVM has.
//!
//! Everything a JNI binding decides — which wire type a value crosses in, which
//! operations read it, what native method and `Java_…` symbol a wrapper gets,
//! how a failure reaches the JVM — is stated as data by [`super`] when the
//! binding is built, and the registry plans from that alone. What is left here
//! is writing the operations only the JVM has: a property getter, the two ways
//! a failure is reported, and the three a callback needs — keeping a Kotlin
//! callable alive, calling it from whatever thread Rust calls from, and saying
//! when that failed. The Kotlin declarations are the frontend's
//! own writer's, in [`super::kotlin`], over the finished generation.

use prebindgen_registry_v2::{
    Artifact, Failure, FailureCategory, Fed, OperationFeed, Target, TargetOp, WireKind, WireType,
    WireTypeFeed, Written,
};
use quote::{format_ident, quote};

/// The kinds of JVM wire type, which is what JNI's capabilities are stated
/// in: what a data class can have as properties, what a callback can take.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JniWireKind {
    /// A `jlong` holding a number.
    Long,
    /// A `jint`: a fieldless enum's value.
    Int,
    /// A `jlong` holding a Rust address, inside a Kotlin handle class.
    Handle,
    /// A reference to a Kotlin data class instance, read through getters.
    Object,
    /// A reference to a Kotlin `fun interface`, called through `run`.
    Callable,
}

impl WireKind for JniWireKind {
    const ALL: &'static [Self] = &[
        JniWireKind::Long,
        JniWireKind::Int,
        JniWireKind::Handle,
        JniWireKind::Object,
        JniWireKind::Callable,
    ];

    fn name(self) -> &'static str {
        match self {
            JniWireKind::Long => "long",
            JniWireKind::Int => "int",
            JniWireKind::Handle => "handle",
            JniWireKind::Object => "object",
            JniWireKind::Callable => "callable",
        }
    }

    /// A data class's properties are what a getter returning a `long` reads.
    /// A callback's arguments are what `run` takes as a JVM primitive: a
    /// number, an enum's number, an address.
    fn parts(self) -> &'static [Self] {
        match self {
            JniWireKind::Object => &[JniWireKind::Long],
            JniWireKind::Callable => &[JniWireKind::Long, JniWireKind::Int, JniWireKind::Handle],
            _ => &[],
        }
    }
}

/// A JVM wire type: its kind, and the Kotlin names the JVM side reads it as.
/// Its descriptor and Kotlin type follow from those.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JniWireType {
    /// A `jlong` Kotlin reads as a `Long`.
    Long,
    /// A `jint` Kotlin reads as a value of this `enum class`.
    Int { kotlin_enum: String },
    /// A `jlong` Kotlin wraps in this handle class.
    Handle { kotlin_class: String },
    /// A `JObject` of this data class.
    Object { kotlin_class: String },
    /// A `JObject` implementing this `fun interface` — or, when an argument
    /// is a handle or an enum, `raw`: the one over the arguments' wire forms,
    /// which the `external` method takes.
    Callable {
        interface: String,
        raw: Option<String>,
    },
}

impl WireType for JniWireType {
    type Kind = JniWireKind;

    fn kind(&self) -> JniWireKind {
        match self {
            JniWireType::Long => JniWireKind::Long,
            JniWireType::Int { .. } => JniWireKind::Int,
            JniWireType::Handle { .. } => JniWireKind::Handle,
            JniWireType::Object { .. } => JniWireKind::Object,
            JniWireType::Callable { .. } => JniWireKind::Callable,
        }
    }

    /// Every JVM wire type is a type the `jni` crate declares; a number and an
    /// address share one, and differ in what the JVM side does with it.
    fn rust(&self) -> syn::Type {
        match self {
            JniWireType::Long | JniWireType::Handle { .. } => syn::parse_quote!(jni::sys::jlong),
            JniWireType::Int { .. } => syn::parse_quote!(jni::sys::jint),
            JniWireType::Object { .. } | JniWireType::Callable { .. } => {
                syn::parse_quote!(jni::objects::JObject<'_>)
            }
        }
    }
}

impl JniWireType {
    /// The JVM type descriptor: `J`, `I`, `Lexample/Stamp;`.
    pub fn descriptor(&self) -> String {
        match self {
            JniWireType::Long | JniWireType::Handle { .. } => "J".to_string(),
            JniWireType::Int { .. } => "I".to_string(),
            JniWireType::Object { kotlin_class } => format!("L{};", kotlin_class.replace('.', "/")),
            JniWireType::Callable { interface, raw } => {
                format!("L{};", raw.as_ref().unwrap_or(interface).replace('.', "/"))
            }
        }
    }

    /// How Kotlin spells it, and how a value of it reaches the `external`
    /// method.
    pub fn kotlin(&self) -> KotlinType {
        match self {
            JniWireType::Long => KotlinType::Value("Long".to_string()),
            JniWireType::Int { kotlin_enum } => KotlinType::Enum(kotlin_enum.clone()),
            JniWireType::Handle { kotlin_class } => KotlinType::Handle(kotlin_class.clone()),
            JniWireType::Object { kotlin_class } => KotlinType::Value(kotlin_class.clone()),
            JniWireType::Callable { interface, raw } => KotlinType::Callback {
                class: interface.clone(),
                raw: raw.clone(),
            },
        }
    }
}

/// A Kotlin type as the public function spells it, and how a value of it
/// reaches the native method.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KotlinType {
    /// Spelled the same on both sides and passed as it is: a scalar, or a data
    /// class the native method reads through the JVM.
    Value(String),
    /// A handle class. The native method takes and returns the address as a
    /// `Long`; the public function unwraps one and wraps the other.
    Handle(String),
    /// An `enum class`. The native method takes and returns the value's own
    /// number as an `Int`; the public function reads `.value` off one and
    /// looks the other up with `fromInt`.
    Enum(String),
    /// A `fun interface` Rust calls back. When an argument is a handle or an
    /// enum, the native method takes `raw`: the interface over the arguments'
    /// wire forms, which the public function adapts the public one to.
    Callback { class: String, raw: Option<String> },
}

impl KotlinType {
    /// What the public function declares.
    pub fn public(&self) -> &str {
        match self {
            KotlinType::Value(kotlin)
            | KotlinType::Handle(kotlin)
            | KotlinType::Enum(kotlin)
            | KotlinType::Callback { class: kotlin, .. } => kotlin,
        }
    }

    /// What the native method declares.
    pub fn native(&self) -> &str {
        match self {
            KotlinType::Value(kotlin) => kotlin,
            KotlinType::Handle(_) => "Long",
            KotlinType::Enum(_) => "Int",
            KotlinType::Callback { class, raw } => raw.as_deref().unwrap_or(class),
        }
    }
}

/// The operations only the JVM has.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JniOp {
    /// Read the part's property off the object through its getter.
    Getter,
    /// Report a failed JVM call: throw what the JVM did not already throw.
    ReportError,
    /// Throw a binding failure's message as an `IllegalStateException`.
    ThrowMessage,
    /// Keep what calling a Kotlin callable needs from any thread: the JVM, a
    /// global reference to the callable, and its `run` method.
    CaptureCallback,
    /// Call `run` on the captured callable, attaching the calling thread to
    /// the JVM first. An exception the callable throws is described and
    /// cleared, since nothing on the Rust side can catch it.
    CallCallback,
    /// Say that a call of a callback failed. There is no caller to throw to,
    /// so it is written to standard error.
    ReportCallbackError,
}

/// Every JVM operation but the last runs through the environment of the
/// wrapper that applies it, and every one that calls into the JVM can fail
/// with the `jni` crate's error. A callback's call is made on whatever thread
/// Rust calls from, so it attaches its own and asks for none; writing out a
/// call's failure needs nothing.
impl TargetOp for JniOp {
    fn contexts(&self) -> &'static [&'static str] {
        match self {
            JniOp::Getter | JniOp::ReportError | JniOp::ThrowMessage | JniOp::CaptureCallback => {
                &["jni.env"]
            }
            JniOp::CallCallback | JniOp::ReportCallbackError => &[],
        }
    }

    fn failure(&self) -> Option<Failure> {
        match self {
            JniOp::ReportCallbackError => None,
            _ => Some(Failure {
                category: FailureCategory::Runtime,
                error: Box::new(syn::parse_quote!(jni::errors::Error)),
            }),
        }
    }
}

/// What the Kotlin writer needs about one output that no plan says.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JniOutput {
    /// A Kotlin `data class`, one property per field.
    DataClass { package: String, class: String },
    /// A Kotlin `enum class` and its values, `(SCREAMING_SNAKE name, number)`.
    ///
    /// The number is an `i32` because a Kotlin `Int` is: an enum numbering a
    /// value outside that range is refused rather than mirrored.
    EnumClass {
        package: String,
        class: String,
        values: Vec<(String, i32)>,
    },
    /// A Kotlin class wrapping a Rust address, and the native method on the
    /// harness that frees one.
    PtrClass {
        package: String,
        class: String,
        native: String,
    },
    /// A Kotlin `fun interface` Rust calls back, and — when `raw` is set —
    /// the interface over the arguments' wire forms the native method takes.
    Callback {
        package: String,
        class: String,
        raw: Option<String>,
    },
    /// A native method on the harness object, and the Kotlin function over it.
    Function {
        package: String,
        /// The Kotlin function's name.
        method: String,
        /// The native method's name on the harness.
        native: String,
    },
}

/// The JNI target: its writers.
#[derive(Default)]
pub struct JniTarget;

/// The Rust name of the error-reporting helper the wrappers call.
pub(crate) const REPORT_ERROR: &str = "report_jni_error";

impl Target for JniTarget {
    const NAME: &'static str = "jni";

    type WireType = JniWireType;
    type Op = JniOp;
    type OutputMeta = JniOutput;

    fn write_operation(&self, op: &JniOp, feed: &OperationFeed<'_, Self>) -> Written {
        match op {
            // One expression: it evaluates to a `Result` and stops there. The
            // `match` on it, and what a failure does, are the wrapper's.
            JniOp::Getter => {
                let env = feed.context("jni.env");
                let (object, _) = feed.value.as_ref().expect("a getter reads an object");
                let part = feed.part.expect("a getter reads one property");
                let name = getter(part.name.as_deref().map(plain).unwrap_or_default());
                let Some(Fed::WireType(result)) = &feed.result else {
                    panic!("a getter produces the wire type of its property");
                };
                let descriptor = format!("(){}", result.descriptor());
                let extract = format_ident!("{}", extractor(&result.descriptor()));
                Written::new(quote! {
                    #env.call_method(&#object, #name, #descriptor, &[])
                        .and_then(|value| value.#extract())
                })
            }
            JniOp::ReportError => {
                let env = feed.context("jni.env");
                let error = feed.error.as_ref().expect("a report is handed the error");
                let report = format_ident!("{REPORT_ERROR}");
                Written::new(quote!(#report(&mut #env, #error)))
                    .with_helper(Artifact::new(REPORT_ERROR, report_jni_error()))
            }
            JniOp::ThrowMessage => {
                let env = feed.context("jni.env");
                let error = feed.error.as_ref().expect("a report is handed the error");
                Written::new(quote!(#env.throw_new("java/lang/IllegalStateException", #error)))
            }
            // The method is looked up once, here, on the callable's own class:
            // `run` with the arguments' descriptors, returning nothing.
            JniOp::CaptureCallback => {
                let env = feed.context("jni.env");
                let (callable, _) = feed
                    .value
                    .as_ref()
                    .expect("a capture is applied to the callable");
                let descriptor = format!(
                    "({})V",
                    feed.args
                        .iter()
                        .map(|(_, wire_type)| wire_type.descriptor())
                        .collect::<String>()
                );
                Written::new(quote! {
                    (|| -> jni::errors::Result<_> {
                        let class = #env.get_object_class(&#callable)?;
                        let method = #env.get_method_id(&class, "run", #descriptor)?;
                        Ok((#env.get_java_vm()?, #env.new_global_ref(&#callable)?, method))
                    })()
                })
            }
            // Borrowed whole, so the closure around it moves in the captured
            // triple rather than its fields one by one.
            JniOp::CallCallback => {
                let (captured, _) = feed
                    .value
                    .as_ref()
                    .expect("a call is applied to the capture");
                let args = feed.args.iter().map(|(name, wire_type)| {
                    let field = format_ident!("{}", extractor(&wire_type.descriptor()));
                    quote!(jni::sys::jvalue { #field: #name })
                });
                Written::new(quote! {
                    (|| -> jni::errors::Result<()> {
                        let (vm, callable, method) = &#captured;
                        let mut env = vm.attach_current_thread_as_daemon()?;
                        let called = unsafe {
                            env.call_method_unchecked(
                                callable,
                                *method,
                                jni::signature::ReturnType::Primitive(
                                    jni::signature::Primitive::Void,
                                ),
                                &[#(#args),*],
                            )
                        };
                        if called.is_err() {
                            let _ = env.exception_describe();
                            let _ = env.exception_clear();
                        }
                        called.map(|_| ())
                    })()
                })
            }
            JniOp::ReportCallbackError => {
                let error = feed.error.as_ref().expect("a report is handed the error");
                Written::new(quote!(::std::eprintln!("a callback failed: {}", #error)))
            }
        }
    }

    /// A JVM wire type is a type the `jni` crate already declares; the classes
    /// they hold are Kotlin's, written by the frontend's own writer.
    fn write_wire_type(&self, _: &WireTypeFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
        Vec::new()
    }
}

/// Which `JValueOwned` accessor reads a value of this descriptor.
fn extractor(descriptor: &str) -> &'static str {
    match descriptor.chars().next() {
        Some('J') => "j",
        Some('I') => "i",
        Some('Z') => "z",
        Some('B') => "b",
        Some('S') => "s",
        Some('C') => "c",
        Some('F') => "f",
        Some('D') => "d",
        _ => "l",
    }
}

/// A source identifier's spelling without Rust's raw-identifier prefix.
pub(crate) fn plain(name: &str) -> &str {
    name.trim_start_matches("r#")
}

/// A source identifier as Kotlin spells it: the raw-identifier prefix Rust
/// needs for `r#type` dropped, and a Kotlin keyword back-ticked. What a Kotlin
/// declaration and the expression calling it both use, so the two agree.
pub(crate) fn kotlin_ident(name: &str) -> String {
    kotlin_codegen::escape_kotlin_ident(plain(name))
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

/// A name for a wrapper parameter the convention adds — the environment, the
/// receiver — that no source parameter already uses.
pub(crate) fn free_name(
    preferred: &str,
    function: &prebindgen_registry::flat::Function,
) -> syn::Ident {
    let taken = |name: &str| {
        function
            .params
            .iter()
            .any(|param| plain(&param.name.to_string()) == name)
    };
    let mut candidate = preferred.to_string();
    while taken(&candidate) {
        candidate.push('_');
    }
    format_ident!("{candidate}")
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
