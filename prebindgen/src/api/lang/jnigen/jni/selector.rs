//! Structural converter-selection policy for [`Declarations`].

use super::{trait_impl::WrapperShape, *};
use crate::api::core::registry::Conversions;

/// Whether a decoded `Vec<T>` local can be borrowed where `referent` is expected.
///
/// A **spelling** question, deliberately: it decides what the generated Rust must
/// be able to say, and Rust distinguishes forms the boundary classification does
/// not. `[T]` is reached by deref coercion from `&Vec<T>` and `Vec<T>` is the
/// thing itself; a transparent wrapper such as `Box<Vec<T>>` or `Cow<'_, [T]>`
/// classifies identically and cannot be reconstructed from the decoded local.
fn decoded_vec_satisfies(referent: &syn::Type) -> bool {
    match referent {
        syn::Type::Slice(_) => true,
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "Vec" && tp.path.segments.len() == 1),
        _ => false,
    }
}

/// Whether a spelling has no size, so no by-value converter can name it.
///
/// A **spelling** question, like [`decoded_vec_satisfies`]: `[T]` and `Vec<T>`
/// are one concept to the model — both `Sequence` — and Rust can return only
/// one of them. A bare slice is reached exclusively through a borrow, whose own
/// arm handles it; claiming it here would generate `fn f(..) -> [T]`.
///
/// `str` is the same shape of fact and is handled the same way, one layer up:
/// its terminal arm resolves it to the borrowed `&str` converter rather than
/// pretending an owned `str` exists.
fn is_unsized_spelling(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Slice(_))
}

impl Declarations {
    /// Select the input converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        use crate::api::core::flat::RefMode;

