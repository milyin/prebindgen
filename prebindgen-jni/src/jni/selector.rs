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
    /// The `Option<X>` **input** shape.
    ///
    /// Borrowed opaque handles are intercepted by the registry compiler and kept as
    /// a frozen non-owning-carrier plan; this legacy selector now handles only the
    /// structural Optional fallback.
    pub(crate) fn input_optional(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // What the converter YIELDS: this crossing's own reading, so a
        // `Box<Option<T>>` crossing produces a `Box<Option<T>>`.
        let produced = crate::jni::trait_impl::Produced::Reading(ty);
        let inner = ty.optional_inner()?;
        // A wrapped optional borrow has no value the fallback can rebuild. Letting
        // the shallow handler decode it would treat the jlong as a `*mut &T`, so
        // it still stops here rather than resolving wrong.
        if inner.borrow_target().is_some() && !ty.erased_wrappers().is_empty() {
            return None;
        }
        let mut c = self.input_wrapper_shape(WrapperShape::Optional, &produced, inner, emit)?;
        c.subs = vec![inner.key()];
        Some(c)
    }

    /// The `Vec<X>` / `&[X]` / `&Vec<X>` **input** shapes.
    ///
    /// A shared slice borrow has no owned `[T]` to decode into, so it reuses the
    /// `Vec<_>` shape: the Java `List<T>` becomes an owned `Vec<T>` and the call
    /// site borrows it (`&Vec<T>` deref-coerces to `&[T]`). `&mut [T]` is
    /// deliberately not supported, because the decoded `Vec` is never written
    /// back.
    pub(crate) fn input_run(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let produced = crate::jni::trait_impl::Produced::Reading(ty);
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(ty)) {
            let mut c = self.input_wrapper_shape(WrapperShape::Sequence, &produced, elem, emit)?;
            c.subs = vec![elem.key()];
            return Some(c);
        }
        // Two questions, and only the first is `kind`'s. That it is a run of
        // values makes this arm a *candidate*; whether the decoded `Vec<T>` can
        // be handed to the Rust function is a question about the **spelling**,
        // because the generated glue is the one consumer that can tell
        // `&Vec<T>` from `&Box<Vec<T>>`.
        let prebindgen_registry::flat::TypeKind::Ref { mutable, .. } = ty.unwrapped().kind() else {
            return None;
        };
        let inner = ty.borrow_target().expect("a borrow");
        if *mutable || !decoded_vec_satisfies(inner) {
            return None;
        }
        let elem = inner.sequence_elem()?;
        let elem_ty = emit.spell(elem);
        // The one place `produced` is NOT the crossing's spelling: there is no
        // owned `[T]` to decode into, so the converter yields an owned `Vec<T>`
        // and the call site borrows it.
        let produced = crate::jni::trait_impl::Produced::Composed(syn::parse_quote!(Vec<#elem_ty>));
        let mut c = self.input_wrapper_shape(WrapperShape::Sequence, &produced, elem, emit)?;
        c.subs = vec![elem.key()];
        Some(c)
    }

    /// `Result<T, E>` **output**: succeeds as `T`, routes `E` to the error sink.
    ///
    /// Read off the model, which calls this shape `TypeKind::Fallible`.
    pub(crate) fn result_shape(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        registry: &impl Conversions,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let (ok, err) = fallible_parts(ty, emit)?;
        self.result_peel(ty, &ok, &err, registry, emit)
    }

    /// The `Option<X>` **output** shape.
    ///
    /// An `Option<&Handle>` resolves via the shallow `Optional` whose inner
    /// conversion is the `&Handle` borrow's; there is no deep output handler.
    pub(crate) fn output_optional(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let produced = crate::jni::trait_impl::Produced::Reading(ty);
        let inner = ty.optional_inner()?;
        let mut c = self.output_wrapper_shape(WrapperShape::Optional, &produced, inner, emit)?;
        c.subs = vec![inner.key()];
        Some(c)
    }

    /// The `Vec<X>` / `&[X]` **output** shapes.
    ///
    /// A shared slice borrow is a callback argument crossing native to JVM: it
    /// builds a `List<T>` from the borrowed run. Dual of [`Self::input_run`],
    /// and split the same way — `kind` says it is a borrow of a run of values,
    /// and whether the generated Rust can iterate the borrow directly is a
    /// question about the spelling.
    pub(crate) fn output_run(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let produced = crate::jni::trait_impl::Produced::Reading(ty);
        if let Some(elem) = ty.sequence_elem().filter(|_| !is_unsized_spelling(ty)) {
            let mut c = self.output_wrapper_shape(WrapperShape::Sequence, &produced, elem, emit)?;
            c.subs = vec![elem.key()];
            return Some(c);
        }
        let prebindgen_registry::flat::TypeKind::Ref { mutable, .. } = ty.unwrapped().kind() else {
            return None;
        };
        let inner = ty.borrow_target().expect("a borrow");
        if *mutable || !decoded_vec_satisfies(inner) {
            return None;
        }
        let elem = inner.sequence_elem()?;
        self.output_slice(elem, emit)
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
