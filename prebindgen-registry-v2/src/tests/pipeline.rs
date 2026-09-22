//! What the engine decides, over a target that answers in one line.
//!
//! The adapter here is deliberately not a language: it exists to make the
//! engine's own contracts observable — which conversions are shared, what a
//! missing capability takes down with it, what happens when a boundary leaves a
//! declared failure unrouted. The two real adapters, and the generated code
//! rustc compiles, live in `examples/v2check`.

use std::collections::BTreeMap;

use prebindgen_flat::flat::{Flat, ScalarKind, TypeKind, TypeRef};

use crate::{
    decl::Declaration,
    outcome::{EngineError, Outcome},
    plan::generate,
    run::Generation,
    target::{
        AbiSpec, Access, BoundarySpec, ChildValue, FailureCategory, Layout, OperandSpec, Operation,
        OperationType, OutputPlacement, ParamRole, PlanningError, Position, PrimitiveFailure,
        PrimitiveSpec, Protocol, Relation, ReprSpec, ResolvedShape, ResolvedValues, Selection,
        SelectionQuery, SiteDescriptor, SourceItem, StandardOp, SurfaceRequest, SurfaceSpec,
        Target, TargetAttempt, TargetSupport, Terminal, Unsupported, WireType, WrapperParam,
    },
};

/// Two structs and three functions, one of which has a field nothing can carry;
/// an opaque type, and the two functions that hand one out and take it back.
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
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    items
}

/// What this target was told about one value or one function.
///
/// Also its [`Target::ConversionKey`]: these are plain data, so two values the
/// binding configured the same way are converted the same way, which is
/// exactly what the key has to mean. An adapter whose settings held something
/// incomparable would intern them and key on the index instead.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Choice {
    #[default]
    Scalar,
    /// A scalar carried as itself, through an operation of this target's own
    /// rather than the registry's identity: the same `i64` on the wire, a
    /// different conversion. What makes "two supported children" observable
    /// without one of them failing.
    ScalarThrough,
    /// A struct read through its members.
    Struct,
    /// The same, with member reads that can fail — which is what makes a
    /// boundary's failure routes observable.
    FallibleStruct,
    /// A struct that converts, and whose public declaration this target
    /// refuses — which is what drives the retention loop rather than value
    /// planning.
    StructWithoutSurface,
    /// A struct whose public declaration requires another declaration's, so a
    /// refusal has to travel two edges.
    StructRequiring(String),
    /// An opaque type carried whole as an address, released through
    /// `<name>_free`.
    Handle,
    /// The same, for a target with no way to place a release — which is what
    /// makes the type, and everything taking it, unsupported.
    HandleWithoutRelease,
    Function {
        symbol: String,
        routes: Routes,
        /// Names for the wrapper parameters, in order. Empty means `arg0`,
        /// `arg1`, … — the well-behaved case that hides a name collision.
        param_names: Vec<String>,
        /// Attributes the wrapper carries and whether it is `unsafe`: the part
        /// of its form a target may state.
        attrs: Vec<syn::Attribute>,
        unsafety: bool,
    },
    /// A boundary that passes its first parameter as a carrier its conversion
    /// does not read — an adapter defect the registry has to catch rather than
    /// emit.
    FunctionWithWrongInput,
    /// A struct whose public declaration is Rust the target contributes: a
    /// mirror of the source struct, one member per field. Only this choice
    /// produces an artifact, so every other test's generated file is unchanged.
    StructWithMirror,
}

/// What a boundary does about failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Routes {
    /// None declared, which skips any function whose conversions can fail.
    None,
    /// One route per category a conversion here can raise, each reporting
    /// through an operation that needs no context.
    Reported,
    /// One route whose reporting operation needs a runtime context the
    /// boundary does not supply.
    ReporterNeedsContext,
}

fn exported(symbol: &str, routes: Routes) -> Choice {
    Choice::Function {
        symbol: symbol.to_string(),
        routes,
        param_names: Vec::new(),
        attrs: Vec::new(),
        unsafety: false,
    }
}

#[derive(Clone, Debug)]
enum Payload {
    ReadFallibly,
    Report,
    /// The binding-failure reporter, which is handed a `String`.
    ReportMessage,
    /// The operation a [`Choice::ScalarThrough`] value crosses by.
    Rebase,
}

/// The miniature target, and the binding storage it answers from.
///
/// Everything the engine used to hold for an adapter — a default, a choice per
/// source type, an override per site, and what each requested output was
/// declared as — lives here, because #766 is exactly that move. The two real
/// adapters keep the same four kinds of entry in their own builders; this is
/// the smallest thing that has all four.
#[derive(Default)]
struct Mini {
    /// How every value of this source type crosses, wherever it appears.
    types: BTreeMap<String, Choice>,
    /// How one value inside one declaration crosses instead: keyed by the
    /// declaration and the site's path, `param 0` or `param 0.field secs`.
    sites: BTreeMap<(Declaration, String), Choice>,
}

impl Mini {
    /// The settings for a value at this position — the site override, then the
    /// type's own, then the default — as a conversion key.
    ///
    /// Same precedence the engine used to apply, now a target's own business.
    /// It is a pure function of the position and the type, so the same value
    /// asked twice answers the same: an adapter that let this drift would hand
    /// the registry two keys for one conversion and share nothing.
    fn conversion(&self, position: &Position, declared: &Choice, ty: &TypeRef) -> Choice {
        // A declared type's own crossing is planned as that declaration says.
        if position.is_declared_type() {
            return declared.clone();
        }
        let site = (position.declaration.clone(), position.path.join("."));
        if let Some(choice) = self.sites.get(&site) {
            return choice.clone();
        }
        self.types
            .get(ty.key().as_str())
            .cloned()
            .unwrap_or(Choice::Scalar)
    }
}

