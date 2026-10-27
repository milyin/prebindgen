//! Signature defaults and occurrence overrides for source callbacks.
use crate::{Input, InputRequest, Output, OutputRequest};
use prebindgen_flat::{callback_signature, Flat, Result, Schema};
use std::collections::{BTreeMap, BTreeSet};
use syn::Type;

/// A retained callback's foreign declaration and independently selected conversions.
///
/// Arguments leave Rust and accept output declarations; the returned value enters
/// Rust and accepts an input declaration. Unspecified settings inherit the
/// callback signature's defaults, followed by ordinary value-type defaults.
/// Each overridden argument/result replaces its entire conversion tree.
///
/// ```
/// use prebindgen_declarations::{callback, expand_param, expand_return, fun};
/// let handler = callback!(impl Fn(Packet) -> Decision)
///     .name("PacketHandler")
///     .expand_param(0, expand_return!(Packet).field(fun!(packet_payload)))
///     .expand_return(expand_param!(Decision).variant(fun!(decision_new)));
/// let registration = fun!(register).expand_param("callback", handler);
/// ```
#[derive(Clone, Debug)]
pub struct CallbackDecl {
    /// Source `impl Fn(...) -> R` signature; marker bounds do not identify defaults.
    pub ty: Type,
    /// Literal destination interface name.
    pub foreign: Option<String>,
    /// Destination namespace, supplied by the language adapter's placement declaration.
    pub namespace: Option<String>,
    /// Argument output overrides, indexed in source invocation order.
    pub params: BTreeMap<usize, Output<OutputRequest>>,
    /// Returned-value input override.
    pub ret: Option<Input<InputRequest>>,
}
impl CallbackDecl {
    /// Declare defaults or an occurrence override for a source callback signature.
    pub fn new(ty: Type) -> Self {
        Self {
            ty,
            foreign: None,
            namespace: None,
            params: BTreeMap::new(),
            ret: None,
        }
    }
    /// Set a literal destination interface name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.foreign = Some(name.into());
        self
    }
    /// Override one argument's Rust-to-foreign conversion.
    ///
    /// Like a function parameter override, duplicate indices panic. Build checks
    /// that the index exists and the conversion matches the source argument type.
    pub fn expand_param(mut self, index: usize, output: impl Into<Output<OutputRequest>>) -> Self {
        assert!(
            self.params.insert(index, output.into()).is_none(),
            "duplicate callback parameter override"
        );
        self
    }
    /// Override the callback result's foreign-to-Rust conversion.
    pub fn expand_return(mut self, input: impl Into<Input<InputRequest>>) -> Self {
        self.ret = Some(input.into());
        self
    }
}
impl From<CallbackDecl> for Input<InputRequest> {
    fn from(decl: CallbackDecl) -> Self {
        Self::for_type(
            decl.ty.clone(),
            Self::Request(InputRequest::Callback(Box::new(decl))),
        )
    }
}

/// Prepared conversions for one callback occurrence.
///
/// Both directions are fully resolved before the adapter compiles the callback.
#[derive(Clone, Debug)]
pub struct CallbackInput {
    /// Canonical source callback signature, excluding marker bounds.
    pub ty: Type,
    /// Literal foreign interface name, if explicitly selected.
    pub foreign: Option<String>,
    /// Foreign namespace selected by the adapter.
    pub namespace: Option<String>,
    /// Rust-to-foreign argument conversions in source order.
    pub arguments: Vec<Output>,
    /// Foreign-to-Rust returned-value conversion.
    pub return_value: Box<Input>,
    /// Arguments explicitly expanded by a callback declaration or occurrence override.
    pub explicit_arguments: BTreeSet<usize>,
    /// Whether the callback result has an explicit callback-level conversion.
    pub explicit_return: bool,
}

impl CallbackInput {
    pub(crate) fn validate_for<M: Schema>(&self, flat: &Flat<M>, ty: &Type) -> Result<()> {
        if input_key(flat, ty)? != prebindgen_flat::type_key(&self.ty) {
            return Err("prepared callback type does not match occurrence".into());
        }
        let signature =
            callback_signature(&self.ty)?.ok_or("prepared callback requires impl Fn(...) -> R")?;
        if self.arguments.len() != signature.arguments.len() {
            return Err("prepared callback has the wrong argument count".into());
        }
        if self
            .explicit_arguments
            .iter()
            .any(|&i| i >= self.arguments.len())
        {
            return Err("prepared callback parameter override index is out of range".into());
        }
        Ok(())
    }
}

/// Declare a callback interface, defaults, or a callback occurrence override.
#[macro_export]
macro_rules! callback {
    ($($ty:tt)+) => { $crate::CallbackDecl::new($crate::declarations::parse_type(stringify!($($ty)+))) };
}

/// Resolve a callback signature for configuration identity, excluding marker bounds.
pub(crate) fn input_type<M: Schema>(flat: &Flat<M>, ty: &Type) -> Result<Type> {
    let Some(signature) = callback_signature(ty)? else {
        return flat.resolve_type(ty);
    };
    let arguments = signature
        .arguments
        .iter()
        .map(|t| flat.resolve_type(t))
        .collect::<Result<Vec<_>>>()?;
    let ret = flat.resolve_type(&signature.return_type)?;
    Ok(syn::parse_quote!(impl Fn(#(#arguments),*) -> #ret))
}

// Source occurrences have already been resolved by Flat. Only callback marker
// bounds need normalization when looking up defaults.
pub(crate) fn input_key<M: Schema>(flat: &Flat<M>, ty: &Type) -> Result<String> {
    if callback_signature(ty)?.is_some() {
        Ok(prebindgen_flat::type_key(&input_type(flat, ty)?))
    } else {
        Ok(prebindgen_flat::type_key(ty))
    }
}
