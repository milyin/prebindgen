//! Lift one element recipe into a runtime-sized collection without changing phases.
use super::*;

use prebindgen_conversions::{Resource, ResourceIdentity};
use prebindgen_flat::SequenceView;
use prebindgen_flat::{WireContainer, WireNode};
use quote::ToTokens;

/// Adapter-supplied array input transport. Element conversion remains shared.
pub struct SequenceInput<G> {
    /// Adapter-owned description of every outer input value. The adapter
    /// chooses their actual function parameters or row fields; element fields remain in the child tree.
    pub metadata: G,
    /// Validate the outer input values and make the supplied length identifier
    /// available, either as a boundary parameter or a local binding.
    pub open: TokenStream,
    /// Per-row setup, including any locals used by `validation_values`.
    pub bind: TokenStream,
    /// Raw element expressions keyed by binding name, used during validation.
    pub validation_values: Vec<(syn::Ident, TokenStream)>,
    /// Reads valid after `release`, used by acquisition/construction and identities.
    /// `None` forbids reading that raw wire after validation (for example a JNI
    /// reference deleted with its row frame). The compiler rejects such reads.
    pub prepared_values: Vec<(syn::Ident, Option<TokenStream>)>,
    /// Per-row cleanup after validation, before deferred closures are retained.
    pub release: TokenStream,
}
/// Adapter-supplied array output transport. Owners are committed only after delivery.
pub struct SequenceOutput<G> {
    /// Commit each element inside `store`, after capturing its prepared wire values
    /// and before invoking a foreign visitor. `store` must contain exactly one
    /// `template_parameter("commit_element")` marker when true. When false, all
    /// element owners remain in Rust until the outer slot commits.
    pub commit_each: bool,
    /// Whether preparing or delivering the array transport can return a binding
    /// error, in addition to failures from converting its elements.
    pub fallible: bool,
    /// Adapter-owned description of the outer delivery values. There may be
    /// several values, such as separate pointer and length parameters.
    pub metadata: G,
    /// Initialize delivery state, with the supplied element count available.
    pub begin: TokenStream,
    /// Per-row setup before element conversion.
    pub enter: TokenStream,
    /// Store or deliver one prepared element's wire values.
    pub store: TokenStream,
    /// Finish preparing the result after every element has been delivered.
    pub finish: TokenStream,
    /// Named expressions for every outer value. The adapter determines ABI order.
    pub values: Vec<(syn::Ident, TokenStream)>,
    /// Complete ownership transfer of the outer carrier, such as a row allocation.
    pub commit: TokenStream,
}