impl Target for Mini {
    const NAME: &'static str = "mini";

    type ConversionKey = Choice;
    type Payload = Payload;

    fn select(&self, query: &SelectionQuery<'_, Choice>) -> TargetSupport<Selection<Choice>> {
        let conversion = self.conversion(query.position, query.declared, &query.crossing.ty);
        let want_struct = matches!(
            conversion,
            Choice::Struct
                | Choice::FallibleStruct
                | Choice::StructWithoutSurface
                | Choice::StructRequiring(_)
                | Choice::StructWithMirror
        );
        for (id, relation) in query.candidates {
            match (relation, want_struct) {
                (Relation::Struct(_), true) | (Relation::Atomic, false) => {
                    return Ok(TargetAttempt::Ready(Selection {
                        relation: *id,
                        conversion,
                    }))
                }
                _ => {}
            }
        }
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.mini.no_relation",
            "no relation for what this value was declared as",
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        conversion: &Choice,
    ) -> TargetSupport<ReprSpec<Payload>> {
        match shape.relation {
            Relation::Atomic
                if matches!(conversion, Choice::Handle | Choice::HandleWithoutRelease) =>
            {
                let carrier = WireType::abi(syn::parse_quote!(*mut Raw));
                let ty = shape.crossing.ty.clone();
                Ok(TargetAttempt::Ready(match shape.crossing.direction {
                    crate::target::Direction::IntoRust => ReprSpec {
                        layout: Layout::Scalar(carrier.clone()),
                        protocol: Protocol::terminal(PrimitiveSpec::from_raw(
                            carrier.clone(),
                            ty.clone(),
                        )),
                        release: Some(PrimitiveSpec::release(carrier, ty)),
                    },
                    crate::target::Direction::OutOfRust => ReprSpec {
                        layout: Layout::Scalar(carrier.clone()),
                        protocol: Protocol::terminal(PrimitiveSpec::into_raw(ty, carrier)),
                        release: None,
                    },
                }))
            }
            Relation::Atomic => {
                if !matches!(shape.crossing.ty.kind(), TypeKind::Scalar(ScalarKind::I64)) {
                    // The one capability this target is missing, and the reason
                    // `Label` cannot cross.
                    return Ok(TargetAttempt::Unsupported(Unsupported::new(
                        "unsupported.mini.carrier",
                        format!("`{}` has no carrier here", shape.crossing.ty.key()),
                    )));
                }
                let carrier = WireType::abi(syn::parse_quote!(i64));
                let codec = match conversion {
                    Choice::ScalarThrough => PrimitiveSpec {
                        operands: vec![OperandSpec::value(
                            OperationType::Carrier(carrier.clone()),
                            Access::Owned,
                        )],
                        result: Some(OperationType::Carrier(carrier.clone())),
                        failure: PrimitiveFailure::Infallible,
                        dependencies: Vec::new(),
                        implementation: Operation::Target(Payload::Rebase),
                    },
                    _ => PrimitiveSpec::identity(OperationType::Carrier(carrier.clone())),
                };
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier),
                    protocol: Protocol::terminal(codec),
                    release: None,
                }))
            }
            Relation::Struct(strukt) => {
                let ident = quote::format_ident!("{}", strukt.name);
                let aggregate = WireType::abi(syn::parse_quote!(#ident));
                let item = shape.strukt.expect("a struct relation carries its strukt");
                let fallible = matches!(conversion, Choice::FallibleStruct);
                let projections = item
                    .fields
                    .iter()
                    .zip(children)
                    .map(|(field, child)| PrimitiveSpec {
                        operands: vec![OperandSpec::value(
                            OperationType::Carrier(aggregate.clone()),
                            Access::Shared,
                        )],
                        result: Some(OperationType::Carrier(child.layout.wire().clone())),
                        failure: if fallible {
                            PrimitiveFailure::fallible(
                                OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                    Error
                                ))),
                                FailureCategory::Runtime,
                            )
                        } else {
                            PrimitiveFailure::Infallible
                        },
                        dependencies: Vec::new(),
                        implementation: if fallible {
                            Operation::Target(Payload::ReadFallibly)
                        } else {
                            Operation::Standard(StandardOp::ReadMember {
                                member: field.member(),
                            })
                        },
                    })
                    .collect();
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Aggregate {
                        ty: aggregate,
                        members: item.fields.iter().map(|field| field.member()).collect(),
                    },
                    protocol: Protocol::Product { projections },
                    release: None,
                }))
            }
        }
    }

    fn boundary(
        &self,
        site: &SiteDescriptor<'_, Choice>,
        values: &ResolvedValues<'_, Payload>,
    ) -> TargetSupport<BoundarySpec<Payload>> {
        // What this wrapper exports is what the binding declared for it, which
        // the site names.
        let choice = site.declared;
        if matches!(choice, Choice::FunctionWithWrongInput) {
            return Ok(TargetAttempt::Ready(BoundarySpec {
                abi: AbiSpec {
                    abi: "C".to_string(),
                    symbol: "wrong".to_string(),
                    params: vec![WrapperParam {
                        name: quote::format_ident!("arg0"),
                        // The conversion reads a `Stamp` aggregate.
                        ty: WireType::abi(syn::parse_quote!(i64)),
                        role: ParamRole::Input(0),
                        mutable: false,
                    }],
                    ret: values.output.map(|value| value.repr.layout.wire().clone()),
                    attrs: Vec::new(),
                    unsafety: false,
                },
                output: OutputPlacement::Return,
                failures: Vec::new(),
            }));
        }
        // A release site carries the handle type's own choice: its symbol is
        // derived from the item's name — the key may carry arguments a symbol
        // cannot — and a null address is not a failure it can raise.
        let release = match (site.function, choice) {
            (None, Choice::Handle) => Some(exported(
                &format!(
                    "{}_free",
                    site.declaration
                        .entity_name()
                        .expect("a handle is an entity")
                ),
                Routes::None,
            )),
            (None, Choice::HandleWithoutRelease) => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    "unsupported.mini.no_release",
                    "this target has nowhere to place a release",
                )))
            }
            _ => None,
        };
        let Choice::Function {
            symbol,
            routes,
            param_names,
            attrs,
            unsafety,
        } = release.as_ref().unwrap_or(choice)
        else {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` is declared as a value, not as a function to export",
                site.declaration.name()
            )));
        };
        Ok(TargetAttempt::Ready(BoundarySpec {
            abi: AbiSpec {
                abi: "C".to_string(),
                symbol: symbol.clone(),
                params: values
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| WrapperParam {
                        name: match param_names.get(index) {
                            Some(name) => quote::format_ident!("{name}"),
                            None => quote::format_ident!("arg{index}"),
                        },
                        ty: value.repr.layout.wire().clone(),
                        role: ParamRole::Input(index),
                        mutable: false,
                    })
                    .collect(),
                ret: values.output.map(|value| value.repr.layout.wire().clone()),
                attrs: attrs.clone(),
                unsafety: *unsafety,
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            failures: if *routes == Routes::None {
                Vec::new()
            } else {
                vec![
                    crate::target::FailureRoute {
                        category: FailureCategory::Binding,
                        report: Some(PrimitiveSpec {
                            operands: vec![OperandSpec::error(OperationType::Carrier(
                                WireType::internal(syn::parse_quote!(String)),
                            ))],
                            result: None,
                            failure: PrimitiveFailure::Infallible,
                            dependencies: Vec::new(),
                            implementation: Operation::Target(Payload::ReportMessage),
                        }),
                        on_report_failure: Terminal::Abort,
                        terminate: Terminal::Return(syn::parse_quote!(0)),
                    },
                    crate::target::FailureRoute {
                        category: FailureCategory::Runtime,
                        report: Some(PrimitiveSpec {
                            operands: {
                                let mut operands =
                                    vec![OperandSpec::error(OperationType::Carrier(
                                        WireType::internal(syn::parse_quote!(Error)),
                                    ))];
                                if *routes == Routes::ReporterNeedsContext {
                                    operands.push(OperandSpec::context(
                                        "mini.log",
                                        OperationType::Carrier(WireType::internal(
                                            syn::parse_quote!(Log),
                                        )),
                                        Access::Exclusive,
                                    ));
                                }
                                operands
                            },
                            result: None,
                            failure: PrimitiveFailure::Infallible,
                            dependencies: Vec::new(),
                            implementation: Operation::Target(Payload::Report),
                        }),
                        on_report_failure: Terminal::Abort,
                        terminate: Terminal::Return(syn::parse_quote!(0)),
                    },
                ]
            },
        }))
    }

    fn surface(
        &self,
        request: &SurfaceRequest<'_, Choice>,
        values: &ResolvedValues<'_, Payload>,
    ) -> TargetSupport<SurfaceSpec<Payload>> {
        let choice = request.declared;
        let requires = values
            .inputs
            .iter()
            .filter_map(|value| crate::target::Requirement::of(value))
            .collect();
        if matches!(
            (choice, request.item),
            (Choice::StructWithoutSurface, SourceItem::Struct(_))
        ) {
            return Ok(TargetAttempt::Unsupported(Unsupported::new(
                "unsupported.mini.no_public_struct",
                "this struct converts, and has no public declaration here",
            )));
        }
        // The mirror is what a real C target contributes: a struct of its own,
        // one member per source field, each under that field's condition.
        let rust = match (choice, request.item) {
            (Choice::StructWithMirror, SourceItem::Struct(strukt)) => {
                let ident = &strukt.name;
                let conditions = request.field_conditions();
                let members = strukt.fields.iter().zip(&conditions).map(|(field, under)| {
                    let name = field.name.as_ref().expect("the fixture names its fields");
                    quote::quote!(#(#under)* pub #name: i64)
                });
                vec![crate::target::Artifact::new(
                    strukt.name.to_string(),
                    quote::quote!(#[repr(C)] pub struct #ident { #(#members),* }),
                )]
            }
            _ => Vec::new(),
        };
        Ok(TargetAttempt::Ready(SurfaceSpec {
            declaration: request.declaration.clone(),
            requires: match (choice, request.item) {
                (_, SourceItem::Function(_)) => requires,
                (Choice::StructRequiring(other), SourceItem::Struct(_)) => {
                    vec![crate::target::Requirement::type_named(other)]
                }
                (_, SourceItem::Struct(_) | SourceItem::Extern(_)) => Vec::new(),
            },
            rust,
            payload: None,
        }))
    }

    fn render_operation(
        &self,
        payload: &Payload,
        operands: &[syn::Ident],
    ) -> proc_macro2::TokenStream {
        match payload {
            Payload::ReadFallibly => {
                let value = &operands[0];
                quote::quote!(read(#value))
            }
            Payload::Report => {
                let error = &operands[0];
                quote::quote!(report(#error))
            }
            Payload::ReportMessage => {
                let error = &operands[0];
                quote::quote!(report_message(#error))
            }
            Payload::Rebase => {
                let value = &operands[0];
                quote::quote!(rebase(#value))
            }
        }
    }
}

/// A miniature frontend: what the binding declared, and the work list it hands
/// the engine.
///
/// Every test states its binding here. A declared value crossing goes into
/// [`Mini`], which answers about values of a type wherever they turn up; what
/// each output *is* travels with the output itself.
#[derive(Default)]
struct Binding {
    target: Mini,
    outputs: Vec<(Declaration, Choice)>,
}

impl Binding {
    /// How every value of this source type crosses — without asking for the
    /// type itself to be declared.
    fn crossing(&mut self, name: &str, choice: Choice) -> &mut Self {
        let key = prebindgen_flat::TypeKey::parse(name).expect("a test names a type");
        self.target.types.insert(key.as_str().to_string(), choice);
        self
    }

    /// A declared type: how its values cross, and what its public declaration
    /// is. The two are one declarator in both real frontends.
    fn declare_type(&mut self, name: &str, choice: Choice) -> &mut Self {
        self.crossing(name, choice.clone());
        self.declare(ty(name), choice)
    }

    /// An exported source function.
    fn declare_fn(&mut self, name: &str, choice: Choice) -> &mut Self {
        self.declare(function(name), choice)
    }

    /// One requested output, as the binding declared it.
    fn declare(&mut self, declaration: Declaration, choice: Choice) -> &mut Self {
        self.outputs.push((declaration, choice));
        self
    }

    /// How one value inside one declaration crosses, overriding its type's
    /// own: `("stamp_max", "param 0")`, `("stamp_max", "param 0.field secs")`.
    fn at_site(&mut self, function_name: &str, path: &str, choice: Choice) -> &mut Self {
        self.target
            .sites
            .insert((function(function_name), path.to_string()), choice);
        self
    }

    /// Plan this binding over `flat`.
    fn generate(self, flat: Flat) -> Result<Generation<Payload>, EngineError> {
        let Binding { target, outputs } = self;
        generate(flat, &target, outputs, syn::parse_quote!(source))
    }
}

fn binding() -> Binding {
    Binding::default()
}

fn ty(name: &str) -> Declaration {
    Declaration::Type(prebindgen_flat::TypeKey::parse(name).expect("a test names a type"))
}

fn function(name: &str) -> Declaration {
    Declaration::Function(syn::parse_str(name).expect("a test names an ident"))
}

/// What became of the declaration that prints as `id`.
///
/// Skipped when the run left it out, emitted when it produced a public
/// declaration, and a panic when the binding never asked for it — so a typo
/// in a test cannot read as success. An entity declared twice has two outputs
/// under one printed name; a test that declares one twice counts the skips
/// instead of naming them.
fn outcome<P>(generation: &crate::run::Generation<P>, id: &str) -> Outcome {
    if let Some((_, skip)) = generation
        .skipped()
        .iter()
        .find(|(declaration, _)| declaration.to_string() == id)
    {
        return Outcome::Skipped(skip.clone());
    }
    assert!(
        generation
            .surfaces()
            .iter()
            .any(|surface| surface.declaration.to_string() == id),
        "nothing was declared as {id}"
    );
    Outcome::Emitted
}

/// How many declarations the run generated: one public declaration each.
fn emitted<P>(generation: &crate::run::Generation<P>) -> usize {
    generation.surfaces().len()
}

/// Two exported functions taking the same struct the same way share its
/// conversion — and its two field conversions, and the result conversion.
#[test]
fn one_conversion_serves_every_value_that_crosses_the_same_way() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Stamp` into Rust, `i64` into Rust, `i64` out of Rust. Twice over, and
    // once for the struct's own request, is still three.
    assert_eq!(generation.values().len(), 3);
    assert_eq!(generation.functions().len(), 2);
}