        // What the type IS comes from `kind`; what generated Rust must SPELL it
        // comes from here. The converter yields this spelling, so a
        // `Box<Option<T>>` crossing produces a `Box<Option<T>>` — the shape it
        // is dispatched as no longer decides what it is called.
        let syntax = ty.syntax();

        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.input_terminal(ty, registry) {
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
                let mutable = matches!(
                    inner.kind(),
                    crate::api::core::flat::TypeKind::Ref {
                        mode: RefMode::Exclusive,
                        ..
                    }
                );
                if let Some(mut c) = self.input_wrapper_shape(
                    WrapperShape::OptionRef { mutable },
                    syntax,
                    target,
                    registry,
                ) {
                    c.subs = vec![target.syntax().clone()];
                    return Some(c);
                }
            }
            // An optional BORROW is the deep handler's alone. It declined —
            // either the inner is not a handle (then the shallow handler below
            // is right, and only for the canonical spelling) or the spelling
            // carries a wrapper it cannot bridge. The shallow handler cannot
            // tell those apart and would decode the jlong as a `*mut &T`, so a
            // wrapped optional borrow stops here rather than resolving wrong.
            if inner.borrow_target().is_some() {
                let canonical: syn::Type = {
                    let b = inner.syntax();
                    syn::parse_quote!(Option<#b>)
                };
                if syntax.to_token_stream().to_string() != canonical.to_token_stream().to_string() {
                    return None;
                }
            }
            if let Some(mut c) =
                self.input_wrapper_shape(WrapperShape::Optional, syntax, inner, registry)
            {
                c.subs = vec![inner.syntax().clone()];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(syntax)) {
            if let Some(mut c) =
                self.input_wrapper_shape(WrapperShape::Sequence, syntax, elem, registry)
            {
                c.subs = vec![elem.syntax().clone()];
                return Some(c);
            }
            return None;
        }
        if let crate::api::core::flat::TypeKind::Ref { mode, inner } = ty.kind() {
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
            if matches!(mode, RefMode::Shared) && decoded_vec_satisfies(inner.syntax()) {
                if let Some(elem) = inner.sequence_elem() {
                    let elem_ty = elem.syntax().clone();
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
                    let produced: syn::Type = syn::parse_quote!(Vec<#elem_ty>);
                    if let Some(mut c) =
                        self.input_wrapper_shape(WrapperShape::Sequence, &produced, elem, registry)
                    {
                        c.subs = vec![elem_ty];
                        return Some(c);
                    }
                    return None;
                }
            }
            let mutable = matches!(mode, RefMode::Exclusive);
            if let Some(mut c) =
                self.input_wrapper_shape(WrapperShape::Borrow { mutable }, syntax, inner, registry)
            {
                c.subs = vec![inner.syntax().clone()];
                return Some(c);
            }
        }
        // 4. Last resort: the spelling differs from something convertible only
        //    by the wrappers the model erased. Nothing that resolves above
        //    reaches here, so this adds routes rather than changing them.
        self.input_transparent_bridge(ty, registry)
    }

    /// Select the output converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_output_type(
        &self,
        ty: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        use crate::api::core::flat::RefMode;

        // What the type IS comes from `kind`; the spelling is what generated
        // Rust must say. This direction used to be handed only the spelling —
        // `convert_crossing` fetched the reading and threw it away — so it
        // detected its layers with `option_inner_type`/`vec_inner_type`, which
        // read the last path segment's ident. A `Box<Option<T>>` answered
        // "neither", and got no converter at all (#270).
        let syntax = ty.syntax();

        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.output_terminal(ty, registry) {
            return Some(c);
        }
        // 2. `Result<T, E>`: succeeds as `T`, routes `E` to the error sink.
        //    Read off the model, which calls this shape `TypeKind::Fallible`.
        //    `result_parts` covers a `Result` the adapter composed itself, which
        //    the frontend never read.
        if let Some((ok, err)) = fallible_parts(syntax, registry) {
            if let Some(c) = self.result_peel(syntax, &ok, &err, registry) {
                return Some(c);
            }
        }
        // 3. Built-in wrapper shapes, dispatched on what the model says the
        //    type IS. An `Option<&Handle>` resolves via the shallow `Optional`
        //    whose inner converter is the `&Handle` borrow entry (no deep
        //    output handler).
        if let Some(inner) = ty.optional_inner() {
            if let Some(mut c) =
                self.output_wrapper_shape(WrapperShape::Optional, syntax, inner, registry)
            {
                c.subs = vec![inner.syntax().clone()];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(syntax)) {
            if let Some(mut c) =
                self.output_wrapper_shape(WrapperShape::Sequence, syntax, elem, registry)
            {
                c.subs = vec![elem.syntax().clone()];
                return Some(c);
            }
            return None;
        }
        if let crate::api::core::flat::TypeKind::Ref { mode, inner } = ty.kind() {
            // `&[T]` shared slice (a callback argument crossing native→JVM):
            // build a `List<T>` from the borrowed slice. Dual of the `&[T]`
            // input branch, and the same split: `kind` says it is a borrow of a
            // run of values; whether the generated Rust can iterate the borrow
            // directly is a question about the SPELLING.
            if matches!(mode, RefMode::Shared) && decoded_vec_satisfies(inner.syntax()) {
                if let Some(elem) = inner.sequence_elem() {
                    return self.output_slice(elem.syntax(), registry);
                }
            }
            let mutable = matches!(mode, RefMode::Exclusive);
            if let Some(mut c) =
                self.output_wrapper_shape(WrapperShape::Borrow { mutable }, syntax, inner, registry)
            {
                c.subs = vec![inner.syntax().clone()];
                return Some(c);
            }
        }
        // 4. Last resort: the spelling differs from something convertible only
        //    by the wrappers the model erased. Dual of the input side's step 4,
        //    and reached the same way — after every layer arm, so nothing that
        //    resolves today changes route (#309).
        self.output_transparent_bridge(ty, registry)
    }
}

/// The `Ok`/`Err` of a `Result`, preferring the frontend's reading.
///
/// The model classifies a `Result` as [`TypeKind::Fallible`]; the syntactic
/// fallback is for a `Result` the adapter composed itself, which no captured
/// item spells and the frontend therefore never read.
///
/// **Measured: the fallback never fires in-tree** — zero occurrences across
/// covertest-kotlin and perftest-kotlin, because #246 indexes a binding-local
/// fn's types, so even a `sig!((..) -> Result<Summary, String>)` has a reading.
/// It is kept rather than made a hard error because an out-of-tree consumer may
/// compose a `Result` the model never sees, and it costs nothing: `result_parts`
/// already exists and already has six other callers.
fn fallible_parts(
    ty: &syn::Type,
    registry: &impl Conversions<KotlinMeta>,
) -> Option<(syn::Type, syn::Type)> {
    use crate::api::core::flat::TypeKind;
    if let Some(TypeKind::Fallible { ok, err }) = registry.flat().type_ref(ty).map(|t| t.kind()) {
        return Some((ok.syntax().clone(), err.syntax().clone()));
    }
    crate::api::core::types_util::result_parts(ty)
}
