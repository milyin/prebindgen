//! What the engine decides, over a target that writes in one line.
//!
//! The adapter here is deliberately not a language: it exists to make the
//! engine's own contracts observable — which conversions are shared, which
//! rule a value takes, what a missing capability takes down with it, what
//! happens when a form leaves a declared failure unrouted. The two real
//! adapters, and the generated code rustc compiles, live in `examples/v2check`.

use prebindgen_flat::flat::Flat;

use crate::{
    binding::{
        Binding, Failure, FailureRoute, FunctionForm, InReprId, InRepresentation, Operation,
        OutReprId, OutRepresentation, OutputForm, OutputFormOf, Report, ReprId, Scope, StandardOp,
        Step, TargetOp, ValuePath, Via, WireKind, WireType,
    },
    decl::Declaration,
    outcome::{EngineError, Outcome},
    plan::generate,
    run::Generation,
    target::{
        FailureCategory, OperationFeed, PlanningError, Target, Terminal, Unsupported, WireTypeFeed,
        Written,
    },
};

/// Three structs and four functions over them, one struct with a field
/// nothing carries, one holding another; an opaque type, and the two functions
/// that hand one out and take it back.
fn model() -> Flat {
    Flat::builder()
        .items(model_items())
        .build()
        .expect("the fixture builds a model")
}

/// The same, as the items a builder takes — for a test that adds to them.
fn model_items() -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
        syn::parse_quote!(
            pub struct Stamp {
                pub secs: i64,
                pub nanos: i64,
            }
        ),
        syn::parse_quote!(
            pub struct Label {
                pub text: String,
            }
        ),
        syn::parse_quote!(
            pub struct Point {
                pub x: i64,
                pub y: i64,
            }
        ),
        syn::parse_quote!(
            pub struct Wrap {
                pub stamp: Stamp,
                pub id: i64,
            }
        ),
        syn::parse_quote!(
            pub fn stamp_sum(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn stamp_max(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn label_len(label: Label) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn stamp_pick(stamp: Stamp, fallback: i64) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn wrap_sum(wrap: Wrap) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub type Token = external::Token;
        ),
        syn::parse_quote!(
            pub fn token_new() -> Token {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn token_use(token: Token) -> i64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn stamp_each(stamp: Stamp, f: impl Fn(i64) + Send + Sync + 'static) {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn token_watch(f: impl Fn(Token, i64) + Send + Sync + 'static) {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn stamp_emit(f: impl Fn(Stamp) + Send + Sync + 'static) {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn each_new() -> impl Fn(i64) + Send + Sync + 'static {
                unimplemented!()
            }
        ),
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    items
}

/// This target's kinds of wire type. Beside one of each kind a real target
/// has, it has kinds whose capabilities are narrower, which is how a test
/// states a limit of the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    Scalar,
    /// A struct holding scalars, other structs and pointers.
    Aggregate,
    /// A struct that can hold only pointers.
    PointerAggregate,
    /// A struct that can hold only scalars.
    ScalarAggregate,
    /// A struct a wrapper cannot take as a parameter.
    Local,
    Pointer,
    /// A callable taking scalars and pointers.
    Closure,
    /// A callable that can take only scalars.
    ScalarClosure,
    /// An address carried as an integer.
    Address,
}

impl WireKind for Kind {
    const ALL: &'static [Self] = &[
        Kind::Scalar,
        Kind::Aggregate,
        Kind::PointerAggregate,
        Kind::ScalarAggregate,
        Kind::Local,
        Kind::Pointer,
        Kind::Closure,
        Kind::ScalarClosure,
        Kind::Address,
    ];

    fn name(self) -> &'static str {
        match self {
            Kind::Scalar => "scalar",
            Kind::Aggregate => "aggregate",
            Kind::PointerAggregate => "pointer_aggregate",
            Kind::ScalarAggregate => "scalar_aggregate",
            Kind::Local => "local",
            Kind::Pointer => "pointer",
            Kind::Closure => "closure",
            Kind::ScalarClosure => "scalar_closure",
            Kind::Address => "address",
        }
    }

    fn parts(self) -> &'static [Self] {
        match self {
            Kind::Aggregate | Kind::Local => &[Kind::Scalar, Kind::Aggregate, Kind::Pointer],
            Kind::PointerAggregate => &[Kind::Pointer],
            Kind::ScalarAggregate | Kind::ScalarClosure => &[Kind::Scalar],
            Kind::Closure => &[Kind::Scalar, Kind::Pointer],
            _ => &[],
        }
    }
}

/// This target's wire types. Only a mirrored struct needs a declaration of its
/// own, so every other test's generated file holds wrappers alone.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Wire {
    Scalar,
    Address,
    /// A struct of one of the aggregate kinds, named, and mirrored or not.
    Aggregate {
        kind: Kind,
        name: syn::Ident,
        mirror: bool,
    },
    Pointer {
        name: syn::Ident,
    },
    /// A callable of one of the closure kinds.
    Closure {
        kind: Kind,
        name: syn::Ident,
    },
}

impl WireType for Wire {
    type Kind = Kind;

    fn kind(&self) -> Kind {
        match self {
            Wire::Scalar => Kind::Scalar,
            Wire::Address => Kind::Address,
            Wire::Aggregate { kind, .. } | Wire::Closure { kind, .. } => *kind,
            Wire::Pointer { .. } => Kind::Pointer,
        }
    }

    fn rust(&self) -> syn::Type {
        match self {
            Wire::Scalar => syn::parse_quote!(i64),
            Wire::Address => syn::parse_quote!(usize),
            Wire::Aggregate { name, .. } | Wire::Closure { name, .. } => syn::parse_quote!(#name),
            Wire::Pointer { name } => syn::parse_quote!(*mut #name),
        }
    }
}

/// This target's own operations.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Op {
    /// A member read that can fail — which is what makes a form's failure
    /// routes observable.
    ReadFallibly,
    Report,
    /// The binding-failure reporter, which is handed a `String`.
    ReportMessage,
    /// A scalar carried as itself, through an operation of this target's own
    /// rather than the registry's identity: the same `i64` on the wire, a
    /// different conversion.
    Rebase,
    /// What a callback keeps from its wire type.
    Capture,
    /// One call of the foreign callable, which cannot fail.
    Invoke,
    /// The same, failing at runtime.
    InvokeFallibly,
    /// The same, needing a runtime context.
    InvokeNeedingContext,
    /// A reporter needing a runtime context no form here supplies.
    ReportNeedingContext,
}

/// The one runtime context this target's operations know, and the one error
/// its fallible operations raise.
impl TargetOp for Op {
    fn contexts(&self) -> &'static [&'static str] {
        match self {
            Op::InvokeNeedingContext | Op::ReportNeedingContext => &["mini.log"],
            _ => &[],
        }
    }

    fn failure(&self) -> Option<Failure> {
        match self {
            Op::ReadFallibly | Op::InvokeFallibly => Some(Failure {
                category: FailureCategory::Runtime,
                error: Box::new(syn::parse_quote!(Error)),
            }),
            _ => None,
        }
    }
}

/// The miniature target: a writer for each of its operations, and a mirror
/// for a struct that asks for one.
struct Mini;

impl Target for Mini {
    const NAME: &'static str = "mini";

    type WireType = Wire;
    type Op = Op;

