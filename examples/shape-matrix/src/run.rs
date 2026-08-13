//! Building one fixture and pushing it through the real generators.
//!
//! Nothing here decides whether a shape is supported. It builds the Rust a user
//! would write, hands it to each target's generator exactly as a build script
//! would, and records the answer that comes back — including the answers that
//! arrive as a panic instead of a message.

use std::panic::AssertUnwindSafe;

use prebindgen::SourceLocation;
use prebindgen_c::Cbindgen;
use prebindgen_jni::{
    ClassDecl, DataClassDecl, EnumClassDecl, JniGen, PtrClassDecl, SealedClassDecl,
};
use prebindgen_registry::{ExpandReturnDecl, FunctionDecl};

use crate::corpus::{Call, Need, Position, Shape};

/// The crate name every fixture item is stamped with, and so the module the
/// generated code qualifies its calls through.
///
/// Not `probe`: the fixture's function is called that, and a consumer mounting
/// the source crate as `mod probe` next to a generated `pub fn probe` wrapper
/// does not compile. The two names are different things and this crate found
/// out by conflating them.
pub const SOURCE_CRATE: &str = "flat";

/// The function every fixture declares to the target, and — since neither
/// binding renames it — the symbol a C caller links against.
pub const PROBE_FN: &str = "probe";

/// The struct or enum a `Field` / `Payload` fixture wraps the shape in.
const PROBE_TY: &str = "Probe";

/// The accessor every target needs in order to turn an error value into
/// something a foreign caller can read.
const ERROR_MESSAGE_FN: &str = "zerror_message";

/// What happened to one cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The placement is not legal Rust, so there is nothing to ask about.
    NotApplicable(&'static str),
    /// The generator refused it, and said why. **This is a good outcome** — a
    /// shape that is not supported should be reported as one.
    Rejected(String),
    /// The generator failed without producing a diagnosis. Also unsupported,
    /// but the user gets a stack trace instead of a sentence; see #191.
    Panicked(String),
    /// Generation succeeded. Note what this does *not* claim: nothing here
    /// compiles the result, links it, or runs it.
    PlanSupported,
}

impl State {
    /// The report cell.
    pub fn cell(&self) -> String {
        match self {
            State::NotApplicable(_) => "—".to_string(),
            State::PlanSupported => "plan".to_string(),
            State::Rejected(_) => "rejected".to_string(),
            State::Panicked(_) => "**panic**".to_string(),
        }
    }

    /// What the generator said, as one line.
    pub fn detail(&self) -> Option<String> {
        match self {
            State::NotApplicable(_) | State::PlanSupported => None,
            State::Rejected(msg) | State::Panicked(msg) => Some(one_line(msg)),
        }
    }
}

/// The **whole** message on one line.
///
/// Not its first line: a resolution failure opens with `1 required type(s)
/// could not be resolved:` and names the type on the next one, so a
/// first-line summary would drop the only part that identifies the cell.
fn one_line(msg: &str) -> String {
    let joined = msg
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" — ")
        .replace('|', "\\|");
    if joined.chars().count() > 240 {
        let cut: String = joined.chars().take(237).collect();
        format!("{cut}…")
    } else {
        joined
    }
}

/// Which target generated the answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    C,
    Jni,
}

impl Target {
    pub const ALL: &'static [Target] = &[Target::C, Target::Jni];

    pub fn as_str(self) -> &'static str {
        match self {
            Target::C => "C",
            Target::Jni => "Kotlin/JNI",
        }
    }

    /// The part of a cell id this target contributes. See
    /// [`Position::slug`](crate::corpus::Position::slug).
    pub fn slug(self) -> &'static str {
        match self {
            Target::C => "c",
            Target::Jni => "jni",
        }
    }
}

