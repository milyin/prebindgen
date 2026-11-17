//! A public-API conversion compiler with a deliberately small Rust boundary.
use prebindgen_conversions::{
    staged::emit_call, value, FunctionConversions, InputPlan, OutputPlan,
};
use prebindgen_declarations::{
    compile_call, Codec, CodecInputPlan, CodecOutputPlan, Compiler, Condition, GroupKind,
    InputLeaf, InputNode, InputOps, OutputLeaf, OutputNode, OutputOps, Representation,
    ResolvedCall, ResolvedInput, ResolvedOutput, ResolvedSlot, ValueAccess,
};
use prebindgen_flat::{same_type, Flat, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{convert::Infallible, path::Path, process::Command};
use syn::{Ident, Type};

#[derive(Clone, Debug)]
struct Incoming;
#[derive(Clone, Debug)]
struct Outgoing;
#[derive(Clone, Debug)]
enum InputUnary {
    Optional,
}
impl InputOps for Incoming {
    type Leaf = ();
    type Unary = InputUnary;
    type Branch = Infallible;
    type Arrow = Infallible;
}
impl OutputOps for Outgoing {
    type Leaf = ();
    type Unary = Infallible;
    type Branch = Infallible;
    type Arrow = Infallible;
}
type Input = InputNode<Incoming, Outgoing>;
type Output = OutputNode<Incoming, Outgoing>;

struct Scalars;
impl Codec for Scalars {
    type InOps = Incoming;
    type OutOps = Outgoing;
    type CompileState = ();
    // Source attachments are unit, independently of our prepared plan types.
    type Schema = ();
    type Input = InputPlan<Type>;
    type Output = OutputPlan<Type>;
    type Context = ();
    type Extra = ();

    fn compile_input_tree(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedInput<Self>,
        extra: &(),
        active: &Condition,
        names: Option<(&Ident, &Ident)>,
    ) -> Result<CodecInputPlan<Self>> {
        match tree {
            Input::None => cx.input_unit(ty),
            Input::Leaf(()) => self.input_leaf(InputLeaf::new(
                cx.flat,
                ty,
                extra,
                names.map_or_else(|| cx.name(), |(wire, _)| wire.clone()),
                names.map_or_else(|| cx.name(), |(_, target)| target.clone()),
                active,
            )),
            Input::Unary {
                operation: InputUnary::Optional,
                child,
            } => {
                // Composition is shared. Without input_optional(), the codec's
                // default rejects the encoding instead of accepting a fallback.
                cx.input_optional(ty, extra, active, names, cx.name(), child)
            }
            Input::Branch { operation, .. } | Input::Arrow { operation, .. } => match *operation {},
        }
    }

    fn compile_output_tree(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedOutput<Self>,
        extra: &(),
        source: TokenStream,
        borrowed: bool,
        wire: Option<&Ident>,
    ) -> Result<CodecOutputPlan<Self>> {
        match tree {
            Output::None => cx.output_unit(ty, cx.empty_output()?),
            Output::Leaf(()) => self.output_leaf(OutputLeaf::new(
                cx.flat,
                ty,
                extra,
                wire.cloned().unwrap_or_else(|| cx.name()),
                source,
                if borrowed {
                    ValueAccess::Shared
                } else {
                    ValueAccess::Owned
                },
            )),
            Output::Unary { operation, .. }
            | Output::Branch { operation, .. }
            | Output::Arrow { operation, .. } => match *operation {},
        }
    }

    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInputPlan<Self>> {
        require_scalar(r.ty, r.representation)?;
        let wire = &r.wire;
        value::input(r.ty, r.wire.clone(), r.target, quote!(#wire), r.ty.clone())
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutputPlan<Self>> {
        require_scalar(r.ty, r.representation)?;
        value::output(
            r.ty,
            r.wire,
            r.value,
            r.access.is_shared(),
            |v| v,
            r.ty.clone(),
        )
    }
    fn output_consumes(&self, _: &Flat, _: &Type, _: &()) -> bool {
        false // i64 is copied; no allocation is transferred.
    }
    fn input_group(&self, _: GroupKind) -> Result<()> {
        Ok(()) // Unit needs an empty field group.
    }
    fn output_group(&self, _: GroupKind) -> Result<()> {
        Ok(()) // Root composition and unit need empty groups.
    }
    fn input_field(&self, _: &Ident) -> Result<()> {
        Err("source record fields are unsupported by this adapter".into())
    }
    fn output_field(&self, _: &Ident) -> Result<()> {
        Err("source record fields are unsupported by this adapter".into())
    }
    fn finish_output(&self, plan: &mut CodecOutputPlan<Self>, _: &()) -> Result<()> {
        // The default compile_output_root finalizes once, including unit results.
        // A scalar returns its one wire; unit returns no wire and evaluates ().
        plan.delivery.result = match plan.values.as_slice() {
            [] => quote!(()),
            [(_, value)] => value.clone(),
            _ => return Err("multiple result wires are unsupported by this adapter".into()),
        };
        Ok(())
    }
}

fn require_scalar(ty: &Type, representation: Representation) -> Result<()> {
    if representation != Representation::Ordinary || !same_type(ty, &syn::parse_quote!(i64)) {
        return Err("only ordinary i64 scalar leaves are supported by this adapter".into());
    }
    Ok(())
}

fn bridge(flat: &Flat, name: &str, inputs: Vec<Input>, output: Output) -> Result<TokenStream> {
    let resolved = ResolvedCall {
        inputs: inputs.into_iter().map(ResolvedSlot::Tree).collect(),
        output: ResolvedSlot::Tree(output),
    };
    let prepared = compile_call(flat, &Scalars, name, resolved, &())?;
    let source = flat.function(name)?;
    let export = format_ident!("bridge_{name}");
    // The prepared call borrows the authoritative signature, with no slot edits.
    let view = prepared.view(source, &export, &())?;
    let wires: Vec<_> = view.input_wire_definitions()?.collect();
    let names: Vec<_> = wires.iter().map(|wire| wire.name.clone()).collect();
    let params = wires.iter().map(|wire| {
        let name = &wire.name;
        let ty = wire.rust_type();
        quote!(#name: #ty)
    });
    let result = &source.ret.ty;
    let body = emit_call(&view, &(), &format_ident!("_source_result"), result, &names)?;
    Ok(quote! {
        fn #export(#(#params),*) -> Result<#result, String> { #body }
    })
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("usage: adapter-minimal <output-directory>")?;
    let output = Path::new(&output);
    std::fs::create_dir_all(output)?;
    let source = syn::parse_file(
        "pub fn add(a:i64,b:i64)->i64 { a+b }
         pub fn observe(value:i64) { assert_eq!(value,42); }
         pub fn unit(value:()) { assert_eq!(value,()); }
         pub fn maybe(value:Option<i64>)->i64 { value.unwrap_or(0) }",
    )?;
    let flat = Flat::parse(
        source
            .items
            .iter()
            .cloned()
            .map(|item| (item, Default::default())),
    )?;
    let add = bridge(
        &flat,
        "add",
        vec![Input::Leaf(()), Input::Leaf(())],
        Output::Leaf(()),
    )?;
    let observe = bridge(&flat, "observe", vec![Input::Leaf(())], Output::None)?;
    let unit = bridge(&flat, "unit", vec![Input::None], Output::None)?;
    let rejection = bridge(
        &flat,
        "maybe",
        vec![Input::Unary {
            operation: InputUnary::Optional,
            child: Box::new(Input::Leaf(())),
        }],
        Output::Leaf(()),
    )
    .expect_err("optional input encoding must remain unsupported");
    assert!(
        rejection.contains("optional conversion is unsupported by this codec"),
        "{rejection}"
    );
    println!("expected rejection for maybe's optional input: {rejection}");

    let generated = output.join("generated.rs");
    std::fs::write(
        &generated,
        prebindgen_conversions::rust_source(quote! {
            #source
            #add
            #observe
            #unit
            fn main() {
                assert_eq!(bridge_add(20, 22).unwrap(), 42);
                bridge_observe(42).unwrap();
                bridge_unit().unwrap();
                println!("generated scalar/unit assertions passed");
            }
        })?,
    )?;
    let binary = output.join(format!("generated{}", std::env::consts::EXE_SUFFIX));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    if !Command::new(rustc)
        .arg("--edition=2021")
        .arg(&generated)
        .arg("-o")
        .arg(&binary)
        .status()?
        .success()
    {
        return Err("generated Rust compilation failed".into());
    }
    if !Command::new(binary).status()?.success() {
        return Err("generated Rust assertions failed".into());
    }
    Ok(())
}