    const PARAMS: &'static [Kind] = &[
        Kind::Scalar,
        Kind::Aggregate,
        Kind::PointerAggregate,
        Kind::ScalarAggregate,
        Kind::Pointer,
        Kind::Closure,
        Kind::ScalarClosure,
        Kind::Address,
    ];
    type OutputMeta = ();

    fn write_operation(&self, op: &Op, feed: &OperationFeed<'_, Self>) -> Written {
        let value = feed.value.as_ref().map(|(value, _)| value);
        let error = feed.error.as_ref();
        Written::new(match op {
            Op::ReadFallibly => quote::quote!(read(#value)),
            Op::Report | Op::ReportNeedingContext => quote::quote!(report(#error)),
            Op::ReportMessage => quote::quote!(report_message(#error)),
            Op::Rebase => quote::quote!(rebase(#value)),
            Op::Capture => {
                let carried = feed.args.iter().map(|(_, wire_type)| wire_type.rust());
                quote::quote!(capture::<(#(#carried,)*)>(#value))
            }
            Op::Invoke | Op::InvokeFallibly | Op::InvokeNeedingContext => {
                let args = feed.args.iter().map(|(name, _)| name);
                quote::quote!(invoke(&#value, #(#args),*))
            }
        })
    }

    /// The mirror is what a real C target contributes: a struct of its own,
    /// one member per source field, each under that field's condition.
    fn write_wire_type(&self, feed: &WireTypeFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
        if !matches!(feed.wire_type, Wire::Aggregate { mirror: true, .. }) {
            return Vec::new();
        }
        let ty = feed.wire_type.rust();
        let members = feed.parts.iter().map(|(part, wire_type)| {
            let name = quote::format_ident!("{}", part.label());
            let conditions = &part.conditions;
            let member = wire_type.rust();
            quote::quote!(#(#conditions)* pub #name: #member)
        });
        vec![quote::quote!(#[repr(C)] pub struct #ty { #(#members),* })]
    }
}

/// What a form does about failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Routes {
    /// None declared, which skips any function whose conversions can fail.
    None,
    /// One route per category a conversion here can raise, each reporting
    /// through an operation that needs no context.
    Reported,
    /// One route whose reporting operation needs a runtime context the form
    /// does not supply.
    ReporterNeedsContext,
}

/// A form exporting a function under `symbol`. Its wrapper parameters are
/// named `arg0`, `arg1`, … unless a test names them.
fn exported(symbol: &str, routes: Routes) -> FunctionForm<Op> {
    let report = |op: Op, error: syn::Type| Report {
        error,
        operation: Operation::Target(op),
    };
    FunctionForm {
        abi: "C".to_string(),
        symbol: symbol.to_string(),
        context: Vec::new(),
        inputs: Vec::new(),
        routes: match routes {
            Routes::None => Vec::new(),
            Routes::Reported | Routes::ReporterNeedsContext => vec![
                FailureRoute {
                    category: FailureCategory::Binding,
                    report: Some(report(Op::ReportMessage, syn::parse_quote!(String))),
                    on_report_failure: Terminal::Abort,
                    terminate: Terminal::Return(syn::parse_quote!(0)),
                },
                FailureRoute {
                    category: FailureCategory::Runtime,
                    report: Some(report(
                        match routes {
                            Routes::ReporterNeedsContext => Op::ReportNeedingContext,
                            _ => Op::Report,
                        },
                        syn::parse_quote!(Error),
                    )),
                    on_report_failure: Terminal::Abort,
                    terminate: Terminal::Return(syn::parse_quote!(0)),
                },
            ],
        },
        attrs: Vec::new(),
        unsafety: false,
    }
}

/// How a test's callback calls through.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Call {
    /// An invocation that cannot fail.
    Infallible,
    /// One that can, with a route logging the failure.
    Routed,
    /// One that can, with no route for it.
    Unrouted,
    /// An invocation asking for a runtime context, which nothing inside a
    /// call supplies.
    NeedsContext,
}

/// The callback signature `impl Fn(<args>) + Send + Sync + 'static`.
fn callback_key(args: &str) -> prebindgen_flat::TypeKey {
    key(&format!("impl Fn({args}) + Send + Sync + 'static"))
}

/// A representation, by what a test needs of it.
#[derive(Clone)]
enum Shape {
    /// A struct read member by member.
    Struct(&'static str),
    /// The same, with member reads that can fail.
    FallibleStruct(&'static str),
    /// The same, with a declaration of its own: a mirror of the source struct.
    MirroredStruct(&'static str),
    /// A struct whose wire type is of the given aggregate kind.
    StructOf(&'static str, Kind),
    /// An opaque value carried whole as an address, released through a
    /// release of its own.
    Handle,
    /// A handle taken back as an address and handed out as an integer: two
    /// wire types, one per direction.
    SplitHandle,
    /// An `i64` carried through this target's own operation.
    ScalarThrough,
}

/// How a type's values cross, one representation per direction they cross in:
/// what a frontend states for one declared type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Repr {
    into_rust: InReprId,
    out_of_rust: Option<OutReprId>,
}

impl Repr {
    fn ids(self) -> impl Iterator<Item = ReprId> {
        std::iter::once(ReprId::In(self.into_rust)).chain(self.out_of_rust.map(ReprId::Out))
    }

    /// The one for values crossing at the end of `path`, which a rule there
    /// has to serve.
    fn at(self, path: &[Step]) -> ReprId {
        let out_of_rust = path.iter().fold(false, |out, step| match step {
            Step::Param(_) => false,
            Step::Return | Step::Arg(_) => true,
            Step::Field(_) => out,
        });
        match out_of_rust {
            false => ReprId::In(self.into_rust),
            true => ReprId::Out(
                self.out_of_rust
                    .expect("a rule out of Rust names a representation with that direction"),
            ),
        }
    }
}

/// A miniature frontend: what the binding declares, stated as a binding.
///
/// Rules at a position name the function by name and are resolved to its
/// output when the binding is built, so a test may state one before or after
/// the function's declaration.
struct Fixture {
    binding: Binding<Mini>,
    scalar: Repr,
    outputs: Vec<(Declaration, OutputFormOf<Mini>)>,
    rules: Vec<(Scope, ReprId)>,
    at: Vec<(Declaration, Vec<Step>, Repr)>,
}

fn fixture() -> Fixture {
    let mut binding = Binding::new();
    let i64_wire = binding.wire_type(Wire::Scalar);
    let scalar = Repr {
        into_rust: binding.in_representation(InRepresentation::Whole {
            wire_type: i64_wire,
            operation: Operation::Standard(StandardOp::Identity),
        }),
        out_of_rust: Some(binding.out_representation(OutRepresentation::Whole {
            wire_type: i64_wire,
            operation: Operation::Standard(StandardOp::Identity),
            release: None,
        })),
    };
    Fixture {
        binding,
        scalar,
        outputs: Vec::new(),
        rules: scalar
            .ids()
            .map(|id| (Scope::Type(key("i64")), id))
            .collect(),
        at: Vec::new(),
    }
}

impl Fixture {
    /// Declare a type's representations.
    fn repr(&mut self, shape: Shape) -> Repr {
        let aggregate = |binding: &mut Binding<Mini>, name: &str, kind, mirror| {
            binding.wire_type(Wire::Aggregate {
                kind,
                name: quote::format_ident!("{name}"),
                mirror,
            })
        };
        let parts = |wire_type| InRepresentation::Parts {
            via: Via::Fields,
            wire_type,
            read: Operation::Standard(StandardOp::ReadMember),
        };
        let (into_rust, out_of_rust) = match shape {
            Shape::Struct(name) => (
                parts(aggregate(&mut self.binding, name, Kind::Aggregate, false)),
                None,
            ),
            Shape::FallibleStruct(name) => (
                InRepresentation::Parts {
                    via: Via::Fields,
                    wire_type: aggregate(&mut self.binding, name, Kind::Aggregate, false),
                    read: Operation::Target(Op::ReadFallibly),
                },
                None,
            ),
            Shape::MirroredStruct(name) => (
                parts(aggregate(&mut self.binding, name, Kind::Aggregate, true)),
                None,
            ),
            Shape::StructOf(name, kind) => {
                (parts(aggregate(&mut self.binding, name, kind, false)), None)
            }
            Shape::Handle => {
                let pointer = self.binding.wire_type(Wire::Pointer {
                    name: quote::format_ident!("Raw"),
                });
                (
                    InRepresentation::Whole {
                        wire_type: pointer,
                        operation: Operation::Standard(StandardOp::FromRaw),
                    },
                    Some(OutRepresentation::Whole {
                        wire_type: pointer,
                        operation: Operation::Standard(StandardOp::IntoRaw),
                        release: Some(Operation::Standard(StandardOp::Release)),
                    }),
                )
            }
            Shape::SplitHandle => {
                let pointer = self.binding.wire_type(Wire::Pointer {
                    name: quote::format_ident!("Raw"),
                });
                let integer = self.binding.wire_type(Wire::Address);
                (
                    InRepresentation::Whole {
                        wire_type: pointer,
                        operation: Operation::Standard(StandardOp::FromRaw),
                    },
                    Some(OutRepresentation::Whole {
                        wire_type: integer,
                        operation: Operation::Standard(StandardOp::IntoRaw),
                        release: Some(Operation::Standard(StandardOp::Release)),
                    }),
                )
            }
            Shape::ScalarThrough => {
                let i64_wire = self.binding.wire_type(Wire::Scalar);
                (
                    InRepresentation::Whole {
                        wire_type: i64_wire,
                        operation: Operation::Target(Op::Rebase),
                    },
                    Some(OutRepresentation::Whole {
                        wire_type: i64_wire,
                        operation: Operation::Target(Op::Rebase),
                        release: None,
                    }),
                )
            }
        };
        Repr {
            into_rust: self.binding.in_representation(into_rust),
            out_of_rust: out_of_rust.map(|out| self.binding.out_representation(out)),
        }
    }

    /// A callback representation: a closure wire type holding scalars and
    /// pointers, a capture, and an invocation that calls as `call` says.
    fn callback_repr(&mut self, call: Call) -> Repr {
        let wire_type = self.binding.wire_type(Wire::Closure {
            kind: Kind::Closure,
            name: quote::format_ident!("Closure"),
        });
        let invoke = match call {
            Call::Infallible => Operation::Target(Op::Invoke),
            Call::NeedsContext => Operation::Target(Op::InvokeNeedingContext),
            Call::Routed | Call::Unrouted => Operation::Target(Op::InvokeFallibly),
        };
        let routes = match call {
            Call::Routed => vec![FailureRoute {
                category: FailureCategory::Runtime,
                report: Some(Report {
                    error: syn::parse_quote!(Error),
                    operation: Operation::Target(Op::Report),
                }),
                on_report_failure: Terminal::Abort,
                terminate: Terminal::Return(syn::parse_quote!(())),
            }],
            _ => Vec::new(),
        };
        Repr {
            into_rust: self.binding.in_representation(InRepresentation::Callable {
                wire_type,
                capture: Operation::Target(Op::Capture),
                invoke,
                routes,
            }),
            out_of_rust: None,
        }
    }

    /// A declared callback signature, as the C frontend's `callback!` states
    /// one: the rule for every value of it, and the output exposing it.
    fn declare_callback(&mut self, args: &str, call: Call) -> Repr {
        let repr = self.callback_repr(call);
        let key = callback_key(args);
        self.crossing_key(key.clone(), repr);
        self.outputs.push((
            Declaration::Callback(key),
            OutputForm::Type {
                into_rust: repr.into_rust,
                out_of_rust: None,
                release: None,
                meta: (),
            },
        ));
        repr
    }

    /// Every value of this type crosses as `repr` — without asking for the
    /// type itself to be declared.
    fn crossing(&mut self, name: &str, repr: Repr) -> &mut Self {
        self.crossing_key(key(name), repr)
    }

    /// The same, for a type named by its key.
    fn crossing_key(&mut self, key: prebindgen_flat::TypeKey, repr: Repr) -> &mut Self {
        for id in repr.ids() {
            self.rules.push((Scope::Type(key.clone()), id));
        }
        self
    }

    /// A declared type: how its values cross, and the output exposing that
    /// representation. The two are one declarator in both real frontends. A
    /// handle gets a release exported as `<name>_free`.
    fn declare_type(&mut self, name: &str, shape: Shape) -> Repr {
        let repr = self.repr(shape);
        self.crossing(name, repr);
        self.expose(name, repr);
        repr
    }

    /// An output exposing `repr` of the type `name`, with a release when the
    /// out-of-Rust representation has one.
    fn expose(&mut self, name: &str, repr: Repr) -> &mut Self {
        let handed_out = repr
            .out_of_rust
            .map(|id| self.binding.out_representation_of(id));
        let release = match handed_out {
            Some(OutRepresentation::Whole {
                release: Some(_), ..
            }) => {
                let item = key(name).short_name().expect("a named type");
                let mut release = exported(&format!("{item}_free"), Routes::None);
                release.inputs = vec![quote::format_ident!("arg0")];
                Some(release)
            }
            _ => None,
        };
        self.expose_with(name, repr, release)
    }

    /// The same, with the release form stated.
    fn expose_with(
        &mut self,
        name: &str,
        repr: Repr,
        release: Option<FunctionForm<Op>>,
    ) -> &mut Self {
        self.outputs.push((
            ty(name),
            OutputForm::Type {
                into_rust: repr.into_rust,
                out_of_rust: repr.out_of_rust,
                release,
                meta: (),
            },
        ));
        self
    }

    /// An exported source function.
    fn declare_fn(&mut self, name: &str, form: FunctionForm<Op>) -> &mut Self {
        self.outputs
            .push((function(name), OutputForm::Function { form, meta: () }));
        self
    }

    /// One requested output, as the binding declared it.
    fn declare(&mut self, declaration: Declaration, form: OutputFormOf<Mini>) -> &mut Self {
        self.outputs.push((declaration, form));
        self
    }

    /// How one value inside one function crosses, overriding its type's rule:
    /// `("stamp_max", [param stamp])`, `("stamp_max", [param stamp, field secs])`.
    fn at(&mut self, function_name: &str, path: Vec<Step>, repr: Repr) -> &mut Self {
        self.at.push((function(function_name), path, repr));
        self
    }

    /// The binding, with wrapper parameters named `argN` where a test named
    /// none, and each rule at a position resolved to the output it names.
    fn build(self, flat: &Flat) -> Binding<Mini> {
        let Fixture {
            mut binding,
            outputs,
            mut rules,
            at,
            ..
        } = self;
        let mut ids = Vec::new();
        for (declaration, mut form) in outputs {
            if let (Declaration::Function(ident), OutputForm::Function { form, .. }) =
                (&declaration, &mut form)
            {
                if form.inputs.is_empty() {
                    let params = flat
                        .function(&ident.to_string())
                        .map(|function| function.params.len())
                        .unwrap_or_default();
                    form.inputs = (0..params)
                        .map(|index| quote::format_ident!("arg{index}"))
                        .collect();
                }
            }
            ids.push((declaration.clone(), binding.output(declaration, form)));
        }
        for (declaration, path, repr) in at {
            let (_, output) = ids
                .iter()
                .find(|(declared, _)| *declared == declaration)
                .expect("a rule at a position names a declared output");
            let id = repr.at(&path);
            rules.push((Scope::At(*output, ValuePath(path)), id));
        }
        for (scope, repr) in rules {
            binding.rule(scope, repr);
        }
        binding
    }

    /// Plan this binding over `flat`.
    fn generate(self, flat: Flat) -> Result<Generation<Mini>, EngineError> {
        let binding = self.build(&flat);
        generate(flat, &Mini, binding, syn::parse_quote!(source))
    }
}

fn key(name: &str) -> prebindgen_flat::TypeKey {
    prebindgen_flat::TypeKey::parse(name).expect("a test names a type")
}

fn ty(name: &str) -> Declaration {
    Declaration::Type(key(name))
}

fn function(name: &str) -> Declaration {
    Declaration::Function(syn::parse_str(name).expect("a test names an ident"))
}

fn param(name: &str) -> Step {
    Step::Param(name.to_string())
}

fn field(name: &str) -> Step {
    Step::Field(name.to_string())
}

/// What became of the declaration that prints as `id`.
///
/// Skipped when the run left it out, emitted when it survived, and a panic
/// when the binding never asked for it — so a typo in a test cannot read as
/// success. An entity declared twice has two outputs under one printed name; a
/// test that declares one twice counts the skips instead of naming them.
fn outcome(generation: &Generation<Mini>, id: &str) -> Outcome {
    if let Some((_, skip)) = generation
        .skipped()
        .iter()
        .find(|(declaration, _)| declaration.to_string() == id)
    {
        return Outcome::Skipped(skip.clone());
    }
    assert!(
        generation
            .retained()
            .iter()
            .any(|retained| retained.declaration.to_string() == id),
        "nothing was declared as {id}"
    );
    Outcome::Emitted
}

/// How many declarations the run generated.
fn emitted(generation: &Generation<Mini>) -> usize {
    generation.retained().len()
}

/// Two exported functions taking the same struct the same way share its
/// conversion — and its two field conversions, and the result conversion.
#[test]
fn one_conversion_serves_every_value_that_crosses_the_same_way() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Stamp` into Rust, `i64` into Rust, `i64` out of Rust. Twice over, and
    // once for the struct's own request, is still three.
    assert_eq!(generation.values().len(), 3);
    assert_eq!(generation.functions().len(), 2);
}

/// A rule at a position gives that value a different representation, so it is
/// a different conversion — which is why the representation is part of a
/// node's identity.
#[test]
fn a_rule_at_a_position_does_not_share_the_type_rule() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));
    let fallible = fixture.repr(Shape::FallibleStruct("Stamp"));
    fixture.expose("Stamp", fallible);
    fixture.at("stamp_max", vec![param("stamp")], fallible);

    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 4, "{:?}", generation.skipped());
    // The two `Stamp` conversions are distinct; their `i64` children still are
    // not, because nothing overrode them.
    assert_eq!(generation.values().len(), 4);
    let fallible = generation
        .functions()
        .iter()
        .find(|plan| plan.symbol == "stamp_max")
        .expect("stamp_max is emitted");
    // Both routes the form declared travel with the plan, whether or not a
    // conversion here raises their category.
    assert_eq!(fallible.routes.len(), 2);
    assert!(generation.rust().contains("read(arg0)"));
    assert!(generation.rust().contains("report(error)"));
}

/// A value no rule covers is refused, and takes down its struct and everything
/// requiring it, and leaves everything else alone.
#[test]
fn a_field_no_rule_covers_skips_its_struct_and_its_callers() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_type("Label", Shape::Struct("Label"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    assert!(matches!(
        outcome(&generation, "type:Stamp"),
        Outcome::Emitted
    ));
    assert!(matches!(
        outcome(&generation, "fn:stamp_sum"),
        Outcome::Emitted
    ));
    let Outcome::Skipped(strukt) = outcome(&generation, "type:Label") else {
        panic!("`Label` has a field nothing covers");
    };
    assert_eq!(strukt.capability.as_str(), "unsupported.conversion.no_rule");
    let Outcome::Skipped(caller) = outcome(&generation, "fn:label_len") else {
        panic!("a function taking `Label` cannot be generated either");
    };
    // One cause, two casualties, each with its own path to it.
    assert_eq!(caller.capability.as_str(), "unsupported.conversion.no_rule");
    assert_eq!(caller.dependency_path.first().unwrap(), "fn:label_len");
    // Nothing partial is emitted: no wrapper for the skipped function.
    assert_eq!(generation.functions().len(), 1);
    assert!(!generation.rust().contains("label_len"));
}

/// An item the binding defines itself is an entity like a captured one, and
/// the writer reaches it where the binding said it is.
///
/// What differs is that the model was told about `stamp_zero` by the binding,
/// with a signature and a path, rather than by a capture. The wrapper it gets
/// is called through that path, and a type the binding declared over one the
/// source never exported is a handle like any captured alias.
#[test]
fn an_entity_the_binding_defines_is_planned_and_reached_where_it_says() {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![(
        syn::parse_quote!(
            pub fn token_use(token: Token) -> i64 {
                unimplemented!()
            }
        ),
        location,
    )];
    let flat = Flat::builder()
        .items(items)
        .local_function(
            syn::parse_quote!(fn stamp_zero() -> i64),
            syn::parse_quote!(crate::helpers),
        )
        // `Token` is what the source function takes and never exported; the
        // binding gives it a handle representation.
        .local_type(syn::parse_quote!(Token))
        .build()
        .expect("the fixture builds a model");
    assert!(flat.is_binding_local("stamp_zero"));
    assert!(flat.is_binding_local("Token"));
    assert_eq!(flat.captured().count(), 1);

    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    fixture.declare_fn("token_use", exported("token_use", Routes::Reported));
    fixture.declare_fn("stamp_zero", exported("stamp_zero", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    // The report counts the API it was generated against, which the
    // binding's own items are not part of.
    assert_eq!(generation.flat().captured().count(), 1);
    let rust = generation.rust();
    assert!(rust.contains("crate::helpers::stamp_zero()"), "{rust}");
    assert!(rust.contains("source::token_use("), "{rust}");
    assert!(rust.contains("as *mut crate::Token"), "{rust}");
}

/// A helper's signature is normalized like a captured one: `fixture::Stamp`
/// in what the binding stated — the captured crate's own spelling — is the
/// `Stamp` the captured struct is indexed as, and the helper is a function
/// the engine can plan.
#[test]
fn a_local_function_names_captured_types_as_the_source_spells_them() {
    let flat = Flat::builder()
        .items(model_items())
        .local_function(
            syn::parse_quote!(fn stamp_twice(stamp: fixture::Stamp) -> i64),
            syn::parse_quote!(crate::helpers),
        )
        .build()
        .expect("the fixture builds a model");
    let helper = flat.function("stamp_twice").unwrap_or_else(|| {
        panic!(
            "a local function is an element: {:?}",
            flat.element("stamp_twice")
        )
    });
    assert_eq!(helper.params[0].ty.key().as_str(), "Stamp");

    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_twice", exported("stamp_twice", Routes::None));
    let generation = fixture.generate(flat).expect("plans");
    assert_eq!(emitted(&generation), 2, "{:?}", generation.skipped());
    assert!(
        generation
            .rust()
            .contains("crate::helpers::stamp_twice(v2)"),
        "{}",
        generation.rust()
    );
}

/// A declared type's key may carry arguments the item does not:
/// `Token<'static>` names the item `Token`. And a conversion names no entity
/// at all — it is the binding's own wire mapping about a type, `Option<Foo>`
/// or otherwise — so it needs nothing in the model to be requested.
#[test]
fn a_type_key_with_arguments_names_its_item_and_a_conversion_names_none() {
    let mut fixture = fixture();
    // The item is `Token`; the rule is recorded under the key as declared,
    // and the type's own planning has to find it there.
    fixture.declare_type("Token<'static>", Shape::Handle);
    fixture.declare(
        Declaration::Conversion(key("Option<Stamp>")),
        OutputForm::Unsupported(Unsupported::new("unsupported.mini.convert", "no")),
    );
    let generation = fixture.generate(model()).expect("both are valid requests");
    assert!(
        matches!(
            outcome(&generation, "type:Token < 'static >"),
            Outcome::Emitted
        ),
        "{:?}",
        generation.skipped()
    );
    assert!(
        generation.rust().contains("as *mut source::Token<'static>"),
        "the release is over the type as declared:\n{}",
        generation.rust()
    );
    let Outcome::Skipped(skip) = outcome(&generation, "conversion:Option < Stamp >") else {
        panic!("a conversion is a capability the engine lacks, not a missing item");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.conversion.not_implemented"
    );
}

/// One function declared twice: each declaration is planned under its own
/// form and exports its own wrapper. Sharing is by conversion, as between two
/// different functions — the `Stamp` both take crosses the same way and is
/// planned once.
#[test]
fn one_function_declared_twice_is_two_outputs() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum_a", Routes::None));
    fixture.declare_fn("stamp_sum", exported("stamp_sum_b", Routes::Reported));

    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    let mut declared: Vec<String> = generation
        .retained()
        .iter()
        .map(|retained| retained.declaration.to_string())
        .collect();
    declared.sort();
    assert_eq!(declared, ["fn:stamp_sum", "fn:stamp_sum", "type:Stamp"]);
    let symbols: Vec<&str> = generation
        .functions()
        .iter()
        .map(|plan| plan.symbol.as_str())
        .collect();
    assert_eq!(symbols, ["stamp_sum_a", "stamp_sum_b"]);
    // `Stamp` into Rust once, `i64` each way once: the two wrappers share
    // every conversion, because nothing about the value differs.
    assert_eq!(generation.values().len(), 3);

    // One entity declared twice in the same form is the binding saying one
    // thing twice: two plans for one foreign declaration.
    let mut twice = self::fixture();
    twice.declare_type("Stamp", Shape::Struct("Stamp"));
    twice.declare_fn("stamp_sum", exported("stamp_sum_a", Routes::None));
    twice.declare_fn("stamp_sum", exported("stamp_sum_a", Routes::None));
    let error = twice
        .generate(model())
        .expect_err("one form, one declaration");
    assert!(matches!(error, EngineError::DuplicateDeclaration { .. }));
}

/// A value crossing as a representation no type output exposes is skipped,
/// even though the type is exposed another way: the foreign signature would
/// name a type the output does not declare.
#[test]
fn a_value_is_skipped_when_no_output_exposes_its_representation() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let handle = fixture.repr(Shape::Handle);
    fixture.at("stamp_max", vec![param("stamp")], handle);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));

    let generation = fixture.generate(model()).expect("plans");
    assert!(matches!(
        outcome(&generation, "fn:stamp_sum"),
        Outcome::Emitted
    ));
    match outcome(&generation, "fn:stamp_max") {
        Outcome::Skipped(skip) => {
            assert_eq!(
                skip.capability.as_str(),
                "unsupported.requirement.unrequested"
            );
            assert!(skip.explanation.contains("no type output"), "{skip:?}");
        }
        other => panic!("stamp_max is skipped, and is {other:?}"),
    }
}

/// One type exposed as two representations, and a value requires the one it
/// crosses as.
///
/// `Stamp` is exposed as a struct and as a handle. Values of it cross as a
/// struct by the type's rule; `stamp_max`'s parameter crosses as a handle by a
/// rule at its position. Each function requires the output exposing the
/// representation its value crosses as — so when the handle cannot be placed,
/// `stamp_max` goes with it and `stamp_sum` does not.
#[test]
fn a_value_requires_the_output_exposing_its_representation() {
    let plan = |release: bool| {
        let mut fixture = fixture();
        let strukt = fixture.repr(Shape::Struct("Stamp"));
        let handle = fixture.repr(Shape::Handle);
        fixture.expose("Stamp", strukt);
        match release {
            true => fixture.expose("Stamp", handle),
            false => fixture.expose_with("Stamp", handle, None),
        };
        fixture.crossing("Stamp", strukt);
        fixture.at("stamp_max", vec![param("stamp")], handle);
        fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
        fixture.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));
        fixture.generate(model()).expect("plans")
    };

    // Both exposed: everything is emitted, and each function's requirement
    // went to the one it crosses as.
    let generation = plan(true);
    assert_eq!(emitted(&generation), 4, "{:?}", generation.skipped());
    let rust = generation.rust();
    assert!(
        rust.contains("pub extern \"C\" fn stamp_sum(arg0: Stamp)"),
        "{rust}"
    );
    assert!(
        rust.contains("pub extern \"C\" fn stamp_max(arg0: *mut Raw)"),
        "{rust}"
    );

    // The handle exposed with no release to export: only the function
    // crossing that way requires it, and only that function is skipped.
    let generation = plan(false);
    let skipped: Vec<(String, &str)> = generation
        .skipped()
        .iter()
        .map(|(declaration, skip)| (declaration.to_string(), skip.capability.as_str()))
        .collect();
    // Declaration order: the handle first, then the function that crosses as
    // it.
    assert_eq!(
        skipped,
        [
            ("type:Stamp".to_string(), "unsupported.type.no_release"),
            ("fn:stamp_max".to_string(), "unsupported.type.no_release"),
        ]
    );
    assert_eq!(
        generation
            .retained()
            .iter()
            .filter(|retained| retained.declaration.to_string() == "type:Stamp")
            .count(),
        1,
        "the struct declaration of `Stamp` is emitted"
    );
    assert!(matches!(
        outcome(&generation, "fn:stamp_sum"),
        Outcome::Emitted
    ));
    let (_, skip) = &generation.skipped()[1];
    assert_eq!(skip.dependency_path, ["fn:stamp_max", "type:Stamp"]);
}

/// A value crossing as a representation no output of its type exposes
/// requires what is not there, when the type has several outputs to choose
/// from.
#[test]
fn a_value_crossing_as_no_exposed_representation_is_skipped() {
    let mut fixture = fixture();
    let strukt = fixture.repr(Shape::Struct("Stamp"));
    let handle = fixture.repr(Shape::Handle);
    let fallible = fixture.repr(Shape::FallibleStruct("Stamp"));
    fixture.expose("Stamp", strukt);
    fixture.expose("Stamp", handle);
    fixture.crossing("Stamp", strukt);
    fixture.at("stamp_max", vec![param("stamp")], fallible);
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_max") else {
        panic!("neither output of `Stamp` exposes what its parameter crosses as");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.requirement.unrequested"
    );
}

/// A type whose conversion needs its own is refused, rather than recursed on
/// until the stack runs out.
#[test]
fn a_conversion_that_needs_its_own_is_refused() {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
        syn::parse_quote!(
            pub struct Node {
                pub head: i64,
                pub tail: Node,
            }
        ),
        syn::parse_quote!(
            pub fn node_sum(node: Node) -> i64 {
                unimplemented!()
            }
        ),
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    let flat = Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model");

    let mut fixture = fixture();
    fixture.declare_type("Node", Shape::Struct("Node"));
    fixture.declare_fn("node_sum", exported("node_sum", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "type:Node") else {
        panic!("reading `Node`'s fields needs a `Node` conversion");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.conversion.recursive");
    // And the caller goes with it, by its own path to the same cause.
    let Outcome::Skipped(caller) = outcome(&generation, "fn:node_sum") else {
        panic!("a function taking `Node` cannot be generated either");
    };
    assert_eq!(
        caller.capability.as_str(),
        "unsupported.conversion.recursive"
    );
    assert!(generation.rust().is_empty());
}

/// A function whose value needs a type the binding never declared is skipped,
/// not emitted against a type that will not exist.
#[test]
fn a_function_needing_an_undeclared_public_type_is_skipped() {
    let mut fixture = fixture();
    // Its values cross, and the type itself was never asked for.
    let strukt = fixture.repr(Shape::Struct("Stamp"));
    fixture.crossing("Stamp", strukt);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("the aggregate it takes is not a declared public type");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.requirement.unrequested"
    );
    assert!(generation.rust().is_empty());
}

/// A conversion that can fail needs a route. A form that declares none does not
/// get a default — the function is skipped and the report says why.
#[test]
fn a_declared_failure_with_no_route_skips_the_function() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::FallibleStruct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("its member reads can fail and the form routes nothing");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.unrouted_failure"
    );
    // The struct itself is unaffected: its conversion is fine, and it is the
    // wrapper that could not be assembled.
    assert!(matches!(
        outcome(&generation, "type:Stamp"),
        Outcome::Emitted
    ));
}

/// Contradictory configuration fails the build; it is never turned into a
/// capability the engine claims to be missing.
#[test]
fn a_type_form_on_an_exported_function_is_an_error() {
    let mut fixture = fixture();
    let strukt = fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    // A type's form where a function's belongs.
    fixture.declare(
        function("stamp_sum"),
        OutputForm::Type {
            into_rust: strukt.into_rust,
            out_of_rust: None,
            release: None,
            meta: (),
        },
    );

    let error = fixture.generate(model()).expect_err("refuses");
    assert!(matches!(
        error,
        EngineError::Planning(PlanningError::InvalidInput(_))
    ));
}

/// A declaration the source never captured is an error, not a skip.
#[test]
fn a_declaration_naming_nothing_is_an_error() {
    let mut fixture = fixture();
    fixture.declare_fn("nope", exported("nope", Routes::None));

    let error = fixture.generate(model()).expect_err("refuses");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
}

/// Two runs over unchanged inputs produce the same report and the same code, so
/// a diff of either means something.
#[test]
fn a_run_over_unchanged_input_produces_the_same_output() {
    let run = || {
        let mut fixture = fixture();
        fixture.declare_type("Stamp", Shape::Struct("Stamp"));
        fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
        let generation = fixture.generate(model()).expect("plans");
        (
            format!("{:?}", generation.skipped()),
            generation.rust().to_string(),
        )
    };
    assert_eq!(run(), run());
}

/// A rule recorded for a *field* makes its struct a different conversion,
/// whichever order the two uses are planned in.
///
/// The cache is consulted after the children are planned for exactly this
/// reason: keyed on the struct's own representation alone, the second use
/// would inherit the first one's conversion and its support outcome, in
/// whichever direction the two happened to be requested.
#[test]
fn a_field_rule_is_part_of_its_struct_conversion() {
    let plan = |defaults_first: bool| {
        let mut fixture = fixture();
        let strukt = fixture.repr(Shape::Struct("Stamp"));
        fixture.crossing("Stamp", strukt);
        // Read through fields, on a scalar field: an `i64` has no fields, so
        // this child cannot be planned at all. A representation of its own,
        // since `Stamp`'s converts `Stamp` and nothing else.
        let fields = fixture.repr(Shape::Struct("Secs"));
        fixture.at("stamp_max", vec![param("stamp"), field("secs")], fields);
        // Order matters twice over: which function is planned first, and
        // whether the struct's own request primed the conversion before either
        // of them. The refusal-first case must not be primed, or it would not
        // test what happens when a refusal is met before any success.
        if defaults_first {
            fixture.expose("Stamp", strukt);
            fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
            fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));
        } else {
            fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));
            fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
            fixture.expose("Stamp", strukt);
        }
        fixture.generate(model()).expect("plans")
    };
    for defaults_first in [true, false] {
        let generation = plan(defaults_first);
        let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_max") else {
            panic!("its `secs` field is ruled a way nothing can serve");
        };
        assert_eq!(skip.capability.as_str(), "unsupported.type.not_a_struct");
        // The function that configured nothing keeps its conversion.
        assert!(matches!(
            outcome(&generation, "fn:stamp_sum"),
            Outcome::Emitted
        ));
        assert!(generation.rust().contains("stamp_sum"));
        assert!(!generation.rust().contains("stamp_max"));
    }
}

/// A temporary never shadows a parameter the wrapper still needs.
///
/// A form is free to name a parameter `v0`, and a temporary taking that name
/// would compile and feed the wrong value to the source call.
#[test]
fn a_temporary_never_takes_a_live_parameter_name() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let mut form = exported("stamp_pick", Routes::None);
    // The names a writer would otherwise allocate for itself.
    form.inputs = vec![quote::format_ident!("v0"), quote::format_ident!("v1")];
    fixture.declare_fn("stamp_pick", form);

    let generation = fixture.generate(model()).expect("plans");
    assert!(matches!(
        outcome(&generation, "fn:stamp_pick"),
        Outcome::Emitted
    ));
    let rust = generation.rust();
    // The second argument is the parameter itself, not a temporary that took
    // its name: `v1` reaches the call unshadowed, and the fields are read into
    // names the signature does not use.
    assert!(rust.contains("let v2 = v0.secs;"), "{rust}");
    assert!(rust.contains("let v3 = v0.nanos;"), "{rust}");
    assert!(
        rust.contains("source::stamp_pick(v4, v1)"),
        "the fallback argument must still be the parameter:\n{rust}"
    );
}

/// A failure route whose reporting operation needs a context the form does
/// not supply skips the function.
///
/// The conversions here need no context at all, so only the reporter can
/// reveal it — which is what makes this different from the conversion-side
/// check.
#[test]
fn a_reporter_needing_an_unsupplied_context_skips_the_function() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::FallibleStruct("Stamp"));
    fixture.declare_fn(
        "stamp_sum",
        exported("stamp_sum", Routes::ReporterNeedsContext),
    );

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("its reporter needs a context this form does not supply");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.missing_context"
    );
}

