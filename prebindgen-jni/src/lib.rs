//! Generate Rust/JNI wrappers and Kotlin bindings from conversion slots.
//! Supported declarations include `package!`, `ptr_class!`, `data_class!`, `fun!`,
//! expansion declarations, retained `impl Fn` inputs, `.name()`, source selection,
//! package prefix and native initialization.
//! Declaration processing fills occurrence slots before wrappers are generated.
#![warn(missing_docs)]
use prebindgen_flat::{type_key, Conversion, Flat, OutputConversion, Result, WireKind};

pub use prebindgen_declarations::{callback, CallbackDecl, Projection, ProjectionMode};
mod metadata;
pub use metadata::{JniFunction, JniModule, JniSchema, JniType};
mod data;
mod enums;
pub use enums::{EnumClassDecl, JniEnum};
mod callback;
#[cfg(test)]
mod callback_tests;
mod optional;
mod sequence;
mod sequence_callback;
pub use callback::JniCallback;
pub use optional::JniNiche;
mod group;
mod scalar;
pub use data::{ClassDecl, DataClassDecl, DataGroup, JniField};
pub use group::{JniBranch, JniChain, JniGroup, JniOptional};
use prebindgen_conversions::boxed::BoxedWire;
use prebindgen_conversions::value::Value;
use prebindgen_flat::Primitive;
pub use sequence::JniSequence;
mod wire;
use prebindgen_conversions::Access;
use prebindgen_declarations::Codec;
pub use wire::{JniArgument, JniWire};
mod emission;
mod errors;
pub use emission::JniContext;
type InputPlan<W> =
    prebindgen_conversions::InputPlan<W, JniContext, crate::JniChain, crate::JniBranch>;
type OutputPlan<W> =
    prebindgen_conversions::OutputPlan<W, JniContext, crate::JniChain, crate::JniBranch>;
mod kotlin;
pub use kotlin::{
    JniInput, JniOutput, KotlinBinding, KotlinClass, KotlinContext, KotlinInput,
    KotlinInputConversion, KotlinOutput, KotlinOutputConversion,
};
/// Flat source model with JNI conversion data for wrapper emission.
///
/// Use [`JniCodec`] when binding conversions. Rust rejects a C codec because its
/// wire type belongs to a different adapter:
/// ```compile_fail
/// use prebindgen_jni::Bindings;
/// use prebindgen_declarations::{bind_input, Input};
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
pub type Bindings = Flat<JniSchema>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

