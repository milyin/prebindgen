//! The function wrapper writer.
//!
//! A wrapper is one exported function that receives wires, converts them to
//! the source function's arguments, calls it, and hands the result back as
//! wires:
//!
//! ```text
//! <attrs>
//! pub unsafe extern "C" fn <name><generics>(<leading>, <param wires>, <return wires>, <trailing>) -> <return type> {
//!     <prologue>
//!     let <param> = <param input>;          // on failure: <fail>
//!     ...
//!     let __result = <callee>(<params>);
//!     <return body>
//! }
//! ```
//!
//! The writer owns the shape; the adapter owns every decision, through
//! [`FunctionCallbacks`]: which wires a parameter becomes and how they turn
//! back into the argument, which wires the result leaves on, and what the
//! wrapper does when a conversion fails. The builder methods customize the
//! header and the fixed parts of the body.

use prebindgen_flat::flat::{Function, Param, TypeRef};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::wire::{Input, Wire};

/// How a wrapper delivers the source function's result.
#[derive(Clone, Debug, Default)]
pub struct Return {
    /// The wrapper's return type; `None` for `()`.
    pub ty: Option<TokenStream>,
    /// Extra parameters the result needs — out-parameters, a builder object,
    /// an error slot — placed after the parameters' wires.
    pub wires: Vec<Wire>,
    /// Statements that consume [`RESULT`] and end in the wrapper's return
    /// value (a tail expression, or nothing for `()`).
    pub body: TokenStream,
}

/// The local the source function's result is bound to; [`Return::body`]
/// reads it.
pub const RESULT: &str = "__result";

/// The local a failed conversion's message (`String`) is bound to;
/// [`FunctionCallbacks::fail`] reads it.
pub const ERROR: &str = "__err";

/// The result local, as an ident.
pub fn result_ident() -> syn::Ident {
    syn::Ident::new(RESULT, proc_macro2::Span::call_site())
}

/// The error local, as an ident.
pub fn error_ident() -> syn::Ident {
    syn::Ident::new(ERROR, proc_macro2::Span::call_site())
}

/// The adapter's decisions for one wrapper.
pub trait FunctionCallbacks {
    type Error;

    /// The wires `param` arrives on and how they become the argument.
    /// Called after [`Self::ret`], in parameter order.
    fn param(&mut self, func: &Function, param: &Param) -> Result<Input, Self::Error>;

    /// How the result of type `ret` leaves the wrapper. Called first.
    fn ret(&mut self, func: &Function, ret: &TypeRef) -> Result<Return, Self::Error>;

    /// Statements run when a parameter's conversion fails, with the message
    /// bound to [`ERROR`]. They must diverge — return from the wrapper or
    /// panic. `ret` is what [`Self::ret`] returned, for the default value.
    fn fail(&mut self, func: &Function, ret: &Return) -> TokenStream;
}

/// Writes one wrapper around a flat [`Function`].
pub struct FunctionWriter<'f> {
    func: &'f Function,
    name: syn::Ident,
    callee: TokenStream,
    attrs: Vec<TokenStream>,
    vis: TokenStream,
    abi: Option<String>,
    unsafety: bool,
    generics: TokenStream,
    leading: Vec<Wire>,
    trailing: Vec<Wire>,
    prologue: TokenStream,
    call: Option<Box<dyn Fn(&TokenStream, &[TokenStream]) -> TokenStream + 'f>>,
}

impl<'f> FunctionWriter<'f> {
    /// A wrapper around `func`, called through `callee` (usually
    /// [`Qualifier::path`](crate::Qualifier::path) of the function's name),
    /// exported under the function's own name until renamed.
    pub fn new(func: &'f Function, callee: impl ToTokens) -> Self {
        Self {
            func,
            name: func.name.clone(),
            callee: callee.to_token_stream(),
            attrs: Vec::new(),
            vis: quote!(pub),
            abi: Some("C".to_string()),
            unsafety: true,
            generics: TokenStream::new(),
            leading: Vec::new(),
            trailing: Vec::new(),
            prologue: TokenStream::new(),
            call: None,
        }
    }