/// A skip says where the walk stopped, as a value path: which parameter of
/// which exported function, and which field inside it.
#[test]
fn a_skip_names_the_parameter_and_the_field_that_stopped_it() {
    let mut fixture = fixture();
    let strukt = fixture.repr(Shape::Struct("Stamp"));
    fixture.crossing("Stamp", strukt);
    fixture.declare_type("Label", Shape::Struct("Label"));
    fixture.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(caller) = outcome(&generation, "fn:label_len") else {
        panic!("`Label` has a field nothing covers");
    };
    assert_eq!(
        caller.dependency_path,
        vec!["fn:label_len", "param label", "field text"]
    );
    // The struct reached the same cause by its own path.
    let Outcome::Skipped(strukt) = outcome(&generation, "type:Label") else {
        panic!("the struct cannot be represented either");
    };
    assert_eq!(strukt.dependency_path, vec!["type:Label", "field text"]);
}

/// A type output the frontend refuses skips every declaration requiring it,
/// even though every conversion involved was planned successfully.
///
/// This is the retention loop rather than value planning: `Stamp`'s values
/// cross fine, and it is the public `Stamp` that does not exist.
#[test]
fn a_refused_type_output_skips_what_requires_it() {
    let mut fixture = fixture();
    let strukt = fixture.repr(Shape::Struct("Stamp"));
    fixture.crossing("Stamp", strukt);
    fixture.declare(
        ty("Stamp"),
        OutputForm::Unsupported(Unsupported::new(
            "unsupported.mini.no_public_struct",
            "this struct converts, and has no public declaration here",
        )),
    );
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    // Unrelated, and skipped for a cause of its own: `Label` holds a `String`,
    // which nothing here covers.
    fixture.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(strukt) = outcome(&generation, "type:Stamp") else {
        panic!("this target declares no public struct");
    };
    assert_eq!(
        strukt.capability.as_str(),
        "unsupported.mini.no_public_struct"
    );
    let Outcome::Skipped(caller) = outcome(&generation, "fn:stamp_sum") else {
        panic!("a wrapper taking a type that is not declared is unusable");
    };
    // The same cause, reached through this function's own requirement.
    assert_eq!(
        caller.capability.as_str(),
        "unsupported.mini.no_public_struct"
    );
    assert_eq!(caller.dependency_path, ["fn:stamp_sum", "type:Stamp"]);
    assert!(generation.rust().is_empty());
}

