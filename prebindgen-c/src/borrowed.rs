//! Non-owning numeric collection views, with no per-element output conversion.
use super::*;
use crate::operations::{CInputLeafOp, COutput, COutputLeafOp};
use prebindgen_declarations::SequenceDelivery;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn compatible(element: Primitive) -> bool {
    !matches!(
        element,
        Primitive::Bool | Primitive::Char | Primitive::I128 | Primitive::U128
    )
}
/// A primitive whose configured C representation is still the native scalar.
pub(crate) fn primitive(
    ty: &syn::Type,
    types: &BTreeMap<String, prebindgen_flat::TypeInfo<CType>>,
) -> Option<Primitive> {
    let primitive = Primitive::from_type(ty).filter(|p| compatible(*p))?;
    if types.values().any(|info| {
        Primitive::from_type(&info.ty) == Some(primitive)
            && (info.metadata.pointer.is_some()
                || info.metadata.owned.is_some()
                || info.metadata.enumeration.is_some())
    }) {
        return None;
    }
    Some(primitive)
}
pub(crate) fn native_element_type(element: Primitive) -> syn::Type {
    let name = format_ident!("{}", prebindgen_flat::type_key(&element.rust_type()));
    syn::parse_quote!(::core::primitive::#name)
}
pub(crate) fn ident(element: Primitive) -> syn::Ident {
    format_ident!(
        "__PbSliceView_{}",
        prebindgen_flat::type_key(&element.rust_type())
    )
}
pub(crate) fn c_name(element: Primitive, prefix: &str) -> String {
    format!(
        "{prefix}_{}_slice_view",
        prebindgen_flat::type_key(&element.rust_type())
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn sequence(
    flat: &CBindings,
    ty: &syn::Type,
    wire: syn::Ident,
    value: TokenStream,
    recipe: &COutput,
    delivery: SequenceDelivery,
    temporary: bool,
) -> Result<Option<COutputPlan<CWire>>> {
    // An explicitly requested visitor does not allocate row storage. Its leaves
    // still enforce their own ownership/copy policies.
    if delivery == SequenceDelivery::Callback {
        return Ok(None);
    }
    if temporary
        && !matches!(ty, syn::Type::Reference(r) if r.lifetime.as_ref().is_some_and(|l|l.ident=="static"))
    {
        return Err("borrowed C collection view cannot escape a temporary owner; return its owner alongside a generated heap view getter or explicitly permit a collection copy".into());
    }
    let (element_type, view) =
        prebindgen_flat::sequence_parts(ty).ok_or("borrowed sequence requires a collection")?;
    if view == prebindgen_flat::SequenceView::Mutable {
        return Err("mutable C collection views are unsupported".into());
    }
    let primitive = primitive(element_type, &flat.types);
    let Some(element) = primitive.filter(|_| matches!(recipe, COutput::Leaf(COutputLeafOp::Value)))
    else {
        return Err(format!("borrowed C collection `{}` cannot allocate converted rows implicitly; use an address-preserving numeric element representation or explicitly permit a collection-level copy with expand_return!(&[T]).field_self_ref() (not an element-only expansion)", prebindgen_flat::type_key(ty)));
    };
    let element_type = native_element_type(element);
    let carrier = ident(element);
    let source = format_ident!("{wire}_source");
    Ok(Some(COutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        commit: TokenStream::new(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: CWire::SliceView { element },
            }]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(
            let #source = &(#value);
            let #wire: &[#element_type] = ::core::convert::AsRef::<[#element_type]>::as_ref(#source);
            let #wire = #carrier { data: <[#element_type]>::as_ptr(#wire), len: <[#element_type]>::len(#wire) };
        ),
        values: vec![(wire.clone(), quote!(#wire))],
    }))
}
/// A directly borrowed foreign slice, without reconstructing a Rust Vec owner.
pub(crate) fn input_sequence(
    flat: &CBindings,
    ty: &syn::Type,
    wire: syn::Ident,
    var: syn::Ident,
    recipe: &crate::operations::CInput,
    active: &prebindgen_declarations::Condition,
) -> Result<CInputPlan<CWire>> {
    let syn::Type::Reference(reference) = ty else {
        return Err("borrowed C input requires a reference".into());
    };
    if reference.mutability.is_some() {
        return Err("mutable C collection inputs are unsupported".into());
    }
    if reference
        .lifetime
        .as_ref()
        .is_some_and(|l| l.ident == "static")
    {
        return Err("shared C collection inputs cannot promise a 'static lifetime".into());
    }
    let target = &reference.elem;
    let (source_element, array_length) = match target.as_ref() {
        syn::Type::Slice(slice) => (slice.elem.as_ref(), None),
        syn::Type::Array(array) => (array.elem.as_ref(), Some(&array.len)),
        _ => return Err(format!("shared C input `{}` cannot reconstruct an owning collection implicitly; declare a native opaque owner input or explicitly select expand_param!({}).reconstruct()",prebindgen_flat::type_key(ty),prebindgen_flat::type_key(ty))),
    };
    let Some(element) = primitive(source_element, &flat.types)
        .filter(|_| matches!(recipe, crate::operations::CInput::Leaf(CInputLeafOp::Value)))
    else {
        return Err(format!("shared C input `{}` cannot allocate converted elements implicitly; use a compatible numeric slice, a native opaque owner, or explicitly select expand_param!({}).reconstruct()",prebindgen_flat::type_key(ty),prebindgen_flat::type_key(ty)));
    };
    let native = native_element_type(element);
    let condition = active.rust();
    let pointer = format_ident!("{var}_pointer");
    let slice = format_ident!("{var}_slice");
    let argument = if array_length.is_some() {
        quote!(unsafe {&*(#slice.as_ptr() as *const [#native;#array_length])})
    } else {
        quote!(#slice)
    };
    Ok(CInputPlan::new(
        prebindgen_flat::WireNode::Leaf(vec![Wire {
            name: wire.clone(),
            kind: CWire::SliceView { element },
        }]),
        quote!(let #var=if #condition {
        let #pointer=if #wire.len==0 && #wire.data.is_null(){::core::ptr::NonNull::<#native>::dangling().as_ptr() as *const #native}else{#wire.data};
        let #slice=unsafe {::core::slice::from_raw_parts(#pointer,#wire.len)};Some(#argument)
    } else {None};),
        quote!(#var.expect("active slice view")),
    ))
}
/// A loan anywhere in a packet, including fields inside converted rows.
pub(crate) fn has_loans(node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>) -> bool {
    prebindgen_flat::wire_definitions(std::iter::once(node)).any(|wire| {
        matches!(
            wire.kind,
            CWire::View
                | CWire::MutableView
                | CWire::SliceView { .. }
                | CWire::Owned {
                    access: Access::Shared | Access::Exclusive,
                    ..
                }
        ) || matches!(&wire.kind, CWire::Pointer {pointer,..} if pointer.access != Access::Move)
    })
}
pub(crate) fn collect(
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    out: &mut BTreeMap<String, Primitive>,
) {
    for wire in prebindgen_flat::wire_definitions(std::iter::once(node)) {
        match &wire.kind {
            CWire::SliceView { element } => {
                out.insert(prebindgen_flat::type_key(&element.rust_type()), *element);
            }
            CWire::Callback { callback } => callback.slice_views(out),
            _ => {}
        }
    }
}
pub(crate) fn emit(
    element: Primitive,
    prefix: &str,
    symbols: &mut BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    let name = c_name(element, prefix);
    if !symbols.insert(name.clone()) {
        return Err("generated C slice-view type collides with another C symbol".into());
    }
    let rust = ident(element);
    let ty = native_element_type(element);
    tokens.extend(quote!(#[allow(non_camel_case_types)] #[repr(C)] #[derive(Clone,Copy)] pub struct #rust {pub data: *const #ty,pub len: usize}));
    let c = c_scalar(element, prefix);
    header.push_str(&format!("/* Shared borrowed slice; never free or consume. Keep source storage alive; do not mutate it or obtain overlapping exclusive/mutable access until the view's last use. Empty views may have a non-null dangling data pointer; never dereference when len is zero. Callback argument views last only for that callback invocation. */\ntypedef struct {{ const {c} *data; size_t len; }} {name};\n"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expand_return, fun};
    use prebindgen_declarations::ProjectionMode;
    fn model(source: &str) -> CBindings {
        CBindings::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    fn sequence(
        copy: bool,
        element: crate::COutput<CInputRequest, COutputRequest>,
    ) -> crate::COutput<CInputRequest, COutputRequest> {
        crate::COutput::Unary {
            operation: (COutputUnary::Sequence {
                delivery: SequenceDelivery::Value,
                mode: copy.then_some(ProjectionMode::Borrowed),
            })
            .into(),
            child: Box::new(element),
        }
    }
    #[test]
    fn numeric_slice_array_optional_and_result_outputs_are_views() {
        for element in [
            "i8", "i16", "i32", "i64", "isize", "u16", "u32", "u64", "usize", "f32", "f64",
        ] {
            let source=format!("pub struct Owner{{pub n:i64}}pub fn slice(v:&Owner)->&[{element}]{{}}pub fn array(v:&Owner)->&[{element};3]{{}}pub fn maybe(v:&Owner)->Option<&[{element}]>{{}}pub fn checked(v:&Owner)->Result<&[{element}],String>{{}}pub fn message(e:&String)->String{{}}");
            let g = Cbindgen::new(model(&source))
                .opaque_ptr("owner", syn::parse_quote!(Owner))
                .opaque_error(syn::parse_quote!(String), fun!(message))
                .function(fun!(slice))
                .panic()
                .function(fun!(array))
                .panic()
                .function(fun!(maybe))
                .panic()
                .function(fun!(checked))
                .build()
                .unwrap();
            assert!(
                g.header.contains(&format!("pb_{element}_slice_view")),
                "{element}"
            );
            assert!(!g.rust.contains("OutputRows"));
            assert!(!g.header.contains("rows_free"));
            assert!(g.header.contains("int32_t pb_maybe("));
        }
    }
    #[test]
    fn unsupported_borrowed_collections_require_explicit_copy_permission() {
        for element in [
            "bool", "char", "i128", "u128", "Tag", "Row", "String", "Vec<i64>",
        ] {
            let source=format!("pub enum Tag{{A,B}}pub struct Row{{pub n:i64}}pub fn values()->&'static [{element}]{{}}");
            let error = Cbindgen::new(model(&source))
                .enum_type("tag", syn::parse_quote!(Tag))
                .function(fun!(values))
                .build()
                .err()
                .unwrap();
            assert!(
                error.contains("cannot allocate converted rows implicitly"),
                "{element}: {error}"
            );
        }
        let g = Cbindgen::new(model("pub fn values()->&'static [bool]{}"))
            .function(fun!(values).output(sequence(true, crate::COutput::auto())))
            .build()
            .unwrap();
        assert!(g.rust.contains("OutputRows"));
        assert!(g.header.contains("rows_free"));
        // Copying a leaf never implicitly authorizes an owning outer container.
        let error = Cbindgen::new(model("pub fn values()->&'static [i64]{}"))
            .function(
                fun!(values).output(sequence(false, expand_return!(i64).field_self_ref().into())),
            )
            .build()
            .err()
            .unwrap();
        assert!(error.contains("cannot allocate converted rows implicitly"));
    }
    #[test]
    fn getter_and_record_observations_cannot_lend_temporary_storage() {
        let source="pub struct Owner{pub values:Vec<i64>}pub fn make()->Owner{}pub fn values(v:&Owner)->&[i64]{}";
        let error = Cbindgen::new(model(source))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .function(fun!(make).expand_return(expand_return!(Owner).field(fun!(values))))
            .build()
            .err()
            .unwrap();
        assert!(error.contains("temporary owner"), "{error}");
        let g = Cbindgen::new(model(source))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .function(
                fun!(make)
                    .expand_return(expand_return!(Owner).field_self_into().field(fun!(values))),
            )
            .build()
            .unwrap();
        assert!(g.header.contains("pb_i64_slice_view *out1"));
        let error = Cbindgen::new(model(source))
            .function(fun!(make).output(crate::COutput::Branch {
                operation: (COutputBranch::Fields).into(),
                children: vec![sequence(false, crate::COutput::auto())],
            }))
            .build()
            .err()
            .unwrap();
        assert!(error.contains("temporary owner"), "{error}");
    }
    #[test]
    fn explicitly_reconstructed_reference_inputs_cannot_lend_views() {
        let error = Cbindgen::new(model("pub fn echo(v:&[i64])->&[i64]{}"))
            .function(fun!(echo).expand_param("v", crate::expand_param!(&[i64]).reconstruct()))
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reconstructed reference input"), "{error}");
        let g = Cbindgen::new(model("pub fn echo(v:&[i64])->&[i64]{}"))
            .function(fun!(echo).output(sequence(true, crate::COutput::auto())))
            .panic()
            .build()
            .unwrap();
        assert!(g.rust.contains("OutputRows"));
    }
    #[test]
    fn explicit_outer_copy_copies_nested_sequences_and_never_emits_views() {
        let g = Cbindgen::new(model("pub fn values()->&'static [Vec<i64>]{}"))
            .function(fun!(values).output(sequence(true, sequence(false, crate::COutput::auto()))))
            .build()
            .unwrap();
        assert!(g.rust.matches("OutputRows").count() >= 2);
        let output = &g.rust[g.rust.find("fn pb_values(").unwrap()..];
        assert!(!output.contains("__PbSliceView"));
    }
    #[test]
    fn numeric_inline_vec_getters_lend_only_heap_storage() {
        let g = Cbindgen::new(model("pub fn make()->Vec<i64>{}"))
            .opaque_owned_struct(
                "numbers",
                syn::parse_quote!(Vec<i64>),
                OpaqueLayout::auto::<Vec<i64>>(),
            )
            .function(
                fun!(make).expand_return(
                    expand_return!(Vec<i64>)
                        .field_self_into()
                        .field(Cbindgen::owned_view("numbers")),
                ),
            )
            .build()
            .unwrap();
        assert!(g.header.contains("pb_i64_slice_view pb_numbers_view("));
        assert!(g.header.contains("pb_i64_slice_view *out1"));
        assert!(!g.rust.contains("OutputRows"));
    }
    #[test]
    fn qualified_byte_vectors_use_the_same_helper_and_typedef_predicate() {
        let g = Cbindgen::new(model("pub fn make()->Vec<::core::primitive::u8>{}"))
            .function(fun!(make))
            .build()
            .unwrap();
        assert!(g.rust.contains("__PbSliceView_u8"));
        assert!(g.header.contains("pb_u8_slice_view"));
        assert!(g.rust.contains("fn __pb_owned_"));
    }
    #[test]
    fn low_level_borrowed_buffers_never_implicitly_clone() {
        let flat = model("");
        for ty in [syn::parse_quote!(String), syn::parse_quote!(Vec<u8>)] {
            let error = CCodec::raw()
                .output_leaf(prebindgen_declarations::OutputLeaf::new(
                    &flat,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    prebindgen_declarations::ValueAccess::Shared,
                ))
                .err()
                .unwrap();
            assert!(error.contains("explicit owning copy"));
            let copied = CCodec::raw()
                .output_projected(
                    prebindgen_declarations::OutputLeaf::new(
                        &flat,
                        &ty,
                        &(),
                        syn::parse_quote!(wire),
                        quote!(value),
                        prebindgen_declarations::ValueAccess::Shared,
                    ),
                    &prebindgen_declarations::Projection::borrowed(),
                )
                .unwrap();
            assert!(copied.statements.to_string().contains("Clone :: clone"));
        }
        for ty in [syn::parse_quote!(&str), syn::parse_quote!(&[u8])] {
            let view = CCodec::raw()
                .output_leaf(prebindgen_declarations::OutputLeaf::new(
                    &flat,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    prebindgen_declarations::ValueAccess::Owned,
                ))
                .unwrap();
            assert!(
                matches!(view.outcomes.success,prebindgen_flat::WireNode::Leaf(ref w) if matches!(w[0].kind,CWire::View))
            );
            let copied = CCodec::raw()
                .output_projected(
                    prebindgen_declarations::OutputLeaf::new(
                        &flat,
                        &ty,
                        &(),
                        syn::parse_quote!(wire),
                        quote!(value),
                        prebindgen_declarations::ValueAccess::Shared,
                    ),
                    &prebindgen_declarations::Projection::borrowed(),
                )
                .unwrap();
            assert!(copied
                .statements
                .to_string()
                .contains("ToOwned :: to_owned"));
            assert!(
                matches!(copied.outcomes.success,prebindgen_flat::WireNode::Leaf(ref w) if matches!(w[0].kind,CWire::Buffer))
            );
        }
    }
    #[test]
    fn a_callback_only_primitive_view_has_its_own_typedef_and_no_destructor() {
        let g = Cbindgen::new(model("pub fn emit(callback:impl Fn(&[f32])){}"))
            .function(fun!(emit))
            .panic()
            .build()
            .unwrap();
        assert!(g.header.contains("pb_f32_slice_view arg0"));
        assert_eq!(g.header.matches("} pb_f32_slice_view;").count(), 1);
        assert!(!g.header.contains("rows_free"));
        assert!(!g.rust.contains("OutputRows"));
    }
    #[test]
    fn inline_array_getter_cannot_use_the_vec_heap_view_exception() {
        let error=Cbindgen::new(model("pub struct Owner{pub values:[i64;3]}pub fn make()->Owner{}pub fn values(v:&Owner)->&[i64;3]{}"))
            .opaque_owned_struct("owner",syn::parse_quote!(Owner),OpaqueLayout::auto::<[i64;3]>())
            .function(fun!(make).expand_return(expand_return!(Owner).field_self_into().field(fun!(values))))
            .build().err().unwrap();
        assert!(error.contains("temporary owner"), "{error}");
    }
    #[test]
    fn source_getter_cannot_impersonate_a_generated_heap_view() {
        let source = "pub fn make()->[i64;3]{}pub fn __pb_owned_values_view(v:&[i64;3])->&[i64]{}";
        let error = Cbindgen::new(model(source))
            .opaque_owned_struct(
                "values",
                syn::parse_quote!([i64; 3]),
                OpaqueLayout::auto::<[i64; 3]>(),
            )
            .function(
                fun!(make).expand_return(
                    expand_return!([i64; 3])
                        .field_self_into()
                        .field(fun!(__pb_owned_values_view)),
                ),
            )
            .build()
            .err()
            .unwrap();
        assert!(error.contains("temporary owner"), "{error}");
    }
    #[test]
    fn loans_inside_rows_are_not_hidden_by_the_owning_container() {
        let node = prebindgen_flat::WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            child: Box::new(prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: syn::parse_quote!(view),
                kind: CWire::SliceView {
                    element: Primitive::I64,
                },
            }])),
            meta: CChain::Sequence(CSequence::Buffered),
        };
        assert!(has_loans(&node));
    }
    #[test]
    fn primitive_handle_overrides_cannot_be_bypassed_by_a_view() {
        let error = Cbindgen::new(model("pub fn values()->&'static [i64]{}"))
            .opaque_ptr("number", syn::parse_quote!(i64))
            .function(fun!(values))
            .build()
            .err()
            .unwrap();
        assert!(
            error.contains("cannot allocate converted rows implicitly"),
            "{error}"
        );
    }
    #[test]
    fn qualified_primitive_override_cannot_be_bypassed_by_an_unqualified_view() {
        let error = Cbindgen::new(model("pub fn values()->&'static [i64]{}"))
            .opaque_ptr("number", syn::parse_quote!(::core::primitive::i64))
            .function(fun!(values))
            .build()
            .err()
            .unwrap();
        assert!(
            error.contains("cannot allocate converted rows implicitly"),
            "{error}"
        );
    }
    #[test]
    fn error_mappers_cannot_return_borrowed_values() {
        for result in ["&[i64]", "&str", "&'static [i64]", "&'static str"] {
            let source = format!("pub struct Error{{pub code:i64}}pub fn message(e:&Error)->{result}{{}}pub fn run()->Result<(),Error>{{}}");
            let error = Cbindgen::new(model(&source))
                .opaque_error(syn::parse_quote!(Error), fun!(message))
                .function(fun!(run))
                .build()
                .err()
                .unwrap();
            assert!(error.contains("return String"), "{result}: {error}");
        }
    }
    #[test]
    fn numeric_slice_and_array_inputs_borrow_without_a_collection_decoder() {
        for element in [
            "i8", "i16", "i32", "i64", "isize", "u16", "u32", "u64", "usize", "f32", "f64",
        ] {
            let source=format!("pub fn inspect(values:&[{element}])->usize{{}}pub fn fixed(values:&[{element};3])->usize{{}}pub fn maybe(values:Option<&[{element}]>)->usize{{}}pub fn echo(values:&[{element}])->&[{element}]{{}}");
            let g = Cbindgen::new(model(&source))
                .function(fun!(inspect))
                .panic()
                .function(fun!(fixed))
                .panic()
                .function(fun!(maybe))
                .panic()
                .function(fun!(echo))
                .panic()
                .build()
                .unwrap();
            assert!(
                g.header.contains(&format!("pb_{element}_slice_view")),
                "{element}"
            );
            assert!(!g.rust.contains("validate_slice"));
            assert!(!g.rust.contains("reserve_exact"));
            assert!(!g.header.contains("_rows"));
        }
    }
    #[test]
    fn incompatible_borrowed_inputs_reject_implicit_reconstruction() {
        for ty in [
            "&[bool]",
            "&[char]",
            "&[i128]",
            "&[u128]",
            "&[Row]",
            "&[Tag]",
            "&[String]",
            "&[Vec<i64>]",
            "&Vec<i64>",
            "&String",
            "&Vec<u8>",
        ] {
            let source=format!("pub struct Row{{pub n:i64}}pub enum Tag{{A,B}}pub fn inspect(values:{ty})->usize{{}}");
            let error = Cbindgen::new(model(&source))
                .enum_type("tag", syn::parse_quote!(Tag))
                .function(fun!(inspect))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(error.contains("reconstruct()"), "{ty}: {error}");
        }
    }
    #[test]
    fn shared_input_primitive_overrides_cannot_be_reinterpreted() {
        let error = Cbindgen::new(model("pub fn inspect(values:&[i64])->usize{}"))
            .opaque_ptr("number", syn::parse_quote!(::core::primitive::i64))
            .function(fun!(inspect))
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reconstruct()"), "{error}");
        let error = Cbindgen::new(model(
            "pub fn make(v:i64)->i64{}pub fn inspect(values:&[i64])->usize{}",
        ))
        .function(
            fun!(inspect).expand_param("values", crate::expand_param!(i64).variant(fun!(make))),
        )
        .panic()
        .build()
        .err()
        .unwrap();
        assert!(error.contains("reconstruct()"), "{error}");
    }
    #[test]
    fn reconstruction_is_explicit_and_can_be_global_local_optional_or_nested() {
        use crate::{expand_param, CInput};
        for ty in [
            "&[i64]",
            "&Vec<i64>",
            "&String",
            "&Vec<u8>",
            "&str",
            "&[u8]",
            "&[i64;3]",
        ] {
            let source = format!("pub fn inspect(values:{ty})->usize{{}}");
            let declaration = crate::expand_param(syn::parse_str(ty).unwrap()).reconstruct();
            let g = Cbindgen::new(model(&source))
                .function(fun!(inspect).expand_param("values", declaration))
                .panic()
                .build()
                .unwrap();
            assert!(g.rust.contains("active temporary"), "{ty}");
        }
        let g=Cbindgen::new(model("pub struct Row{pub n:usize}pub fn new(values:Option<&String>)->Row{}pub fn inspect(values:Vec<Row>)->usize{}"))
            .expand(expand_param!(&String).reconstruct())
            .function(fun!(inspect).expand_param("values",expand_param!(Row).variant(fun!(new))))
            .panic().build().unwrap();
        assert!(g.rust.contains("active temporary"));
        assert!(!g.rust.contains("_prepared"));
        assert!(
            Cbindgen::new(model("pub fn inspect(values:&String)->usize{}"))
                .function(fun!(inspect).input("values", CInput::auto().reconstruct()))
                .panic()
                .build()
                .is_ok()
        );
    }
    #[test]
    fn reconstruction_builder_order_and_local_identity_overrides_are_checked() {
        use crate::expand_param;
        for declaration in [
            expand_param!(&String).reconstruct().variant(fun!(make)),
            expand_param!(&String).variant(fun!(make)).reconstruct(),
        ] {
            let g = Cbindgen::new(model(
                "pub fn make(value:&str)->String{}pub fn inspect(value:&String)->usize{}",
            ))
            .function(fun!(inspect).expand_param("value", declaration))
            .panic()
            .build()
            .unwrap();
            assert!(g.rust.contains("active temporary"));
            assert!(g.rust.contains("make("));
        }
        let error = Cbindgen::new(model("pub fn inspect(value:&String)->usize{}"))
            .expand(expand_param!(&String).reconstruct())
            .function(fun!(inspect).expand_param("value", expand_param!(&String).variant_self()))
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reconstruct()"), "{error}");
        let g = Cbindgen::new(model("pub fn inspect(value:&String)->usize{}"))
            .opaque_ptr("text", syn::parse_quote!(String))
            .expand(expand_param!(&String).reconstruct())
            .function(fun!(inspect).expand_param("value", expand_param!(&String).variant_self()))
            .panic()
            .build()
            .unwrap();
        assert!(g.header.contains("const pb_text *"));
        assert!(!g.rust.contains("active temporary"));
    }
    #[test]
    fn reconstructed_inputs_cannot_back_escaping_loans() {
        for ty in ["&[i64]", "&str", "&[u8]"] {
            let source = format!("pub fn echo(values:{ty})->{ty}{{}}");
            let error = Cbindgen::new(model(&source))
                .function(fun!(echo).expand_param(
                    "values",
                    crate::expand_param(syn::parse_str(ty).unwrap()).reconstruct(),
                ))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(
                error.contains("reconstructed reference input"),
                "{ty}: {error}"
            );
        }
        for ty in ["&mut Vec<i64>", "Option<&[i64]>", "Vec<i64>"] {
            let source = format!("pub fn inspect(values:{ty})->usize{{}}");
            let error = Cbindgen::new(model(&source))
                .function(fun!(inspect).expand_param(
                    "values",
                    crate::expand_param(syn::parse_str(ty).unwrap()).reconstruct(),
                ))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(error.contains("reconstruct()"), "{ty}: {error}");
        }
    }
    #[test]
    fn direct_native_vector_and_string_references_keep_their_original_owner() {
        for (ty, name, layout) in [
            (
                syn::parse_quote!(Vec<i64>),
                "numbers",
                OpaqueLayout::auto::<Vec<i64>>(),
            ),
            (
                syn::parse_quote!(String),
                "text",
                OpaqueLayout::auto::<String>(),
            ),
        ] {
            let source = format!(
                "pub fn inspect(values:&{})->usize{{}}",
                prebindgen_flat::type_key(&ty)
            );
            let g = Cbindgen::new(model(&source))
                .opaque_owned_struct(name, ty, layout)
                .function(fun!(inspect))
                .panic()
                .build()
                .unwrap();
            assert!(g.header.contains(&format!("const pb_{name} *")));
            assert!(!g.rust.contains("reserve_exact"));
            assert!(!g.rust.contains("active temporary"));
        }
    }
    #[test]
    fn low_level_static_input_views_are_rejected() {
        let flat = model("");
        let error = input_sequence(
            &flat,
            &syn::parse_quote!(&'static [i64]),
            syn::parse_quote!(wire),
            syn::parse_quote!(value),
            &crate::operations::CInput::Leaf(CInputLeafOp::Value),
            &Default::default(),
        )
        .err()
        .unwrap();
        assert!(error.contains("'static lifetime"), "{error}");
        assert!(CCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &flat,
                &syn::parse_quote!(&'static str),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &Default::default()
            ))
            .err()
            .unwrap()
            .contains("'static reference lifetime"));
    }
    #[test]
    fn malformed_views_are_rejected() {
        assert!(CWire::SliceView {
            element: Primitive::Bool
        }
        .validate()
        .is_err());
    }
}
