use prebindgen_registry::Conversions;

use super::*;

impl CbindgenBuilder {
    /// Whether the generated layer hands `char*` data memory to C — a `String`
    /// return value, or a declared data struct that is produced as output and has
    /// a `String` field. When true, a `free_memory_function` must be declared.
    pub(super) fn needs_free(&self, registry: &Registry) -> bool {
        let string_ty: syn::Type = syn::parse_quote!(String);
        // A `String` return hands out a `char*` — unless `String` is declared
        // `opaque_ptr` (then it crosses as `string_t *`, freed by `string_drop`).
        if registry
            .reading_of(&string_ty)
            .and_then(|tr| self.out_frag(&tr))
            .is_some()
            && !self.opaque.contains_key(&TypeKey::from_type(&string_ty))
        {
            return true;
        }
        // Opaque error types are marshalled to a malloc'd `char*` message.
        if self.opaque_errors.keys().any(|key| {
            registry
                .reading(key)
                .and_then(|tr| self.out_frag(&tr))
                .is_some()
        }) {
            return true;
        }
        // A tagged union with a `String` payload hands out a `char*` per active
        // arm — allocated by its output converter, released by its typed drop.
        if self.tagged_unions.keys().any(|key| {
            let Some(reading) = registry.reading(key) else {
                return false;
            };
            self.out_frag(&reading).is_some()
                && self
                    .enum_alternatives(registry, key)
                    .map(|alts| {
                        alts.iter().flat_map(|a| a.fields.iter()).any(|f| {
                            matches!(f.ty.kind(), prebindgen_registry::flat::TypeKind::String)
                        })
                    })
                    .unwrap_or(false)
        }) {
            return true;
        }
        self.data.keys().any(|key| {
            let Some(reading) = registry.reading(key) else {
                return false;
            };
            self.out_frag(&reading).is_some()
                && self
                    .struct_fields(registry, key)
                    .map(|fields| fields.iter().any(|(_, fty)| r_is_string(fty)))
                    .unwrap_or(false)
        })
    }

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
    /// then has to reach through and release each of them (see
    /// [`CbindgenBuilder::payload_free_stmt`]). Without this a `String` or handle
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
    /// This is the emission condition of that drop
    /// ([`CbindgenBuilder::prereq_tagged_unions`]) *and* the test for whether a
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