/// Why a placement is not legal Rust to begin with.
///
/// Stated as a rule rather than discovered, because the fixture generator emits
/// declarations without lifetime parameters: a borrowed field would need one,
/// and `impl Trait` is not a field type in any case. Everything else is asked
/// of the generator rather than pre-judged here.
pub fn not_applicable(shape: &Shape, position: Position) -> Option<&'static str> {
    let in_declaration = matches!(position, Position::Field | Position::Payload);
    if !in_declaration {
        return None;
    }
    if shape.spelling.contains("impl Fn") {
        return Some("`impl Trait` is not a field type");
    }
    if shape.spelling.contains('&') {
        return Some("a borrowed field needs a lifetime parameter on its declaration");
    }
    None
}

/// The same spelling, with every borrow given an explicit `'static`.
///
/// A returned borrow needs a lifetime and the probe function has no parameter
/// to elide one from, so `-> &Handle` is not Rust at all. Writing `'static`
/// keeps the shape and makes the fixture legal; the alternative — adding an
/// anchor parameter — would change what the cell measures.
///
/// This was found the hard way: before it, every borrow-returning cell reported
/// `plan` for a fixture that could not have compiled.
fn anchored(spelling: &str) -> String {
    let mut out = String::new();
    let mut chars = spelling.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c != '&' {
            continue;
        }
        // Already written with a lifetime — `Cow<'static, str>` and friends.
        if chars.peek() == Some(&'\'') {
            continue;
        }
        out.push_str("'static ");
    }
    out
}

/// The fixture's Rust source: the shape's supporting declarations, the wrapper
/// declaration the position needs, and the function that makes it cross.
pub fn fixture_source(shape: &Shape, position: Position) -> String {
    let mut items: Vec<String> = shape.needs.iter().map(|n| n.source().to_string()).collect();
    let ty = shape.spelling;

    match position {
        Position::Param => {
            items.push(format!("pub fn {PROBE_FN}(v: {ty}) {{ let _ = v; }}"));
        }
        Position::Return => {
            let ty = anchored(ty);
            items.push(format!(
                "pub fn {PROBE_FN}() -> {ty} {{ unimplemented!() }}"
            ));
        }
        Position::Field => {
            items.push(format!(
                "#[derive(Clone)] pub struct {PROBE_TY} {{ pub v: {ty} }}"
            ));
            items.push(format!(
                "pub fn {PROBE_FN}() -> {PROBE_TY} {{ unimplemented!() }}"
            ));
        }
        Position::Payload => {
            items.push(format!(
                "#[derive(Clone)] pub enum {PROBE_TY} {{ Carried({ty}), Empty }}"
            ));
            items.push(format!(
                "pub fn {PROBE_FN}() -> {PROBE_TY} {{ unimplemented!() }}"
            ));
        }
    }
    items.join("\n")
}

/// The fixture for a call: one function taking every parameter of the call
/// shape, and the declarations they need.
pub fn call_fixture_source(call: &Call) -> String {
    let mut items: Vec<String> = call.needs.iter().map(|n| n.source().to_string()).collect();
    let params = call
        .params
        .iter()
        .enumerate()
        .map(|(n, ty)| format!("p{n}: {ty}"))
        .collect::<Vec<_>>()
        .join(", ");
    let uses = (0..call.params.len())
        .map(|n| format!("let _ = p{n};"))
        .collect::<Vec<_>>()
        .join(" ");
    let ret = call.ret.map(|r| format!(" -> {r}")).unwrap_or_default();
    let body = match call.ret {
        // A fallible call must actually return: its failure path is the thing
        // under test, and a body that panicked would abort instead — a panic
        // cannot cross `extern "C"`.
        Some(ret) if ret.starts_with("Result<()") => format!("{uses} Ok(())"),
        Some(_) => format!("{uses} unimplemented!()"),
        None => uses,
    };
    items.push(format!("pub fn {PROBE_FN}({params}){ret} {{ {body} }}"));
    items.join("\n")
}

/// What a call fixture declares. A call places nothing in a struct or an enum,
/// so there is no wrapper type — only the supporting declarations.
pub fn call_declarations(call: &Call) -> Vec<Decl> {
    call.needs.iter().map(|n| Decl::of(*n)).collect()
}

