use std::collections::HashSet;

use prebindgen_flat::flat::TypeRef;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{names, Form, Input, Output, Wire, WireType};

/// Direction relative to the flat Rust value, independently of its location.
///
/// A callback argument usually leaves Rust, even though it is a parameter.
/// Defaults and overrides are keyed by this direction, not by parameter/return.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Destination value → wires → flat Rust value.
    IntoRust,
    /// Flat Rust value → wires → destination value.
    OutOfRust,
}

/// A selected conversion for one occurrence of a flat type.
///
/// `W` is the generator's closed wire enum; `M` describes the destination
/// value as a whole (a class, constructor, builder, or an adapter-owned tree).
/// The [`Form`] records shared conversion structure; neither writer needs to
/// look up defaults again. Input and output plans are resolved independently.
///
/// Constructors validate wire-name uniqueness. The resolver also validates
/// the root type and direction. These checks validate the plan's structure,
/// not arbitrary Rust token semantics or the existence of trait impls: compile
/// generated Rust to verify those.
#[derive(Clone, Debug)]
pub struct ConversionPlan<W, M = ()> {
    code: Code<W>,
    metadata: M,
}

#[derive(Clone, Debug)]
enum Code<W> {
    Input {
        conversion: Input<W>,
        setup: TokenStream,
    },
    Output {
        conversion: Output<W>,
        source: syn::Ident,
    },
}

impl<W: WireType, M> ConversionPlan<W, M> {
    /// Select a wires-to-Rust conversion with destination metadata.
    pub fn input(conversion: Input<W>, metadata: M) -> Result<Self, String> {
        Self::input_with_setup(conversion, TokenStream::new(), metadata)
    }

    /// Select an input that needs backing storage or a runtime guard.
    ///
    /// `setup` runs once before the converted value is bound. It may use `?`
    /// in the enclosing error scope. Storage and RAII guards declared here
    /// remain alive through the body passed to [`Self::with_input`], including
    /// its failure paths. The conversion expression may borrow that storage.
    ///
    /// This iteration supports setup at the plan root. Conditional backing
    /// storage inside optional/choice children and explicit non-RAII cleanup
    /// are left to the generator; tools does not synthesize them yet.
    pub fn input_with_setup(
        conversion: Input<W>,
        setup: TokenStream,
        metadata: M,
    ) -> Result<Self, String> {
        check_wires(&conversion.form)?;
        Ok(Self {
            code: Code::Input { conversion, setup },
            metadata,
        })
    }

    /// Select a Rust-to-wires conversion.
    ///
    /// Its expression reads the flat value through `source`. [`Self::emit_output`]
    /// binds that local once, before evaluating the conversion.
    pub fn output(source: syn::Ident, conversion: Output<W>, metadata: M) -> Result<Self, String> {
        check_wires(&conversion.form)?;
        Ok(Self {
            code: Code::Output { conversion, source },
            metadata,
        })
    }

    /// The original flat type at this occurrence, including borrowing.
    pub fn source_type(&self) -> &TypeRef {
        &self.form().ty
    }

    /// The direction this plan implements.
    pub fn direction(&self) -> Direction {
        match self.code {
            Code::Input { .. } => Direction::IntoRust,
            Code::Output { .. } => Direction::OutOfRust,
        }
    }

    /// The shared relation tree, already selected for this occurrence.
    pub fn form(&self) -> &Form<W> {
        match &self.code {
            Code::Input { conversion, .. } => &conversion.form,
            Code::Output { conversion, .. } => &conversion.form,
        }
    }

    /// Boundary slots in conversion order; zero, one, or several.
    pub fn wires(&self) -> Vec<&Wire<W>> {
        self.form().wires()
    }

    /// Destination-language meaning of the converted value as a whole.
    ///
    /// Per-slot semantics belong in `W`; structural/class/builder information
    /// belongs here. An adapter can retain child metadata in its own `M` tree.
    pub fn metadata(&self) -> &M {
        &self.metadata
    }

    /// Consume a selected child for an input combinator, retaining its metadata.
    ///
    /// A child with setup storage cannot be flattened by the current expression
    /// combinators; emit it through `with_input` at the element level instead.
    pub fn into_input(self) -> Result<(Input<W>, M), String> {
        match self.code {
            Code::Input { conversion, setup } if setup.is_empty() => {
                Ok((conversion, self.metadata))
            }
            Code::Input { .. } => {
                Err("input setup requires with_input; child setup composition is deferred".into())
            }
            Code::Output { .. } => Err("into_input requires an IntoRust plan".into()),
        }
    }

    /// Consume a selected child for an output combinator, retaining its metadata.
    ///
    /// Bind the child's flat `value` once to the source local its policy chose.
    pub fn into_output(self, value: impl ToTokens) -> Result<(Output<W>, M), String> {
        match self.code {
            Code::Output {
                mut conversion,
                source,
            } => {
                let (value, expr) = (value.to_token_stream(), &conversion.expr);
                conversion.expr = quote!({ let #source = #value; #expr });
                Ok((conversion, self.metadata))
            }
            Code::Input { .. } => Err("into_output requires an OutOfRust plan".into()),
        }
    }

    /// Emit setup, bind the converted input once, and use it within that scope.
    ///
    /// The enclosing generator supplies the Rust error scope. This deliberately
    /// does not put the conversion in a separate `Result` closure: a borrowed
    /// value must remain with its storage through the flat call. `body` should
    /// consume the borrow within this scope, rather than return it.
    /// Names for `local`, setup locals, and wires must be distinct.
    /// Returns an error when called on an output plan.
    pub fn with_input(
        &self,
        local: &syn::Ident,
        body: impl FnOnce(TokenStream) -> TokenStream,
    ) -> Result<TokenStream, String> {
        let Code::Input { conversion, setup } = &self.code else {
            return Err("with_input requires an IntoRust plan".into());
        };
        let expr = &conversion.expr;
        let body = body(local.to_token_stream());
        Ok(quote!({ #setup let #local = #expr; #body }))
    }

    /// Emit a conversion that evaluates `value` once and produces its wires.
    ///
    /// Zero wires produce `()`, one produces a bare value, several a tuple.
    /// `?` propagates to the enclosing generator's error scope. The source
    /// local shadows names only inside this block, after `value` is evaluated.
    /// Returns an error when called on an input plan.
    pub fn emit_output(&self, value: impl ToTokens) -> Result<TokenStream, String> {
        let Code::Output { conversion, source } = &self.code else {
            return Err("emit_output requires an OutOfRust plan".into());
        };
        let (value, expr) = (value.to_token_stream(), &conversion.expr);
        Ok(quote!({ let #source = #value; #expr }))
    }
}

fn check_wires<W>(form: &Form<W>) -> Result<(), String> {
    let mut names = HashSet::new();
    for wire in form.wires() {
        if !names.insert(names::bare(&wire.name)) {
            return Err(format!("duplicate wire `{}`", wire.name));
        }
    }
    Ok(())
}
