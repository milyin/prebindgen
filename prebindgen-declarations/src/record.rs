//! Shared record composition for declaration recipes and adapter-defined leaves.
use super::*;

/// Construct a named Rust record from field conversion plans in declaration order.
/// The layout has a shared `Fields` parent and one named `Field` child per field,
/// preserving empty fields and fields represented by multiple physical wires.
///
/// Adapters can use this when a single language-level leaf represents a record.
/// The caller supplies every field exactly once, with the correct source type,
/// and chooses a fresh `var` identifier. Child plans must use the same context
/// and activation condition and must not have been finalized with `finish_input`.
/// Validation, acquisition, resources, acknowledgements and construction remain
/// in their respective phases. Field paths are prepended through the codec.
/// Finalize only the complete occurrence with [`Codec::finish_input`].
pub fn record_input<C: Codec>(
    codec: &C,
    ty: &Type,
    var: syn::Ident,
    fields: impl IntoIterator<Item = Result<(syn::Ident, CodecInput<C>)>>,
) -> Result<CodecInput<C>> {
    let mut out: CodecInput<C> = InputPlan {
        context: Default::default(),
        acknowledgement: None,
        layout: Default::default(),
        validation: TokenStream::new(),
        acquire: TokenStream::new(),
        resources: Vec::new(),
        statements: TokenStream::new(),
        argument: quote!(#var),
    };
    let mut values = Vec::new();
    for field in fields {
        let (name, mut child) = field?;
        for wire in &mut child.layout {
            codec.prefix_component(&mut wire.kind, ComponentSegment::Field(name.to_string()));
        }
        out.validation.extend(child.validation);
        out.acquire.extend(child.acquire);
        out.resources.extend(child.resources);
        merge_acknowledgement(&mut out.acknowledgement, child.acknowledgement)?;
        out.statements.extend(child.statements);
        child
            .layout
            .enclose(CompositionGroup::new(GroupStructure::Field(name.clone())));
        out.layout.extend(child.layout);
        let argument = child.argument;
        values.push(quote!(#name: #argument));
    }
    out.statements
        .extend(quote!(let #var: #ty = #ty { #(#values),* };));
    out.layout
        .enclose(CompositionGroup::new(GroupStructure::Fields));
    Ok(out)
}

/// Observe a named Rust record through shared field references, evaluating it once.
/// The layout retains the same `Fields`/`Field` structure as [`record_input`].
///
/// The caller validates the field count and types against the source record.
/// `fields` supplies source field names in declaration order. `convert` receives
/// each field's index and a shared-reference expression; it must convert that
/// borrowed field without consuming it. Choose a fresh `var` identifier and use
/// the same context for all children. Field paths are prepended through the codec.
/// Child delivery must remain unset; finalize only the complete occurrence with
/// [`Codec::finish_output`]. Commit phases are preserved until successful delivery.
pub fn record_output<C: Codec>(
    codec: &C,
    var: syn::Ident,
    value: TokenStream,
    fields: impl IntoIterator<Item = syn::Ident>,
    mut convert: impl FnMut(usize, TokenStream) -> Result<CodecOutput<C>>,
) -> Result<CodecOutput<C>> {
    let mut fields = fields.into_iter().peekable();
    let binding = if fields.peek().is_some() {
        quote!(let #var = &(#value);)
    } else {
        quote!(let _ = &(#value);)
    };
    let mut out = OutputPlan {
        context: Default::default(),
        delivery: Default::default(),
        commit: TokenStream::new(),
        layout: Default::default(),
        statements: binding,
        values: Vec::new(),
    };
    for (index, name) in fields.enumerate() {
        let mut child = convert(index, quote!(&#var.#name))?;
        for wire in &mut child.layout {
            codec.prefix_component(&mut wire.kind, ComponentSegment::Field(name.to_string()));
        }
        child
            .layout
            .enclose(CompositionGroup::new(GroupStructure::Field(name)));
        append(&mut out, child)?;
    }
    out.layout
        .enclose(CompositionGroup::new(GroupStructure::Fields));
    Ok(out)
}

/// Decompose an owned record into field conversions in declaration order.
/// The layout retains the same `Fields`/`Field` structure as [`record_input`].
///
/// Unlike [`record_output`], each child receives an owned field expression and
/// may move it. The caller must provide each field exactly once, ensure Rust
/// permits moving its fields (in particular, the record must not implement
/// `Drop`), and mark the containing conversion as consuming. Choose a fresh
/// `var`. Unmoved fields and prepared child allocations retain Rust ownership
/// on failure. Children must leave delivery unset; commit follows final delivery.
pub fn record_output_owned<C: Codec>(
    codec: &C,
    var: syn::Ident,
    value: TokenStream,
    fields: impl IntoIterator<Item = syn::Ident>,
    mut convert: impl FnMut(usize, TokenStream) -> Result<CodecOutput<C>>,
) -> Result<CodecOutput<C>> {
    let mut fields = fields.into_iter().peekable();
    let binding = if fields.peek().is_some() {
        quote!(let #var = #value;)
    } else {
        quote!(let _ = #value;)
    };
    let mut out = OutputPlan {
        context: Default::default(),
        delivery: Default::default(),
        commit: TokenStream::new(),
        layout: Default::default(),
        statements: binding,
        values: Vec::new(),
    };
    for (index, name) in fields.enumerate() {
        let mut child = convert(index, quote!(#var.#name))?;
        for wire in &mut child.layout {
            codec.prefix_component(&mut wire.kind, ComponentSegment::Field(name.to_string()));
        }
        child
            .layout
            .enclose(CompositionGroup::new(GroupStructure::Field(name)));
        append(&mut out, child)?;
    }
    out.layout
        .enclose(CompositionGroup::new(GroupStructure::Fields));
    Ok(out)
}