    /// Whether any declared function returns a `Vec<_>` (possibly nested under
    /// `Result`/`Option`), so the array builder/freer prelude must be emitted.
    ///
    /// A run of values is the whole question — `Vec<T>` and `[T]` alike, and
    /// through a transparent wrapper, so `Cow<'_, [T]>` counts as the `Vec<T>`
    /// it crosses as. [`sequence_elem`](prebindgen_registry::flat::TypeRef::sequence_elem)
    /// answers all three, which is why the two spellings this used to test
    /// separately need no arms of their own.
    pub(super) fn produces_array(&self, registry: &Registry) -> bool {
        self.functions.keys().any(|orig| {
            registry
                .flat()
                .function(&orig)
                // The model already decided that an elided return and `-> ()`
                // are one thing, so there is no second arm to write here.
                .map(|f| f.ret.walk().iter().any(|t| t.sequence_elem().is_some()))
                .unwrap_or(false)
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

    /// Assemble the `#[no_mangle] extern "C"` wrapper for one declared fn.
    pub(super) fn emit_function_wrapper(
        &self,
        f: &prebindgen_registry::flat::Function,
        registry: &Registry,
        emit: &prebindgen_registry::Emit,
    ) -> TokenStream {
        let orig = &f.name;
        let call_path = self.src_fn(orig);
        let sym = self.fn_symbol(orig);

        // The ELEMENT: a signature is a parameter list and a return, both
        // already classified. An elided return is `TypeKind::Unit`, which is
        // the `ReturnType::Default` arm this used to write.

        let has_fallible_input = f.params.iter().any(|p| {
            self.in_frag(&p.ty)
                .map(|e| e.function.call().fallible())
                .unwrap_or(false)
        });

        // Peel an outer `Result<_, E>`; `value_ty` is the success/return value.
        // Off `TypeKind::Fallible`, where `result_parts` found the `Result` in a
        // path first — and both sides come back as readings, so everything
        // downstream of here reads too.
        let (value_ty, err_reading) = match f.ret.fallible_parts() {
            Some((ok, e)) => (ok, Some(e)),
            None => (&f.ret, None),
        };
        let err_ty: Option<syn::Type> = err_reading.map(|t| spelled(t, emit));
        let has_fallible_output = self.output_is_fallible(value_ty);

        // Error wiring: the error type must be declared via `.error()`.
        let err_bits = err_ty.as_ref().map(|err_ty| {
            assert!(
                self.error.contains(&TypeKey::from_type(err_ty)),
                "Cbindgen: function `{}` returns `Result<_, {}>` but `{}` is not a \
                 declared error type — add `.data_struct({}).error()`",
                orig,
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
            );
            let entry = registry
                .reading_of(err_ty)
                .and_then(|tr| self.out_frag(&tr))
                .unwrap_or_else(|| {
                    panic!(
                        "Cbindgen::on_function: error type `{}` of `{}` has no output converter",
                        TypeKey::from_type(err_ty),
                        orig
                    )
                });
            (
                entry.destination.clone(),
                entry.function.call().ident().clone(),
                self.src_ty(err_ty),
            )
        });

        // No `Result` channel ⇒ a fallible input must be declared `.panic()`.
        if err_ty.is_none() {
            let allows_panic = self.functions.get(orig).map(|c| c.panic).unwrap_or(false);
            assert!(
                !(has_fallible_input || has_fallible_output) || allows_panic,
                "Cbindgen: function `{}` has a fallible binding conversion but does not \
                 return `Result`; add \
                 `.panic()` after its `.function(...)` declaration to allow aborting \
                 on the internal error, or change its signature",
                orig,
            );
        }

        // Structural lowering of the (present/ok) value, then the null-niche rule:
        //   * Result + a free pointer niche  → NULL marks `Err` (value in-band);
        //   * Result without a free niche     → `bool` status, value to out-params;
        //   * no Result                       → field 0 is the C return, rest out.
        let shape = self.lower_shape(value_ty, registry);
        let result_slot = shape.niches.clone().carve().map(|(slot, _)| slot);
        let result_in_band = err_ty.is_some() && result_slot.is_some();
        let field0_is_return = result_in_band || err_ty.is_none();

        // Partition fields into the (optional) C return value + out-parameters,
        // and pick C names for the out-params (see `out_param_name`).
        let mut targets: Vec<TokenStream> = Vec::new();
        let mut out_fields: Vec<&WireField> = Vec::new();
        // `field0_wire` is the wire of the value's primary field when that field
        // is carried by the C return slot (modes A/D); `None` for mode B and unit.
        let field0_wire: Option<syn::Type> = if field0_is_return {
            shape.fields.first().map(|f| f.wire.clone())
        } else {
            None
        };
        if field0_is_return {
            if !shape.fields.is_empty() {
                targets.push(quote!(__ret));
                out_fields.extend(shape.fields[1..].iter());
            }
        } else {
            out_fields.extend(shape.fields.iter());
        }
        let prefixed = out_fields.iter().any(|wf| wf.suffix.is_empty());
        let out_names: Vec<syn::Ident> = out_fields
            .iter()
            .map(|wf| out_param_name(wf.suffix, prefixed))
            .collect();
        for name in &out_names {
            targets.push(quote!(*#name));
        }
        let out_param_decls: Vec<TokenStream> = out_fields
            .iter()
            .zip(&out_names)
            .map(|(wf, name)| {
                let wire = &wf.wire;
                quote!(#name: *mut #wire)
            })
            .collect();

        // C wrapper return type: the payload's field 0 (modes A/D), `bool` status
        // (mode B), or `void` (a unit value with no `Result`).
        let c_return: Option<syn::Type> = if field0_is_return {
            field0_wire.clone()
        } else {
            Some(syn::parse_quote!(bool))
        };

        // Input decode: route a fallible-input failure to the error out-param
        // (with the wrapper's fail value) when there is a `Result`, else panic.
        let fail_return = if result_in_band {
            let slot = result_slot.as_ref().expect("in-band result has a niche");
            let value = &slot.value;
            quote!(#value)
        } else {
            quote!(false)
        };
        let input_route = match &err_bits {
            Some((_, e_conv, e_ty_src)) => ErrRoute::Result {
                e_conv,
                e_ty_src: e_ty_src.clone(),
                fail_return: fail_return.clone(),
            },
            None => ErrRoute::Panic,
        };
        let (in_params, decodes, call_args) =
            self.emit_inputs(orig, f, registry, &input_route, emit);
        let call = quote!(#call_path(#(#call_args),*));

        let e_param = err_bits
            .as_ref()
            .map(|(err_wire, _, _)| quote!(e: *mut #err_wire));
        let ret_arrow = c_return.as_ref().map(|w| quote!(-> #w));

        // Assemble the body per the three structural modes.
        let body = match (&err_bits, field0_is_return) {
            // No `Result`: straight-line. `void` when there are no fields.
            (None, _) => {
                if let Some(field0_wire) = field0_wire.as_ref() {
                    let enc =
                        self.encode_value(value_ty, quote!(__v), &targets, registry, &input_route);
                    quote!(
                        #(#decodes)*
                        let __v = #call;
                        let __ret: #field0_wire;
                        #enc
                        __ret
                    )
                } else {
                    quote!( #(#decodes)* #call; )
                }
            }
            // `Result` with a free niche: value in-band, NULL marks `Err`.
            (Some((_, e_conv, _)), true) => {
                let field0_wire = field0_wire.as_ref().expect("in-band ⇒ pointer return");
                let null = &result_slot
                    .as_ref()
                    .expect("in-band result has a niche")
                    .value;
                let enc =
                    self.encode_value(value_ty, quote!(__v), &targets, registry, &input_route);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { let __ret: #field0_wire; #enc __ret }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            #null
                        }
                    }
                )
            }
            // `Result` without a free niche: `bool` status, value to out-params.
            (Some((_, e_conv, _)), false) => {
                let enc =
                    self.encode_value(value_ty, quote!(__v), &targets, registry, &input_route);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { #enc true }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            false
                        }
                    }
                )
            }
        };

        quote! {
            #[no_mangle]
            #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
            pub unsafe extern "C" fn #sym(
                #(#in_params,)*
                #(#out_param_decls,)*
                #e_param
            ) #ret_arrow {
                #body
            }
        }
    }

    /// Lower how a *present / ok* value of `ty` is carried over the C ABI: an
    /// ordered list of wire components plus the representation niches still
    /// available for enclosing `Option`/`Result` layers. Mirrors the
    /// niche-stacking model in `core::niches`.
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn lower_shape(&self, ty: &TypeRef, registry: &impl Conversions) -> ValueShape {
        if matches!(ty.kind(), TypeKind::Unit) {
            return ValueShape {
                fields: vec![],
                niches: Niches::empty(),
            };
        }
        // A DECLARED conversion beats the shape, at every level and not only the
        // outermost. `select_output_type` tries `out_custom` before
        // `out_wrappers`, so a `convert!`-declared `Option<T>` has a wire of its
        // own; decomposing it anyway would describe a different ABI from the one
        // the converter table hands out, and the two must agree (#428 review).
        // The base case below already does this for a type with no shape arm.
        if !self.has_own_wire(ty) {
            // `Vec<T>` → `T_wire* + size_t`. The element must lower to a single C
            // value (one converter); a composite element is unsupported.
            // `TypeKind::Vec` and not `sequence_elem`: that reading peels the
            // erased wrappers first, so a `Cow<'_, [u8]>` would answer here and
            // take the `Vec` lowering instead of its own arm below.
            if let TypeKind::Vec(elem) = ty.kind() {
                let entry = self.out_frag(elem).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: `Vec` element `{}` has no output converter",
                        elem.key()
                    )
                });
                // The element must lower to ONE C value, and that is the converter
                // table's answer rather than a list of shapes: a marker destination
                // means "no wire of its own", whatever put it there — an `Option`, a
                // shared slice, another run, the unit. A composite element WITH a
                // declared conversion has a wire and is fine.
                assert!(
                    !marker_destination(&entry.destination),
                    "Cbindgen: `Vec<{}>` element has no wire of its own, so there is \
                 nothing for the array to hold — give it a `convert!` \
                 declaration or deliver its parts separately",
                    elem.key(),
                );
                let elem_wire = entry.destination.clone();
                return ValueShape {
                    fields: vec![
                        WireField {
                            suffix: "",
                            wire: syn::parse_quote!(*mut #elem_wire),
                        },
                        WireField {
                            suffix: "_len",
                            wire: syn::parse_quote!(usize),
                        },
                    ],
                    niches: Niches::empty(),
                };
            }
            // `Cow<'_, [T]>` and `&[T]` → `T_wire* + size_t`. The C side receives an
            // owned malloc'd copy, just like `Vec<T>` outputs.
            //
            // A shared slice reaches this only as a RETURN: a slice parameter is a
            // pointer pair the caller supplies, and a slice callback argument is
            // lowered by `prereq_callback_structs`. Without the second predicate a
            // slice return fell through to the base-value path and took the `()`
            // destination of the marker converter that exists for that callback
            // lowering, so the wrapper returned nothing and called the marker with
            // an argument it does not take (#413).
            if let Some(elem) = r_cow_slice_elem(ty).or_else(|| r_scalar_slice_elem(ty)) {
                let entry = self.out_frag(elem).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: `Cow` slice element `{}` has no output converter",
                        elem.key()
                    )
                });
                let elem_wire = entry.destination.clone();
                return ValueShape {
                    fields: vec![
                        WireField {
                            suffix: "",
                            wire: syn::parse_quote!(*mut #elem_wire),
                        },
                        WireField {
                            suffix: "_len",
                            wire: syn::parse_quote!(usize),
                        },
                    ],
                    niches: Niches::empty(),
                };
            }
            // `Option<T>` consumes one available inner niche. This includes NULL
            // pointers and invalid scalar values declared by `convert!`; without a
            // niche it prepends an explicit `present: bool`.
            if let Some(inner_ty) = ty.optional_inner() {
                let inner = self.lower_shape(inner_ty, registry);
                if let Some((_slot, rest)) = inner.niches.clone().carve() {
                    return ValueShape {
                        fields: inner.fields,
                        niches: rest,
                    };
                }
                let mut fields = vec![WireField {
                    suffix: "_present",
                    wire: syn::parse_quote!(bool),
                }];
                fields.extend(inner.fields);
                return ValueShape {
                    fields,
                    niches: Niches::empty(),
                };
            }
        }
        // Base value: one wire component from its rank-0/1 converter. Custom
        // conversions may declare scalar niches; otherwise a pointer wire
        // (String, opaque handle, `&'static`) carries a free NULL niche.
        let entry = self.out_frag(ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: type `{}` has no output converter",
                ty.key()
            )
        });
        let wire = entry.destination.clone();
        let niches = if entry.niches.is_empty() && matches!(wire, syn::Type::Ptr(_)) {
            let null = null_for(&wire);
            Niches::one(syn::parse_quote!(#null), syn::parse_quote!(v.is_null()))
        } else {
            entry.niches.clone()
        };
        ValueShape {
            fields: vec![WireField { suffix: "", wire }],
            niches,
        }
    }

    /// Emit the statements that write a native value `val` of type `ty` into the
    /// `targets` lvalues (one per field of `lower_shape(ty)`, in order).
    pub(super) fn encode_value(
        &self,
        ty: &TypeRef,
        val: TokenStream,
        targets: &[TokenStream],
        registry: &impl Conversions,
        route: &ErrRoute,
    ) -> TokenStream {
        if matches!(ty.kind(), TypeKind::Unit) {
            return quote!();
        }
        // The peer of `lower_shape`'s guard: a node with a wire of its own is
        // encoded by its own converter, whatever its shape. The two walk the
        // same value and must stop at the same places (#428 review).
        if !self.has_own_wire(ty) {
            if let TypeKind::Vec(elem) = ty.kind() {
                let entry = self.out_frag(elem).expect("Vec element converter");
                let elem_conv = entry.function.call().ident().clone();
                let elem_map = map_arg(&elem_conv, entry.function.call().unsafe_());
                let elem_wire = entry.destination.clone();
                let t_ptr = &targets[0];
                let t_len = &targets[1];
                if entry.function.call().fallible() {
                    let converted = route_result(quote!(#elem_conv(__value)), route);
                    return quote!(
                        let mut __arr: ::std::vec::Vec<#elem_wire> = ::std::vec::Vec::new();
                        for __value in #val {
                            __arr.push(#converted);
                        }
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    );
                } else {
                    return quote!(
                        let __arr: ::std::vec::Vec<#elem_wire> =
                            #val.into_iter().map(#elem_map).collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    );
                }
            }
            if let Some(elem) = r_cow_slice_elem(ty).or_else(|| r_scalar_slice_elem(ty)) {
                let entry = self.out_frag(elem).expect("slice element converter");
                let elem_conv = entry.function.call().ident().clone();
                let elem_map = map_arg(&elem_conv, entry.function.call().unsafe_());
                let elem_wire = entry.destination.clone();
                let t_ptr = &targets[0];
                let t_len = &targets[1];
                if entry.function.call().fallible() {
                    let converted = route_result(quote!(#elem_conv(__value)), route);
                    return quote!(
                        let mut __arr: ::std::vec::Vec<#elem_wire> = ::std::vec::Vec::new();
                        for __value in #val.iter().copied() {
                            __arr.push(#converted);
                        }
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    );
                } else {
                    return quote!(
                        let __arr: ::std::vec::Vec<#elem_wire> =
                            #val.iter().copied().map(#elem_map).collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #t_ptr = __p;
                        #t_len = __n;
                    );
                }
            }
            if let Some(inner_ty) = ty.optional_inner() {
                let inner = self.lower_shape(inner_ty, registry);
                if let Some((slot, _rest)) = inner.niches.clone().carve() {
                    // None reuses the next inner niche; Some encodes inline.
                    let inner_enc =
                        self.encode_value(inner_ty, quote!(__x), targets, registry, route);
                    let null = &slot.value;
                    let t0 = &targets[0];
                    return quote!(
                        match #val {
                            ::core::option::Option::Some(__x) => { #inner_enc }
                            ::core::option::Option::None => { #t0 = #null; }
                        }
                    );
                }
                // Explicit `present` flag in targets[0]; inner value follows.
                let present = &targets[0];
                let inner_enc =
                    self.encode_value(inner_ty, quote!(__x), &targets[1..], registry, route);
                return quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => { #present = true; #inner_enc }
                        ::core::option::Option::None => { #present = false; }
                    }
                );
            }
        }
        // Base value: run its output converter into the single target.
        let entry = self.out_frag(ty).expect("base value converter");
        let conv = entry.function.call().ident().clone();
        let t0 = &targets[0];
        if entry.function.call().fallible() {
            let converted = route_result(quote!(#conv(#val)), route);
            quote!( #t0 = #converted; )
        } else {
            quote!( #t0 = #conv(#val); )
        }
    }

    fn output_is_fallible(&self, ty: &TypeRef) -> bool {
        // The third walk over the same value, and it stops where the other two
        // do: a node with a wire of its own is encoded by its own converter, so
        // whether the encode can fail is THAT converter's answer. Peeling past
        // it asks about a converter that never runs — which decides whether the
        // binding needs `.panic()`, so the two disagreeing is a wrapper that
        // aborts where nothing opted in, or an opt-in demanded for a conversion
        // that cannot fail (#428 review).
        if !self.has_own_wire(ty) {
            let vec_elem = match ty.kind() {
                TypeKind::Vec(e) => Some(&**e),
                _ => None,
            };
            if let Some(inner) = ty
                .optional_inner()
                .or(vec_elem)
                .or_else(|| r_cow_slice_elem(ty))
                .or_else(|| r_scalar_slice_elem(ty))
            {
                return self.output_is_fallible(inner);
            }
        }
        self.out_frag(ty)
            .is_some_and(|entry| entry.function.call().fallible())
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
    fn alias_slot_of(&self, ty: &TypeRef) -> Option<(TypeKey, AliasAccess)> {
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

    /// The runtime **alias preflight**: reject a call whose arguments name the
    /// same resource in a combination that would reconstruct or invalidate it
    /// twice, *before* any conversion runs.
    ///
    /// `z_combine(primary: ZThing, fallback: ZThing)` is a supported
    /// declaration; called as `z_combine(x, x)` it reaches `Box::from_raw`
    /// twice on one allocation. So is `f(a: ZThing, b: &ZThing)` — the borrow
    /// dangles the moment the consume takes ownership — and `f(a: &mut T, b:
    /// &T)`, where the exclusive reference is not exclusive at all. Rejecting
    /// the *declaration* is not an option: these are shapes that ship today, so
    /// removing them would be a regression that a later stage would have to
    /// undo.
    ///
    /// Emitted whenever a call has **at least one `Consume` or
    /// `ExclusiveBorrow`** and **any other active access in the same resource
    /// domain**. Stated that way rather than as "two or more consumed
    /// parameters", which would skip exactly the two mixed cases above.
    ///
    /// A NULL pointer names no resource — two NULLs are not an alias — and is
    /// rejected by each converter's own null check, so the comparison skips it.
    ///
    /// Shared/shared is *not* rejected: two `&T` to one resource is legal Rust
    /// and legal C.
    pub(super) fn alias_preflight(
        &self,
        f: &prebindgen_registry::flat::Function,
        route: &ErrRoute,
    ) -> Option<TokenStream> {
        let mut slots: Vec<(syn::Ident, TypeKey, AliasAccess)> = Vec::new();
        for p in &f.params {
            if let Some((key, access)) = self.alias_slot_of(&p.ty) {
                slots.push((p.name.clone(), key, access));
            }
        }

        let on_err = route_message(route);
        let mut checks: Vec<TokenStream> = Vec::new();
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                let (a, a_key, a_access) = &slots[i];
                let (b, b_key, b_access) = &slots[j];
                if a_key != b_key {
                    continue;
                }
                // At least one side must be exclusive about the resource.
                if matches!(a_access, AliasAccess::Shared)
                    && matches!(b_access, AliasAccess::Shared)
                {
                    continue;
                }
                let msg = format!(
                    "aliasing arguments: `{a}` ({}) and `{b}` ({}) are the same `{}` — a \
                     consumed or exclusively-borrowed resource may not be named twice in one call",
                    a_access.describe(),
                    b_access.describe(),
                    a_key.as_str(),
                );
                checks.push(quote!(
                    if !(#a as *const ()).is_null() && (#a as *const ()) == (#b as *const ()) {
                        let __msg = ::std::string::String::from(#msg);
                        #on_err
                    }
                ));
            }
        }
        (!checks.is_empty()).then(|| quote!(#(#checks)*))
    }

    /// Build the wire param list, per-input decode statements, and call-site
    /// argument expressions. Fallible inputs (converter returns `Result<_,
    /// String>`) route their `Err(msg)` per `route`; infallible inputs decode
    /// directly.
    pub(super) fn emit_inputs(
        &self,
        orig: &syn::Ident,
        f: &prebindgen_registry::flat::Function,
        registry: &Registry,
        route: &ErrRoute,
        emit: &prebindgen_registry::Emit,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let mut params = Vec::new();
        // The alias preflight runs BEFORE every decode, which is the whole
        // point: by the time the first converter has run, one of the aliased
        // arguments has already been consumed.
        let mut decodes: Vec<TokenStream> = self.alias_preflight(f, route).into_iter().collect();
        let mut call_args = Vec::new();

        for param in &f.params {
            let ident = &param.name;
            let arg_reading = &param.ty;
            let arg_ty = &emit.spell_ty(arg_reading);

            // `&[E]` slice (scalar `E`): two wire params (`*const E`, `usize`),
            // decoded zero-copy. NULL pointer ⇒ empty slice (not an error).
            if let Some(elem) = r_scalar_slice_elem(arg_reading).map(|t| spelled(t, emit)) {
                let len_id = format_ident!("{}_len", ident);
                params.push(quote!(#ident: *const #elem));
                params.push(quote!(#len_id: usize));
                decodes.push(quote!(
                    let #ident: &[#elem] = if #ident.is_null() {
                        &[]
                    } else {
                        ::core::slice::from_raw_parts(#ident, #len_id)
                    };
                ));
                call_args.push(quote!(#ident));
                continue;
            }

            // `&[E]` slice (inline-opaque by-value `E`, e.g. a `repr_c_struct`):
            // two wire params (`*const E_counterpart`, `usize`), reinterpreted to
            // `&[E]` zero-copy. The counterpart is layout-identical to `E` (asserted
            // by a generated `const _`), so the whole block transmutes in one shot —
            // the slice analogue of the single-`&E` `__cbg_in_*` converter. NULL ⇒
            // empty slice.
            if let Some(elem) = self
                .r_value_opaque_slice_elem(arg_reading)
                .map(|t| spelled(t, emit))
            {
                // The C wire element is the inline-opaque counterpart (e.g. the
                // generated `payload_t` mirror), layout-identical to the Rust value.
                let elem_wire = self
                    .value_opaque_ty(&elem)
                    .expect("value_opaque_slice_elem guaranteed a value_opaque element")
                    .clone();
                let src = self.src_ty(&elem);
                let len_id = format_ident!("{}_len", ident);
                params.push(quote!(#ident: *const #elem_wire));
                params.push(quote!(#len_id: usize));
                decodes.push(quote!(
                    let #ident: &[#src] = if #ident.is_null() {
                        &[]
                    } else {
                        ::core::slice::from_raw_parts(#ident as *const #src, #len_id)
                    };
                ));
                call_args.push(quote!(#ident));
                continue;
            }

            let entry = registry
                .reading_of(arg_ty)
                .and_then(|tr| self.in_frag(&tr))
                .unwrap_or_else(|| {
                    panic!(
                        "Cbindgen::on_function: input type `{}` of `{}` has no input converter",
                        TypeKey::from_type(arg_ty),
                        orig
                    )
                });
            let wire = &entry.destination;
            let conv = entry.function.call().ident();

            params.push(quote!(#ident: #wire));

            if entry.function.call().fallible() {
                let on_err = route_message(route);
                decodes.push(quote!(
                    let #ident = match #conv(#ident) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__msg) => { #on_err }
                    };
                ));
            } else {
                decodes.push(quote!(let #ident = #conv(#ident);));
            }

            // Each input converter produces exactly the source param type
            // (`String` by value, `&T` for borrows, owned `T` for consume), so
            // the decoded binding is passed straight through.
            call_args.push(quote!(#ident));
        }

        (params, decodes, call_args)
    }
}
