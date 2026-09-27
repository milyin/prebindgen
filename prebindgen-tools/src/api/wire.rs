use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// One slot of the generated boundary: a name and a type as the generated
/// file spells it.
#[derive(Clone, Debug)]
pub struct Wire {
    pub name: syn::Ident,
    pub ty: TokenStream,
}

impl Wire {
    pub fn new(name: syn::Ident, ty: impl ToTokens) -> Self {
        Self {
            name,
            ty: ty.to_token_stream(),
        }
    }

    /// `name: ty`, for a parameter list or a struct body.
    pub fn decl(&self) -> TokenStream {
        let (n, t) = (&self.name, &self.ty);
        quote!(#n: #t)
    }
}

/// Wires → a source value.
#[derive(Clone, Debug)]
pub struct Input {
    /// The wires the value arrives on, in order.
    pub wires: Vec<Wire>,
    /// Evaluates to the source value, reading the wires by name.
    pub expr: TokenStream,
    /// Whether [`Self::expr`] uses `?` (on a `Result<_, String>`).
    pub fallible: bool,
    /// How a function wrapper passes the bound value to the callee, when it
    /// is not passed as is — `&*s` for a borrow of a local. `None` passes
    /// the local itself.
    pub pass: Option<TokenStream>,
}

impl Input {
    /// An infallible conversion.
    pub fn new(wires: Vec<Wire>, expr: impl ToTokens) -> Self {
        Self {
            wires,
            expr: expr.to_token_stream(),
            fallible: false,
            pass: None,
        }
    }

    /// A conversion whose expression uses `?`.
    pub fn fallible(wires: Vec<Wire>, expr: impl ToTokens) -> Self {
        Self {
            fallible: true,
            ..Self::new(wires, expr)
        }
    }

    /// One wire read as is: the value *is* the wire.
    pub fn identity(wire: Wire) -> Self {
        let n = wire.name.clone();
        Self::new(vec![wire], n)
    }

    /// Wrap the produced value: `f(expr)`.
    pub fn map(mut self, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.expr = f(self.expr);
        self
    }

    /// Wrap the produced value in a step that may fail; `f` must produce an
    /// expression using `?`.
    pub fn and_then(mut self, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.expr = f(self.expr);
        self.fallible = true;
        self
    }

    /// Mark how a function wrapper passes the bound local.
    pub fn with_pass(mut self, pass: impl ToTokens) -> Self {
        self.pass = Some(pass.to_token_stream());
        self
    }

    /// The conversion as a `Result<T, String>` expression, whatever its
    /// fallibility — for placing it where the error has to be inspected.
    pub fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }

    /// Several inputs side by side: their wires concatenated, their values
    /// handed to `combine` in order.
    pub fn combine(
        parts: Vec<Input>,
        combine: impl FnOnce(Vec<TokenStream>) -> TokenStream,
    ) -> Self {
        let fallible = parts.iter().any(|p| p.fallible);
        let mut wires = Vec::new();
        let mut exprs = Vec::new();
        for p in parts {
            wires.extend(p.wires);
            exprs.push(p.expr);
        }
        Self {
            wires,
            expr: combine(exprs),
            fallible,
            pass: None,
        }
    }

    /// `Some(inner)` when `present` holds, else `None`: an optional value
    /// that arrives as a presence test beside the inner value's own wires.
    /// `presence` is prepended to the wires.
    pub fn optional(presence: Option<Wire>, present: impl ToTokens, inner: Input) -> Self {
        let mut wires: Vec<Wire> = presence.into_iter().collect();
        wires.extend(inner.wires);
        let e = inner.expr;
        let present = present.to_token_stream();
        Self {
            wires,
            expr: quote!(if #present { ::core::option::Option::Some(#e) } else { ::core::option::Option::None }),
            fallible: inner.fallible,
            pass: None,
        }
    }
}

/// A source value → wires.
#[derive(Clone, Debug)]
pub struct Output {
    /// The wires produced, in order.
    pub wires: Vec<Wire>,
    /// Evaluates to the wire value, or a tuple of them in order when there
    /// are several (`()` for none).
    pub expr: TokenStream,
    /// Whether [`Self::expr`] uses `?` (on a `Result<_, String>`).
    pub fallible: bool,
}

impl Output {
    pub fn new(wires: Vec<Wire>, expr: impl ToTokens) -> Self {
        Self {
            wires,
            expr: expr.to_token_stream(),
            fallible: false,
        }
    }

    pub fn fallible(wires: Vec<Wire>, expr: impl ToTokens) -> Self {
        Self {
            fallible: true,
            ..Self::new(wires, expr)
        }
    }

    /// The single wire the value leaves on, built from `expr`.
    pub fn single(wire: Wire, expr: impl ToTokens) -> Self {
        Self::new(vec![wire], expr)
    }

    /// Nothing crosses: the value is dropped.
    pub fn none(value: impl ToTokens) -> Self {
        let v = value.to_token_stream();
        Self::new(Vec::new(), quote!({ let _ = #v; }))
    }

    /// `let <wires> = expr;` — binds every wire by its own name.
    pub fn bind(&self) -> TokenStream {
        let pat = self.pattern();
        let e = &self.expr;
        quote!(let #pat = #e;)
    }

    /// The binding pattern for this output's wires: `w`, `(a, b)`, or `()`.
    pub fn pattern(&self) -> TokenStream {
        let names = self.wires.iter().map(|w| &w.name);
        if self.wires.len() == 1 {
            let n = &self.wires[0].name;
            quote!(#n)
        } else {
            quote!((#(#names),*))
        }
    }

    /// The wire names as a value in the same shape [`Self::expr`] produces.
    pub fn values(&self) -> TokenStream {
        self.pattern()
    }

    /// The conversion as a `Result<_, String>` expression.
    pub fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }

    /// Several outputs side by side, flattened into one: each part is bound
    /// to its own wires, then all wires form one tuple. `prelude` runs first
    /// — typically a `let` destructuring the source value the parts read.
    pub fn concat(prelude: TokenStream, parts: Vec<Output>) -> Self {
        let fallible = parts.iter().any(|p| p.fallible);
        let binds: Vec<TokenStream> = parts.iter().map(Output::bind).collect();
        let wires: Vec<Wire> = parts.into_iter().flat_map(|p| p.wires).collect();
        let out = Output::new(wires, TokenStream::new());
        let values = out.values();
        Self {
            expr: quote!({ #prelude #(#binds)* #values }),
            ..out
        }
        .with_fallible(fallible)
    }

    fn with_fallible(mut self, f: bool) -> Self {
        self.fallible = f;
        self
    }
}

/// `expr` as a `Result<_, String>`: an immediately-invoked closure when it
/// uses `?`, an `Ok` otherwise.
pub fn result_expr(expr: &TokenStream, fallible: bool) -> TokenStream {
    if fallible {
        quote!((|| -> ::core::result::Result<_, ::std::string::String> {
            ::core::result::Result::Ok(#expr)
        })())
    } else {
        quote!(::core::result::Result::<_, ::std::string::String>::Ok(#expr))
    }
}
