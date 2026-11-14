//! Source expansion builders specialized by adapter-provided node constructors.
use crate::{DeclFunction, InputNode, InputOps, OutputNode, OutputOps, Projection};
use std::{collections::BTreeMap, convert::Infallible};
use syn::Type;

/// Alternatives used to construct a source input, optionally as a borrowed temporary.
#[derive(Clone, Debug)]
pub struct DeclInputAlternatives<T> {
    alternatives: Vec<T>,
    reconstruct: bool,
}
impl<T> Default for DeclInputAlternatives<T> {
    fn default() -> Self {
        Self {
            alternatives: Vec::new(),
            reconstruct: false,
        }
    }
}
impl<T> DeclInputAlternatives<T> {
    /// Append an input alternative.
    pub fn push(&mut self, input: T) {
        self.alternatives.push(input);
    }
    /// Use an owned temporary to supply a shared-reference occurrence.
    pub fn reconstruct(&mut self) {
        self.reconstruct = true;
    }
    /// Build alternatives; an empty reconstructed expansion uses the owned default.
    pub fn build(
        self,
        selector: impl FnOnce(Vec<T>) -> T,
        default: impl FnOnce() -> T,
        temporary: impl FnOnce(T) -> T,
    ) -> T {
        let input = if self.reconstruct && self.alternatives.is_empty() {
            default()
        } else {
            selector(self.alternatives)
        };
        if self.reconstruct {
            temporary(input)
        } else {
            input
        }
    }
}

/// A source type's input alternatives, with optional adapter-defined settings.
/// The adapter supplies node constructors; the fluent declaration logic is shared.
pub struct DeclExpandParam<In, FunctionDecl, Extension = Infallible> {
    /// Source type targeted by this default or occurrence override.
    pub ty: Type,
    /// Adapter-defined settings; absent until explicitly configured.
    pub extension: Option<Extension>,
    input: DeclInputAlternatives<In>,
    constructor: fn(FunctionDecl) -> In,
    direct: fn() -> In,
    finish: fn(Type, DeclInputAlternatives<In>, Option<Extension>) -> In,
}
impl<In: Clone, F, E: Clone> Clone for DeclExpandParam<In, F, E> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            extension: self.extension.clone(),
            input: self.input.clone(),
            constructor: self.constructor,
            direct: self.direct,
            finish: self.finish,
        }
    }
}
impl<In: std::fmt::Debug, F, E: std::fmt::Debug> std::fmt::Debug for DeclExpandParam<In, F, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeclExpandParam")
            .field("ty", &self.ty)
            .field("extension", &self.extension)
            .field("input", &self.input)
            .finish_non_exhaustive()
    }
}
impl<In, F, E> DeclExpandParam<In, F, E> {
    /// Specialize input expansion for one adapter's constructor, direct value and final tree.
    pub fn new(
        ty: Type,
        constructor: fn(F) -> In,
        direct: fn() -> In,
        finish: fn(Type, DeclInputAlternatives<In>, Option<E>) -> In,
    ) -> Self {
        Self {
            ty,
            extension: None,
            input: Default::default(),
            constructor,
            direct,
            finish,
        }
    }
    /// Configure adapter-specific options without replacing the common declaration.
    pub fn with_extension(mut self, extension: E) -> Self {
        self.extension = Some(extension);
        self
    }
    /// Explicitly construct an owned temporary and borrow it for the source call.
    /// With no alternatives, use the owned referent's default input conversion.
    /// Alternatives describe the owned referent, not the reference itself.
    pub fn reconstruct(mut self) -> Self {
        self.input.reconstruct();
        self
    }
    /// Append a source constructor whose arguments use their type defaults.
    pub fn variant(mut self, function: impl Into<F>) -> Self {
        self.input.push((self.constructor)(function.into()));
        self
    }
    /// Append the adapter's direct value representation.
    pub fn variant_self(mut self) -> Self {
        self.input.push((self.direct)());
        self
    }
    /// Produce the adapter's typed input tree, including its extension settings.
    pub fn into_input(self) -> In {
        (self.finish)(self.ty, self.input, self.extension)
    }
}
impl<I: InputOps, O: OutputOps, F, E> From<DeclExpandParam<InputNode<I, O>, F, E>>
    for InputNode<I, O>
{
    fn from(declaration: DeclExpandParam<Self, F, E>) -> Self {
        declaration.into_input()
    }
}

