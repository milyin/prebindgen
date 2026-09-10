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
        syn::parse_quote!(
            pub fn stamp_pick(stamp: Stamp, fallback: i64) -> i64 {
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
    /// A record that converts, and whose public declaration this target
    /// refuses — which is what drives the retention loop rather than value
    /// planning.
    RecordWithoutSurface,
    Function {
        symbol: String,
        routes: Routes,
        /// Names for the native parameters, in order. Empty means `arg0`,
        /// `arg1`, … — the well-behaved case that hides a name collision.
        param_names: Vec<String>,
    },
}

/// What a boundary does about failures.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Routes {
    /// None declared, which skips any function whose conversions can fail.
    None,
    /// One route, reporting through an operation that needs no context.
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
    }
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
        let want_record = matches!(
            query.policy,
            Policy::Record | Policy::FallibleRecord | Policy::RecordWithoutSurface
        );
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
        let Policy::Function {
            symbol,
            routes,
            param_names,
        } = policy
        else {
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
            },
            output: match values.output {
                Some(_) => OutputPlacement::Return,
                None => OutputPlacement::Void,
            },
            failures: if *routes == Routes::None {
                Vec::new()
            } else {
                vec![crate::target::FailureRoute {
                    category: FailureCategory::Runtime,
                    report: Some(PrimitiveSpec {
                        operands: {
                            let mut operands = vec![OperandSpec::error(OperationType::Carrier(
                                WireType::internal(syn::parse_quote!(Error)),
                            ))];
                            if *routes == Routes::ReporterNeedsContext {
                                operands.push(OperandSpec::context(
                                    "mini.log",
                                    OperationType::Carrier(WireType::internal(syn::parse_quote!(
                                        Log
                                    ))),
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
                }]
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
        if matches!(
            (request.policy, request.item),
            (Policy::RecordWithoutSurface, SourceItem::Record(_))
        ) {
            return Ok(TargetAttempt::Unsupported(Unsupported::new(
                "unsupported.mini.no_public_record",
                "this record converts, and has no public declaration here",
            )));
        }
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
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let max = requests.policy(exported("stamp_max", Routes::None));
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
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let max = requests.policy(exported("stamp_max", Routes::Reported));
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
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let len = requests.policy(exported("label_len", Routes::None));
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
    let record = requests.policy(Policy::FallibleRecord);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
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
    let sum = requests.policy(exported("nope", Routes::None));
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
        let sum = requests.policy(exported("stamp_sum", Routes::None));
        requests.output(ty("Stamp"), record);
        requests.output(function("stamp_sum"), sum);
        let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
        (generation.report().to_json(), generation.rust().to_string())
    };
    assert_eq!(run(), run());
}

/// A conversion recorded for a *field* makes its record a different
/// conversion, whichever order the two uses are planned in.
///
/// The cache is consulted after the children are planned for exactly this
/// reason: keyed on the record's own policy alone, the second use would inherit
/// the first one's conversion and its support outcome, in whichever direction
/// the two happened to be requested.
#[test]
fn a_field_override_is_part_of_its_record_conversion() {
    let plan = |defaults_first: bool| {
        let mut requests = requests();
        let record = requests.policy(Policy::Record);
        requests.type_policies.insert("Stamp".to_string(), record);
        let sum = requests.policy(exported("stamp_sum", Routes::None));
        let max = requests.policy(exported("stamp_max", Routes::None));
        // A record policy on a scalar field: the target offers no record
        // relation for an `i64`, so this child cannot be selected at all.
        requests.site_policies.insert(
            (
                crate::decl::ElementId::new(ElementKind::Function, "stamp_max"),
                "param 0.field secs".to_string(),
            ),
            record,
        );
        requests.output(ty("Stamp"), record);
        if defaults_first {
            requests.output(function("stamp_sum"), sum);
            requests.output(function("stamp_max"), max);
        } else {
            requests.output(function("stamp_max"), max);
            requests.output(function("stamp_sum"), sum);
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
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    let pick = requests.policy(Policy::Function {
        symbol: "stamp_pick".to_string(),
        routes: Routes::None,
        // The names a writer would otherwise allocate for itself.
        param_names: vec!["v0".to_string(), "v1".to_string()],
    });
    requests.output(ty("Stamp"), record);
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
    let record = requests.policy(Policy::FallibleRecord);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(exported("stamp_sum", Routes::ReporterNeedsContext));
    requests.output(ty("Stamp"), record);
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

/// A skip says where the walk stopped, not only which element vanished.
#[test]
fn a_skip_names_the_parameter_and_the_field_that_stopped_it() {
    let mut requests = requests();
    let record = requests.policy(Policy::Record);
    requests.type_policies.insert("Stamp".to_string(), record);
    requests.type_policies.insert("Label".to_string(), record);
    let len = requests.policy(exported("label_len", Routes::None));
    requests.output(ty("Label"), record);
    requests.output(function("label_len"), len);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    let Outcome::Skipped(caller) = outcome(&generation, "fn:label_len") else {
        panic!("`Label` has a field nothing can carry");
    };
    assert_eq!(
        caller.dependency_path,
        vec!["fn:label_len", "param 0", "field text"]
    );
    // The record reached the same cause by its own path.
    let Outcome::Skipped(record) = outcome(&generation, "type:Label") else {
        panic!("the record cannot be represented either");
    };
    assert_eq!(record.dependency_path, vec!["type:Label", "field text"]);
}

/// A public declaration the target refuses skips every declaration requiring
/// it, even though every conversion involved was planned successfully.
///
/// This is the retention loop rather than value planning: the record's
/// conversion is fine, and it is the public `Stamp` that does not exist.
#[test]
fn a_refused_public_declaration_skips_what_requires_it() {
    let mut requests = requests();
    let record = requests.policy(Policy::RecordWithoutSurface);
    requests.type_policies.insert("Stamp".to_string(), record);
    let sum = requests.policy(exported("stamp_sum", Routes::None));
    let len = requests.policy(exported("label_len", Routes::None));
    requests.output(ty("Stamp"), record);
    requests.output(function("stamp_sum"), sum);
    // Unrelated, and skipped for a cause of its own.
    requests
        .type_policies
        .insert("Label".to_string(), requests.default_policy);
    requests.output(function("label_len"), len);

    let generation = generate(model(), &Mini, requests, "fixture").expect("plans");
    let Outcome::Skipped(record) = outcome(&generation, "type:Stamp") else {
        panic!("this target declares no public record");
    };
    assert_eq!(
        record.capability.as_str(),
        "unsupported.mini.no_public_record"
    );
    let Outcome::Skipped(caller) = outcome(&generation, "fn:stamp_sum") else {
        panic!("a wrapper taking a type that is not declared is unusable");
    };
    // The same cause, reached through this function's own requirement.
    assert_eq!(
        caller.capability.as_str(),
        "unsupported.mini.no_public_record"
    );
    assert_eq!(caller.dependency_path.first().unwrap(), "fn:stamp_sum");
    assert!(caller
        .dependency_path
        .iter()
        .any(|step| step == "type:Stamp"));
    assert!(generation.rust().is_empty());
}