/// The declaration axis: what a declared type is declared **as**.
///
/// Named in the JNI adapter's vocabulary on purpose. Its class kinds are a
/// closed, public enum ([`ClassDecl`]), so [`kind_of`] can be exhaustive over
/// it and a fifth kind stops this crate compiling — the same chain
/// [`tag_of`](crate::tag::tag_of) gives the type axis.
///
/// The C build-script API has no such closure: eleven declarator methods and no
/// type unifying them. It is the older surface and is to be reworked in this
/// style (#192, #399), so the harness models the JNI vocabulary and
/// [translates](to_c) to C. Building the harness around C's current shape would
/// bake a quirk of the API being replaced into the thing meant to outlive it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassKind {
    /// An opaque handle the foreign side owns and releases.
    Ptr,
    /// A product of fields, crossing by value.
    Data,
    /// An enum whose alternatives carry payloads.
    Sealed,
    /// An enum whose alternatives are all fieldless.
    Enum,
}

impl ClassKind {
    pub const ALL: &'static [ClassKind] = &[
        ClassKind::Ptr,
        ClassKind::Data,
        ClassKind::Sealed,
        ClassKind::Enum,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ClassKind::Ptr => "opaque handle",
            ClassKind::Data => "value struct",
            ClassKind::Sealed => "enum with payloads",
            ClassKind::Enum => "fieldless enum",
        }
    }

    /// The part of a cell id this kind contributes. See
    /// [`Position::slug`](crate::corpus::Position::slug).
    pub fn slug(self) -> &'static str {
        match self {
            ClassKind::Ptr => "ptr",
            ClassKind::Data => "data",
            ClassKind::Sealed => "sealed",
            ClassKind::Enum => "enum",
        }
    }

    /// This kind as the JNI adapter's own declaration.
    pub fn decl(self, ty: syn::Type) -> ClassDecl {
        match self {
            ClassKind::Ptr => PtrClassDecl::new(ty).into(),
            ClassKind::Data => DataClassDecl::new(ty).into(),
            ClassKind::Sealed => SealedClassDecl::new(ty).into(),
            ClassKind::Enum => EnumClassDecl::new(ty).into(),
        }
    }
}

/// The gate on the declaration axis.
///
/// Exhaustive over the adapter's [`ClassDecl`], so a fifth class kind is a
/// compile error here, and `class_kind_vocabulary_matches_the_adapter` then
/// fails until this crate enumerates it.
pub fn kind_of(decl: &ClassDecl) -> ClassKind {
    match decl {
        ClassDecl::Ptr(_) => ClassKind::Ptr,
        ClassDecl::Data(_) => ClassKind::Data,
        ClassDecl::Sealed(_) => ClassKind::Sealed,
        ClassDecl::Enum(_) => ClassKind::Enum,
    }
}

/// One type the fixture declares.
#[derive(Clone)]
pub struct Decl {
    pub name: String,
    pub class: ClassKind,
    /// True for the `E` of a `Result`, which is not a fifth class kind but a
    /// **role** a declared type plays. Each target states it its own way — a
    /// handle whose declared return fields feed the generated error handler,
    /// or an opaque error plus the function that renders its message — and
    /// stating neither would make every fallible cell a measurement of this
    /// harness rather than of the generator.
    pub error: bool,
}

impl Decl {
    fn of(need: Need) -> Decl {
        let (class, error) = match need {
            Need::Record => (ClassKind::Data, false),
            Need::Handle => (ClassKind::Ptr, false),
            Need::Sum => (ClassKind::Sealed, false),
            Need::UnitEnum => (ClassKind::Enum, false),
            Need::Error => (ClassKind::Ptr, true),
        };
        Decl {
            name: need.type_name().to_string(),
            class,
            error,
        }
    }
}

/// Every type the fixture declares, with the kind each target is told to treat
/// it as. One canonical declaration per kind — varying the declaration of a
/// given type is the adapter-policy axis, a separate expansion of this table
/// rather than something to change silently here.
pub fn declarations(shape: &Shape, position: Position) -> Vec<Decl> {
    let mut decls: Vec<Decl> = shape.needs.iter().map(|n| Decl::of(*n)).collect();
    let wrapper = match position {
        Position::Field => Some(ClassKind::Data),
        Position::Payload => Some(ClassKind::Sealed),
        Position::Param | Position::Return => None,
    };
    if let Some(class) = wrapper {
        decls.push(Decl {
            name: PROBE_TY.to_string(),
            class,
            error: false,
        });
    }
    decls
}

