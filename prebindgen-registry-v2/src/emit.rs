//! The common Rust writer: one component, both targets.
//!
//! It owns the shape of the generated Rust — the locals and their order, the
//! branches on failure, the construction of the source value, the call, the
//! return — and allocates every temporary from the plan, so two operations
//! rendered into one wrapper cannot collide over a name. A target contributes
//! one expression per operation of its own ([`Target::write_operation`]) and
//! the declarations its wire types need ([`Target::write_wire_type`]); it
//! contributes no control flow.
//!
//! Source types and struct shapes are generated through Flat's emission
//! capability, never from retained syntax and never spelled by an adapter.

use std::collections::{HashMap, HashSet};

use prebindgen_flat::{flat::Flat, Conditioned, RustEmitter};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    binding::{
        Binding, FailureRoute, Implementation, OutputForm, Representation, StandardOp, WireType,
        WireTypeId,
    },
    body::{Instr, Operand, Stmt, ValueId},
    plan::{Applied, FunctionPlan, ParamRole, Retained, Slot, ValuePlan},
    target::{
        Artifact, FailureCategory, Fed, OperationFeed, Target, Terminal, WireTypeFeed, Written,
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn render<T: Target>(
    flat: &Flat,
    target: &T,
    binding: &Binding<T>,
    reach: &Reach,
    retained: &[Retained],
    nodes: &[ValuePlan],
    primitives: &[Applied<T::Op>],
    functions: &[FunctionPlan<T::Op>],
) -> String {
    // The guards come first, and they are emitted whatever else this run
    // retained. Each one is a compile-time assertion the capture reader
    // injected — today, that the source crate's features match the set the
    // capture was filtered by — and a binding that dropped it would compile
    // happily against a source crate it disagrees with. It belongs to no
    // declaration, so nothing in retention decides whether to keep it.
    let guards = flat.guards().map(|guard| Writer.guard(guard));
    let wire_types = wire_types(flat, target, binding, retained, nodes);
    // A helper an operation needs is emitted once, before the wrappers, in
    // the order the wrappers first need it.
    let mut helpers: Vec<Artifact> = Vec::new();
    let wrappers: Vec<TokenStream> = functions
        .iter()
        .map(|function| {
            wrapper(
                flat,
                target,
                binding,
                reach,
                primitives,
                function,
                &mut helpers,
            )
        })
        .collect();
    let helpers = helpers.iter().map(|helper| &helper.rust);
    let tokens = quote! {
        #(#guards)*
        #(#wire_types)*
        #(#helpers)*
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

/// The declarations the wire types of every retained type need, each once, in
/// the order the types were declared.
///
/// A wire type's declaration exists only where the source item its type names
/// does — the `repr(C)` mirror of a conditional struct is itself conditional —
/// so every item the target writes carries that item's conditions.
fn wire_types<T: Target>(
    flat: &Flat,
    target: &T,
    binding: &Binding<T>,
    retained: &[Retained],
    nodes: &[ValuePlan],
) -> Vec<TokenStream> {
    let mut written: std::collections::HashSet<WireTypeId> = std::collections::HashSet::new();
    let mut items = Vec::new();
    for values in retained {
        let OutputForm::Type { representation, .. } = binding.form_of(values.output) else {
            continue;
        };
        let conditions = values
            .declaration
            .captured(flat)
            .map(|element| Writer.conditions(Conditioned::Item(element)))
            .unwrap_or_default();
        let unit = values
            .declaration
            .entity_name()
            .and_then(|name| flat.unit_enum(&name));
        let root = values.inputs.first().map(|root| &nodes[root.0]);
        let used: Vec<(WireTypeId, bool)> = match binding.representation_of(*representation) {
            Representation::Terminal {
                into_rust,
                out_of_rust,
                ..
            } => into_rust
                .iter()
                .chain(out_of_rust.iter())
                .map(|codec| (codec.wire_type, false))
                .collect(),
            Representation::Product { wire_type, .. }
            | Representation::Callback { wire_type, .. } => {
                vec![(*wire_type, true)]
            }
            Representation::Unsupported(_) => Vec::new(),
        };
        for (wire_type, with_parts) in used {
            if !written.insert(wire_type) {
                continue;
            }
            let parts = match (with_parts, root) {
                (true, Some(root)) => root
                    .relation
                    .parts()
                    .iter()
                    .zip(&root.children)
                    .map(|(part, child)| (part, binding.wire_type_of(nodes[child.0].wire_type)))
                    .collect(),
                _ => Vec::new(),
            };
            let feed = WireTypeFeed {
                wire_type: binding.wire_type_of(wire_type),
                parts,
                unit,
            };
            for item in target.write_wire_type(&feed) {
                items.push(quote!(#(#conditions)* #item));
            }
        }
    }
    items
}

/// One exported function.
fn wrapper<T: Target>(
    flat: &Flat,
    target: &T,
    binding: &Binding<T>,
    reach: &Reach,
    primitives: &[Applied<T::Op>],
    function: &FunctionPlan<T::Op>,
    helpers: &mut Vec<Artifact>,
) -> TokenStream {
    // Parameters are named by the form, because a convention that requires an
    // environment operand has to say what it is called. Everything else is
    // named here, and around those names: a temporary that shadowed a live
    // parameter would compile and read the wrong value.
    // Reserved by their canonical spelling: `r#v0` and `v0` are one name to
    // Rust, so reserving the raw form would leave the plain one free to shadow
    // it.
    let mut taken: HashSet<String> = function
        .params
        .iter()
        .map(|(_, p)| unraw(&p.name))
        .collect();
    let ok_binding = free_name("value", &mut taken);
    let error_binding = free_name("error", &mut taken);
    let mut scribe = Scribe {
        flat,
        target,
        binding,
        reach,
        primitives,
        helpers,
        names: HashMap::new(),
        taken,
        next_local: 0,
        ok_binding,
        error_binding,
    };
    let params: Vec<TokenStream> = function
        .params
        .iter()
        .map(|(id, param)| {
            scribe.names.insert(*id, param.name.clone());
            let name = &param.name;
            let ty = &param.ty;
            let mutable = param.mutable.then(|| quote!(mut));
            quote!(#mutable #name: #ty)
        })
        .collect();

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
    // carries wire types only, and the one operation that spells a source type —
    // a handle taken back or released, cast to `*mut source::Ledger` — is a
    // standard one the registry writes, so it is found here too.
    let mut conditions = Vec::new();
    let mut conditioned: HashSet<String> = HashSet::new();
    for step in &function.instrs {
        let named = match &step.instr {
            Instr::Construct { name, .. } => name.clone(),
            Instr::Call { function, .. } => function.clone(),
            Instr::Apply { primitive, .. } => {
                let applied = &primitives[primitive.0];
                match &applied.implementation {
                    Implementation::Standard(StandardOp::FromRaw | StandardOp::Release) => {
                        match applied.subject.kind() {
                            prebindgen_flat::flat::TypeKind::Named { id, .. } => id.name.clone(),
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
            // What a call converts leaves Rust, and names nothing a condition
            // is on that the wrapper's call does not name too.
            Instr::Closure { .. } => continue,
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

    let frame = Frame {
        routes: &function.routes,
        contexts: function
            .params
            .iter()
            .filter_map(|(_, param)| match &param.role {
                ParamRole::Context(name) => Some((name.clone(), param.name.clone())),
                _ => None,
            })
            .collect(),
    };
    let statements = scribe.statements(&function.instrs, &frame);

    let tail = match function.result {
        Some(result) => {
            let name = &scribe.names[&result];
            quote!(#name)
        }
        None => quote!(),
    };
    let symbol = format_ident!("{}", function.symbol);
    let abi = &function.abi;
    let attrs = &function.attrs;
    // A wire type whose bits are read as an integer holds whatever the caller
    // put in it. A foreign caller passes a number, which the match checks;
    // safe Rust could pass storage never initialized, which no check can
    // look at. So the wrapper is `unsafe`, and says what it is owed.
    let reads_bits = function.instrs.iter().any(|step| match &step.instr {
        Instr::Apply { primitive, .. } => matches!(
            primitives[primitive.0].implementation,
            Implementation::Standard(StandardOp::EnumIn { bits: Some(_), .. })
        ),
        _ => false,
    });
    let unsafety = (function.unsafety || reads_bits).then(|| quote!(unsafe));
    let safety = reads_bits.then(|| {
        quote! {
            /// # Safety
            ///
            /// Every argument that carries an enum as its bits must be
            /// initialized. The wrapper reads those bits as an integer and
            /// refuses a number no value has, but reading storage that was
            /// never initialized is undefined behaviour.
        }
    });
    let ret = function
        .ret
        .as_ref()
        .map(|ty| quote!(-> #ty))
        .unwrap_or_default();
    quote! {
        #safety
        #(#conditions)*
        #[no_mangle]
        #(#attrs)*
        pub #unsafety extern #abi fn #symbol(#(#params),*) #ret {
            #(#statements)*
            #tail
        }
    }
}

/// Where statements are rendered: the routes a failure there takes, and the
/// names the runtime contexts go by. A wrapper's body is one frame, and the
/// body of a callback's closure is another, with the callback's routes and no
/// contexts.
struct Frame<'a, Op> {
    routes: &'a [FailureRoute<Op>],
    contexts: HashMap<String, syn::Ident>,
}

impl<Op> Frame<'_, Op> {
    fn context(&self, name: &str) -> syn::Ident {
        self.contexts
            .get(name)
            .cloned()
            .expect("a missing runtime context is refused before assembly")
    }
}

/// Renders the statements of one wrapper, and allocates every name in it.
struct Scribe<'a, T: Target> {
    flat: &'a Flat,
    target: &'a T,
    binding: &'a Binding<T>,
    reach: &'a Reach,
    primitives: &'a [Applied<T::Op>],
    helpers: &'a mut Vec<Artifact>,
    names: HashMap<ValueId, syn::Ident>,
    taken: HashSet<String>,
    /// Temporaries are numbered by definition order, which is why the same
    /// operation used twice cannot collide with itself.
    next_local: usize,
    ok_binding: syn::Ident,
    error_binding: syn::Ident,
}

impl<T: Target> Scribe<'_, T> {
    /// A fresh name for `id`, one nothing in this wrapper uses.
    fn local(&mut self, id: ValueId) -> syn::Ident {
        let name = loop {
            let candidate = format!("v{}", self.next_local);
            self.next_local += 1;
            if self.taken.insert(candidate.clone()) {
                break format_ident!("{candidate}");
            }
        };
        self.names.insert(id, name.clone());
        name
    }

    fn statements(&mut self, instrs: &[Stmt], frame: &Frame<'_, T::Op>) -> Vec<TokenStream> {
        let mut statements = Vec::new();
        for step in instrs {
            // What a field's condition holds the statement to: a member
            // declared only sometimes is read only sometimes, and the
            // initializer that consumes the read exists only then. The
            // statement is one `let`, so the attribute goes in front of it.
            let guarded = &step.conditions;
            let statement = self.statement(&step.instr, frame);
            statements.push(quote!(#(#guarded)* #statement));
        }
        statements
    }

    fn statement(&mut self, instr: &Instr, frame: &Frame<'_, T::Op>) -> TokenStream {
        match instr {
            Instr::Apply {
                primitive,
                operands,
                result,
            } => {
                let applied = &self.primitives[primitive.0];
                let operand_names: Vec<syn::Ident> = operands
                    .iter()
                    .map(|operand| match operand {
                        Operand::Value(id) => self.names[id].clone(),
                        Operand::Context(name) => frame.context(name),
                    })
                    .collect();
                let expression = operation(
                    self.target,
                    self.binding,
                    self.reach,
                    applied,
                    &operand_names,
                    self.helpers,
                );
                let Some(failure) = &applied.failure else {
                    return match result {
                        Some(result) => {
                            let name = self.local(*result);
                            quote!(let #name = #expression;)
                        }
                        None => quote!(#expression;),
                    };
                };
                let error = error_pattern(frame.routes, failure.category, &self.error_binding);
                let route = failure_arm(
                    self.target,
                    frame,
                    failure.category,
                    &self.error_binding,
                    self.helpers,
                );
                match result {
                    Some(result) => {
                        let name = self.local(*result);
                        let ok = &self.ok_binding;
                        quote! {
                            let #name = match #expression {
                                Ok(#ok) => #ok,
                                Err(#error) => { #route }
                            };
                        }
                    }
                    None => quote! {
                        if let Err(#error) = #expression { #route }
                    },
                }
            }
            Instr::Construct {
                name,
                parts,
                result,
            } => {
                let item = self
                    .flat
                    .struct_type(name.as_str())
                    .expect("a construction names a struct the model declares");
                let ident = &item.name;
                let module = self.reach.of(&ident.to_string());
                let head = quote!(#module::#ident);
                // An initializer carries its own field's condition: the value
                // it names was bound by a statement under the same one, so a
                // field the source does not have is neither read nor filled in.
                //
                // Read from the model here, while the statements got theirs
                // from `Part::conditions` at plan time. The two agree because
                // both read the same field.
                let bound: Vec<TokenStream> = item
                    .fields
                    .iter()
                    .zip(parts)
                    .map(|(field, value)| {
                        let conditions = Writer.conditions(Conditioned::Field(field));
                        let bound = field.bind(&self.names[value]);
                        quote!(#(#conditions)* #bound)
                    })
                    .collect();
                let value = Writer.shape_struct(item, head, &bound);
                let name = self.local(*result);
                quote!(let #name = #value;)
            }
            Instr::Call {
                function: callee,
                args,
                result,
            } => {
                let module = self.reach.of(callee);
                let callee = format_ident!("{callee}");
                let args: Vec<syn::Ident> =
                    args.iter().map(|value| self.names[value].clone()).collect();
                let call = quote!(#module::#callee(#(#args),*));
                // A source function returning nothing produces no value, and a
                // `let` over it would name one nobody can use.
                match result {
                    Some(result) => {
                        let name = self.local(*result);
                        quote!(let #name = #call;)
                    }
                    None => quote!(#call;),
                }
            }
            Instr::Closure {
                representation,
                params,
                instrs,
                result,
                ..
            } => {
                let Representation::Callback { routes, .. } =
                    self.binding.representation_of(*representation)
                else {
                    unreachable!("a closure is built for a callback representation")
                };
                let params: Vec<TokenStream> = params
                    .iter()
                    .map(|(id, ty)| {
                        let name = self.local(*id);
                        let ty =
                            Writer.emit_source_type(ty, &self.reach.modules, &self.reach.default);
                        quote!(#name: #ty)
                    })
                    .collect();
                // Inside a call no runtime context exists: planning refused
                // any operation there that asks for one.
                let inner = Frame {
                    routes,
                    contexts: HashMap::new(),
                };
                let body = self.statements(instrs, &inner);
                let name = self.local(*result);
                quote! {
                    let #name = move |#(#params),*| {
                        #(#body)*
                    };
                }
            }
        }
    }
}

/// Keep `written`'s helpers, each once.
fn keep(helpers: &mut Vec<Artifact>, written: Written) -> TokenStream {
    for helper in written.helpers {
        if !helpers.iter().any(|kept| kept.name == helper.name) {
            helpers.push(helper);
        }
    }
    written.text
}

/// One operation's expression: the registry writes its own, the target writes
/// its own from what the plan feeds it.
fn operation<T: Target>(
    target: &T,
    binding: &Binding<T>,
    reach: &Reach,
    applied: &Applied<T::Op>,
    operands: &[syn::Ident],
    helpers: &mut Vec<Artifact>,
) -> TokenStream {
    // The handle operations spell a source type, which is what makes them the
    // registry's: an adapter has no way to, and no business doing it.
    let source_type = |ty| Writer.emit_source_type(ty, &reach.modules, &reach.default);
    let wire_rust = |slot: Option<Slot>| match slot {
        Some(Slot::WireType(wire_type)) => {
            let ty = binding.wire_type_of(wire_type).rust();
            quote!(#ty)
        }
        _ => source_type(&applied.subject),
    };
    let value = &operands[0];
    match &applied.implementation {
        Implementation::Standard(StandardOp::Identity) => quote!(#value),
        Implementation::Standard(StandardOp::ReadMember) => {
            let member = applied
                .part
                .as_ref()
                .expect("a member read is applied to a part")
                .member();
            quote!(#value.#member)
        }
        Implementation::Standard(StandardOp::IntoRaw) => {
            let wire_type = wire_rust(applied.result);
            quote!(Box::into_raw(Box::new(#value)) as #wire_type)
        }
        Implementation::Standard(StandardOp::FromRaw) => {
            let ty = source_type(&applied.subject);
            let message = format!("null `{}` handle", applied.subject.key());
            quote! {
                ::core::ptr::NonNull::new(#value as *mut #ty)
                    .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
                    .ok_or_else(|| String::from(#message))
            }
        }
        Implementation::Standard(StandardOp::EnumOut { values }) => {
            let ty = source_type(&applied.subject);
            let arms = values.iter().map(|arm| {
                let pattern = enum_value(&ty, arm);
                let carried = &arm.carried;
                quote!(#pattern => #carried)
            });
            quote!(match #value { #(#arms),* })
        }
        Implementation::Standard(StandardOp::EnumIn {
            values,
            invalid,
            bits,
        }) => {
            let value = match bits {
                Some(bits) => quote!(unsafe { ::core::mem::transmute_copy::<_, #bits>(&(#value)) }),
                None => quote!(#value),
            };
            let ty = source_type(&applied.subject);
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
        Implementation::Standard(StandardOp::Release) => {
            let ty = source_type(&applied.subject);
            quote! {
                drop(::core::ptr::NonNull::new(#value as *mut #ty)
                    .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }))
            }
        }
        Implementation::Target(op) => {
            let fed = |slot: Slot| match slot {
                Slot::Source => Fed::Source(&applied.subject),
                Slot::WireType(wire_type) => Fed::WireType(binding.wire_type_of(wire_type)),
                Slot::Captured => Fed::Captured,
            };
            // After the value and the contexts come the arguments a callback's
            // `invoke` is handed; its `capture` is told their wire types only.
            let named = &operands[1 + applied.contexts.len()..];
            let feed = OperationFeed {
                value: Some((value.clone(), fed(applied.value))),
                contexts: applied
                    .contexts
                    .iter()
                    .cloned()
                    .zip(operands[1..].iter().cloned())
                    .collect(),
                error: None,
                result: applied.result.map(fed),
                part: applied.part.as_ref(),
                args: applied
                    .args
                    .iter()
                    .enumerate()
                    .map(|(index, wire_type)| {
                        (named.get(index).cloned(), binding.wire_type_of(*wire_type))
                    })
                    .collect(),
            };
            keep(helpers, target.write_operation(op, &feed))
        }
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
fn error_pattern<Op>(
    routes: &[FailureRoute<Op>],
    category: FailureCategory,
    error: &syn::Ident,
) -> TokenStream {
    let reported = routes
        .iter()
        .any(|route| route.category == category && route.report.is_some());
    if reported {
        quote!(#error)
    } else {
        quote!(_)
    }
}

/// What one failure category does at this boundary: report if the form has an
/// operation for it, then terminate.
fn failure_arm<T: Target>(
    target: &T,
    frame: &Frame<'_, T::Op>,
    category: FailureCategory,
    error: &syn::Ident,
    helpers: &mut Vec<Artifact>,
) -> TokenStream {
    let route = frame
        .routes
        .iter()
        .find(|route| route.category == category)
        .expect("an unrouted failure category is refused before assembly");
    let report = route.report.as_ref().map(|report| {
        let contexts: Vec<(String, syn::Ident)> = report
            .operation
            .context
            .iter()
            .map(|name| (name.clone(), frame.context(name)))
            .collect();
        let expression = match &report.operation.implementation {
            Implementation::Target(op) => {
                let feed = OperationFeed::<T> {
                    value: None,
                    contexts,
                    error: Some(error.clone()),
                    result: None,
                    part: None,
                    args: Vec::new(),
                };
                keep(helpers, target.write_operation(op, &feed))
            }
            // A standard operation converts a value, and a report has none to
            // convert; a binding that stated one is refused before assembly.
            Implementation::Standard(_) => {
                unreachable!("a reporting operation is the target's own")
            }
        };
        let on_failure = terminal(&route.on_report_failure);
        match report.operation.failure {
            None => quote!(#expression;),
            Some(_) => quote! {
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