    /// The exported name.
    pub fn name(mut self, name: syn::Ident) -> Self {
        self.name = name;
        self
    }

    /// An attribute on the wrapper (`#[no_mangle]`, `#[allow(..)]`).
    pub fn attr(mut self, attr: impl ToTokens) -> Self {
        self.attrs.push(attr.to_token_stream());
        self
    }

    /// The visibility (default `pub`).
    pub fn vis(mut self, vis: impl ToTokens) -> Self {
        self.vis = vis.to_token_stream();
        self
    }

    /// The ABI (default `"C"`); `None` writes a plain Rust function.
    pub fn abi(mut self, abi: Option<&str>) -> Self {
        self.abi = abi.map(str::to_string);
        self
    }

    /// Whether the wrapper is an `unsafe fn` (default yes: it dereferences
    /// what the foreign side hands it).
    pub fn unsafety(mut self, unsafety: bool) -> Self {
        self.unsafety = unsafety;
        self
    }

    /// Generic parameters, brackets included: `<'a>`.
    pub fn generics(mut self, generics: impl ToTokens) -> Self {
        self.generics = generics.to_token_stream();
        self
    }

    /// A parameter before the parameters' wires (`env`, `class`).
    pub fn leading(mut self, wire: Wire) -> Self {
        self.leading.push(wire);
        self
    }

    /// A parameter after everything else (an error sink).
    pub fn trailing(mut self, wire: Wire) -> Self {
        self.trailing.push(wire);
        self
    }

    /// Statements at the top of the body, before any conversion.
    pub fn prologue(mut self, stmts: impl ToTokens) -> Self {
        self.prologue.extend(stmts.to_token_stream());
        self
    }

    /// Replace the call expression. `f` gets the callee and the arguments as
    /// they would be passed.
    pub fn call_with(mut self, f: impl Fn(&TokenStream, &[TokenStream]) -> TokenStream + 'f) -> Self {
        self.call = Some(Box::new(f));
        self
    }

    /// Write the wrapper.
    pub fn write<C: FunctionCallbacks>(self, cb: &mut C) -> Result<TokenStream, C::Error> {
        let func = self.func;
        // The result first: how a failed input is reported depends on it.
        let ret = cb.ret(func, &func.ret)?;
        let mut inputs = Vec::with_capacity(func.params.len());
        for p in &func.params {
            inputs.push(cb.param(func, p)?);
        }
        let fail = cb.fail(func, &ret);

        let mut params: Vec<TokenStream> = self.leading.iter().map(Wire::decl).collect();
        let mut stmts = Vec::new();
        let mut args = Vec::new();
        let err = error_ident();
        for (p, input) in func.params.iter().zip(&inputs) {
            params.extend(input.wires.iter().map(Wire::decl));
            let name = &p.name;
            let e = &input.expr;
            if input.fallible {
                let r = input.result();
                stmts.push(quote! {
                    let #name = match #r {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(#err) => { #fail }
                    };
                });
            } else {
                stmts.push(quote!(let #name = #e;));
            }
            args.push(input.pass.clone().unwrap_or_else(|| quote!(#name)));
        }
        params.extend(ret.wires.iter().map(Wire::decl));
        params.extend(self.trailing.iter().map(Wire::decl));

        let callee = &self.callee;
        let call = match &self.call {
            Some(f) => f(callee, &args),
            None => quote!(#callee(#(#args),*)),
        };
        let result = result_ident();
        let body = &ret.body;
        let ret_ty = ret.ty.as_ref().map(|t| quote!(-> #t));
        let attrs = &self.attrs;
        let vis = &self.vis;
        let unsafety = self.unsafety.then(|| quote!(unsafe));
        let abi = self.abi.as_ref().map(|a| {
            let lit = syn::LitStr::new(a, proc_macro2::Span::call_site());
            quote!(extern #lit)
        });
        let name = &self.name;
        let generics = &self.generics;
        let prologue = &self.prologue;
        Ok(quote! {
            #(#attrs)*
            #vis #unsafety #abi fn #name #generics(#(#params),*) #ret_ty {
                #prologue
                #(#stmts)*
                let #result = #call;
                #body
            }
        })
    }
}
