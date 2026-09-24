//! What a language adapter writes, and the shared vocabulary the binding is
//! stated in.
//!
//! A target knows how to write its language and nothing else. Everything a
//! plan depends on — which carrier holds a value, which operations move it,
//! what form an exported function takes — is stated before planning starts,
//! as the [`Binding`](crate::binding::Binding) a frontend hands
//! [`generate`](crate::generate). The registry plans from that data alone and
//! calls no target code while it does. Only once the plan is complete does it
//! call the target's writers, each with a **feed**: everything one piece of
//! text needs, already worked out.
//!
//! # The words the doc comments use
//!
//! - The **binding** is the crate a user builds and ships — `zenoh-flat-c`,
//!   `zenoh-flat-jni` — whose `build.rs` declares what to expose. It is the
//!   consumer of everything here.
//! - The **target** is a language together with the calling interface it
//!   reaches Rust through — C, or Kotlin through JNI — and, in code, the
//!   adapter implementing [`Target`] for it. The same crate is the
//!   **frontend** when it faces the binding's `build.rs`, and the target
//!   adapter when it faces the registry.
//! - The **registry** is this crate: it plans and writes the Rust around what
//!   the target writes.
//! - A **source function** is a Rust function in the source crate — the
//!   `#[prebindgen]`-marked crate the captures come from — that the binding
//!   asked to expose. It is never exported itself; it stays where it is and is
//!   called. An **exported function**, or **wrapper**, is the Rust function
//!   the registry generates around it: one the target's calling interface can
//!   reach, which converts what arrives, calls the source function once, and
//!   converts what it returns.
//! - A **carrier** is a Rust type generated code may hold a value in at the
//!   boundary — `i64`, `*mut ledger_t`, a `repr(C)` `Stamp`, a `JObject` —
//!   declared by the frontend as a [`WireType`](crate::binding::WireType).
//!
//! This is the first increment (docs/v2). What it carries is the scalar, the
//! owned struct, the owned opaque handle and the fieldless enum;
//! `docs/v2/implementation.md` lists exactly what is absent.

use prebindgen_flat::flat::{Enum, EnumValue, FieldShape, TypeRef};
use proc_macro2::TokenStream;

use crate::{
    binding::{CarrierOf, WireClass},
    outcome::Capability,
};

/// Which way a value crosses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Produce the Rust value a source function expects.
    IntoRust,
    /// Encode a Rust value for foreign code.
    OutOfRust,
}

impl Direction {
    /// The other way.
    pub fn reversed(self) -> Self {
        match self {
            Direction::IntoRust => Direction::OutOfRust,
            Direction::OutOfRust => Direction::IntoRust,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Direction::IntoRust => "into_rust",
            Direction::OutOfRust => "out_of_rust",
        }
    }
}

/// One value conversion to plan: an exact source type and a direction.
#[derive(Clone, Debug)]
pub struct Crossing {
    pub ty: TypeRef,
    pub direction: Direction,
}

/// A valid request needing a capability nobody has implemented yet.
///
/// Not an error: the run continues, the affected outputs are skipped, and the
/// report says which capability would unblock them. A frontend records one in
/// the binding for a declaration it cannot lower — a declarator it has no
/// representation for, an enum numbered beyond its carrier — and the registry
/// reports it wherever a value or an output meets it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unsupported {
    pub capability: Capability,
    pub explanation: String,
}

impl Unsupported {
    pub fn new(capability: impl Into<String>, explanation: impl Into<String>) -> Self {
        Unsupported {
            capability: Capability::new(capability),
            explanation: explanation.into(),
        }
    }
}

/// The input or the generator is wrong. Never a capability question.
#[derive(Clone, Debug)]
pub enum PlanningError {
    /// Malformed or contradictory configuration.
    InvalidInput(String),
    /// A generator defect or a violated internal contract.
    InternalInvariant(String),
}

impl std::fmt::Display for PlanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanningError::InvalidInput(message) => write!(f, "invalid input: {message}"),
            PlanningError::InternalInvariant(message) => {
                write!(f, "internal invariant: {message}")
            }
        }
    }
}

impl std::error::Error for PlanningError {}

// ---------------------------------------------------------------------------
// Relations: how a Rust value is built or read
// ---------------------------------------------------------------------------

/// One field of a struct relation, or one argument of a constructor relation.
#[derive(Clone, Debug)]
pub struct Part {
    /// The field's name, absent for a positional field.
    pub name: Option<String>,
    /// Its position in the relation, which is its identity when it has no name.
    pub index: usize,
    /// Its exact source type.
    pub ty: TypeRef,
    /// The `#[cfg]` conditions the source field was written under, which the
    /// capture reader could not answer — empty in the ordinary case.
    ///
    /// A target declaring this part as a member of its own struct puts them
    /// on that member. It must not read them: the registry puts the same
    /// conditions on every instruction that serves this part, so a member
    /// declared under them is read under them.
    pub conditions: Vec<TokenStream>,
}

