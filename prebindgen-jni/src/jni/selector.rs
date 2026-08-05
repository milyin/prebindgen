//! Structural converter-selection policy for [`Declarations`].

use prebindgen_registry::Conversions;

use super::{trait_impl::WrapperShape, *};

/// Whether a decoded `Vec<T>` local can be borrowed where `referent` is expected.
///
/// A question about the **form**, and it always was: `[T]` is reached by deref
/// coercion from `&Vec<T>` and `Vec<T>` is the thing itself, while a transparent
/// wrapper — `Box<Vec<T>>`, `Cow<'_, [T]>` — cannot be rebuilt from the decoded
/// local.
///
/// It asked the spelling because it had to. This doc used to say so: *"Rust
/// distinguishes forms the boundary classification does not"*, and that was true
/// when `Vec<T>` and `[T]` were one `Sequence` and a `Box` was erased. `TypeKind`
/// **is** the accepted syntax now, so the kind draws every distinction this needs
/// — `Vec`, `Slice`, `Boxed` and `Cow` are four kinds — and the question is
/// answered by the model instead of by a `match` on `syn`.
fn decoded_vec_satisfies(referent: &prebindgen_registry::flat::TypeRef) -> bool {
    matches!(
        referent.kind(),
        prebindgen_registry::flat::TypeKind::Slice(_) | prebindgen_registry::flat::TypeKind::Vec(_)
    )
}

/// Whether a type has no size, so no by-value converter can name it.
///
/// The peer of [`decoded_vec_satisfies`], and it stopped being a *spelling*
/// question for the same reason: `[T]` and `Vec<T>` were one concept once and
/// are two kinds now. A bare slice is reached exclusively through a borrow,
/// whose own arm handles it; claiming it here would generate `fn f(..) -> [T]`.
///
/// `str` is the same shape of fact and is handled the same way, one layer up:
/// its terminal arm resolves it to the borrowed `&str` converter rather than
/// pretending an owned `str` exists — which is why `Str` is not an arm here.
fn is_unsized_spelling(ty: &prebindgen_registry::flat::TypeRef) -> bool {
    matches!(ty.kind(), prebindgen_registry::flat::TypeKind::Slice(_))
}

