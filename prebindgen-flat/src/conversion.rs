//! Wire descriptions and generator-defined expression requests.
use crate::Result;
use proc_macro2::TokenStream;
use quote::quote;

/// Type information for one logical value. A generator may pass it directly or
/// encode it as a packed field/callback argument.
/// The returned type describes the boundary value, not the source value being converted.
/// For example, a boolean conversion may use `u8` as its Rust carrier type.
///
/// ```
/// use prebindgen_flat::WireKind;
/// struct BoolCarrier;
/// impl WireKind for BoolCarrier {
///     fn rust_type(&self) -> syn::Type { syn::parse_quote!(u8) }
/// }
/// let ty = BoolCarrier.rust_type();
/// assert_eq!(quote::quote!(#ty).to_string(), "u8");
/// ```
pub trait WireKind {
    /// Rust type for this individual value; this does not prescribe a signature.
    fn rust_type(&self) -> syn::Type;
}
impl WireKind for syn::Type {
    fn rust_type(&self) -> syn::Type {
        self.clone()
    }
}

/// One typed value definition in the logical input/output contract.
///
/// `W` must implement [`WireKind`]. Call [`Wire::rust_type`]
/// to obtain the Rust carrier type supplied by the kind.
///
/// ```
/// use prebindgen_flat::Wire;
/// let wire: Wire<syn::Type> = Wire {
///     name: syn::parse_quote!(value),
///     kind: syn::parse_quote!(i64),
/// };
/// let ty = wire.rust_type();
/// assert_eq!(quote::quote!(#ty).to_string(), "i64");
/// ```
/// A wire kind without a Rust carrier type is rejected:
///
/// ```compile_fail
/// use prebindgen_flat::Wire;
/// struct Untyped;
/// let wire = Wire { name: syn::parse_quote!(value), kind: Untyped };
/// ```
#[derive(Clone, Debug)]
pub struct Wire<W: WireKind> {
    /// Identifier used by conversion code within its element/branch context.
    pub name: syn::Ident,
    /// Individual value information implementing [`WireKind`].
    /// Stores information about this individual value, such as conversion access
    /// and available niches. Relationships between values belong on parent tree nodes.
    pub kind: W,
}

impl<W: WireKind> Wire<W> {
    /// Rust carrier type supplied by this wire's [`WireKind`].
    /// A generator can use this type directly or select a packed transport.
    pub fn rust_type(&self) -> syn::Type {
        self.kind.rust_type()
    }
}

/// Check that every bound function has an input converter for each parameter
/// and an output converter, including unit results.
///
/// Functions with entirely empty slots remain source-only helpers. This check
/// neither builds conversions nor changes the model. Adapters call it before
/// writing wrappers. It does not validate code fragments supplied directly by callers.
pub fn validate_bindings<M: crate::Schema>(flat: &crate::Flat<M>) -> Result<()> {
    for f in flat.functions.values() {
        if f.ret.slot.is_some() {
            for p in &f.params {
                if p.ty.slot.is_none() {
                    return Err(format!(
                        "`{}`: parameter `{}` has no input converter",
                        f.name, p.name
                    ));
                }
            }
        } else if f.params.iter().any(|p| p.ty.slot.is_some()) {
            return Err(format!(
                "`{}`: bound inputs have no output converter",
                f.name
            ));
        }
    }
    Ok(())
}
/// Format generated tokens as a Rust source file.
///
/// `tokens` must be a sequence of Rust items. Returns a syntax error if they do not form a file. This does not
/// resolve names or type-check generated code; the binding crate must compile it.
pub fn rust_source(tokens: TokenStream) -> Result<String> {
    let file = syn::parse2(tokens).map_err(|e| format!("generated Rust: {e}"))?;
    Ok(prettyplease::unparse(&file))
}