/// The fixture's items, as the **model** sees them.
///
/// Filtered to the four item kinds a `#[prebindgen]` surface declares, the same
/// filter `examples/emitcheck` applies to its own source file. A fixture is a
/// real Rust file and so carries things a flat API does not declare — an
/// `impl Display for ZError` is part of what makes the crate compile and is not
/// part of its boundary.
///
/// Found by feeding them: an `impl` block reaches the frontend as *"is an item
/// kind the prebindgen source language does not model"*, which failed the whole
/// binding and turned 32 cells into rejections that had nothing to do with
/// their shape.
fn items(source: &str) -> Vec<(syn::Item, SourceLocation)> {
    let loc = SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    syn::parse_file(source)
        .expect("fixture parses")
        .items
        .into_iter()
        .filter(|item| {
            matches!(
                item,
                syn::Item::Fn(_) | syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Const(_)
            )
        })
        .map(|item| (item, loc.clone()))
        .collect()
}

fn ty(name: &str) -> syn::Type {
    syn::parse_str(name).expect("declared type name parses")
}

fn ident(name: &str) -> syn::Ident {
    syn::parse_str(name).expect("ident parses")
}

/// What one cell produced: the generator's answer, and — when it answered at
/// all — the Rust it emitted, which is what the next stage type-checks.
pub struct Outcome {
    pub state: State,
    /// The generated Rust. `Some` exactly when `state` is `PlanSupported`.
    pub emitted: Option<String>,
}

/// Run one cell, catching a panic as an outcome rather than letting it end the
/// run. A generator that panics on an unsupported shape is reporting something
/// — badly — and the table says so.
pub fn run(shape: &Shape, position: Position, target: Target) -> Outcome {
    if let Some(reason) = not_applicable(shape, position) {
        return Outcome {
            state: State::NotApplicable(reason),
            emitted: None,
        };
    }
    generate(
        &fixture_source(shape, position),
        &declarations(shape, position),
        target,
    )
}

/// Run one value cell with its declared types declared **differently**.
///
/// The same Rust, the same position, one declaration changed — which is the
/// only way to see whether an answer belongs to the shape or to the policy. A
/// `Rec` declared as a value struct and the same `Rec` declared as an opaque
/// handle are the same source code and, quite often, not the same answer.
///
/// The override applies to the shape's supporting types and not to the wrapper
/// a field or payload fixture declares: the question is how the *subject*
/// crosses, and re-declaring its container would ask a different one.
pub fn run_policy(shape: &Shape, position: Position, class: ClassKind, target: Target) -> Outcome {
    if let Some(reason) = not_applicable(shape, position) {
        return Outcome {
            state: State::NotApplicable(reason),
            emitted: None,
        };
    }
    let mut decls = declarations(shape, position);
    for decl in &mut decls {
        // An error type keeps its role: the `E` of a `Result` is declared the
        // way each target documents, and overriding that would measure the
        // error channel rather than the policy.
        if !decl.error && shape.needs.iter().any(|n| n.type_name() == decl.name) {
            decl.class = class;
        }
    }
    generate(&fixture_source(shape, position), &decls, target)
}

/// Run one call shape. Same driver, different fixture: a call is a second axis
/// over the same generators, not a second harness.
pub fn run_call(call: &Call, target: Target) -> Outcome {
    generate(&call_fixture_source(call), &call_declarations(call), target)
}

fn generate(source: &str, decls: &[Decl], target: Target) -> Outcome {
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| match target {
        Target::C => run_c(source, decls),
        Target::Jni => run_jni(source, decls),
    }));

    match outcome {
        Ok(Ok(emitted)) => Outcome {
            state: State::PlanSupported,
            emitted: Some(emitted),
        },
        Ok(Err(msg)) => Outcome {
            state: State::Rejected(msg),
            emitted: None,
        },
        Err(payload) => Outcome {
            state: State::Panicked(panic_message(payload)),
            emitted: None,
        },
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panicked with a non-string payload".to_string()
    }
}

