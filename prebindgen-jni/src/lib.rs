//! Generate Rust/JNI wrappers and Kotlin bindings from conversion slots.
//! Supported declarations include `package!`, `ptr_class!`, `fun!`,
//! expansion declarations, `.name()`, source selection, package prefix and native initialization.
//! Declaration processing fills occurrence slots before wrappers are generated.
#![warn(missing_docs)]
use prebindgen_flat::{type_key, Flat, Result};
mod scalar;
use prebindgen_flat::Primitive;
use prebindgen_tools::{boxed::BoxedWire, value::Value};
/// The carrier of one JNI wire.
///
/// This adapter-specific type keeps C and JNI converters from being mixed in
/// one model: their codecs use different carriers and allocation layouts.
#[derive(Clone, Debug)]
pub enum JniWire {
    /// A primitive in its JVM carrier.
    Scalar(Primitive),
    /// A `java.lang.String`.
    Text,
    /// A `byte[]`.
    Bytes,
    /// A boxed allocation address in a `long`, owned by a Kotlin handle class.
    Handle(Box<BoxedWire>),
    /// An `int` selector arm index.
    Selector,
}
use prebindgen_tools::{Access, Bindings as GenericBindings, Codec};
/// Flat source model with JNI handle conversion data for wrapper emission.
///
/// Use [`JniCodec`] when binding conversions. Rust rejects a C codec because its
/// wire type belongs to a different adapter:
/// ```compile_fail
/// use prebindgen_jni::Bindings;
/// use prebindgen_tools::{bind_input, Input};
/// fn wrong(flat: &mut Bindings) {
///     bind_input(flat, &prebindgen_c::CCodec, "echo", "x", &Input::Leaf, &()).unwrap();
/// }
/// ```
/// A completed JNI model also cannot be passed to the C generator:
/// ```compile_fail
/// fn wrong(flat: prebindgen_jni::Bindings) {
///     prebindgen_c::Cbindgen::new(flat);
/// }
/// ```
pub type Bindings = GenericBindings<JniWire>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

