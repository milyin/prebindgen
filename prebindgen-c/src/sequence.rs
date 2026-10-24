//! Array rows retain their element schema in WireNode::Sequence, not in CWire.
use super::*;
use prebindgen_declarations::{SequenceInput, SequenceOutput};

use prebindgen_flat::WireNode;

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> Result<SequenceInput<CGroup>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let mut bind = TokenStream::new();
    let mut values = Vec::new();
    for (position, field) in crate::group::row_fields(element).enumerate() {
        let name = &field.name;
        let member = format_ident!("v{position}");
        bind.extend(
            quote!(let #name=unsafe {(*((#wire.data as *const #row).add(#index))).#member};),
        );
        values.push((field.name.clone(), quote!(#name)));
    }
    Ok(SequenceInput {
        metadata: CGroup::Sequence(Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }),
        open: quote!(let #length=#wire.len;::prebindgen_runtime::validate_slice(#wire.data as *const #row,#length)?;),
        bind,
        validation_values: values.clone(),
        prepared_values: values.into_iter().map(|(n, v)| (n, Some(v))).collect(),
        release: quote!(),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    element: &[prebindgen_flat::WireNode<CWire, CGroup>],
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<CGroup>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let fields = crate::group::row_values(element, values)?
        .into_iter()
        .enumerate()
        .map(|(i, (_, value))| {
            let field = format_ident!("v{i}");
            quote!(#field:#value)
        });
    Ok(SequenceOutput {
        commit_each: false,
        fallible: false,
        metadata: CGroup::Sequence(Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }),
        begin: quote!(let mut #wire:Vec<#row>=Vec::with_capacity(#length);),
        enter: quote!(),
        store: quote!(#wire.push(#row {#(#fields),*});),
        finish: quote!(let #wire=#wire.into_boxed_slice();),
        values: vec![(
            wire.clone(),
            quote!(__PbRows {data:if #wire.is_empty(){::core::ptr::null()}else{#wire.as_ptr().cast()},len:#wire.len()}),
        )],
        commit: quote!(let _=Box::into_raw(#wire);),
    })
}
pub(crate) fn schemas<'a>(
    layout: &'a [prebindgen_flat::WireNode<CWire, CGroup>],
    out: &mut Vec<(&'a Wire, &'a [prebindgen_flat::WireNode<CWire, CGroup>])>,
) {
    for node in layout {
        match node {
            WireNode::Wire(_) => {}
            WireNode::Outcomes { success, .. } => {
                schemas(success, out);
            }
            WireNode::Alternatives { alternatives, .. } => {
                for arm in alternatives {
                    schemas(arm, out);
                }
            }
            WireNode::Group { children, .. } => schemas(children, out),
            WireNode::Optional(c) => schemas(&c.element, out),
            WireNode::Sequence(c) => {
                schemas(&c.element, out);
                if let Some(carrier) = c.metadata.as_ref().and_then(CGroup::sequence_wire) {
                    out.push((carrier, &c.element));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_schemas(
    schemas: Vec<(&Wire, &[prebindgen_flat::WireNode<CWire, CGroup>])>,
    output_carriers: &std::collections::BTreeSet<String>,
    context: &CContext,
    prefix: &str,
    pointers: &[(String, syn::Type)],
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    for (carrier, element) in schemas {
        if !matches!(carrier.kind, CWire::Rows) {
            continue;
        }
        let row = &context.row_types[&carrier.name.to_string()];
        let base = row.to_string();
        let base = base.trim_start_matches("__PbRow_");
        let c_row = format!("{prefix}_{base}_row");
        let c_rows = format!("{prefix}_{base}_rows");
        let free_rows = format_ident!("{prefix}_{base}_rows_free");
        let is_output = output_carriers.contains(&carrier.name.to_string());
        let mut row_symbols = vec![c_row.clone(), c_rows.clone()];
        if is_output {
            row_symbols.push(free_rows.to_string());
        }
        for symbol in row_symbols {
            if !symbols.insert(symbol) {
                return Err("duplicate C collection symbol".into());
            }
        }
        let fields = crate::group::row_fields(element)
            .enumerate()
            .map(|(i, w)| {
                let name = format_ident!("v{i}");
                let ty = w.rust_type();
                quote!(pub #name:#ty)
            })
            .collect::<Vec<_>>();
        tokens.extend(quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #row {#(#fields),*}));
        header.push_str("typedef struct {\n");
        for (i, w) in crate::group::row_fields(element).enumerate() {
            header.push_str(&format!(
                "    {} v{i};\n",
                c_ty(w, prefix, pointers, context)
            ));
        }
        header.push_str(&format!(
            "}} {c_row};\ntypedef struct {{ {c_row} *data; size_t len; }} {c_rows};\n"
        ));
        if is_output {
            header.push_str(&format!("/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */\nvoid {free_rows}({c_rows} rows);\n"));
            tokens.extend(quote! {
                    #[doc="# Safety\nThe rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields."]
                    #[no_mangle]pub unsafe extern "C" fn #free_rows(rows:__PbRows){if rows.len!=0{drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(rows.data as *mut #row,rows.len)));}}
                });
        }
    }
    Ok(())
}

/// Reclaim a committed output that will not reach its caller, including nested rows.
pub(crate) fn release(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
    fields: &[(&Wire, TokenStream)],
    context: &CContext,
    catch_panics: bool,
) -> TokenStream {
    let mut schemas = Vec::new();
    self::schemas(layout, &mut schemas);
    let mut code = TokenStream::new();
    for (wire, value) in fields {
        match &wire.kind {
            CWire::Buffer => code.extend(quote! {if !(#value).data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut u8,(#value).len)));}}),
            CWire::Pointer { pointer, .. } => {
                let ty = &pointer.ty;
                let drop = quote!(drop(Box::from_raw((#value) as *mut #ty)););
                let drop = if catch_panics {
                    quote!(let _ = ::prebindgen_runtime::boundary(|| { #drop Ok(()) });)
                } else { drop };
                code.extend(quote! { if !(#value).is_null() { #drop } });
            }
            CWire::Rows => {
                let row=&context.row_types[&wire.name.to_string()];
                let element=schemas.iter().find(|(carrier,_)|carrier.name==wire.name).unwrap().1;
                let fields=crate::group::row_fields(element).enumerate().map(|(i,wire)|{let field=format_ident!("v{i}");(wire,quote!(row.#field))}).collect::<Vec<_>>();
                let child=release(element,&fields,context,catch_panics);
                code.extend(quote! {if (#value).len != 0 {let rows=Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut #row,(#value).len));for row in rows.iter(){#child}}});
            }
            _=>{}
        }
    }
    code
}

pub(crate) fn callback_output(
    wire: syn::Ident,
    values: &[(syn::Ident, TokenStream)],
    element: &[prebindgen_flat::WireNode<CWire, CGroup>],
    length: syn::Ident,
) -> Result<SequenceOutput<CGroup>> {
    let values = prebindgen_conversions::match_output_fields(
        crate::group::callback_parameters(element),
        values,
    )?;
    let names = (0..values.len())
        .map(|i| format_ident!("__pb_callback_value_{i}"))
        .collect::<Vec<_>>();
    let prepare = names
        .iter()
        .zip(values)
        .map(|(name, (_, value))| quote!(let #name=#value;));
    let commit = prebindgen_conversions::template_parameter("commit_element");
    Ok(SequenceOutput {
        commit_each: true,
        fallible: true, // A callback pointer must be provided; checked before source invocation.
        metadata: CGroup::Sequence(Wire {
            name: wire.clone(),
            kind: CWire::scalar(Primitive::Usize),
        }),
        begin: quote!(),
        enter: quote!(),
        store: quote!(#(#prepare)* #commit unsafe { __pb_callback.expect("validated callback")(__pb_callback_context, #(#names),*); }),
        finish: quote!(),
        values: vec![(wire.clone(), quote!(#length))],
        commit: quote!(),
    })
}

/// A visitor may replace exactly one collection, optionally with its presence flag.
pub(crate) fn callback_shape(layout: &[prebindgen_flat::WireNode<CWire, CGroup>]) -> bool {
    fn one(nodes: &[WireNode<CWire, CGroup>]) -> bool {
        match nodes {
            [WireNode::Sequence(c)] => c
                .metadata
                .as_ref()
                .and_then(CGroup::sequence_wire)
                .is_some_and(|carrier| {
                    matches!(
                        carrier.kind,
                        CWire::Scalar {
                            primitive: Primitive::Usize,
                            ..
                        }
                    )
                }),
            [WireNode::Optional(c)] => match &c.metadata {
                Some(CGroup::Optional(o)) if o.niche.is_none() => {
                    matches!(o.controls.as_slice(),[flag] if flag.name==o.discriminator)
                        && one(c.element.as_slice())
                }
                _ => false,
            },
            [WireNode::Group { children, .. }] => one(children.as_slice()),
            _ => false,
        }
    }
    one(match layout {
        [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
        nodes => nodes,
    })
}

/// The adapter selects its native scope before looking for a sequence.
#[allow(clippy::type_complexity)]
pub(crate) fn return_sequence(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> Option<(
    &prebindgen_flat::Wire<CWire>,
    &[prebindgen_flat::WireNode<CWire, CGroup>],
)> {
    let success = match layout {
        [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
        nodes => nodes,
    };
    prebindgen_declarations::outer_sequences(success)
        .into_iter()
        .find_map(|c| {
            c.metadata
                .as_ref()
                .and_then(CGroup::sequence_wire)
                .map(|w| (w, c.element.as_slice()))
        })
}
pub(crate) fn callback_target(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> Result<syn::Ident> {
    let success = match layout {
        [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
        nodes => nodes,
    };
    if prebindgen_declarations::outer_sequences(success).len() != 1
        || crate::group::result_packet_fields(layout).len() > 2
    {
        return Err(
            "callback_return permits only one sequence and its optional presence flag".into(),
        );
    }
    return_sequence(layout)
        .map(|(wire, _)| wire.name.clone())
        .ok_or_else(|| "callback_return requires a sequence boundary".into())
}
