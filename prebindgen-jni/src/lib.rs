//! Generate Rust/JNI wrappers and Kotlin bindings from conversion slots.
//! Supported declarations include `package!`, `ptr_class!`, `data_class!`, `fun!`,
//! expansion declarations, `.name()`, source selection, package prefix and native initialization.
//! Declaration processing fills occurrence slots before wrappers are generated.
#![warn(missing_docs)]
use prebindgen_conversions::NativeConversionExt;
use prebindgen_conversions::{NativeConversion, NativeFunction};
use prebindgen_flat::{type_key, Flat, Result, WireKind};

pub use prebindgen_declarations::{Projection, ProjectionMode};
mod data;
mod enums;
pub use enums::{EnumClassDecl, JniEnum};
mod optional;
mod sequence;
mod sequence_callback;
pub use optional::JniNiche;
mod group;
mod scalar;
pub use data::{ClassDecl, DataClassDecl, DataGroup};
pub use group::{JniGroup, JniOptional};
use prebindgen_conversions::boxed::BoxedWire;
use prebindgen_conversions::value::Value;
use prebindgen_flat::Primitive;
mod wire;
use prebindgen_conversions::Access;
use prebindgen_declarations::Codec;
pub use wire::{JniArgument, JniWire};
mod emission;
mod errors;
pub use emission::JniContext;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, JniContext, JniGroup>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, JniContext, JniGroup>;
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
pub type Bindings = Flat<JniInput, JniOutput>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

