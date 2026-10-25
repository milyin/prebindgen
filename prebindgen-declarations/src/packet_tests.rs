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
    unknown_prepared_read: bool,
}
impl Codec for CodecImpl {
    type Metadata = ();
    type Wire = Type;
    type Chain = PacketChain;
    type Branch = Metadata;
    fn group_metadata(&self) -> Metadata {
        Metadata::Components
    }
    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<NicheReservation<Self::Niche>> {
        select_optional_niche(self, PointerLenAbi::values(payload))
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        optional_presence(self, PointerLenAbi::values(payload), reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        absent_output_values(self, PointerLenAbi::values(payload), reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        retire_optional_niches(self, PointerLenAbi::values_mut(payload), reservation)
    }
    fn validate_input(&self, plan: &CodecInput<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.node)
    }
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.outcomes.success)?;
        PointerLenAbi::result_packet(&plan.outcomes.success, &plan.values)?;
        Ok(())
    }

    type Niche = ();
    type Context = ();
    type Extra = ();
    type Input = CodecInput<Self>;
    type Output = CodecOutput<Self>;
    fn optional_metadata(
        &self,
        _: syn::Ident,
        _: syn::Ident,
        _: Option<()>,
        _controls: &[Wire<Type>],
    ) -> Result<PacketChain> {
        Ok(PacketChain::Optional)
    }
    fn alternatives_metadata(&self, _selector: &Wire<Type>) -> Result<Metadata> {
        Ok(Metadata::Alternatives)
    }
    fn field_group(&self, _: syn::Ident) -> Metadata {
        Metadata::Field
    }
    fn fields_group(&self) -> Metadata {
        Metadata::Fields
    }
    fn selector(&self, _: &()) -> Type {
        syn::parse_quote!(i32)
    }
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
    fn consumes(&self, _: &Type, _: &()) -> bool {
        false
    }
    fn input_leaf(
        &self,
        ty: &Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<CodecInput<Self>> {
        value::input(ty, w.clone(), v, &active.rust(), quote!(#w), ty.clone())
    }
    fn output_leaf(
        &self,
        ty: &Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<Self>> {
        value::output(ty, w, v, borrowed, |v| v, ty.clone())
    }
    fn sequence_input(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &prebindgen_flat::WireNode<Type, PacketChain, Metadata>,
        _: &Condition,
    ) -> Result<SequenceInput<Type, PacketChain>> {
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
            validation_values: vec![(field.clone(), quote!(#field))],
            prepared_values: vec![(
                if self.unknown_prepared_read {
                    syn::parse_quote!(unknown)
                } else {
                    field.clone()
                },
                Some(quote!(#field)),
            )],
            release: quote!(),
        })
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        _: syn::Ident,
        element: &prebindgen_flat::WireNode<Type, PacketChain, Metadata>,
        values: &[(syn::Ident, TokenStream)],
        _delivery: crate::SequenceDelivery,
    ) -> Result<SequenceOutput<Type, PacketChain>> {
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
    fn finish_output(&self, plan: &mut CodecOutput<Self>, _: &()) -> Result<()> {
        let values = PointerLenAbi::result_packet(&plan.outcomes.success, &plan.values)?;
        // The result packet contains presence, data and length; preparation owns
        // the buffer until these expressions have been saved for delivery.
        let flag = values[0].1;
        let data = values[1].1;
        let len = values[2].1;
        plan.delivery = prebindgen_conversions::Delivery {
            statements: quote!(let delivered=Returned{present:#flag,data:#data,len:#len};),
            result: quote!(delivered),
        };
        Ok(())
    }
}

#[test]
fn optional_sequences_with_two_outer_values_compile_and_execute() {
    for reverse in [false, true] {
        pointer_length_round_trip(reverse);
    }
}
fn pointer_length_round_trip(reverse: bool) {
    use prebindgen_conversions::staged::emit_call;
    let source =
        syn::parse_file("pub fn echo(values:Option<Vec<i32>>)->Option<Vec<i32>>{} ").unwrap();
    let mut flat = Flat::<CodecInput<CodecImpl>, CodecOutput<CodecImpl>>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let defaults = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
    defaults
        .fill(
            &mut flat,
            &CodecImpl {
                reverse_values: reverse,
                ..Default::default()
            },
            &fun!(echo),
            &(),
        )
        .unwrap();
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
    let cloned: CodecOutput<CodecImpl> = output.clone();
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
    let mut flat = Flat::<CodecInput<CodecImpl>, CodecOutput<CodecImpl>>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let bad = CodecImpl {
        bad_sequence: true,
        ..Default::default()
    };
    let recipe = Output::Sequence {
        delivery: SequenceDelivery::Value,
        element: Box::new(Output::Sequence {
            delivery: SequenceDelivery::Value,
            element: Box::new(Output::Leaf),
            mode: None,
        }),
        mode: None,
    };
    let error = bind_output(&mut flat, &bad, "output", &recipe, &()).unwrap_err();
    assert!(error.contains("missing output binding"), "{error}");
    assert!(flat.function("output").unwrap().ret.slot.is_none());
    let bad = CodecImpl {
        unknown_prepared_read: true,
        ..Default::default()
    };
    let error = bind_input(
        &mut flat,
        &bad,
        "input",
        "values",
        &Input::Sequence(Box::new(Input::Leaf)),
        &(),
    )
    .unwrap_err();
    assert!(error.contains("prepared input names differ"), "{error}");
    assert!(flat.function("input").unwrap().params[0].ty.slot.is_none());
}
