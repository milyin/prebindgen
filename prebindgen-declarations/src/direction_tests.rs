//! The compiler must not identify incoming and outgoing carrier vocabularies.
use super::*;
use prebindgen_conversions::{value, WireKind};

#[derive(Clone, Debug)]
struct IncomingWire(Type);
#[derive(Clone, Debug)]
struct OutgoingWire(Type);
impl WireKind for IncomingWire {
    fn rust_type(&self) -> Type {
        self.0.clone()
    }
}
impl WireKind for OutgoingWire {
    fn rust_type(&self) -> Type {
        self.0.clone()
    }
}
#[derive(Clone, Debug)]
enum IncomingChain {
    Optional,
}
#[derive(Clone, Debug)]
enum IncomingAnnotation {
    Field,
}
#[derive(Clone, Debug)]
enum OutgoingChain {
    Optional,
}
#[derive(Clone, Debug)]
enum OutgoingAnnotation {
    Field,
}
#[derive(Clone, Debug)]
struct IncomingGroup;
#[derive(Clone, Debug)]
struct OutgoingGroup;
struct Directional;
impl Codec for Directional {
    type Input = InputPlan<IncomingWire, (), IncomingChain, IncomingGroup, IncomingAnnotation, ()>;
    type Output =
        OutputPlan<OutgoingWire, (), OutgoingChain, OutgoingGroup, OutgoingAnnotation, ()>;
    // Source attachments are independent of the converters' wire types.
    type Schema = prebindgen_conversions::PlanSchema<Type>;
    type Context = ();
    type Extra = ();
    test_operations!();

    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInputPlan<Self>> {
        let wire = &r.wire;
        value::input(
            r.ty,
            r.wire.clone(),
            r.target,
            quote!(#wire),
            IncomingWire(r.ty.clone()),
        )
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutputPlan<Self>> {
        value::output(
            r.ty,
            r.wire,
            r.value,
            r.access.is_shared(),
            |v| v,
            OutgoingWire(r.ty.clone()),
        )
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &Type, _: &()) -> bool {
        false
    }
    fn input_group(&self, _: GroupKind) -> Result<IncomingGroup> {
        Ok(IncomingGroup)
    }
    fn output_group(&self, _: GroupKind) -> Result<OutgoingGroup> {
        Ok(OutgoingGroup)
    }
    fn input_field(&self, _name: &syn::Ident) -> Result<IncomingAnnotation> {
        Ok(IncomingAnnotation::Field)
    }
    fn output_field(&self, _name: &syn::Ident) -> Result<OutgoingAnnotation> {
        Ok(OutgoingAnnotation::Field)
    }
    fn input_optional(
        &self,
        r: InputOptional<'_, Self>,
    ) -> Result<OptionalEncoding<IncomingWire, IncomingChain>> {
        Ok(OptionalEncoding {
            presence: Presence::Flag(Wire {
                name: r.presence,
                kind: IncomingWire(syn::parse_quote!(i32)),
            }),
            metadata: IncomingChain::Optional,
            absent_values: vec![],
        })
    }
    fn output_optional(
        &self,
        r: OutputOptional<'_, Self>,
    ) -> Result<OptionalEncoding<OutgoingWire, OutgoingChain>> {
        Ok(OptionalEncoding {
            presence: Presence::Flag(Wire {
                name: r.presence,
                kind: OutgoingWire(syn::parse_quote!(i32)),
            }),
            metadata: OutgoingChain::Optional,
            absent_values: prebindgen_conversions::wire_definitions(std::iter::once(&*r.payload))
                .map(|w| (w.name.clone(), quote!(0)))
                .collect(),
        })
    }
    fn binding_failure_layout(&self, _: &()) -> Result<Option<CodecOutputNode<Self>>> {
        Ok(Some(WireNode::Leaf(vec![Wire {
            name: syn::parse_quote!(failure),
            kind: OutgoingWire(syn::parse_quote!(i32)),
        }])))
    }
}

#[test]
fn shared_compiler_accepts_unrelated_input_and_output_tree_families() {
    let flat = Flat::parse(
        syn::parse_file(
            "pub struct Pair { pub x:i64, pub y:Option<i64> } pub fn exchange(p:Pair)->Pair{}",
        )
        .unwrap()
        .items
        .into_iter()
        .map(|item| (item, Default::default())),
    )
    .unwrap();
    let input = TestInput::Branch {
        operation: TestInputBranch::Fields,
        children: vec![
            TestInput::Leaf(TestInputLeafOp::Value),
            TestInput::Unary {
                operation: TestInputUnary::Optional,
                child: Box::new(TestInput::Leaf(TestInputLeafOp::Value)),
            },
        ],
    };
    let output = TestOutput::Branch {
        operation: TestOutputBranch::Fields,
        children: vec![
            TestOutput::Leaf(TestOutputLeafOp::Value),
            TestOutput::Unary {
                operation: TestOutputUnary::Optional,
                child: Box::new(TestOutput::Leaf(TestOutputLeafOp::Value)),
            },
        ],
    };
    let incoming = compile_input(&flat, &Directional, "exchange", "p", &input, &()).unwrap();
    let outgoing = compile_output(&flat, &Directional, "exchange", &output, &()).unwrap();
    let _: &WireNode<IncomingWire, IncomingChain, IncomingGroup, IncomingAnnotation, ()> =
        &incoming.node;
    let _: &WireNode<OutgoingWire, OutgoingChain, OutgoingGroup, OutgoingAnnotation, ()> =
        &outgoing.outcomes.success;
    let WireNode::Branch { children, .. } = &incoming.node else {
        panic!("record input must group fields")
    };
    assert!(children.iter().all(|n| matches!(
        n,
        WireNode::Annotation {
            meta: IncomingAnnotation::Field,
            ..
        }
    )));
    let WireNode::Branch { children, .. } = &outgoing.outcomes.success else {
        panic!("record output must group fields")
    };
    assert!(children.iter().all(|n| matches!(
        n,
        WireNode::Annotation {
            meta: OutgoingAnnotation::Field,
            ..
        }
    )));
    assert_eq!(
        prebindgen_conversions::wire_definitions(std::iter::once(&incoming.node)).count(),
        3
    );
    assert_eq!(outgoing.values.len(), 3);
    let failure = outgoing.outcomes.binding_failure.as_ref().unwrap();
    assert_eq!(
        prebindgen_conversions::wire_definitions(std::iter::once(failure))
            .next()
            .unwrap()
            .name,
        "failure"
    );
}
