//! Build-script declarations for input alternatives and output views.
//! [`Declarations`] builds converters directly in boundary occurrence slots.

use crate::{option_inner, result_parts, sequence_parts};
use crate::{Codec, Compiler, Condition, Input, Output, Projection};
use prebindgen_flat::{type_key, Flat, Result};
use std::collections::BTreeMap;

/// An ordered list of input representations for one source type.
///
/// One variant becomes a direct recipe; multiple variants become a selector.
/// Constructor arguments recursively use the registered type defaults.
#[derive(Clone)]
pub struct ExpandParamDecl {
    /// Source type to which this declaration applies, resolved against the flat model.
    pub ty: syn::Type,
    variants: Vec<Option<FunctionDecl>>,
}
impl ExpandParamDecl {
    /// Start an input expansion with no variants. Add at least one before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            variants: Vec::new(),
        }
    }
    /// Append a source constructor as an input alternative.
    ///
    /// Only the constructor name is supported here; renaming and per-function
    /// overrides on this reference are rejected when resolving declarations.
    pub fn variant(mut self, f: FunctionDecl) -> Self {
        self.variants.push(Some(f));
        self
    }
    /// Append the type itself as a leaf, converted by the adapter's codec.
    ///
    /// This bypasses recursive expansion of the same type.
    pub fn variant_self(mut self) -> Self {
        self.variants.push(None);
        self
    }
}
/// An ordered list of simultaneous output views for one source type.
///
/// One view becomes a direct recipe; multiple views become [`Output::Combine`].
#[derive(Clone)]
pub struct ExpandReturnDecl {
    /// Source type whose result occurrences use this expansion.
    pub ty: syn::Type,
    fields: Vec<ReturnField>,
}
#[derive(Clone)]
enum ReturnField {
    Identity,
    Projected(Projection),
    IntoFields(FieldsDecl),
    Getter(Box<FunctionDecl>),
    Fields(FieldsDecl),
}
/// Project the named fields of a source getter's returned record.
///
/// The getter must accept a shared reference to the converted type. Fields use
/// their type defaults unless overridden with [`Self::field`].
#[derive(Clone)]
pub struct FieldsDecl {
    function: String,
    overrides: BTreeMap<String, ExpandReturnDecl>,
}
impl FieldsDecl {
    /// Select the source getter whose returned record will be projected.
    ///
    /// `function` is the getter's unqualified name.
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace the output expansion for one named field of the returned record.
    ///
    /// # Arguments
    ///
    /// * `name` — the field's name in the source record.
    /// * `decl` — how to deliver that field instead of its type's default.
    ///
    /// # Panics
    /// Panics if the same field is overridden twice. Unknown fields and mismatched
    /// source types are rejected during declaration resolution.
    pub fn field(mut self, name: impl AsRef<str>, decl: ExpandReturnDecl) -> Self {
        assert!(
            self.overrides.insert(name.as_ref().into(), decl).is_none(),
            "duplicate field override"
        );
        self
    }
}
impl ExpandReturnDecl {
    /// Append the fields of a getter's returned record as output views.
    pub fn fields(mut self, decl: FieldsDecl) -> Self {
        self.fields.push(ReturnField::Fields(decl));
        self
    }
    /// Consume the source accessor's receiver, then move its returned record fields.
    /// The accessor must take `T`, not `&T`. Per-field expansions still override
    /// type defaults. Nested data classes retain their borrowed default unless an
    /// explicit field expansion or type default selects owned projection.
    /// Observing sibling views run before this consuming view.
    pub fn fields_into(mut self, decl: FieldsDecl) -> Self {
        self.fields.push(ReturnField::IntoFields(decl));
        self
    }
    /// Deprecated alias for [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, decl: FieldsDecl) -> Self {
        self.fields_into(decl)
    }
    /// Append a leaf converted with explicit ownership and optional field policies.
    pub fn project_self(mut self, projection: Projection) -> Self {
        self.fields.push(ReturnField::Projected(projection));
        self
    }
    /// Append a shared-reference view; record fields inherit this policy.
    pub fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an owned view; record fields inherit this policy.
    pub fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Start an output expansion with no views. Add at least one before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            fields: Vec::new(),
        }
    }
    /// Append a shared-borrow getter, recursively expanding its owned result.
    ///
    /// The reference supports only the source name, without renaming or overrides.
    pub fn field(mut self, f: FunctionDecl) -> Self {
        self.fields.push(ReturnField::Getter(Box::new(f)));
        self
    }
    /// Append the value itself as a leaf, converted by the adapter's codec.
    ///
    /// Ownership follows the adapter default: JniGen data classes borrow, while
    /// owning leaf conversions consume after observing other views.
    pub fn field_self(mut self) -> Self {
        self.fields.push(ReturnField::Identity);
        self
    }
}
/// Explicit default error-message mapper for one source error type.
pub struct ErrorDecl {
    /// Source error type, resolved against the flat model.
    pub ty: syn::Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: FunctionDecl,
}
impl ErrorDecl {
    /// Declare a type-wide mapper; per-function `on_error` takes precedence.
    pub fn new(ty: syn::Type, mapper: FunctionDecl) -> Self {
        Self { ty, mapper }
    }
}
impl From<ErrorDecl> for ExpandDecl {
    fn from(value: ErrorDecl) -> Self {
        Self::Error(Box::new(value))
    }
}
/// A generator-wide conversion or error-mapping default, registered by source type.
pub enum ExpandDecl {
    /// Default error-message mapper for Result returns with this error type.
    Error(Box<ErrorDecl>),
    /// Default input alternatives.
    Param(ExpandParamDecl),
    /// Default simultaneous output views.
    Return(ExpandReturnDecl),
}
impl From<ExpandParamDecl> for ExpandDecl {
    fn from(d: ExpandParamDecl) -> Self {
        Self::Param(d)
    }
}
impl From<ExpandReturnDecl> for ExpandDecl {
    fn from(d: ExpandReturnDecl) -> Self {
        Self::Return(d)
    }
}
#[macro_export]
/// Create an [`ExpandParamDecl`] from Rust type syntax.
///
/// The tokens are parsed, not evaluated: the source type need not be in scope
/// in `build.rs`. Invalid type syntax panics. See [`Declarations`] for resolution rules.
macro_rules! expand_param {($($ty:tt)+)=>{$crate::ExpandParamDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
/// Create an [`ExpandReturnDecl`] from Rust type syntax.
///
/// Like [`expand_param!`], this does not require the source type to be in scope.
/// Invalid type syntax panics.
macro_rules! expand_return {($($ty:tt)+)=>{$crate::ExpandReturnDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}

/// Resolved type defaults used to build boundary converters.
///
/// Per-function overrides replace type defaults. Existing converters are preserved
/// unless explicitly overridden. By default unit has no wires and every other
/// type is a leaf, converted by the adapter's [`Codec`]. Recursive expansion
/// cycles are rejected. Recipes are local to [`Self::fill`]; only compiled conversion
/// data is attached to Flat. See [`Codec`] for a buildable example of constructor
/// alternatives, combined outputs and manual overrides.
pub struct Declarations {
    inputs: BTreeMap<String, ExpandParamDecl>,
    outputs: BTreeMap<String, ExpandReturnDecl>,
    errors: BTreeMap<String, FunctionDecl>,
}
impl Declarations {
    /// Index expansions by resolved source type.
    ///
    /// Returns an error for an unresolved type or duplicate declarations for the
    /// same type and direction. Recipes are resolved later by [`Self::fill`].
    ///
    /// # Arguments
    ///
    /// * `flat` — the model in which each declaration's type is resolved.
    /// * `decls` — type-wide defaults, at most one per type and direction.
    pub fn new<I: prebindgen_flat::Conversion, O: prebindgen_flat::Conversion>(
        flat: &Flat<I, O>,
        decls: Vec<ExpandDecl>,
    ) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        };
        for d in decls {
            match d {
                ExpandDecl::Error(d) => {
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    let f = &d.mapper;
                    if !f.params.is_empty()
                        || f.ret.is_some()
                        || f.foreign.is_some()
                        || f.error.is_some()
                    {
                        return Err("error mapper references only support their source name".into());
                    }
                    if out.errors.insert(key, d.mapper).is_some() {
                        return Err("duplicate error mapper for source type".into());
                    }
                }
                ExpandDecl::Param(d) => {
                    if crate::contains_result(&d.ty) {
                        return Err("Result expansions are unsupported; configure the success type and an error mapper".into());
                    }
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.inputs.insert(key, d).is_some() {
                        return Err("duplicate input expansion".into());
                    }
                }
                ExpandDecl::Return(d) => {
                    if crate::contains_result(&d.ty) {
                        return Err("Result expansions are unsupported; configure the success type and an error mapper".into());
                    }
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.outputs.insert(key, d).is_some() {
                        return Err("duplicate output expansion".into());
                    }
                }
            }
        }
        Ok(out)
    }
    /// Build one function's input and output converters from its declarations.
    ///
    /// Existing converters are retained unless explicitly overridden. `extra` is
    /// used for every converter built by this call; retained converters are untouched.
    /// For different data at individual occurrences, use [`crate::bind_input`] and
    /// [`crate::bind_output`] before filling defaults, or after applying overrides.
    /// No construction data is retained for later overrides.
    ///
    /// Resolves and compiles all required conversions before modifying any slot.
    /// An unknown parameter, recursive expansion, incompatible type or invalid
    /// constructor/getter leaves the entire function unchanged.
    ///
    /// # Arguments
    ///
    /// * `flat` — the model holding the function; its slots receive the converters.
    /// * `codec` — the adapter's leaf conversions.
    /// * `decl` — the function to export, with its per-parameter and result overrides.
    /// * `extra` — occurrence data passed to every codec call made by this fill.
    pub fn fill<C: Codec>(
        &self,
        flat: &mut prebindgen_flat::Flat<C::Input, C::Output>,
        codec: &C,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<()> {
        let source = flat.function(&decl.name)?;
        for name in decl.params.keys() {
            if !source.params.iter().any(|p| p.name == name.as_str()) {
                return Err(format!("`{}`: unknown parameter `{name}`", decl.name));
            }
        }
        let mut inputs = Vec::new();
        for (index, p) in source.params.iter().enumerate() {
            let explicit = decl.params.get(&p.name.to_string());
            inputs.push(if explicit.is_some() || p.ty.slot.is_none() {
                let recipe = self.input(flat, &p.ty.ty, explicit, &mut Vec::new())?;
                Some(Compiler::new(flat, codec, format!("p{index}")).input(
                    &p.ty.ty,
                    &recipe,
                    extra,
                    &Condition::default(),
                )?)
            } else {
                None
            });
        }
        if decl.error.is_some() && result_parts(&source.ret.ty).is_none() {
            return Err("on_error requires a top-level Result<T, E> return".into());
        }
        let mut output = if decl.ret.is_some() || decl.error.is_some() || source.ret.slot.is_none()
        {
            let recipe = if let Some((ok, error_ty)) = result_parts(&source.ret.ty) {
                let mapper = decl.error.as_deref().or_else(|| self.errors.get(&type_key(error_ty)))
                    .ok_or_else(|| format!("Result error `{}` requires an explicit error_mapper or on_error mapper", type_key(error_ty)))?;
                if !mapper.params.is_empty()
                    || mapper.ret.is_some()
                    || mapper.foreign.is_some()
                    || mapper.error.is_some()
                {
                    return Err("error mapper references only support their source name".into());
                }
                Output::Result {
                    ok: Box::new(self.output(flat, ok, decl.ret.as_ref(), &mut Vec::new())?),
                    error: mapper.name.clone(),
                }
            } else {
                self.output(flat, &source.ret.ty, decl.ret.as_ref(), &mut Vec::new())?
            };
            Some(Compiler::new(flat, codec, "ret".into()).output_root(
                &source.ret.ty,
                &recipe,
                extra,
                prebindgen_conversions::source_result_expr(),
                false,
            )?)
        } else {
            None
        };
        for input in inputs.iter_mut().flatten() {
            codec.finish_input(input, extra)?;
            input.layout.validate()?;
        }
        if let Some(output) = &mut output {
            codec.finish_output(output, extra)?;
            output.validate_value_count()?;
            output.layout.validate()?;
        }
        // Install slots only after every conversion has compiled successfully.
        let dest = flat.function_mut(&decl.name)?;
        for (p, input) in dest.params.iter_mut().zip(inputs) {
            if let Some(input) = input {
                p.ty.slot = Some(input.into());
            }
        }
        if let Some(output) = output {
            dest.ret.slot = Some(output.into());
        }
        Ok(())
    }
    fn input<I: prebindgen_flat::Conversion, O: prebindgen_flat::Conversion>(
        &self,
        flat: &Flat<I, O>,
        ty: &syn::Type,
        explicit: Option<&ExpandParamDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let key = type_key(ty);
        if let (Some(inner), Some(decl)) = (option_inner(ty), explicit) {
            if type_key(&flat.resolve_type(&decl.ty)?) != key {
                return Ok(Input::Optional(Box::new(
                    self.input(flat, inner, explicit, stack)?,
                )));
            }
        }
        if let (Some((element, _)), Some(decl)) = (sequence_parts(ty), explicit) {
            if type_key(&flat.resolve_type(&decl.ty)?) != key {
                return Ok(Input::Sequence(Box::new(
                    self.input(flat, element, explicit, stack)?,
                )));
            }
        }
        let d = explicit.or_else(|| self.inputs.get(&key));
        if let Some(d) = d {
            if type_key(&flat.resolve_type(&d.ty)?) != key {
                return Err("input expansion type does not match occurrence".into());
            }
            if stack.contains(&key) {
                return Err(format!("recursive input expansion `{key}`"));
            }
            stack.push(key);
            let mut arms = Vec::new();
            for variant in &d.variants {
                arms.push(match variant {
                    None => if let Some((element,_))=sequence_parts(ty) {
                        Input::Sequence(Box::new(self.input(flat,element,None,stack)?))
                    } else { Input::Leaf },
                    Some(f) => {
                        if !f.params.is_empty() || f.ret.is_some() || f.foreign.is_some() || f.error.is_some() {
                            return Err("constructor references only support their source name in this subset".into());
                        }
                        let func = flat.function(&f.name)?;
                        let args = func.params.iter()
                            .map(|p| self.input(flat, &p.ty.ty, None, stack))
                            .collect::<Result<Vec<_>>>()?;
                        Input::constructor(&f.name, args)
                    }
                });
            }
            stack.pop();
            return match arms.len() {
                0 => Err("empty input expansion".into()),
                1 => Ok(arms.remove(0)),
                _ => Ok(Input::selector(arms)),
            };
        }
        if let Some(inner) = option_inner(ty) {
            return Ok(Input::Optional(Box::new(
                self.input(flat, inner, None, stack)?,
            )));
        }
        if let Some((element, _)) = sequence_parts(ty) {
            return Ok(Input::Sequence(Box::new(
                self.input(flat, element, None, stack)?,
            )));
        }
        let key = match ty {
            syn::Type::Reference(r) => type_key(&r.elem),
            _ => key,
        };
        Ok(if key == "()" {
            Input::fields([])
        } else {
            Input::Leaf
        })
    }
    fn output<I: prebindgen_flat::Conversion, O: prebindgen_flat::Conversion>(
        &self,
        flat: &Flat<I, O>,
        ty: &syn::Type,
        explicit: Option<&ExpandReturnDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Output> {
        let key = type_key(ty);
        if let (Some(inner), Some(decl)) = (option_inner(ty), explicit) {
            if type_key(&flat.resolve_type(&decl.ty)?) != key {
                return Ok(Output::Optional(Box::new(
                    self.output(flat, inner, explicit, stack)?,
                )));
            }
        }
        if let (Some((element, _view)), Some(decl)) = (sequence_parts(ty), explicit) {
            if type_key(&flat.resolve_type(&decl.ty)?) != key {
                return Ok(Output::Sequence {
                    element: Box::new(self.output(flat, element, explicit, stack)?),
                    mode: None,
                });
            }
        }
        let d = explicit.or_else(|| self.outputs.get(&key));
        if let Some(d) = d {
            if type_key(&flat.resolve_type(&d.ty)?) != key {
                return Err("output expansion type does not match occurrence".into());
            }
            if stack.contains(&key) {
                return Err(format!("recursive output expansion `{key}`"));
            }
            stack.push(key);
            let mut parts = Vec::new();
            for field in &d.fields {
                parts.push(match field {
                    ReturnField::Identity => {
                        if let Some((element, _view)) = sequence_parts(ty) {
                            Output::Sequence {
                                element: Box::new(self.output(flat, element, None, stack)?),
                                mode: None,
                            }
                        } else {
                            Output::Leaf
                        }
                    }
                    ReturnField::Projected(p) => {
                        if let Some((element, _)) = sequence_parts(ty) {
                            if !p.fields.is_empty() {
                                return Err(
                                    "collection field projections belong on the element type"
                                        .into(),
                                );
                            }
                            Output::Sequence {
                                element: Box::new(self.output(flat, element, None, stack)?),
                                mode: Some(p.mode),
                            }
                        } else {
                            Output::Projected(p.clone())
                        }
                    }
                    ReturnField::Getter(f) => {
                        if !f.params.is_empty()
                            || f.ret.is_some()
                            || f.foreign.is_some()
                            || f.error.is_some()
                        {
                            return Err(
                                "getter references only support their source name in this subset"
                                    .into(),
                            );
                        }
                        let func = flat.function(&f.name)?;
                        Output::getter(&f.name, self.output(flat, &func.ret.ty, None, stack)?)
                    }
                    ReturnField::Fields(d) | ReturnField::IntoFields(d) => {
                        let f = flat.function(&d.function)?;
                        let record = flat.record(&f.ret.ty)?;
                        for name in d.overrides.keys() {
                            if !record.fields.iter().any(|p| p.name == name.as_str()) {
                                return Err(format!("unknown output field `{name}`"));
                            }
                        }
                        let children = record
                            .fields
                            .iter()
                            .map(|p| {
                                self.output(
                                    flat,
                                    &p.ty,
                                    d.overrides.get(&p.name.to_string()),
                                    stack,
                                )
                            })
                            .collect::<Result<Vec<_>>>()?;
                        if matches!(field, ReturnField::IntoFields(_)) {
                            Output::Into {
                                function: d.function.clone(),
                                output: Box::new(Output::OwnedFields(children)),
                            }
                        } else {
                            Output::getter(&d.function, Output::fields(children))
                        }
                    }
                });
            }
            stack.pop();
            return match parts.len() {
                0 => Err("empty output expansion is outside this subset".into()),
                1 => Ok(parts.remove(0)),
                _ => Ok(Output::combine(parts)),
            };
        }
        if let Some(inner) = option_inner(ty) {
            return Ok(Output::Optional(Box::new(
                self.output(flat, inner, None, stack)?,
            )));
        }
        if let Some((element, _view)) = sequence_parts(ty) {
            return Ok(Output::Sequence {
                element: Box::new(self.output(flat, element, None, stack)?),
                mode: None,
            });
        }
        Ok(if key == "()" {
            Output::fields([])
        } else {
            Output::Leaf
        })
    }
}

#[doc(hidden)]
pub fn parse_type(s: &str) -> syn::Type {
    syn::parse_str(s).expect("type syntax")
}
#[doc(hidden)]
pub fn parse_ident(s: &str) -> syn::Ident {
    syn::parse_str(s).expect("identifier syntax")
}
#[macro_export]
/// Reference a source function by its unqualified Rust identifier.
///
/// Returns a [`FunctionDecl`]; the function need not be in `build.rs` scope.
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new($crate::declarations::parse_ident(stringify!($name)))
    };
}
#[macro_export]
/// Reference a source getter returning a record whose fields should be exposed.
///
/// Returns a [`FieldsDecl`] for use with [`ExpandReturnDecl::fields`].
macro_rules! fields {
    ($name:ident) => {
        $crate::FieldsDecl::new($crate::declarations::parse_ident(stringify!($name)))
    };
}

/// A source callable to export, with optional destination name and occurrence overrides.
#[derive(Clone)]
pub struct FunctionDecl {
    /// Unqualified source function name in the flat namespace.
    pub name: String,
    /// Explicit destination name; supported by JNI, currently rejected by C.
    pub foreign: Option<String>,
    /// Input expansion overrides keyed by source parameter name.
    pub params: BTreeMap<String, ExpandParamDecl>,
    /// Result expansion override, replacing the type default.
    pub ret: Option<ExpandReturnDecl>,
    /// Source mapper for a top-level Result error; it takes E or &E and returns String.
    pub error: Option<Box<FunctionDecl>>,
}
impl FunctionDecl {
    /// Reference a source function without overriding names or conversions.
    ///
    /// `name` is the function's unqualified source name.
    pub fn new(name: syn::Ident) -> Self {
        Self {
            name: name.to_string(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
            error: None,
        }
    }
    /// Map the error of a top-level `Result<T, E>` return with a source function.
    ///
    /// The helper must take exactly `E` or `&E` and return `String`. Only its
    /// source name is accepted; it is selected for capture but is not exported.
    /// The success value uses normal return defaults and `expand_return` overrides.
    /// Replaces any previously selected mapper. Non-Result returns reject this option.
    pub fn on_error(mut self, mapper: FunctionDecl) -> Self {
        self.error = Some(Box::new(mapper));
        self
    }
    /// Set a literal destination name, subject to adapter validation.
    ///
    /// The JNI adapter uses it instead of its default camel-case name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.foreign = Some(name.into());
        self
    }
    /// Replace the expansion for one source parameter.
    ///
    /// # Arguments
    ///
    /// * `name` — the parameter's name, as written in the source.
    /// * `decl` — how to build that argument instead of its type's default.
    ///
    /// # Panics
    /// Panics if that parameter already has an override. Resolution checks that the
    /// parameter exists and the expansion type matches.
    pub fn expand_param(mut self, name: impl AsRef<str>, decl: ExpandParamDecl) -> Self {
        assert!(
            self.params.insert(name.as_ref().into(), decl).is_none(),
            "duplicate parameter override"
        );
        self
    }
    /// Replace the result expansion, including any previously assigned override.
    pub fn expand_return(mut self, decl: ExpandReturnDecl) -> Self {
        self.ret = Some(decl);
        self
    }
}

/// Collect source callable names referenced by bindings and nested expansions.
///
/// Used when selecting captured items before constructing a flat model. The
/// returned list can contain duplicates; collecting names does not resolve or
/// validate the recipes or export their helper functions.
///
/// # Arguments
///
/// * `expansions` — type-wide defaults; constructors, getters and error mappers are collected.
/// * `functions` — exported functions; their names and those referenced by their
///   overrides are collected.
pub fn source_functions<'a>(
    expansions: &[ExpandDecl],
    functions: impl IntoIterator<Item = &'a FunctionDecl>,
) -> Vec<String> {
    fn input(d: &ExpandParamDecl, names: &mut Vec<String>) {
        for f in d.variants.iter().flatten() {
            function(f, names);
        }
    }
    fn output(d: &ExpandReturnDecl, names: &mut Vec<String>) {
        for field in &d.fields {
            match field {
                ReturnField::Identity | ReturnField::Projected(_) => {}
                ReturnField::Getter(f) => function(f, names),
                ReturnField::Fields(d) | ReturnField::IntoFields(d) => {
                    names.push(d.function.clone());
                    for d in d.overrides.values() {
                        output(d, names);
                    }
                }
            }
        }
    }
    fn function(f: &FunctionDecl, names: &mut Vec<String>) {
        names.push(f.name.clone());
        if let Some(mapper) = &f.error {
            function(mapper, names);
        }
        for d in f.params.values() {
            input(d, names);
        }
        if let Some(d) = &f.ret {
            output(d, names);
        }
    }
    let mut names = Vec::new();
    for f in functions {
        function(f, &mut names);
    }
    for d in expansions {
        match d {
            ExpandDecl::Error(d) => function(&d.mapper, &mut names),
            ExpandDecl::Param(d) => input(d, &mut names),
            ExpandDecl::Return(d) => output(d, &mut names),
        }
    }
    names
}