/// Prepared slot conversion with a generator-defined request protocol.
///
/// Input and output slots share this emission interface. Their payload types
/// may define different wire kinds and request enums. [`InputConversion`] adds
/// an input value description; [`OutputConversion`] adds output completion
/// channels. An output request can carry the source-result identifier without
/// imposing a shared context type.
///
/// Every successful request returns one Rust expression, including `()` for
/// a no-op. Statements may appear inside a block expression. The caller decides
/// how expressions are combined and binds any values needed by later requests.
/// `expected_type` is the caller's expectation, not a type-checking guarantee:
/// the implementation may use it to configure emission or reject a request.
/// Rust compilation performs the final type check. No implicit casts are required.
///
/// Request enums belong to the generator. Each variant can carry only the names,
/// settings or other data it needs; no common context or phase order is imposed.
/// Their lifetime parameter allows requests to borrow that data.
///
/// The tree describes logical values. Generators choose transport, runtime
/// repetition and branch selection. A leaf traversal index is not an ABI
/// parameter position. Requests needing an element/branch context carry it in
/// the generator-defined request enum; no per-wire ABI protocol is mandatory.
///
/// A reusable owned-box converter can define its own small request vocabulary.
/// The pointee is preparation data, not a hardcoded application type. The slot
/// request borrows an error message only for validation; acquisition needs no
/// context. The caller supplies `expected_type` to every request and binds the
/// resulting value itself. This example chooses syntactic type matching as its
/// policy; other implementations may use the expectation to configure emission.
///
/// ```
/// use prebindgen_flat::{bind_fragment, same_type, Conversion, InputConversion, Result, Wire, WireKind, WireNode, wire_definitions};
/// use proc_macro2::TokenStream;
/// use quote::quote;
/// use syn::{parse_quote, Type};
///
/// enum Carrier { OpaquePointer }
/// impl WireKind for Carrier {
///     fn rust_type(&self) -> Type { parse_quote!(*mut ::std::ffi::c_void) }
/// }
/// enum SlotRequest<'a> {
///     Validate { null_message: &'a str },
///     TakeOwnership,
/// }
/// struct OwnedBoxInput {
///     pointee: Type,
///     node: WireNode<Carrier>,
/// }
/// fn expect(actual: &Type, expected: &Type) -> Result<()> {
///     if same_type(actual, expected) { Ok(()) }
///     else { Err("unsupported expected type".into()) }
/// }
/// impl Conversion for OwnedBoxInput {
///     type Wire = Carrier;
///     type Chain = ();
///     type Branch = ();
///     type SlotFragment<'a> = SlotRequest<'a>;
///     fn code_fragment_slot(&self, mode: SlotRequest<'_>, expected: &Type)
///         -> Result<TokenStream>
///     {
///         let name = &wire_definitions(std::iter::once(&self.node)).next().unwrap().name;
///         let pointee = &self.pointee;
///         match mode {
///             SlotRequest::Validate { null_message } => {
///                 expect(&parse_quote!(()), expected)?;
///                 Ok(quote!({
///                     if #name.is_null() { return Err(#null_message.into()); }
///                 }))
///             }
///             SlotRequest::TakeOwnership => {
///                 expect(&parse_quote!(::std::boxed::Box<#pointee>), expected)?;
///                 // The caller guarantees a live, uniquely owned allocation
///                 // created by compatible Box::into_raw. Validation alone cannot.
///                 Ok(quote!(unsafe {
///                     ::std::boxed::Box::<#pointee>::from_raw(#name.cast::<#pointee>())
///                 }))
///             }
///         }
///     }
/// }
/// impl InputConversion for OwnedBoxInput {
///     fn input_node(&self) -> &WireNode<Carrier> { &self.node }
/// }
/// let slot = OwnedBoxInput {
///     pointee: parse_quote!(u64), // Supplied by preparation from the source type.
///     node: WireNode::Leaf(vec![Wire { name: parse_quote!(raw_value), kind: Carrier::OpaquePointer }]),
/// };
/// let validation = slot.code_fragment_slot(
///     SlotRequest::Validate { null_message: "null owned handle" }, &parse_quote!(()))?;
/// let expected = parse_quote!(::std::boxed::Box<u64>);
/// let value = slot.code_fragment_slot(SlotRequest::TakeOwnership, &expected)?;
/// let binding = bind_fragment(&parse_quote!(argument), &expected, value)?;
/// let _: syn::Block = syn::parse2(quote!({ #validation; #binding }))?;
/// let raw_parameter = wire_definitions(std::iter::once(slot.input_node())).next().unwrap();
/// assert_eq!(raw_parameter.name, "raw_value");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Conversion {
    /// Generator-defined type information for individual logical values.
    type Wire: WireKind;
    /// Adapter metadata for parents with exactly one child description.
    type Chain;
    /// Adapter metadata for parents with an ordered list of child descriptions.
    type Branch;
    /// Requests concerning the whole slot.
    type SlotFragment<'a>;
    /// Emit an expression for the whole slot.
    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected_type: &syn::Type,
    ) -> Result<TokenStream>;
}

/// Conversion attached to one source function parameter.
#[allow(clippy::type_complexity)] // Each parent variant has independently typed metadata.
pub trait InputConversion: Conversion {
    /// Description of the parameter value, independent of its native placement.
    fn input_node(&self) -> &crate::WireNode<Self::Wire, Self::Chain, Self::Branch>;
}
/// Conversion attached to a function return, including its completion channels.
#[allow(clippy::type_complexity)] // Keep completion channels and value metadata explicit.
pub trait OutputConversion: Conversion {
    /// Success and declared errors; outcomes cannot be children of a value node.
    fn output_outcomes(
        &self,
    ) -> &crate::Outcomes<crate::WireNode<Self::Wire, Self::Chain, Self::Branch>>;
}

/// Bind an emitted expression to a caller-selected name and expected Rust type.
///
/// Rejects empty or statement-only fragments. The generated type annotation
/// asks the Rust compiler to check the expectation in the wrapper's environment.
pub fn bind_fragment(
    target: &syn::Ident,
    expected_type: &syn::Type,
    fragment: TokenStream,
) -> Result<TokenStream> {
    let expression: syn::Expr =
        syn::parse2(fragment).map_err(|error| format!("expected a Rust expression: {error}"))?;
    Ok(quote!(let #target: #expected_type = #expression;))
}
impl<I: Conversion, O: Conversion, M> crate::Function<I, O, M> {
    /// Filled input slots, preserving source parameter order and grouping.
    pub fn input_conversions(&self) -> Result<Vec<&I>> {
        self.params
            .iter()
            .map(|p| {
                p.ty.slot.as_ref().ok_or_else(|| {
                    format!("`{}`: parameter `{}` has no conversion", self.name, p.name)
                })
            })
            .collect()
    }
    /// Filled result slot, borrowed without requesting any generated code.
    pub fn output_conversion(&self) -> Result<&O> {
        let output = self
            .ret
            .slot
            .as_ref()
            .ok_or_else(|| format!("`{}`: result has no conversion", self.name))?;
        Ok(output)
    }
    /// Logical input definitions, including each element/branch schema once.
    pub fn input_wire_definitions(&self) -> Result<impl Iterator<Item = &Wire<I::Wire>>>
    where
        I: InputConversion,
    {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| crate::wire_definitions(std::iter::once(i.input_node()))))
    }
}
