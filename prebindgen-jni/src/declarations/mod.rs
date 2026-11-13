//! Language-owned configuration and source resolution.
use super::*;
use prebindgen_declarations::{option_inner, result_parts, sequence_parts};
use prebindgen_declarations::{
    CallbackArgumentOwnership, DeclConvert, FunctionRecipe, ResolvedSchema,
};
use prebindgen_declarations::{Codec, ResolvedInput, ResolvedOutput};
use prebindgen_declarations::{
    InputNode, OutputNode, Projection, ProjectionMode, SequenceDelivery,
};
use prebindgen_flat::ValueConversion;
use prebindgen_flat::{same_type, type_key, Flat, Result, Schema};
use std::collections::BTreeMap;
use syn::Type;
/// Function binding choices containing this adapter's conversion trees.
/// A C declaration cannot be registered with JNI:
/// ```compile_fail
/// use prebindgen_jni::package;
/// let _ = package!().fun(prebindgen_c::fun!(send));
/// ```
pub type DeclJniFunction = prebindgen_declarations::DeclFunction<
    JniInput<JniInputRequest, JniOutputRequest>,
    JniOutput<JniInputRequest, JniOutputRequest>,
>;

mod requests;
pub use requests::*;
mod builders;
pub use builders::*;
mod resolution;
pub use resolution::*;
mod references;
pub(crate) use references::source_items;
#[macro_export]
/// Start a typed input declaration using the production builder API.
macro_rules! expand_param {($($ty:tt)+)=>{$crate::expand_param($crate::__declaration_type(stringify!($($ty)+)))};}
#[macro_export]
/// Start a typed output declaration using the production builder API.
macro_rules! expand_return {($($ty:tt)+)=>{$crate::expand_return($crate::__declaration_type(stringify!($($ty)+)))};}
#[macro_export]
/// Decompose a source getter's returned record into named field views.
macro_rules! fields {
    ($name:ident) => {
        $crate::fields($crate::__declaration_ident(stringify!($name)).expect("function identifier"))
    };
}

/// Declare a callback interface, defaults, or a callback occurrence override.
#[macro_export]
macro_rules! callback {
    ($($ty:tt)+) => { $crate::DeclJniCallback::new($crate::__declaration_type(stringify!($($ty)+))) };
}

#[doc(hidden)]
pub fn __declaration_ident(name: &str) -> Result<syn::Ident> {
    prebindgen_declarations::ident(name)
}

/// Resolve and bind one input with no generator-wide defaults.
pub fn bind_input(
    flat: &mut JniBindings,
    codec: &JniCodec,
    function: &str,
    parameter: &str,
    tree: &JniInput<JniInputRequest, JniOutputRequest>,
    extra: &(),
) -> Result<()> {
    let defaults = DeclJniDefaults::new(flat, vec![], vec![], vec![])?;
    let ty = &flat
        .function(function)?
        .params
        .iter()
        .find(|p| p.name == parameter)
        .ok_or("unknown parameter")?
        .ty
        .ty;
    let tree = defaults.resolve_input(flat, codec, ty, tree, extra)?;
    prebindgen_declarations::bind_input(flat, codec, function, parameter, &tree, extra)
}
/// Resolve and bind one output with no generator-wide defaults.
pub fn bind_output(
    flat: &mut JniBindings,
    codec: &JniCodec,
    function: &str,
    tree: &JniOutput<JniInputRequest, JniOutputRequest>,
    extra: &(),
) -> Result<()> {
    let defaults = DeclJniDefaults::new(flat, vec![], vec![], vec![])?;
    let tree =
        defaults.resolve_output(flat, codec, &flat.function(function)?.ret.ty, tree, extra)?;
    prebindgen_declarations::bind_output(flat, codec, function, &tree, extra)
}

#[doc(hidden)]
pub fn __declaration_type(text: &str) -> syn::Type {
    prebindgen_declarations::declarations::parse_type(text)
}