/// Two supported but different child conversions make two struct conversions.
///
/// Nothing is refused here, so this says the children belong to a conversion's
/// identity on their own rather than only when one of them fails.
#[test]
fn two_supported_children_make_two_struct_conversions() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    // One field of one parameter crosses the same `i64` a different way. The
    // struct above it is ruled identically in both functions, so only the
    // child can make the two conversions differ.
    let through = fixture.repr(Shape::ScalarThrough);
    fixture.at("stamp_max", vec![param("stamp"), field("secs")], through);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Stamp` twice, `i64` into Rust twice, `i64` out of Rust once.
    assert_eq!(generation.values().len(), 5);
    assert_eq!(generation.functions().len(), 2);
    // And the second conversion is the one that was asked for, not the first
    // one shared under a different name.
    let rust = generation.rust();
    assert_eq!(rust.matches("rebase(").count(), 1, "{rust}");
}

/// A rule restating the type's own representation shares its conversion,
/// however far apart the two are: equal representations are one id, and the
/// id is what a conversion's identity holds.
#[test]
fn a_rule_restating_the_type_rule_shares_its_conversion() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let scalar = fixture.scalar;
    fixture.at("stamp_max", vec![param("stamp"), field("secs")], scalar);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    // The same three as if nothing had been recorded for that field.
    assert_eq!(generation.values().len(), 3);
}

/// Declaring a representation equal to one already declared returns its id, so
/// equal settings are equal data wherever they were stated.
#[test]
fn equal_representations_are_one_representation() {
    let mut fixture = fixture();
    let first = fixture.repr(Shape::Struct("Stamp"));
    let again = fixture.repr(Shape::Struct("Stamp"));
    let other = fixture.repr(Shape::FallibleStruct("Stamp"));
    assert_eq!(first, again);
    assert_ne!(first, other);
}

/// A raw identifier and its plain spelling are one name, and the writer treats
/// them as one.
///
/// `r#v0` reserves `v0`: a temporary that took the plain spelling would shadow
/// the parameter, which is the same defect as an ordinary collision wearing a
/// different hat.
#[test]
fn a_raw_identifier_parameter_reserves_its_plain_spelling() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let mut form = exported("stamp_pick", Routes::None);
    form.inputs = vec![
        syn::parse_str("r#v0").expect("a raw ident"),
        syn::parse_str("r#v1").expect("a raw ident"),
    ];
    fixture.declare_fn("stamp_pick", form);

    let generation = fixture.generate(model()).expect("plans");
    let rust = generation.rust();
    assert!(
        !rust.contains("let v0 ="),
        "`v0` names the parameter:\n{rust}"
    );
    assert!(
        !rust.contains("let v1 ="),
        "`v1` names the parameter:\n{rust}"
    );
    assert!(
        rust.contains("source::stamp_pick(v4, r#v1)"),
        "the fallback argument must still be the parameter:\n{rust}"
    );
}