pub use prebindgen_tools::{
    expand_param, expand_return, fields, fun, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl,
};
/// A Kotlin owning handle class for one nonzero-sized source type.
///
/// Generated code requires `Send` because the handle can be closed on any JVM
/// thread. Foreign output observations also require `Clone`. Inputs may own,
/// share or exclusively borrow the source value.
#[derive(Clone)]
pub struct PtrClassDecl {
    /// Source type, resolved against the selected flat model.
    pub ty: syn::Type,
    /// Literal Kotlin class name; absent means the last segment of the source type.
    pub name: Option<String>,
}
impl PtrClassDecl {
    /// Declare a handle class with a name derived from the source type.
    ///
    /// `ty` is the source type the class owns, as written in the source crate.
    pub fn new(ty: syn::Type) -> Self {
        Self { ty, name: None }
    }
    /// Override the Kotlin short class name with `name`, validated during build.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
/// A Kotlin package containing declared handle classes and top-level functions.
///
/// Its name is relative to [`JniGenBuilder::set_package_prefix`].
#[derive(Clone, Default)]
pub struct PackageDecl {
    /// Relative dotted package name; empty means the root package prefix.
    pub name: String,
    /// Owning handle classes to emit in this package.
    pub classes: Vec<PtrClassDecl>,
    /// Source functions to export as top-level Kotlin functions.
    pub functions: Vec<FunctionDecl>,
}
impl PackageDecl {
    /// Create an empty package relative to the configured root prefix.
    ///
    /// `name` is a dotted package name relative to that prefix, or empty for the
    /// prefix itself.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    /// Add an owning handle class. Duplicate source types or names fail during build.
    pub fn class(mut self, class: PtrClassDecl) -> Self {
        self.classes.push(class);
        self
    }
    /// Add a top-level Kotlin function with optional occurrence overrides.
    ///
    /// Names default to camel case; [`FunctionDecl::name`] overrides that spelling.
    pub fn fun(mut self, f: FunctionDecl) -> Self {
        self.functions.push(f);
        self
    }
}
#[macro_export]
/// Declare a handle class from Rust type syntax, without requiring that type in scope.
///
/// Returns [`PtrClassDecl`]. Invalid type syntax panics. See [`JniGen`] for an example.
macro_rules! ptr_class {($($ty:tt)+)=>{$crate::PtrClassDecl::new($crate::__type(stringify!($($ty)+)))};}
#[macro_export]
/// Create a [`PackageDecl`] at the root prefix (`package!()`) or a relative
/// dotted package (`package!("example.api")`).
macro_rules! package {
    () => {
        $crate::PackageDecl::new("")
    };
    ($name:expr) => {
        $crate::PackageDecl::new($name)
    };
}
#[doc(hidden)]
pub fn __type(s: &str) -> syn::Type {
    syn::parse_str(s).expect("type syntax")
}

/// Generated Rust/JNI bindings and their Kotlin source files.
///
/// Start with [`Self::builder`]. Source capture, expansion declarations and package
/// placement are resolved before wrapper emission. Selectors keep all alternatives
/// in one native signature; multiple outputs use a Kotlin builder callback.
///
/// ```
/// use prebindgen_jni::{JniGen, package, ptr_class, fun};
/// let source = syn::parse_file(r#"
///     pub struct Counter { pub value: i64 }
///     pub fn new_counter(value: i64) -> Counter { Counter { value } }
/// "#)?;
/// let generated = JniGen::builder()
///     .items(source.items.into_iter().map(|item| (item, Default::default())))
///     .set_package_prefix("org.example")
///     .package(package!()
///         .class(ptr_class!(Counter))
///         .fun(fun!(new_counter)))
///     .build()?;
/// assert!(generated.kotlin.contains_key("org/example/Counter.kt"));
/// assert!(generated.kotlin["org/example/Functions.kt"].contains("newCounter"));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// In `build.rs`, replace `.items(...)` with `.source(flat_crate::PREBINDGEN_OUT_DIR)`,
/// then write both outputs. Add the source crate as both a build dependency and
/// a normal dependency. Include the Rust file in a binding crate depending on
/// `prebindgen-runtime`, `prebindgen-jni-runtime` and `konst` (used by captured
/// source feature guards). Compile all generated Kotlin
/// files together. Calls must go through those wrappers: their locks, consumed
/// state and ownership acknowledgement are part of the native safety contract.
pub struct JniGen {
    /// Formatted native Rust source, including conversion code and flat-model guards.
    pub rust: String,
    /// Kotlin sources keyed by package-relative paths such as `org/example/Counter.kt`.
    pub kotlin: BTreeMap<String, String>,
}
impl JniGen {
    /// Start configuring sources, declarations and output placement.
    pub fn builder() -> JniGenBuilder {
        JniGenBuilder::default()
    }
}
type Configure = Box<dyn FnOnce(&mut Bindings) -> Result<()>>;
/// Collect declarations before resolving a JNI/Kotlin binding.
///
/// Supply exactly one of [`Self::source`], [`Self::items`] or [`Self::flat`], plus
/// a nonempty package prefix. Type-wide expansions are order-independent. Custom
/// configuration callbacks run in registration order before declarations fill slots.
#[derive(Default)]
pub struct JniGenBuilder {
    configure: Vec<Configure>,
    flat: Option<Bindings>,
    source: Option<std::path::PathBuf>,
    items: Option<Vec<(syn::Item, prebindgen::SourceLocation)>>,
    expansions: Vec<ExpandDecl>,
    prefix: String,
    init: String,
    packages: Vec<PackageDecl>,
}
impl JniGenBuilder {
    /// Create an empty builder; equivalent to [`JniGen::builder`].
    pub fn new() -> Self {
        Self::default()
    }
    /// Adjust the selected flat model before declaration defaults are applied.
    ///
    /// `f` receives the model and may attach converters to it.
    ///
    /// Use this to attach converters with [`prebindgen_tools::bind_input`] and
    /// [`prebindgen_tools::bind_output`], using [`JniCodec`]. Callbacks run in
    /// registration order; errors stop generation. Source selection has already happened, so references
    /// introduced here must already exist in the model. Per-function declarations
    /// can explicitly override the converters assigned by these callbacks.
    ///
    /// ```
    /// use prebindgen_jni::{Bindings, JniGen, JniCodec, fun, package};
    /// use prebindgen_tools::{bind_input, bind_output, Input, Output};
    /// let source = syn::parse_file("pub fn echo(x: i64) -> i64 { x }")?;
    /// let flat = Bindings::parse(
    ///     source.items.into_iter().map(|item| (item, Default::default())),
    /// )?;
    /// let generated = JniGen::builder().flat(flat)
    ///     .set_package_prefix("example")
    ///     .configure(|flat| {
    ///         bind_input(flat, &JniCodec, "echo", "x", &Input::Leaf, &())?;
    ///         bind_output(flat, &JniCodec, "echo", &Output::Leaf, &())
    ///     })
    ///     .package(package!().fun(fun!(echo)))
    ///     .build()?;
    /// syn::parse_file(&generated.rust)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn configure(mut self, f: impl FnOnce(&mut Bindings) -> Result<()> + 'static) -> Self {
        self.configure.push(Box::new(f));
        self
    }
    /// Append captured items, each paired with its source location.
    ///
    /// `items` usually comes from `prebindgen::Source::items_all`.
    ///
    /// Calls accumulate items. Only declared functions, expansion helpers and required
    /// types are selected for validation. Automatic type discovery follows
    /// unqualified source names (see [`Flat::parse_selected`]); types referenced
    /// only by qualified paths can be selected with an unqualified `ptr_class!`
    /// declaration, or included through [`Self::flat`]. Mutually exclusive with
    /// source and flat input.
    pub fn items(
        mut self,
        items: impl IntoIterator<Item = (syn::Item, prebindgen::SourceLocation)>,
    ) -> Self {
        self.items.get_or_insert_with(Vec::new).extend(items);
        self
    }
    /// Read captures from a source crate's `PREBINDGEN_OUT_DIR`, passed as `path`.
    ///
    /// Uses the same selection as [`Self::items`]. A repeated call replaces the path;
    /// combining it with items or a flat model is rejected during build.
    pub fn source(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.source = Some(path.as_ref().to_owned());
        self
    }
    /// Register a type-wide input or output default, built with `expand_param!`
    /// or `expand_return!`.
    ///
    /// Per-function overrides replace defaults. Duplicate declarations for the same
    /// type and direction are rejected during build.
    pub fn expand(mut self, decl: impl Into<ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    /// Supply an already parsed model with input and output conversion slots.
    ///
    /// Bound functions still need a Kotlin package declaration. Mutually
    /// exclusive with source and item input.
    pub fn flat(mut self, flat: Bindings) -> Self {
        self.flat = Some(flat);
        self
    }
    /// Set the required nonempty root Kotlin package, such as `org.example.bindings`.
    ///
    /// All package declarations are relative to this prefix. The last setting wins.
    pub fn set_package_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
    /// Set Kotlin statements for the generated `JNINative` object's initialization.
    ///
    /// Typically loads the native library, e.g. `System.loadLibrary("bindings")`.
    /// The text is emitted verbatim and checked by the Kotlin compiler.
    pub fn set_jni_native_init(mut self, init: impl Into<String>) -> Self {
        self.init = init.into();
        self
    }
    /// Append a package's class and function declarations.
    ///
    /// Multiple declarations may share a package, but their exported names must be unique.
    pub fn package(mut self, package: PackageDecl) -> Self {
        self.packages.push(package);
        self
    }
    /// Resolve declarations and generate Rust and Kotlin texts without writing files.
    ///
    /// Rejects ambiguous source selection, invalid or duplicate names, unsupported
    /// recipes, undeclared handle types and signatures exceeding JVM parameter limits.
    /// Layout, Rust trait bounds and injected Kotlin syntax are checked by the
    /// respective compilers when consuming the generated output.
    pub fn build(self) -> Result<JniGen> {
        let references = prebindgen_tools::declarations::source_functions(
            &self.expansions,
            self.packages.iter().flat_map(|p| &p.functions),
        );
        let types: Vec<_> = self
            .packages
            .iter()
            .flat_map(|p| p.classes.iter().map(|c| c.ty.clone()))
            .collect();
        let mut flat = match (self.flat, self.source, self.items) {
            (Some(flat), None, None) => flat,
            (None, Some(path), None) => Flat::parse_selected(
                prebindgen::Source::new(path).items_all(),
                &references,
                &types,
            )?,
            (None, None, Some(items)) => Flat::parse_selected(items, &references, &types)?,
            _ => return Err("supply exactly one source, item stream or flat model".into()),
        };
        for configure in self.configure {
            configure(&mut flat)?;
        }
        let defaults = prebindgen_tools::Declarations::new(&flat, self.expansions)?;
        validate_package(&self.prefix)?;
        let mut classes = BTreeMap::new();
        let mut functions = Vec::new();
        let mut seen = BTreeSet::new();
        let mut bound = BTreeSet::new();
        for p in self.packages {
            let pkg = if p.name.is_empty() {
                self.prefix.clone()
            } else {
                format!("{}.{}", self.prefix, p.name)
            };
            validate_package(&pkg)?;
            for c in p.classes {
                let ty = flat.resolve_type(&c.ty)?;
                let name = match c.name {
                    Some(n) => n,
                    None => match &c.ty {
                        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                        _ => return Err("handle class needs a name".into()),
                    },
                };
                validate_name(&name)?;
                if name == "JNINative" || name == "Functions" || name == "NativeHandle" {
                    return Err("reserved generated class name".into());
                }
                let fqn = format!("{pkg}.{name}");
                if !seen.insert(fqn.clone()) {
                    return Err(format!("duplicate foreign name `{fqn}`"));
                }
                if classes
                    .insert(
                        type_key(&ty),
                        Class {
                            pkg: pkg.clone(),
                            name,
                            ty,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate handle class for source type".into());
                }
            }
            for f in p.functions {
                defaults.fill(&mut flat, &JniCodec, &f, &())?;
                let name = f.foreign.unwrap_or_else(|| camel(&f.name));
                validate_name(&name)?;
                if !seen.insert(format!("{pkg}.{name}")) {
                    return Err("duplicate foreign function name".into());
                }
                if !bound.insert(f.name.clone()) {
                    return Err("binding one source function twice is outside this subset".into());
                }
                functions.push((f.name, pkg.clone(), name));
            }
        }
        prebindgen_tools::validate_bindings(&flat)?;
        for (name, f) in &flat.functions {
            if f.ret.slot.is_some() && !bound.contains(name) {
                return Err(format!(
                    "configured function `{name}` has no Kotlin declaration"
                ));
            }
        }
        generate(flat, &self.prefix, &self.init, classes, functions)
    }
}
struct Class {
    pkg: String,
    name: String,
    ty: syn::Type,
}
impl Class {
    fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}
/// Leaf conversions used by [`JniGen`].
/// Use with [`prebindgen_tools::bind_input`] and [`prebindgen_tools::bind_output`]
/// to fill [`Bindings`] manually.
pub struct JniCodec;
impl Codec for JniCodec {
    type Extra = ();
    type Wire = JniWire;
    fn selector(&self, _: &()) -> JniWire {
        JniWire::Selector
    }
    fn consumes(&self, ty: &syn::Type, _: &()) -> bool {
        Value::of(ty).is_none()
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_tools::Condition,
    ) -> Result<prebindgen_tools::InputConversion<JniWire>> {
        let (decode, kind) = match Value::of(ty) {
            Some(Value::Scalar(p)) => (scalar::decode(p, &w), JniWire::Scalar(p)),
            Some(Value::Text) => (
                quote!(::prebindgen_jni_runtime::text(&mut env,&#w)?),
                JniWire::Text,
            ),
            Some(Value::Bytes) => (
                quote!(::prebindgen_jni_runtime::bytes(&mut env,&#w)?),
                JniWire::Bytes,
            ),
            None => {
                return prebindgen_tools::boxed::input(&storage(), ty, w, v, active, |b| {
                    JniWire::Handle(Box::new(b))
                })
            }
        };
        prebindgen_tools::value::input(ty, w, v, active, decode, kind)
    }
    fn output_leaf(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<prebindgen_tools::OutputConversion<JniWire>> {
        // Text and bytes stay Rust values until the wrapper delivers them.
        let (encode, kind): (fn(TokenStream) -> TokenStream, _) = match Value::of(ty) {
            Some(Value::Scalar(p)) => {
                return prebindgen_tools::value::output(
                    ty,
                    w,
                    v,
                    borrowed,
                    |v| scalar::encode(p, v),
                    JniWire::Scalar(p),
                )
            }
            Some(Value::Text) => (|v| v, JniWire::Text),
            Some(Value::Bytes) => (|v| v, JniWire::Bytes),
            None => {
                return prebindgen_tools::boxed::output(&storage(), ty, w, v, borrowed, |b| {
                    JniWire::Handle(Box::new(b))
                })
            }
        };
        prebindgen_tools::value::output(ty, w, v, borrowed, encode, kind)
    }
}
fn storage() -> prebindgen_tools::boxed::Storage {
    prebindgen_tools::boxed::Storage::Wrapped(syn::parse_quote!(
        ::prebindgen_jni_runtime::Allocation
    ))
}
/// Another name for the generated Rust and Kotlin sources in [`JniGen`].
pub type Generation = JniGen;
impl JniGen {
    /// Write native Rust source, returning the destination path.
    ///
    /// Relative paths resolve under Cargo's `OUT_DIR`; absolute paths are used
    /// directly. Creates parent directories and overwrites an existing file.
    /// Returns an error if `OUT_DIR` is needed but absent, or filesystem access fails.
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf> {
        let path = path.as_ref();
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?)
                .join(path)
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, &self.rust).map_err(|e| e.to_string())?;
        Ok(path)
    }
    /// Write all Kotlin files beneath `root`, creating package directories.
    ///
    /// Relative roots use the current directory, not `OUT_DIR`. Existing generated
    /// paths are overwritten; stale files are not removed. Returns the written paths
    /// or a filesystem error. Earlier writes remain if a later write fails.
    pub fn write_kotlin(&self, root: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        let mut written = Vec::new();
        for (path, text) in &self.kotlin {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&path, text).map_err(|e| e.to_string())?;
            written.push(path);
        }
        Ok(written)
    }
}
fn validate_name(s: &str) -> Result<()> {
    if s.is_empty()
        || s.chars().all(|c| c == '_')
        || [
            "as",
            "break",
            "class",
            "continue",
            "do",
            "else",
            "false",
            "for",
            "fun",
            "if",
            "in",
            "interface",
            "is",
            "null",
            "object",
            "package",
            "return",
            "super",
            "this",
            "throw",
            "true",
            "try",
            "typealias",
            "typeof",
            "val",
            "var",
            "when",
            "while",
        ]
        .contains(&s)
        || !s
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
    {
        return Err(format!("invalid Kotlin name `{s}`"));
    }
    Ok(())
}
fn validate_package(s: &str) -> Result<()> {
    for p in s.split('.') {
        validate_name(p)?;
    }
    Ok(())
}
fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}
/// JNI escaping is defined over UTF-16, including underscores in identifiers.
fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.encode_utf16() {
        match c {
            46 | 47 => out.push('_'),
            95 => out.push_str("_1"),
            59 => out.push_str("_2"),
            91 => out.push_str("_3"),
            c if (c as u8).is_ascii_alphanumeric() && c < 128 => out.push(char::from(c as u8)),
            c => out.push_str(&format!("_0{c:04x}")),
        }
    }
    out
}
fn symbol(pkg: &str, method: &str) -> syn::Ident {
    format_ident!(
        "Java_{}_{}",
        escape(&format!("{pkg}.JNINative")),
        escape(method)
    )
}
fn raw_rust(kind: &JniWire) -> TokenStream {
    match kind {
        JniWire::Scalar(p) => scalar::rust_type(*p),
        JniWire::Handle(_) => quote!(i64),
        JniWire::Selector => quote!(i32),
        JniWire::Text => quote!(::prebindgen_jni_runtime::jni::objects::JString<'local>),
        JniWire::Bytes => quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>),
    }
}
fn raw_kt(kind: &JniWire) -> &'static str {
    match kind {
        JniWire::Scalar(p) => scalar::raw_kotlin(*p),
        JniWire::Handle(_) => "Long",
        JniWire::Selector => "Int",
        JniWire::Text => "String?",
        JniWire::Bytes => "ByteArray?",
    }
}
fn class<'a>(classes: &'a BTreeMap<String, Class>, ty: &syn::Type) -> Result<&'a Class> {
    classes
        .get(&type_key(ty))
        .ok_or_else(|| format!("no handle class for `{}`", type_key(ty)))
}
fn generate(
    flat: Bindings,
    prefix: &str,
    init: &str,
    classes: BTreeMap<String, Class>,
    functions: Vec<(String, String, String)>,
) -> Result<JniGen> {
    let mut rust = TokenStream::new();
    let mut files = BTreeMap::new();
    let mut packages: BTreeMap<String, String> = BTreeMap::new();
    let mut native = format!("package {prefix}\n\ninternal object JNINative {{\n");
    if !init.is_empty() {
        native.push_str(&format!("    init {{ {init} }}\n"));
    }
    files.insert(
        format!("{}/NativeHandle.kt", prefix.replace('.', "/")),
        native_handle(prefix),
    );
    for (index, c) in classes.values().enumerate() {
        let method = format!("drop{index}");
        let sym = symbol(prefix, &method);
        let ty = &c.ty;
        rust.extend(quote!{#[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their object monitors held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64){if let Err(e)=::prebindgen_runtime::boundary(||unsafe {::prebindgen_jni_runtime::drop_pointer::<#ty>(id)}){::prebindgen_jni_runtime::fail(&mut env,&e);}}});
        native.push_str(&format!("    @JvmStatic external fun {method}(id: Long)\n"));
        let text=format!("package {}\n\npublic class {} internal constructor(raw: Long) : {prefix}.NativeHandle(raw) {{\n    override fun freeNative(raw: Long) {{ {prefix}.JNINative.{method}(raw) }}\n}}\n",c.pkg,c.name);
        files.insert(format!("{}/{}.kt", c.pkg.replace('.', "/"), c.name), text);
    }
    for (index, (source, pkg, name)) in functions.into_iter().enumerate() {
        let f = flat.function(&source)?;
        let Some(output) = &f.ret.slot else {
            return Err(format!("`{source}` has no output converter"));
        };
        let method = format!("call{index}");
        let sym = symbol(prefix, &method);
        let mut params = Vec::new();
        let mut raw_params = Vec::new();
        let mut kt_params = Vec::new();
        let mut kt_args = Vec::new();
        let mut body = TokenStream::new();
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut locks = Vec::new();
        let mut consumed = Vec::new();
        let mut args = Vec::new();
        for p in &f.params {
            let Some(input) = &p.ty.slot else {
                unreachable!()
            };
            for w in &input.wires {
                let n = &w.name;
                let rt = raw_rust(&w.kind);
                params.push(quote!(#n:#rt));
                raw_params.push(format!("{n}: {}", raw_kt(&w.kind)));
                let (kt, arg) = match &w.kind {
                    JniWire::Scalar(p) => scalar::input(*p, n),
                    JniWire::Handle(handle) => {
                        let condition = kotlin_condition(&w.active);
                        locks.push(format!(
                            "if ({condition}) requireNotNull({n}) {{ \"null handle\" }} else null"
                        ));
                        if handle.access == Access::Move {
                            consumed.push(format!("if ({condition}) {n}!!.markConsumed()"));
                        }
                        (
                            format!("{}?", class(&classes, &handle.ty)?.fqn()),
                            format!("{n}?.raw ?: 0L"),
                        )
                    }
                    _ => (raw_kt(&w.kind).into(), n.to_string()),
                };
                kt_params.push(format!("{n}: {kt}"));
                kt_args.push(arg);
            }
            validation.extend(input.validation.clone());
            acquire.extend(input.acquire.clone());
            resources.extend(input.resources.clone());
            body.extend(input.statements.clone());
            args.push(input.argument.clone());
        }
        let has_consumes = !consumed.is_empty();
        let ownership_param = if has_consumes {
            params.push(
                quote!(__ownership: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>),
            );
            raw_params.push("ownership: BooleanArray".into());
            kt_args.push("ownership".into());
            quote!(if env.get_array_length(&__ownership).map_err(|e|e.to_string())? != 1 {return Err("invalid ownership acknowledgement".into());}
                env.set_boolean_array_region(&__ownership, 0, &[1]).map_err(|e|e.to_string())?;)
        } else {
            quote!()
        };
        let slots: usize = usize::from(has_consumes)
            + f.params
                .iter()
                .map(|p| {
                    let Some(i) = &p.ty.slot else { unreachable!() };
                    i.wires
                        .iter()
                        .map(|w| match w.kind {
                            JniWire::Handle(_) => 2,
                            JniWire::Scalar(p) => scalar::parameter_slots(p),
                            _ => 1,
                        })
                        .sum::<usize>()
                })
                .sum::<usize>();
        if slots > 255 {
            return Err(format!(
                "`{source}` needs {slots} JVM parameter slots; the limit is 255"
            ));
        }
        let path = &f.path;
        let output_body = &output.statements;
        let commit = &output.commit;
        let aliases = prebindgen_tools::validate_resources(&resources);
        let count = output.wires.len();
        let mut encode = TokenStream::new();
        let mut delivered = Vec::new();
        let mut result_types = Vec::new();
        for (i, (w, v)) in output.wires.iter().zip(&output.values).enumerate() {
            let (expr, kt, decode) = match &w.kind {
                JniWire::Scalar(p) => scalar::output(*p, v, i),
                JniWire::Handle(handle) => {
                    let c = class(&classes, &handle.ty)?.fqn();
                    (
                        quote!(::prebindgen_jni_runtime::handle_object(&mut env,(#v) as u64)?),
                        c.clone(),
                        format!("{c}(result[{i}] as Long)"),
                    )
                }
                JniWire::Text => (
                    quote!(::prebindgen_jni_runtime::text_object(&mut env,&#v)?),
                    "String".into(),
                    format!("result[{i}] as String"),
                ),
                JniWire::Bytes => (
                    quote!(::prebindgen_jni_runtime::bytes_object(&mut env,&#v)?),
                    "ByteArray".into(),
                    format!("result[{i}] as ByteArray"),
                ),
                JniWire::Selector => return Err("selector output is unsupported".into()),
            };
            let i = i as i32;
            encode.extend(quote!{let object=#expr;env.set_object_array_element(&result,#i,&object).map_err(|e|e.to_string())?;env.delete_local_ref(object).map_err(|e|e.to_string())?;});
            delivered.push(decode);
            result_types.push(kt);
        }
        rust.extend(quote!{
   #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
   #[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their object monitors held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym<'local>(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,#(#params),*)->::prebindgen_jni_runtime::jni::sys::jobjectArray{
    match ::prebindgen_runtime::boundary(||{
     #validation #aliases #ownership_param #acquire #body let __result=#path(#(#args),*);#output_body
     let result=::prebindgen_jni_runtime::array(&mut env,#count)?;#encode
     #commit Ok(result.into_raw())
    }){Ok(result)=>result,Err(e)=>{::prebindgen_jni_runtime::fail(&mut env,&e);std::ptr::null_mut()}}
   }
  });
        native.push_str(&format!(
            "    @JvmStatic external fun {method}({}): Array<Any?>\n",
            raw_params.join(", ")
        ));
        let mut output_locals = String::new();
        let mut materialize = String::new();
        let mut cleanup = String::new();
        for (i, w) in output.wires.iter().enumerate() {
            if let JniWire::Handle(handle) = &w.kind {
                let c = class(&classes, &handle.ty)?.fqn();
                output_locals.push_str(&format!("        var output{i}: {c}? = null\n"));
                materialize.push_str(&format!("            output{i} = {}\n", delivered[i]));
                delivered[i] = format!("output{i}");
                let index = classes
                    .keys()
                    .position(|key| *key == type_key(&handle.ty))
                    .unwrap();
                cleanup.push_str(&format!("            try {{ if (output{i} != null) output{i}.close() else {prefix}.JNINative.drop{index}(result[{i}] as Long) }} catch (cleanup: Throwable) {{ failure.addSuppressed(cleanup) }}\n"));
            }
        }
        let (generic, ret, end) = match delivered.len() {
            0 => ("", "Unit".into(), "Unit".into()),
            1 => ("", result_types[0].clone(), delivered[0].clone()),
            _ => {
                kt_params.push(format!("build: ({}) -> R", result_types.join(", ")));
                (
                    "<R> ",
                    "R".into(),
                    format!("build({})", delivered.join(", ")),
                )
            }
        };
        let end = if cleanup.is_empty() {
            end
        } else {
            format!("{output_locals}        try {{\n{materialize}            {end}\n        }} catch (failure: Throwable) {{\n{cleanup}            throw failure\n        }}")
        };
        let call = format!("{prefix}.JNINative.{method}({})", kt_args.join(", "));
        let call = if has_consumes {
            format!("val ownership = BooleanArray(1)\n        try {{ {call} }} finally {{ if (ownership[0]) {{ {} }} }}", consumed.join("; "))
        } else {
            call
        };
        let native_call = if locks.is_empty() {
            format!("val result = {call}\n        {end}")
        } else {
            format!("val result = {prefix}.withHandleLocks(listOfNotNull({})) {{\n        {call}\n    }}\n{end}", locks.join(", "))
        };
        let text = packages.entry(pkg).or_default();
        text.push_str(&format!(
            "public fun {generic}`{name}`({}): {ret} = run {{\n    {native_call}\n}}\n\n",
            kt_params.join(", ")
        ));
    }
    native.push_str("}\n");
    files.insert(format!("{}/JNINative.kt", prefix.replace('.', "/")), native);
    for (pkg, body) in packages {
        let path = format!("{}/Functions.kt", pkg.replace('.', "/"));
        if files
            .insert(
                path,
                format!("@file:Suppress(\"UNUSED_VARIABLE\")\npackage {pkg}\n\n{body}"),
            )
            .is_some()
        {
            return Err("Kotlin file collision".into());
        }
    }
    let guards = &flat.guards;
    rust.extend(quote!(#(#guards)*));
    Ok(JniGen {
        rust: prebindgen_tools::rust_source(rust)?,
        kotlin: files,
    })
}

#[cfg(test)]
mod tests;

fn kotlin_condition(condition: &prebindgen_tools::Condition) -> String {
    if condition.0.is_empty() {
        return "true".into();
    }
    condition
        .0
        .iter()
        .map(|(tag, index)| format!("{tag} == {index}"))
        .collect::<Vec<_>>()
        .join(" && ")
}
fn native_handle(prefix: &str) -> String {
    include_str!("native_handle.kt").replace("PACKAGE", prefix)
}