/// The generated Rust, as the target wrote it.
///
/// Through a file rather than a string because that is the only surface either
/// generator offers — a build script writes `generated_bindings.rs` and the
/// consumer `include!`s it, so this is also exactly what a consumer compiles.
fn read_back(
    write: impl FnOnce(&std::path::Path) -> Result<std::path::PathBuf, String>,
) -> Result<String, String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let written = write(&dir.path().join("generated.rs"))?;
    std::fs::read_to_string(written).map_err(|e| e.to_string())
}

fn run_jni(source: &str, decls: &[Decl]) -> Result<String, String> {
    let mut pkg = prebindgen_jni::package!().fun(FunctionDecl::new(ident(PROBE_FN)));
    let mut error_decls: Vec<ExpandReturnDecl> = Vec::new();
    for decl in decls {
        pkg = if decl.error {
            error_decls.push(
                ExpandReturnDecl::new(ty(&decl.name))
                    .field(FunctionDecl::new(ident(ERROR_MESSAGE_FN))),
            );
            pkg.class(
                PtrClassDecl::new(ty(&decl.name))
                    .method(FunctionDecl::new(ident(ERROR_MESSAGE_FN))),
            )
        } else {
            pkg.class(decl.class.decl(ty(&decl.name)))
        };
    }
    let mut builder = JniGen::builder()
        .items(items(source))
        .set_package_prefix("io.prebindgen.matrix")
        .package(pkg);
    for decl in error_decls {
        builder = builder.expand(decl);
    }
    let generation = builder.build().map_err(|e| e.to_string())?;

    // Kotlin is half of what this target produces, and until it is asked for,
    // a cell reporting success has shown only the Rust half. A binding whose
    // Kotlin cannot be written is not a binding.
    let kotlin_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let written = generation
        .write_kotlin(kotlin_dir.path())
        .map_err(|e| e.to_string())?;
    if written.is_empty() {
        return Err("the binding produced no Kotlin at all".to_string());
    }

    read_back(|path| generation.write_rust(path).map_err(|e| e.to_string()))
}

/// The declaration axis, spoken to the C builder.
///
/// A **translation**, and deliberately not the model. The C build-script API is
/// the older of the two: a declarator per shape, no closed kind vocabulary to
/// match on, and a rework in the JNI style is planned (#192, #399). So this
/// function is the one place that knows C's current spelling, and it is
/// expected to be rewritten wholesale when that lands — nothing else in the
/// crate is shaped around it.
pub fn to_c(cbindgen: prebindgen_c::CbindgenBuilder, decl: &Decl) -> prebindgen_c::CbindgenBuilder {
    if decl.error {
        return cbindgen
            .opaque_error(ty(&decl.name), ident(ERROR_MESSAGE_FN))
            .ignore_function(ident(ERROR_MESSAGE_FN));
    }
    match decl.class {
        ClassKind::Ptr => cbindgen.opaque_ptr(ty(&decl.name)),
        ClassKind::Data => cbindgen.data_struct(ty(&decl.name)),
        ClassKind::Sealed => cbindgen.tagged_union(ty(&decl.name)),
        ClassKind::Enum => cbindgen.enum_type(ty(&decl.name)),
    }
}

fn run_c(source: &str, decls: &[Decl]) -> Result<String, String> {
    let mut cbindgen = Cbindgen::builder()
        .items(items(source))
        .source_module(syn::parse_str(SOURCE_CRATE).expect("crate name is a path"))
        .free_memory_function("matrix_free")
        .function(ident(PROBE_FN))
        // The probe function does not return `Result`, and a C binding whose
        // inputs can fail must say what happens then. `.panic()` is the
        // documented answer for exactly that shape; without it every cell with
        // a fallible input would report this harness\'s omission.
        .panic();
    for decl in decls {
        cbindgen = to_c(cbindgen, decl);
    }
    let generation = cbindgen.build().map_err(|e| e.to_string())?;

    read_back(|path| generation.write_rust(path).map_err(|e| e.to_string()))
}
