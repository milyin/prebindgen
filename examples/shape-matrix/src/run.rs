//! Building one fixture and pushing it through the real generators.
//!
//! Nothing here decides whether a shape is supported. It builds the Rust a user
//! would write, hands it to each target's generator exactly as a build script
//! would, and records the answer that comes back — including the answers that
//! arrive as a panic instead of a message.

use std::panic::AssertUnwindSafe;

use prebindgen::SourceLocation;
use prebindgen_c::Cbindgen;
use prebindgen_jni::{DataClassDecl, EnumClassDecl, JniGen, PtrClassDecl, SealedClassDecl};
use prebindgen_registry::{ExpandReturnDecl, FunctionDecl};

use crate::corpus::{Need, Position, Shape};

/// The crate name every fixture item is stamped with.
const SOURCE_CRATE: &str = "probe";

/// The function every fixture declares to the target.
const PROBE_FN: &str = "probe";

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
    if shape.spelling.starts_with('&') {
        return Some("a borrowed field needs a lifetime parameter on its declaration");
    }
    None
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
            items.push(format!(
                "pub fn {PROBE_FN}() -> {ty} {{ unimplemented!() }}"
            ));
        }
        Position::Field => {
            items.push(format!("pub struct {PROBE_TY} {{ pub v: {ty} }}"));
            items.push(format!(
                "pub fn {PROBE_FN}() -> {PROBE_TY} {{ unimplemented!() }}"
            ));
        }
        Position::Payload => {
            items.push(format!("pub enum {PROBE_TY} {{ Carried({ty}), Empty }}"));
            items.push(format!(
                "pub fn {PROBE_FN}() -> {PROBE_TY} {{ unimplemented!() }}"
            ));
        }
    }
    items.join("\n")
}

/// How a declared type is presented to the targets.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Value,
    Handle,
    Sum,
    UnitEnum,
    /// The `E` of a `Result`. Declared the way both targets document it — C
    /// takes an opaque error plus the function that renders its message, JNI
    /// takes a handle whose declared return fields feed the generated error
    /// handler. Anything less would make every fallible cell a measurement of
    /// this harness rather than of the generator.
    Error,
}

impl Kind {
    fn of(need: Need) -> Kind {
        match need {
            Need::Record => Kind::Value,
            Need::Handle => Kind::Handle,
            Need::Error => Kind::Error,
            Need::Sum => Kind::Sum,
            Need::UnitEnum => Kind::UnitEnum,
        }
    }
}

/// Every type the fixture declares, with the kind each target is told to treat
/// it as. One canonical declaration per kind — the adapter-policy axis is a
/// separate expansion of this table, not something to vary silently here.
fn declarations(shape: &Shape, position: Position) -> Vec<(String, Kind)> {
    let mut decls: Vec<(String, Kind)> = shape
        .needs
        .iter()
        .map(|n| (n.type_name().to_string(), Kind::of(*n)))
        .collect();
    match position {
        Position::Field => decls.push((PROBE_TY.to_string(), Kind::Value)),
        Position::Payload => decls.push((PROBE_TY.to_string(), Kind::Sum)),
        Position::Param | Position::Return => {}
    }
    decls
}

fn items(source: &str) -> Vec<(syn::Item, SourceLocation)> {
    let loc = SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    syn::parse_file(source)
        .expect("fixture parses")
        .items
        .into_iter()
        .map(|item| (item, loc.clone()))
        .collect()
}

fn ty(name: &str) -> syn::Type {
    syn::parse_str(name).expect("declared type name parses")
}

fn ident(name: &str) -> syn::Ident {
    syn::parse_str(name).expect("ident parses")
}

/// Run one cell, catching a panic as an outcome rather than letting it end the
/// run. A generator that panics on an unsupported shape is reporting something
/// — badly — and the table says so.
pub fn run(shape: &Shape, position: Position, target: Target) -> State {
    if let Some(reason) = not_applicable(shape, position) {
        return State::NotApplicable(reason);
    }
    let source = fixture_source(shape, position);
    let decls = declarations(shape, position);

    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| match target {
        Target::C => run_c(&source, &decls),
        Target::Jni => run_jni(&source, &decls),
    }));

    match outcome {
        Ok(Ok(())) => State::PlanSupported,
        Ok(Err(msg)) => State::Rejected(msg),
        Err(payload) => State::Panicked(panic_message(payload)),
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

fn run_jni(source: &str, decls: &[(String, Kind)]) -> Result<(), String> {
    let mut pkg = prebindgen_jni::package!().fun(FunctionDecl::new(ident(PROBE_FN)));
    let mut error_decls: Vec<ExpandReturnDecl> = Vec::new();
    for (name, kind) in decls {
        pkg = match kind {
            Kind::Value => pkg.class(DataClassDecl::new(ty(name))),
            Kind::Handle => pkg.class(PtrClassDecl::new(ty(name))),
            Kind::Sum => pkg.class(SealedClassDecl::new(ty(name))),
            Kind::UnitEnum => pkg.class(EnumClassDecl::new(ty(name))),
            Kind::Error => {
                error_decls.push(
                    ExpandReturnDecl::new(ty(name))
                        .field(FunctionDecl::new(ident(ERROR_MESSAGE_FN))),
                );
                pkg.class(
                    PtrClassDecl::new(ty(name)).method(FunctionDecl::new(ident(ERROR_MESSAGE_FN))),
                )
            }
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

    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    generation
        .write_rust(dir.path().join("generated.rs"))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn run_c(source: &str, decls: &[(String, Kind)]) -> Result<(), String> {
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
    for (name, kind) in decls {
        cbindgen = match kind {
            Kind::Value => cbindgen.data_struct(ty(name)),
            Kind::Handle => cbindgen.opaque_ptr(ty(name)),
            Kind::Sum => cbindgen.tagged_union(ty(name)),
            Kind::UnitEnum => cbindgen.enum_type(ty(name)),
            Kind::Error => cbindgen
                .opaque_error(ty(name), ident(ERROR_MESSAGE_FN))
                .ignore_function(ident(ERROR_MESSAGE_FN)),
        };
    }
    let generation = cbindgen.build().map_err(|e| e.to_string())?;

    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    generation
        .write_rust(dir.path().join("generated.rs"))
        .map_err(|e| e.to_string())?;
    Ok(())
}
