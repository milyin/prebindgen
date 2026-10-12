//! Conversion composition. Adapters supply leaf codecs; wrappers consume prepared slots.
use prebindgen_flat::{same_type, type_key, Flat, Primitive, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

/// Inputs are recipes, not wire layouts. A selector denotes a logical choice.
#[derive(Clone, Debug)]
pub enum Input<L> {
    Scalar,
    Foreign(L),
    Text,
    Bytes,
    Fields(Vec<Input<L>>),
    Constructor {
        function: String,
        args: Vec<Input<L>>,
    },
    Selector(Vec<Input<L>>),
}
impl<L> Input<L> {
    pub fn fields(fields: impl IntoIterator<Item = Input<L>>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input<L>>) -> Self {
        Self::Constructor {
            function: name.into(),
            args: args.into_iter().collect(),
        }
    }
    pub fn selector(arms: impl IntoIterator<Item = Input<L>>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
#[derive(Clone, Debug)]
pub enum Output<L> {
    Scalar,
    /// The adapter owns the representation and ownership policy.
    Foreign(L),
    Text,
    Bytes,
    Fields(Vec<Output<L>>),
    /// A shared-borrow accessor, followed by the output conversion of its result.
    Getter {
        function: String,
        output: Box<Output<L>>,
    },
    Combine(Vec<Output<L>>),
}
impl<L> Output<L> {
    pub fn fields(fields: impl IntoIterator<Item = Output<L>>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    pub fn getter(name: impl Into<String>, output: Output<L>) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    pub fn combine(parts: impl IntoIterator<Item = Output<L>>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
    fn consumes<C: Codec<Leaf = L>>(&self, codec: &C) -> bool {
        match self {
            Self::Foreign(leaf) => codec.consumes(leaf),
            Self::Combine(p) => p.iter().any(|p| p.consumes(codec)),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Slot<L> {
    #[default]
    Empty,
    Input(Input<L>),
    Output(Output<L>),
}

/// A wire's role remains available to both sides, even when carriers coincide.
#[derive(Clone, Debug)]
pub enum WireKind<W> {
    Scalar(Primitive),
    Foreign(W),
    Text,
    Bytes,
    Selector,
}
#[derive(Clone, Debug)]
pub struct Wire<W> {
    pub name: syn::Ident,
    pub kind: WireKind<W>,
    pub active: Condition,
}

/// Language-specific terminal operations. Recursive composition stays in tools.
///
/// Foreign input leaves must validate without consuming in `validation`, then
/// acquire ownership without failure or source calls in `acquire`. Each acquired
/// value needs an ordinary Rust owner so later failures drop it. `statements`
/// can construct arguments from those owners. Every foreign wire must retain
/// the supplied activation condition. Output leaves keep ownership in Rust
/// locals until their infallible `commit` runs after successful delivery.
pub trait Codec {
    type Leaf: Clone + std::fmt::Debug;
    type Wire: Clone + std::fmt::Debug;
    /// Explicit identity and unexpanded domain types are adapter policy.
    fn identity(&self, ty: &Type) -> Result<Self::Leaf>;
    fn consumes(&self, leaf: &Self::Leaf) -> bool;
    fn input_foreign(
        &self,
        ty: &Type,
        leaf: &Self::Leaf,
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<PreparedInput<Self::Wire>>;
    fn output_foreign(
        &self,
        ty: &Type,
        leaf: &Self::Leaf,
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<PreparedOutput<Self::Wire>>;
    fn decode_scalar(&self, primitive: Primitive, wire: &syn::Ident) -> TokenStream;
    fn encode_scalar(&self, primitive: Primitive, value: TokenStream) -> TokenStream;
    fn decode_text(&self, wire: &syn::Ident, ty: &Type) -> TokenStream;
    fn decode_bytes(&self, wire: &syn::Ident, ty: &Type) -> TokenStream;
}
#[derive(Clone, Debug)]
pub struct PreparedInput<W> {
    pub wires: Vec<Wire<W>>,
    /// Validate and prepare every active leaf before any ownership transfer.
    pub validation: TokenStream,
    /// Infallible acquisition of all owned inputs, before constructors can run.
    pub acquire: TokenStream,
    pub resources: Vec<Resource>,
    pub statements: TokenStream,
    pub argument: TokenStream,
}
#[derive(Clone, Debug)]
pub struct PreparedOutput<W> {
    pub wires: Vec<Wire<W>>,
    /// Transfer ownership only after all foreign delivery operations succeed.
    pub commit: TokenStream,
    pub statements: TokenStream,
    pub values: Vec<TokenStream>,
}
#[derive(Clone, Debug, Default)]
pub enum Prepared<W> {
    #[default]
    Empty,
    Input(PreparedInput<W>),
    Output(PreparedOutput<W>),
}

/// Fill slots on the same flat structure, rejecting incomplete bound functions.
/// A function is exported iff its result has an output slot, including `fields([])` for unit.
pub fn prepare<C: Codec>(flat: Flat<Slot<C::Leaf>>, codec: &C) -> Result<Flat<Prepared<C::Wire>>> {
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
            Slot::Input(recipe) => {
                Prepared::Input(compiler.input(ty, &recipe, &Condition::default())?)
            }
            Slot::Output(recipe) => {
                Prepared::Output(compiler.output(ty, &recipe, quote!(__result), false)?)
            }
        })
    })
}
struct Compiler<'a, C: Codec> {
    flat: &'a Flat<Slot<C::Leaf>>,
    codec: &'a C,
    next: usize,
}
impl<C: Codec> Compiler<'_, C> {
    fn name(&mut self) -> syn::Ident {
        let n = format_ident!("w{}", self.next);
        self.next += 1;
        n
    }
    fn input(
        &mut self,
        ty: &Type,
        recipe: &Input<C::Leaf>,
        active: &Condition,
    ) -> Result<PreparedInput<C::Wire>> {
        let var = self.name();
        let mut wires = Vec::new();
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let condition = active.rust();
        let mut statements = TokenStream::new();
        let mut argument = quote!(#var);
        match recipe {
            Input::Scalar => {
                let primitive =
                    Primitive::from_type(ty).ok_or("scalar input needs a primitive type")?;
                let w = self.name();
                let decode = self.codec.decode_scalar(primitive, &w);
                wires.push(Wire {
                    name: w,
                    kind: WireKind::Scalar(primitive),
                    active: active.clone(),
                });
                validation = quote!(let #var = if #condition {Some(#decode)} else {None};);
                statements = quote!(let #var:#ty=#var.expect("active scalar"););
            }
            Input::Foreign(leaf) => {
                let w = self.name();
                return self.codec.input_foreign(ty, leaf, w, var, active);
            }
            Input::Text | Input::Bytes => {
                let text = matches!(recipe, Input::Text);
                if matches!(ty, Type::Reference(r) if r.mutability.is_some()) {
                    return Err(
                        "mutable text/byte views require an adapter-specific converter".into(),
                    );
                }
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
                    active: active.clone(),
                    kind: if text {
                        WireKind::Text
                    } else {
                        WireKind::Bytes
                    },
                });
                let decode = if text {
                    self.codec.decode_text(&w, ty)
                } else {
                    self.codec.decode_bytes(&w, ty)
                };
                validation = quote!(let #var = if #condition {Some(#decode)} else {None};);
                statements = quote!(let #var=#var.expect("active buffer"););
                if borrowed {
                    argument = quote!(&#var);
                }
            }
            Input::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(PreparedInput {
                        wires,
                        validation,
                        acquire,
                        resources,
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
                    let p = self.input(&field.ty.ty, child, active)?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
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
                    let p = self.input(&param.ty.ty, child, active)?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
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
                    active: active.clone(),
                });
                let count = arms.len() as i32;
                let invalid = active.and(quote!(!(0..#count).contains(&#tag)));
                validation.extend(quote!(if #invalid {return Err("invalid selector".into());}));
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let p = self.input(ty, arm, &active.arm(tag.clone(), i as i32))?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    let body = p.statements;
                    let value = p.argument;
                    wires.extend(p.wires);
                    let i = i as i32;
                    cases.push(quote!(#i=>{#body #value}));
                }
                statements = quote!(let #var:#ty=match #tag{#(#cases,)* _=>unreachable!("validated selector")};);
            }
        }
        Ok(PreparedInput {
            wires,
            validation,
            acquire,
            resources,
            statements,
            argument,
        })
    }
    fn output(
        &mut self,
        ty: &Type,
        recipe: &Output<C::Leaf>,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<PreparedOutput<C::Wire>> {
        let mut out = PreparedOutput {
            commit: TokenStream::new(),
            wires: Vec::new(),
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Scalar => {
                let primitive =
                    Primitive::from_type(ty).ok_or("scalar output needs a primitive type")?;
                let value = if borrowed {
                    let reference = self.name();
                    out.statements
                        .extend(quote!(let #reference: &#ty = #value;));
                    quote!(*#reference)
                } else {
                    value
                };
                let expr = self.codec.encode_scalar(primitive, value);
                let w = self.name();
                out.statements.extend(quote!(let #w=#expr;));
                out.values.push(quote!(#w));
                out.wires.push(Wire {
                    name: w,
                    kind: WireKind::Scalar(primitive),
                    active: Condition::default(),
                });
            }
            Output::Foreign(leaf) => {
                let w = self.name();
                return self.codec.output_foreign(ty, leaf, w, value, borrowed);
            }
            Output::Text | Output::Bytes => {
                let text = matches!(recipe, Output::Text);
                if type_key(ty) != if text { "String" } else { "Vec < u8 >" } {
                    return Err(format!("wrong output type `{}`", type_key(ty)));
                }
                let expr = if borrowed {
                    quote!(::core::clone::Clone::clone(#value))
                } else {
                    value
                };
                let w = self.name();
                out.statements = quote!(let #w = #expr;);
                out.values.push(quote!(#w));
                out.wires.push(Wire {
                    name: w,
                    kind: if text {
                        WireKind::Text
                    } else {
                        WireKind::Bytes
                    },
                    active: Condition::default(),
                });
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
                if parts.iter().filter(|p| p.consumes(self.codec)).count() > 1 {
                    return Err("a combination cannot consume the same value twice".into());
                }
                let mut bodies = Vec::new();
                for child in parts {
                    let consuming = child.consumes(self.codec);
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
fn append<W>(to: &mut PreparedOutput<W>, from: PreparedOutput<W>) {
    to.commit.extend(from.commit);
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

/// A leaf is active only on its selected path. Adapters use the same condition
/// for foreign-side resource locking and native preparation.
#[derive(Clone, Debug, Default)]
pub struct Condition(pub Vec<(syn::Ident, i32)>);
impl Condition {
    pub fn arm(&self, tag: syn::Ident, index: i32) -> Self {
        let mut result = self.clone();
        result.0.push((tag, index));
        result
    }
    pub fn and(&self, predicate: TokenStream) -> TokenStream {
        if self.0.is_empty() {
            predicate
        } else {
            let condition = self.rust();
            quote!((#condition) && (#predicate))
        }
    }
    pub fn rust(&self) -> TokenStream {
        self.0
            .iter()
            .map(|(tag, i)| quote!(#tag == #i))
            .reduce(|a, b| quote!((#a) && (#b)))
            .unwrap_or_else(|| quote!(true))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Shared,
    Exclusive,
    Move,
}
/// The adapter supplies identity; tools only enforce call-wide access conflicts.
#[derive(Clone, Debug)]
pub struct Resource {
    pub identity: TokenStream,
    pub access: Access,
    pub active: Condition,
}
pub fn validate_resources(resources: &[Resource]) -> TokenStream {
    let mut result = TokenStream::new();
    for (i, a) in resources.iter().enumerate() {
        for b in &resources[i + 1..] {
            if a.access == Access::Shared && b.access == Access::Shared {
                continue;
            }
            let x = &a.identity;
            let y = &b.identity;
            // Different alternatives of the same selector cannot conflict.
            if a.active
                .0
                .iter()
                .any(|(tag, arm)| b.active.0.iter().any(|(t, i)| t == tag && i != arm))
            {
                continue;
            }
            let mut active = a.active.clone();
            for term in &b.active.0 {
                if !active.0.contains(term) {
                    active.0.push(term.clone());
                }
            }
            let conflict = active.and(quote!((#x) == (#y)));
            result.extend(quote!(if #conflict {return Err("conflicting input aliases".into());}));
        }
    }
    result
}

/// Reusable storage-specific conversions; not a required object model.
pub mod boxed;