/// A per-site override makes the target answer with a different conversion
/// key, so it is a different conversion — which is the whole reason the key is
/// part of a node's identity.
#[test]
fn a_site_override_does_not_share_the_default_conversion() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));
    binding.at_site("stamp_max", "param 0", Choice::FallibleStruct);

    let generation = binding.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // The two `Stamp` conversions are distinct; their `i64` children still are
    // not, because nothing overrode them.
    assert_eq!(generation.values().len(), 4);
    let fallible = generation
        .functions()
        .iter()
        .find(|plan| plan.abi.symbol == "stamp_max")
        .expect("stamp_max is emitted");
    // Both routes the boundary declared travel with the plan, whether or not a
    // conversion here raises their category.
    assert_eq!(fallible.failures.len(), 2);
    assert!(generation.rust().contains("read(arg0)"));
    assert!(generation.rust().contains("report(error)"));
}

/// One unsupported field takes down its struct and everything requiring it, and
/// leaves everything else alone.
#[test]
fn an_unsupported_field_skips_its_struct_and_its_callers() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_type("Label", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    assert!(matches!(
        outcome(&generation, "type:Stamp"),
        Outcome::Emitted
    ));
    assert!(matches!(
        outcome(&generation, "fn:stamp_sum"),
        Outcome::Emitted
    ));
    let Outcome::Skipped(strukt) = outcome(&generation, "type:Label") else {
        panic!("`Label` has a field nothing can carry");
    };
    assert_eq!(strukt.capability.as_str(), "unsupported.mini.carrier");
    let Outcome::Skipped(caller) = outcome(&generation, "fn:label_len") else {
        panic!("a function taking `Label` cannot be generated either");
    };
    // One cause, two casualties, each with its own path to it.
    assert_eq!(caller.capability.as_str(), "unsupported.mini.carrier");
    assert_eq!(caller.dependency_path.first().unwrap(), "fn:label_len");
    // Nothing partial is emitted: no wrapper for the skipped function.
    assert_eq!(generation.functions().len(), 1);
    assert!(!generation.rust().contains("label_len"));
}