impl Part {
    /// How a diagnostic and a value path address this part.
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => self.index.to_string(),
        }
    }

    /// How generated Rust reaches this part of a struct: `secs`, or `0`.
    pub fn member(&self) -> syn::Member {
        match &self.name {
            Some(name) => syn::Member::Named(quote::format_ident!("{name}")),
            None => syn::Member::Unnamed(syn::Index::from(self.index)),
        }
    }
}

/// A struct read through, or built from, its fields.
#[derive(Clone, Debug)]
pub struct StructRelation {
    /// The struct's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub name: String,
    pub parts: Vec<Part>,
}

/// How the registry constructs or reads a Rust value.
///
/// Which one a value takes is stated by the representation that applies to it:
/// a `Terminal` one is the atomic relation, and a `Product` one names its
/// relation with a [`Via`](crate::binding::Via).
#[derive(Clone, Debug)]
pub enum Relation {
    /// The whole value converted by one operation: no parts, no recursion.
    Atomic,
    /// The struct's fields.
    Struct(StructRelation),
    /// A callback's arguments, in order: unnamed parts, each crossing the
    /// other way from the callback, since Rust hands them to the callable.
    Callback(Vec<Part>),
}

impl Relation {
    pub fn parts(&self) -> &[Part] {
        match self {
            Relation::Atomic => &[],
            Relation::Struct(strukt) => &strukt.parts,
            Relation::Callback(args) => args,
        }
    }

    /// The word a report uses for this relation.
    pub fn label(&self) -> String {
        match self {
            Relation::Atomic => "atomic".to_string(),
            Relation::Struct(strukt) => format!("{}.fields", strukt.name),
            Relation::Callback(_) => "callback.args".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Failures
// ---------------------------------------------------------------------------

/// Which boundary route a failure takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureCategory {
    /// Reported by an explicitly selected source operation.
    Domain,
    /// An invalid foreign value or a failed representation conversion.
    Binding,
    /// A target runtime failure, such as a JNI call.
    Runtime,
}

impl FailureCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            FailureCategory::Domain => "domain",
            FailureCategory::Binding => "binding",
            FailureCategory::Runtime => "runtime",
        }
    }
}

/// How a failure route ends the wrapper's call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Terminal {
    /// Return this expression to the caller.
    Return(syn::Expr),
    /// Give up: the process cannot continue correctly.
    Abort,
}

/// One generated unit: a helper function, a type declaration.
///
/// Contributed whole by whoever needs it. The name is its identity, so two
/// operations depending on the same helper emit it once.
#[derive(Clone, Debug)]
pub struct Artifact {
    pub name: String,
    pub rust: TokenStream,
}

impl Artifact {
    pub fn new(name: impl Into<String>, rust: TokenStream) -> Self {
        Artifact {
            name: name.into(),
            rust,
        }
    }
}

// ---------------------------------------------------------------------------
// Fieldless enums
// ---------------------------------------------------------------------------

