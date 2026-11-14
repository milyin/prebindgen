//! Signature defaults and occurrence overrides for source callbacks.
use prebindgen_flat::{callback_signature, Flat, Result, Schema};
use syn::Type;

/// Source identity and foreign declaration policy for one callback occurrence.
///
/// The Arrow's domain and codomain own the conversions. Both directions are
/// fully resolved before the adapter compiles the callback.
#[derive(Clone, Debug)]
pub struct CallbackOperation<Extension = std::convert::Infallible> {
    /// Adapter-specific callback settings retained in the resolved tree.
    pub extension: Option<Extension>,
    /// Source callback signature; canonicalized when the configuration resolves.
    pub ty: Type,
    /// Literal foreign interface name, if explicitly selected.
    pub foreign: Option<String>,
    /// Foreign namespace selected by the adapter.
    pub namespace: Option<String>,
    /// Whether the callback result has an explicit callback-level conversion.
    pub explicit_return: bool,
}

impl<Extension> CallbackOperation<Extension> {
    /// Verify the configured signature against this source callback occurrence.
    pub fn validate_for<M: Schema>(&self, flat: &Flat<M>, ty: &Type) -> Result<()> {
        if input_key(flat, ty)? != prebindgen_flat::type_key(&self.ty) {
            return Err("prepared callback type does not match occurrence".into());
        }
        callback_signature(&self.ty)?.ok_or("prepared callback requires impl Fn(...) -> R")?;
        Ok(())
    }
}

/// Resolve a callback signature for configuration identity, excluding marker bounds.
pub fn input_type<M: Schema>(flat: &Flat<M>, ty: &Type) -> Result<Type> {
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
/// Canonical callback identity without lifetimes or marker bounds.
pub fn input_key<M: Schema>(flat: &Flat<M>, ty: &Type) -> Result<String> {
    if callback_signature(ty)?.is_some() {
        Ok(prebindgen_flat::type_key(&input_type(flat, ty)?))
    } else {
        Ok(prebindgen_flat::type_key(ty))
    }
}

/// Ownership of foreign values delivered as a callable argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackArgumentOwnership {
    /// Generated wrappers release produced owners when the invocation ends.
    Scoped,
    /// The callback author receives the produced owners and releases them.
    Transferred,
}

use std::collections::BTreeMap;
/// A retained callback's foreign declaration and independently selected conversions.
///
/// Arguments leave Rust and accept output declarations; the returned value enters
/// Rust and accepts an input declaration. Unspecified settings inherit the
/// callback signature's defaults, followed by ordinary value-type defaults.
/// Each overridden argument/result replaces its entire conversion tree.
///
#[derive(Clone, Debug)]
pub struct DeclCallback<InTree, OutTree, Extension = std::convert::Infallible> {
    /// Adapter-defined settings, preserved by common fluent methods.
    pub extension: Option<Extension>,
    /// Source `impl Fn(...) -> R` signature; marker bounds do not identify defaults.
    pub ty: Type,
    /// Literal destination interface name.
    pub foreign: Option<String>,
    /// Destination namespace, supplied by the language adapter's placement declaration.
    pub namespace: Option<String>,
    /// Argument output overrides, indexed in source invocation order.
    pub params: BTreeMap<usize, OutTree>,
    /// Returned-value input override.
    pub ret: Option<InTree>,
}
impl<InTree, OutTree, Extension> DeclCallback<InTree, OutTree, Extension> {
    /// Declare defaults or an occurrence override for a source callback signature.
    pub fn new(ty: Type) -> Self {
        Self {
            extension: None,
            ty,
            foreign: None,
            namespace: None,
            params: BTreeMap::new(),
            ret: None,
        }
    }
    /// Configure adapter-specific settings without replacing common declaration data.
    pub fn with_extension<Next>(self, extension: Next) -> DeclCallback<InTree, OutTree, Next> {
        DeclCallback {
            extension: Some(extension),
            ty: self.ty,
            foreign: self.foreign,
            namespace: self.namespace,
            params: self.params,
            ret: self.ret,
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
    pub fn expand_param(mut self, index: usize, output: impl Into<OutTree>) -> Self {
        assert!(
            self.params.insert(index, output.into()).is_none(),
            "duplicate callback parameter override"
        );
        self
    }
    /// Override the callback result's foreign-to-Rust conversion.
    pub fn expand_return(mut self, input: impl Into<InTree>) -> Self {
        self.ret = Some(input.into());
        self
    }
}

/// Metadata for a callback whose explicit conversions live on structural tree edges.
#[derive(Clone, Debug)]
pub struct CallbackConfig<Extension = std::convert::Infallible> {
    /// Source identity, foreign placement and adapter settings.
    pub operation: CallbackOperation<Extension>,
    /// Source argument indices corresponding to the domain's explicit children.
    pub arguments: Vec<usize>,
}
/// Adapter construction of a retained callback's complete input tree.
/// This bridge permits the shared declaration's `Into<InputNode>` implementation
/// while the adapter selects its own operation and grouping metadata.
pub trait CallbackInput<InTree, OutTree, Extension = std::convert::Infallible> {
    /// Put explicit argument/result recipes on structural edges, preserving type scope.
    fn callback_input(declaration: DeclCallback<InTree, OutTree, Extension>) -> InTree;
}
/// Assemble a callback declaration using the adapter's structural node constructors.
pub fn callback_input<I, O, E>(
    declaration: DeclCallback<I, O, E>,
    auto: impl FnOnce() -> I,
    arguments: impl FnOnce(Vec<O>) -> O,
    arrow: impl FnOnce(CallbackConfig<E>, O, I) -> I,
    typed: impl FnOnce(Type, I) -> I,
) -> I {
    let ty = declaration.ty;
    let explicit_return = declaration.ret.is_some();
    let (indices, children) = declaration.params.into_iter().unzip();
    let metadata = CallbackConfig {
        operation: CallbackOperation {
            ty: ty.clone(),
            foreign: declaration.foreign,
            namespace: declaration.namespace,
            extension: declaration.extension,
            explicit_return,
        },
        arguments: indices,
    };
    let domain = arguments(children);
    let codomain = declaration.ret.unwrap_or_else(auto);
    typed(ty, arrow(metadata, domain, codomain))
}
impl<I: crate::InputOps, O: crate::OutputOps, E>
    From<DeclCallback<crate::InputNode<I, O>, crate::OutputNode<I, O>, E>>
    for crate::InputNode<I, O>
where
    I: CallbackInput<crate::InputNode<I, O>, crate::OutputNode<I, O>, E>,
{
    fn from(declaration: DeclCallback<Self, crate::OutputNode<I, O>, E>) -> Self {
        I::callback_input(declaration)
    }
}
