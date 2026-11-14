//! Generic function binding declarations. Trees belong to the adapter.
use std::collections::BTreeMap;

#[macro_export]
/// Reference a source function and configure its exported binding.
macro_rules! fun {
    ($name:ident) => {
        $crate::DeclFunction::<_, _>::new(
            $crate::ident(stringify!($name)).expect("function identifier"),
        )
    };
}
/// Which sequence return variants an adapter should expose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackReturn {
    /// Keep the ordinary return and add a callback variant.
    Additional,
    /// Expose only the callback variant, including multi-value Kotlin elements
    /// that have no ordinary List representation.
    Only,
}

/// A source callable to export, with optional destination name and occurrence overrides.
#[derive(Clone, Debug)]
pub struct DeclFunction<InTree, OutTree, Extension = std::convert::Infallible> {
    /// Adapter-defined settings, preserved by common fluent methods.
    pub extension: Option<Extension>,
    /// Treat an outer standard Result as ordinary variant data.
    pub result_value: bool,
    /// Also emit an explicit callback-delivery variant for a sequence return.
    pub callback_return: Option<CallbackReturn>,
    /// Unqualified source function name in the flat namespace.
    pub name: String,
    /// Explicit destination name; supported by JNI, currently rejected by C.
    pub foreign: Option<String>,
    /// Input expansion overrides keyed by source parameter name.
    pub params: BTreeMap<String, InTree>,
    /// Result expansion override, replacing the type default.
    pub ret: Option<OutTree>,
    /// Source mapper for a top-level Result error; it takes E or &E and returns String.
    pub error: Option<String>,
}
impl<InTree, OutTree, Extension> DeclFunction<InTree, OutTree, Extension> {
    /// Reference a source function without overriding names or conversions.
    ///
    /// `name` is the function's unqualified source name.
    pub fn new(name: syn::Ident) -> Self {
        Self {
            extension: None,
            result_value: false,
            callback_return: None,
            name: name.to_string(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
            error: None,
        }
    }
    /// Configure adapter-specific settings without replacing common declaration data.
    pub fn with_extension<Next>(self, extension: Next) -> DeclFunction<InTree, OutTree, Next> {
        DeclFunction {
            extension: Some(extension),
            result_value: self.result_value,
            callback_return: self.callback_return,
            name: self.name,
            foreign: self.foreign,
            params: self.params,
            ret: self.ret,
            error: self.error,
        }
    }
    /// Extract an undecorated helper's source name, rejecting binding-only settings.
    pub fn source_name(self) -> String {
        self.reference_name(false)
    }
    /// Source reference used by constructor, getter and error-mapper declarations.
    /// Extract a helper source name, optionally accepting a getter view name.
    pub fn reference_name(self, allow_name: bool) -> String {
        assert!(
            self.extension.is_none()
                && self.params.is_empty()
                && self.ret.is_none()
                && self.error.is_none()
                && !self.result_value
                && self.callback_return.is_none(),
            "unsupported settings on this source helper"
        );
        assert!(
            allow_name || self.foreign.is_none(),
            "this helper reference does not support renaming"
        );
        self.name
    }
    /// Replace one parameter's default conversion using the production API.
    pub fn expand_param(self, name: impl AsRef<str>, decl: impl Into<InTree>) -> Self {
        self.input(name, decl)
    }
    /// Replace this return's default conversion using the production API.
    pub fn expand_return(self, decl: impl Into<OutTree>) -> Self {
        self.output(decl)
    }
    /// Deliver the whole source Result as data instead of routing its Err arm.
    /// Return overrides then address the complete Result. Incompatible with on_error.
    pub fn result_as_value(mut self) -> Self {
        self.result_value = true;
        self
    }
    /// Add callback delivery alongside the ordinary sequence return.
    ///
    /// C emits `<function>_each`, whose visitor receives each element's converted
    /// fields. Kotlin emits `<function>Fold(initial, ..., fold)`, whose folder
    /// receives an accumulator and each element's converted Kotlin values.
    /// Supports one projected sequence, optionally inside Option and an outer Result.
    /// Existing buffer/list returns are preserved. Adapter builders validate the
    /// return shape and generated-name collisions.
    ///
    /// On adapters supporting bulk, expansions stay within each callback element.
    /// The ordinary variant
    /// and callback variant are prepared separately from the same declaration.
    /// For a source returning `Vec<Row>`, this selects a frame per row for the
    /// callback variant; its ordinary return keeps a frame for the collection:
    /// For a source returning `Vec<Vec<Row>>`, target the inner collection to
    /// select one bulk frame per batch:
    /// An explicit bulk wrapper around the entire callback sequence is invalid;
    /// apply bulk to its element expansion instead. This also applies to a bulk
    /// wrapper supplied by a whole-collection type default. Nested automatic
    /// sequences stay ordinary inside a callback element; explicit bulk opts in
    /// to packing them. Chain order does not affect these policies, and
    /// per-function expansions override type-wide defaults.
    pub fn callback_return(mut self) -> Self {
        self.callback_return = Some(CallbackReturn::Additional);
        self
    }
    /// Expose only callback delivery, without generating an ordinary return.
    ///
    /// This explicitly permits decomposed multi-value Kotlin elements without
    /// requiring a Kotlin data class. Generated names are the same as for
    /// `callback_return`: C `_each` and Kotlin `Fold`.
    pub fn callback_return_only(mut self) -> Self {
        self.callback_return = Some(CallbackReturn::Only);
        self
    }
    /// Map the error of a top-level `Result<T, E>` return with a source function.
    ///
    /// The helper must take exactly `E` or `&E` and return `String`. Only its
    /// source name is accepted; it is selected for capture but is not exported.
    /// The success value uses normal return defaults and `expand_return` overrides.
    /// Replaces any previously selected mapper. Non-Result returns reject this option.
    pub fn on_error(mut self, mapper: impl Into<DeclFunction<InTree, OutTree, Extension>>) -> Self {
        self.error = Some(mapper.into().reference_name(false));
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
    pub fn input(mut self, name: impl AsRef<str>, decl: impl Into<InTree>) -> Self {
        assert!(
            self.params
                .insert(name.as_ref().into(), decl.into())
                .is_none(),
            "duplicate parameter override"
        );
        self
    }
    /// Replace the result expansion, including any previously assigned override.
    ///
    /// Configuration syntax uses the adapter's configuration operation sets.
    /// Resolved trees can be embedded with [`crate::OutputNode::map_into`] before
    /// registration, or passed directly to [`crate::compile_output`].
    pub fn output(mut self, decl: impl Into<OutTree>) -> Self {
        self.ret = Some(decl.into());
        self
    }
}

impl<InTree, OutTree, Extension> From<&str> for DeclFunction<InTree, OutTree, Extension> {
    fn from(name: &str) -> Self {
        Self::new(crate::ident(name).expect("source function identifier"))
    }
}
impl<InTree, OutTree, Extension> From<String> for DeclFunction<InTree, OutTree, Extension> {
    fn from(name: String) -> Self {
        Self::from(name.as_str())
    }
}

/// Parse source type syntax without requiring it to be in scope in build.rs.
pub fn parse_type(text: &str) -> syn::Type {
    syn::parse_str(text).expect("source type syntax")
}
