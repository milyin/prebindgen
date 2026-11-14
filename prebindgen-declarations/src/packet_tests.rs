//! A backend whose sequence boundary has separate pointer and length values.
use super::*;
use prebindgen_conversions::{validate_wire_names, value};
use prebindgen_flat::InputConversion;

#[derive(Clone, Debug)]
enum Metadata {
    Components,
    Alternatives,
    Fields,
    Field,
}
#[derive(Clone, Debug)]
enum PacketChain {
    Optional,
    Sequence,
}
// A test adapter using separate pointer and length parameters.
struct PointerLenAbi;
impl PointerLenAbi {
    fn parameters(node: &WireNode<Type, PacketChain, Metadata>) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(node)
    }
    fn row_fields(node: &WireNode<Type, PacketChain, Metadata>) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(node)
    }
    fn result_fields(
        node: &WireNode<Type, PacketChain, Metadata>,
    ) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(node)
    }
    fn result_packet<'a>(
        node: &'a WireNode<Type, PacketChain, Metadata>,
        bindings: &'a [(syn::Ident, TokenStream)],
    ) -> Result<Vec<(&'a Wire<Type>, &'a TokenStream)>> {
        prebindgen_conversions::match_output_fields(Self::result_fields(node), bindings)
    }
    fn values(node: &WireNode<Type, PacketChain, Metadata>) -> std::vec::IntoIter<&Wire<Type>> {
        fn visit<'a>(
            node: &'a WireNode<Type, PacketChain, Metadata>,
            out: &mut Vec<&'a Wire<Type>>,
        ) {
            match node {
                WireNode::Leaf(w) => out.extend(w),
                WireNode::Chain {
                    meta: PacketChain::Sequence,
                    controls,
                    ..
                } => out.extend(controls),
                WireNode::Chain {
                    child, controls, ..
                } => {
                    out.extend(controls);
                    visit(child, out);
                }
                WireNode::Branch {
                    children, controls, ..
                } => {
                    out.extend(controls);
                    for child in children {
                        visit(child, out);
                    }
                }
            }
        }
        let mut out = vec![];
        visit(node, &mut out);
        out.into_iter()
    }
    fn values_mut(
        node: &mut WireNode<Type, PacketChain, Metadata>,
    ) -> std::vec::IntoIter<&mut Wire<Type>> {
        fn visit<'a>(
            node: &'a mut WireNode<Type, PacketChain, Metadata>,
            out: &mut Vec<&'a mut Wire<Type>>,
        ) {
            match node {
                WireNode::Leaf(w) => out.extend(w),
                WireNode::Chain {
                    meta: PacketChain::Sequence,
                    controls,
                    ..
                } => out.extend(controls),
                WireNode::Chain {
                    child, controls, ..
                } => {
                    out.extend(controls);
                    visit(child, out);
                }
                WireNode::Branch {
                    children, controls, ..
                } => {
                    out.extend(controls);
                    for child in children {
                        visit(child, out);
                    }
                }
            }
        }
        let mut out = vec![];
        visit(node, &mut out);
        out.into_iter()
    }
    fn validate(node: &WireNode<Type, PacketChain, Metadata>) -> Result<()> {
        validate_wire_names(Self::values(node))?;
        match node {
            WireNode::Chain { child, .. } => Self::validate(child),
            WireNode::Branch { children, meta, .. } => {
                if matches!(meta, Metadata::Alternatives) && children.is_empty() {
                    return Err("selection has no alternatives".into());
                }
                for child in children {
                    Self::validate(child)?;
                }
                Ok(())
            }
            WireNode::Leaf(_) => Ok(()),
        }
    }
}
#[derive(Default)]
struct CodecImpl {
    bad_sequence: bool,
    reverse_values: bool,
    unknown_validation_read: bool,
    row_only_leaf: bool,
}
impl Codec for CodecImpl {
    test_operations!();
    type Schema = prebindgen_conversions::PlanSchema<Type, (), PacketChain, Metadata>;
    type Wire = Type;
    type Chain = PacketChain;
    type Branch = Metadata;