/// Simultaneous observations of a source return value, with adapter-defined settings.
pub struct DeclExpandReturn<Out, FunctionDecl, Extension = Infallible> {
    /// Source type targeted by this default or occurrence override.
    pub ty: Type,
    /// Adapter-defined settings; absent until explicitly configured.
    pub extension: Option<Extension>,
    views: Vec<Out>,
    getter: fn(FunctionDecl) -> Out,
    direct: fn() -> Out,
    projected: fn(Projection) -> Out,
    owned_fields: fn(Out) -> Out,
    finish: fn(Type, Vec<Out>, Option<Extension>) -> Out,
}
impl<Out: Clone, F, E: Clone> Clone for DeclExpandReturn<Out, F, E> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            extension: self.extension.clone(),
            views: self.views.clone(),
            getter: self.getter,
            direct: self.direct,
            projected: self.projected,
            owned_fields: self.owned_fields,
            finish: self.finish,
        }
    }
}
impl<Out: std::fmt::Debug, F, E: std::fmt::Debug> std::fmt::Debug for DeclExpandReturn<Out, F, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeclExpandReturn")
            .field("ty", &self.ty)
            .field("extension", &self.extension)
            .field("views", &self.views)
            .finish_non_exhaustive()
    }
}
impl<Out, F, E> DeclExpandReturn<Out, F, E> {
    /// Specialize getter views, direct/projected values, owned fields and the final tree.
    pub fn new(
        ty: Type,
        getter: fn(F) -> Out,
        direct: fn() -> Out,
        projected: fn(Projection) -> Out,
        owned_fields: fn(Out) -> Out,
        finish: fn(Type, Vec<Out>, Option<E>) -> Out,
    ) -> Self {
        Self {
            ty,
            extension: None,
            views: Vec::new(),
            getter,
            direct,
            projected,
            owned_fields,
            finish,
        }
    }
    /// Configure adapter-specific options, preserving previously declared views.
    pub fn with_extension(mut self, extension: E) -> Self {
        self.extension = Some(extension);
        self
    }
    /// Append a checked getter view; an explicit helper name labels the view.
    /// Shared results are observed before another view moves the owner. An
    /// owning opaque copy requires an explicit projection or cloning accessor.
    pub fn field(mut self, function: impl Into<F>) -> Self {
        self.views.push((self.getter)(function.into()));
        self
    }
    /// Append a getter's named fields as separate output values.
    pub fn fields<FE>(self, fields: DeclFields<Out, FE>) -> Self {
        self.view(fields.into_output())
    }
    /// Append another output observation in declaration order.
    pub fn view(mut self, output: impl Into<Out>) -> Self {
        self.views.push(output.into());
        self
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into<FE>(mut self, fields: DeclFields<Out, FE>) -> Self {
        self.views.push((self.owned_fields)(fields.into_output()));
        self
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into<FE>(self, fields: DeclFields<Out, FE>) -> Self {
        self.fields_into(fields)
    }
    /// Append the direct value representation, bypassing this type's boundary default.
    pub fn field_self(mut self) -> Self {
        self.views.push((self.direct)());
        self
    }
    /// Append an explicit shared projection. Owning opaque representations clone
    /// the value; record/collection projections permit copying borrowed leaves.
    pub fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    pub fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and field policies.
    pub fn project_self(mut self, policy: Projection) -> Self {
        self.views.push((self.projected)(policy));
        self
    }
    /// Produce the adapter's typed output tree, preserving empty and singleton views.
    pub fn into_output(self) -> Out {
        (self.finish)(self.ty, self.views, self.extension)
    }
}
impl<I: InputOps, O: OutputOps, F, E> From<DeclExpandReturn<OutputNode<I, O>, F, E>>
    for OutputNode<I, O>
{
    fn from(declaration: DeclExpandReturn<Self, F, E>) -> Self {
        declaration.into_output()
    }
}

/// A source accessor and named field overrides, specialized by the adapter's output tree.
#[derive(Clone, Debug)]
pub struct DeclFields<Out, Extension = Infallible> {
    /// Adapter-defined field-decomposition options.
    pub extension: Option<Extension>,
    function: String,
    overrides: BTreeMap<String, Out>,
    normalize: fn(Out) -> Out,
    finish: fn(String, BTreeMap<String, Out>, Option<Extension>) -> Out,
}
impl<Out, E> DeclFields<Out, E> {
    /// Select an accessor and the adapter's omission normalization and output constructor.
    pub fn new(
        function: syn::Ident,
        normalize: fn(Out) -> Out,
        finish: fn(String, BTreeMap<String, Out>, Option<E>) -> Out,
    ) -> Self {
        Self {
            extension: None,
            function: function.to_string(),
            overrides: BTreeMap::new(),
            normalize,
            finish,
        }
    }
    /// Configure adapter-specific options, preserving field overrides.
    pub fn with_extension(mut self, extension: E) -> Self {
        self.extension = Some(extension);
        self
    }
    /// Replace a field conversion; the adapter recognizes empty-expansion omission.
    pub fn field(self, name: impl AsRef<str>, output: impl Into<Out>) -> Self {
        self.field_override(name.as_ref().to_owned(), output)
    }
    /// Replace a field conversion, rejecting duplicate overrides.
    pub fn field_override(mut self, name: impl Into<String>, output: impl Into<Out>) -> Self {
        assert!(
            self.overrides
                .insert(name.into(), (self.normalize)(output.into()))
                .is_none(),
            "duplicate field override"
        );
        self
    }
    /// Produce the accessor's output tree with all field and extension settings.
    pub fn into_output(self) -> Out {
        (self.finish)(self.function, self.overrides, self.extension)
    }
}
impl<I: InputOps, O: OutputOps, E> From<DeclFields<OutputNode<I, O>, E>> for OutputNode<I, O> {
    fn from(declaration: DeclFields<Self, E>) -> Self {
        declaration.into_output()
    }
}

/// Build a getter observation, preserving its foreign name and checking helper settings.
pub fn getter_view<I, O, E>(
    function: DeclFunction<I, O, E>,
    getter: impl FnOnce(String) -> O,
    named: impl FnOnce(String, O) -> O,
) -> O {
    let name = function.foreign.clone();
    let output = getter(function.reference_name(true));
    match name {
        Some(name) => named(name, output),
        None => output,
    }
}

/// A source-error mapping with an adapter's function reference and optional settings.
#[derive(Clone, Debug)]
pub struct DeclError<FunctionDecl, Extension = Infallible> {
    /// Source error type.
    pub ty: Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: FunctionDecl,
    /// Adapter-defined error delivery settings.
    pub extension: Option<Extension>,
}
impl<F, E> DeclError<F, E> {
    /// Select the source helper for this error type.
    pub fn new(ty: Type, mapper: F) -> Self {
        Self {
            ty,
            mapper,
            extension: None,
        }
    }
    /// Configure adapter-specific error settings.
    pub fn with_extension<Next>(self, extension: Next) -> DeclError<F, Next> {
        DeclError {
            ty: self.ty,
            mapper: self.mapper,
            extension: Some(extension),
        }
    }
}

/// Register an input expansion, output expansion or source-error mapping.
#[derive(Clone, Debug)]
pub enum DeclExpand<ParamDecl, ReturnDecl, ErrorDecl> {
    /// Input alternatives.
    Param(ParamDecl),
    /// Simultaneous output views.
    Return(ReturnDecl),
    /// Source-error mapping.
    Error(Box<ErrorDecl>),
}
impl<I, F, E, R, X> From<DeclExpandParam<I, F, E>> for DeclExpand<DeclExpandParam<I, F, E>, R, X> {
    fn from(value: DeclExpandParam<I, F, E>) -> Self {
        Self::Param(value)
    }
}
impl<O, F, E, P, X> From<DeclExpandReturn<O, F, E>>
    for DeclExpand<P, DeclExpandReturn<O, F, E>, X>
{
    fn from(value: DeclExpandReturn<O, F, E>) -> Self {
        Self::Return(value)
    }
}
impl<F, E, P, R> From<DeclError<F, E>> for DeclExpand<P, R, DeclError<F, E>> {
    fn from(value: DeclError<F, E>) -> Self {
        Self::Error(Box::new(value))
    }
}
