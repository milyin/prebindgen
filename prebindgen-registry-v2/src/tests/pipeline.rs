//! What the engine decides, over a target that answers in one line.
//!
//! The adapter here is deliberately not a language: it exists to make the
//! engine's own contracts observable — which conversions are shared, what a
//! missing capability takes down with it, what happens when a boundary leaves a
//! declared failure unrouted. The two real adapters, and the generated code
//! rustc compiles, live in `examples/v2check`.

use prebindgen_flat::flat::{Flat, ScalarKind, TypeKind};

use crate::{
    decl::{DeclaredElement, ElementKind},
    outcome::{EngineError, Outcome},
    plan::{generate, BindingRequests},
    target::{
        AbiSpec, Access, BoundarySpec, ChildValue, FailureCategory, Layout, NativeParam,
        OperandSpec, Operation, OperationType, OutputPlacement, ParamRole, PlanningError,
        PrimitiveFailure, PrimitiveSpec, Protocol, Relation, RelationId, ReprSpec, ResolvedShape,
        ResolvedValues, SelectionQuery, SiteDescriptor, SourceItem, StandardOp, SurfaceRequest,
        SurfaceSpec, Target, TargetAttempt, TargetSupport, Terminal, Unsupported, WireType,
    },
};

/// Two records and three functions, one of which has a field nothing can carry.
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
    /// A record read through its members.
    Record,
    /// The same, with member reads that can fail — which is what makes a
    /// boundary's failure routes observable.
    FallibleRecord,
    Function {
        symbol: String,
        /// Whether the boundary routes runtime failures at all.
        routes: bool,
    },
}

#[derive(Clone, Debug)]
enum Payload {
    ReadFallibly,
    Report,
}

struct Mini;

impl Target for Mini {
    type Policy = Policy;
    type Payload = Payload;