pub use prebindgen_declarations::{
    expand_param, expand_return, fields, fun, ErrorDecl, ExpandDecl, ExpandParamDecl,
    ExpandReturnDecl, FieldsDecl, FunctionDecl, Input, InputRequest, Output, OutputRequest,
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
/// A Kotlin package containing handle/data classes and top-level functions.
///
/// Its name is relative to [`JniGenBuilder::set_package_prefix`].
#[derive(Clone, Default)]
pub struct PackageDecl {
    /// Relative dotted package name; empty means the root package prefix.
    pub name: String,
    /// Handle and data classes to emit in this package.
    pub classes: Vec<ClassDecl>,
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
    /// Add a handle or data class. Duplicate source types or names fail during build.
    pub fn class(mut self, class: impl Into<ClassDecl>) -> Self {
        self.classes.push(class.into());
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
/// placement are resolved before wrapper emission. Data classes map supported
/// record fields to Kotlin properties, including owning handle members in returns.
/// Handle classes own native allocations.
/// Selectors keep all alternatives
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
    inputs: Vec<prebindgen_declarations::Input<InputRequest>>,
    outputs: Vec<prebindgen_declarations::Output<OutputRequest>>,
    error_mappers: Vec<(syn::Type, String)>,
    prefix: String,
    init: String,
    packages: Vec<PackageDecl>,
    error_classes: Vec<(syn::Type, String)>,
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
    /// Use this to attach converters with [`prebindgen_declarations::bind_input`] and
    /// [`prebindgen_declarations::bind_output`], using [`JniCodec`]. Callbacks run in
    /// registration order; errors stop generation. Source selection and registration
    /// of module settings, class declarations, function placement and `error_class`
    /// policies have already happened. Callbacks run before class completion, error
    /// preparation and converter filling; their placement edits are revalidated.
    /// References introduced here must already exist in the selected source model.
    /// Per-function declarations can explicitly override converters assigned here.
    ///
    /// Class and function packages are fully qualified at registration. Changing
    /// `flat.metadata.package` changes native infrastructure and the fallback error
    /// handler package; it does not relocate already declared classes or functions.
    ///
    /// ```
    /// use prebindgen_jni::{Bindings, JniGen, JniCodec, fun, package};
    /// use prebindgen_declarations::{bind_input, bind_output, Input, Output};
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
    /// or `data_class!` declaration, or included through [`Self::flat`]. Mutually exclusive with
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
    /// Register an input/output default using the declaration syntax.
    /// The builder is consumed directly into its Input/Output tree.
    pub fn expand(self, decl: impl Into<prebindgen_declarations::ExpandDecl>) -> Self {
        use prebindgen_declarations::ExpandDecl;
        match decl.into() {
            ExpandDecl::Param(decl) => self.input(decl),
            ExpandDecl::Return(decl) => self.output(decl),
            ExpandDecl::Error(decl) => self.error_mapper(decl.ty, decl.mapper),
        }
    }
    /// Register a type-wide explicit input tree, or a convertible input builder.
    ///
    /// Per-function overrides replace defaults. Duplicate declarations for the same
    /// type and direction are rejected during build.
    pub fn input(mut self, tree: impl Into<prebindgen_declarations::Input<InputRequest>>) -> Self {
        self.inputs.push(tree.into());
        self
    }
    /// Register a type-wide direct output conversion tree.
    pub fn output(
        mut self,
        tree: impl Into<prebindgen_declarations::Output<OutputRequest>>,
    ) -> Self {
        self.outputs.push(tree.into());
        self
    }
    /// Select a default source error-message mapper for Result returns using `ty`.
    /// The helper takes exactly E or &E and returns String. A function's
    /// `on_error(...)` overrides this default. Duplicate resolved types are rejected.
    pub fn error_mapper(
        mut self,
        ty: syn::Type,
        mapper: impl Into<prebindgen_declarations::FunctionDecl>,
    ) -> Self {
        self.error_mappers.push((ty, mapper.into().source_name()));
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
    /// Supply an optional throwing default for the Kotlin domain-error callback.
    ///
    /// `class` is a fully qualified dotted JVM binary name; use `$` between
    /// nested classes (for example `example.Errors$Rejected`). The class must exist at
    /// Kotlin compile time, extend Throwable, and provide a public String constructor.
    /// Result-returning functions also require `error_mapper` or
    /// `FunctionDecl::on_error` to select their message mapper. Kotlin invokes this
    /// default after JNI and handle locks finish; callers can replace `onError`
    /// with a recovering callback. Without this setting `onError` is required.
    /// Binding failures use the separate `onBindingError` callback.
    /// Duplicate registrations for the same resolved source type are rejected.
    pub fn error_class(mut self, ty: syn::Type, class: impl Into<String>) -> Self {
        self.error_classes.push((ty, class.into()));
        self
    }
    /// Append a package's class and function declarations.
    ///
    /// Multiple declarations may share a package, but their exported names must be unique.
    pub fn package(mut self, package: PackageDecl) -> Self {
        self.packages.push(package);
        self
    }
    /// Resolve declarations and return a complete Flat binding model for emission.
    ///
    /// Rejects ambiguous source selection, invalid or duplicate names, unsupported
    /// recipes, undeclared handle types and signatures exceeding JVM parameter limits.
    /// Layout, Rust trait bounds and injected Kotlin syntax are checked by the
    /// respective compilers when consuming the generated output.
    pub fn prepare(mut self) -> Result<Bindings> {
        for package in &self.packages {
            let namespace = if package.name.is_empty() {
                self.prefix.clone()
            } else {
                format!("{}.{}", self.prefix, package.name)
            };
            for class in &package.classes {
                if let ClassDecl::Callback(declaration) = class {
                    let mut declaration = declaration.clone();
                    declaration.namespace = Some(namespace.clone());
                    self.inputs.push(declaration.into());
                }
            }
        }
        let references = prebindgen_declarations::declarations::source_functions(
            &self.inputs,
            &self.outputs,
            &self.error_mappers,
            self.packages.iter().flat_map(|p| &p.functions),
        );
        let types: Vec<_> = self
            .packages
            .iter()
            .flat_map(|p| p.classes.iter().map(|c| c.ty().clone()))
            .chain(self.error_classes.iter().map(|(ty, _)| ty.clone()))
            .chain(self.error_mappers.iter().map(|(ty, _)| ty.clone()))
            .collect();
        let (mut flat, mut callback_sources) = match (self.flat, self.source, self.items) {
            (Some(flat), None, None) => (flat, Vec::new()),
            (None, source, items) => {
                let items = match (source, items) {
                    (Some(path), None) => prebindgen::Source::new(path).items_all().collect(),
                    (None, Some(items)) => items,
                    _ => return Err("supply exactly one source, item stream or flat model".into()),
                };
                let callbacks = callback::source_parameters(&items);
                (Flat::parse_selected(items, &references, &types)?, callbacks)
            }
            _ => return Err("supply exactly one source, item stream or flat model".into()),
        };
        validate_package(&self.prefix)?;
        flat.metadata = JniModule {
            package: self.prefix,
            native_init: self.init,
            callbacks: Vec::new(),
        };
        let mut declarations = Vec::new();
        let mut class_types = BTreeSet::new();
        let mut bound = BTreeSet::new();
        let mut callback_types = vec![];
        for p in self.packages {
            let pkg = if p.name.is_empty() {
                flat.metadata.package.clone()
            } else {
                format!("{}.{}", flat.metadata.package, p.name)
            };
            validate_package(&pkg)?;
            for c in p.classes {
                if let ClassDecl::Callback(declaration) = c {
                    callback_types.push(declaration.ty);
                    continue;
                }
                let ty = flat.resolve_type(c.ty())?;
                let name = match c.name() {
                    Some(n) => n.clone(),
                    None => match c.ty() {
                        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                        _ => return Err("class needs a name".into()),
                    },
                };
                validate_class_name(&name)?;
                if !class_types.insert(type_key(&ty)) {
                    return Err("duplicate class for source type".into());
                }
                let class = Class {
                    pkg: pkg.clone(),
                    name,
                    data: matches!(c, ClassDecl::Data(_)),
                    enumeration: matches!(c, ClassDecl::Enum(_)),
                    has_handles: false,
                    variant_names: match c {
                        ClassDecl::Enum(c) => c.variant_names,
                        _ => Vec::new(),
                    },
                };
                let metadata = flat.type_metadata_mut(&ty)?;
                metadata.class = Some(std::rc::Rc::new(class));
                metadata.enumeration = None;
            }
            for f in p.functions {
                let name = f.foreign.clone().unwrap_or_else(|| camel(&f.name));
                validate_name(&name)?;
                if !bound.insert(f.name.clone()) {
                    return Err("binding one source function twice is outside this subset".into());
                }
                flat.function_mut(&f.name)?.metadata = Some(JniFunction {
                    package: pkg.clone(),
                    name,
                    order: declarations.len(),
                    error: None,
                });
                declarations.push(f);
            }
        }
        for (ty, class) in self.error_classes {
            let mut parts = class.split('$');
            validate_package(parts.next().unwrap())?;
            for nested in parts {
                validate_name(nested)?;
            }
            let ty = flat.resolve_type(&ty)?;
            if flat
                .type_metadata_mut(&ty)?
                .error_class
                .replace(class)
                .is_some()
            {
                return Err("duplicate exception class for source error type".into());
            }
        }
        let codec = data::DataCodec::default();
        for configure in self.configure {
            configure(&mut flat)?;
        }
        callback_sources.extend(
            flat.functions
                .values()
                .flat_map(|function| &function.params)
                .filter(|parameter| matches!(&parameter.ty.ty, syn::Type::ImplTrait(_)))
                .map(|parameter| parameter.ty.ty.clone()),
        );
        validate_package(&flat.metadata.package)?;
        for declaration in &declarations {
            if flat.function(&declaration.name)?.metadata.is_none() {
                return Err(format!(
                    "declared JNI function `{}` has no export metadata",
                    declaration.name
                ));
            }
        }
        let output_defaults = callback::declared_outputs(&flat, &self.outputs)?;
        let defaults = prebindgen_declarations::Declarations::new(
            &flat,
            self.inputs,
            self.outputs,
            self.error_mappers,
        )?;
        validate_declaration_names(&flat)?;
        data::prepare(&mut flat)?;
        errors::prepare(&mut flat, &codec, &defaults, &declarations)?;
        let codec = data::DataCodec {
            callback_outputs: Some(&output_defaults),
        };
        for ty in callback_types {
            let recipe = defaults.resolve_input(&flat, &codec, &ty, &Input::auto(), &())?;
            let Input::Callback(configuration) = recipe else {
                return Err("callback declaration requires a callback conversion".into());
            };
            callback::validate_source_occurrence(&flat, &configuration, &callback_sources)?;
            let plan = callback::input(
                &flat,
                &configuration,
                &output_defaults,
                &configuration.ty,
                format_ident!("declared_callback"),
                format_ident!("declared_callback_value"),
                &Default::default(),
            )?;
            flat.metadata.callbacks.push(plan.into());
        }
        let mut next_order = declarations.len();
        for declaration in declarations {
            defaults.fill(&mut flat, &codec, &declaration, &())?;
            if declaration.callback_return.is_some() {
                let variant_name = format!("{}_fold", declaration.name);
                sequence::callback_target(
                    &(flat.function(&declaration.name)?.output_conversion()?)
                        .output_outcomes()
                        .success,
                )?;
                let variant = defaults.callback_variant(
                    &mut flat,
                    &codec,
                    &declaration,
                    &variant_name,
                    &(),
                )?;
                let mut placement = flat
                    .function(&declaration.name)?
                    .metadata
                    .as_ref()
                    .ok_or("callback source has no JNI export metadata")?
                    .clone();
                placement.name = format!("{}Fold", placement.name);
                let foreign = format!("{}.{}", placement.package, placement.name);
                if foreign_name_exists(&flat, &foreign) {
                    return Err("callback return foreign name collision".into());
                }
                placement.order = next_order;
                next_order += 1;
                flat.function_mut(&variant_name)?.metadata = Some(placement);
                defaults.fill(&mut flat, &codec, &variant, &())?;
                if declaration.callback_return
                    == Some(prebindgen_declarations::CallbackReturn::Only)
                {
                    let original = flat.functions.get_mut(&declaration.name).unwrap();
                    original.metadata = None;
                    original.ret.slot = None;
                    for param in &mut original.params {
                        param.ty.slot = None;
                    }
                }
            }
        }
        prebindgen_flat::validate_bindings(&flat)?;
        for (name, f) in &flat.functions {
            if f.ret.slot.is_some() && f.metadata.is_none() {
                return Err(format!(
                    "configured function `{name}` has no Kotlin declaration"
                ));
            }
        }
        Ok(flat)
    }
    /// Prepare a complete Flat model, then emit its Rust/JNI and Kotlin bindings.
    pub fn build(self) -> Result<JniGen> {
        generate(&self.prepare()?)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Class {
    pkg: String,
    name: String,
    data: bool,
    enumeration: bool,
    variant_names: Vec<(String, String)>,
    has_handles: bool,
}
impl Class {
    fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}
/// Validate the declarations currently held by Flat, including configuration edits.
fn validate_declaration_names(flat: &Bindings) -> Result<()> {
    let mut names = BTreeSet::new();
    for ty in flat.types.values() {
        if let Some(class) = &ty.metadata.class {
            validate_package(&class.pkg)?;
            validate_name(&class.name)?;
            let name = class.fqn();
            if !names.insert(name.clone()) {
                return Err(format!("duplicate foreign name `{name}`"));
            }
        }
    }
    for function in flat.functions.values() {
        if let Some(m) = &function.metadata {
            validate_package(&m.package)?;
            validate_name(&m.name)?;
            if !names.insert(format!("{}.{}", m.package, m.name)) {
                return Err("duplicate foreign function name".into());
            }
        }
    }
    Ok(())
}
/// Check a prospective generated declaration against canonical class, function and handler names.
fn foreign_name_exists(flat: &Bindings, name: &str) -> bool {
    flat.types.values().any(|t| {
        t.metadata.class.as_ref().is_some_and(|c| c.fqn() == name)
            || t.metadata.errors.values().any(|e| e.handler == name)
    }) || flat.functions.values().any(|f| {
        f.metadata
            .as_ref()
            .is_some_and(|m| format!("{}.{}", m.package, m.name) == name)
    })
}
/// Conversions and slot glue used by [`JniGen`].
/// Prepared code explicitly references the emission context's JNI environment
/// and ownership acknowledgement parameter. Output delivery owns its result local.
/// The writer supplies the error boundary and matching Kotlin protocol.
/// Use with [`prebindgen_declarations::bind_input`] and [`prebindgen_declarations::bind_output`]
/// to fill [`Bindings`] manually.
pub struct JniCodec;
impl Codec for JniCodec {
    type Schema = JniSchema;
    type Niche = crate::optional::JniNiche;
    type Chain = JniChain;
    type Branch = JniBranch;
    fn group_metadata(&self) -> JniBranch {
        JniBranch::Group(JniGroup::Values)
    }

    fn named_output_group(&self, name: &str) -> Result<JniBranch> {
        validate_name(name)?;
        Ok(JniBranch::Group(JniGroup::Named(name.into())))
    }

    fn binding_failure_layout(
        &self,
        _: &(),
    ) -> Result<Option<prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>>> {
        Ok(Some(prebindgen_flat::WireNode::Leaf(vec![
            prebindgen_flat::Wire {
                name: syn::parse_quote!(message),
                kind: JniWire::text(),
            },
        ])))
    }
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
        controls: &[prebindgen_flat::Wire<JniWire>],
    ) -> Result<JniChain> {
        JniOptional::from_parts(presence, discriminator, niche, controls).map(JniChain::Optional)
    }
    fn alternatives_metadata(
        &self,
        selector: &prebindgen_flat::Wire<JniWire>,
    ) -> Result<JniBranch> {
        group::validate_selector(selector)?;
        Ok(JniBranch::Alternatives)
    }
    fn field_group(&self, name: syn::Ident) -> JniBranch {
        JniBranch::Group(JniGroup::Field(name))
    }
    fn fields_group(&self) -> JniBranch {
        JniBranch::Group(JniGroup::Fields(None))
    }
    type Extra = ();
    type Wire = JniWire;
    type Context = JniContext;
    fn finish_input(&self, plan: &mut InputPlan<JniWire>, _: &()) -> Result<()> {
        group::validate_layout(&plan.node)?;
        let env = prebindgen_conversions::template_parameter("environment");
        let ownership = prebindgen_conversions::template_parameter("acknowledgement");
        if plan.resources.iter().any(|r| r.access == Access::Move) {
            plan.acknowledgement = Some(quote!(
                if #env.get_array_length(&#ownership).map_err(|e|e.to_string())? != 1 {
                    return Err("invalid ownership acknowledgement".into());
                }
                #env.set_boolean_array_region(&#ownership, 0, &[1]).map_err(|e|e.to_string())?;
            ));
        }
        Ok(())
    }
    fn finish_output(&self, plan: &mut OutputPlan<JniWire>, _: &()) -> Result<()> {
        plan.fallible = true;
        group::validate_layout(&plan.outcomes.success)?;
        let env = prebindgen_conversions::template_parameter("environment");
        let count = crate::group::result_packet_fields(&plan.outcomes.success).len() + 1;
        let encode = encode_array(
            crate::group::result_packet(&plan.outcomes.success, &plan.values)?,
            &syn::parse_quote!(result),
            1,
        );
        let mark = prebindgen_conversions::template_parameter("mark_output");
        plan.commit
            .extend(prebindgen_conversions::template_parameter(
                "output_committed",
            ));
        plan.delivery = prebindgen_conversions::Delivery {
            statements: quote!(let result = ::prebindgen_jni_runtime::array(&mut #env, #count)?; #encode
                #mark),
            result: quote!(result.into_raw()),
        };
        Ok(())
    }
    fn sequence_input(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        active: &prebindgen_declarations::Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<JniWire, crate::JniChain>> {
        sequence::input(wire, length, index, element, active)
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        values: &[(syn::Ident, TokenStream)],
        delivery: prebindgen_declarations::SequenceDelivery,
    ) -> Result<prebindgen_declarations::SequenceOutput<JniWire, crate::JniChain>> {
        match delivery {
            prebindgen_declarations::SequenceDelivery::Value => {
                sequence::output(wire, length, index, element, values)
            }
            prebindgen_declarations::SequenceDelivery::Callback => {
                sequence_callback::transport(wire, element, values)
            }
        }
    }
    fn niche(&self, wire: &JniWire) -> Option<crate::optional::JniNiche> {
        wire.available_niche()
    }
    fn set_niche(&self, wire: &mut JniWire, niche: Option<JniNiche>) {
        wire.set_niche(niche)
            .expect("JNI codec supplied an incompatible niche");
    }
    fn is_present(
        &self,
        wire: &JniWire,
        niche: &Self::Niche,
        value: TokenStream,
    ) -> Result<TokenStream> {
        Ok(niche.present(wire, value))
    }
    fn remaining_niche(&self, wire: &JniWire, niche: &Self::Niche) -> Option<Self::Niche> {
        optional::remaining(wire, niche)
    }
    fn absent_value(
        &self,
        wire: &JniWire,
        niche: Option<&crate::optional::JniNiche>,
        input: bool,
    ) -> Result<TokenStream> {
        optional::absent(wire, niche, input)
    }
    fn selector(&self, _: &()) -> JniWire {
        JniWire::selector()
    }
    fn consumes(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
    ) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(_flat, inner, &());
        }
        Value::of(ty).is_none()
    }
    fn input_leaf(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_declarations::Condition,
    ) -> Result<InputPlan<JniWire>> {
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        let env = prebindgen_conversions::template_parameter("environment");
        let (decode, kind) = match Value::of(ty) {
            Some(Value::Scalar(p)) => (scalar::decode(p, &w), JniWire::scalar(p)),
            Some(Value::Text) => (
                quote!(::prebindgen_jni_runtime::text(&mut #env,&#w)?),
                JniWire::text(),
            ),
            Some(Value::Bytes) => (
                quote!(::prebindgen_jni_runtime::bytes(&mut #env,&#w)?),
                JniWire::bytes(),
            ),
            None => {
                return prebindgen_conversions::boxed::input(
                    &storage(),
                    ty,
                    w,
                    v,
                    &active.rust(),
                    |b| JniWire::handle(b, kotlin_condition(active)),
                )
            }
        };
        prebindgen_conversions::value::input(ty, w, v, &active.rust(), decode, kind)
    }
    fn output_leaf(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<JniWire>> {
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if let syn::Type::Reference(reference) = ty {
            if reference.mutability.is_some() {
                return Err("mutable-reference outputs are unsupported".into());
            }
            match type_key(&reference.elem).as_str() {
                "str" => {
                    return self.output_leaf(
                        _flat,
                        &syn::parse_quote!(String),
                        &(),
                        w,
                        quote!((#v).to_string()),
                        false,
                    )
                }
                "[u8]" => {
                    return self.output_leaf(
                        _flat,
                        &syn::parse_quote!(Vec<u8>),
                        &(),
                        w,
                        quote!((#v).to_vec()),
                        false,
                    )
                }
                _ => {}
            }
            let value = if borrowed { quote!(*#v) } else { v };
            return self.output_leaf(_flat, &reference.elem, &(), w, value, true);
        }
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        // Text and bytes stay optional Rust payloads until delivery. This keeps
        // a null reference distinct from a present empty string or byte array.
        let (encode, kind): (fn(TokenStream) -> TokenStream, _) = match Value::of(ty) {
            Some(Value::Scalar(p)) => {
                return prebindgen_conversions::value::output(
                    ty,
                    w,
                    v,
                    borrowed,
                    |v| scalar::encode(p, v),
                    JniWire::scalar(p),
                )
            }
            Some(Value::Text) => (|v| quote!(Some(#v)), JniWire::text()),
            Some(Value::Bytes) => (|v| quote!(Some(#v)), JniWire::bytes()),
            None => {
                return prebindgen_conversions::boxed::output(&storage(), ty, w, v, borrowed, |b| {
                    JniWire::handle(b, "true")
                })
            }
        };
        prebindgen_conversions::value::output(ty, w, v, borrowed, encode, kind)
    }

    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<prebindgen_declarations::NicheReservation<Self::Niche>> {
        crate::group::optional_niche(payload)
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &prebindgen_declarations::NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        crate::group::optional_presence(payload, reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        crate::group::absent_output_values(payload, reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        crate::group::retire_niches(payload, reservation)
    }
    fn validate_input(&self, plan: &prebindgen_declarations::CodecInput<Self>) -> Result<()> {
        sequence::validate_buffered(&plan.node)?;
        crate::group::validate_layout(&plan.node)
    }
    fn validate_output(&self, plan: &prebindgen_declarations::CodecOutput<Self>) -> Result<()> {
        sequence::callback_delivery(&plan.outcomes.success)?;
        for error in plan
            .outcomes
            .source_error
            .iter()
            .chain(plan.outcomes.binding_failure.iter())
        {
            sequence::validate_buffered(error)?;
        }
        for node in std::iter::once(&plan.outcomes.success)
            .chain(plan.outcomes.source_error.iter())
            .chain(plan.outcomes.binding_failure.iter())
        {
            crate::group::validate_layout(node)?;
        }
        crate::group::result_packet(&plan.outcomes.success, &plan.values)?;
        Ok(())
    }
}
pub(crate) fn encode_array(
    fields: Vec<(&prebindgen_flat::Wire<JniWire>, &TokenStream)>,
    array: &syn::Ident,
    offset: usize,
) -> TokenStream {
    let env = prebindgen_conversions::template_parameter("environment");
    let mut encode = TokenStream::new();
    for (i, (wire, value)) in fields.into_iter().enumerate() {
        let expr = wire.kind.output_object(value);
        let i = (i + offset) as i32;
        encode.extend(quote! {
            let object = #expr;
            #env.set_object_array_element(&#array, #i, &object).map_err(|e|e.to_string())?;
            #env.delete_local_ref(object).map_err(|e|e.to_string())?;
        });
    }

    encode
}
fn storage() -> prebindgen_conversions::boxed::Storage {
    prebindgen_conversions::boxed::Storage::Wrapped(syn::parse_quote!(
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
fn validate_class_name(name: &str) -> Result<()> {
    validate_name(name)?;
    if [
        "JNINative",
        "Functions",
        "NativeHandle",
        "JniErrorHandler",
        "HandleValidationFailure",
    ]
    .contains(&name)
    {
        return Err("reserved generated class name".into());
    }
    Ok(())
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

/// Emit Rust/JNI and Kotlin using only the prepared Flat model.
/// All project-dependent settings and declarations are read from its metadata.
pub fn generate(flat: &Bindings) -> Result<JniGen> {
    let prefix = &flat.metadata.package;
    validate_package(prefix)?;
    for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
        let metadata = f
            .metadata
            .as_ref()
            .ok_or_else(|| format!("bound JNI function `{}` has no export metadata", f.name))?;
        validate_package(&metadata.package)?;
        validate_name(&metadata.name)?;
    }
    let init = &flat.metadata.native_init;
    let classes = flat
        .types
        .values()
        .filter_map(|t| {
            t.metadata
                .class
                .as_ref()
                .filter(|c| !c.data && !c.enumeration)
                .map(|c| (type_key(&t.ty), (&t.ty, c)))
        })
        .collect::<BTreeMap<_, _>>();
    let mut functions = flat
        .functions
        .values()
        .filter_map(|f| f.metadata.as_ref().map(|m| (&f.name, m)))
        .collect::<Vec<_>>();
    functions.sort_by_key(|(_, m)| m.order);
    let errors = flat
        .functions
        .iter()
        .filter_map(|(name, f)| {
            f.metadata
                .as_ref()
                .and_then(|m| m.error.as_deref())
                .map(|e| (name.clone(), e))
        })
        .collect::<BTreeMap<_, _>>();
    let mut files = BTreeMap::new();
    for t in flat.types.values() {
        if let Some(class) = &t.metadata.class {
            if class.data {
                files.insert(
                    format!("{}/{}.kt", class.pkg.replace('.', "/"), class.name),
                    data::render_record(flat, class, &t.ty)?,
                );
            }
            if class.enumeration {
                files.insert(
                    format!("{}/{}.kt", class.pkg.replace('.', "/"), class.name),
                    t.metadata
                        .enumeration
                        .as_deref()
                        .ok_or("missing prepared enum declaration")?
                        .render(class)?,
                );
            }
        }
    }
    let mut rust = errors::emit(&errors)?;
    let mut packages: BTreeMap<String, String> = BTreeMap::new();
    errors::interfaces(prefix, &mut files)?;
    let mut native = format!("package {prefix}\n\ninternal object JNINative {{\n");
    if !init.is_empty() {
        native.push_str(&format!("    init {{ {init} }}\n"));
    }
    files.insert(
        format!("{}/NativeHandle.kt", prefix.replace('.', "/")),
        native_handle(prefix),
    );
    for (index, (ty, c)) in classes.values().enumerate() {
        let method = format!("drop{index}");
        let sym = symbol(prefix, &method);
        rust.extend(quote!{#[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64){if let Err(e)=::prebindgen_runtime::boundary(||unsafe {::prebindgen_jni_runtime::drop_pointer::<#ty>(id)}){::prebindgen_jni_runtime::fail(&mut env,&e);}}});
        native.push_str(&format!("    @JvmStatic external fun {method}(id: Long)\n"));
        let text=format!("package {}\n\npublic class {} internal constructor(raw: Long) : {prefix}.NativeHandle(raw) {{\n    public fun take(): {} = synchronized(this) {{ check(!isClosed) {{ \"closed or consumed handle\" }}; val owner = {}(raw); markConsumed(); owner }}\n    override fun freeNative(raw: Long) {{ {prefix}.JNINative.{method}(raw) }}\n}}\n",c.pkg,c.name,c.name,c.name);
        files.insert(format!("{}/{}.kt", c.pkg.replace('.', "/"), c.name), text);
    }
    let kotlin_classes = classes
        .iter()
        .enumerate()
        .map(|(index, (ty, (_, class)))| {
            (
                ty.clone(),
                KotlinClass {
                    name: class.fqn(),
                    drop_function: format!("{prefix}.JNINative.drop{index}"),
                },
            )
        })
        .collect();
    callback::files(flat, &kotlin_classes, &mut files)?;
    errors::handler_files(
        &errors,
        &KotlinContext {
            classes: &kotlin_classes,
            result: "result",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "errorOutput",
        },
        &mut files,
    )?;
    for (index, (source, placement)) in functions.into_iter().enumerate() {
        let source = source.to_string();
        let pkg = &placement.package;
        let name = &placement.name;
        let f = flat.function(&source)?;
        let callback =
            sequence::callback_delivery(&f.output_conversion()?.output_outcomes().success)?;
        let error = errors.get(&source);
        let outcomes = f.output_conversion()?.output_outcomes();
        let logical = &outcomes.success;
        for node in std::iter::once(logical)
            .chain(outcomes.source_error.iter())
            .chain(outcomes.binding_failure.iter())
        {
            group::validate_layout(node)?;
        }
        for error in outcomes
            .source_error
            .iter()
            .chain(outcomes.binding_failure.iter())
        {
            sequence::validate_buffered(error)?;
        }
        let has_source_error = outcomes.source_error.is_some();
        let has_binding_failure = outcomes.binding_failure.is_some();
        if has_source_error != error.is_some() {
            return Err("JNI source-error schema must match its configured error converter".into());
        }
        // Every JNI wrapper has a separate binding-error callback, including
        // functions without a source Result. Manual schemas must declare it.
        if !has_binding_failure {
            return Err("JNI native delivery requires a declared binding-failure schema".into());
        }

        let exception = if let Some((_, ty)) = prebindgen_flat::result_parts(&f.ret.ty) {
            flat.type_info(ty)
                .ok()
                .and_then(|t| t.metadata.error_class.as_deref())
        } else {
            None
        };
        let throws = error.map_or_else(String::new, |error| error.throws(exception));
        let env = format_ident!("jni_environment");
        let ownership = format_ident!("ownership_accepted");
        let context = JniContext {
            environment: env.clone(),
            acknowledgement: ownership.clone(),
            domain_output: format_ident!("domain_output"),
            is_domain_error: false,
            output_committed: quote!(),
        };
        let source = format_ident!("source_value");
        let method = format!("call{index}");
        let sym = symbol(prefix, &method);
        let mut params = Vec::new();
        let mut raw_params = Vec::new();
        let kotlin_context = KotlinContext {
            classes: &kotlin_classes,
            result: "result",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "output",
        };
        let mut kt_params = Vec::new();
        let mut kt_args = Vec::new();
        let mut locks = Vec::new();
        let mut lock_groups = Vec::new();
        let mut setup = String::new();
        let mut consumed = Vec::new();
        for w in crate::group::function_parameters(f)? {
            let n = &w.name;
            let rt = w.kind.rust_type();
            params.push(quote!(#n:#rt));
            raw_params.push(format!("{n}: {}", w.kind.kotlin_type()));
        }
        for input in f.input_conversions()? {
            let code = input.kotlin(&kotlin_context)?;
            kt_params.extend(code.parameters);
            kt_args.extend(code.arguments);
            locks.extend(code.locks);
            lock_groups.extend(code.lock_groups);
            for binding in code.setup {
                setup.push_str(&format!("val {} = {}\n", binding.name, binding.expression));
            }
            consumed.extend(code.consumed);
        }
        if kt_args.len() != crate::group::function_parameters(f)?.len() {
            return Err("Kotlin input argument/parameter count differs".into());
        }
        let has_consumes = !consumed.is_empty();
        let has_acknowledgement = f
            .input_conversions()?
            .iter()
            .map(|i| {
                i.code_fragment_slot(
                    prebindgen_conversions::InputFragment::Acknowledge { context: &context },
                    &syn::parse_quote!(()),
                )
                .map(|code| code.to_string() != quote!(()).to_string())
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .any(|present| present);
        if has_acknowledgement != has_consumes {
            return Err("JNI acknowledgement does not match slot ownership updates".into());
        }
        if has_consumes {
            params.push(
                quote!(#ownership: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>),
            );
            raw_params.push("ownership: BooleanArray".into());
            kt_args.push("ownership".into());
        }
        if callback {
            let (_, element, _) =
                sequence::return_sequence(&f.output_conversion()?.output_outcomes().success)
                    .ok_or("missing callback sequence")?;
            let fields = crate::group::row_fields(element)
                .enumerate()
                .map(|(i, w)| format!("v{i}: {}", w.kind.kotlin_type()))
                .collect::<Vec<_>>();
            native.push_str(&format!(
                "    internal fun interface SequenceSink{index} {{ fun run({}) }}\n",
                fields.join(", ")
            ));
            params.push(
                quote!(__pb_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>),
            );
            raw_params.push(format!("callback: SequenceSink{index}"));
            let names = (0..fields.len())
                .map(|i| format!("v{i}"))
                .collect::<Vec<_>>()
                .join(", ");
            setup.push_str(&format!("val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()\nval __pb_sink = {prefix}.JNINative.SequenceSink{index} {{ {names} -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>({names})); kotlin.Unit }}\n"));
            kt_args.push("__pb_sink".into());
            params.push(quote!(__pb_output_accepted: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>));
            raw_params.push("outputAccepted: BooleanArray".into());
            setup.push_str("val __pb_outputAccepted = BooleanArray(1)\n");
            kt_args.push("__pb_outputAccepted".into());
        }
        let slots: usize = usize::from(has_consumes)
            + 2 * usize::from(callback)
            + crate::group::function_parameters(f)?
                .map(|w| w.kind.parameter_slots())
                .sum::<usize>();
        if slots > 255 {
            return Err(format!(
                "`{source}` needs {slots} JVM parameter slots; the limit is 255"
            ));
        }
        let body = prebindgen_conversions::staged::emit_call(
            f,
            &context,
            &source,
            &syn::parse_quote!(::prebindgen_jni_runtime::jni::sys::jobjectArray),
            &crate::group::function_parameters(f)?
                .map(|w| w.name.clone())
                .collect::<Vec<_>>(),
        )?;
        let body = if callback {
            quote!(if #env.get_array_length(&__pb_output_accepted).map_err(|e|e.to_string())? != 1 {return Err("invalid sequence output acknowledgement".into());} if __pb_callback.is_null(){return Err("null sequence callback".into());} #body)
        } else {
            body
        };
        let fold = if callback {
            Some(sequence_callback::fold(
                &f.output_conversion()?.output_outcomes().success,
                &kotlin_context,
            )?)
        } else {
            None
        };
        if let Some(fold) = &fold {
            setup.push_str(&fold.discard);
        }
        let mut kotlin_output = match &fold {
            Some(fold) => fold.output.clone(),
            None => f.output_conversion()?.kotlin(&kotlin_context)?,
        };
        kotlin::remap_output(
            &mut kotlin_output,
            kotlin_context.result,
            kotlin_context.result,
            1,
            crate::group::result_packet_fields(&f.output_conversion()?.output_outcomes().success)
                .len(),
        );
        let build_parameters = kotlin_output.callback_type_parameters()?;
        let KotlinOutput {
            delivered,
            result_types,
            output_locals,
            materialize,
            cleanup,
            ..
        } = kotlin_output;
        if delivered.len() != result_types.len() {
            return Err("Kotlin output expression/type count differs".into());
        }
        rust.extend(quote!{
   #[allow(clippy::needless_borrow, clippy::borrow_deref_ref, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
   #[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym<'local>(mut #env: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,#(#params),*)->::prebindgen_jni_runtime::jni::sys::jobjectArray{
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(||{
     #body
    }){Ok(result)=>result,Err(e)=>{
        if let Some(result) = domain_output { result } else {
            match ::prebindgen_jni_runtime::binding_packet(&mut #env, &e) {
                Ok(result) => result.into_raw(),
                Err(message) => { ::prebindgen_jni_runtime::fail(&mut #env, &message); std::ptr::null_mut() }
            }
        }
    }}
   }
  });
        native.push_str(&format!(
            "    @JvmStatic external fun {method}({}): Array<Any?>\n",
            raw_params.join(", ")
        ));
        let (mut generic, ret, end) = match delivered.len() {
            0 => ("", "Unit".into(), "Unit".into()),
            1 => ("", result_types[0].clone(), delivered[0].clone()),
            _ => {
                kt_params.push(format!("build: ({build_parameters}) -> R"));
                (
                    "<R> ",
                    "R".into(),
                    format!("build({})", delivered.join(", ")),
                )
            }
        };
        if fold.is_some() {
            generic = "<A> ";
            kt_params.push("initial: A".into());
        }
        let failure = kotlin_context.failure;
        let result = kotlin_context.result;
        let end = if cleanup.is_empty() {
            end
        } else {
            format!("{output_locals}        try {{\n{materialize}            {end}\n        }} catch ({failure}: Throwable) {{\n{cleanup}            throw {failure}\n        }}")
        };
        let argument_locals = kt_args.iter().enumerate().map(|(i, arg)| format!("val nativeArgument{i} = try {{ {arg} }} catch (failure: IllegalArgumentException) {{ throw {prefix}.HandleValidationFailure(failure.message ?: \"invalid input\") }}")).collect::<Vec<_>>().join("; ");
        let argument_names = (0..kt_args.len())
            .map(|i| format!("nativeArgument{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let call =
            format!("run {{ {argument_locals}; {prefix}.JNINative.{method}({argument_names}) }}");
        let call = if has_consumes {
            format!("val ownership = BooleanArray(1)\n        try {{ {call} }} finally {{ if (ownership[0]) {{ {} }} }}", consumed.join("; "))
        } else {
            call
        };
        let binding_name = if error.is_some() {
            "onBindingError"
        } else {
            "onError"
        };
        let build = if delivered.len() > 1 {
            kt_params.pop()
        } else {
            None
        };
        kt_params.push(format!("{binding_name}: {prefix}.JniErrorHandler<{ret}> = {prefix}.JniErrorHandler {{ message -> throw IllegalArgumentException(message) }}"));
        if let Some(error) = error {
            kt_params.push(error.parameter(&ret, exception));
        }
        if let Some(build) = build {
            kt_params.push(build);
        }
        if let Some(fold) = &fold {
            kt_params.push(fold.parameter.clone());
        }
        let call = if locks.is_empty() && lock_groups.is_empty() {
            call
        } else if lock_groups.is_empty() && locks.len() <= 3 {
            use std::fmt::Write as _;
            // Evaluate and validate each expression before taking any monitor.
            let names = (0..locks.len())
                .map(|i| format!("handle{i}"))
                .collect::<Vec<_>>();
            let mut locals = String::new();
            for (name, value) in names.iter().zip(&locks) {
                write!(locals, "val {name} = try {{ {value} }} catch (failure: IllegalArgumentException) {{ throw {prefix}.HandleValidationFailure(failure.message ?: \"invalid handle\") }}; ").expect("writing to a String");
            }
            format!(
                "run {{ {locals}{prefix}.withHandleLocks({}) {{\n        {call}\n    }} }}",
                names.join(", ")
            )
        } else {
            let mut all_locks = if locks.is_empty() {
                "kotlin.collections.emptyList<Nothing>()".to_string()
            } else {
                format!("listOfNotNull({})", locks.join(", "))
            };
            for group in lock_groups {
                all_locks = format!("({all_locks} + ({group}))");
            }
            format!("run {{ val handles = try {{ {all_locks} }} catch (failure: IllegalArgumentException) {{ throw {prefix}.HandleValidationFailure(failure.message ?: \"invalid handle\") }}; {prefix}.withHandleLocks(handles) {{\n        {call}\n    }} }}")
        };
        let call = if callback {
            format!("try {{ {call} }} catch (failure: Throwable) {{ if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure }}")
        } else {
            call
        };
        // Catch local handle validation separately from the native invocation: pending JVM
        // exceptions and exceptions from user callbacks must retain their original identity.
        let call = format!("try {{ {call} }} catch (failure: {prefix}.HandleValidationFailure) {{ return@run {binding_name}.run(failure.message) }}");
        let domain = if let Some(error) = error {
            let delivery = errors::delivery(error, &kotlin_context)?;
            format!("true -> {{ {delivery} }}")
        } else {
            String::new()
        };
        let callback_failure = if callback {
            format!("if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException({result}[0] as String)); ")
        } else {
            String::new()
        };
        let native_call = format!("val {result} = {call}\n    when ({result}[0]) {{\n        null -> {{ {end} }}\n        {domain}\n        is String -> {{ {callback_failure}{binding_name}.run({result}[0] as String) }}\n        else -> error(\"invalid native result channel\")\n    }}");
        let text = packages.entry(pkg.clone()).or_default();
        text.push_str(&format!(
            "{throws}public fun {generic}`{name}`({}): {ret} = run {{\n    {setup}{native_call}\n}}\n\n",
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
                format!("@file:Suppress(\"UNUSED_VARIABLE\", \"UNCHECKED_CAST\", \"UNNECESSARY_NOT_NULL_ASSERTION\")\npackage {pkg}\n\n{body}"),
            )
            .is_some()
        {
            return Err("Kotlin file collision".into());
        }
    }
    let guards = &flat.guards;
    rust.extend(quote!(#(#guards)*));
    Ok(JniGen {
        rust: prebindgen_flat::rust_source(rust)?,
        kotlin: files,
    })
}

#[cfg(test)]
mod tests;

fn kotlin_condition(condition: &prebindgen_declarations::Condition) -> String {
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
