//! Shared optional control flow and opt-in niche encoding utilities.
use super::*;

/// One excluded representation reserved for an optional layer.
#[derive(Clone, Debug)]
pub struct NicheReservation<N> {
    /// Existing payload wire selected as discriminator.
    pub discriminator: syn::Ident,
    /// Reserved absent representation, owned by the enclosing metadata.
    pub niche: N,
}
/// Prepared payload state before an adapter adds a flag or optional-node metadata.
pub struct PreparedOptionalPayload<N> {
    /// Reserved discriminator, or None when the adapter must add a flag.
    pub reservation: Option<NicheReservation<N>>,
    /// Boolean presence test for an input niche; absent for output or a flag.
    pub input_test: Option<TokenStream>,
    /// Inactive output expressions. Empty for input.
    pub absent_values: WireValues,
}
/// Select the first available representation from adapter-chosen native-scope wires.
pub fn select_optional_niche<'a, W: prebindgen_flat::WireKind + 'a, N>(
    wires: impl IntoIterator<Item = &'a Wire<W>>,
    mut available: impl FnMut(&W) -> Option<N>,
) -> Option<NicheReservation<N>> {
    wires.into_iter().find_map(|wire| {
        available(&wire.kind).map(|niche| NicheReservation {
            discriminator: wire.name.clone(),
            niche,
        })
    })
}
/// Prepare a total input presence test using the reserved payload discriminator.
pub fn optional_presence<'a, W: prebindgen_flat::WireKind + 'a, N>(
    wires: impl IntoIterator<Item = &'a Wire<W>>,
    reservation: &NicheReservation<N>,
    mut present: impl FnMut(&W, &N, TokenStream) -> Result<TokenStream>,
) -> Result<TokenStream> {
    let wire = wires
        .into_iter()
        .find(|wire| wire.name == reservation.discriminator)
        .ok_or("optional discriminator is absent from payload")?;
    let name = &wire.name;
    present(&wire.kind, &reservation.niche, quote!(#name))
}
/// Prepare absent output values, using the reserved niche on its donor wire.
pub fn absent_output_values<'a, W: prebindgen_flat::WireKind + 'a, N>(
    wires: impl IntoIterator<Item = &'a Wire<W>>,
    reservation: Option<&NicheReservation<N>>,
    mut absent: impl FnMut(&W, Option<&N>) -> Result<TokenStream>,
) -> Result<WireValues> {
    wires
        .into_iter()
        .map(|wire| {
            Ok((
                wire.name.clone(),
                absent(
                    &wire.kind,
                    reservation
                        .filter(|r| r.discriminator == wire.name)
                        .map(|r| &r.niche),
                )?,
            ))
        })
        .collect()
}
/// Retire inactive offers within one native scope, preserving only the donor's
/// remaining domain. Also used for alternatives and foreign-record annotation.
pub fn retire_optional_niches<'a, W: prebindgen_flat::WireKind + 'a, N>(
    wires: impl IntoIterator<Item = &'a mut Wire<W>>,
    reservation: Option<&NicheReservation<N>>,
    mut remaining: impl FnMut(&W, &N) -> Option<N>,
    mut set_available: impl FnMut(&mut W, Option<N>) -> Result<()>,
) -> Result<()> {
    let mut wires = wires.into_iter().collect::<Vec<_>>();
    let donor = reservation
        .map(|r| {
            wires
                .iter()
                .position(|w| w.name == r.discriminator)
                .ok_or("optional discriminator is absent from payload")
        })
        .transpose()?;
    let rest = donor.and_then(|i| remaining(&wires[i].kind, &reservation.unwrap().niche));
    for wire in &mut wires {
        set_available(&mut wire.kind, None)?;
    }
    if let Some(i) = donor {
        set_available(&mut wires[i].kind, rest)?;
    }
    Ok(())
}
/// Prepare a complete optional payload using only adapter-selected scope wires.
/// Tests/defaults are generated before modifying the available representation state.
/// All callbacks execute during generation, not in the generated library.
pub fn prepare_optional_payload<'a, W: prebindgen_flat::WireKind + 'a, N>(
    wires: impl IntoIterator<Item = &'a mut Wire<W>>,
    direction: Direction,
    available: impl FnMut(&W) -> Option<N>,
    set_available: impl FnMut(&mut W, Option<N>) -> Result<()>,
    present: impl FnMut(&W, &N, TokenStream) -> Result<TokenStream>,
    remaining: impl FnMut(&W, &N) -> Option<N>,
    absent: impl FnMut(&W, Option<&N>) -> Result<TokenStream>,
) -> Result<PreparedOptionalPayload<N>> {
    let wires = wires.into_iter().collect::<Vec<_>>();
    let reservation = select_optional_niche(wires.iter().map(|w| &**w), available);
    let input_test = if direction == Direction::Input {
        reservation
            .as_ref()
            .map(|r| optional_presence(wires.iter().map(|w| &**w), r, present))
            .transpose()?
    } else {
        None
    };
    let absent_values = if direction == Direction::Output {
        absent_output_values(wires.iter().map(|w| &**w), reservation.as_ref(), absent)?
    } else {
        Vec::new()
    };
    retire_optional_niches(wires, reservation.as_ref(), remaining, set_available)?;
    Ok(PreparedOptionalPayload {
        reservation,
        input_test,
        absent_values,
    })
}
fn validate_encoding<C: Codec>(
    encoding: &OptionalEncoding<C>,
    tag: &syn::Ident,
    direction: Direction,
) -> Result<()> {
    match &encoding.presence {
        Presence::Flag(wire) if wire.name != *tag => {
            return Err("optional flag must use the requested presence binding".into())
        }
        Presence::Niche { input_test }
            if input_test.is_some() != (direction == Direction::Input) =>
        {
            return Err("optional niche must provide a presence test only for input".into())
        }
        _ => {}
    }
    if direction == Direction::Input && !encoding.absent_values.is_empty() {
        return Err("optional input cannot contain absent output expressions".into());
    }
    Ok(())
}
fn mark<C: Codec>(tree: &mut CodecNode<C>, encoding: OptionalEncoding<C>) {
    let controls = match encoding.presence {
        Presence::Flag(wire) => vec![wire],
        Presence::Niche { .. } => vec![],
    };
    *tree = WireNode::Chain {
        controls,
        child: Box::new(std::mem::take(tree)),
        meta: encoding.metadata,
    };
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
    let encoding = codec.optional(Optional {
        extra,
        payload: &mut child.node,
        presence: tag.clone(),
        direction: Direction::Input,
    })?;
    validate_encoding(&encoding, &tag, Direction::Input)?;
    let mut validation = TokenStream::new();
    if let Presence::Niche {
        input_test: Some(present),
    } = &encoding.presence
    {
        validation.extend(quote!(let #tag: i32 = i32::from(#present);));
    } else if codec.validates_inputs() {
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
    mark(&mut child.node, encoding);

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
    validate_nested_output(&child)?;
    codec.validate_output(&child)?;
    let mut encoding = codec.optional(Optional {
        extra,
        payload: &mut child.outcomes.success,
        presence: tag.clone(),
        direction: Direction::Output,
    })?;
    validate_encoding(&encoding, &tag, Direction::Output)?;
    let uses_niche = matches!(encoding.presence, Presence::Niche { .. });
    let defaults = std::mem::take(&mut encoding.absent_values);
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
    let presence_binding = if uses_niche { quote!(_) } else { quote!(#tag) };
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
    if !uses_niche {
        child.values.insert(0, (tag.clone(), quote!(#tag)));
    }
    mark(&mut child.outcomes.success, encoding);

    Ok(child)
}