    fn select(&self, query: &SelectionQuery<'_, Policy>) -> TargetSupport<RelationId> {
        let want_record = matches!(query.policy, Policy::Record | Policy::FallibleRecord);
        for (id, relation) in query.candidates {
            match (relation, want_record) {
                (Relation::Record(_), true) => return Ok(TargetAttempt::Ready(*id)),
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
                }))
            }
            Relation::Record(record) => {
                let ident = quote::format_ident!("{}", record.record);
                let aggregate = WireType::abi(syn::parse_quote!(#ident));
                let item = shape.record.expect("a record relation carries its record");
                let fallible = matches!(policy, Policy::FallibleRecord);
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
        let Policy::Function { symbol, routes } = policy else {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` is exported under a value policy",
                site.element.rust_origin
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
                    .map(|(index, value)| NativeParam {
                        name: quote::format_ident!("arg{index}"),
                        ty: value.repr.layout.wire().clone(),
                        role: ParamRole::Input(index),
                        mutable: false,
                    })
                    .collect(),
                ret: values.output.map(|value| value.repr.layout.wire().clone()),
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            failures: if *routes {
                vec![crate::target::FailureRoute {
                    category: FailureCategory::Runtime,
                    report: Some(PrimitiveSpec {
                        operands: vec![OperandSpec::error(OperationType::Carrier(
                            WireType::internal(syn::parse_quote!(Error)),
                        ))],
                        result: None,
                        failure: PrimitiveFailure::Infallible,
                        dependencies: Vec::new(),
                        implementation: Operation::Target(Payload::Report),
                    }),
                    on_report_failure: Terminal::Abort,
                    terminate: Terminal::Return(syn::parse_quote!(0)),
                }]
            } else {
                Vec::new()
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
            .filter_map(|value| match value.crossing.ty.kind() {
                TypeKind::Named { id, .. } => Some(crate::decl::ElementId::new(
                    ElementKind::Type,
                    id.name.clone(),
                )),
                _ => None,
            })
            .collect();
        Ok(TargetAttempt::Ready(SurfaceSpec {
            element: request.element.id.clone(),
            requires: match request.item {
                SourceItem::Function(_) => requires,
                SourceItem::Record(_) => Vec::new(),
            },
            rust: Vec::new(),
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
        }
    }
}

fn requests() -> BindingRequests<Policy> {
    BindingRequests::new("mini", syn::parse_quote!(source), Policy::Scalar)
}

fn ty(origin: &str) -> DeclaredElement {
    DeclaredElement::new(ElementKind::Type, origin, origin, "record")
}

fn function(origin: &str) -> DeclaredElement {
    DeclaredElement::new(ElementKind::Function, origin, origin, "function")
}

fn outcome<'a, P>(generation: &'a crate::run::Generation<P>, id: &str) -> &'a Outcome {
    &generation
        .report()
        .elements
        .iter()
        .find(|entry| entry.element.id.as_str() == id)
        .unwrap_or_else(|| panic!("no report entry for {id}"))
        .outcome
}

/// Two exported functions taking the same record the same way share its
/// conversion — and its two field conversions, and the result conversion.
#[test]
fn one_conversion_serves_every_value_that_crosses_the_same_way() {
    let mut requests = requests();
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(Policy::Function {
        symbol: "stamp_sum".to_string(),
        routes: false,
    });
    let max = requests.policy(Policy::Function {
        symbol: "stamp_max".to_string(),
        routes: false,
    });
    requests.output(ty("Stamp"), record);
    requests.output(function("stamp_sum"), sum);
    requests.output(function("stamp_max"), max);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    assert_eq!(generation.report().counts().emitted, 3);
    // `Stamp` into Rust, `i64` into Rust, `i64` out of Rust. Twice over, and
    // once for the record's own request, is still three.
    assert_eq!(generation.values().len(), 3);
    assert_eq!(generation.functions().len(), 2);
}

/// A per-site override is a different effective policy, so it is a different
/// conversion — which is the whole reason the policy is part of a node's
/// identity.
#[test]
fn a_site_override_does_not_share_the_default_conversion() {
    let mut requests = requests();
    let record = requests.policy(Policy::Record);
    let fallible = requests.policy(Policy::FallibleRecord);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(Policy::Function {
        symbol: "stamp_sum".to_string(),
        routes: false,
    });
    let max = requests.policy(Policy::Function {
        symbol: "stamp_max".to_string(),
        routes: true,
    });
    requests.site_policies.insert(
        (
            crate::decl::ElementId::new(ElementKind::Function, "stamp_max"),
            "param 0".to_string(),
        ),
        fallible,
    );
    requests.output(ty("Stamp"), record);
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
    assert_eq!(fallible.failures.len(), 1);
    assert!(generation.rust().contains("read(arg0)"));
    assert!(generation.rust().contains("report(error)"));
}

/// One unsupported field takes down its record and everything requiring it, and
/// leaves everything else alone.
#[test]
fn an_unsupported_field_skips_its_record_and_its_callers() {
    let mut requests = requests();
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    requests.type_policies.insert("Label".to_string(), record);
    let sum = requests.policy(Policy::Function {
        symbol: "stamp_sum".to_string(),
        routes: false,
    });
    let len = requests.policy(Policy::Function {
        symbol: "label_len".to_string(),
        routes: false,
    });
    requests.output(ty("Stamp"), record);
    requests.output(ty("Label"), record);
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
    let Outcome::Skipped(record) = outcome(&generation, "type:Label") else {
        panic!("`Label` has a field nothing can carry");
    };
    assert_eq!(record.capability.as_str(), "unsupported.mini.carrier");
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
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(Policy::Function {
        symbol: "stamp_sum".to_string(),
        routes: false,
    });
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
    let record = requests.policy(Policy::FallibleRecord);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(Policy::Function {
        symbol: "stamp_sum".to_string(),
        routes: false,
    });
    requests.output(ty("Stamp"), record);
    requests.output(function("stamp_sum"), sum);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    let Outcome::Skipped(skip) = outcome(&generation, "fn:stamp_sum") else {
        panic!("its member reads can fail and the boundary routes nothing");
    };
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.boundary.unrouted_failure"
    );
    // The record itself is unaffected: its conversion is fine, and it is the
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
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    requests.output(ty("Stamp"), record);
    // A record policy where a function policy belongs.
    requests.output(function("stamp_sum"), record);

    let error = generate(model(), &Mini, requests, "fixture").expect_err("refuses");
    assert!(matches!(
        error,
        EngineError::Planning(PlanningError::InvalidInput(_))
    ));
}

/// A declared element the source never captured is an error, not a skip — the
/// same rule the engine already held for its report-only run.
#[test]
fn a_declaration_naming_nothing_is_an_error() {
    let mut requests = requests();
    let sum = requests.policy(Policy::Function {
        symbol: "nope".to_string(),
        routes: false,
    });
    requests.output(function("nope"), sum);

    let error = generate(model(), &Mini, requests, "fixture").expect_err("refuses");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
}

/// An ignored element is accounted for apart from a skipped one: an ignore is a
/// decision, not a gap.
#[test]
fn an_ignored_element_is_neither_emitted_nor_skipped() {
    let mut requests = requests();
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    requests.output(ty("Stamp"), record);
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
        let record = requests.policy(Policy::Record);
        requests.type_policies.insert("Stamp".to_string(), record);
        let sum = requests.policy(Policy::Function {
            symbol: "stamp_sum".to_string(),
            routes: false,
        });
        requests.output(ty("Stamp"), record);
        requests.output(function("stamp_sum"), sum);
        let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
        (generation.report().to_json(), generation.rust().to_string())
    };
    assert_eq!(run(), run());
}
