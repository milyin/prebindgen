//! Lift one element recipe into a runtime-sized collection without changing phases.
use super::*;
use prebindgen_conversions::{Resource, ResourceIdentity};
use prebindgen_flat::{SequenceView, WireNode};
use quote::ToTokens;

/// Adapter-supplied array input transport. Element conversion remains shared.
pub struct SequenceInput<W> {
    /// One self-sufficient carrier for the array.
    pub carrier: W,
    /// Validate the carrier and bind the supplied length identifier.
    pub open: TokenStream,
    /// Per-row setup, including any locals used by `validation_values`.
    pub bind: TokenStream,
    /// Raw element-wire expressions, in element layout order, used in validation.
    pub validation_values: Vec<TokenStream>,
    /// Reads valid after `release`, used by acquisition/construction and identities.
    /// `None` forbids reading that raw wire after validation (for example a JNI
    /// reference deleted with its row frame). The compiler rejects such reads.
    pub prepared_values: Vec<Option<TokenStream>>,
    /// Per-row cleanup after validation, before deferred closures are retained.
    pub release: TokenStream,
}
/// Adapter-supplied array output transport. Owners are committed only after delivery.
pub struct SequenceOutput<W> {
    /// One self-sufficient carrier for the array.
    pub carrier: W,
    /// Initialize the raw array builder, with the supplied element count available.
    pub begin: TokenStream,
    /// Per-row setup before element conversion.
    pub enter: TokenStream,
    /// Store one prepared element's wire values in the array.
    pub store: TokenStream,
    /// Finish preparing the carrier after every element has been stored.
    pub finish: TokenStream,
    /// Expression for the prepared array carrier.
    pub value: TokenStream,
    /// Commit the outer array's allocation after successful foreign delivery.
    pub commit: TokenStream,
}

fn replace(
    code: &TokenStream,
    layout: &WireLayout<impl prebindgen_flat::WireKind, impl Sized>,
    values: &[TokenStream],
) -> Result<TokenStream> {
    if layout.len() != values.len() {
        return Err("collection wire/read count differs".into());
    }
    let replacements = layout
        .iter()
        .zip(values)
        .map(|(w, v)| {
            Ok((
                w.name.to_string(),
                syn::parse2::<syn::Expr>(quote!((#v))).map_err(|e| e.to_string())?,
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
    layout: &WireLayout<impl prebindgen_flat::WireKind, impl Sized>,
    values: &[Option<TokenStream>],
) -> Result<TokenStream> {
    if values.len() != layout.len() {
        return Err("collection prepared read count differs".into());
    }
    fn mentions(code: TokenStream, name: &syn::Ident) -> bool {
        code.into_iter().any(|token| match token {
            proc_macro2::TokenTree::Ident(id) => id == *name,
            proc_macro2::TokenTree::Group(group) => mentions(group.stream(), name),
            _ => false,
        })
    }
    let mut reads = Vec::new();
    for (wire, value) in layout.iter().zip(values) {
        if value.is_none() && mentions(code.clone(), &wire.name) {
            return Err(format!(
                "collection wire `{}` cannot be read after row validation",
                wire.name
            ));
        }
        let name = &wire.name;
        reads.push(value.clone().unwrap_or_else(|| quote!(#name)));
    }
    replace(code, layout, &reads)
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
    if child.layout.is_empty() {
        return Err("collection elements must have at least one wire".into());
    }
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
    let validation = replace(
        &child.validation,
        &child.layout,
        &transport.validation_values,
    )?;
    let acquire = prepared(&child.acquire, &child.layout, &transport.prepared_values)?;
    let statements = prepared(&child.statements, &child.layout, &transport.prepared_values)?;
    // Wrap expressions as a local initializer so the syntax visitor can replace paths.
    let child_argument = &child.argument;
    let argument = prepared(
        &quote!(let #var = #child_argument;),
        &child.layout,
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
            &child.layout,
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
        context: Default::default(),
        acknowledgement: None,
        layout: WireLayout::from_nodes(vec![WireNode::Sequence {
            carrier: Wire {
                name: wire,
                kind: transport.carrier,
            },
            element: child.layout,
        }]),
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
    if child.layout.is_empty() {
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
        carrier,
        begin,
        enter,
        store,
        finish,
        value,
        commit,
    } = transport;
    Ok(OutputPlan {
        context: Default::default(),
        delivery: Default::default(),
        layout: WireLayout::from_nodes(vec![WireNode::Sequence {
            carrier: Wire {
                name: wire,
                kind: carrier,
            },
            element: child.layout,
        }]),
        statements: quote! {
            let #owner=#binding; let #length=#owner.len(); let mut #commits=Vec::with_capacity(#length);
            #begin
            for (#index,#element) in #iterator.enumerate(){ #enter #prepare #store #commits.push(move || {#child_commit}); }
            #finish
        },
        values: vec![value],
        commit: quote!(#commit for commit in #commits {commit();}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_read_replacement_preserves_raw_pointer_reborrows() {
        let layout: WireLayout<syn::Type> = vec![Wire {
            name: syn::parse_quote!(raw),
            kind: syn::parse_quote!(i64),
        }]
        .into();
        let code = replace(
            &quote!(let value = unsafe { &(*pointer) }; consume(raw, value);),
            &layout,
            &[quote!(cached)],
        )
        .unwrap();
        assert!(code.to_string().contains(&quote!(&(*pointer)).to_string()));
        assert!(code.to_string().contains("consume (cached , value)"));
    }
    #[test]
    fn deleted_row_references_cannot_escape_into_deferred_phases() {
        let layout: WireLayout<syn::Type> = vec![Wire {
            name: syn::parse_quote!(raw),
            kind: syn::parse_quote!(*const u8),
        }]
        .into();
        assert!(prepared(&quote!(decode(raw);), &layout, &[None])
            .unwrap_err()
            .contains("after row validation"));
        assert!(prepared(&quote!(decode_macro!(raw);), &layout, &[None]).is_err());
        assert!(prepared(&quote!(decode(already_validated);), &layout, &[None]).is_ok());
    }
}
