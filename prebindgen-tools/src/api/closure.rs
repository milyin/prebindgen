use prebindgen_flat::flat::TypeRef;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{wire::Output, Qualifier};

/// The adapter's decision for one closure argument.
pub trait ClosureCallbacks {
    type Error;

    /// The wires argument `index` (of type `ty`, held in `value`) leaves on.
    fn arg(
        &mut self,
        index: usize,
        ty: &TypeRef,
        value: &TokenStream,
    ) -> Result<Output, Self::Error>;
}

/// Writes one closure expression.
pub struct ClosureWriter<'a> {
    args: &'a [TypeRef],
    setup: TokenStream,
    on_error: TokenStream,
}

impl<'a> ClosureWriter<'a> {
    /// A closure over the argument types of a `TypeKind::Callback`.
    pub fn new(args: &'a [TypeRef]) -> Self {
        Self {
            args,
            setup: TokenStream::new(),
            on_error: TokenStream::new(),
        }
    }

    /// Statements run once, when the closure is built; bindings they make
    /// are moved into it.
    pub fn setup(mut self, stmts: impl ToTokens) -> Self {
        self.setup.extend(stmts.to_token_stream());
        self
    }

    /// Statements run when a call fails, with the message in `__err`.
    pub fn on_error(mut self, stmts: impl ToTokens) -> Self {
        self.on_error.extend(stmts.to_token_stream());
        self
    }

    /// The argument names the closure binds: `__a0`, `__a1`, …
    pub fn arg_names(&self) -> Vec<syn::Ident> {
        (0..self.args.len())
            .map(|i| format_ident!("__a{}", i))
            .collect()
    }

    /// Write the closure. `invoke` gets the statements binding every
    /// argument's wires and the arguments' outputs, and produces the
    /// statements that place the bindings and call the foreign side; they may
    /// use `?`.
    pub fn write<C: ClosureCallbacks>(
        self,
        q: &Qualifier<'_>,
        cb: &mut C,
        invoke: impl FnOnce(TokenStream, &[Output]) -> TokenStream,
    ) -> Result<TokenStream, C::Error> {
        let names = self.arg_names();
        let mut outs = Vec::new();
        for (i, (ty, n)) in self.args.iter().zip(&names).enumerate() {
            outs.push(cb.arg(i, ty, &n.to_token_stream())?);
        }
        let tys = self.args.iter().map(|t| q.ty_elided(t));
        let binds: TokenStream = outs.iter().map(Output::bind).collect();
        let invoke = invoke(binds, &outs);
        let (setup, on_error) = (&self.setup, &self.on_error);
        Ok(quote! {{
            #setup
            move |#(#names: #tys),*| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    #invoke
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    #on_error
                }
            }
        }})
    }
}