/// An item the binding defines itself is an entity like a captured one, and
/// the writer reaches it where the binding said it is.
///
/// The same `Mini` target, the same declarations: what differs is that the
/// model was told about `stamp_zero` by the binding, with a signature and a
/// path, rather than by a capture. The wrapper it gets is called through that
/// path, and a type the binding declared over one the source never exported
/// is a handle like any captured alias.
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

    let mut binding = binding();
    binding.declare_type("Token", Choice::Handle);
    binding.declare_fn("token_use", exported("token_use", Routes::Reported));
    binding.declare_fn("stamp_zero", exported("stamp_zero", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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

    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_twice", exported("stamp_twice", Routes::None));
    let generation = binding.generate(flat).expect("plans");
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
    let mut binding = binding();
    // The item is `Token`; the choice is recorded under the key as declared,
    // and the type's own planning has to find it there.
    binding.declare_type("Token<'static>", Choice::Handle);
    binding.declare(
        Declaration::Conversion(prebindgen_flat::TypeKey::parse("Option<Stamp>").unwrap()),
        Choice::Scalar,
    );
    let generation = binding.generate(model()).expect("both are valid requests");
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
/// choice and exports its own wrapper. Sharing is by conversion, as between
/// two different functions — the `Stamp` both take crosses the same way and
/// is planned once.
#[test]
fn one_function_declared_twice_is_two_outputs() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare(function("stamp_sum"), exported("stamp_sum_a", Routes::None));
    binding.declare(
        function("stamp_sum"),
        exported("stamp_sum_b", Routes::Reported),
    );

    let generation = binding.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3, "{:?}", generation.skipped());
    let mut declared: Vec<String> = generation
        .surfaces()
        .iter()
        .map(|surface| surface.declaration.to_string())
        .collect();
    declared.sort();
    assert_eq!(declared, ["fn:stamp_sum", "fn:stamp_sum", "type:Stamp"]);
    let symbols: Vec<&str> = generation
        .functions()
        .iter()
        .map(|plan| plan.abi.symbol.as_str())
        .collect();
    assert_eq!(symbols, ["stamp_sum_a", "stamp_sum_b"]);
    // `Stamp` into Rust once, `i64` each way once: the two wrappers share
    // every conversion, because nothing about the value differs.
    assert_eq!(generation.values().len(), 3);

    // One entity declared twice as the same thing is the binding saying one
    // thing twice: two plans for one foreign declaration.
    let mut twice = Binding::default();
    twice.declare_type("Stamp", Choice::Struct);
    twice.declare(function("stamp_sum"), exported("stamp_sum_a", Routes::None));
    twice.declare(function("stamp_sum"), exported("stamp_sum_a", Routes::None));
    let error = twice
        .generate(model())
        .expect_err("one choice, one declaration");
    assert!(matches!(error, EngineError::DuplicateDeclaration { .. }));
}