    fn validate_input(&self, plan: &CodecInputPlan<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.node)
    }
    fn validate_output(&self, plan: &CodecOutputPlan<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.outcomes.success)?;
        PointerLenAbi::result_packet(&plan.outcomes.success, &plan.values)?;
        Ok(())
    }

    type Context = ();
    type Extra = ();

    fn selector(&self, _: &()) -> Result<Type> {
        Ok(syn::parse_quote!(i32))
    }

    fn output_consumes(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _: &Type,
        _: &(),
    ) -> bool {
        false
    }
    fn input_leaf(&self, request: InputLeaf<'_, Self>) -> Result<CodecInputPlan<Self>> {
        let InputLeaf {
            flat: _flat,
            ty,
            extra: _,
            wire: w,
            target: v,
            ..
        } = request;

        let mut input = value::input(ty, w.clone(), v.clone(), quote!(#w), ty.clone())?;
        if self.row_only_leaf {
            input = InputPlan::new(input.node, quote!(let #v:#ty=#w;), quote!(#v));
        }
        Ok(input)
    }
    fn output_leaf(&self, request: OutputLeaf<'_, Self>) -> Result<CodecOutputPlan<Self>> {
        let OutputLeaf {
            flat: _flat,
            ty,
            extra: _,
            wire: w,
            value: v,
            access,
            ..
        } = request;
        let borrowed = access.is_shared();

        value::output(ty, w, v, borrowed, |v| v, ty.clone())
    }
    fn sequence_input(
        &self,
        request: SequenceInputRequest<'_, Self>,
    ) -> Result<SequenceInput<Type, PacketChain>> {
        let SequenceInputRequest {
            wire,
            length,
            index,
            element,
            ..
        } = request;

        let field = PointerLenAbi::row_fields(element)
            .next()
            .unwrap()
            .name
            .clone();
        Ok(SequenceInput {
            controls: vec![
                Wire {
                    name: wire.clone(),
                    kind: syn::parse_quote!(*const i32),
                },
                Wire {
                    name: length.clone(),
                    kind: syn::parse_quote!(usize),
                },
            ],
            metadata: PacketChain::Sequence,
            open: quote!(if #length>0 && #wire.is_null(){return Err("null sequence".into());}),
            bind: quote!(let #field=unsafe{*#wire.add(#index)};),
            validation_values: vec![(
                if self.unknown_validation_read {
                    syn::parse_quote!(unknown)
                } else {
                    field.clone()
                },
                quote!(#field),
            )],
            release: quote!(),
        })
    }
    fn sequence_output(
        &self,
        request: SequenceOutputRequest<'_, Self>,
    ) -> Result<SequenceOutput<Type, PacketChain>> {
        let SequenceOutputRequest {
            wire,
            length,
            element,
            values,
            ..
        } = request;

        assert_eq!(
            PointerLenAbi::row_fields(element).len(),
            values.len(),
            "shared composition must validate child values before invoking a hook"
        );
        let values = prebindgen_conversions::match_output_fields(
            PointerLenAbi::row_fields(element),
            values,
        )?;
        let value = values[0].1;
        Ok(SequenceOutput {
            controls: vec![
                Wire {
                    name: wire.clone(),
                    kind: syn::parse_quote!(*const i32),
                },
                Wire {
                    name: length.clone(),
                    kind: syn::parse_quote!(usize),
                },
            ],
            metadata: PacketChain::Sequence,
            commit_each: false,
            fallible: false,
            begin: quote!(let mut #wire=Vec::<i32>::with_capacity(#length);),
            enter: quote!(),
            store: quote!(#wire.push(#value);),
            finish: quote!(let #wire=#wire.into_boxed_slice();),
            values: if self.bad_sequence {
                vec![(wire.clone(), quote!(#wire.as_ptr()))]
            } else {
                let mut bindings = vec![
                    (wire.clone(), quote!(#wire.as_ptr())),
                    (length.clone(), quote!(#wire.len())),
                ];
                if self.reverse_values {
                    bindings.reverse();
                }
                bindings
            },
            commit: quote!(let _=Box::into_raw(#wire);),
        })
    }
    fn finish_output(&self, plan: &mut CodecOutputPlan<Self>, _: &()) -> Result<()> {
        let values = PointerLenAbi::result_packet(&plan.outcomes.success, &plan.values)?;
        // The result packet contains presence, data and length; preparation owns
        // the buffer until these expressions have been saved for delivery.
        let flag = values[0].1;
        let data = values[1].1;
        let len = values[2].1;
        plan.delivery = prebindgen_conversions::Delivery {
            policy: ::prebindgen_conversions::DeliveryPolicy::Fixed,
            statements: quote!(let delivered=Returned{present:#flag,data:#data,len:#len};),
            result: quote!(delivered),
        };
        Ok(())
    }

    fn group(&self, kind: GroupKind<'_>) -> Result<Metadata> {
        Ok(match kind {
            GroupKind::Values | GroupKind::NamedOutput(_) => Metadata::Components,
            GroupKind::Fields => Metadata::Fields,
            GroupKind::Field(_) => Metadata::Field,
        })
    }
    fn optional(&self, request: Optional<'_, Self>) -> Result<OptionalEncoding<Self>> {
        let prepared = prepare_optional_payload(
            PointerLenAbi::values_mut(request.payload),
            request.direction,
            |w| self.niche(w),
            |w, n| {
                self.set_niche(w, n);
                Ok(())
            },
            |w, n, v| self.is_present(w, n, v),
            |w, n| self.remaining_niche(w, n),
            |w, n| self.absent_value(w, n, false),
        )?;
        let metadata = PacketChain::Optional;
        let presence = if prepared.reservation.is_some() {
            Presence::Niche {
                input_test: prepared.input_test,
            }
        } else {
            Presence::Flag(Wire {
                name: request.presence,
                kind: self.selector(request.extra)?,
            })
        };
        Ok(OptionalEncoding {
            presence,
            metadata,
            absent_values: prepared.absent_values,
        })
    }
    fn alternatives(&self, request: Alternatives<'_, Self>) -> Result<Metadata> {
        for arm in request.arms {
            retire_optional_niches(
                PointerLenAbi::values_mut(arm),
                None,
                |w, n| self.remaining_niche(w, n),
                |w, n| {
                    self.set_niche(w, n);
                    Ok(())
                },
            )?;
        }
        Ok(Metadata::Alternatives)
    }
}

impl CodecImpl {
    fn niche(&self, _: &Type) -> Option<()> {
        None
    }
    fn set_niche(&self, _: &mut Type, _: Option<()>) {}
    fn absent_value(&self, ty: &Type, _: Option<&()>, _: bool) -> Result<TokenStream> {
        Ok(if matches!(ty, Type::Ptr(_)) {
            quote!(::core::ptr::null())
        } else {
            quote!(Default::default())
        })
    }
    fn is_present(&self, _: &Type, _: &(), _: TokenStream) -> Result<TokenStream> {
        unreachable!()
    }
    fn remaining_niche(&self, _: &Type, _: &()) -> Option<()> {
        None
    }
}

#[test]
fn optional_sequences_with_two_outer_values_compile_and_execute() {
    for reverse in [false, true] {
        for row_only in [false, true] {
            pointer_length_round_trip(reverse, row_only);
        }
    }
}
fn pointer_length_round_trip(reverse: bool, row_only: bool) {
    use prebindgen_conversions::staged::emit_call;
    let source =
        syn::parse_file("pub fn echo(values:Option<Vec<i32>>)->Option<Vec<i32>>{} ").unwrap();
    let mut flat = Flat::<<CodecImpl as Codec>::Schema>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let defaults = TestDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let filled = defaults.fill(
        &mut flat,
        &CodecImpl {
            reverse_values: reverse,
            row_only_leaf: row_only,
            ..Default::default()
        },
        &test_fun!(echo),
        &(),
    );
    filled.unwrap();
    let f = flat.function("echo").unwrap();
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(
            f.input_conversions().unwrap()[0].input_node()
        ))
        .count(),
        4
    );
    let input = f.input_conversions().unwrap()[0];
    let output = f.output_conversion().unwrap();
    assert_eq!(PointerLenAbi::parameters(input.input_node()).len(), 3);
    assert_eq!(output.values.len(), 3);
    let cloned: CodecOutputPlan<CodecImpl> = output.clone();
    assert_eq!(cloned.values.len(), 3);
    let parameters = PointerLenAbi::parameters(&input.node)
        .map(|w| {
            let name = &w.name;
            let ty = w.rust_type();
            quote!(#name:#ty)
        })
        .collect::<Vec<_>>();
    let body = emit_call(
        f,
        &(),
        &syn::parse_quote!(result),
        &syn::parse_quote!(Returned),
        &PointerLenAbi::parameters(&input.node)
            .map(|w| w.name.clone())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let program = quote! {
        #[derive(Debug)]struct Returned{present:i32,data:*const i32,len:usize}
        fn echo(values:Option<Vec<i32>>)->Option<Vec<i32>>{values.map(|v|v.into_iter().map(|x|x+1).collect())}
        fn bridge(#(#parameters),*)->Result<Returned,String>{#body}
        fn main(){
            let empty=bridge(0,std::ptr::null(),usize::MAX).unwrap();assert_eq!(empty.present,0);assert!(empty.data.is_null());assert_eq!(empty.len,0);
            for values in [vec![],vec![1,-2,40]] {
                let r=bridge(1,values.as_ptr(),values.len()).unwrap();assert_eq!(r.present,1);
                let output=unsafe{Box::from_raw(std::ptr::slice_from_raw_parts_mut(r.data as *mut i32,r.len))};
                assert_eq!(&*output,values.iter().map(|x|x+1).collect::<Vec<_>>());
            }
            assert!(bridge(99,std::ptr::null(),0).is_err());
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("packet.rs");
    let exe = dir.path().join("packet");
    std::fs::write(&file, program.to_string()).unwrap();
    let built = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(std::process::Command::new(exe).status().unwrap().success());
}

#[test]
fn malformed_nested_sequence_bindings_are_rejected_before_installation() {
    let source =
        syn::parse_file("pub fn output()->Vec<Vec<i32>>{} pub fn input(values:Vec<i32>){}")
            .unwrap();
    let mut flat = Flat::<<CodecImpl as Codec>::Schema>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let bad = CodecImpl {
        bad_sequence: true,
        ..Default::default()
    };
    let recipe = TestOutputConfig::Unary {
        operation: TestOutputUnary::Sequence {
            delivery: SequenceDelivery::Value,
            mode: None,
        },
        child: Box::new(TestOutputConfig::Unary {
            operation: TestOutputUnary::Sequence {
                delivery: SequenceDelivery::Value,
                mode: None,
            },
            child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
        }),
    };
    let error = test_bind_output(&mut flat, &bad, "output", &recipe, &()).unwrap_err();
    assert!(error.contains("missing output binding"), "{error}");
    assert!(flat.function("output").unwrap().ret.slot.is_none());
    let bad = CodecImpl {
        unknown_validation_read: true,
        ..Default::default()
    };
    let error = test_bind_input(
        &mut flat,
        &bad,
        "input",
        "values",
        &TestInputConfig::Unary {
            operation: TestInputUnary::Sequence,
            child: Box::new(TestInputConfig::Leaf(TestInputLeafOp::Value)),
        },
        &(),
    )
    .unwrap_err();
    assert!(
        error.contains("unknown collection validation binding"),
        "{error}"
    );
    assert!(flat.function("input").unwrap().params[0].ty.slot.is_none());
}

#[test]
fn rows_lend_temporary_values_and_construct_once() {
    let source=syn::parse_file("pub struct Row{pub value:i32}pub fn row(value:&i32)->Row{}pub fn consume(values:Vec<Row>)->i32{}").unwrap();
    let flat = Flat::<<CodecImpl as Codec>::Schema>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let codec = CodecImpl::default();
    let recipe = TestInput::Unary {
        operation: TestInputUnary::Sequence,
        child: Box::new(TestInput::constructor(
            "row",
            [TestInput::Unary {
                operation: TestInputUnary::Temporary,
                child: Box::new(TestInput::Leaf(TestInputLeafOp::Value)),
            }],
        )),
    };
    let plan = compile_input(&flat, &codec, "consume", "values", &recipe, &()).unwrap();
    let target = syn::parse_quote!(values);
    let conversion = plan.bind(&target, quote!(Ok(consume(#target)))).unwrap();
    let parameters = PointerLenAbi::parameters(&plan.node)
        .map(|w| {
            let name = &w.name;
            let ty = w.rust_type();
            quote!(#name:#ty)
        })
        .collect::<Vec<_>>();
    let program = quote! {
        use std::sync::atomic::{AtomicUsize,Ordering};
        static CONSTRUCTED:AtomicUsize=AtomicUsize::new(0);
        struct Row{value:i32}
        fn row(value:&i32)->Row {
            CONSTRUCTED.fetch_add(1,Ordering::SeqCst);Row{value:*value}
        }
        fn consume(values:Vec<Row>)->i32{values.iter().map(|r|r.value).sum()}
        fn bridge(#(#parameters),*)->Result<i32,String>{
            let __prebindgen_input_consumed=std::cell::Cell::new(0usize);
            #conversion
        }
        fn main(){for values in [vec![],vec![1,2,3]] {
            CONSTRUCTED.store(0,Ordering::SeqCst);
            assert_eq!(bridge(values.as_ptr(),values.len()).unwrap(),values.iter().sum());
            assert_eq!(CONSTRUCTED.load(Ordering::SeqCst),values.len());
        }}
    };
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("temporary.rs");
    let exe = directory.path().join("temporary");
    std::fs::write(&file, program.to_string()).unwrap();
    let built = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(std::process::Command::new(exe).status().unwrap().success());
}
