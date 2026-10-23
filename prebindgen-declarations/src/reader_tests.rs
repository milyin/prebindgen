//! A backend whose sequence boundary has separate pointer and length values.
use super::*;
use prebindgen_conversions::{validate_wire_names, value, NativeReader};
use prebindgen_flat::{Conversion, OutcomeKind};

#[derive(Clone, Debug)]
enum Metadata {
    Values(Vec<Wire<Type>>),
    Fields,
    Field,
}
// Reader instances are never stored, cloned or formatted by a plan.
struct PointerLen;
impl NativeReader<Type, Metadata> for PointerLen {
    fn wires(nodes: &[WireNode<Type, Metadata>]) -> std::vec::IntoIter<&Wire<Type>> {
        fn visit<'a>(nodes: &'a [WireNode<Type, Metadata>], out: &mut Vec<&'a Wire<Type>>) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group {
                        metadata: Metadata::Values(ws),
                        ..
                    } => out.extend(ws),
                    WireNode::Group { children, .. } => visit(children.nodes(), out),
                    WireNode::Optional(c) => {
                        if let Some(Metadata::Values(ws)) = &c.metadata {
                            out.extend(ws);
                        }
                        visit(c.element.nodes(), out);
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
                            visit(arm.nodes(), out);
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        if let Some(o) =
                            alternatives.iter().find(|o| o.kind == OutcomeKind::Success)
                        {
                            visit(o.layout.nodes(), out);
                        }
                    }
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn wires_mut(nodes: &mut [WireNode<Type, Metadata>]) -> std::vec::IntoIter<&mut Wire<Type>> {
        fn visit<'a>(nodes: &'a mut [WireNode<Type, Metadata>], out: &mut Vec<&'a mut Wire<Type>>) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group {
                        metadata: Metadata::Values(ws),
                        ..
                    } => out.extend(ws),
                    WireNode::Group { children, .. } => visit(children.nodes_mut(), out),
                    WireNode::Optional(c) => {
                        if let Some(Metadata::Values(ws)) = &mut c.metadata {
                            out.extend(ws);
                        }
                        visit(c.element.nodes_mut(), out);
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
                            visit(arm.nodes_mut(), out);
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        if let Some(o) = alternatives
                            .iter_mut()
                            .find(|o| o.kind == OutcomeKind::Success)
                        {
                            visit(o.layout.nodes_mut(), out);
                        }
                    }
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn validate(layout: &WireLayout<Type, Metadata>) -> Result<()> {
        layout.validate()?;
        if let [WireNode::Outcomes { alternatives }] = layout.nodes() {
            for o in alternatives {
                Self::validate(&o.layout)?;
            }
            return Ok(());
        }
        validate_wire_names(Self::wires(layout.nodes()))?;
        fn visit(nodes: &[WireNode<Type, Metadata>]) -> Result<()> {
            for node in nodes {
                match node {
                    WireNode::Sequence(c) => PointerLen::validate(&c.element)?,
                    WireNode::Optional(c) => visit(c.element.nodes())?,
                    WireNode::Group {
                        metadata: Metadata::Values(_),
                        children,
                    } => PointerLen::validate(children)?,
                    WireNode::Group { children, .. } => visit(children.nodes())?,
                    _ => {}
                }
            }
            Ok(())
        }
        visit(layout.nodes())
    }
}
#[derive(Default)]
struct CodecImpl {
    bad_sequence: bool,
    packed_fields: bool,
}
impl Codec for CodecImpl {
    type Wire = Type;
    type Group = Metadata;
    type Reader = PointerLen;
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
        if self.packed_fields {
            return Metadata::Values(vec![]);
        }
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
        element: &WireLayout<Type, Metadata>,
        _: &Condition,
    ) -> Result<SequenceInput<Metadata>> {
        let field = PointerLen::wires(element.nodes())
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
            validation_values: vec![quote!(#field)],
            prepared_values: vec![Some(quote!(#field))],
            release: quote!(),
        })
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        _: syn::Ident,
        element: &WireLayout<Type, Metadata>,
        values: &[TokenStream],
    ) -> Result<SequenceOutput<Metadata>> {
        assert_eq!(
            PointerLen::wires(element.nodes()).len(),
            values.len(),
            "shared composition must validate child values before invoking a hook"
        );
        let value = &values[0];
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
                vec![quote!(#wire.as_ptr())]
            } else {
                vec![quote!(#wire.as_ptr()), quote!(#wire.len())]
            },
            commit: quote!(let _=Box::into_raw(#wire);),
        })
    }
    fn finish_output(&self, plan: &mut CodecOutput<Self>, _: &()) -> Result<()> {
        let values = &plan.values;
        // The native reader exposes presence, data and length; preparation owns
        // the buffer until these expressions have been saved for delivery.
        let flag = &values[0];
        let data = &values[1];
        let len = &values[2];
        plan.delivery = prebindgen_conversions::Delivery {
            statements: quote!(let delivered=Returned{present:#flag,data:#data,len:#len};),
            result: quote!(delivered),
        };
        Ok(())
    }
}

#[test]
fn optional_sequences_with_two_outer_values_compile_and_execute() {
    use prebindgen_conversions::{staged::emit_call, NativeFunction};
    let source =
        syn::parse_file("pub fn echo(values:Option<Vec<i32>>)->Option<Vec<i32>>{} ").unwrap();
    let mut flat = Flat::<CodecInput<CodecImpl>, CodecOutput<CodecImpl>>::parse(
        source.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let defaults = Declarations::new(&flat, vec![]).unwrap();
    defaults
        .fill(&mut flat, &CodecImpl::default(), &fun!(echo), &())
        .unwrap();
    let f = flat.function("echo").unwrap();
    assert_eq!(f.input_conversions().unwrap()[0].layout().len(), 1);
    let input = f.input_conversions().unwrap()[0];
    let output = f.output_conversion().unwrap();
    assert_eq!(PointerLen::wires(input.layout().nodes()).len(), 3);
    assert_eq!(output.values.len(), 3);
    let cloned: CodecOutput<CodecImpl> = output.clone(); // PointerLen has no Clone implementation.
    assert_eq!(cloned.values.len(), 3);
    let parameters = f
        .native_inputs()
        .unwrap()
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
    let file = dir.path().join("reader.rs");
    let exe = dir.path().join("reader");
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
fn a_reader_can_pack_a_group_without_erasing_its_logical_fields() {
    let wire = |name: &str, ty: Type| Wire {
        name: syn::parse_str(name).unwrap(),
        kind: ty,
    };
    let mut layout: WireLayout<Type, Metadata> = vec![
        wire("first", syn::parse_quote!(i32)),
        wire("second", syn::parse_quote!(i32)),
    ]
    .into();
    layout.enclose(Metadata::Values(vec![
        wire("packed", syn::parse_quote!(*const i32)),
        wire("size", syn::parse_quote!(usize)),
    ]));
    PointerLen::validate(&layout).unwrap();
    assert_eq!(layout.len(), 2);
    assert_eq!(
        PointerLen::wires(layout.nodes())
            .map(|w| w.name.to_string())
            .collect::<Vec<_>>(),
        ["packed", "size"]
    );
    PointerLen::wires_mut(layout.nodes_mut())
        .next()
        .unwrap()
        .name = syn::parse_quote!(address);
    assert_eq!(
        PointerLen::wires(layout.nodes()).next().unwrap().name,
        "address"
    );
    assert_eq!(layout[0].name, "first");
}

#[test]
fn reader_selection_changes_the_view_without_changing_metadata() {
    struct FieldReader;
    impl NativeReader<Type, Metadata> for FieldReader {
        fn wires(nodes: &[WireNode<Type, Metadata>]) -> std::vec::IntoIter<&Wire<Type>> {
            let mut out = vec![];
            for node in nodes {
                match node {
                    WireNode::Group { children, .. } => out.extend(Self::wires(children.nodes())),
                    _ => out.extend(PointerLen::wires(std::slice::from_ref(node))),
                }
            }
            out.into_iter()
        }
        fn wires_mut(
            nodes: &mut [WireNode<Type, Metadata>],
        ) -> std::vec::IntoIter<&mut Wire<Type>> {
            let mut out = vec![];
            for node in nodes {
                match node {
                    WireNode::Group { children, .. } => {
                        out.extend(Self::wires_mut(children.nodes_mut()))
                    }
                    _ => out.extend(PointerLen::wires_mut(std::slice::from_mut(node))),
                }
            }
            out.into_iter()
        }
        fn validate(layout: &WireLayout<Type, Metadata>) -> Result<()> {
            layout.validate()?;
            validate_wire_names(Self::wires(layout.nodes()))
        }
    }
    let wire = |name: &str, kind: Type| Wire {
        name: syn::parse_str(name).unwrap(),
        kind,
    };
    let mut layout: WireLayout<Type, Metadata> = vec![
        wire("first", syn::parse_quote!(i32)),
        wire("second", syn::parse_quote!(i32)),
    ]
    .into();
    layout.enclose(Metadata::Values(vec![
        wire("pointer", syn::parse_quote!(*const i32)),
        wire("length", syn::parse_quote!(usize)),
    ]));
    PointerLen::validate(&layout).unwrap();
    FieldReader::validate(&layout).unwrap();
    assert_eq!(
        PointerLen::wires(layout.nodes())
            .map(|w| w.name.to_string())
            .collect::<Vec<_>>(),
        ["pointer", "length"]
    );
    assert_eq!(
        FieldReader::wires(layout.nodes())
            .map(|w| w.name.to_string())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(std::ptr::eq(
        FieldReader::wires(layout.nodes()).next().unwrap(),
        layout.get(0).unwrap()
    ));
    FieldReader::wires_mut(layout.nodes_mut())
        .next()
        .unwrap()
        .name = syn::parse_quote!(renamed_field);
    assert_eq!(layout.get(0).unwrap().name, "renamed_field");
    assert_eq!(
        PointerLen::wires(layout.nodes()).next().unwrap().name,
        "pointer"
    );
    let WireNode::Group {
        metadata: Metadata::Values(wires),
        ..
    } = &mut layout.nodes_mut()[0]
    else {
        panic!()
    };
    wires.push(wire("extra", syn::parse_quote!(usize)));
    assert_eq!(PointerLen::wires(layout.nodes()).len(), 3);
    assert_eq!(FieldReader::wires(layout.nodes()).len(), 2);
    assert_eq!(layout.len(), 2);
}

#[test]
fn malformed_sequence_counts_and_packed_record_hooks_are_rejected() {
    let mut source = syn::parse_file("pub struct Record { pub x:i32 } pub fn sequence()->Vec<Vec<i32>>{} pub fn record(x:Record)->Record{}").unwrap();
    let mut flat = Flat::<CodecInput<CodecImpl>, CodecOutput<CodecImpl>>::parse(
        source.items.drain(..).map(|i| (i, Default::default())),
    )
    .unwrap();
    let bad_sequence = CodecImpl {
        bad_sequence: true,
        ..Default::default()
    };
    let error = bind_output(
        &mut flat,
        &bad_sequence,
        "sequence",
        &Output::Sequence {
            element: Box::new(Output::Sequence {
                element: Box::new(Output::Leaf),
                mode: None,
            }),
            mode: None,
        },
        &(),
    )
    .unwrap_err();
    assert!(error.contains("wire/value count"), "{error}");
    assert!(flat.function("sequence").unwrap().ret.slot.is_none());
    let packed_fields = CodecImpl {
        packed_fields: true,
        ..Default::default()
    };
    let error = bind_input(
        &mut flat,
        &packed_fields,
        "record",
        "x",
        &Input::fields([Input::Leaf]),
        &(),
    )
    .unwrap_err();
    assert!(
        error.contains("retain the prepared boundary values"),
        "{error}"
    );
    assert!(flat.function("record").unwrap().params[0].ty.slot.is_none());
    for recipe in [
        Output::fields([Output::Leaf]),
        Output::OwnedFields(vec![Output::Leaf]),
    ] {
        let error = bind_output(&mut flat, &packed_fields, "record", &recipe, &()).unwrap_err();
        assert!(
            error.contains("retain the prepared boundary values"),
            "{error}"
        );
        assert!(flat.function("record").unwrap().ret.slot.is_none());
    }
}