/// One type declared twice, and a value resolves its requirement to the
/// declaration it crosses as.
///
/// `Stamp` is declared as a struct and as a handle. Values of it cross as a
/// struct by default; `stamp_max`'s parameter is overridden to cross as a
/// handle. Each function requires the declaration its value crosses as — so
/// when the handle cannot be placed, `stamp_max` goes with it and `stamp_sum`
/// does not.
#[test]
fn a_value_requires_the_declaration_it_crosses_as() {
    let plan = |handle: Choice| {
        let mut binding = binding();
        binding.declare(ty("Stamp"), Choice::Struct);
        binding.declare(ty("Stamp"), handle.clone());
        binding.crossing("Stamp", Choice::Struct);
        binding.at_site("stamp_max", "param 0", handle);
        binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
        binding.declare_fn("stamp_max", exported("stamp_max", Routes::Reported));
        binding.generate(model()).expect("plans")
    };

    // Both declarations placed: everything is emitted, and each function's
    // requirement went to the one it crosses as.
    let generation = plan(Choice::Handle);
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

    // The handle declaration refused: only the function crossing that way
    // requires it, and only that function is skipped.
    let generation = plan(Choice::HandleWithoutRelease);
    // One of the two `Stamp` declarations is skipped — the handle, which has
    // nowhere to place a release — and the other is emitted. They print the
    // same, so what says which is that exactly one survived.
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
            ("type:Stamp".to_string(), "unsupported.mini.no_release"),
            ("fn:stamp_max".to_string(), "unsupported.mini.no_release"),
        ]
    );
    assert_eq!(
        generation
            .surfaces()
            .iter()
            .filter(|surface| surface.declaration.to_string() == "type:Stamp")
            .count(),
        1,
        "the struct declaration of `Stamp` is emitted"
    );
    assert!(matches!(
        outcome(&generation, "fn:stamp_sum"),
        Outcome::Emitted
    ));
    // `stamp_max` went down with the declaration its parameter crosses as, and
    // says so.
    let (_, skip) = &generation.skipped()[1];
    assert_eq!(skip.dependency_path, ["fn:stamp_max", "type:Stamp"]);
}

/// A requirement stated by name alone cannot choose between two declarations
/// of one type.
#[test]
fn a_requirement_by_name_is_ambiguous_over_two_declarations() {
    let mut binding = binding();
    binding.declare(ty("Stamp"), Choice::Struct);
    binding.declare(ty("Stamp"), Choice::Handle);
    binding.crossing("Stamp", Choice::Struct);
    // `Point` requires `Stamp` by name, not through a value of it.
    binding.declare_type("Point", Choice::StructRequiring("Stamp".to_string()));

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "type:Point") else {
        panic!("a name does not say which declaration");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.requirement.ambiguous"
    );
}

