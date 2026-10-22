//! Array rows retain their element schema in WireNode::Sequence, not in CWire.
use super::*;
use prebindgen_declarations::{SequenceInput, SequenceOutput};
use prebindgen_flat::{WireLayout, WireNode};

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &WireLayout<CWire, CGroup>,
) -> Result<SequenceInput<CWire>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let mut bind = TokenStream::new();
    let mut values = Vec::new();
    for (position, field) in element.iter().enumerate() {
        let name = &field.name;
        let member = format_ident!("v{position}");
        bind.extend(
            quote!(let #name=unsafe {(*((#wire.data as *const #row).add(#index))).#member};),
        );
        values.push(quote!(#name));
    }
    Ok(SequenceInput {
        carrier: CWire::Rows,
        open: quote!(let #length=#wire.len;::prebindgen_runtime::validate_slice(#wire.data as *const #row,#length)?;),
        bind,
        validation_values: values.clone(),
        prepared_values: values.into_iter().map(Some).collect(),
        release: quote!(),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    element: &WireLayout<CWire, CGroup>,
    values: &[TokenStream],
) -> Result<SequenceOutput<CWire>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    if element.len() != values.len() {
        return Err("collection row/value count differs".into());
    }
    let fields = values.iter().enumerate().map(|(i, value)| {
        let field = format_ident!("v{i}");
        quote!(#field:#value)
    });
    Ok(SequenceOutput {
        carrier: CWire::Rows,
        begin: quote!(let mut #wire:Vec<#row>=Vec::with_capacity(#length);),
        enter: quote!(),
        store: quote!(#wire.push(#row {#(#fields),*});),
        finish: quote!(let #wire=#wire.into_boxed_slice();),
        value: quote!(__PbRows {data:if #wire.is_empty(){::core::ptr::null()}else{#wire.as_ptr().cast()},len:#wire.len()}),
        commit: quote!(let _=Box::into_raw(#wire);),
    })
}
pub(crate) fn schemas<'a>(
    layout: &'a WireLayout<CWire, CGroup>,
    out: &mut Vec<(&'a Wire, &'a WireLayout<CWire, CGroup>)>,
) {
    for node in layout.nodes() {
        match node {
            WireNode::Wire(_) => {}
            WireNode::Group { children, .. } => schemas(children, out),
            WireNode::Sequence { carrier, element } => {
                schemas(element, out);
                out.push((carrier, element));
            }
        }
    }
}
