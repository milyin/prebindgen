//! Shared optional conversion composition and wire niches.
use super::*;

/// An excluded representation reserved to encode one absent optional payload.
/// The binding name identifies the value to inspect; `niche` belongs to the codec.
#[derive(Clone, Debug)]
pub struct NicheReservation<N> {
    /// Payload binding whose excluded representation encodes absence.
    pub discriminator: syn::Ident,
    /// Excluded representation consumed by this optional layer.
    pub niche: N,
}

/// Choose the first usable absence representation among explicit candidate values.
/// The adapter supplies the candidates for this optional payload.
pub fn select_optional_niche<'a, C: Codec>(
    codec: &C,
    candidates: impl IntoIterator<Item = &'a Wire<C::Wire>>,
) -> Option<NicheReservation<C::Niche>>
where
    C::Wire: 'a,
{
    candidates.into_iter().find_map(|w| {
        codec.niche(&w.kind).map(|niche| NicheReservation {
            discriminator: w.name.clone(),
            niche,
        })
    })
}
/// Emit a presence test for a reservation in the adapter-selected optional payload.
pub fn optional_presence<'a, C: Codec>(
    codec: &C,
    candidates: impl IntoIterator<Item = &'a Wire<C::Wire>>,
    reservation: &NicheReservation<C::Niche>,
) -> Result<TokenStream>
where
    C::Wire: 'a,
{
    let wire = candidates
        .into_iter()
        .find(|w| w.name == reservation.discriminator)
        .ok_or("optional discriminator is absent from payload")?;
    let name = &wire.name;
    codec.is_present(&wire.kind, &reservation.niche, quote!(#name))
}
/// Prepare inert expressions for the named fields of an absent output payload.
pub fn absent_output_values<'a, C: Codec>(
    codec: &C,
    fields: impl IntoIterator<Item = &'a Wire<C::Wire>>,
    reservation: Option<&NicheReservation<C::Niche>>,
) -> Result<Vec<(syn::Ident, TokenStream)>>
where
    C::Wire: 'a,
{
    fields
        .into_iter()
        .map(|w| {
            Ok((
                w.name.clone(),
                codec.absent_value(
                    &w.kind,
                    reservation
                        .filter(|r| r.discriminator == w.name)
                        .map(|r| &r.niche),
                    false,
                )?,
            ))
        })
        .collect()
}
/// Retire niches in values that may be inactive. A reservation's donor keeps only
/// its remaining excluded domain for enclosing optional layers.
pub fn retire_optional_niches<'a, C: Codec>(
    codec: &C,
    fields: impl IntoIterator<Item = &'a mut Wire<C::Wire>>,
    reservation: Option<&NicheReservation<C::Niche>>,
) -> Result<()>
where
    C::Wire: 'a,
{
    let mut fields = fields.into_iter().collect::<Vec<_>>();
    let donor = reservation
        .map(|r| {
            fields
                .iter()
                .position(|w| w.name == r.discriminator)
                .ok_or("optional discriminator is absent from payload")
        })
        .transpose()?;
    let remaining =
        donor.and_then(|i| codec.remaining_niche(&fields[i].kind, &reservation.unwrap().niche));
    for w in &mut fields {
        codec.set_niche(&mut w.kind, None);
    }
    if let Some(i) = donor {
        codec.set_niche(&mut fields[i].kind, remaining);
    }
    Ok(())
}
/// Wrap an input plan whose activation condition includes `tag == 1`.
/// The payload's construction is guarded as well as its validation/acquisition.
/// Choose `tag` and `var` fresh within the complete slot.
pub fn optional_input<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tag: syn::Ident,
    var: syn::Ident,
    active: &Condition,
    mut child: CodecInput<C>,
) -> Result<CodecInput<C>> {
    let reservation = codec.optional_niche(&child.nodes);
    let mut validation = TokenStream::new();
    if let Some(reservation) = &reservation {
        let present = codec.optional_presence(&child.nodes, reservation)?;
        validation.extend(quote!(let #tag: i32 = i32::from(#present);));
    } else {
        child.fallible = true;
        let bad = active.and(quote!(#tag != 0 && #tag != 1));
        validation.extend(quote!(if #bad {return Err("invalid option presence".into());}));
    }
    validation.extend(child.validation);
    child.validation = validation;
    let body = child.statements;
    let arg = child.argument;
    let selected = active.arm(tag.clone(), 1).rust();
    child.statements = quote!(let #var = if #selected {#body Some(#arg)} else {None};);
    child.argument = quote!(#var);
    mark(codec, extra, &mut child.nodes, tag, reservation)?;

    Ok(child)
}
/// Prepare only a present output, retaining its owners in a deferred commit closure.
/// `child` must be unfinalized and refer to the supplied payload binding.
/// Choose `tag` and `payload` fresh within the complete slot.
/// Values must own their data or point into owners retained by `child.commit`.
/// The commit must be infallible and must not borrow an emission context that
/// delivery still needs: it is captured in a move closure until delivery succeeds.
pub fn optional_output<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tag: syn::Ident,
    payload: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    mut child: CodecOutput<C>,
) -> Result<CodecOutput<C>> {
    if !child.delivery.statements.is_empty()
        || child.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("optional child has delivery code".into());
    }
    codec.validate_output(&child)?;
    let reservation = codec.optional_niche(&child.nodes);
    let defaults = codec.absent_output_values(&child.nodes, reservation.as_ref())?;
    prebindgen_conversions::validate_value_bindings(&defaults)?;
    let mut defaults_by_name = defaults
        .into_iter()
        .map(|(n, v)| (n.to_string(), v))
        .collect::<std::collections::BTreeMap<_, _>>();
    let defaults = child
        .values
        .iter()
        .map(|(name, _)| {
            defaults_by_name
                .remove(&name.to_string())
                .ok_or_else(|| format!("missing absent output binding `{name}`"))
        })
        .collect::<Result<Vec<_>>>()?;
    if !defaults_by_name.is_empty() {
        return Err("unexpected absent output binding".into());
    }
    let names = (0..child.values.len())
        .map(|i| format_ident!("{tag}_value{i}"))
        .collect::<Vec<_>>();
    let presence_binding = if reservation.is_some() {
        quote!(_)
    } else {
        quote!(#tag)
    };
    let commit_var = format_ident!("{tag}_commit");
    let values_var = format_ident!("{tag}_values");
    let action_var = format_ident!("{tag}_action");
    let body = child.statements;
    let (keys, values): (Vec<_>, Vec<_>) = child.values.into_iter().unzip();
    let commit = child.commit;
    let source = if borrowed {
        quote!((#value).as_ref())
    } else {
        value
    };
    // Compute wire values before moving the owners into the closure. Dropping an
    // uncalled closure drops those owners on any subsequent conversion/delivery failure.
    child.statements = quote! {
        let (#presence_binding, (#(#names,)*), #commit_var) = if let Some(#payload) = #source {
            #body
            let #values_var = (#(#values,)*);
            (1i32, #values_var, Some(move || {#commit}))
        } else { (0i32, (#(#defaults,)*), None) };
    };
    child.commit = quote!(if let Some(#action_var) = #commit_var {#action_var();});
    child.values = keys
        .into_iter()
        .zip(names.iter().map(|n| quote!(#n)))
        .collect();
    if reservation.is_none() {
        child.values.insert(0, (tag.clone(), quote!(#tag)));
    }
    mark(codec, extra, &mut child.nodes, tag, reservation)?;

    Ok(child)
}

fn mark<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tree: &mut Vec<prebindgen_flat::WireNode<C::Wire, C::Group>>,
    tag: syn::Ident,
    reservation: Option<NicheReservation<C::Niche>>,
) -> Result<()> {
    let discriminator = reservation
        .as_ref()
        .map(|r| r.discriminator.clone())
        .unwrap_or_else(|| tag.clone());
    codec.retire_niches(tree, reservation.as_ref())?;
    let controls = if reservation.is_none() {
        vec![Wire {
            name: tag.clone(),
            kind: codec.selector(extra),
        }]
    } else {
        vec![]
    };
    let metadata =
        codec.optional_metadata(tag, discriminator, reservation.map(|r| r.niche), controls);
    let node = WireNode::Optional(prebindgen_flat::WireContainer {
        element: std::mem::take(tree),
        metadata: Some(metadata),
    });
    *tree = vec![node];
    Ok(())
}