fn replace(code: &TokenStream, values: &[(syn::Ident, TokenStream)]) -> Result<TokenStream> {
    prebindgen_conversions::validate_value_bindings(values)?;
    let replacements = values
        .iter()
        .map(|(name, value)| {
            Ok((
                name.to_string(),
                syn::parse2::<syn::Expr>(quote!((#value))).map_err(|e| e.to_string())?,
            ))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
    struct Replace<'a>(&'a std::collections::BTreeMap<String, syn::Expr>);
    impl syn::visit_mut::VisitMut for Replace<'_> {
        fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
            if let syn::Expr::Path(p) = expr {
                if p.qself.is_none() && p.path.segments.len() == 1 {
                    if let Some(value) = self.0.get(&p.path.segments[0].ident.to_string()) {
                        *expr = value.clone();
                        return;
                    }
                }
            }
            syn::visit_mut::visit_expr_mut(self, expr);
            // An argument already has its own expression boundary. Keep grouping
            // elsewhere (notably when replacing the operand of `&wire`).
            if let syn::Expr::Match(value) = expr {
                if let syn::Expr::Paren(paren) = &*value.expr {
                    value.expr = paren.expr.clone();
                }
            }
            let args = match expr {
                syn::Expr::Call(call) => Some(&mut call.args),
                syn::Expr::MethodCall(call) => Some(&mut call.args),
                _ => None,
            };
            if let Some(args) = args {
                for arg in args {
                    if let syn::Expr::Paren(paren) = arg {
                        *arg = *paren.expr.clone();
                    }
                }
            }
        }
    }
    let mut expr: syn::Expr = syn::parse2(quote!({ #code })).map_err(|e| e.to_string())?;
    syn::visit_mut::VisitMut::visit_expr_mut(&mut Replace(&replacements), &mut expr);
    let syn::Expr::Block(block) = expr else {
        unreachable!()
    };
    Ok(block
        .block
        .stmts
        .iter()
        .map(ToTokens::to_token_stream)
        .collect())
}

fn prepared(
    code: &TokenStream,
    values: &[(syn::Ident, Option<TokenStream>)],
) -> Result<TokenStream> {
    fn mentions(code: TokenStream, name: &syn::Ident) -> bool {
        code.into_iter().any(|token| match token {
            proc_macro2::TokenTree::Ident(id) => id == *name,
            proc_macro2::TokenTree::Group(group) => mentions(group.stream(), name),
            _ => false,
        })
    }
    let mut reads = Vec::new();
    for (name, value) in values {
        if value.is_none() && mentions(code.clone(), name) {
            return Err(format!(
                "collection wire `{}` cannot be read after row validation",
                name
            ));
        }
        reads.push((name.clone(), value.clone().unwrap_or_else(|| quote!(#name))));
    }
    replace(code, &reads)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn input<C: Codec>(
    codec: &C,
    ty: &Type,
    extra: &C::Extra,
    wire: syn::Ident,
    var: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    active: &Condition,
    child: CodecInput<C>,
) -> Result<CodecInput<C>> {
    let (element, view) = prebindgen_flat::sequence_parts(ty)
        .ok_or("sequence recipe requires Vec<T>, &[T] or &Vec<T>")?;
    if view == SequenceView::Mutable {
        return Err("mutable collections require explicit write-back support".into());
    }
    if matches!(element, Type::Reference(_)) {
        return Err("collection reference elements are unsupported; use owned elements".into());
    }
    codec.validate_input(&child)?;
    if view != SequenceView::Owned
        && child
            .resources
            .iter()
            .any(|r| r.access == prebindgen_conversions::Access::Move)
    {
        return Err("borrowed collection inputs cannot consume elements; declare an element constructor using borrowed inputs, or use Vec<T>".into());
    }
    if child.acknowledgement.is_some() {
        return Err("collection element acknowledgement belongs to the whole call".into());
    }
    let transport = codec.sequence_input(
        extra,
        wire.clone(),
        length.clone(),
        index.clone(),
        &child.layout,
        active,
    )?;
    if transport.validation_values.is_empty() {
        return Err("collection elements must have at least one input binding".into());
    }
    prebindgen_conversions::validate_value_bindings(&transport.validation_values)?;
    let names = transport
        .validation_values
        .iter()
        .map(|(n, _)| n.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let prepared_names = transport
        .prepared_values
        .iter()
        .map(|(n, _)| n.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    if names != prepared_names || prepared_names.len() != transport.prepared_values.len() {
        return Err("collection prepared input names differ from validation bindings".into());
    }
    let validation = replace(&child.validation, &transport.validation_values)?;
    let acquire = prepared(&child.acquire, &transport.prepared_values)?;
    let statements = prepared(&child.statements, &transport.prepared_values)?;
    // Wrap expressions as a local initializer so the syntax visitor can replace paths.
    let child_argument = &child.argument;
    let argument = prepared(
        &quote!(let #var = #child_argument;),
        &transport.prepared_values,
    )?;
    let mut resources = Vec::new();
    let mut declarations = TokenStream::new();
    let mut collect = TokenStream::new();
    for (position, resource) in child.resources.into_iter().enumerate() {
        let ids = format_ident!("{wire}_identities_{position}");
        declarations.extend(quote!(let mut #ids:Vec<usize>=Vec::new();));
        let condition = &resource.active;
        let identity = match resource.identity {
            ResourceIdentity::Single(code) => quote!(#ids.push((#code) as usize);),
            ResourceIdentity::Many(code) => quote!(#ids.extend(#code);),
        };
        collect.extend(prepared(
            &quote!(if #condition {#identity}),
            &transport.prepared_values,
        )?);
        resources.push(Resource {
            identity: ResourceIdentity::Many(quote!(#ids.iter().copied())),
            active: active.rust(),
            access: resource.access,
        });
    }
    let staged = format_ident!("{wire}_staged");
    let acquired = format_ident!("{wire}_acquired");
    let open = transport.open;
    let bind = transport.bind;
    let release = transport.release;
    let condition = active.rust();
    let value = match view {
        SequenceView::Owned => quote!(#var),
        SequenceView::Vector => quote!(&#var),
        SequenceView::Slice => quote!(#var.as_slice()),
        SequenceView::Mutable => unreachable!(),
    };
    Ok(InputPlan {
        fallible: true,
        context: Default::default(),
        acknowledgement: None,
        layout: WireLayout::from_nodes(vec![WireNode::Sequence(WireContainer {
            element: child.layout,
            metadata: Some(transport.metadata),
        })]),
        validation: quote! {
            #declarations
            let mut #staged=Vec::new();
            if #condition { #open for #index in 0..#length {
                #bind #validation #collect #release
                #staged.push(move || { #acquire move || -> ::core::result::Result<#element,String> { #statements #argument Ok(#var) } });
            }}
        },
        acquire: quote!(let #acquired:Vec<_>=#staged.into_iter().map(|acquire|acquire()).collect();),
        statements: quote!(let #var:Vec<#element>=#acquired.into_iter().map(|construct|construct()).collect::<::core::result::Result<_,String>>()?;),
        argument: value,
        resources,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn output<C: Codec>(
    codec: &C,
    ty: &Type,
    extra: &C::Extra,
    wire: syn::Ident,
    element: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    mode: ProjectionMode,
    child: CodecOutput<C>,
) -> Result<CodecOutput<C>> {
    let (element_type, view) =
        prebindgen_flat::sequence_parts(ty).ok_or("sequence recipe requires a collection")?;
    if matches!(element_type, Type::Reference(_)) {
        return Err("collection reference elements are unsupported; use owned elements".into());
    }
    if view == SequenceView::Mutable {
        return Err("mutable collection outputs are unsupported".into());
    }
    if mode == ProjectionMode::Owned && (borrowed || view != SequenceView::Owned) {
        return Err("owned collection projection cannot consume a borrowed collection".into());
    }
    codec.validate_output(&child)?;
    if child.values.is_empty() {
        return Err("collection elements must have at least one wire".into());
    }
    if !child.delivery.statements.is_empty() {
        return Err("collection element delivery belongs to the outer slot".into());
    }
    let transport = codec.sequence_output(
        extra,
        wire.clone(),
        length.clone(),
        index.clone(),
        &child.layout,
        &child.values,
    )?;
    let owner = format_ident!("{wire}_source");
    let commits = format_ident!("{wire}_commits");
    let (binding, iterator) = if mode == ProjectionMode::Owned {
        (value, quote!(#owner.into_iter()))
    } else {
        (quote!(&(#value)), quote!(#owner.iter()))
    };
    let prepare = child.statements;
    let child_commit = child.commit;
    let SequenceOutput {
        commit_each,
        fallible,
        metadata,
        begin,
        enter,
        store,
        finish,
        values,
        commit,
    } = transport;
    let store = if commit_each {
        let count = std::cell::Cell::new(0);
        let store = prebindgen_conversions::render_template(&store, &|name| {
            if name == "commit_element" {
                count.set(count.get() + 1);
                Ok(child_commit.clone())
            } else {
                Ok(prebindgen_conversions::template_parameter(name))
            }
        })?;
        if count.get() != 1 {
            return Err("per-element delivery requires exactly one commit_element marker".into());
        }
        store
    } else {
        quote!(#store #commits.push(move || {#child_commit});)
    };
    let declarations = if commit_each {
        quote!()
    } else {
        quote!(let mut #commits=Vec::with_capacity(#length);)
    };
    let commit_elements = if commit_each {
        quote!()
    } else {
        quote!(for commit in #commits {commit();})
    };
    let out: CodecOutput<C> = OutputPlan {
        fallible: child.fallible || fallible,
        context: Default::default(),
        delivery: Default::default(),
        layout: WireLayout::from_nodes(vec![WireNode::Sequence(WireContainer {
            element: child.layout,
            metadata: Some(metadata),
        })]),
        statements: quote! {
            let #owner=#binding; let #length=#owner.len(); #declarations
            #begin
            for (#index,#element) in #iterator.enumerate(){ #enter #prepare #store }
            #finish
        },
        values,
        commit: quote!(#commit #commit_elements),
    };
    codec.validate_output(&out)?;
    Ok(out)
}

/// Find outer logical sequence nodes, including through groups and options.
/// This inspects structure only. The caller selects an outcome and decides which
/// sequences belong to its emission scope; sequence elements are not traversed.
pub fn outer_sequences<W: prebindgen_flat::WireKind, G>(
    layout: &WireLayout<W, G>,
) -> Vec<&WireContainer<W, G>> {
    fn visit<'a, W: prebindgen_flat::WireKind, G>(
        nodes: &'a [WireNode<W, G>],
        out: &mut Vec<&'a WireContainer<W, G>>,
    ) {
        for node in nodes {
            match node {
                WireNode::Sequence(c) => out.push(c),
                WireNode::Optional(c) => visit(c.element.nodes(), out),
                WireNode::Group { children, .. } => visit(children.nodes(), out),
                WireNode::Alternatives { alternatives, .. } => {
                    for arm in alternatives {
                        visit(arm.nodes(), out);
                    }
                }
                WireNode::Wire(_) | WireNode::Outcomes { .. } => {}
            }
        }
    }
    let mut out = vec![];
    visit(layout.nodes(), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_discovery_is_logical_and_outcome_selection_is_explicit() {
        let mut layout: WireLayout<syn::Type> = vec![Wire {
            name: syn::parse_quote!(value),
            kind: syn::parse_quote!(i64),
        }]
        .into();
        layout.enclose_sequence(None);
        layout.enclose_optional(None);
        layout.enclose(());
        assert_eq!(outer_sequences(&layout).len(), 1);
        layout.set_outcome(
            prebindgen_flat::OutcomeKind::BindingFailure,
            WireLayout::default(),
        );
        assert!(outer_sequences(&layout).is_empty());
        assert_eq!(
            outer_sequences(
                layout
                    .outcome(prebindgen_flat::OutcomeKind::Success)
                    .unwrap()
            )
            .len(),
            1
        );
    }
    #[test]
    fn row_read_replacement_preserves_raw_pointer_reborrows() {
        let code = replace(
            &quote!(let value = unsafe { &(*pointer) }; consume(raw, value);),
            &[(syn::parse_quote!(raw), quote!(cached))],
        )
        .unwrap();
        assert!(code.to_string().contains(&quote!(&(*pointer)).to_string()));
        assert!(code.to_string().contains("consume (cached , value)"));
    }
    #[test]
    fn deleted_row_references_cannot_escape_into_deferred_phases() {
        assert!(
            prepared(&quote!(decode(raw);), &[(syn::parse_quote!(raw), None)])
                .unwrap_err()
                .contains("after row validation")
        );
        assert!(prepared(
            &quote!(decode_macro!(raw);),
            &[(syn::parse_quote!(raw), None)]
        )
        .is_err());
        assert!(prepared(
            &quote!(decode(already_validated);),
            &[(syn::parse_quote!(raw), None)]
        )
        .is_ok());
    }
}
