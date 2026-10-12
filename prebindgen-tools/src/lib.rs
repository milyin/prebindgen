//! Conversion composition. Adapters supply leaf codecs; wrappers consume prepared slots.
use prebindgen_flat::{same_type, type_key, Flat, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

/// Inputs are recipes, not wire layouts. A selector denotes a logical choice.
#[derive(Clone, Debug)]
pub enum Input {
    Handle,
    Text,
    Bytes,
    Fields(Vec<Input>),
    Constructor { function: String, args: Vec<Input> },
    Selector(Vec<Input>),
}
impl Input {
    pub fn fields(fields: impl IntoIterator<Item = Input>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input>) -> Self {
        Self::Constructor {
            function: name.into(),
            args: args.into_iter().collect(),
        }
    }
    pub fn selector(arms: impl IntoIterator<Item = Input>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
#[derive(Clone, Debug)]
pub enum Output {
    /// Transfer this owned value into a handle.
    Handle,
    /// Make an independent handle from a borrowed value. Requires Rust `Clone`.
    ClonedHandle,
    Text,
    Bytes,
    Fields(Vec<Output>),
    /// A shared-borrow accessor, followed by the output conversion of its result.
    Getter {
        function: String,
        output: Box<Output>,
    },
    Combine(Vec<Output>),
}
impl Output {
    pub fn fields(fields: impl IntoIterator<Item = Output>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    pub fn getter(name: impl Into<String>, output: Output) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    pub fn combine(parts: impl IntoIterator<Item = Output>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
    fn consumes(&self) -> bool {
        match self {
            Self::Handle => true,
            Self::Combine(p) => p.iter().any(Self::consumes),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub enum Slot {
    #[default]
    Empty,
    Input(Input),
    Output(Output),
}

/// A wire's role remains available to both sides, even when carriers coincide.
#[derive(Clone, Debug)]
pub enum WireKind {
    Handle(Box<Type>),
    Text,
    Bytes,
    Selector,
}
#[derive(Clone, Debug)]
pub struct Wire {
    pub name: syn::Ident,
    pub kind: WireKind,
}

/// Language-specific terminal operations. Recursive composition stays in tools.
pub trait Codec {
    fn decode_text(&self, wire: &syn::Ident) -> TokenStream;
    fn decode_bytes(&self, wire: &syn::Ident) -> TokenStream;
}
#[derive(Clone, Debug)]
pub struct PreparedInput {
    pub wires: Vec<Wire>,
    pub statements: TokenStream,
    pub argument: TokenStream,
}
#[derive(Clone, Debug)]
pub struct PreparedOutput {
    pub wires: Vec<Wire>,
    pub statements: TokenStream,
    pub values: Vec<TokenStream>,
}
#[derive(Clone, Debug, Default)]
pub enum Prepared {
    #[default]
    Empty,
    Input(PreparedInput),
    Output(PreparedOutput),
}

/// Fill slots on the same flat structure, rejecting incomplete bound functions.
/// A function is exported iff its result has an output slot, including `fields([])` for unit.
pub fn prepare(flat: Flat<Slot>, codec: &impl Codec) -> Result<Flat<Prepared>> {
    for f in flat.functions.values() {
        match &f.ret.slot {
            Slot::Output(_) => {
                for p in &f.params {
                    if !matches!(p.ty.slot, Slot::Input(_)) {
                        return Err(format!(
                            "`{}`: parameter `{}` has no input converter",
                            f.name, p.name
                        ));
                    }
                }
            }
            Slot::Empty if f.params.iter().all(|p| matches!(p.ty.slot, Slot::Empty)) => {}
            _ => {
                return Err(format!(
                    "`{}`: incomplete or wrong-direction conversion slots",
                    f.name
                ))
            }
        }
    }
    let source = flat.clone();
    let mut compiler = Compiler {
        flat: &source,
        codec,
        next: 0,
    };
    flat.map_slots(|ty, slot| {
        Ok(match slot {
            Slot::Empty => Prepared::Empty,
            Slot::Input(recipe) => Prepared::Input(compiler.input(ty, &recipe)?),
            Slot::Output(recipe) => {
                Prepared::Output(compiler.output(ty, &recipe, quote!(__result), false)?)
            }
        })
    })
}
struct Compiler<'a, C> {
    flat: &'a Flat<Slot>,
    codec: &'a C,
    next: usize,
}
impl<C: Codec> Compiler<'_, C> {
    fn name(&mut self) -> syn::Ident {
        let n = format_ident!("w{}", self.next);
        self.next += 1;
        n
    }
    fn input(&mut self, ty: &Type, recipe: &Input) -> Result<PreparedInput> {
        let var = self.name();
        let mut wires = Vec::new();
        let mut statements = TokenStream::new();
        let mut argument = quote!(#var);
        match recipe {
            Input::Handle => {
                if matches!(ty, Type::Reference(_)) {
                    return Err("handle inputs in this subset transfer ownership; borrowed handles are unsupported".into());
                }
                let w = self.name();
                wires.push(Wire {
                    name: w.clone(),
                    kind: WireKind::Handle(Box::new(ty.clone())),
                });
                statements = quote!(let #var:#ty=::prebindgen_runtime::take::<#ty>(#w as u64)?;);
            }
            Input::Text | Input::Bytes => {
                let text = matches!(recipe, Input::Text);
                let (core, borrowed) = match ty {
                    Type::Reference(r) => (&*r.elem, true),
                    _ => (ty, false),
                };
                let key = type_key(core);
                if !(if text {
                    key == "String" || (borrowed && key == "str")
                } else {
                    key == "Vec < u8 >" || (borrowed && key == "[u8]")
                }) {
                    return Err(format!(
                        "{} input cannot produce `{}`",
                        if text { "text" } else { "bytes" },
                        type_key(ty)
                    ));
                }
                let w = self.name();
                wires.push(Wire {
                    name: w.clone(),
                    kind: if text {
                        WireKind::Text
                    } else {
                        WireKind::Bytes
                    },
                });
                let decode = if text {
                    self.codec.decode_text(&w)
                } else {
                    self.codec.decode_bytes(&w)
                };
                statements = quote!(let #var=#decode;);
                if borrowed {
                    argument = quote!(&#var);
                }
            }
            Input::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(PreparedInput {
                        wires,
                        statements,
                        argument: quote!(()),
                    });
                }
                let r = self.flat.record(ty)?.clone();
                if r.fields.len() != children.len() {
                    return Err(format!(
                        "`{}`: expected {} field converters, got {}",
                        type_key(ty),
                        r.fields.len(),
                        children.len()
                    ));
                }
                let mut fields = Vec::new();
                for (field, child) in r.fields.iter().zip(children) {
                    let p = self.input(&field.ty.ty, child)?;
                    statements.extend(p.statements);
                    wires.extend(p.wires);
                    let name = &field.name;
                    let arg = p.argument;
                    fields.push(quote!(#name:#arg));
                }
                statements.extend(quote!(let #var:#ty=#ty{#(#fields),*};));
            }
            Input::Constructor { function, args } => {
                let f = self.flat.function(function)?.clone();
                if !same_type(&f.ret.ty, ty) {
                    return Err(format!(
                        "constructor `{function}` returns `{}`, expected `{}`",
                        type_key(&f.ret.ty),
                        type_key(ty)
                    ));
                }
                if f.params.len() != args.len() {
                    return Err(format!("constructor `{function}`: wrong argument count"));
                }
                let mut call_args = Vec::new();
                for (param, child) in f.params.iter().zip(args) {
                    let p = self.input(&param.ty.ty, child)?;
                    statements.extend(p.statements);
                    wires.extend(p.wires);
                    call_args.push(p.argument);
                }
                let path = &f.path;
                statements.extend(quote!(let #var:#ty=#path(#(#call_args),*);));
            }
            Input::Selector(arms) => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                if matches!(ty, Type::Reference(_)) {
                    return Err("selector results must be owned in this subset".into());
                }
                let tag = self.name();
                wires.push(Wire {
                    name: tag.clone(),
                    kind: WireKind::Selector,
                });
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let p = self.input(ty, arm)?;
                    let body = p.statements;
                    let value = p.argument;
                    wires.extend(p.wires);
                    let i = i as i32;
                    cases.push(quote!(#i=>{#body #value}));
                }
                statements = quote!(let #var:#ty=match #tag{#(#cases,)* _=>return Err("invalid selector".into())};);
            }
        }
        Ok(PreparedInput {
            wires,
            statements,
            argument,
        })
    }
    fn output(
        &mut self,
        ty: &Type,
        recipe: &Output,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<PreparedOutput> {
        let mut out = PreparedOutput {
            wires: Vec::new(),
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Handle | Output::ClonedHandle | Output::Text | Output::Bytes => {
                let w = self.name();
                let (kind, expr) = match recipe {
                    Output::Handle => {
                        if borrowed {
                            return Err(
                                "cannot move a borrowed field into a handle; use ClonedHandle"
                                    .into(),
                            );
                        }
                        (
                            WireKind::Handle(Box::new(ty.clone())),
                            quote!(__outputs.insert(#value)?),
                        )
                    }
                    Output::ClonedHandle => {
                        let v = if borrowed { value } else { quote!(&#value) };
                        (
                            WireKind::Handle(Box::new(ty.clone())),
                            quote!(__outputs.insert(::core::clone::Clone::clone(#v))?),
                        )
                    }
                    Output::Text | Output::Bytes => {
                        let text = matches!(recipe, Output::Text);
                        if type_key(ty) != if text { "String" } else { "Vec < u8 >" } {
                            return Err(format!("wrong output type `{}`", type_key(ty)));
                        }
                        let v = if borrowed { value } else { quote!(&#value) };
                        (
                            if text {
                                WireKind::Text
                            } else {
                                WireKind::Bytes
                            },
                            quote!(::core::clone::Clone::clone(#v)),
                        )
                    }
                    _ => unreachable!(),
                };
                out.statements = quote!(let #w=#expr;);
                out.values.push(quote!(#w));
                out.wires.push(Wire { name: w, kind });
            }
            Output::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(out);
                }
                let r = self.flat.record(ty)?.clone();
                if r.fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                for (field, child) in r.fields.iter().zip(children) {
                    let name = &field.name;
                    let p = self.output(&field.ty.ty, child, quote!(&(#value).#name), true)?;
                    append(&mut out, p);
                }
            }
            Output::Getter { function, output } => {
                let f = self.flat.function(function)?.clone();
                let expected: Type = syn::parse_quote!(&#ty);
                if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                    return Err(format!(
                        "getter `{function}` must take exactly `&{}`",
                        type_key(ty)
                    ));
                }
                if matches!(f.ret.ty, Type::Reference(_)) {
                    return Err("borrowed getter results are outside this subset".into());
                }
                let v = self.name();
                let path = &f.path;
                let arg = if borrowed { value } else { quote!(&#value) };
                out.statements = quote!(let #v=#path(#arg););
                append(&mut out, self.output(&f.ret.ty, output, quote!(#v), false)?);
            }
            Output::Combine(parts) => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                if parts.iter().filter(|p| p.consumes()).count() > 1 {
                    return Err("a combination cannot consume the same value twice".into());
                }
                let mut bodies = Vec::new();
                for child in parts {
                    let consuming = child.consumes();
                    let arg = if borrowed || consuming {
                        value.clone()
                    } else {
                        quote!(&#value)
                    };
                    let mut p = self.output(ty, child, arg, borrowed || !consuming)?;
                    bodies.push((consuming, std::mem::take(&mut p.statements)));
                    append(&mut out, p);
                }
                // Wire order follows the declaration. Observations execute before the move.
                bodies.sort_by_key(|(consumes, _)| *consumes);
                for (_, body) in bodies {
                    out.statements.extend(body);
                }
            }
        }
        Ok(out)
    }
}
fn append(to: &mut PreparedOutput, from: PreparedOutput) {
    to.wires.extend(from.wires);
    to.statements.extend(from.statements);
    to.values.extend(from.values);
}

/// Produce formatted Rust, rejecting malformed generated syntax before writing.
pub fn rust_source(tokens: TokenStream) -> Result<String> {
    let file = syn::parse2(tokens).map_err(|e| format!("generated Rust: {e}"))?;
    Ok(prettyplease::unparse(&file))
}

/// Source names used in declarations are deliberately small and explicit.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

#[cfg(test)]
mod tests;

pub mod declarations;
pub use declarations::{
    Declarations, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl, FunctionDecl,
};
