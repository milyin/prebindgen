//! A backend whose sequence boundary has separate pointer and length values.
use super::*;
use prebindgen_conversions::{validate_wire_names, value};
use prebindgen_flat::Conversion;

#[derive(Clone, Debug)]
enum Metadata {
    Values(Vec<Wire<Type>>),
    Fields,
    Field,
}
// A test adapter using separate pointer and length parameters.
struct PointerLenAbi;
impl PointerLenAbi {
    fn parameters(
        layout: &[prebindgen_flat::WireNode<Type, Metadata>],
    ) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(layout)
    }
    fn row_fields(
        element: &[prebindgen_flat::WireNode<Type, Metadata>],
    ) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(element)
    }
    fn result_fields(
        layout: &[prebindgen_flat::WireNode<Type, Metadata>],
    ) -> std::vec::IntoIter<&Wire<Type>> {
        Self::values(match layout {
            [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
            nodes => nodes,
        })
    }
    fn result_packet<'a>(
        layout: &'a [prebindgen_flat::WireNode<Type, Metadata>],
        bindings: &'a [(syn::Ident, TokenStream)],
    ) -> Result<Vec<(&'a Wire<Type>, &'a TokenStream)>> {
        prebindgen_conversions::match_output_fields(Self::result_fields(layout), bindings)
    }
    fn values(nodes: &[WireNode<Type, Metadata>]) -> std::vec::IntoIter<&Wire<Type>> {
        fn visit<'a>(nodes: &'a [WireNode<Type, Metadata>], out: &mut Vec<&'a Wire<Type>>) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group {
                        metadata: Metadata::Values(ws),
                        ..
                    } => out.extend(ws),
                    WireNode::Group { children, .. } => visit(children.as_slice(), out),
                    WireNode::Optional(c) => {
                        if let Some(Metadata::Values(ws)) = &c.metadata {
                            out.extend(ws);
                        }
                        visit(c.element.as_slice(), out);
                    }
                    WireNode::Sequence(c) => {
                        if let Some(Metadata::Values(ws)) = &c.metadata {
                            out.extend(ws);
                        }
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(Metadata::Values(ws)) = metadata {
                            out.extend(ws);
                        }
                        for arm in alternatives {
                            visit(arm.as_slice(), out);
                        }
                    }
                    WireNode::Outcomes { success, .. } => visit(success, out),
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn values_mut(nodes: &mut [WireNode<Type, Metadata>]) -> std::vec::IntoIter<&mut Wire<Type>> {
        fn visit<'a>(nodes: &'a mut [WireNode<Type, Metadata>], out: &mut Vec<&'a mut Wire<Type>>) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group {
                        metadata: Metadata::Values(ws),
                        ..
                    } => out.extend(ws),
                    WireNode::Group { children, .. } => visit(children.as_mut_slice(), out),
                    WireNode::Optional(c) => {
                        if let Some(Metadata::Values(ws)) = &mut c.metadata {
                            out.extend(ws);
                        }
                        visit(c.element.as_mut_slice(), out);
                    }
                    WireNode::Sequence(c) => {
                        if let Some(Metadata::Values(ws)) = &mut c.metadata {
                            out.extend(ws);
                        }
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(Metadata::Values(ws)) = metadata {
                            out.extend(ws);
                        }
                        for arm in alternatives {
                            visit(arm.as_mut_slice(), out);
                        }
                    }
                    WireNode::Outcomes { success, .. } => {
                        visit(success.as_mut_slice(), out);
                    }
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn validate(layout: &[prebindgen_flat::WireNode<Type, Metadata>]) -> Result<()> {
        prebindgen_flat::validate_wire_nodes(layout.iter())?;
        if let [WireNode::Outcomes {
            success,
            source_error,
            binding_failure,
        }] = layout
        {
            for nodes in std::iter::once(success)
                .chain(source_error)
                .chain(binding_failure)
            {
                Self::validate(nodes)?;
            }
            return Ok(());
        }
        validate_wire_names(Self::values(layout))?;
        fn visit(nodes: &[WireNode<Type, Metadata>]) -> Result<()> {
            for node in nodes {
                match node {
                    WireNode::Sequence(c) => PointerLenAbi::validate(&c.element)?,
                    WireNode::Optional(c) => visit(c.element.as_slice())?,
                    WireNode::Group {
                        metadata: Metadata::Values(_),
                        children,
                    } => PointerLenAbi::validate(children)?,
                    WireNode::Group { children, .. } => visit(children.as_slice())?,
                    _ => {}
                }
            }
            Ok(())
        }
        visit(layout)
    }
}
#[derive(Default)]
struct CodecImpl {
    bad_sequence: bool,
    reverse_values: bool,
    unknown_prepared_read: bool,
}
impl Codec for CodecImpl {
    type Wire = Type;
    type Group = Metadata;
    fn optional_niche(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
    ) -> Option<NicheReservation<Self::Niche>> {
        select_optional_niche(self, PointerLenAbi::values(payload))
    }
    fn optional_presence(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
        reservation: &NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        optional_presence(self, PointerLenAbi::values(payload), reservation)
    }
    fn absent_output_values(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        absent_output_values(self, PointerLenAbi::values(payload), reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut Vec<prebindgen_flat::WireNode<Self::Wire, Self::Group>>,
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        retire_optional_niches(
            self,
            PointerLenAbi::values_mut(payload.as_mut_slice()),
            reservation,
        )
    }
    fn validate_input(&self, plan: &CodecInput<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.nodes)
    }
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        PointerLenAbi::validate(&plan.nodes)?;
        PointerLenAbi::result_packet(&plan.nodes, &plan.values)?;
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
        controls: Vec<Wire<Type>>,
    ) -> Metadata {
        Metadata::Values(controls)
    }
    fn alternatives_metadata(&self, selector: Wire<Type>) -> Metadata {
        Metadata::Values(vec![selector])
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
        element: &[prebindgen_flat::WireNode<Type, Metadata>],
        _: &Condition,
    ) -> Result<SequenceInput<Metadata>> {
        let field = PointerLenAbi::row_fields(element)
            .next()
            .unwrap()
            .name
            .clone();
        Ok(SequenceInput {
            metadata: Metadata::Values(vec![
                Wire {
                    name: wire.clone(),
                    kind: syn::parse_quote!(*const i32),
                },
                Wire {
                    name: length.clone(),
                    kind: syn::parse_quote!(usize),
                },
            ]),
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
        element: &[prebindgen_flat::WireNode<Type, Metadata>],
        values: &[(syn::Ident, TokenStream)],
    ) -> Result<SequenceOutput<Metadata>> {
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
            metadata: Metadata::Values(vec![
                Wire {
                    name: wire.clone(),
                    kind: syn::parse_quote!(*const i32),
                },
                Wire {
                    name: length.clone(),
                    kind: syn::parse_quote!(usize),
                },
            ]),
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
        let values = PointerLenAbi::result_packet(&plan.nodes, &plan.values)?;
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
    let defaults = Declarations::new(&flat, vec![]).unwrap();
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
        prebindgen_flat::wire_definitions(f.input_conversions().unwrap()[0].wire_nodes().iter())
            .count(),
        1
    );
    let input = f.input_conversions().unwrap()[0];
    let output = f.output_conversion().unwrap();
    assert_eq!(PointerLenAbi::parameters(input.wire_nodes()).len(), 3);
    assert_eq!(output.values.len(), 3);
    let cloned: CodecOutput<CodecImpl> = output.clone();
    assert_eq!(cloned.values.len(), 3);
    let parameters = PointerLenAbi::parameters(&input.nodes)
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
        &PointerLenAbi::parameters(&input.nodes)
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
        element: Box::new(Output::Sequence {
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
