//! The JNI target, as the v2 engine calls it: the operations only the JVM has.
//!
//! Everything a JNI binding decides — which carrier a value crosses in, which
//! operations read it, what native method and `Java_…` symbol a wrapper gets,
//! how a failure reaches the JVM — is stated as data by [`super`] when the
//! binding is built, and the registry plans from that alone. What is left here
//! is writing the three operations only the JVM has: a property getter, and the
//! two ways a failure is reported. The Kotlin declarations are the frontend's
//! own writer's, in [`super::kotlin`], over the finished generation.

use prebindgen_registry_v2::{Artifact, CarrierFeed, Fed, OperationFeed, Target, Written};
use quote::{format_ident, quote};

/// The JVM wire types a binding's carriers are, which is what a data class's
/// properties and a native method's parameters are allowed to be stated in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JniClass {
    /// A `jlong` holding a number.
    Long,
    /// A `jint`: a fieldless enum's value.
    Int,
    /// A `jlong` holding a Rust address, inside a Kotlin handle class.
    Handle,
    /// A JVM object reference.
    Object,
}

impl JniClass {
    /// Every class: what a native method's parameter or result may be.
    pub(crate) fn all() -> [JniClass; 4] {
        [
            JniClass::Long,
            JniClass::Int,
            JniClass::Handle,
            JniClass::Object,
        ]
    }
}

/// What the JNI writers need to know of a carrier: how the JVM describes it,
/// and how Kotlin spells it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Jvm {
    /// The JVM type descriptor: `J`, `I`, `Lexample/Stamp;`.
    pub descriptor: String,
    pub kotlin: KotlinType,
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
}

impl KotlinType {
    /// What the public function declares.
    pub fn public(&self) -> &str {
        match self {
            KotlinType::Value(kotlin) | KotlinType::Handle(kotlin) | KotlinType::Enum(kotlin) => {
                kotlin
            }
        }
    }

    /// What the native method declares.
    pub fn native(&self) -> &str {
        match self {
            KotlinType::Value(kotlin) => kotlin,
            KotlinType::Handle(_) => "Long",
            KotlinType::Enum(_) => "Int",
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

    type WireClass = JniClass;
    type CarrierMeta = Jvm;
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
                let Some(Fed::Carrier(result)) = &feed.result else {
                    panic!("a getter produces the carrier of its property");
                };
                let descriptor = format!("(){}", result.meta.descriptor);
                let extract = format_ident!("{}", extractor(&result.meta.descriptor));
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
        }
    }

    /// A JVM carrier is a type the `jni` crate already declares; the classes
    /// they hold are Kotlin's, written by the frontend's own writer.
    fn write_carrier(&self, _: &CarrierFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
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