/// A type whose conversion needs its own is refused, rather than recursed on
/// until the stack runs out.
///
/// The mark that catches it is keyed on the conversion the target named, so
/// this is what holds a target to the second half of [`Selection`]'s
/// contract: an adapter minting a fresh key per visit would make the inner
/// `Node` look like a different conversion, and the walk would not come back.
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

    let mut binding = binding();
    binding.declare_type("Node", Choice::Struct);
    binding.declare_fn("node_sum", exported("node_sum", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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

/// A public declaration that requires a type the binding never declared is
/// skipped, not emitted against a type that will not exist.
#[test]
fn a_function_needing_an_undeclared_public_type_is_skipped() {
    let mut binding = binding();
    // Its values cross, and the type itself was never asked for.
    binding.crossing("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("the aggregate it takes is not a declared public type");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.requirement.unrequested"
    );
    assert!(generation.rust().is_empty());
}

/// A conversion that can fail needs a route. A boundary that declares none does
/// not get a default — the function is skipped and the report says why.
#[test]
fn a_declared_failure_with_no_route_skips_the_function() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::FallibleStruct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("its member reads can fail and the boundary routes nothing");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.unrouted_failure"
    );
    // The struct itself is unaffected: its conversion is fine, and it is the
    // boundary that could not be assembled.
    assert!(matches!(
        outcome(&generation, "type:Stamp"),
        Outcome::Emitted
    ));
}

/// Contradictory configuration fails the build; it is never turned into a
/// capability the engine claims to be missing.
#[test]
fn a_value_declaration_on_an_exported_function_is_an_error() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    // A struct declarator where a function declarator belongs.
    binding.declare(function("stamp_sum"), Choice::Struct);

    let error = binding.generate(model()).expect_err("refuses");
    assert!(matches!(
        error,
        EngineError::Planning(PlanningError::InvalidInput(_))
    ));
}

/// A declaration the source never captured is an error, not a skip — the
/// same rule the engine already held for its report-only run.
#[test]
fn a_declaration_naming_nothing_is_an_error() {
    let mut binding = binding();
    binding.declare_fn("nope", exported("nope", Routes::None));

    let error = binding.generate(model()).expect_err("refuses");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
}

/// Two runs over unchanged inputs produce the same report and the same code, so
/// a diff of either means something.
#[test]
fn a_run_over_unchanged_input_produces_the_same_output() {
    let run = || {
        let mut binding = binding();
        binding.declare_type("Stamp", Choice::Struct);
        binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
        let generation = binding.generate(model()).expect("plans");
        (
            format!("{:?}", generation.skipped()),
            generation.rust().to_string(),
        )
    };
    assert_eq!(run(), run());
}

/// A conversion recorded for a *field* makes its struct a different
/// conversion, whichever order the two uses are planned in.
///
/// The cache is consulted after the children are planned for exactly this
/// reason: keyed on the struct's own conversion alone, the second use would
/// inherit the first one's conversion and its support outcome, in whichever
/// direction the two happened to be requested.
#[test]
fn a_field_override_is_part_of_its_struct_conversion() {
    let plan = |defaults_first: bool| {
        let mut binding = binding();
        binding.crossing("Stamp", Choice::Struct);
        // Read through fields, on a scalar field: the target offers no struct
        // relation for an `i64`, so this child cannot be selected at all.
        binding.at_site("stamp_max", "param 0.field secs", Choice::Struct);
        // Order matters twice over: which function is planned first, and
        // whether the struct's own request primed the conversion before either
        // of them. The refusal-first case must not be primed, or it would not
        // test what happens when a refusal is met before any success.
        if defaults_first {
            binding.declare_type("Stamp", Choice::Struct);
            binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
            binding.declare_fn("stamp_max", exported("stamp_max", Routes::None));
        } else {
            binding.declare_fn("stamp_max", exported("stamp_max", Routes::None));
            binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
            binding.declare_type("Stamp", Choice::Struct);
        }
        binding.generate(model()).expect("plans")
    };
    for defaults_first in [true, false] {
        let generation = plan(defaults_first);
        let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_max") else {
            panic!("its `secs` field is declared a way nothing can serve");
        };
        assert_eq!(skip.capability.as_str(), "unsupported.mini.no_relation");
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
/// The reference adapters name every parameter `argN`, which hides this: a
/// boundary is free to name one `v0`, and a temporary taking that name would
/// compile and feed the wrong value to the source call.
#[test]
fn a_temporary_never_takes_a_live_parameter_name() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn(
        "stamp_pick",
        Choice::Function {
            symbol: "stamp_pick".to_string(),
            routes: Routes::None,
            // The names a writer would otherwise allocate for itself.
            param_names: vec!["v0".to_string(), "v1".to_string()],
            attrs: Vec::new(),
            unsafety: false,
        },
    );

    let generation = binding.generate(model()).expect("plans");
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

/// A failure route whose reporting operation needs a context the boundary does
/// not supply skips the function.
///
/// The conversions here need no context at all, so only the reporter's operands
/// can reveal it — which is what makes this different from the conversion-side
/// check.
#[test]
fn a_reporter_needing_an_unsupplied_context_skips_the_function() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::FallibleStruct);
    binding.declare_fn(
        "stamp_sum",
        exported("stamp_sum", Routes::ReporterNeedsContext),
    );

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("its reporter needs a context this boundary does not supply");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.missing_context"
    );
}