/// A refusal travels every dependency edge, not just the first.
///
/// The chain is declared so that propagation needs more than one pass: the
/// function is decided before the struct it requires, and that struct before
/// the one *it* requires — through a field.
#[test]
fn a_refusal_travels_a_chain_of_requirements() {
    let mut fixture = fixture();
    // Every conversion here succeeds: what fails is a type output, two edges
    // away from the function that needs it.
    fixture.declare_fn("wrap_sum", exported("wrap_sum", Routes::None));
    fixture.declare_type("Wrap", Shape::Struct("Wrap"));
    let stamp = fixture.repr(Shape::Struct("Stamp"));
    fixture.crossing("Stamp", stamp);
    fixture.declare(
        ty("Stamp"),
        OutputForm::Unsupported(Unsupported::new(
            "unsupported.mini.no_public_struct",
            "this struct converts, and has no public declaration here",
        )),
    );

    let generation = fixture.generate(model()).expect("plans");
    for declaration in ["type:Stamp", "type:Wrap", "fn:wrap_sum"] {
        let Outcome::Skipped(skip) = outcome(&generation, declaration) else {
            panic!("{declaration} depends on a type output this target refuses");
        };
        assert_eq!(
            skip.capability.as_str(),
            "unsupported.mini.no_public_struct",
            "{declaration} carries the one cause"
        );
        assert_eq!(skip.dependency_path.first().unwrap(), declaration);
    }
    // Two edges, so the far end of the chain names both of them.
    let Outcome::Skipped(caller) = outcome(&generation, "fn:wrap_sum") else {
        unreachable!("checked above");
    };
    assert_eq!(
        caller.dependency_path,
        vec!["fn:wrap_sum", "type:Wrap", "type:Stamp"]
    );
    assert!(generation.rust().is_empty());
}

