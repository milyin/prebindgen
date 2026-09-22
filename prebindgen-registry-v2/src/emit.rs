//! The common Rust writer: one component, both targets.
//!
//! It owns the shape of the generated Rust — the locals and their order, the
//! branches on failure, the construction of the source value, the call, the
//! return — and allocates every temporary from the plan, so two operations
//! rendered into one wrapper cannot collide over a name. A target contributes
//! one expression per operation of its own ([`Target::render_operation`]) and
//! whole [`Artifact`]s; it contributes no control flow.
//!
//! Source types and struct shapes are generated through Flat's emission
//! capability, never from retained syntax and never spelled by an adapter.

use std::collections::HashMap;

use prebindgen_flat::{flat::Flat, Conditioned, RustEmitter};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    body::{Instr, Operand, ValueId},
    plan::FunctionPlan,
    target::{
        Artifact, FailureCategory, Operation, OutputPlacement, ParamRole, PrimitiveFailure,
        PrimitiveSpec, StandardOp, Target, Terminal,
    },
};

/// This engine's emission key.
///
/// Implementing [`RustEmitter`] is a collector's deliberate decision to
/// establish an emission boundary; v2 establishes its own rather than borrowing
/// v1's.
pub(crate) struct Writer;

impl RustEmitter for Writer {}

/// How generated code names each entity: the module it is reached through.
///
/// A captured item is reached through the source module the frontend
/// configured for the run. An item the binding defines itself is reached
/// where the binding said it is — `crate::helpers::x` — which its origin
/// records as its crate stamp, since for a binding-local item that is what
/// the stamp means. One value carries both so the writer asks one question.
pub(crate) struct Reach {
    /// Name → module, for every entity that is not reached through the
    /// default. The shape [`RustEmitter::emit_source_type`] takes.
    modules: HashMap<String, syn::Path>,
    default: syn::Path,
}

impl Reach {
    pub(crate) fn new(flat: &Flat, default: syn::Path) -> Self {
        let modules = flat
            .elements()
            .filter_map(|element| Some((element.name()?, element)))
            .filter(|(name, _)| flat.is_binding_local(*name))
            .filter_map(|(name, element)| {
                let module = element.location().crate_name.as_deref()?;
                Some((name.to_string(), syn::parse_str(module).ok()?))
            })
            .collect();
        Reach { modules, default }
    }

    /// The module the entity with this name is reached through.
    fn of(&self, name: &str) -> &syn::Path {
        self.modules.get(name).unwrap_or(&self.default)
    }
}

/// The whole generated Rust file, as text.
pub(crate) fn render<T: Target>(
    flat: &Flat,
    target: &T,
    reach: &Reach,
    artifacts: &[Artifact],
    primitives: &[PrimitiveSpec<T::Payload>],
    functions: &[FunctionPlan<T::Payload>],
) -> String {
    // The guards come first, and they are emitted whatever else this run
    // retained. Each one is a compile-time assertion the capture reader
    // injected — today, that the source crate's features match the set the
    // capture was filtered by — and a binding that dropped it would compile
    // happily against a source crate it disagrees with. It belongs to no
    // declaration, so nothing in retention decides whether to keep it.
    let guards = flat.guards().map(|guard| Writer.guard(guard));
    let artifacts = artifacts.iter().map(|artifact| &artifact.rust);
    let wrappers = functions
        .iter()
        .map(|function| wrapper(flat, target, reach, primitives, function));
    let tokens = quote! {
        #(#guards)*
        #(#artifacts)*
        #(#wrappers)*
    };
    match syn::parse2::<syn::File>(tokens.clone()) {
        Ok(file) => prettyplease::unparse(&file),
        // Unparseable output is a defect in this writer, and hiding it behind
        // pretty printing would only move the compiler error somewhere less
        // useful.
        Err(_) => tokens.to_string(),
    }
}