impl Declarations {
    /// Select the input converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // What the converter YIELDS: this crossing's own reading, so a
        // `Box<Option<T>>` crossing produces a `Box<Option<T>>` — the shape it
        // is dispatched as does not decide what it is called. The one arm that
        // yields something else says so with `crate::jni::trait_impl::Produced::Composed`.
        let produced = crate::jni::trait_impl::Produced::Reading(ty);

        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.input_terminal(ty, registry, emit) {
            return Some(c);
        }
        // 3. Built-in wrapper shapes, read one layer at a time rather than as a
        //    whole stack: each arm handles exactly one, and hands the rest back
        //    through `subs` to be selected on its own. Peeling further here would
        //    claim a shape this selector does not emit.
        if let Some(inner) = ty.optional_inner() {
            // `Option<&T>` tries the DEEP `OptionRef` (borrowed-handle →
            // `Option<OwnedObject<T>>`) before the shallow `Optional`; the shape
            // that resolves correctly wins.
            if let Some(target) = inner.borrow_target() {
                let mutable = inner.is_exclusive_borrow();
                if let Some(mut c) = self.input_wrapper_shape(
                    WrapperShape::OptionRef { mutable },
                    &produced,
                    target,
                    registry,
                    emit,
                ) {
                    c.subs = vec![target.key()];
                    return Some(c);
                }
            }
            // An optional BORROW is the deep handler's alone. It declined —
            // either the inner is not a handle (then the shallow handler below
            // is right, and only for the canonical spelling) or the spelling
            // carries a wrapper it cannot bridge. The shallow handler cannot
            // tell those apart and would decode the jlong as a `*mut &T`, so a
            // wrapped optional borrow stops here rather than resolving wrong.
            if inner.borrow_target().is_some() && !ty.erased_wrappers().is_empty() {
                // "the spelling is exactly `Option<inner>`", off the model: this
                // rebuilt that canonical form and compared token strings, where
                // a wrapper over it is what `erased_wrappers` reports.
                return None;
            }
            if let Some(mut c) =
                self.input_wrapper_shape(WrapperShape::Optional, &produced, inner, registry, emit)
            {
                c.subs = vec![inner.key()];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(ty)) {
            if let Some(mut c) =
                self.input_wrapper_shape(WrapperShape::Sequence, &produced, elem, registry, emit)
            {
                c.subs = vec![elem.key()];
                return Some(c);
            }
            return None;
        }
        if let prebindgen_registry::flat::TypeKind::Ref { mutable, .. } = ty.unwrapped().kind() {
            // The target through the accessor: an out-parameter's `MaybeUninit`
            // is the slot a `T` goes in, and it is the `T` that converts.
            let inner = ty.borrow_target().expect("a borrow");
            // `&[T]` shared slice borrow: there is no owned `[T]` to decode, so
            // reuse the `Vec<_>` shape — decode the Java `List<T>` into an owned
            // `Vec<T>`; the call site borrows it (`&Vec<T>` deref-coerces to
            // `&[T]`). Wire/Kotlin type are `List<T>`, identical to a by-value
            // `Vec<T>` input (the writer dedupes the shared converter fn by ident,
            // so the two can coexist). `&mut [T]` is intentionally not supported
            // (no write-back of the decoded Vec).
            // Two questions, and only the first is `kind`'s. That it is a run of
            // values makes this arm a *candidate*; whether the decoded `Vec<T>`
            // can be handed to the Rust function is a question about the
            // **spelling**, because the generated glue is the one consumer that
            // can tell `&Vec<T>` from `&Box<Vec<T>>` — the exact thing
            // `TypeRef::origin` exists to carry.
            //
            // `&[T]` deref-coerces from `&Vec<T>` and `&Vec<T>` is already it, so
            // both are satisfied by the decoded local. A transparent wrapper —
            // `Box<Vec<T>>`, `Cow<'_, [T]>` — is `Sequence` all the same and is
            // NOT: passing `&Vec<T>` there does not compile. Those fall through to
            // the plain borrow arm below, which hands the whole spelling on as the
            // sub, exactly as the old syntactic slice check did.
            if !*mutable && decoded_vec_satisfies(inner) {
                if let Some(elem) = inner.sequence_elem() {
                    let elem_ty = emit.spell(elem);
                    // The one place `produced` is NOT the crossing's spelling:
                    // there is no owned `[T]` to decode into, so the converter
                    // yields an owned `Vec<T>` and the call site borrows it.
                    //
                    // It is also why `produced` stays a spelling while `t1`
                    // becomes a reading: this one is composed by the ADAPTER,
                    // and #280 sealed minting to the model — there is no
                    // `Vec<T>` reading for `api::lang` to make. Which is
                    // consistent rather than awkward: `produced` is defined as
                    // the tokens the converter yields, and every question asked
                    // of it (`is_canonical_spelling`, the `Type::Reference`
                    // bridgeability guards) is a spelling question.
                    let produced = crate::jni::trait_impl::Produced::Composed(
                        syn::parse_quote!(Vec<#elem_ty>),
                    );
                    if let Some(mut c) = self.input_wrapper_shape(
                        WrapperShape::Sequence,
                        &produced,
                        elem,
                        registry,
                        emit,
                    ) {
                        c.subs = vec![elem.key()];
                        return Some(c);
                    }
                    return None;
                }
            }
            let mutable = ty.is_exclusive_borrow();
            if let Some(mut c) = self.input_wrapper_shape(
                WrapperShape::Borrow { mutable },
                &produced,
                inner,
                registry,
                emit,
            ) {
                c.subs = vec![inner.key()];
                return Some(c);
            }
        }
        // 4. Last resort: the spelling differs from something convertible only
        //    by the wrappers the model erased. Nothing that resolves above
        //    reaches here, so this adds routes rather than changing them.
        self.input_transparent_bridge(ty, registry, emit)
    }

    /// Select the output converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_output_type(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // What the converter YIELDS. This direction used to be handed only the
        // spelling — `convert_crossing` fetched the reading and threw it away —
        // so it detected its layers with `option_inner_type`/`vec_inner_type`,
        // which read the last path segment's ident. A `Box<Option<T>>` answered
        // "neither", and got no converter at all (#270).
        let produced = crate::jni::trait_impl::Produced::Reading(ty);

        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.output_terminal(ty, registry, emit) {
            return Some(c);
        }
        // 2. `Result<T, E>`: succeeds as `T`, routes `E` to the error sink.
        //    Read off the model, which calls this shape `TypeKind::Fallible`.
        //    `result_parts` covers a `Result` the adapter composed itself, which
        //    the frontend never read.
        if let Some((ok, err)) = fallible_parts(ty, emit) {
            if let Some(c) = self.result_peel(ty, &ok, &err, registry, emit) {
                return Some(c);
            }
        }
        // 3. Built-in wrapper shapes, dispatched on what the model says the
        //    type IS. An `Option<&Handle>` resolves via the shallow `Optional`
        //    whose inner converter is the `&Handle` borrow entry (no deep
        //    output handler).
        if let Some(inner) = ty.optional_inner() {
            if let Some(mut c) =
                self.output_wrapper_shape(WrapperShape::Optional, &produced, inner, registry, emit)
            {
                c.subs = vec![inner.key()];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(ty)) {
            if let Some(mut c) =
                self.output_wrapper_shape(WrapperShape::Sequence, &produced, elem, registry, emit)
            {
                c.subs = vec![elem.key()];
                return Some(c);
            }
            return None;
        }
        if let prebindgen_registry::flat::TypeKind::Ref { mutable, .. } = ty.unwrapped().kind() {
            // The target through the accessor, as on the input side.
            let inner = ty.borrow_target().expect("a borrow");
            // `&[T]` shared slice (a callback argument crossing native→JVM):
            // build a `List<T>` from the borrowed slice. Dual of the `&[T]`
            // input branch, and the same split: `kind` says it is a borrow of a
            // run of values; whether the generated Rust can iterate the borrow
            // directly is a question about the SPELLING.
            if !*mutable && decoded_vec_satisfies(inner) {
                if let Some(elem) = inner.sequence_elem() {
                    return self.output_slice(elem, registry, emit);
                }
            }
            let mutable = ty.is_exclusive_borrow();
            if let Some(mut c) = self.output_wrapper_shape(
                WrapperShape::Borrow { mutable },
                &produced,
                inner,
                registry,
                emit,
            ) {
                c.subs = vec![inner.key()];
                return Some(c);
            }
        }
        // 4. Last resort: the spelling differs from something convertible only
        //    by the wrappers the model erased. Dual of the input side's step 4,
        //    and reached the same way — after every layer arm, so nothing that
        //    resolves today changes route (#309).
        self.output_transparent_bridge(ty, registry, emit)
    }
}

/// The `Ok`/`Err` of a `Result`, spelled.
///
/// The model classifies a `Result` as [`TypeKind::Fallible`], so a reading
/// answers this directly — no lookup, and no syntactic fallback.
///
/// The fallback there used to be (`types_util::result_parts` over the node)
/// existed because the caller held only a **spelling**: a `Result` the adapter
/// composed itself would have no entry in `flat`, so the reading lookup could
/// miss. A caller holding a `TypeRef` cannot be in that position — #280 sealed
/// minting to the model, so every reading reaching here was classified, and
/// `kind` is what says whether it is a `Result`. The fallback was measured
/// never to fire in-tree; it is now unreachable by construction.
fn fallible_parts(
    ty: &prebindgen_registry::flat::TypeRef,
    emit: &prebindgen_registry::Emit,
) -> Option<(syn::Type, syn::Type)> {
    let (ok, err) = ty.fallible_parts()?;
    let (ok, err) = (emit.spell(ok), emit.spell(err));
    Some((syn::parse_quote!(#ok), syn::parse_quote!(#err)))
}