/// A form naming a wrapper parameter for each input it does not have is
/// contradictory input: the registry names the inputs' parameters from it.
#[test]
fn a_form_naming_the_wrong_number_of_inputs_is_an_error() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let mut form = exported("stamp_pick", Routes::None);
    form.inputs = vec![quote::format_ident!("only")];
    fixture.declare_fn("stamp_pick", form);

    let error = fixture.generate(model()).expect_err("refuses");
    let EngineError::Planning(PlanningError::InvalidInput(message)) = error else {
        panic!("a form that does not fit its function is invalid input");
    };
    assert!(
        message.contains("1 wrapper parameter(s) for 2 input(s)"),
        "{message}"
    );
}

/// Two rules for one value are the binding saying two things about it, and fail
/// the build before anything is planned — including a rule at a type output's
/// root, which the output's own representation already is.
#[test]
fn two_rules_for_one_value_are_an_error() {
    let mut fixture = fixture();
    let strukt = fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.crossing("Stamp", strukt);
    let error = fixture.generate(model()).expect_err("refuses");
    assert!(
        error
            .to_string()
            .contains("two rules cover every `Stamp` into Rust"),
        "{error}"
    );

    let mut fixture = self::fixture();
    let strukt = fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let mut binding = fixture.build(&model());
    binding.rule(
        Scope::At(binding.output_id(0), ValuePath::root()),
        strukt.into_rust,
    );
    let error = generate(model(), &Mini, binding, syn::parse_quote!(source)).expect_err("refuses");
    assert!(
        error
            .to_string()
            .contains("two rules cover `type:Stamp` itself into Rust"),
        "{error}"
    );
}

/// A rule at a position the output does not have fails the build, rather than
/// sitting unused while the binding believes it configured something.
#[test]
fn a_rule_at_a_position_that_does_not_exist_is_an_error() {
    let refused = |path: Vec<Step>| {
        let mut fixture = fixture();
        fixture.declare_type("Stamp", Shape::Struct("Stamp"));
        fixture.declare_type("Token", Shape::Handle);
        fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
        fixture.declare_fn("token_use", exported("token_use", Routes::Reported));
        let function = match &path[0] {
            Step::Param(name) if name == "token" => "token_use",
            _ => "stamp_sum",
        };
        let scalar = fixture.scalar;
        fixture.at(function, path, scalar);
        fixture
            .generate(model())
            .expect_err("a rule at no position is refused")
            .to_string()
    };
    assert!(
        refused(vec![param("nope")]).contains("the function takes no `nope`"),
        "a parameter the function does not take"
    );
    assert!(
        refused(vec![param("stamp"), field("nope")]).contains("`Stamp` has no field `nope`"),
        "a field the struct does not have"
    );
    assert!(
        refused(vec![param("token"), field("inner")])
            .contains("`Token` is not read through its fields there"),
        "a field of a value carried whole"
    );
}

/// A member whose part resolved to a wire type its wire type does not hold
/// refuses the struct, where the member is.
#[test]
fn a_member_its_wire_type_cannot_have_refuses_the_value() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::StructOf("Stamp", Kind::PointerAggregate));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "type:Stamp") else {
        panic!("a pointer aggregate cannot have a scalar member");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.mini.member.scalar");
    assert_eq!(skip.dependency_path, ["type:Stamp", "field secs"]);
    let Outcome::Skipped(caller) = outcome(&generation, "fn:stamp_sum") else {
        panic!("and a function taking it goes with it");
    };
    assert_eq!(
        caller.dependency_path,
        ["fn:stamp_sum", "param stamp", "field secs"]
    );
}

/// A value of a kind the target cannot pass to a wrapper refuses the
/// function, where the parameter is.
#[test]
fn a_parameter_the_target_cannot_pass_refuses_the_function() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::StructOf("Stamp", Kind::Local));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("the target cannot pass a local struct");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.mini.param.local");
    assert_eq!(skip.dependency_path, ["fn:stamp_sum", "param stamp"]);
}

/// A type rule no value was planned by is listed, not refused: a type rule is
/// a default for many values, and one a binding never needed is harmless.
#[test]
fn a_type_rule_nothing_used_is_listed() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let point = fixture.repr(Shape::Struct("Point"));
    fixture.crossing("Point", point);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let unused: Vec<String> = generation
        .unused_rules()
        .iter()
        .map(|scope| format!("{scope:?}"))
        .collect();
    assert_eq!(unused.len(), 1, "{unused:?}");
    assert!(unused[0].contains("Point"), "{unused:?}");
}

/// The binding prints every field planning reads, one line per
/// wire type, representation, rule and output.
#[test]
fn a_binding_prints_what_planning_reads() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let printed = fixture.build(&model()).to_string();
    for line in [
        "wire     w0  i64  scalar  Scalar",
        "wire     w1  Stamp  aggregate  Aggregate { kind: Aggregate, name: Ident(Stamp), mirror: false }",
        "repr     in0  whole  w0 Standard(Identity)",
        "repr     in1  parts  w1  Fields  read: Standard(ReadMember)",
        "repr     out0  whole  w0 Standard(Identity)",
        "rule     type i64  in0",
        "rule     type i64  out0",
        "rule     type Stamp  in1",
        "output   type:Stamp  type in1 -  ()",
        "output   fn:stamp_sum  function  ()",
        "         form  extern \"C\" stamp_sum  context []  inputs [arg0]",
    ] {
        assert!(printed.contains(line), "missing `{line}` in:\n{printed}");
    }
}

/// What planning reads and the printout left out would let two bindings print
/// alike and plan differently: a failing read, a route, what a holder accepts.
#[test]
fn bindings_that_plan_differently_print_differently() {
    let print = |shape: Shape, routes: Routes| {
        let mut fixture = fixture();
        fixture.declare_type("Stamp", shape);
        fixture.declare_fn("stamp_sum", exported("stamp_sum", routes));
        fixture.build(&model()).to_string()
    };
    let plain = print(Shape::Struct("Stamp"), Routes::None);
    for other in [
        print(Shape::FallibleStruct("Stamp"), Routes::None),
        print(Shape::Struct("Stamp"), Routes::Reported),
        print(
            Shape::StructOf("Stamp", Kind::ScalarAggregate),
            Routes::None,
        ),
    ] {
        assert_ne!(plain, other);
    }
}

/// A release frees what was handed out, so its wrapper takes the out-of-Rust
/// wire type even where the representation takes values back in through another.
#[test]
fn a_release_takes_the_wire_type_that_was_handed_out() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::SplitHandle);
    let generation = fixture.generate(model()).expect("plans");
    let rust = generation.rust();
    assert!(
        rust.contains("pub extern \"C\" fn Stamp_free(arg0: usize)"),
        "{rust}"
    );
}

/// An id is valid only in the binding that issued it, and handing one to
/// another binding fails where it is handed over, whether or not the same
/// index exists there.
#[test]
#[should_panic(expected = "a representation id issued by another binding")]
fn an_id_from_another_binding_is_refused() {
    let mut other = fixture();
    let foreign = other.repr(Shape::Struct("Stamp"));
    let mut fixture = fixture();
    fixture.repr(Shape::Struct("Stamp"));
    fixture
        .binding
        .rule(Scope::Type(key("Stamp")), foreign.into_rust);
}

/// A guard the capture reader injected reaches the generated file, whatever
/// else the run retained.
///
/// The one in production asserts that the source crate's features match the set
/// the capture was filtered by. It belongs to no declaration, so nothing in
/// retention decides its fate, and a run that emitted no wrapper at all must
/// still carry it.
#[test]
fn a_capture_guard_is_emitted_whatever_else_the_run_retains() {
    let guarded = || {
        let location = prebindgen::SourceLocation {
            crate_name: Some("fixture".to_string()),
            ..Default::default()
        };
        let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
            syn::parse_quote!(
                const _: () = {
                    konst::assertc_eq!(fixture::FEATURES, "fixture/unstable", "mismatch");
                };
            ),
            syn::parse_quote!(
                pub struct Stamp {
                    pub secs: i64,
                    pub nanos: i64,
                }
            ),
            syn::parse_quote!(
                pub fn stamp_sum(stamp: Stamp) -> i64 {
                    unimplemented!()
                }
            ),
        ]
        .into_iter()
        .map(|item| (item, location.clone()))
        .collect();
        Flat::builder()
            .items(items)
            .build()
            .expect("the fixture builds a model")
    };

    // With something to emit.
    let mut full = fixture();
    full.declare_type("Stamp", Shape::Struct("Stamp"));
    full.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let generation = full.generate(guarded()).expect("plans");
    assert_eq!(emitted(&generation), 2);
    assert!(
        generation.rust().contains("konst::assertc_eq!"),
        "the guard must reach the generated file:\n{}",
        generation.rust()
    );

    // And with nothing to emit: the declaration is skipped, and the guard is
    // still there.
    let mut bare = fixture();
    bare.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let generation = bare.generate(guarded()).expect("plans");
    assert_eq!(emitted(&generation), 0);
    assert!(
        generation.rust().contains("konst::assertc_eq!"),
        "a run that emitted nothing still carries its guard:\n{}",
        generation.rust()
    );
}

/// A small model of a struct and a function over it, some of it written under
/// conditions the capture reader could not answer.
fn conditioned(struct_item: syn::Item, function_item: syn::Item) -> Flat {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    Flat::builder()
        .items(vec![
            (struct_item, location.clone()),
            (function_item, location),
        ])
        .build()
        .expect("the fixture builds a model")
}

