use prebindgen_registry::Conversions;

use super::*;

impl CbindgenBuilder {
    /// The alternatives of a declared sum, by **identity**.
    ///
    /// Off the element: an `Alternative` holds its fields already classified,
    /// so the callers that ask "does any payload own memory" read a `TypeRef`
    /// per field instead of a `syn::Field`. This took a node, took its last
    /// path segment, and fetched the `syn::ItemEnum` to reach the same list.
    pub(super) fn enum_alternatives<'r>(
        &self,
        registry: &'r Registry,
        key: &TypeKey,
    ) -> Option<&'r [prebindgen_registry::flat::Alternative]> {
        match registry.flat().declared_type(&key.ident()?)? {
            prebindgen_registry::flat::Type::Variant(v) => Some(&v.alternatives),
            _ => None,
        }
    }

    /// The declared `opaque_ptr` under a union payload's spelling, when there is
    /// one and the spelling does **not** already carry a `Box` —
    /// `Option<Handle>` / `Handle` → `Some(Handle)`, `Option<Box<Handle>>` →
    /// `None` (that shape keeps its own arm, so its emitted Rust does not move).
    ///
    /// Keyed on `stripped_key`, because a **declaration** is about the type: a
    /// wrapper the model erases cannot change which declaration a payload
    /// matches. See [`Self::payload_field_wire`] for why a union payload asks
    /// this at all where a `repr_c_struct` mirror must not.
    pub(super) fn declared_opaque_payload_inner(&self, fty: &TypeRef) -> Option<TypeKey> {
        if r_boxed_inner(fty).is_some() {
            return None;
        }
        let core = fty.optional_inner().unwrap_or(fty);
        let key = core.stripped_key();
        // The IDENTITY, not the stripped spelling: every caller turned that
        // spelling straight back into this key (`c_type_ident`, `type_short`)
        // or into the source path it names (`src_ty_of`).
        self.opaque.contains_key(&key).then_some(key)
    }

    /// Wire type of one **tagged-union payload field**: the
    /// [`Self::mirror_field_wire`] policy (scalar / declared `enum_type` /
    /// opaque pointer `Option<Box<T>>` / `Box<T>`) extended with `String` →
    /// a malloc'd `*mut c_char`, the same `char *` lowering a `data_struct`
    /// field gets. `None` ⇒ the payload type cannot cross a tagged union.
    ///
    /// Unlike a `repr_c_struct` mirror — whose fields are reinterpreted
    /// wholesale by one `Transmute` — a tagged union is rebuilt arm by arm,
    /// so each wire here is produced by a real per-field conversion. That is
    /// what lets `String` join the set.
    ///
    /// Every wire this returns is **bit-pattern-agnostic**: integers, floats and
    /// raw pointers hold any bits legally, and the two Rust types that do *not*
    /// — a declared `enum_type` (a discriminant no variant has) and `bool`
    /// (anything but `0`/`1`) — are wrapped in [`::core::mem::MaybeUninit`] so
    /// they do too. That is what makes the mirror's tag the *only* thing
    /// [`CbindgenBuilder::in_tagged_union`] has to validate before `assume_init`.
    ///
    /// A `bool` reached through a nested `data_struct` payload is covered by
    /// the same [`bool_wire`] policy, which [`c_field_wire`] and the plain
    /// `bool` parameter now share (#170).
    /// Why this type can **never** be a union payload, whatever converts it.
    ///
    /// The half of [`Self::payload_field_wire`] that is a fact about the union
    /// rather than about the conversion: one union field carries one C wire, so
    /// a shape needing two cannot ride in one however well it converts. The
    /// other half — "no resolved converter" — stops being a question once the
    /// payload is composed from its own part, which is a conversion in hand.
    pub(super) fn payload_shape_refusal(&self, fty: &TypeRef) -> Result<(), String> {
        if r_is_vec(fty) {
            return Err(
                "a `Vec` needs TWO C wires (pointer + length) and one union field carries only \
                 one, so its length would be silently dropped — hand the sequence over through \
                 a separate function, or wrap it in a declared `opaque_ptr` handle"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub(super) fn payload_field_wire(&self, fty: &TypeRef) -> Result<syn::Type, String> {
        // `String` is the one type whose two directions disagree on the wire
        // (`*const c_char` in, `*mut c_char` out), so the union field fixes the
        // OWNING form and the per-arm expressions convert by hand.
        if r_is_string(fty) {
            return Ok(syn::parse_quote!(*mut ::core::ffi::c_char));
        }
        if self.enums.contains_key(&fty.key()) {
            let c = self.c_type_ident(&fty.key());
            return Ok(syn::parse_quote!(::core::mem::MaybeUninit<#c>));
        }
        // `bool` is the one scalar with a restricted domain: `2` is a byte a C
        // caller can write into the union and NOT a Rust `bool`, so holding it
        // in the mirror is the same UB an out-of-range discriminant is. Same
        // remedy everywhere C writes a `bool` — see `bool_wire`.
        if r_is_bool(fty) {
            return Ok(bool_wire());
        }
        // A `Vec` payload needs TWO C wires (pointer + length) and one union
        // field can carry only one, so its length would be silently dropped.
        // Rejected explicitly, because the converter-destination rule below
        // would otherwise hand back the pointer alone and look like it worked.
        if r_is_vec(fty) {
            return Err(
                "a `Vec` needs TWO C wires (pointer + length) and one union field carries only \
                 one, so its length would be silently dropped — hand the sequence over through \
                 a separate function, or wrap it in a declared `opaque_ptr` handle"
                    .to_string(),
            );
        }
        // Layout-identical shapes first (scalar, `Box<T>`/`Option<Box<T>>`
        // opaque pointer), so what already worked keeps its exact wire.
        if let Some(w) = self.mirror_field_wire(fty) {
            return Ok(w);
        }
        // The opaque-pointer arm again, keyed on the **declaration** instead of
        // on the spelling — which is what a *converted* position must do.
        //
        // `mirror_field_wire` above answers for a `repr_c_struct`, where the C
        // type is a **layout** fact: the mirror is reinterpreted from the source
        // struct's bytes, so `Box<T>` (a pointer) and `T` (inline) genuinely are
        // different C types and the spelling is load-bearing. A union payload is
        // not mirrored — it is rebuilt arm by arm through real conversions — so
        // that reasoning does not carry over, and reusing the same
        // `Box`-in-the-spelling test made an erased wrapper decide what C sees.
        //
        // Concretely: `Option<Box<Handle>>` crossed as `*mut handle_t` while
        // `Option<Handle>` — the same optional handle to every destination
        // language — was REFUSED, because it fell through to the
        // converter-destination rule below where its output side is a structural
        // marker (`()`) that cannot agree with the input's pointer. A wrapper the
        // model erases decided whether the shape was expressible at all.
        //
        // So: peel the optional off the model and ask whether what is under it is
        // a declared `opaque_ptr`. `stripped_key` rather than `key`, because a
        // declaration is about the TYPE — see the same rule on the JNI side
        // (#292). The two spellings now share this C type; their converter bodies
        // differ, which is exactly the split (`kind` decides what C sees, syntax
        // decides how the value is built).
        if let Some(inner) = self.declared_opaque_payload_inner(fty) {
            let c = self.c_type_ident(&inner);
            return Ok(syn::parse_quote!(*mut #c));
        }
        // Otherwise the payload's wire is its **resolved converter
        // destination** — the same source a `data_struct` field effectively
        // uses. A union is rebuilt arm by arm through real per-field
        // conversions, so its payloads are not constrained to the
        // layout-preserving shapes a `repr_c_struct` mirror needs; this is
        // what admits a nested `data_struct`, a bare `opaque_ptr` handle, and
        // a converted leaf (`Duration` → `u64`).
        //
        // One field serves both directions, so they must agree on it. They can
        // legitimately differ (a `String`'s const-ness above), which is why a
        // disagreement is `None` — a rejection naming the payload — rather
        // than a silent pick of one side.
        let out_entry = self.out_frag(fty).ok_or_else(|| {
            "no resolved OUTPUT converter — a payload crosses as its converter's destination, so \
             it must be a scalar, a `String`, or a type this binding declares (`enum_type`, \
             `data_struct`, `opaque_ptr`, or a `convert!` conversion)"
                .to_string()
        })?;
        // Decided here rather than at the emit site, so every reason a payload
        // can be refused is reported from ONE place, at the declaration.
        if out_entry.function.call().fallible() {
            return Err(
                "its OUTPUT converter is fallible, but a union is encoded without an error \
                 channel — the encoder always writes a live arm, so there is nowhere for the \
                 failure to go. Use an infallible conversion for this payload"
                    .to_string(),
            );
        }
        let out = out_entry.destination.clone();
        if let Some(inp) = self.in_frag(fty) {
            if TypeKey::from_type(&inp.destination) != TypeKey::from_type(&out) {
                return Err(format!(
                    "its input and output converters disagree on the wire (`{}` in, `{}` out) \
                     and one union field serves both directions",
                    inp.destination.to_token_stream(),
                    out.to_token_stream(),
                ));
            }
        }
        Ok(out)
    }

    /// Wire type of a `data_struct` field: the free [`c_field_wire`] policy
    /// (`String` → `char *`, scalar → itself) plus a declared
    /// [`CbindgenBuilder::tagged_union`] field, which crosses **by value** as its
    /// `#[repr(C)]` mirror — the same way it crosses as a parameter or a
    /// return. `None` ⇒ the field type is unsupported in a data struct.
    ///
    /// A union field with an owning payload keeps the data struct's existing
    /// contract: the struct has no destructor, and each owning field is
    /// released individually — here through the union's own typed drop.
    ///
    /// Like a union **parameter**, the field is wrapped in
    /// [`::core::mem::MaybeUninit`]: one mirror struct serves both directions,
    /// and on the way in its bytes are C's, so the field may not be a Rust enum
    /// until its tag has been validated. Invisible in C either way.
    pub(super) fn data_field_wire(&self, fty: &TypeRef) -> Option<syn::Type> {
        if self.tagged_unions.contains_key(&fty.key()) {
            let c = self.c_type_ident(&fty.key());
            return Some(syn::parse_quote!(::core::mem::MaybeUninit<#c>));
        }
        if r_is_string(fty) {
            return Some(syn::parse_quote!(*mut ::core::ffi::c_char));
        }
        // #170 instance 2: the field arrives from C by value, so it may not be
        // a Rust `bool` until the byte has been normalised.
        if r_is_bool(fty) {
            return Some(bool_wire());
        }
        scalar_ty(fty)
    }

    /// True when a payload wire hands owned memory to C — a `char *` block or
    /// an opaque pointer — and therefore has to be released by the union's
    /// typed drop.
    ///
    /// A nested `data_struct` payload crosses BY VALUE, so the wire itself is
    /// not a pointer, but its mirror's own fields may be: the union's drop
    /// then has to reach through and release each of them through the recursive
    /// payload-cleanup artifact plan. Without this a `String` or handle
    /// inside a struct payload would leak, silently, for exactly the shape
    /// zenoh-flat#30 needs.
    pub(super) fn payload_wire_owns(
        &self,
        fty: &TypeRef,
        wire: &syn::Type,
        registry: &Registry,
    ) -> bool {
        if matches!(wire, syn::Type::Ptr(_)) {
            return true;
        }
        !self.owning_data_struct_fields(fty, registry).is_empty()
    }

    /// The `(name, type)` of every field of a declared `data_struct` whose own
    /// C wire owns memory. Empty when `fty` is not a declared data struct.
    pub(super) fn owning_data_struct_fields<'r>(
        &self,
        fty: &TypeRef,
        registry: &'r Registry,
    ) -> Vec<(syn::Ident, &'r TypeRef)> {
        if !self.data.contains_key(&fty.key()) {
            return Vec::new();
        }
        self.struct_fields(registry, &fty.key())
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, fty)| self.data_field_owns(fty, registry))
            .collect()
    }

    /// Whether one `data_struct` **field** hands owned memory to C: its own wire
    /// is a pointer (`String` → `char *`), or it is a declared
    /// [`CbindgenBuilder::tagged_union`] with an owning arm — which crosses by value,
    /// so the pointer it owns is one level further down.
    fn data_field_owns(&self, fty: &TypeRef, registry: &Registry) -> bool {
        if matches!(self.data_field_wire(fty), Some(syn::Type::Ptr(_))) {
            return true;
        }
        self.tagged_union_has_drop(fty, registry)
    }

    /// Whether a declared `tagged_union` gets a typed `<base>_drop` — i.e. it is
    /// produced at all, and some arm's payload owns memory.
    ///
    /// This is the emission condition of the tagged-union artifact's drop *and*
    /// the test for whether a
    /// containing struct has to call it, so a union nested inside a payload
    /// cannot be freed through a symbol that was never emitted. `false` for
    /// anything that is not a declared tagged union.
    pub(super) fn tagged_union_has_drop(&self, fty: &TypeRef, registry: &Registry) -> bool {
        if !self.tagged_unions.contains_key(&fty.key()) || self.out_frag(fty).is_none() {
            return false;
        }
        self.enum_alternatives(registry, &fty.key())
            .unwrap_or_default()
            .iter()
            .flat_map(|a| a.fields.iter())
            .any(|f| match self.payload_field_wire(&f.ty) {
                // A rejected payload is reported from the emission site, which
                // panics before any of this matters.
                Err(_) => false,
                Ok(wire) => self.payload_wire_owns(&f.ty, &wire, registry),
            })
    }

    /// Fields (`name`, `type`) of a declared data struct, looked up from the
    /// registry's indexed structs. `None` if the type isn't an indexed named
    /// struct.
    pub(super) fn struct_fields<'r>(
        &self,
        registry: &'r impl Conversions,
        key: &TypeKey,
    ) -> Option<Vec<(syn::Ident, &'r TypeRef)>> {
        // The element, not its item. A `Struct` holds the field list the
        // `syn::Fields::Named` match used to dig out, each field's name beside
        // the reading of its type — so the name lookup is the key's own ident
        // and the positional case is `f.name` being `None` rather than a
        // `Fields` variant.
        let st = registry.flat().struct_type(&key.ident()?)?;
        st.fields
            .iter()
            .map(|f| Some((f.name.clone()?, &f.ty)))
            .collect()
    }

    /// Wire type of a `repr_c_struct` field in the generated **visible** mirror: a
    /// scalar passes through; a declared [`CbindgenBuilder::enum_type`] becomes its C enum;
    /// an opaque pointer `Option<Box<T>>` / `Box<T>` (with `T` a declared
    /// [`CbindgenBuilder::opaque_ptr`]) becomes `*mut t_t`. The whole-struct `Transmute`
    /// (size/align-equal, asserted) then reinterprets each source field's bits into
    /// this wire. `None` ⇒ the field type is unsupported in a `repr_c_struct`.
    ///
    /// Note what this policy cannot express: the wire *is* the source field's
    /// type, so a field whose Rust type has restricted validity is reinterpreted
    /// from C's bytes with no chance to check them. That gap is audited
    /// separately by [`Self::restricted_validity_field`] — this function keeps
    /// answering what the layout is.
    pub(super) fn mirror_field_wire(&self, fty: &TypeRef) -> Option<syn::Type> {
        if let Some(t) = scalar_ty(fty) {
            return Some(t);
        }
        if self.enums.contains_key(&fty.key()) {
            let c = self.c_type_ident(&fty.key());
            return Some(syn::parse_quote!(#c));
        }
        // Opaque pointer: `Option<Box<T>>` (nullable, null-niche ↔ NULL) or `Box<T>`
        // where `T` is a declared `opaque_ptr` → `*mut t_t`.
        if let Some(inner) = r_boxed_inner(fty) {
            if self.opaque.contains_key(&inner.key()) {
                let c = self.c_type_ident(&inner.key());
                return Some(syn::parse_quote!(*mut #c));
            }
        }
        None
    }

    /// Why a `repr_c_struct` mirror field is **not** safe to reinterpret from
    /// C-supplied bytes, or `None` when every bit pattern of its wire is a valid
    /// value of the source field's type.
    ///
    /// Integers, floats and raw pointers hold any bits legally (holding a
    /// garbage pointer is sound; dereferencing it is the caller's contract).
    /// Two Rust types do not: `bool`, whose domain is `0`/`1` (#170 instance 3),
    /// and a declared `enum_type`, whose domain is the declared discriminants
    /// (#158 instance 3).
    ///
    /// The other positions these types cross — a parameter, a `data_struct`
    /// field, a tagged-union payload — all go through a *per-value* converter,
    /// so each has a hook: `bool` normalises through [`bool_wire`], an enum
    /// validates its discriminant in `in_enum`. A `repr_c_struct` has no such
    /// hook: one whole-struct `Transmute` reinterprets the mirror into the
    /// source type, and by the time any generated code could look, the invalid
    /// value already exists. Wrapping the mirror field in `MaybeUninit` moves
    /// the problem rather than solving it — the transmute's *output* still has
    /// the real field.
    pub(super) fn restricted_validity_field(&self, fty: &TypeRef) -> Option<&'static str> {
        if r_is_bool(fty) {
            return Some("`bool` — only `0` and `1` are valid");
        }
        if self.enums.contains_key(&fty.key()) {
            return Some("a declared `enum_type` — only its declared discriminants are valid");
        }
        None
    }

    /// The audit of [`Self::restricted_validity_field`] over one
    /// `repr_c_struct`'s mirror: the offending `(field, reason)` pairs, in
    /// declaration order. Empty ⇒ the whole mirror is safe to reinterpret.
    pub(super) fn restricted_validity_fields(
        &self,
        registry: &Registry,
        key: &TypeKey,
    ) -> Vec<(syn::Ident, &'static str)> {
        self.struct_fields(registry, key)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(fname, fty)| {
                self.restricted_validity_field(fty)
                    .map(|reason| (fname, reason))
            })
            .collect()
    }

    /// Exported `#[no_mangle]` symbol for a declared function:
    /// [`Self::mangle_function`] over the base — a `.base_name(...)` override when
    /// set, else the Rust fn ident — or that base verbatim when no mangler is set.
    pub(super) fn fn_symbol(&self, orig: &syn::Ident) -> syn::Ident {
        let base = self
            .functions
            .get(orig)
            .and_then(|c| c.base.clone())
            .unwrap_or_else(|| orig.to_string());
        match &self.mangle_function {
            Some(f) => format_ident!("{}", f(&base)),
            None => format_ident!("{}", base),
        }
    }

    /// How one parameter *uses* the resource it names — the axis the alias rule
    /// is stated on. `None` ⇒ the parameter names no single owned resource (a
    /// scalar, a string, a slice block, an undeclared type), so it cannot alias
    /// one.
    ///
    /// `T` and `Option<T>` share a resource domain: both arrive as the same
    /// handle pointer, and comparing the syntactic parameter type instead would
    /// miss `f(x: ZThing, y: Option<ZThing>)` called with the same handle
    /// twice.
    /// [`Self::alias_slot`] off the classification: the optional peel, the
    /// borrow and its mutability, and the `MaybeUninit` under a `&mut` are all
    /// what `TypeKind` states — where this walked `Option`'s type argument, a
    /// `syn::Type::Reference` and its `mutability` field.
    pub(super) fn alias_slot_of(&self, ty: &TypeRef) -> Option<(TypeKey, AliasAccess)> {
        let inner = ty.optional_inner().unwrap_or(ty);
        let declared =
            |key: &TypeKey| self.opaque.contains_key(key) || self.value_opaque.contains_key(key);
        if let TypeKind::Ref { mutable, inner, .. } = inner.kind() {
            // `&mut MaybeUninit<T>` borrows T's slot exclusively just as
            // `&mut T` does; the wire is the same pointer.
            let elem = match inner.kind() {
                TypeKind::Uninit(t) => t,
                _ => inner,
            };
            let key = elem.key();
            if !declared(&key) {
                return None;
            }
            return Some((
                key,
                if *mutable {
                    AliasAccess::Exclusive
                } else {
                    AliasAccess::Shared
                },
            ));
        }
        let key = inner.key();
        declared(&key).then_some((key, AliasAccess::Consume))
    }
}