/// The values of a fieldless enum a target may mirror, or why it may not.
///
/// What stops a mirror stops every target alike, so the check lives here
/// rather than once per adapter:
///
/// * the type is not a fieldless enum at all, which is the binding declaring
///   one thing as another;
/// * a value's number could not be evaluated — a `const`, arithmetic,
///   anything but a literal — and the numbers are what a mirror is made of;
/// * a value was written under a `#[cfg]`. The model numbers every value as
///   present, so a conditional value followed by an implicit one gives
///   numbers the compiled enum disagrees with, and a mirror entry for an
///   absent value names a variant that is not there;
/// * the enum, or any of its values, is `#[non_exhaustive]`. Both put a value
///   out of another crate's reach, and a binding crate is always another
///   crate. A non-exhaustive enum needs a wildcard arm, and going out of Rust
///   there is nothing for that arm to produce — the target has a value for
///   each value it knows, and none for one it does not. A non-exhaustive
///   *value* cannot be constructed from outside at all, and its pattern needs
///   a `..`; that holds for a unit value too, whose constructor is private
///   outside the crate that declared it. Preserving delimiters does not make
///   such a value constructible, so the enum is refused;
/// * the enum has no values, and an enumeration of nothing is not one a
///   target can declare.
///
/// `language` is the adapter's own name, for the capability code: a refusal
/// reads `unsupported.c.enum_discriminant`, and the next target's reads its
/// own.
pub fn mirrored_enum<'a>(
    unit: Option<&'a Enum>,
    declared_as: &str,
    language: &str,
) -> Result<&'a [EnumValue], Unsupported> {
    let Some(unit) = unit else {
        return Err(Unsupported::new(
            format!("unsupported.{language}.not_an_enum"),
            format!("`{declared_as}` is declared as an enum, and is not a fieldless enum"),
        ));
    };
    if let Err(value) = unit.discriminant_values() {
        return Err(Unsupported::new(
            format!("unsupported.{language}.enum_discriminant"),
            format!(
                "`{declared_as}` has a value `{value}` whose number the model cannot \
                 evaluate, and what crosses is the numbers"
            ),
        ));
    }
    if unit.is_non_exhaustive() {
        return Err(Unsupported::new(
            format!("unsupported.{language}.non_exhaustive_enum"),
            format!(
                "`{declared_as}` is `#[non_exhaustive]`, or one of its values is, and a \
                 binding crate cannot name such a value"
            ),
        ));
    }
    if unit.has_conditional_value() {
        return Err(Unsupported::new(
            format!("unsupported.{language}.conditional_value"),
            format!(
                "`{declared_as}` has a value written under a `#[cfg]`, and the numbers \
                 count every value as present"
            ),
        ));
    }
    if unit.values.is_empty() {
        return Err(Unsupported::new(
            format!("unsupported.{language}.empty_enum"),
            format!("`{declared_as}` has no values, and an enumeration needs one"),
        ));
    }
    Ok(&unit.values)
}

/// [`mirrored_enum`]'s values with their numbers as an `i32`, for a target
/// whose carrier is 32 bits: a C `int`, a JNI `jint`.
///
/// Everything [`mirrored_enum`] refuses, plus a number outside `i32`.
/// `#[repr(i64)] enum P { High = 2147483648 }` is valid Rust, and a 32-bit
/// carrier cannot hold it, so the enum is refused rather than mirrored with a
/// number the target's side truncates or does not accept.
pub fn mirrored_i32_enum<'a>(
    unit: Option<&'a Enum>,
    declared_as: &str,
    language: &str,
) -> Result<Vec<(&'a EnumValue, i32)>, Unsupported> {
    mirrored_enum(unit, declared_as, language)?
        .iter()
        .map(|value| {
            let number = value
                .discriminant
                .expect("mirrored_enum refuses an enum with a number it could not evaluate");
            match i32::try_from(number) {
                Ok(number) => Ok((value, number)),
                Err(_) => Err(Unsupported::new(
                    format!("unsupported.{language}.enum_range"),
                    format!(
                        "`{declared_as}` numbers `{}` {number}, which a 32-bit carrier cannot \
                         hold",
                        value.name
                    ),
                )),
            }
        })
        .collect()
}

/// One value of a fieldless enum, as the two enum operations match it.
///
/// [`Self::shape`] is why this is not a bare name: `enum Op { Add(), Mul {} }`
/// has no fields and is a fieldless enum to the model, but its values are
/// spelled `Add()` and `Mul {}` in a pattern and a constructor alike. The
/// registry renders through the model's own speller, so what it writes is
/// what the source declared.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumArm {
    /// The value's name in the source enum.
    pub name: syn::Ident,
    /// Its constructor and pattern shape, from the model.
    pub shape: FieldShape,
    /// What it is carried as: the target's own enum value, or a number.
    pub carried: syn::Expr,
}

// ---------------------------------------------------------------------------
// The writers
// ---------------------------------------------------------------------------

/// The language adapter, as the engine sees it: a set of writers.
///
/// Nothing here decides anything a plan depends on. Which carrier holds a
/// value, which operations move it, what form an exported function takes and
/// which wire types each place may hold are all stated in the
/// [`Binding`](crate::binding::Binding) before planning starts, and the
/// registry plans from that alone. A writer is called only once the plan is
/// complete, is fed everything its piece of text needs, and cannot refuse:
/// whatever could make a value unsupported was decided when the binding was
/// built.
///
/// The associated types are the adapter's own vocabulary, which the registry
/// compares and hashes but never reads.
pub trait Target: Sized {
    /// This target's name in a report — `"c"`, `"jni"`. Intrinsic to the
    /// adapter, so nothing has to carry it alongside the requests.
    const NAME: &'static str;

    /// The adapter's few wire types, which acceptance is stated in: C's `I64`,
    /// `Pointer`, `Aggregate`; JNI's `Long`, `Int`, `Object`.
    type WireClass: WireClass;

