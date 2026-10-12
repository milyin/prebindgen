//! The retained JniGen expansion vocabulary translates into tools recipes.

use crate::{Input, Output, Recipe, Slot};
use prebindgen_flat::{type_key, Flat, Primitive, Result};
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
    /// Append the type itself: a scalar recipe for primitives, otherwise a foreign recipe.
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
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace the output expansion for one named field of the returned record.
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
    /// Append the value itself: a scalar recipe for primitives, otherwise a foreign recipe.
    ///
    /// For an owning adapter this consumes the value, after observing other views.
    pub fn field_self(mut self) -> Self {
        self.fields.push(ReturnField::Identity);
        self
    }
}
/// A generator-wide input or output expansion, registered by source type.
pub enum ExpandDecl {
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
/// in `build.rs`. Invalid type syntax panics. See [`Declarations`] for an example.
macro_rules! expand_param {($($ty:tt)+)=>{$crate::ExpandParamDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
/// Create an [`ExpandReturnDecl`] from Rust type syntax.
///
/// Like [`expand_param!`], this does not require the source type to be in scope.
/// Invalid type syntax panics.
macro_rules! expand_return {($($ty:tt)+)=>{$crate::ExpandReturnDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}

/// Resolved type defaults used to fill boundary occurrence recipes.
///
/// Per-function overrides replace type defaults. Existing nonempty recipes are
/// preserved unless explicitly overridden; occurrence metadata is always preserved.
/// Defaults cover primitives, text, bytes and unit, with other types delegated to
/// the adapter as foreign values. Recursive expansion cycles are rejected.
///
/// ```
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::{
///     expand_param, expand_return, fun, Declarations, Input, Output, Recipe, Slot,
/// };
///
/// let source = syn::parse_file(r#"
///     pub type Key = String;
///     pub fn key_from_text(s: String) -> Key { s }
///     pub fn key_text(k: &Key) -> String { k.clone() }
///     pub fn round_trip(k: Key) -> Key { k }
/// "#)?;
/// let mut flat = Flat::<Slot<()>>::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// let defaults = Declarations::new(&flat, vec![
///     expand_param!(Key).variant(fun!(key_from_text)).variant_self().into(),
///     expand_return!(Key).field(fun!(key_text)).field_self().into(),
/// ])?;
/// defaults.fill(&mut flat, &fun!(round_trip))?;
/// let function = flat.function("round_trip")?;
/// assert!(matches!(&function.params[0].ty.slot.recipe,
///     Recipe::Input(Input::Selector(arms)) if arms.len() == 2));
/// assert!(matches!(&function.ret.slot.recipe,
///     Recipe::Output(Output::Combine(parts)) if parts.len() == 2));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Declarations {
    inputs: BTreeMap<String, ExpandParamDecl>,
    outputs: BTreeMap<String, ExpandReturnDecl>,
}
impl Declarations {
    /// Index expansions by resolved source type.
    ///
    /// Returns an error for an unresolved type or duplicate declarations for the
    /// same type and direction. Recipes are resolved later by [`Self::fill`].
    pub fn new<S>(flat: &Flat<S>, decls: Vec<ExpandDecl>) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        };
        for d in decls {
            match d {
                ExpandDecl::Param(d) => {
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.inputs.insert(key, d).is_some() {
                        return Err("duplicate input expansion".into());
                    }
                }
                ExpandDecl::Return(d) => {
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.outputs.insert(key, d).is_some() {
                        return Err("duplicate output expansion".into());
                    }
                }
            }
        }
        Ok(out)
    }
    /// Fill one function's argument and result recipes, retaining each slot's `extra`.
    ///
    /// All recipes are resolved before modifying the function. Returns an error for
    /// unknown functions/parameters, type mismatches, recursive or empty expansions,
    /// or unsupported constructor/getter options. Full recipe/signature compatibility
    /// is checked by [`crate::prepare`].
    pub fn fill<E>(&self, flat: &mut Flat<Slot<E>>, decl: &FunctionDecl) -> Result<()> {
        let source = flat.function(&decl.name)?;
        for name in decl.params.keys() {
            if !source.params.iter().any(|p| p.name == name.as_str()) {
                return Err(format!("`{}`: unknown parameter `{name}`", decl.name));
            }
        }
        let mut inputs = Vec::new();
        for p in &source.params {
            let explicit = decl.params.get(&p.name.to_string());
            inputs.push(
                if explicit.is_some() || matches!(p.ty.slot.recipe, Recipe::Empty) {
                    Some(self.input(flat, &p.ty.ty, explicit, &mut Vec::new())?)
                } else {
                    None
                },
            );
        }
        let output = if decl.ret.is_some() || matches!(source.ret.slot.recipe, Recipe::Empty) {
            Some(self.output(flat, &source.ret.ty, decl.ret.as_ref(), &mut Vec::new())?)
        } else {
            None
        };
        let dest = flat.function_mut(&decl.name)?;
        for (p, input) in dest.params.iter_mut().zip(inputs) {
            if let Some(input) = input {
                p.ty.slot.recipe = Recipe::Input(input);
            }
        }
        if let Some(output) = output {
            dest.ret.slot.recipe = Recipe::Output(output);
        }
        Ok(())
    }
    fn input<S>(
        &self,
        flat: &Flat<S>,
        ty: &syn::Type,
        explicit: Option<&ExpandParamDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let key = type_key(ty);
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
                    None if Primitive::from_type(ty).is_some() => Input::Scalar,
                    None => Input::Foreign,
                    Some(f) => {
                        if !f.params.is_empty() || f.ret.is_some() || f.foreign.is_some() {
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
        if Primitive::from_type(ty).is_some() {
            return Ok(Input::Scalar);
        }
        let key = match ty {
            syn::Type::Reference(r) => type_key(&r.elem),
            _ => key,
        };
        Ok(match key.as_str() {
            "String" | "str" => Input::Text,
            "Vec < u8 >" | "[u8]" => Input::Bytes,
            "()" => Input::fields([]),
            _ => Input::Foreign,
        })
    }
    fn output<S>(
        &self,
        flat: &Flat<S>,
        ty: &syn::Type,
        explicit: Option<&ExpandReturnDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Output> {
        let key = type_key(ty);
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
                    ReturnField::Identity if Primitive::from_type(ty).is_some() => Output::Scalar,
                    ReturnField::Identity => Output::Foreign,
                    ReturnField::Getter(f) => {
                        if !f.params.is_empty() || f.ret.is_some() || f.foreign.is_some() {
                            return Err(
                                "getter references only support their source name in this subset"
                                    .into(),
                            );
                        }
                        let func = flat.function(&f.name)?;
                        Output::getter(&f.name, self.output(flat, &func.ret.ty, None, stack)?)
                    }
                    ReturnField::Fields(d) => {
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
                                    &p.ty.ty,
                                    d.overrides.get(&p.name.to_string()),
                                    stack,
                                )
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Output::getter(&d.function, Output::fields(children))
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
        if Primitive::from_type(ty).is_some() {
            return Ok(Output::Scalar);
        }
        Ok(match key.as_str() {
            "String" => Output::Text,
            "Vec < u8 >" => Output::Bytes,
            "()" => Output::fields([]),
            _ => Output::Foreign,
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
}
impl FunctionDecl {
    /// Reference a source function without overriding names or conversions.
    pub fn new(name: syn::Ident) -> Self {
        Self {
            name: name.to_string(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
        }
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
                ReturnField::Identity => {}
                ReturnField::Getter(f) => function(f, names),
                ReturnField::Fields(d) => {
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
            ExpandDecl::Param(d) => input(d, &mut names),
            ExpandDecl::Return(d) => output(d, &mut names),
        }
    }
    names
}
