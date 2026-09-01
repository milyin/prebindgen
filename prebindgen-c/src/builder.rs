use super::*;

impl CbindgenBuilder {
    fn clear_current(&mut self) {
        self.current = None;
    }

    /// Create an adapter with no declarations (emits an empty library).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the module path the original `#[prebindgen]` items live under
    /// (e.g. `syn::parse_quote!(zenoh_flat)`). Root-level modifier: resets the
    /// current declaration, so it can't be followed by `.base_name()`/`.error()`/etc.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = Some(p);
        self.clear_current();
        self
    }

    /// Set the name of the universal memory-freeing function (a type-agnostic C
    /// `free`) the generated layer exports for releasing `char*` data it hands to
    /// C — string returns and `String` fields of data structs. Root-level
    /// modifier: resets the current declaration. Required whenever the adapter
    /// produces such string memory; otherwise that's a build error.
    pub fn free_memory_function(mut self, name: impl Into<String>) -> Self {
        self.free_fn = Some(name.into());
        self.clear_current();
        self
    }

    /// Set the **base** Rust-type mangler: maps a type's Rust short name (e.g.
    /// `ZKeyExpr`) to a canonical token (e.g. `keyexpr`). Its output feeds
    /// [`Self::mangle_type_name`], [`Self::mangle_destructor`] and
    /// [`Self::mangle_callback`], so a one-off spelling fix (e.g. `KeyExpr` →
    /// `keyexpr`) lives in a single place instead of a per-declaration
    /// `.base_name()` exception. Root-level modifier (resets the current
    /// declaration). The adapter ships no default — unset, the base defaults to the
    /// `snake_case` of the Rust short name.
    pub fn mangle_rust_type(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_rust_type = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Set the type-name mangler: base (see [`Self::mangle_rust_type`]) → the C
    /// type name emitted for a `opaque_ptr` / `data_struct` / `enum_type` (e.g.
    /// `keyexpr` → `z_keyexpr_t`). The base can be overridden per declaration by
    /// [`.base_name()`](Self::base_name). Root-level modifier.
    pub fn mangle_type_name(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_type_name = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Set the destructor mangler: base → an opaque handle's `_drop` symbol (e.g.
    /// `keyexpr` → `z_keyexpr_drop`). Root-level modifier.
    pub fn mangle_destructor(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_destructor = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Set the "take" mangler: base → the public move symbol of a `value_opaque`
    /// type used as a [`Self::takeable_param`] (e.g. `sample` → `z_sample_take`).
    /// When unset, the take symbol defaults to `<destructor-base>_take`. Root-level
    /// modifier.
    pub fn mangle_take(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_take = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Set the callback-struct mangler: the bases of a callback's argument types
    /// → the closure struct's C name (e.g. `["sample"]` → `z_closure_sample_t`,
    /// `[]` → `z_closure_drop_t`). A per-declaration [`.base_name()`](Self::base_name)
    /// replaces the args' bases with a single explicit base. Root-level modifier.
    pub fn mangle_callback(mut self, f: impl Fn(&[String]) -> String + 'static) -> Self {
        self.mangle_callback = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Set the function mangler: a `#[prebindgen]` function's Rust ident → its
    /// exported `#[no_mangle]` symbol (e.g. prefix `z_`). Functions are not types,
    /// so this does not go through the base mangler; the ident can be overridden
    /// per declaration by [`.base_name()`](Self::base_name). Root-level modifier.
    pub fn mangle_function(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_function = Some(Box::new(f));
        self.clear_current();
        self
    }

    /// Declare a `#[prebindgen]` function to convert into the C layer.
    /// Every `#[prebindgen]` item captured in `dir` — see
    /// `JniGenBuilder::source`.
    pub fn source<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.sources = std::mem::take(&mut self.sources).source(dir);
        self
    }

    /// The same, for a dependency this crate **renames** in `Cargo.toml`.
    pub fn source_named<P: AsRef<std::path::Path>>(
        mut self,
        dir: P,
        crate_name: impl Into<String>,
    ) -> Self {
        self.sources = std::mem::take(&mut self.sources).source_named(dir, crate_name);
        self
    }

    /// Add a captured item stream. Accumulates, so it mixes with
    /// [`Self::source`].
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (syn::Item, prebindgen::SourceLocation)>,
    {
        self.sources = std::mem::take(&mut self.sources).items(items);
        self
    }

    pub fn function(mut self, ident: syn::Ident) -> Self {
        assert!(
            !self.ignored_functions.contains(&ident),
            "Cbindgen::function cannot declare `{}` because it is already ignored",
            ident
        );
        self.functions.insert(ident.clone(), FnCfg::default());
        self.current = Some(CurrentDecl::Function(ident));
        self
    }

    /// Declare a canonical scalar conversion shared with JniGenBuilder. A domain on
    /// the [`ConvertDecl`] is validated in both directions; invalid scalar
    /// values become by-value niches for `Option`/`Result`, with public C
    /// constants derived from the conversion's naming base.
    pub fn convert(mut self, decl: ConvertDecl) -> Self {
        assert!(
            decl.input_spec().is_some() || decl.output_spec().is_some(),
            "Cbindgen::convert declares no input or output conversion"
        );
        let key = decl.key().clone();
        assert!(
            !self
                .convert_decls
                .iter()
                .any(|existing| *existing.key() == key),
            "Cbindgen::convert already declares {}",
            key
        );
        self.convert_decls.push(decl);
        self.current = Some(CurrentDecl::Convert(key));
        self
    }

    /// Mark a `#[prebindgen]` function as intentionally ignored by this
    /// adapter. Root-level modifier: suppresses the registry's
    /// "skipping undeclared" warning for that function without scanning or
    /// emitting it.
    pub fn ignore_function(mut self, ident: syn::Ident) -> Self {
        assert!(
            !self.functions.contains_key(&ident),
            "Cbindgen::ignore_function cannot ignore `{}` because it is already declared",
            ident
        );
        self.ignored_functions.insert(ident);
        self.clear_current();
        self
    }

    /// Allow the most recently declared [`Self::function`] to `panic!` on an
    /// internal error message. Required when a non-`Result` function has a
    /// fallible input (otherwise that's a build error) — a null borrow, an
    /// invalid `String`, or an out-of-range discriminant for a declared
    /// [`Self::enum_type`].
    pub fn panic(mut self) -> Self {
        match &self.current {
            Some(CurrentDecl::Function(ident)) => {
                let ident = ident.clone();
                self.functions
                    .get_mut(&ident)
                    .expect("function entry vanished")
                    .panic = true;
            }
            other => panic!(
                "Cbindgen::panic must be chained after a `function(...)` call, \
                 not after {}",
                describe_current(other)
            ),
        }
        self
    }

    /// Declare a pointer-struct (opaque-handle) type — a `Box`-owned Rust value
    /// the C side holds as `#[repr(C)] struct T { _0: *mut c_void }`. Its C
    /// struct + `<name>_drop` destructor are generated. (Mirrors `JniExt`'s
    /// `ptr_class`.)
    pub fn opaque_ptr(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::opaque_ptr cannot declare `{}` because it is already ignored",
            key
        );
        self.opaque.insert(key.clone(), TypeCfg::new(ty));
        self.current = Some(CurrentDecl::Ptr(key));
        self
    }

    /// Declare a by-value `#[repr(C)]` data struct (e.g. `Error`).
    pub fn data_struct(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::data_struct cannot declare `{}` because it is already ignored",
            key
        );
        self.data.insert(key.clone(), TypeCfg::new(ty));
        self.current = Some(CurrentDecl::Data(key));
        self
    }

    /// Declare an **inline-opaque by-value, plain-data** type: the Rust value
    /// `rust_ty` is passed across the C ABI *by value* (no `Box`) by transmuting it
    /// to/from `opaque_ty`, an opaque `#[repr(C, align(_))]` counterpart of
    /// identical size+align (typically produced by a size/align probe generator and
    /// defined elsewhere). Use this for types that hold **no external resource**
    /// (typically `Copy` — e.g. a timestamp): consuming one simply moves it out,
    /// leaving the source's bitwise duplicate harmlessly droppable, so **no
    /// gravestone write-back and no `prebindgen_c_runtime::Gravestone` impl are needed** —
    /// only the autogenerated `prebindgen_c_runtime::Transmute` glue (emitted here) plus a
    /// fail-closed `const _` size+align equality assert. Contrast
    /// [`Self::opaque_owned_struct`] for types owning external data.
    pub fn opaque_data_struct(self, rust_ty: syn::Type, opaque_ty: syn::Type) -> Self {
        self.declare_opaque(rust_ty, opaque_ty, OpaqueKind::Data)
    }

    /// Declare an **inline-opaque by-value, owns-external-data** type: like
    /// [`Self::opaque_data_struct`], but for a Rust value that owns external resources
    /// (refcounts / heap — e.g. a byte buffer, a sample). Passed *by value* (no
    /// `Box`) via the `opaque_ty` transmute counterpart; the converters move values
    /// via `prebindgen_c_runtime::Transmute` and write a **gravestone** back on consume
    /// (safe drop-after-move), exposing an `Option<rust_ty>` null niche. The
    /// consumer must implement `prebindgen_c_runtime::Gravestone` for `opaque_ty` — only
    /// its *logic* (`rust_gravestone`).
    pub fn opaque_owned_struct(self, rust_ty: syn::Type, opaque_ty: syn::Type) -> Self {
        self.declare_opaque(rust_ty, opaque_ty, OpaqueKind::Owned)
    }

    /// Shared body of [`Self::opaque_data_struct`] / [`Self::opaque_owned_struct`].
    pub(super) fn declare_opaque(
        mut self,
        rust_ty: syn::Type,
        opaque_ty: syn::Type,
        kind: OpaqueKind,
    ) -> Self {
        let method = match kind {
            OpaqueKind::Data => "opaque_data_struct",
            OpaqueKind::Owned => "opaque_owned_struct",
        };
        let key = TypeKey::from_type(&rust_ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::{method} cannot declare `{key}` because it is already ignored",
        );
        self.value_opaque.insert(
            key.clone(),
            ValueOpaqueCfg {
                opaque: opaque_ty,
                kind,
                generate_mirror: false,
                assume_c_field_validity: false,
                cfg: TypeCfg::new(rust_ty),
            },
        );
        self.current = Some(CurrentDecl::ValueOpaque(key));
        self
    }

    /// Declare a **`#[repr(C)]`, FFI-safe value struct** crossed **by direct
    /// reinterpret** (zero-copy) — the C struct's memory *is* the Rust struct's
    /// memory. Unlike [`Self::data_struct`] (which copies each field, lowering a
    /// `String` to `char*`), this passes the whole struct by value via
    /// `prebindgen_c_runtime::Transmute` and a `&T` borrow / `impl Fn(&T)` callback as a
    /// zero-copy `*const` pointer cast — the value-opaque machinery, but with an
    /// **auto-generated visible-field** `#[repr(C)]` C mirror (so C reads the
    /// fields directly) instead of an opaque blob.
    ///
    /// Every field must be FFI-safe: a primitive, a declared
    /// [`Self::enum_type`], or an **opaque pointer** `Option<Box<T>>` / `Box<T>`
    /// where `T` is a declared [`Self::opaque_ptr`] (rendered `*mut t_t`; this is
    /// how a heap `String` rides along — `Option<Box<String>>` → `string_t *`). The
    /// source type **must** be `#[repr(C)]`; a fail-closed `size_of`/`align_of`
    /// assert against the generated mirror proves the reinterpret sound at compile
    /// time. Call after the manglers are configured (the mirror name is resolved
    /// through them). A `<base>_drop` is generated.
    ///
    /// **Owned-ness is inferred** from the fields: a struct with an opaque-pointer field
    /// owns external resources, so a by-value consume cleans the moved-from slot (nulls
    /// the owned pointers) to keep the caller's later `_drop` a no-op — no `.owned()`
    /// modifier and no double-free footgun. A struct with only scalar/enum fields is
    /// plain data (a by-value crossing is a bitwise copy with no write-back). The source
    /// type needs `Default` **only** if it has a bare `Box<T>` field (whose gravestone
    /// can't be a NULL pointer); `Option<Box<T>>` fields are nulled in place.
    ///
    /// # Why the spelling is load-bearing here
    ///
    /// This is the one position that is **exempt** from the rule stated on
    /// [`prebindgen_registry::Prebindgen`] — *same `kind` ⇒ same
    /// destination-language type* — and the exemption is structural rather than
    /// a concession. A mirror is not converted; it is **reinterpreted from the
    /// source struct's bytes**, so its field types are a *layout* fact. `Box<T>`
    /// is a pointer and `T` is inline: to C they really are different types, and
    /// the size/align assert above would reject a mirror that pretended
    /// otherwise. So this path reads the wrapper the model erases —
    /// [`kind`](prebindgen_registry::flat::TypeRef::kind) rather than
    /// [`unwrapped`](prebindgen_registry::flat::TypeRef::unwrapped) — on
    /// purpose. It is the one place the usual "classify off `kind`, spell off
    /// the syntax" split inverts, and it inverts because the contract is layout
    /// rather than surface.
    ///
    /// That is also why a wrapper the model erases must **not** be refused here
    /// (prebindgen#230). `Option<Box<String>>` is how a source crate says "this
    /// field is a nullable pointer" — a layout statement a zero-copy mirror is
    /// entitled to read, not the source naming a C type. There is no competing
    /// spelling to prefer: `Option<String>` is a 24-byte niche-optimised value
    /// with no C representation at all, and declaring one is a hard error naming
    /// the field. Rejecting the `Box` would leave a nullable-pointer field
    /// inexpressible.
    pub fn repr_c_struct(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::repr_c_struct cannot declare `{}` because it is already ignored",
            key
        );
        let mirror = self.c_type_ident(&key);
        self.value_opaque.insert(
            key.clone(),
            ValueOpaqueCfg {
                opaque: syn::parse_quote!(#mirror),
                kind: OpaqueKind::Data,
                generate_mirror: true,
                assume_c_field_validity: false,
                cfg: TypeCfg::new(ty),
            },
        );
        self.current = Some(CurrentDecl::ValueOpaque(key));
        self
    }

    /// Accept a [`Self::repr_c_struct`] whose mirror has **restricted-validity**
    /// fields, taking responsibility for their bytes.
    ///
    /// A `repr_c_struct` crosses IN by one whole-struct reinterpret, so there is
    /// no per-field hook where a C-supplied byte could be normalised or checked
    /// before the source struct exists. A field whose Rust type accepts only
    /// *some* bit patterns — `bool` (`0`/`1`) or a declared [`Self::enum_type`]
    /// (the declared discriminants) — is therefore undefined behaviour the
    /// moment C writes anything else into the mirror and hands it back. The
    /// generator rejects such a declaration by default (#170 instance 3, #158
    /// instance 3); the real fix is a raw-wire lowering, which does not exist
    /// yet.
    ///
    /// This modifier is the acknowledgement, not a fix: it says the C side of
    /// this binding is trusted to write only in-domain bytes into those fields.
    /// It exists so that the audit rejects **silently unsound new declarations**
    /// without removing bindings that already ship. Chain it directly after the
    /// [`Self::repr_c_struct`] it applies to; the panic message names every
    /// field it would cover.
    ///
    /// Prefer, in order: move the field into a [`Self::data_struct`] (per-field
    /// wires, so `bool` normalises), pass it as a separate scalar parameter, or
    /// widen it to an integer the whole domain of which is valid.
    pub fn assume_c_field_validity(mut self) -> Self {
        match self.current.clone() {
            Some(CurrentDecl::ValueOpaque(key)) => {
                self.value_opaque
                    .get_mut(&key)
                    .expect("entry vanished")
                    .assume_c_field_validity = true;
            }
            other => panic!(
                "Cbindgen::assume_c_field_validity must follow a `repr_c_struct` declaration, \
                 not {}",
                describe_current(&other)
            ),
        }
        self
    }

    /// Mark a `#[prebindgen]` type as intentionally ignored by this adapter.
    /// Root-level modifier: suppresses the registry's "skipping undeclared"
    /// warning for that type without scanning or emitting it.
    pub fn ignore_type(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.opaque.contains_key(&key)
                && !self.data.contains_key(&key)
                && !self.value_opaque.contains_key(&key)
                && !self.enums.contains_key(&key)
                && !self.tagged_unions.contains_key(&key),
            "Cbindgen::ignore_type cannot ignore `{}` because it is already declared",
            key
        );
        self.ignored_types.insert(key);
        self.clear_current();
        self
    }

    /// Set the **base name** token of the **current declaration** (universal
    /// modifier): the per-declaration base fed to the name manglers, replacing the
    /// auto-derived one. For a type it replaces the `mangle_rust_type` base (so
    /// `mangle_type_name`/`mangle_destructor`/`mangle_take` all see it); for a
    /// function it replaces the ident fed to `mangle_function`; for a callback it
    /// is the sole base fed to `mangle_callback` (replacing the args' bases —
    /// useful to disambiguate e.g. `&T` from `T` closures). E.g.
    /// `.callback(...).base_name("sample_ref")` with a `|bases| "z_closure_{…}_t"`
    /// mangler → `z_closure_sample_ref_t`. Panics if not chained directly after a
    /// declaration.
    pub fn base_name(mut self, base: impl Into<String>) -> Self {
        let base = base.into();
        match self.current.clone() {
            Some(CurrentDecl::Ptr(key)) => {
                self.opaque.get_mut(&key).expect("entry vanished").base = Some(base);
            }
            Some(CurrentDecl::Data(key)) => {
                self.data.get_mut(&key).expect("entry vanished").base = Some(base);
            }
            Some(CurrentDecl::ValueOpaque(key)) => {
                self.value_opaque
                    .get_mut(&key)
                    .expect("entry vanished")
                    .cfg
                    .base = Some(base);
            }
            Some(CurrentDecl::Enum(key)) => {
                self.enums.get_mut(&key).expect("entry vanished").base = Some(base);
            }
            Some(CurrentDecl::TaggedUnion(key)) => {
                self.tagged_unions
                    .get_mut(&key)
                    .expect("entry vanished")
                    .base = Some(base);
            }
            Some(CurrentDecl::Callback(key)) => {
                self.callbacks.get_mut(&key).expect("entry vanished").base = Some(base);
            }
            Some(CurrentDecl::Function(ident)) => {
                self.functions.get_mut(&ident).expect("entry vanished").base = Some(base);
            }
            Some(CurrentDecl::Convert(key)) => {
                self.convert_bases.insert(key, base);
            }
            None => panic!(
                "Cbindgen::base_name must be chained directly after a declaration \
                 (`opaque_ptr` / `data_struct` / `enum_type` / `tagged_union` / `callback` / \
                 `function` / `convert`)"
            ),
        }
        self
    }

    /// Mark the current declaration (which must be a [`Self::data_struct`]) as an
    /// error type: it may appear as the `E` of a `Result<_, E>` return. The type
    /// must implement `From<String>`. Panics if the current declaration is not a
    /// data struct.
    pub fn error(mut self) -> Self {
        match &self.current {
            Some(CurrentDecl::Data(key)) => {
                self.error.insert(key.clone());
            }
            other => panic!(
                "Cbindgen::error must be chained after a `data_struct(...)` call \
                 (error types are marshalled by value), not after {}",
                describe_current(other)
            ),
        }
        self
    }

    /// Declare an **opaque error type** — one that appears as the `E` of a
    /// `Result<_, E>` but is *not* a by-value [`Self::data_struct`] (e.g.
    /// `ZError = Box<dyn Error + Send + Sync>`). Such an error is marshalled to C
    /// as a `char*` message obtained by calling `message_fn(&err) -> String`
    /// (e.g. `z_error_message`); the generated wrapper's error out-param becomes
    /// `char **e`. The type must implement `From<String>` (so a fallible input's
    /// internal message can be lifted into it). Root-level modifier (resets the
    /// current declaration).
    pub fn opaque_error(mut self, error_ty: syn::Type, message_fn: syn::Ident) -> Self {
        let key = TypeKey::from_type(&error_ty);
        self.error.insert(key.clone());
        self.opaque_errors.insert(key, message_fn);
        self.clear_current();
        self
    }

    /// Declare a C-like (fieldless) enum type to convert. (Mirrors `JniExt`'s
    /// `enum_class`.)
    ///
    /// Crossing **into** Rust, the caller's discriminant is validated before any
    /// Rust enum is built — an out-of-range one is a fallible-input error, so a
    /// non-`Result` function taking this enum by value needs [`Self::panic`].
    /// See the module docs for why the wire is `MaybeUninit<mirror>`.
    pub fn enum_type(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::enum_type cannot declare `{}` because it is already ignored",
            key
        );
        self.enums.insert(key.clone(), TypeCfg::new(ty));
        self.current = Some(CurrentDecl::Enum(key));
        self
    }

    /// Declare a **data-carrying** enum: it crosses by value as a `#[repr(C)]`
    /// enum with payload variants, which cbindgen renders as the idiomatic C
    /// tagged union (a tag enum plus a `union` of the variant bodies). The
    /// counterpart of [`Self::enum_type`], which is for the unit-variant-only
    /// case a plain C `enum` can hold. (Mirrors `JniExt`'s `sealed_class`.)
    ///
    /// Each payload field crosses as its own wire, chosen by the same policy a
    /// [`Self::repr_c_struct`] field uses, extended with `String` → `char *`:
    /// a scalar passes through, a declared [`Self::enum_type`] becomes its C
    /// enum, a `String` becomes a malloc'd `char *`, and an opaque pointer
    /// `Option<Box<T>>` / `Box<T>` (with `T` a declared [`Self::opaque_ptr`])
    /// becomes `*mut t_t`. Anything else is a generation error.
    ///
    /// **Ownership.** The union crosses by value, so when any variant's
    /// payload wire owns memory (`char *`, an opaque pointer) a typed
    /// `<base>_drop(t_t *)` is generated that frees the **active arm** —
    /// consistent with the existing typed per-pointer drops. A union whose
    /// payloads are all plain data needs no drop and gets none.
    ///
    /// **Validity.** Crossing **into** Rust, the tag a C caller supplied is
    /// range-checked before any Rust enum is built — so, exactly like
    /// [`Self::enum_type`], a union taken by value is a fallible input, and a
    /// function taking one without a `Result` return needs [`Self::panic`].
    /// A data struct carrying a union field inherits that fallibility. See the
    /// module docs for the wire this uses and why.
    pub fn tagged_union(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::tagged_union cannot declare `{}` because it is already ignored",
            key
        );
        self.tagged_unions.insert(key.clone(), TypeCfg::new(ty));
        self.current = Some(CurrentDecl::TaggedUnion(key));
        self
    }

    /// Declare a callback signature so its `impl Fn(...)` parameters resolve and
    /// a `#[repr(C)]` closure struct (`{ void *context; call; drop }`) is
    /// emitted for it. `ty` must be `impl Fn(Args...) + Send + Sync + 'static`.
    /// Identical signatures share one struct. Sets the declaration cursor, so a
    /// following `.base_name("...")` sets the base fed to
    /// [`mangle_callback`](Self::mangle_callback) (else the args' bases drive
    /// the generated name).
    pub fn callback(mut self, ty: syn::Type) -> Self {
        let args = extract_fn_trait_args(&ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::callback expects `impl Fn(Args...) + Send + Sync + 'static`, got `{}`",
                ty.to_token_stream()
            )
        });
        let key: CallbackKey = args.iter().map(TypeKey::from_type).collect();
        self.callbacks.insert(key.clone(), CbCfg::new());
        self.current = Some(CurrentDecl::Callback(key));
        self
    }

    /// Mark argument `idx` of the **current callback declaration** as a *takeable
    /// owned pointer*: the C `call` receives `*mut z_x_t` (not by value); the
    /// callee may take the value (`z_x_take` moves it out, leaving a gravestone) or
    /// just read it; the trampoline drops it after the call (no-op if taken). The
    /// arg type must be an inline-opaque type ([`Self::opaque_owned_struct`] /
    /// [`Self::opaque_data_struct`]). Chain after `.callback(...)` (and any `.base_name(...)`).
    pub fn takeable_param(mut self, idx: usize) -> Self {
        match &self.current {
            Some(CurrentDecl::Callback(key)) => {
                let key = key.clone();
                self.callbacks
                    .get_mut(&key)
                    .expect("entry vanished")
                    .takeable
                    .insert(idx);
            }
            other => panic!(
                "Cbindgen::takeable_param must be chained after a `callback(...)` call, not after {}",
                describe_current(other)
            ),
        }
        self
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Path to a source function (e.g. `zenoh_flat::z_keyexpr_try_from`).
    pub(super) fn src_fn(&self, ident: &syn::Ident) -> syn::Path {
        match &self.source_module {
            Some(m) => {
                let mut p = m.clone();
                p.segments.push(syn::PathSegment::from(ident.clone()));
                p
            }
            None => syn::Path::from(ident.clone()),
        }
    }

    /// Wire-only callback-slice classification for the Invoke planner.
    ///
    /// Resolution retains only the adapter-owned wire; the shared Invoke
    /// composer owns the source spelling until final render time.
    pub(super) fn callback_slice_elem_wire_type_of(&self, ty: &TypeRef) -> Option<syn::Type> {
        let elem = super::r_shared_slice_elem(ty)?;
        self.value_opaque_ty_of(&elem.key())
            .cloned()
            .or_else(|| super::scalar_ty(elem))
    }

    /// Config of a declared type (across the opaque/data/enum maps), by key.
    pub(super) fn type_cfg(&self, key: &TypeKey) -> Option<&TypeCfg> {
        self.opaque
            .get(key)
            .or_else(|| self.data.get(key))
            .or_else(|| self.value_opaque.get(key).map(|c| &c.cfg))
            .or_else(|| self.enums.get(key))
            .or_else(|| self.tagged_unions.get(key))
    }

    /// The inline-opaque C wire type by semantic identity.
    pub(super) fn value_opaque_ty_of(&self, key: &TypeKey) -> Option<&syn::Type> {
        self.value_opaque.get(key).map(|c| &c.opaque)
    }

    /// Type keys used as a takeable callback parameter (any `.takeable_param(idx)`
    /// across all declared callbacks). These value_opaque types get a public
    /// `<base>_take(dst, src)` move function.
    pub(super) fn takeable_type_keys(&self) -> HashSet<TypeKey> {
        let mut s = HashSet::new();
        for (key, cfg) in &self.callbacks {
            for &idx in &cfg.takeable {
                if let Some(tk) = key.get(idx) {
                    s.insert(tk.clone());
                }
            }
        }
        s
    }

    /// Public "take" (move) symbol for a takeable value_opaque type:
    /// [`Self::mangle_take`] over the base, else `<base>_take` (e.g.
    /// `z_sample_take`). Symmetric with [`Self::destructor_symbol`].
    pub(super) fn take_symbol(&self, key: &TypeKey) -> syn::Ident {
        if let Some(f) = &self.mangle_take {
            return format_ident!("{}", f(&self.rust_base(key)));
        }
        format_ident!("{}_take", self.rust_base(key))
    }

    /// Base token for a Rust type: [`Self::mangle_rust_type`] applied to the Rust
    /// short name, or the short name verbatim when unset. Feeds the type-name,
    /// destructor and callback manglers.
    pub(super) fn rust_base(&self, key: &TypeKey) -> String {
        if let Some(b) = self.type_cfg(key).and_then(|c| c.base.clone()) {
            return b;
        }
        let short = type_short(key);
        match &self.mangle_rust_type {
            Some(f) => f(&short),
            // No mangler: a C-like `snake_case` default (so destructors/take/type
            // names read e.g. `sample_drop`, not `Sample_drop`).
            None => snake_case(&short),
        }
    }

    /// Emitted C type name of a declared type: [`Self::mangle_type_name`] over the
    /// base, else the base (which is the `mangle_rust_type`/`.base_name` token).
    pub(super) fn c_type_name(&self, key: &TypeKey) -> String {
        let base = self.rust_base(key);
        match &self.mangle_type_name {
            Some(f) => f(&base),
            None => base,
        }
    }

    /// C type identifier (the `#[repr(C)]` struct/enum name + the wire type used
    /// across converters and wrappers).
    pub(super) fn c_type_ident(&self, key: &TypeKey) -> syn::Ident {
        format_ident!("{}", self.c_type_name(key))
    }

    /// Destructor symbol of an opaque handle: [`Self::mangle_destructor`] over the
    /// base, else `<base>_drop`.
    pub(super) fn destructor_symbol(&self, key: &TypeKey) -> syn::Ident {
        if let Some(f) = &self.mangle_destructor {
            return format_ident!("{}", f(&self.rust_base(key)));
        }
        format_ident!("{}_drop", self.rust_base(key))
    }

    /// Emitted C type name of a callback's closure struct: [`Self::mangle_callback`]
    /// over the bases — a `.base_name(...)` override (as the sole base) when set,
    /// else the args' derived bases — or, with no mangler, a generic default
    /// (`closure` for zero bases, `closure_<base0>_<base1>…` otherwise). The
    /// adapter's own default carries no target-language naming convention.
    pub(super) fn callback_c_name(&self, key: &CallbackKey) -> String {
        let base_override = self.callbacks.get(key).and_then(|c| c.base.clone());
        if let Some(f) = &self.mangle_callback {
            // The override (when set) is the sole base; otherwise the args' bases.
            let bases: Vec<String> = match &base_override {
                Some(b) => vec![b.clone()],
                None => key.iter().map(|k| self.rust_base(k)).collect(),
            };
            return f(&bases);
        }
        // No mangler: an explicit base is the name as-is; otherwise compose from
        // the args' bases.
        if let Some(b) = base_override {
            return b;
        }
        if key.is_empty() {
            "closure".to_string()
        } else {
            let parts: Vec<String> = key.iter().map(|k| self.rust_base(k)).collect();
            format!("closure_{}", parts.join("_"))
        }
    }

    /// C struct identifier for a callback's closure type (see
    /// [`Self::callback_c_name`]).
    pub(super) fn callback_c_ident(&self, key: &CallbackKey) -> syn::Ident {
        format_ident!("{}", self.callback_c_name(key))
    }
}

/// Human-readable description of the current declaration, for panic messages.
fn describe_current(current: &Option<CurrentDecl>) -> String {
    match current {
        None => "no declaration".to_string(),
        Some(CurrentDecl::Ptr(k)) => format!("opaque_ptr `{}`", k.as_str()),
        Some(CurrentDecl::Data(k)) => format!("data_struct `{}`", k.as_str()),
        Some(CurrentDecl::ValueOpaque(k)) => format!("value_opaque `{}`", k.as_str()),
        Some(CurrentDecl::Enum(k)) => format!("enum_type `{}`", k.as_str()),
        Some(CurrentDecl::TaggedUnion(k)) => format!("tagged_union `{}`", k.as_str()),
        Some(CurrentDecl::Callback(k)) => {
            let args: Vec<&str> = k.iter().map(|t| t.as_str()).collect();
            format!("callback `impl Fn({})`", args.join(", "))
        }
        Some(CurrentDecl::Function(i)) => format!("function `{i}`"),
        Some(CurrentDecl::Convert(k)) => format!("convert `{k}`"),
    }
}
