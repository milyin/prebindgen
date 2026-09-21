//! What the engine decides, over a target that answers in one line.
//!
//! The adapter here is deliberately not a language: it exists to make the
//! engine's own contracts observable — which conversions are shared, what a
//! missing capability takes down with it, what happens when a boundary leaves a
//! declared failure unrouted. The two real adapters, and the generated code
//! rustc compiles, live in `examples/v2check`.

use prebindgen_flat::flat::{Flat, ScalarKind, TypeKind};

use crate::{
    decl::Declaration,
    outcome::{EngineError, Outcome},
    plan::{generate, BindingRequests},
    target::{
        AbiSpec, Access, BoundarySpec, ChildValue, Described, FailureCategory, Layout, OperandSpec,
        Operation, OperationType, OutputPlacement, ParamRole, PlanningError, PrimitiveFailure,
        PrimitiveSpec, Protocol, Relation, RelationId, ReprSpec, ResolvedShape, ResolvedValues,
        SelectionQuery, SiteDescriptor, SourceItem, StandardOp, SurfaceRequest, SurfaceSpec,
        Target, TargetAttempt, TargetSupport, Terminal, Unsupported, WireType, WrapperParam,
    },
};

/// Two structs and three functions, one of which has a field nothing can carry;
/// an opaque type, and the two functions that hand one out and take it back.
fn model() -> Flat {
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
    Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model")
}

/// What this target was told about one value or one function.
#[derive(Clone, Debug, PartialEq)]
enum Policy {
    Scalar,
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
    /// mirror of the source struct, one member per field. Only this policy
    /// produces an artifact, so every other test's generated file is unchanged.
    StructWithMirror,
}

/// What a boundary does about failures.
#[derive(Clone, Copy, Debug, PartialEq)]
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