/// A condition the capture reader could not evaluate reaches the wrapper
/// generated for the item that carries it, so the wrapper exists exactly where
/// the function it calls does.
#[test]
fn a_condition_the_reader_could_not_evaluate_reaches_the_wrapper() {
    let flat = conditioned(
        syn::parse_quote!(
            pub struct Stamp {
                pub secs: i64,
                pub nanos: i64,
            }
        ),
        syn::parse_quote!(
            #[cfg(some_custom_flag)]
            pub fn stamp_sum(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
    );
    // The model takes the item as it is: an attribute it cannot interpret is
    // not a reason to refuse one.
    assert_eq!(flat.unsupported().count(), 0);
    assert!(flat.function("stamp_sum").is_some());

    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    assert_eq!(emitted(&generation), 2);
    let rust = generation.rust();
    assert!(
        rust.contains("#[cfg(some_custom_flag)]"),
        "the wrapper carries the condition of the function it calls:\n{rust}"
    );
    assert_eq!(
        rust.matches("cfg").count(),
        1,
        "and nothing else in the file acquires one:\n{rust}"
    );
}

/// The same, for a struct the wrapper constructs rather than the function it
/// calls: the wrapper names both, so it inherits from both, and one condition
/// two of them carry is stated once.
#[test]
fn a_wrapper_inherits_the_condition_of_every_source_item_it_names() {
    let flat = conditioned(
        syn::parse_quote!(
            #[cfg(some_custom_flag)]
            #[cfg(another_custom_flag)]
            pub struct Stamp {
                pub secs: i64,
                pub nanos: i64,
            }
        ),
        syn::parse_quote!(
            #[cfg(some_custom_flag)]
            pub fn stamp_sum(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
    );

    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    let rust = generation.rust();
    assert_eq!(
        rust.matches("#[cfg(some_custom_flag)]").count(),
        1,
        "the condition both items carry is stated once:\n{rust}"
    );
    assert_eq!(
        rust.matches("#[cfg(another_custom_flag)]").count(),
        1,
        "and the one only the struct carries reaches the wrapper too:\n{rust}"
    );
}

/// A field's condition reaches everything the wrapper generates for that field,
/// and nothing it generates for the others — and the member the target
/// declares for it, which it is fed with the part.
#[test]
fn a_field_condition_reaches_every_statement_that_serves_the_field() {
    let flat = conditioned(
        syn::parse_quote!(
            pub struct Stamp {
                pub secs: i64,
                #[cfg(some_custom_flag)]
                pub nanos: i64,
            }
        ),
        syn::parse_quote!(
            pub fn stamp_sum(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
    );

    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::MirroredStruct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    let rust = generation.rust();
    // The member, the read, and the initializer — three places, one condition,
    // and the unconditional field in none of them.
    assert_eq!(
        rust.matches("#[cfg(some_custom_flag)]").count(),
        3,
        "the member, its read and its initializer each carry it:\n{rust}"
    );
    for under in [
        "#[cfg(some_custom_flag)]\n    pub nanos",
        "#[cfg(some_custom_flag)]\n    let v1 = arg0.nanos;",
        "#[cfg(some_custom_flag)]\n        nanos: v1,",
    ] {
        assert!(rust.contains(under), "missing:\n{under}\n\nin:\n{rust}");
    }
    assert!(
        rust.contains("let v0 = arg0.secs;"),
        "the unconditional field is untouched:\n{rust}"
    );
}

/// An item's condition reaches the declaration the target writes for a
/// wire type of that item, not only the wrapper.
///
/// A mirror emitted where the struct it mirrors is absent is a type the foreign
/// API declares and the build does not have.
#[test]
fn an_item_condition_reaches_the_declaration_a_target_writes() {
    let flat = conditioned(
        syn::parse_quote!(
            #[cfg(some_custom_flag)]
            pub struct Stamp {
                pub secs: i64,
                pub nanos: i64,
            }
        ),
        syn::parse_quote!(
            #[cfg(some_custom_flag)]
            pub fn stamp_sum(stamp: Stamp) -> i64 {
                unimplemented!()
            }
        ),
    );

    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::MirroredStruct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = fixture.generate(flat).expect("plans");
    let rust = generation.rust();
    // Once on the mirror, once on the wrapper. The fields carry none of their
    // own: the condition is the struct's.
    assert_eq!(
        rust.matches("#[cfg(some_custom_flag)]").count(),
        2,
        "the mirror and the wrapper, and nothing else:\n{rust}"
    );
    assert!(
        rust.contains("#[cfg(some_custom_flag)]\n#[repr(C)]"),
        "the mirror carries it:\n{rust}"
    );
}

/// The part of a wrapper's form a binding may state — attributes beyond
/// `#[no_mangle]`, and `unsafe` — is rendered as stated; the linkage is the
/// writer's, and a form restating it is contradictory input.
#[test]
fn a_form_states_a_wrappers_attributes_and_safety_but_not_its_linkage() {
    let form = |attrs: Vec<syn::Attribute>, unsafety: bool| {
        let mut fixture = fixture();
        fixture.declare_type("Stamp", Shape::Struct("Stamp"));
        let mut form = exported("stamp_sum", Routes::None);
        form.attrs = attrs;
        form.unsafety = unsafety;
        fixture.declare_fn("stamp_sum", form);
        fixture.generate(model())
    };

    let generation = form(vec![syn::parse_quote!(#[allow(non_snake_case)])], true).expect("plans");
    let rust = generation.rust();
    assert!(
        rust.contains(
            "#[no_mangle]\n#[allow(non_snake_case)]\npub unsafe extern \"C\" fn stamp_sum("
        ),
        "{rust}"
    );

    for linkage in [
        syn::parse_quote!(#[no_mangle]),
        syn::parse_quote!(#[export_name = "other"]),
    ] {
        let error = form(vec![linkage], false).expect_err("the linkage is the writer's");
        assert!(
            error.to_string().contains("whose linkage the writer owns"),
            "{error}"
        );
    }
}

/// An opaque type is carried as an address both ways, and gets a release of its
/// own; the two functions that hand a token out and take one back share those
/// conversions with the type's own request.
#[test]
fn a_handle_is_carried_both_ways_and_released() {
    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    fixture.declare_fn("token_new", exported("token_new", Routes::None));
    fixture.declare_fn("token_use", exported("token_use", Routes::Reported));

    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Token` out of Rust, `Token` into Rust, `i64` out of Rust — and the type's
    // own request planned nothing the functions did not.
    assert_eq!(generation.values().len(), 3);
    // Two exported functions, plus the release under the type's own identity.
    let symbols: Vec<&str> = generation
        .functions()
        .iter()
        .map(|plan| plan.symbol.as_str())
        .collect();
    assert_eq!(symbols, ["Token_free", "token_new", "token_use"]);
    let release = &generation.functions()[0];
    assert_eq!(release.declaration.to_string(), "type:Token");
    assert!(release.result.is_none(), "a release delivers nothing");

    let rust = generation.rust();
    assert!(
        rust.contains("Box::into_raw(Box::new(v0)) as *mut Raw"),
        "{rust}"
    );
    assert!(
        rust.contains("NonNull::new(arg0 as *mut source::Token)"),
        "{rust}"
    );
    assert!(rust.contains("report_message(error)"), "{rust}");
    // Taking a handle back moves the value out of its box; releasing one drops
    // the box without converting, and takes no route: null is not a failure
    // there.
    assert!(
        rust.contains("unsafe { *Box::from_raw(handle.as_ptr()) }"),
        "{rust}"
    );
    assert!(
        rust.contains("unsafe { Box::from_raw(handle.as_ptr()) }"),
        "{rust}"
    );
    assert!(release.routes.is_empty());
}

/// A null address arriving where a handle is consumed is a binding failure, and
/// the form has to say where it goes like any other.
#[test]
fn a_consumed_handle_needs_a_binding_route() {
    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    fixture.declare_fn("token_use", exported("token_use", Routes::None));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:token_use") else {
        panic!("a null handle can arrive and nothing routes it");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.unrouted_failure"
    );
    assert!(skip.explanation.contains("binding"), "{}", skip.explanation);
    assert!(matches!(
        outcome(&generation, "type:Token"),
        Outcome::Emitted
    ));
}

/// A handle nobody can free is not a handle: the type is skipped where its
/// release would be, and the function taking it with it.
#[test]
fn a_handle_without_a_release_skips_the_type_and_what_takes_it() {
    let mut fixture = fixture();
    let handle = fixture.repr(Shape::Handle);
    fixture.crossing("Token", handle);
    fixture.expose_with("Token", handle, None);
    fixture.declare_fn("token_use", exported("token_use", Routes::Reported));

    let generation = fixture.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "type:Token") else {
        panic!("its release has nowhere to go");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.type.no_release");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:token_use") else {
        panic!("it takes a type that was not declared");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.type.no_release");
    assert_eq!(skip.dependency_path, ["fn:token_use", "type:Token"]);
    assert!(generation.rust().is_empty());
}

/// A callback enters Rust as a closure the registry builds: the wire type is
/// captured once, and each call converts its arguments out of Rust and hands
/// them to the invocation.
#[test]
fn a_callback_enters_rust_as_a_closure_that_calls_through() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_callback("i64", Call::Infallible);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    let rust = generation.rust();
    let compact: String = rust.split_whitespace().collect();
    for expected in [
        "pubextern\"C\"fnstamp_each(arg0:Stamp,arg1:Closure)",
        "letv3=capture::<(i64,)>(arg1);",
        "letv5=move|v4:i64|{invoke(&v3,v4);};",
        "source::stamp_each(v2,v5);",
    ] {
        assert!(
            compact.contains(expected),
            "missing `{expected}` in:\n{rust}"
        );
    }
}

/// An argument crosses the other way from its callback, and takes the rule
/// its own type has: a handle is handed out, inside each call.
#[test]
fn a_callback_argument_leaves_rust_inside_each_call() {
    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    fixture.declare_callback("Token, i64", Call::Infallible);
    fixture.declare_fn("token_watch", exported("token_watch", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    let rust = generation.rust();
    let compact: String = rust.split_whitespace().collect();
    assert!(compact.contains("capture::<(*mutRaw,i64)>(arg0)"), "{rust}");
    assert!(
        compact.contains(
            "move|v1:source::Token,v2:i64|{letv3=Box::into_raw(Box::new(v1))as*mutRaw;invoke(&v0,v3,v2);}"
        ),
        "{rust}"
    );
}

/// A failure inside a call takes the callback's own route, not the wrapper's:
/// the closure has no caller to hand it to, and the wrapper routes nothing a
/// call raises.
#[test]
fn a_failure_inside_a_call_takes_the_callbacks_route() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_callback("i64", Call::Routed);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    let compact: String = generation.rust().split_whitespace().collect();
    assert!(
        compact.contains("ifletErr(error)=invoke(&v3,v4){report(error);return();}"),
        "{}",
        generation.rust()
    );
}

/// A call that can fail with no route of the callback's for it is refused
/// where the callback is, and takes down what requires it.
#[test]
fn an_unrouted_failure_inside_a_call_refuses_the_callback() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_callback("i64", Call::Unrouted);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::Reported));
    let generation = fixture.generate(model()).expect("plans");
    for id in ["callback:impl Fn(i64)+Send+Sync+'static", "fn:stamp_each"] {
        match outcome(&generation, id) {
            Outcome::Skipped(skip) => assert_eq!(
                skip.capability.as_str(),
                "unsupported.callback.unrouted_failure",
                "{id}: {skip:?}"
            ),
            other => panic!("{id} is skipped, and is {other:?}"),
        }
    }
}

/// An argument no rule carries out of Rust refuses the callback at that
/// argument, and the function taking it goes down with it.
#[test]
fn an_argument_that_cannot_leave_rust_refuses_the_callback() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_callback("Stamp", Call::Infallible);
    fixture.declare_fn("stamp_emit", exported("stamp_emit", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:stamp_emit") {
        Outcome::Skipped(skip) => {
            assert_eq!(skip.capability.as_str(), "unsupported.conversion.no_rule");
            assert!(skip.explanation.contains("out of Rust"), "{skip:?}");
            assert_eq!(
                skip.dependency_path.last().map(String::as_str),
                Some("arg 0")
            );
        }
        other => panic!("stamp_emit is skipped, and is {other:?}"),
    }
}

/// A function taking a callback no output declares is skipped, as one taking
/// an undeclared type is: the foreign signature would name a callable type
/// nothing emits.
#[test]
fn a_callback_no_output_declares_is_required_and_missing() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let repr = fixture.callback_repr(Call::Infallible);
    fixture.crossing_key(callback_key("i64"), repr);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:stamp_each") {
        Outcome::Skipped(skip) => {
            assert_eq!(
                skip.capability.as_str(),
                "unsupported.requirement.unrequested"
            );
            assert!(skip.explanation.contains("callback"), "{skip:?}");
        }
        other => panic!("stamp_each is skipped, and is {other:?}"),
    }
}

/// A rule can address one argument of one callback parameter, below the
/// parameter that takes it.
#[test]
fn a_rule_can_address_a_callback_argument() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let repr = fixture.callback_repr(Call::Infallible);
    let key = callback_key("i64");
    fixture.crossing_key(key.clone(), repr);
    fixture.declare(
        Declaration::Callback(key),
        OutputForm::Type {
            into_rust: repr.into_rust,
            out_of_rust: None,
            release: None,
            meta: (),
        },
    );
    let through = fixture.repr(Shape::ScalarThrough);
    fixture.at("stamp_each", vec![param("f"), Step::Arg(0)], through);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    let rust = generation.rust();
    assert!(rust.contains("rebase("), "{rust}");

    let mut wrong = self::fixture();
    wrong.declare_type("Stamp", Shape::Struct("Stamp"));
    let through = wrong.repr(Shape::ScalarThrough);
    wrong.at("stamp_each", vec![param("stamp"), Step::Arg(0)], through);
    wrong.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let error = wrong
        .generate(model())
        .expect_err("a struct takes no arguments");
    assert!(error.to_string().contains("takes no arguments"), "{error}");
}

/// A callback's representation prints with its capture, invocation and
/// routes.
#[test]
fn a_callback_representation_prints_what_a_call_does() {
    let mut fixture = fixture();
    fixture.declare_callback("i64", Call::Routed);
    let printed = fixture.build(&model()).to_string();
    let expected = "callable  w1  capture: Target(Capture)  invoke: Target(InvokeFallibly) \
                    fails runtime Error  routes: [runtime: report Error by Target(Report), if \
                    that fails abort, then return ()]";
    assert!(
        printed.contains(expected),
        "missing `{expected}` in:\n{printed}"
    );
    assert!(
        printed.contains("output   callback:impl Fn(i64)+Send+Sync+'static  type in1 -  ()"),
        "{printed}"
    );
}

/// Nothing inside a call supplies a runtime context — the wrapper that had one
/// may have returned — so a call that needs one refuses the callback.
#[test]
fn a_call_needing_a_runtime_context_refuses_the_callback() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_callback("i64", Call::NeedsContext);
    fixture.declare_fn("stamp_each", exported("stamp_each", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:stamp_each") {
        Outcome::Skipped(skip) => assert_eq!(
            skip.capability.as_str(),
            "unsupported.callback.missing_context"
        ),
        other => panic!("stamp_each is skipped, and is {other:?}"),
    }
}

/// A callable never leaves Rust: no representation carries one out, so a
/// function returning one is refused where its result is.
#[test]
fn a_callback_leaving_rust_is_refused() {
    let mut fixture = fixture();
    fixture.declare_callback("i64", Call::Infallible);
    fixture.declare_fn("each_new", exported("each_new", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:each_new") {
        Outcome::Skipped(skip) => {
            assert_eq!(skip.capability.as_str(), "unsupported.conversion.no_rule");
            assert_eq!(
                skip.dependency_path.last().map(String::as_str),
                Some("return")
            );
        }
        other => panic!("each_new is skipped, and is {other:?}"),
    }
}

/// A callback representation on a value that is not an `impl Fn(..)` has no
/// arguments to plan, and is refused as not a callback.
#[test]
fn a_callback_representation_on_another_type_is_refused() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    let callback = fixture.callback_repr(Call::Infallible);
    fixture.at("stamp_sum", vec![param("stamp")], callback);
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:stamp_sum") {
        Outcome::Skipped(skip) => {
            assert_eq!(skip.capability.as_str(), "unsupported.type.not_a_callback")
        }
        other => panic!("stamp_sum is skipped, and is {other:?}"),
    }
}

/// An argument of a kind the callback's wire type cannot have as a part
/// refuses the callback at that argument.
#[test]
fn an_argument_the_callable_cannot_have_refuses_the_callback() {
    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    let scalars_only = fixture.binding.wire_type(Wire::Closure {
        kind: Kind::ScalarClosure,
        name: quote::format_ident!("ScalarClosure"),
    });
    let repr = Repr {
        into_rust: fixture
            .binding
            .in_representation(InRepresentation::Callable {
                wire_type: scalars_only,
                capture: Operation::Target(Op::Capture),
                invoke: Operation::Target(Op::Invoke),
                routes: Vec::new(),
            }),
        out_of_rust: None,
    };
    let key = callback_key("Token, i64");
    fixture.crossing_key(key.clone(), repr);
    fixture.declare(
        Declaration::Callback(key),
        OutputForm::Type {
            into_rust: repr.into_rust,
            out_of_rust: None,
            release: None,
            meta: (),
        },
    );
    fixture.declare_fn("token_watch", exported("token_watch", Routes::None));
    let generation = fixture.generate(model()).expect("plans");
    match outcome(&generation, "fn:token_watch") {
        Outcome::Skipped(skip) => {
            assert_eq!(skip.capability.as_str(), "unsupported.mini.arg.pointer");
            assert_eq!(
                skip.dependency_path.last().map(String::as_str),
                Some("arg 0")
            );
        }
        other => panic!("token_watch is skipped, and is {other:?}"),
    }
}

/// A kind's name is part of a capability code, so a target naming two kinds
/// alike is refused before anything is planned.
#[test]
fn two_kinds_of_one_name_are_refused() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Twins {
        Left,
        Right,
    }
    impl WireKind for Twins {
        const ALL: &'static [Self] = &[Twins::Left, Twins::Right];
        fn name(self) -> &'static str {
            "twin"
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct Twin;
    impl WireType for Twin {
        type Kind = Twins;
        fn kind(&self) -> Twins {
            Twins::Left
        }
        fn rust(&self) -> syn::Type {
            syn::parse_quote!(i64)
        }
    }
    struct Twinned;
    impl Target for Twinned {
        const NAME: &'static str = "twinned";
        type WireType = Twin;
        type Op = ();
        type OutputMeta = ();
        fn write_operation(&self, _: &(), _: &OperationFeed<'_, Self>) -> Written {
            unreachable!("nothing is planned")
        }
        fn write_wire_type(&self, _: &WireTypeFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
            unreachable!("nothing is planned")
        }
    }
    let error = generate(model(), &Twinned, Binding::new(), syn::parse_quote!(source))
        .expect_err("two kinds named `twin`");
    assert!(
        error.to_string().contains("two wire kinds `twin`"),
        "{error}"
    );
}

/// A representation converts values of one type, so a binding ruling one for
/// two types — a type rule and a rule at a position whose value is of another
/// type — fails the build before anything is planned.
#[test]
fn a_representation_ruled_for_two_types_is_an_error() {
    let mut fixture = fixture();
    let strukt = fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.at("stamp_max", vec![param("stamp"), field("secs")], strukt);
    fixture.declare_fn("stamp_max", exported("stamp_max", Routes::None));
    let error = fixture
        .generate(model())
        .expect_err("one representation, two types");
    assert!(
        error
            .to_string()
            .contains("converts `Stamp` for every `Stamp` and `i64` for `param stamp.field secs`"),
        "{error}"
    );
}

/// A handle's release frees what it handed out, so it belongs to the
/// out-of-Rust representation.
#[test]
fn a_release_is_part_of_what_is_handed_out() {
    let mut fixture = fixture();
    fixture.declare_type("Token", Shape::Handle);
    let printed = fixture.build(&model()).to_string();
    assert!(
        printed.contains("repr     out1  whole  w1 Standard(IntoRaw)  release: Standard(Release)"),
        "{printed}"
    );
}

/// A position crosses in one direction — a parameter into Rust, a result out
/// of it — so a rule there naming a representation of the other direction
/// fails the build rather than sitting unused.
#[test]
fn a_rule_of_the_other_direction_at_a_position_is_an_error() {
    let mut fixture = fixture();
    fixture.declare_type("Stamp", Shape::Struct("Stamp"));
    fixture.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let scalar = fixture.scalar;
    let mut binding = fixture.build(&model());
    let function = binding.output_id(1);
    binding.rule(
        Scope::At(function, ValuePath(vec![Step::Return])),
        scalar.into_rust,
    );
    let error = generate(model(), &Mini, binding, syn::parse_quote!(source))
        .expect_err("a result does not cross into Rust");
    assert!(
        error.to_string().contains(
            "the value there crosses out of Rust, and the rule's in0 serves values crossing \
             into Rust"
        ),
        "{error}"
    );
}