pub use prebindgen_declarations::{
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
    expansions: Vec<ExpandDecl>,
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
    /// registration order; errors stop generation. Source selection has already happened, so references
    /// introduced here must already exist in the model. Per-function declarations
    /// can explicitly override the converters assigned by these callbacks.
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
    /// Register a type-wide input or output default, built with `expand_param!`
    /// or `expand_return!`.
    ///
    /// Per-function overrides replace defaults. Duplicate declarations for the same
    /// type and direction are rejected during build.
    pub fn expand(mut self, decl: impl Into<ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    /// Select a default source error-message mapper for Result returns using `ty`.
    /// The helper takes exactly E or &E and returns String. A function's
    /// `on_error(...)` overrides this default. Duplicate resolved types are rejected.
    pub fn error_mapper(
        mut self,
        ty: syn::Type,
        mapper: prebindgen_declarations::FunctionDecl,
    ) -> Self {
        self.expansions
            .push(prebindgen_declarations::ErrorDecl::new(ty, mapper).into());
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
    /// Resolve declarations and generate Rust and Kotlin texts without writing files.
    ///
    /// Rejects ambiguous source selection, invalid or duplicate names, unsupported
    /// recipes, undeclared handle types and signatures exceeding JVM parameter limits.
    /// Layout, Rust trait bounds and injected Kotlin syntax are checked by the
    /// respective compilers when consuming the generated output.
    pub fn build(self) -> Result<JniGen> {
        let references = prebindgen_declarations::declarations::source_functions(
            &self.expansions,
            self.packages.iter().flat_map(|p| &p.functions),
        );
        let types: Vec<_> = self
            .packages
            .iter()
            .flat_map(|p| p.classes.iter().map(|c| c.ty().clone()))
            .chain(self.error_classes.iter().map(|(ty, _)| ty.clone()))
            .chain(self.expansions.iter().filter_map(|d| match d {
                prebindgen_declarations::ExpandDecl::Error(d) => Some(d.ty.clone()),
                _ => None,
            }))
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
        let defaults = prebindgen_declarations::Declarations::new(&flat, self.expansions)?;
        validate_package(&self.prefix)?;
        let mut classes = BTreeMap::new();
        let mut functions = Vec::new();
        let mut declarations = Vec::new();
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
                let ty = flat.resolve_type(c.ty())?;
                let name = match c.name() {
                    Some(n) => n.clone(),
                    None => match c.ty() {
                        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                        _ => return Err("class needs a name".into()),
                    },
                };
                validate_name(&name)?;
                if [
                    "JNINative",
                    "Functions",
                    "NativeHandle",
                    "JniErrorHandler",
                    "HandleValidationFailure",
                ]
                .contains(&name.as_str())
                {
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
                            data: matches!(c, ClassDecl::Data(_)),
                            enumeration: matches!(c, ClassDecl::Enum(_)),
                            variant_names: match c {
                                ClassDecl::Enum(c) => c.variant_names,
                                _ => Vec::new(),
                            },
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate class for source type".into());
                }
            }
            for f in p.functions {
                declarations.push(f.clone());
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
        let mut codec = data::DataCodec::new(&flat, &classes)?;
        for (ty, class) in self.error_classes {
            let mut parts = class.split('$');
            validate_package(parts.next().unwrap())?;
            for nested in parts {
                validate_name(nested)?;
            }
            let ty = flat.resolve_type(&ty)?;
            if codec.error_classes.insert(type_key(&ty), class).is_some() {
                return Err("duplicate exception class for source error type".into());
            }
        }
        for enumeration in codec.enums.values() {
            flat.guards.extend(enumeration.mapping.guards());
        }
        let mut errors = errors::prepare(
            &mut flat,
            &mut codec,
            &defaults,
            &declarations,
            &classes,
            &self.prefix,
            &mut seen,
        )?;
        let mut callbacks = BTreeMap::new();
        for declaration in declarations {
            defaults.fill(&mut flat, &codec, &declaration, &())?;
            if declaration.callback_return.is_some() {
                let variant_name = format!("{}_fold", declaration.name);
                let (variant, wire) =
                    defaults.callback_variant(&mut flat, &declaration, &variant_name)?;
                codec.callback_wire = Some(wire);
                defaults.fill(&mut flat, &codec, &variant, &())?;
                codec.callback_wire = None;
                if flat
                    .function(&variant_name)?
                    .ret
                    .slot
                    .as_ref()
                    .unwrap()
                    .transport()
                    .len()
                    != 1
                {
                    return Err("callback_return cannot discard additional return values".into());
                }
                if let Some(error) = errors.get(&declaration.name).cloned() {
                    errors.insert(variant_name.clone(), error);
                }
                let (_, pkg, name) = functions
                    .iter()
                    .find(|(source, _, _)| source == &declaration.name)
                    .unwrap();
                let (pkg, name) = (pkg.clone(), format!("{name}Fold"));
                if !seen.insert(format!("{pkg}.{name}")) {
                    return Err("callback return foreign name collision".into());
                }
                functions.push((variant_name.clone(), pkg, name));
                bound.insert(variant_name.clone());
                callbacks.insert(
                    variant_name.clone(),
                    flat.function(&variant_name)?
                        .ret
                        .slot
                        .as_ref()
                        .unwrap()
                        .transport()
                        .clone(),
                );
                if declaration.callback_return
                    == Some(prebindgen_declarations::CallbackReturn::Only)
                {
                    functions.retain(|(source, _, _)| source != &declaration.name);
                    let original = flat.functions.get_mut(&declaration.name).unwrap();
                    original.ret.slot = None;
                    for param in &mut original.params {
                        param.ty.slot = None;
                    }
                }
            }
        }
        prebindgen_flat::validate_bindings(&flat)?;
        for (name, f) in &flat.functions {
            if f.ret.slot.is_some() && !bound.contains(name) {
                return Err(format!(
                    "configured function `{name}` has no Kotlin declaration"
                ));
            }
        }
        let mut data_files = BTreeMap::new();
        for class in classes.values().filter(|class| class.data) {
            data_files.insert(
                format!("{}/{}.kt", class.pkg.replace('.', "/"), class.name),
                codec.render(class)?,
            );
        }
        for class in classes.values().filter(|class| class.enumeration) {
            data_files.insert(
                format!("{}/{}.kt", class.pkg.replace('.', "/"), class.name),
                codec.enums[&type_key(&class.ty)].render(class)?,
            );
        }
        classes.retain(|_, class| !class.data && !class.enumeration);
        generate(
            flat,
            &self.prefix,
            &self.init,
            classes,
            functions,
            data_files,
            OutputBindings { errors, callbacks },
        )
    }
}
struct Class {
    pkg: String,
    name: String,
    ty: syn::Type,
    data: bool,
    enumeration: bool,
    variant_names: Vec<(String, String)>,
}
impl Class {
    fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}
/// Conversions and slot glue used by [`JniGen`].
/// Prepared code explicitly references the emission context's JNI environment
/// and ownership acknowledgement parameter. Output delivery owns its result local.
/// The writer supplies the error boundary and matching Kotlin protocol.
/// Use with [`prebindgen_declarations::bind_input`] and [`prebindgen_declarations::bind_output`]
/// to fill [`Bindings`] manually.
pub struct JniCodec;
impl Codec for JniCodec {
    type Niche = crate::optional::JniNiche;
    type Group = JniGroup;
    fn binding_failure_layout(
        &self,
        _: &(),
    ) -> Result<Option<prebindgen_flat::WireLayout<Self::Wire, Self::Group>>> {
        Ok(Some(
            vec![prebindgen_flat::Wire {
                name: syn::parse_quote!(message),
                kind: JniWire::text(),
            }]
            .into(),
        ))
    }
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
    ) -> JniGroup {
        JniGroup::Optional(JniOptional {
            presence,
            discriminator,
            niche,
        })
    }
    fn field_group(&self, name: syn::Ident) -> JniGroup {
        JniGroup::Field(name)
    }
    fn fields_group(&self) -> JniGroup {
        JniGroup::Fields(None)
    }
    type Extra = ();
    type Wire = JniWire;
    type Context = JniContext;
    type Input = JniInput;
    type Output = JniOutput;
    fn finish_input(&self, plan: &mut InputPlan<JniWire>, _: &()) -> Result<()> {
        group::validate_layout(&plan.transport)?;
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
        group::validate_layout(&plan.transport)?;
        let env = prebindgen_conversions::template_parameter("environment");
        let count = plan.transport.len() + 1;
        let encode = encode_array(&plan.transport, &plan.values, &syn::parse_quote!(result), 1);
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
        element: &prebindgen_conversions::TransportLayout<JniWire, JniGroup>,
        active: &prebindgen_declarations::Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<JniWire>> {
        sequence::input(wire, length, index, element, active)
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &prebindgen_conversions::TransportLayout<JniWire, JniGroup>,
        values: &[TokenStream],
    ) -> Result<prebindgen_declarations::SequenceOutput<JniWire>> {
        sequence::output(wire, length, index, element, values)
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
    fn consumes(&self, ty: &syn::Type, _: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(inner, &());
        }
        Value::of(ty).is_none()
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_declarations::Condition,
    ) -> Result<InputPlan<JniWire>> {
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{w}_present");
            let child = self.input_leaf(
                inner,
                &(),
                format_ident!("{w}_some"),
                format_ident!("{v}_some"),
                &active.arm(tag.clone(), 1),
            )?;
            return prebindgen_declarations::optional_input(self, &(), tag, v, active, child);
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
                        &syn::parse_quote!(String),
                        &(),
                        w,
                        quote!((#v).to_string()),
                        false,
                    )
                }
                "[u8]" => {
                    return self.output_leaf(
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
            return self.output_leaf(&reference.elem, &(), w, value, true);
        }
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{w}_present");
            let payload = format_ident!("{w}_payload");
            let child = self.output_leaf(
                inner,
                &(),
                format_ident!("{w}_some"),
                quote!(#payload),
                borrowed,
            )?;
            return prebindgen_declarations::optional_output(
                self,
                &(),
                tag,
                payload,
                v,
                borrowed,
                child,
            );
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
}
pub(crate) fn encode_array(
    layout: &prebindgen_conversions::TransportLayout<JniWire, JniGroup>,
    values: &[TokenStream],
    array: &syn::Ident,
    offset: usize,
) -> TokenStream {
    let env = prebindgen_conversions::template_parameter("environment");
    let mut encode = TokenStream::new();
    for (i, (wire, value)) in layout.iter().zip(values).enumerate() {
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

struct OutputBindings {
    errors: BTreeMap<String, errors::ErrorBinding>,
    callbacks: BTreeMap<String, prebindgen_conversions::TransportLayout<JniWire, JniGroup>>,
}

fn generate<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: Flat<I, O>,
    prefix: &str,
    init: &str,
    classes: BTreeMap<String, Class>,
    functions: Vec<(String, String, String)>,
    mut files: BTreeMap<String, String>,
    outputs: OutputBindings,
) -> Result<JniGen> {
    let OutputBindings { errors, callbacks } = outputs;
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
    for (index, c) in classes.values().enumerate() {
        let method = format!("drop{index}");
        let sym = symbol(prefix, &method);
        let ty = &c.ty;
        rust.extend(quote!{#[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64){if let Err(e)=::prebindgen_runtime::boundary(||unsafe {::prebindgen_jni_runtime::drop_pointer::<#ty>(id)}){::prebindgen_jni_runtime::fail(&mut env,&e);}}});
        native.push_str(&format!("    @JvmStatic external fun {method}(id: Long)\n"));
        let text=format!("package {}\n\npublic class {} internal constructor(raw: Long) : {prefix}.NativeHandle(raw) {{\n    public fun take(): {} = lock.run {{ lock(); try {{ check(!isClosed) {{ \"closed or consumed handle\" }}; val owner = {}(raw); markConsumed(); owner }} finally {{ unlock() }} }}\n    override fun freeNative(raw: Long) {{ {prefix}.JNINative.{method}(raw) }}\n}}\n",c.pkg,c.name,c.name,c.name);
        files.insert(format!("{}/{}.kt", c.pkg.replace('.', "/"), c.name), text);
    }
    let kotlin_classes = classes
        .iter()
        .enumerate()
        .map(|(index, (ty, class))| {
            (
                ty.clone(),
                KotlinClass {
                    name: class.fqn(),
                    drop_function: format!("{prefix}.JNINative.drop{index}"),
                },
            )
        })
        .collect();
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
    for (index, (source, pkg, name)) in functions.into_iter().enumerate() {
        let f = flat.function(&source)?;
        let callback_layout = callbacks.get(&source);
        let callback = callback_layout.is_some();
        let error = errors.get(&source);
        let logical = f.output_conversion()?.layout();
        logical.validate()?;
        // Every JNI wrapper has a separate binding-error callback, including
        // functions without a source Result. Manual schemas must declare it.
        if logical
            .outcome(prebindgen_flat::OutcomeKind::SourceError)
            .is_some()
            != error.is_some()
        {
            return Err("JNI source-error schema must match its configured error converter".into());
        }
        if logical
            .outcome(prebindgen_flat::OutcomeKind::BindingFailure)
            .is_none()
        {
            return Err("JNI native delivery requires a declared binding-failure schema".into());
        }

        let throws = error.map_or_else(String::new, |error| error.throws());
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
        for w in f.native_inputs()? {
            let n = &w.name;
            let rt = w.kind.rust_type();
            params.push(quote!(#n:#rt));
            raw_params.push(format!("{n}: {}", w.kind.kotlin_type()));
        }
        for input in f.input_conversions()? {
            let code = input.kotlin(&kotlin_context)?;
            if code.arguments.len() != input.native_wires().len() {
                return Err("Kotlin input argument/wire count differs".into());
            }
            kt_params.extend(code.parameters);
            kt_args.extend(code.arguments);
            locks.extend(code.locks);
            lock_groups.extend(code.lock_groups);
            for binding in code.setup {
                setup.push_str(&format!("val {} = {}\n", binding.name, binding.expression));
            }
            consumed.extend(code.consumed);
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
            let (_, element) =
                prebindgen_declarations::sequence_return(f.output_conversion()?.transport())
                    .ok_or("missing callback sequence")?;
            let fields = element
                .iter()
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
            + f.native_inputs()?
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
        )?;
        let body = if callback {
            quote!(if #env.get_array_length(&__pb_output_accepted).map_err(|e|e.to_string())? != 1 {return Err("invalid sequence output acknowledgement".into());} if __pb_callback.is_null(){return Err("null sequence callback".into());} #body)
        } else {
            body
        };
        let fold = if callback {
            Some(sequence_callback::fold(
                callback_layout.unwrap(),
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
            f.output_conversion()?.native_wires().len(),
        );
        let KotlinOutput {
            delivered,
            result_types,
            output_locals,
            materialize,
            cleanup,
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
                kt_params.push(format!("build: ({}) -> R", result_types.join(", ")));
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
            kt_params.push(error.parameter(&ret));
        }
        if let Some(build) = build {
            kt_params.push(build);
        }
        if let Some(fold) = &fold {
            kt_params.push(fold.parameter.clone());
        }
        let call = if locks.is_empty() && lock_groups.is_empty() {
            call
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
        let text = packages.entry(pkg).or_default();
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