    /// What a carrier tells the writers beyond its Rust type: C's name for
    /// it, a JVM descriptor and Kotlin type.
    ///
    /// Compared, because two carriers of one Rust type may differ in it —
    /// every JVM object is a `JObject`, and one holding an `example.Stamp` is
    /// a different carrier from one holding an `other.Stamp`.
    type CarrierMeta: Clone + Eq + std::hash::Hash + std::fmt::Debug;

    /// The target's own operations: a JVM getter call, a throw. C has none.
    type Op: Clone + Eq + std::hash::Hash + std::fmt::Debug;

    /// What only the target's foreign writer reads about an output: a Kotlin
    /// package and name. C has none.
    type OutputMeta: Clone + Eq + std::hash::Hash + std::fmt::Debug;

    /// One of this target's operations, as a single Rust expression.
    ///
    /// One expression and nothing around it: no `let`, no `match`, no return.
    /// The feed names every operand and says what each holds.
    fn write_operation(&self, op: &Self::Op, feed: &OperationFeed<'_, Self>) -> Written;

    /// The Rust items a carrier needs declared, in order — a `repr(C)` struct
    /// or enum mirror, an incomplete type behind a pointer — or none for a
    /// type Rust already has, such as `i64` or `JObject`.
    ///
    /// The registry puts the condition of the source item the carrier serves
    /// on every item returned, so each is one item.
    fn write_carrier(&self, feed: &CarrierFeed<'_, Self>) -> Vec<TokenStream>;
}

/// What a writer produced, and the helpers it needs emitted once beside it.
#[derive(Clone, Debug)]
pub struct Written {
    pub text: TokenStream,
    /// Generated units this text needs in order to compile — a helper
    /// function — each emitted once whoever needs it.
    pub helpers: Vec<Artifact>,
}

impl Written {
    /// Text needing no helper.
    pub fn new(text: TokenStream) -> Self {
        Written {
            text,
            helpers: Vec::new(),
        }
    }

    /// The same, needing `helper` emitted too.
    pub fn with_helper(mut self, helper: Artifact) -> Self {
        self.helpers.push(helper);
        self
    }
}

/// What the registry knows about one operand or result of an operation.
pub enum Fed<'a, T: Target> {
    /// An exact Rust type from the source model.
    Source(&'a TypeRef),
    /// A carrier the binding declared.
    Carrier(&'a CarrierOf<T>),
    /// What a callback's `capture` produced: a value of the target's own
    /// making, which the registry moves into the closure and names nothing
    /// about.
    Captured,
}

/// Everything one application of a target operation needs written.
pub struct OperationFeed<'a, T: Target> {
    /// The value the operation is applied to, already named, and what it
    /// holds — `None` for a reporting operation, which is handed the error.
    pub value: Option<(syn::Ident, Fed<'a, T>)>,
    /// The runtime contexts the operation asked for, by name, each bound to
    /// the wrapper parameter that supplies it.
    pub contexts: Vec<(String, syn::Ident)>,
    /// The error a reporting operation reports.
    pub error: Option<syn::Ident>,
    /// What the expression must produce, if anything.
    pub result: Option<Fed<'a, T>>,
    /// For an operation applied once per part — a `Product`'s `read` — the
    /// part it is applied to.
    pub part: Option<&'a Part>,
    /// For a callback's `capture` and `invoke`: the carriers of the callback's
    /// arguments, in order — each named for `invoke`, which is handed them,
    /// and unnamed for `capture`, which runs before any call.
    pub args: Vec<(Option<syn::Ident>, &'a CarrierOf<T>)>,
}

impl<T: Target> OperationFeed<'_, T> {
    /// The wrapper parameter supplying the context the operation asked for
    /// under `name`.
    ///
    /// # Panics
    ///
    /// When the operation did not ask for it, which is the adapter reading a
    /// context it never declared.
    pub fn context(&self, name: &str) -> &syn::Ident {
        self.contexts
            .iter()
            .find(|(asked, _)| asked == name)
            .map(|(_, ident)| ident)
            .unwrap_or_else(|| panic!("the operation did not ask for the `{name}` context"))
    }
}

/// Everything one carrier's declaration needs written.
pub struct CarrierFeed<'a, T: Target> {
    pub carrier: &'a CarrierOf<T>,
    /// For the carrier of a `Product` or a `Callback`: each part — a field,
    /// or an argument — with the carrier it resolved to, in part order.
    pub members: Vec<(&'a Part, &'a CarrierOf<T>)>,
    /// For a carrier of a fieldless enum's value: that enum, from the model.
    pub unit: Option<&'a Enum>,
}