/// One exported function.
fn wrapper<T: Target>(
    flat: &Flat,
    target: &T,
    reach: &Reach,
    primitives: &[PrimitiveSpec<T::Payload>],
    function: &FunctionPlan<T::Payload>,
) -> TokenStream {
    let mut names: HashMap<ValueId, syn::Ident> = HashMap::new();
    // Parameters are named by the boundary, because a target that requires an
    // environment operand has to say what it is called. Everything else is
    // named here, and around those names: a temporary that shadowed a live
    // parameter would compile and read the wrong value.
    // Reserved by their canonical spelling: `r#v0` and `v0` are one name to
    // Rust, so reserving the raw form would leave the plain one free to shadow
    // it.
    let mut taken: std::collections::HashSet<String> = function
        .params
        .iter()
        .map(|(_, p)| unraw(&p.name))
        .collect();
    let params: Vec<TokenStream> = function
        .params
        .iter()
        .map(|(id, param)| {
            names.insert(*id, param.name.clone());
            let name = &param.name;
            let ty = &param.ty.ty;
            let mutable = param.mutable.then(|| quote!(mut));
            quote!(#mutable #name: #ty)
        })
        .collect();
    let ok_binding = free_name("value", &mut taken);
    let error_binding = free_name("error", &mut taken);
    // Temporaries are numbered by definition order, which is why the same
    // operation used twice cannot collide with itself.
    let mut next_local = 0;
    let mut local = |names: &mut HashMap<ValueId, syn::Ident>,
                     taken: &mut std::collections::HashSet<String>,
                     id: ValueId| {
        let name = loop {
            let candidate = format!("v{next_local}");
            next_local += 1;
            if taken.insert(candidate.clone()) {
                break format_ident!("{candidate}");
            }
        };
        names.insert(id, name.clone());
        name
    };

    // A wrapper names source items — the function it calls, and every struct it
    // constructs — and compiles only where all of them exist. So it inherits
    // each one's condition: a `#[cfg]` the capture reader could not answer and
    // rewrote back onto the item. Rust conjoins repeated `#[cfg]` attributes on
    // one item, so carrying them side by side settles the wrapper's own
    // condition without anything here reading what they say. Two items carrying
    // the same condition state it once, which conjunction makes cosmetic and
    // review makes worth doing.
    //
    // The instructions are where a wrapper spells a source item: its signature
    // carries target wire types only, and the one operation that spells a
    // source type — a handle taken back or released, cast to
    // `*mut source::Ledger` — is a standard one the registry renders, so it is
    // found here too.
    let mut conditions = Vec::new();
    let mut conditioned: std::collections::HashSet<String> = std::collections::HashSet::new();
    for step in &function.instrs {
        let named = match &step.instr {
            Instr::Construct { name, .. } => name.clone(),
            Instr::Call { function, .. } => function.clone(),
            Instr::Apply { primitive, .. } => match &primitives[primitive.0].implementation {
                Operation::Standard(
                    StandardOp::FromRaw { source } | StandardOp::Release { source },
                ) => match source.kind() {
                    prebindgen_flat::flat::TypeKind::Named { id, .. } => id.name.clone(),
                    _ => continue,
                },
                _ => continue,
            },
        };
        // Planning names only items the model declares.
        let Some(element) = flat.element(named.as_str()) else {
            continue;
        };
        for condition in Writer.conditions(Conditioned::Item(element)) {
            if conditioned.insert(condition.to_string()) {
                conditions.push(condition);
            }
        }
    }

    let mut statements = Vec::new();
    for step in &function.instrs {
        // What a field's condition holds the statement to: a member declared
        // only sometimes is read only sometimes, and the initializer that
        // consumes the read exists only then. The statement is one `let`, so
        // the attribute goes in front of it.
        let guarded = &step.conditions;
        match &step.instr {
            Instr::Apply {
                primitive,
                operands,
                result,
            } => {
                let primitive = &primitives[primitive.0];
                let operand_names: Vec<syn::Ident> = operands
                    .iter()
                    .map(|operand| operand_name(&names, function, operand))
                    .collect();
                let expression = operation(target, reach, primitive, &operand_names);
                let statement = match (&primitive.failure, result) {
                    (PrimitiveFailure::Infallible, Some(result)) => {
                        let name = local(&mut names, &mut taken, *result);
                        quote!(let #name = #expression;)
                    }
                    (PrimitiveFailure::Infallible, None) => quote!(#expression;),
                    (PrimitiveFailure::Fallible { category, .. }, Some(result)) => {
                        let name = local(&mut names, &mut taken, *result);
                        let route = failure_arm(target, reach, function, *category, &error_binding);
                        let ok = &ok_binding;
                        let error = error_pattern(function, *category, &error_binding);
                        quote! {
                            let #name = match #expression {
                                Ok(#ok) => #ok,
                                Err(#error) => { #route }
                            };
                        }
                    }
                    (PrimitiveFailure::Fallible { category, .. }, None) => {
                        let route = failure_arm(target, reach, function, *category, &error_binding);
                        let error = error_pattern(function, *category, &error_binding);
                        quote! {
                            if let Err(#error) = #expression { #route }
                        }
                    }
                };
                statements.push(quote!(#(#guarded)* #statement));
            }
            Instr::Construct {
                name,
                parts,
                result,
            } => {
                let item = flat
                    .struct_type(name.as_str())
                    .expect("a construction names a struct the model declares");
                let ident = &item.name;
                let module = reach.of(&ident.to_string());
                let head = quote!(#module::#ident);
                // An initializer carries its own field's condition: the value
                // it names was bound by a statement under the same one, so a
                // field the source does not have is neither read nor filled in.
                //
                // Read from the model here, while the statements got theirs
                // from `Part::conditions` at plan time. The two agree because
                // both read the same field; should a part's conditions ever
                // become something a target contributes to, this has to read
                // the part instead — an initializer written under a condition
                // its read does not share names a value that is not there.
                let bound: Vec<TokenStream> = item
                    .fields
                    .iter()
                    .zip(parts)
                    .map(|(field, value)| {
                        let conditions = Writer.conditions(Conditioned::Field(field));
                        let bound = field.bind(&names[value]);
                        quote!(#(#conditions)* #bound)
                    })
                    .collect();
                let value = Writer.shape_struct(item, head, &bound);
                let name = local(&mut names, &mut taken, *result);
                statements.push(quote!(#(#guarded)* let #name = #value;));
            }
            Instr::Call {
                function: callee,
                args,
                result,
            } => {
                let module = reach.of(callee);
                let callee = format_ident!("{callee}");
                let args: Vec<syn::Ident> = args.iter().map(|value| names[value].clone()).collect();
                let call = quote!(#module::#callee(#(#args),*));
                // A source function returning nothing produces no value, and a
                // `let` over it would name one nobody can use.
                let statement = match result {
                    Some(result) => {
                        let name = local(&mut names, &mut taken, *result);
                        quote!(let #name = #call;)
                    }
                    None => quote!(#call;),
                };
                statements.push(quote!(#(#guarded)* #statement));
            }
        }
    }

    let tail = match (&function.output, function.result) {
        (OutputPlacement::Return, Some(result)) => {
            let name = &names[&result];
            quote!(#name)
        }
        _ => quote!(),
    };
    let symbol = format_ident!("{}", function.abi.symbol);
    let abi = &function.abi.abi;
    let attrs = &function.abi.attrs;
    let unsafety = function.abi.unsafety.then(|| quote!(unsafe));
    let ret = function
        .abi
        .ret
        .as_ref()
        .map(|wire| {
            let ty = &wire.ty;
            quote!(-> #ty)
        })
        .unwrap_or_default();
    quote! {
        #(#conditions)*
        #[no_mangle]
        #(#attrs)*
        pub #unsafety extern #abi fn #symbol(#(#params),*) #ret {
            #(#statements)*
            #tail
        }
    }
}

/// One operation's expression: the registry renders its own, the target renders
/// its own.
fn operation<T: Target>(
    target: &T,
    reach: &Reach,
    primitive: &PrimitiveSpec<T::Payload>,
    operands: &[syn::Ident],
) -> TokenStream {
    // The handle operations spell a source type, which is what makes them the
    // registry's: an adapter has no way to, and no business doing it.
    let source_type = |ty| Writer.emit_source_type(ty, &reach.modules, &reach.default);
    match &primitive.implementation {
        Operation::Standard(StandardOp::Identity) => {
            let value = &operands[0];
            quote!(#value)
        }
        Operation::Standard(StandardOp::ReadMember { member }) => {
            let value = &operands[0];
            quote!(#value.#member)
        }
        Operation::Standard(StandardOp::IntoRaw { carrier }) => {
            let value = &operands[0];
            quote!(Box::into_raw(Box::new(#value)) as #carrier)
        }
        Operation::Standard(StandardOp::FromRaw { source }) => {
            let value = &operands[0];
            let ty = source_type(source);
            let message = format!("null `{}` handle", source.key());
            quote! {
                ::core::ptr::NonNull::new(#value as *mut #ty)
                    .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
                    .ok_or_else(|| String::from(#message))
            }
        }
        Operation::Standard(StandardOp::EnumOut { source, values }) => {
            let value = &operands[0];
            let ty = source_type(source);
            let arms = values.iter().map(|arm| {
                let pattern = enum_value(&ty, arm);
                let carried = &arm.carried;
                quote!(#pattern => #carried)
            });
            quote!(match #value { #(#arms),* })
        }
        Operation::Standard(StandardOp::EnumIn {
            source,
            values,
            invalid,
        }) => {
            let value = &operands[0];
            let ty = source_type(source);
            let arms = values.iter().map(|arm| {
                let carried = &arm.carried;
                let constructed = enum_value(&ty, arm);
                match invalid {
                    Some(_) => quote!(#carried => ::core::result::Result::Ok(#constructed)),
                    None => quote!(#carried => #constructed),
                }
            });
            match invalid {
                Some(message) => quote! {
                    match #value {
                        #(#arms,)*
                        other => ::core::result::Result::Err(::std::format!(#message, other)),
                    }
                },
                None => quote!(match #value { #(#arms),* }),
            }
        }
        Operation::Standard(StandardOp::Release { source }) => {
            let value = &operands[0];
            let ty = source_type(source);
            quote! {
                drop(::core::ptr::NonNull::new(#value as *mut #ty)
                    .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }))
            }
        }
        Operation::Target(payload) => target.render_operation(payload, operands),
    }
}

/// One value of a fieldless enum, spelled as the source declared it.
///
/// A pattern and a constructor are the same text for a fieldless value, so
/// this serves both sides of the two enum operations. `Add` stays `Add`, and
/// `Add()` and `Mul {}` keep their delimiters — the model's speller decides,
/// from the shape it captured.
fn enum_value(ty: &TokenStream, arm: &crate::target::EnumArm) -> TokenStream {
    let name = &arm.name;
    arm.shape.spell_fieldless(quote!(#ty::#name))
}

/// The pattern a failure arm binds the error with: the name when a reporter
/// reads it, `_` when the route only terminates.
fn error_pattern<P>(
    function: &FunctionPlan<P>,
    category: FailureCategory,
    error: &syn::Ident,
) -> TokenStream {
    let reported = function
        .failures
        .iter()
        .any(|route| route.category == category && route.report.is_some());
    if reported {
        quote!(#error)
    } else {
        quote!(_)
    }
}

/// What one failure category does at this boundary: report if the target has an
/// operation for it, then terminate.
fn failure_arm<T: Target>(
    target: &T,
    reach: &Reach,
    function: &FunctionPlan<T::Payload>,
    category: FailureCategory,
    error: &syn::Ident,
) -> TokenStream {
    let route = function
        .failures
        .iter()
        .find(|route| route.category == category)
        .expect("an unrouted failure category is refused before assembly");
    let report = route.report.as_ref().map(|report| {
        let operands: Vec<syn::Ident> = report
            .operands
            .iter()
            .map(|operand| match &operand.role {
                crate::target::OperandRole::Context(name) => context_name(function, name),
                crate::target::OperandRole::Error => error.clone(),
                // Refused when the boundary is assembled: a reporting operation
                // is given the error and the contexts, and has no value to read.
                crate::target::OperandRole::Value => {
                    unreachable!("a reporting operation with a value operand is refused")
                }
            })
            .collect();
        let expression = operation(target, reach, report, &operands);
        let on_failure = terminal(&route.on_report_failure);
        match report.failure {
            PrimitiveFailure::Infallible => quote!(#expression;),
            PrimitiveFailure::Fallible { .. } => quote! {
                if #expression.is_err() { #on_failure }
            },
        }
    });
    let terminate = terminal(&route.terminate);
    quote! {
        #report
        #terminate
    }
}

fn terminal(terminal: &Terminal) -> TokenStream {
    match terminal {
        Terminal::Abort => quote!(std::process::abort();),
        Terminal::Return(expr) => quote!(return #expr;),
    }
}

fn operand_name<P>(
    names: &HashMap<ValueId, syn::Ident>,
    function: &FunctionPlan<P>,
    operand: &Operand,
) -> syn::Ident {
    match operand {
        Operand::Value(id) => names[id].clone(),
        Operand::Context(name) => context_name(function, name),
    }
}

/// An identifier's canonical spelling: `r#type` and `type` name one binding.
fn unraw(ident: &syn::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}

/// A name nothing in this wrapper already uses.
fn free_name(preferred: &str, taken: &mut std::collections::HashSet<String>) -> syn::Ident {
    if taken.insert(preferred.to_string()) {
        return format_ident!("{preferred}");
    }
    for suffix in 1.. {
        let candidate = format!("{preferred}_{suffix}");
        if taken.insert(candidate.clone()) {
            return format_ident!("{candidate}");
        }
    }
    unreachable!("the search above only ends by returning")
}

/// The wrapper parameter supplying a runtime context.
fn context_name<P>(function: &FunctionPlan<P>, context: &str) -> syn::Ident {
    function
        .params
        .iter()
        .find(|(_, param)| matches!(&param.role, ParamRole::Context(name) if name == context))
        .map(|(_, param)| param.name.clone())
        .expect("a missing runtime context is refused before assembly")
}