/// A skip says where the walk stopped, not only which declaration vanished.
#[test]
fn a_skip_names_the_parameter_and_the_field_that_stopped_it() {
    let mut binding = binding();
    binding.crossing("Stamp", Choice::Struct);
    binding.declare_type("Label", Choice::Struct);
    binding.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(caller) = outcome(&generation, "fn:label_len") else {
        panic!("`Label` has a field nothing can carry");
    };
    assert_eq!(
        caller.dependency_path,
        vec!["fn:label_len", "param 0", "field text"]
    );
    // The struct reached the same cause by its own path.
    let Outcome::Skipped(strukt) = outcome(&generation, "type:Label") else {
        panic!("the struct cannot be represented either");
    };
    assert_eq!(strukt.dependency_path, vec!["type:Label", "field text"]);
}

/// A public declaration the target refuses skips every declaration requiring
/// it, even though every conversion involved was planned successfully.
///
/// This is the retention loop rather than value planning: the struct's
/// conversion is fine, and it is the public `Stamp` that does not exist.
#[test]
fn a_refused_public_declaration_skips_what_requires_it() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::StructWithoutSurface);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    // Unrelated, and skipped for a cause of its own: `Label` holds a `String`,
    // which nothing here carries.
    binding.declare_fn("label_len", exported("label_len", Routes::None));

    let generation = binding.generate(model()).expect("plans");
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
    assert_eq!(caller.dependency_path.first().unwrap(), "fn:stamp_sum");
    assert!(caller
        .dependency_path
        .iter()
        .any(|step| step == "type:Stamp"));
    assert!(generation.rust().is_empty());
}

/// Two supported but different child conversions make two struct conversions.
///
/// Nothing is refused here, so this says the children belong to a conversion's
/// identity on their own rather than only when one of them fails.
#[test]
fn two_supported_children_make_two_struct_conversions() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    // One field of one parameter crosses the same `i64` a different way. The
    // struct above it is declared identically in both functions, so only the
    // child can make the two conversions differ.
    binding.at_site("stamp_max", "param 0.field secs", Choice::ScalarThrough);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Stamp` twice, `i64` into Rust twice, `i64` out of Rust once.
    assert_eq!(generation.values().len(), 5);
    assert_eq!(generation.functions().len(), 2);
    // And the second conversion is the one that was asked for, not the first
    // one shared under a different name.
    let rust = generation.rust();
    assert_eq!(rust.matches("rebase(").count(), 1, "{rust}");
}

/// Two values the binding declared the same way share one conversion, however
/// far apart the declarations are.
///
/// The other half of the contract a conversion key carries: different keys
/// must not share, and equal keys must. Here the site override restates the
/// type's own declaration, so the key it yields is equal and nothing new is
/// planned — a target interning a fresh identity per lookup would silently
/// double the conversions instead.
#[test]
fn an_override_restating_the_default_shares_its_conversion() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.at_site("stamp_max", "param 0.field secs", Choice::Scalar);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_fn("stamp_max", exported("stamp_max", Routes::None));

    let generation = binding.generate(model()).expect("plans");
    // The same three as if nothing had been recorded for that field.
    assert_eq!(generation.values().len(), 3);
}

/// A raw identifier and its plain spelling are one name, and the writer treats
/// them as one.
///
/// `r#v0` reserves `v0`: a temporary that took the plain spelling would shadow
/// the parameter, which is the same defect as an ordinary collision wearing a
/// different hat.
#[test]
fn a_raw_identifier_parameter_reserves_its_plain_spelling() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn(
        "stamp_pick",
        Choice::Function {
            symbol: "stamp_pick".to_string(),
            routes: Routes::None,
            param_names: vec!["r#v0".to_string(), "r#v1".to_string()],
            attrs: Vec::new(),
            unsafety: false,
        },
    );

    let generation = binding.generate(model()).expect("plans");
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
/// the one *it* requires.
#[test]
fn a_refusal_travels_a_chain_of_public_requirements() {
    let mut binding = binding();
    // Every conversion here succeeds: what fails is a public declaration, two
    // edges away from the function that needs it.
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    binding.declare_type("Stamp", Choice::StructRequiring("Point".to_string()));
    binding.declare_type("Point", Choice::StructWithoutSurface);

    let generation = binding.generate(model()).expect("plans");
    for declaration in ["type:Point", "type:Stamp", "fn:stamp_sum"] {
        let Outcome::Skipped(skip) = outcome(&generation, declaration) else {
            panic!("{declaration} depends on a public declaration this target refuses");
        };
        assert_eq!(
            skip.capability.as_str(),
            "unsupported.mini.no_public_struct",
            "{declaration} carries the one cause"
        );
        assert_eq!(skip.dependency_path.first().unwrap(), declaration);
    }
    // Two edges, so the far end of the chain names both of them.
    let Outcome::Skipped(caller) = outcome(&generation, "fn:stamp_sum") else {
        unreachable!("checked above");
    };
    assert_eq!(
        caller.dependency_path,
        vec!["fn:stamp_sum", "type:Stamp", "type:Point"]
    );
    assert!(generation.rust().is_empty());
}

/// A boundary that passes a carrier its conversion does not read is a defect,
/// and fails the build rather than emitting a wrapper that reads the wrong
/// thing.
#[test]
fn a_wrapper_parameter_must_carry_what_its_conversion_reads() {
    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", Choice::FunctionWithWrongInput);

    let error = binding.generate(model()).expect_err("refuses");
    let EngineError::Planning(PlanningError::InternalInvariant(message)) = error else {
        panic!("an adapter describing an impossible boundary is a defect, not a capability gap");
    };
    assert!(message.contains("its conversion reads"), "{message}");
}