fn exported(symbol: &str, routes: Routes) -> Policy {
    Policy::Function {
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
}

struct Mini;

impl Target for Mini {
    type Policy = Policy;
    type Payload = Payload;

    fn select(&self, query: &SelectionQuery<'_, Policy>) -> TargetSupport<RelationId> {
        let want_struct = matches!(
            query.policy,
            Policy::Struct
                | Policy::FallibleStruct
                | Policy::StructWithoutSurface
                | Policy::StructRequiring(_)
                | Policy::StructWithMirror
        );
        for (id, relation) in query.candidates {
            match (relation, want_struct) {
                (Relation::Struct(_), true) => return Ok(TargetAttempt::Ready(*id)),
                (Relation::Atomic, false) => return Ok(TargetAttempt::Ready(*id)),
                _ => {}
            }
        }
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.mini.no_relation",
            "no relation for this policy",
        )))
    }

    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        policy: &Policy,
    ) -> TargetSupport<ReprSpec<Payload>> {
        match shape.relation {
            Relation::Atomic if matches!(policy, Policy::Handle | Policy::HandleWithoutRelease) => {
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
                Ok(TargetAttempt::Ready(ReprSpec {
                    layout: Layout::Scalar(carrier.clone()),
                    protocol: Protocol::terminal(PrimitiveSpec::identity(OperationType::Carrier(
                        carrier,
                    ))),
                    release: None,
                }))
            }
            Relation::Struct(strukt) => {
                let ident = quote::format_ident!("{}", strukt.name);
                let aggregate = WireType::abi(syn::parse_quote!(#ident));
                let item = shape.strukt.expect("a struct relation carries its strukt");
                let fallible = matches!(policy, Policy::FallibleStruct);
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
        site: &SiteDescriptor<'_>,
        values: &ResolvedValues<'_, Payload>,
        policy: &Policy,
    ) -> TargetSupport<BoundarySpec<Payload>> {
        if matches!(policy, Policy::FunctionWithWrongInput) {
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
        // A release site carries the handle type's own policy: its symbol is
        // derived, and a null address is not a failure it can raise.
        let release = match (site.function, policy) {
            (None, Policy::Handle) => Some(exported(
                &format!("{}_free", site.declaration.name()),
                Routes::None,
            )),
            (None, Policy::HandleWithoutRelease) => {
                return Ok(TargetAttempt::Unsupported(Unsupported::new(
                    "unsupported.mini.no_release",
                    "this target has nowhere to place a release",
                )))
            }
            _ => None,
        };
        let Policy::Function {
            symbol,
            routes,
            param_names,
            attrs,
            unsafety,
        } = release.as_ref().unwrap_or(policy)
        else {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` is exported under a value policy",
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
        request: &SurfaceRequest<'_, Policy>,
        values: &ResolvedValues<'_, Payload>,
    ) -> TargetSupport<SurfaceSpec<Payload>> {
        let requires = values
            .inputs
            .iter()
            .filter_map(|value| crate::target::Requirement::of(&value.crossing.ty))
            .collect();
        if matches!(
            (request.policy, request.item),
            (Policy::StructWithoutSurface, SourceItem::Struct(_))
        ) {
            return Ok(TargetAttempt::Unsupported(Unsupported::new(
                "unsupported.mini.no_public_struct",
                "this struct converts, and has no public declaration here",
            )));
        }
        // The mirror is what a real C target contributes: a struct of its own,
        // one member per source field, each under that field's condition.
        let rust = match (request.policy, request.item) {
            (Policy::StructWithMirror, SourceItem::Struct(strukt)) => {
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
            requires: match (request.policy, request.item) {
                (_, SourceItem::Function(_)) => requires,
                (Policy::StructRequiring(other), SourceItem::Struct(_)) => {
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
        }
    }

    /// The report column is the policy's own word; the placement is the
    /// symbol a function exports, and a type's own name otherwise — this
    /// target has no foreign spelling of its own.
    fn describe(&self, policy: &Policy) -> Described {
        match policy {
            Policy::Function { symbol, .. } => Described::new("function", symbol),
            Policy::Scalar => Described::new("scalar", ""),
            _ => Described::new("strukt", ""),
        }
    }
}

fn requests() -> BindingRequests<Policy> {
    BindingRequests::new("mini", syn::parse_quote!(source), Policy::Scalar)
}

fn ty(name: &str) -> Declaration {
    Declaration::Type(prebindgen_flat::TypeKey::parse(name).expect("a test names a type"))
}

fn function(name: &str) -> Declaration {
    Declaration::Function(syn::parse_str(name).expect("a test names an ident"))
}

fn outcome<'a, P>(generation: &'a crate::run::Generation<P>, id: &str) -> &'a Outcome {
    &generation
        .report()
        .declarations
        .iter()
        .find(|entry| entry.declaration.to_string() == id)
        .unwrap_or_else(|| panic!("no report entry for {id}"))
        .outcome
}

/// Two exported functions taking the same struct the same way share its
/// conversion — and its two field conversions, and the result conversion.
#[test]
fn one_conversion_serves_every_value_that_crosses_the_same_way() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let max = requests.policy(exported("stamp_max", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);
    requests.output(function("stamp_max"), max);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 3);
    // `Stamp` into Rust, `i64` into Rust, `i64` out of Rust. Twice over, and
    // once for the struct's own request, is still three.
    assert_eq!(generation.values().len(), 3);
    assert_eq!(generation.functions().len(), 2);
}

/// A per-site override is a different effective policy, so it is a different
/// conversion — which is the whole reason the policy is part of a node's
/// identity.
#[test]
fn a_site_override_does_not_share_the_default_conversion() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    let fallible = requests.policy(Policy::FallibleStruct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let max = requests.policy(exported("stamp_max", Routes::Reported));
    requests.site_policies.insert(
        (
            Declaration::Function(syn::parse_quote!(stamp_max)),
            "param 0".to_string(),
        ),
        fallible,
    );
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);
    requests.output(function("stamp_max"), max);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 3);
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    requests.type_policies.insert("Label".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let len = requests.policy(exported("label_len", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(ty("Label"), strukt);
    requests.output(function("stamp_sum"), sum);
    requests.output(function("label_len"), len);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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

/// A public declaration that requires a type the binding never declared is
/// skipped, not emitted against a type that will not exist.
#[test]
fn a_function_needing_an_undeclared_public_type_is_skipped() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(function("stamp_sum"), sum);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::FallibleStruct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
fn a_value_policy_on_an_exported_function_is_an_error() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    requests.output(ty("Stamp"), strukt);
    // A struct policy where a function policy belongs.
    requests.output(function("stamp_sum"), strukt);

    let error = generate(model(), &Mini, requests, "fixture").expect_err("refuses");
    assert!(matches!(
        error,
        EngineError::Planning(PlanningError::InvalidInput(_))
    ));
}

/// A declaration the source never captured is an error, not a skip — the
/// same rule the engine already held for its report-only run.
#[test]
fn a_declaration_naming_nothing_is_an_error() {
    let mut requests = requests();
    let sum = requests.policy(exported("nope", Routes::None));
    requests.output(function("nope"), sum);

    let error = generate(model(), &Mini, requests, "fixture").expect_err("refuses");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
}

/// An ignored declaration is accounted for apart from a skipped one: an ignore is a
/// decision, not a gap.
#[test]
fn an_ignored_declaration_is_neither_emitted_nor_skipped() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    requests.output(ty("Stamp"), strukt);
    requests.ignored.push(function("stamp_max"));

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    let counts = generation.report().counts();
    assert_eq!((counts.emitted, counts.skipped, counts.ignored), (1, 0, 1));
    assert!(matches!(
        outcome(&generation, "fn:stamp_max"),
        Outcome::Ignored
    ));
}

/// Two runs over unchanged inputs produce the same report and the same code, so
/// a diff of either means something.
#[test]
fn a_run_over_unchanged_input_produces_the_same_output() {
    let run = || {
        let mut requests = requests();
        let strukt = requests.policy(Policy::Struct);
        requests.type_policies.insert("Stamp".to_string(), strukt);
        let sum = requests.policy(exported("stamp_sum", Routes::None));
        requests.output(ty("Stamp"), strukt);
        requests.output(function("stamp_sum"), sum);
        let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
        (generation.report().to_json(), generation.rust().to_string())
    };
    assert_eq!(run(), run());
}

/// A conversion recorded for a *field* makes its struct a different
/// conversion, whichever order the two uses are planned in.
///
/// The cache is consulted after the children are planned for exactly this
/// reason: keyed on the struct's own policy alone, the second use would inherit
/// the first one's conversion and its support outcome, in whichever direction
/// the two happened to be requested.
#[test]
fn a_field_override_is_part_of_its_struct_conversion() {
    let plan = |defaults_first: bool| {
        let mut requests = requests();
        let strukt = requests.policy(Policy::Struct);
        requests.type_policies.insert("Stamp".to_string(), strukt);
        let sum = requests.policy(exported("stamp_sum", Routes::None));
        let max = requests.policy(exported("stamp_max", Routes::None));
        // A struct policy on a scalar field: the target offers no struct
        // relation for an `i64`, so this child cannot be selected at all.
        requests.site_policies.insert(
            (
                Declaration::Function(syn::parse_quote!(stamp_max)),
                "param 0.field secs".to_string(),
            ),
            strukt,
        );
        // Order matters twice over: which function is planned first, and
        // whether the struct's own request primed the conversion before either
        // of them. The refusal-first case must not be primed, or it would not
        // test what happens when a refusal is met before any success.
        if defaults_first {
            requests.output(ty("Stamp"), strukt);
            requests.output(function("stamp_sum"), sum);
            requests.output(function("stamp_max"), max);
        } else {
            requests.output(function("stamp_max"), max);
            requests.output(function("stamp_sum"), sum);
            requests.output(ty("Stamp"), strukt);
        }
        generate(model(), &Mini, requests, "fixture").expect("plans")
    };
    for defaults_first in [true, false] {
        let generation = plan(defaults_first);
        let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_max") else {
            panic!("its `secs` field is configured with a policy nothing can serve");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let pick = requests.policy(Policy::Function {
        symbol: "stamp_pick".to_string(),
        routes: Routes::None,
        // The names a writer would otherwise allocate for itself.
        param_names: vec!["v0".to_string(), "v1".to_string()],
        attrs: Vec::new(),
        unsafety: false,
    });
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_pick"), pick);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::FallibleStruct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::ReporterNeedsContext));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    requests.type_policies.insert("Label".to_string(), strukt);
    let len = requests.policy(exported("label_len", Routes::None));
    requests.output(ty("Label"), strukt);
    requests.output(function("label_len"), len);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::StructWithoutSurface);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let len = requests.policy(exported("label_len", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);
    // Unrelated, and skipped for a cause of its own.
    requests
        .type_policies
        .insert("Label".to_string(), requests.default_policy);
    requests.output(function("label_len"), len);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    // A second entry with the same settings is still a second entry: sharing a
    // conversion means sharing the policy, not writing an equal-looking one.
    let other_scalar = requests.policy(Policy::Scalar);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let max = requests.policy(exported("stamp_max", Routes::None));
    requests.site_policies.insert(
        (
            Declaration::Function(syn::parse_quote!(stamp_max)),
            "param 0.field secs".to_string(),
        ),
        other_scalar,
    );
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);
    requests.output(function("stamp_max"), max);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 3);
    // `Stamp` twice, `i64` into Rust twice, `i64` out of Rust once.
    assert_eq!(generation.values().len(), 5);
    assert_eq!(generation.functions().len(), 2);
}

/// A raw identifier and its plain spelling are one name, and the writer treats
/// them as one.
///
/// `r#v0` reserves `v0`: a temporary that took the plain spelling would shadow
/// the parameter, which is the same defect as an ordinary collision wearing a
/// different hat.
#[test]
fn a_raw_identifier_parameter_reserves_its_plain_spelling() {
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let pick = requests.policy(Policy::Function {
        symbol: "stamp_pick".to_string(),
        routes: Routes::None,
        param_names: vec!["r#v0".to_string(), "r#v1".to_string()],
        attrs: Vec::new(),
        unsafety: false,
    });
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_pick"), pick);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    // Every conversion here succeeds: what fails is a public declaration, two
    // edges away from the function that needs it.
    let requiring = requests.policy(Policy::StructRequiring("Point".to_string()));
    let refused = requests.policy(Policy::StructWithoutSurface);
    requests
        .type_policies
        .insert("Stamp".to_string(), requiring);
    requests.type_policies.insert("Point".to_string(), refused);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(function("stamp_sum"), sum);
    requests.output(ty("Stamp"), requiring);
    requests.output(ty("Point"), refused);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let wrong = requests.policy(Policy::FunctionWithWrongInput);
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), wrong);

    let error = generate(model(), &Mini, requests, "fixture").expect_err("refuses");
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
    let mut full = requests();
    let strukt = full.policy(Policy::Struct);
    full.type_policies.insert("Stamp".to_string(), strukt);
    let sum = full.policy(exported("stamp_sum", Routes::None));
    full.output(ty("Stamp"), strukt);
    full.output(function("stamp_sum"), sum);
    let generation = generate(guarded(), &Mini, full, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 2);
    assert!(
        generation.rust().contains("konst::assertc_eq!"),
        "the guard must reach the generated file:\n{}",
        generation.rust()
    );

    // And with nothing to emit: the declaration is skipped, and the guard is
    // still there.
    let mut bare = requests();
    let sum = bare.policy(exported("stamp_sum", Routes::None));
    bare.output(function("stamp_sum"), sum);
    let generation = generate(guarded(), &Mini, bare, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 0);
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

    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(flat, &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 2);
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

    let mut requests = requests();
    let strukt = requests.policy(Policy::Struct);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(flat, &Mini, requests, "fixture").expect("plans");
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

    let mut requests = requests();
    let strukt = requests.policy(Policy::StructWithMirror);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(flat, &Mini, requests, "fixture").expect("plans");
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

    let mut requests = requests();
    let strukt = requests.policy(Policy::StructWithMirror);
    requests.type_policies.insert("Stamp".to_string(), strukt);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    requests.output(ty("Stamp"), strukt);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(flat, &Mini, requests, "fixture").expect("plans");
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
        let mut requests = requests();
        let strukt = requests.policy(Policy::Struct);
        requests.type_policies.insert("Stamp".to_string(), strukt);
        let sum = requests.policy(Policy::Function {
            symbol: "stamp_sum".to_string(),
            routes: Routes::None,
            param_names: Vec::new(),
            attrs,
            unsafety,
        });
        requests.output(ty("Stamp"), strukt);
        requests.output(function("stamp_sum"), sum);
        generate(model(), &Mini, requests, "fixture")
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
    let mut requests = requests();
    let handle = requests.policy(Policy::Handle);
    requests.type_policies.insert("Token".to_string(), handle);
    let new = requests.policy(exported("token_new", Routes::None));
    let use_ = requests.policy(exported("token_use", Routes::Reported));
    requests.output(ty("Token"), handle);
    requests.output(function("token_new"), new);
    requests.output(function("token_use"), use_);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 3);
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
    let mut requests = requests();
    let handle = requests.policy(Policy::Handle);
    requests.type_policies.insert("Token".to_string(), handle);
    let use_ = requests.policy(exported("token_use", Routes::None));
    requests.output(ty("Token"), handle);
    requests.output(function("token_use"), use_);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
    let mut requests = requests();
    let handle = requests.policy(Policy::HandleWithoutRelease);
    requests.type_policies.insert("Token".to_string(), handle);
    let use_ = requests.policy(exported("token_use", Routes::Reported));
    requests.output(ty("Token"), handle);
    requests.output(function("token_use"), use_);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
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