/// A guard the capture reader injected reaches the generated file, whatever
/// else the run retained.
///
/// The one in production asserts that the source crate's features match the set
/// the capture was filtered by. It belongs to no declaration, so nothing in
/// retention decides its fate, and a run that emitted no wrapper at all must
/// still carry it — otherwise a binding that skipped everything would also skip
/// the check that its source crate is the one it was generated against.
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
    let mut full = binding();
    full.declare_type("Stamp", Choice::Struct);
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
    let mut bare = binding();
    bare.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));
    let generation = bare.generate(guarded()).expect("plans");
    assert_eq!(emitted(&generation), 0);
    assert!(
        generation.rust().contains("konst::assertc_eq!"),
        "a run that emitted nothing still carries its guard:\n{}",
        generation.rust()
    );
}

/// A condition the capture reader could not evaluate reaches the wrapper
/// generated for the item that carries it.
///
/// The reader rewrites such a condition back onto the item rather than guessing
/// — `unix`, or a custom `--cfg` flag, is not something it has a rule for. The
/// model then carries it uninterpreted, and the writer re-applies it, so the
/// wrapper exists exactly where the function it calls does. Without that, a
/// binding whose condition is false where the source crate compiles would call
/// a function that is not there, and fail to compile.
#[test]
fn a_condition_the_reader_could_not_evaluate_reaches_the_wrapper() {
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
            #[cfg(some_custom_flag)]
            pub fn stamp_sum(stamp: Stamp) -> i64 {
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
    // The model takes the item as it is: an attribute it cannot interpret is
    // not a reason to refuse one.
    assert_eq!(flat.unsupported().count(), 0);
    assert!(flat.function("stamp_sum").is_some());

    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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
///
/// The struct carries a second condition the function does not, which is what
/// separates per-condition dedup from comparing whole attribute sets. Restating
/// a condition would compile — conjunction is idempotent — so what this holds to
/// is the generated file being readable.
#[test]
fn a_wrapper_inherits_the_condition_of_every_source_item_it_names() {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    let flat = Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model");

    let mut binding = binding();
    binding.declare_type("Stamp", Choice::Struct);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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
/// and nothing it generates for the others.
///
/// A member that exists only sometimes can only be read sometimes, and the
/// initializer that consumes the read exists only then; leaving any one of the
/// three unconditional is what used to make the wrapper name a field the source
/// struct may not have. The mirror the target declares carries it too, which is
/// what makes the member and its read agree.
#[test]
fn a_field_condition_reaches_every_statement_that_serves_the_field() {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    let flat = Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model");

    let mut binding = binding();
    binding.declare_type("Stamp", Choice::StructWithMirror);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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

/// An item's condition reaches the Rust a target contributes for its public
/// declaration of that item, not only the wrapper.
///
/// A mirror emitted where the struct it mirrors is absent is a type the foreign
/// API declares and the build does not have.
#[test]
fn an_item_condition_reaches_the_declaration_a_target_contributes() {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
    ]
    .into_iter()
    .map(|item| (item, location.clone()))
    .collect();
    let flat = Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model");

    let mut binding = binding();
    binding.declare_type("Stamp", Choice::StructWithMirror);
    binding.declare_fn("stamp_sum", exported("stamp_sum", Routes::None));

    let generation = binding.generate(flat).expect("plans");
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

/// The part of a wrapper's form a target may state — attributes beyond
/// `#[no_mangle]`, and `unsafe` — is rendered as stated; the linkage is the
/// writer's, and a target restating it is contradictory input.
#[test]
fn a_target_states_a_wrappers_attributes_and_safety_but_not_its_linkage() {
    let form = |attrs: Vec<syn::Attribute>, unsafety: bool| {
        let mut binding = binding();
        binding.declare_type("Stamp", Choice::Struct);
        binding.declare_fn(
            "stamp_sum",
            Choice::Function {
                symbol: "stamp_sum".to_string(),
                routes: Routes::None,
                param_names: Vec::new(),
                attrs,
                unsafety,
            },
        );
        binding.generate(model())
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
    let mut binding = binding();
    binding.declare_type("Token", Choice::Handle);
    binding.declare_fn("token_new", exported("token_new", Routes::None));
    binding.declare_fn("token_use", exported("token_use", Routes::Reported));

    let generation = binding.generate(model()).expect("plans");
    assert_eq!(emitted(&generation), 3);
    // `Token` out of Rust, `Token` into Rust, `i64` out of Rust — and the type's
    // own request planned nothing the functions did not.
    assert_eq!(generation.values().len(), 3);
    // Two exported functions, plus the release under the type's own identity.
    let symbols: Vec<&str> = generation
        .functions()
        .iter()
        .map(|plan| plan.abi.symbol.as_str())
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
    assert!(release.failures.is_empty());
}

/// A null address arriving where a handle is consumed is a binding failure, and
/// the boundary has to say where it goes like any other.
#[test]
fn a_consumed_handle_needs_a_binding_route() {
    let mut binding = binding();
    binding.declare_type("Token", Choice::Handle);
    binding.declare_fn("token_use", exported("token_use", Routes::None));

    let generation = binding.generate(model()).expect("plans");
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

/// A handle nobody can free is not a handle: the type is skipped where the
/// release could not be placed, and the function taking it with it.
#[test]
fn a_handle_without_a_release_skips_the_type_and_what_takes_it() {
    let mut binding = binding();
    binding.declare_type("Token", Choice::HandleWithoutRelease);
    binding.declare_fn("token_use", exported("token_use", Routes::Reported));

    let generation = binding.generate(model()).expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "type:Token") else {
        panic!("its release has nowhere to go");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.mini.no_release");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:token_use") else {
        panic!("it takes a type that was not declared");
    };
    assert_eq!(skip.capability.as_str(), "unsupported.mini.no_release");
    assert_eq!(skip.dependency_path, ["fn:token_use", "type:Token"]);
    assert!(generation.rust().is_empty());
}
