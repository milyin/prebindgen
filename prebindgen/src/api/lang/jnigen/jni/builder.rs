//! Builder API for [`JniGen`].
//!
//! [`JniGen::new`] starts from defaults; global settings are applied with
//! the `set_*` methods (`config.rs`) and declarations are *accepted* as
//! pre-built objects (`decl.rs`) via [`JniGen::package`], [`JniGen::expand`],
//! and [`JniGen::convert`] — there is no fluent typestate cursor. Carved from the former monolithic
//! JNI module; shares the `jni` namespace via `use super::*`.

use super::*;

impl JniGen {
    /// The module path a generated call to `#[prebindgen]` fn `ident` must be
    /// qualified with: the fn's **origin crate** as recorded from its
    /// stream's `SourceLocation` stamp (multi-source bindings — helper
    /// crates layered on the flat crate), else the registry's default
    /// module (first-seen stream origin), else `crate`.
    pub(crate) fn fn_module(
        &self,
        registry: &Registry<KotlinMeta>,
        ident: &syn::Ident,
    ) -> syn::Path {
        registry
            .origin_module(ident)
            .or_else(|| registry.default_module())
            .unwrap_or_else(|| syn::parse_quote!(crate))
    }

    /// The module for source references with no per-item origin (declared
    /// types with no `#[prebindgen]` item, glob imports): the registry's
    /// default module (first source), `crate` for an origin-less registry.
    pub(crate) fn default_module(&self, registry: &Registry<KotlinMeta>) -> syn::Path {
        registry
            .default_module()
            .unwrap_or_else(|| syn::parse_quote!(crate))
    }

    /// Whether `ty` was registered via an `EnumClassDecl` — used by the
    /// Kotlin wrapper generator to decide if a parameter needs a `.value`
    /// projection between the typed enum (Kotlin signature) and the `Int`
    /// wire (JNI `external fun`).
    pub(crate) fn is_kotlin_enum(&self, ty: &syn::Type) -> bool {
        let key = TypeKey::from_type(ty);
        self.types
            .get(&key)
            .and_then(|c| c.enum_cfg.as_ref())
            .is_some()
    }
}

impl JniGen {
    /// Start a binding generator with default settings: empty base
    /// package, no `JNINative` init block, identity
    /// name-mangling, handle locks enabled. Adjust settings with the `set_*`
    /// methods, add declarations with [`package`](Self::package),
    /// [`expand`](Self::expand), [`convert`](Self::convert), etc., then run the
    /// result through `Registry::resolve` → `Generation::write_rust` /
    /// `write_kotlin`. Settings and
    /// declarations may be interleaved in any order — the builder stores
    /// only raw inputs, and every setting-derived name is computed at the
    /// point of use.
    pub fn new() -> Self {
        let mut jni = Self {
            package: String::new(),
            fun_name_mangle: None,
            ptr_class_name_mangle: None,
            data_class_name_mangle: None,
            enum_name_mangle: None,
            member_name_mangle: None,
            harness_name_mangle: None,
            interface_name_mangle: None,
            types: HashMap::new(),
            packages: BTreeMap::new(),
            input_wrappers: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            output_wrappers: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            emit_handle_locks: true,
            jni_native_init: None,
            expansions: crate::api::core::expand::Expansions::default(),
            deconstructors: crate::api::core::unfold::Deconstructors::default(),
            convert_decls: Vec::new(),
            param_expand_decls: Vec::new(),
            return_expand_decls: Vec::new(),
            fn_param_expands: Vec::new(),
            fn_return_expands: Vec::new(),
            class_members: HashMap::new(),
            ignored_fns: std::collections::HashSet::new(),
            ignored_name_predicates: Vec::new(),
            ignored_class_types: std::collections::HashSet::new(),
            ignored_const_idents: std::collections::HashSet::new(),
        };
        // Built-in rank-2 `Result<_, _>` peel: every Result<T, E> succeeds
        // as T and routes E to the error-sink on Err. The rank tables are
        // internal — this is their only entry; `convert!` covers concrete
        // types at rank 0.
        let pattern: syn::Type = syn::parse_quote!(Result<_, _>);
        let key = TypeKey::from_type(&pattern);
        jni.output_wrappers[2].insert(
            key,
            Arc::new(|args: &[syn::Type], _: &Registry<KotlinMeta>| {
                Some((args[0].clone(), Some(args[1].clone()), syn::parse_quote!(v)))
            }),
        );
        jni
    }

    /// Apply the fun-name mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset. Called everywhere the framework
    /// derives a function-shaped Kotlin/JNI short name — scanned
    /// `#[prebindgen]` extern symbols, the synthetic `freePtr` destructor,
    /// and the Kotlin-side `external fun` that pairs with each.
    pub(crate) fn mangle_fun(&self, name: &str) -> String {
        match &self.fun_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the ptr-class mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset.
    pub(crate) fn mangle_ptr_class(&self, name: &str) -> String {
        match &self.ptr_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the data-class mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset.
    pub(crate) fn mangle_data_class(&self, name: &str) -> String {
        match &self.data_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the enum mangle closure to `name`, returning the closure result
    /// or `name` verbatim when unset.
    pub(crate) fn mangle_enum(&self, name: &str) -> String {
        match &self.enum_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the member mangle closure to `name` (the namespace-stripped
    /// camelCase default), returning the closure result or `name` verbatim
    /// when unset.
    pub(crate) fn mangle_member(&self, name: &str) -> String {
        match &self.member_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the harness mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset — identity default, same
    /// contract as the other five hooks.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.harness_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// The name of the centralized Native object that hosts every JNI
    /// `external fun`: the explicit default value `"JNINative"` run through
    /// the harness mangle hook (identity when unset). Drives both the
    /// Kotlin class emission and the JNI extern symbol path on the Rust
    /// side.
    pub(crate) fn jni_native_class_name(&self) -> String {
        self.mangle_harness("JNINative")
    }

    /// Resolve a relative class name against [`Self::package`] +
    /// `subpackage` (dot-separated; empty `subpackage` = the base package).
    /// Panics if `name` contains a `.` (a check that catches accidental FQNs
    /// in the relative-name builders) — a binding crate owns one package and
    /// must not write classes into anyone else's namespace.
    pub(crate) fn resolve_class_fqn(&self, subpackage: &str, name: &str) -> String {
        assert!(
            !name.contains('.'),
            "Kotlin class name `{}` must be relative (no dots) — FQNs are derived from the base \
             package + subpackage",
            name
        );
        let base: String = match (&self.package, subpackage) {
            (p, sub) if !sub.is_empty() && !p.is_empty() => format!("{}.{}", p, sub),
            (p, sub) if !sub.is_empty() && p.is_empty() => sub.to_string(),
            (p, _) => p.clone(),
        };
        if base.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", base, name)
        }
    }
}

impl Default for JniGen {
    fn default() -> Self {
        Self::new()
    }
}

// ── Accepting a `PackageDecl` ────────────────────────────────────────────

impl JniGen {
    /// Register a package's worth of classes, functions and consts (a
    /// [`PackageDecl`], built with [`package!`](crate::package)). Call it once
    /// per package, or several times for the same package name — the
    /// declarations merge, so you can split a large package across calls.
    pub fn package(mut self, decl: PackageDecl) -> Self {
        let PackageDecl {
            name,
            classes,
            functions,
            constants,
        } = decl;
        self.packages.entry(name.clone()).or_default();
        for class in classes {
            self.accept_class(&name, class);
        }
        for func in functions {
            self.accept_function(&name, func);
        }
        // One acceptor, dispatched on the decl's value source. The `.with`
        // source was already lowered to an expression (`path()`) at decl
        // time, so only three storage kinds exist internally.
        for c in constants {
            let pkg = self.packages.entry(name.clone()).or_default();
            match c.source {
                super::decl::ConstSource::Item => {
                    let mut entry = MethodEntry::new(c.rust_ident);
                    entry.kotlin_name_override = c.kotlin_name_override;
                    pkg.constants.push(entry);
                }
                super::decl::ConstSource::Fun(ref fn_ident) => {
                    let mut entry = MethodEntry::new(fn_ident.clone());
                    entry.kotlin_name_override = Some(c.val_name());
                    pkg.constant_functions.push(entry);
                }
                super::decl::ConstSource::Expr { ref ty, ref expr } => {
                    pkg.constant_exprs.push(super::decl::ConstExprDecl {
                        kotlin_name: c.val_name(),
                        ty: ty.clone(),
                        expr: expr.clone(),
                    });
                }
            }
        }
        self
    }

    /// Acknowledge a `#[prebindgen]` item this binding deliberately does
    /// NOT bind: nothing is emitted for it and the registry's per-item
    /// "skipping undeclared" warning is suppressed. Global — an ignored
    /// item belongs to no package. One acceptor, the kind carried by the
    /// decl (see [`IgnoreDecl`]): `fun!` / `ty!` / `constant!` for exact
    /// items, [`matching`](crate::lang::matching) for a name-family
    /// predicate over ANY item kind.
    pub fn ignore(mut self, decl: impl Into<IgnoreDecl>) -> Self {
        match decl.into().0 {
            super::decl::IgnoreKind::Fun(ident) => {
                self.ignored_fns.insert(ident);
            }
            super::decl::IgnoreKind::Type(key) => {
                self.ignored_class_types.insert(key);
            }
            super::decl::IgnoreKind::Const(ident) => {
                self.ignored_const_idents.insert(ident);
            }
            super::decl::IgnoreKind::Matching(pred) => {
                self.ignored_name_predicates.push(pred);
            }
        }
        self
    }

    fn accept_class(&mut self, subpackage: &str, decl: ClassDecl) {
        match decl {
            ClassDecl::Ptr(d) => self.accept_ptr_class(subpackage, d),
            ClassDecl::Enum(d) => self.accept_enum_class(subpackage, d),
            ClassDecl::Data(d) => self.accept_data_class(subpackage, d),
            ClassDecl::Value(d) => self.accept_value_class(subpackage, d),
        }
    }

    /// Store one class declaration's raw [`NameSpec`] in the type table.
    /// No FQN is derived here — names materialize at read time via
    /// [`JniGen::fqn_of`], against whatever the settings are then.
    fn register_class_name(&mut self, key: &TypeKey, spec: NameSpec) {
        // Early failure for a bad per-decl `.name()`: the FQN itself is only
        // derived at write time, but a dotted relative name is a declaration
        // mistake and should surface in the declaring call (the same check
        // `resolve_class_fqn` repeats at derivation time).
        if let NameSpec {
            name_override: Some(n),
            ..
        } = &spec
        {
            assert!(
                !n.contains('.'),
                "Kotlin class name `{}` must be relative (no dots) — FQNs are derived from the \
                 base package + subpackage",
                n
            );
        }
        let entry = self.types.entry(key.clone()).or_default();
        entry.class_decl = true;
        entry.name_spec = Some(spec);
    }

    /// Merge a decl's interface options into the type's [`TypeConfig`]
    /// (reopened decls merge; the `.interface()` switch and name override are
    /// sticky-OR / last-wins, a repeated `.implements` interface is
    /// idempotent).
    fn store_iface_opts(&mut self, key: &TypeKey, iface: IfaceOpts) {
        let cfg = self
            .types
            .get_mut(key)
            .expect("register_class_name created the entry");
        cfg.interface_enabled |= iface.enabled;
        if iface.name_override.is_some() {
            cfg.interface_name_override = iface.name_override;
        }
        for i in iface.implements {
            if !cfg.interfaces.contains(&i) {
                cfg.interfaces.push(i);
            }
        }
    }

    fn accept_ptr_class(&mut self, subpackage: &str, decl: PtrClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        self.register_class_name(
            &key,
            NameSpec {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Ptr,
            },
        );
        self.types
            .get_mut(&key)
            .expect("register_class_name created the entry")
            .opaque
            .get_or_insert_with(OpaqueConfig::default);
        self.store_iface_opts(&key, decl.iface);
        self.accept_members(&key, decl.members);
    }

    fn accept_enum_class(&mut self, subpackage: &str, decl: EnumClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        assert!(
            self.types
                .get(&key)
                .is_none_or(|entry| entry.opaque.is_none()),
            "EnumClassDecl: `{}` is already registered as an opaque handle via a PtrClassDecl — \
             a type can be one or the other, not both",
            short
        );
        self.register_class_name(
            &key,
            NameSpec {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Enum,
            },
        );
        self.types
            .get_mut(&key)
            .expect("register_class_name created the entry")
            .enum_cfg = Some(EnumConfig::default());
        self.store_iface_opts(&key, decl.iface);
    }

    fn data_value_name_spec(
        subpackage: &str,
        short: String,
        name_override: Option<String>,
    ) -> NameSpec {
        NameSpec {
            subpackage: subpackage.to_string(),
            short,
            name_override,
            kind: NameKind::DataOrValue,
        }
    }

    fn accept_data_class(&mut self, subpackage: &str, decl: DataClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        let spec = Self::data_value_name_spec(subpackage, short, decl.name_override);
        self.register_class_name(&key, spec);
        self.store_iface_opts(&key, decl.iface);
        self.accept_members(&key, decl.members);
    }

    fn accept_value_class(&mut self, subpackage: &str, decl: ValueClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        let spec = Self::data_value_name_spec(subpackage, short, decl.name_override);
        self.register_class_name(&key, spec);
        self.types
            .get_mut(&key)
            .expect("register_class_name created the entry")
            .value_blob = true;
        self.store_iface_opts(&key, decl.iface);
        self.accept_members(&key, decl.members);
    }

    /// Shared tail of the member-bearing class kinds (`ptr` / `value` /
    /// `data` — every kind whose instance can re-enter Rust): each member's
    /// per-fn expand overrides apply exactly as a free function's would; a
    /// constructor member's return is additionally never output-flattened
    /// (it's a factory); then the members join the class's registered set.
    fn accept_members(&mut self, key: &TypeKey, members: Vec<(FunctionDecl, MemberKind)>) {
        for (decl, kind) in members {
            let rust_ident = decl.rust_ident.clone();
            let kotlin_name_override = decl.kotlin_name_override.clone();
            self.accept_fn_expands(decl);
            if kind == MemberKind::Constructor {
                self.deconstructors
                    .add_skip_default_output(rust_ident.clone());
            }
            self.class_members
                .entry(key.clone())
                .or_default()
                .push(ClassMember {
                    rust_ident,
                    kotlin_name_override,
                    kind,
                });
        }
    }

    fn accept_function(&mut self, subpackage: &str, decl: FunctionDecl) {
        let mut entry = MethodEntry::new(decl.rust_ident.clone());
        entry.kotlin_name_override = decl.kotlin_name_override.clone();
        self.packages
            .entry(subpackage.to_string())
            .or_default()
            .functions
            .push(entry);
        self.accept_fn_expands(decl);
    }

    /// Move a [`FunctionDecl`]'s per-fn expand overrides
    /// (`.expand_param(name, …)` / `.expand_return(…)`) into raw storage.
    /// Shared by [`Self::accept_function`] (free package fns) and
    /// [`Self::accept_members`] (class members) — the overrides mean the same
    /// thing in both positions. Nothing is lowered here: variant/field lists
    /// are interpreted at the point of use ([`Self::build_expansions`] /
    /// [`Self::build_deconstructors`]) so field-name inheritance and the
    /// rust-side-only checks see the complete declaration set.
    fn accept_fn_expands(&mut self, decl: FunctionDecl) {
        let FunctionDecl {
            rust_ident,
            kotlin_name_override: _,
            param_expands,
            return_expand,
        } = decl;
        for (param, pdecl) in param_expands {
            self.fn_param_expands
                .push((rust_ident.clone(), param, pdecl));
        }
        if let Some(rdecl) = return_expand {
            self.fn_return_expands.push((rust_ident.clone(), rdecl));
        }
    }
}

// ── Accepting boundary decls ─────────────────────────────────────────────

impl JniGen {
    /// Declare a type's **default boundary behavior** — either of the two
    /// [`ExpandDecl`] directions, the direction carried by the decl object
    /// (the boundary-decl peer of [`PackageDecl::class`]):
    ///
    /// * [`expand_param!`](crate::expand_param) — the input side: how a
    ///   parameter of the type may be supplied, as an OR-list of build
    ///   variants.
    /// * [`expand_return!`](crate::expand_return) — the output side: the
    ///   AND-set of fields a returned / callback-delivered / `Result`-error
    ///   value of the type decomposes into.
    ///
    /// Applies to every function mentioning the type, in any package; a
    /// single function overrides via the [`FunctionDecl`] `param_expand*` /
    /// `return_expand*` methods.
    pub fn expand(mut self, decl: impl Into<ExpandDecl>) -> Self {
        match decl.into() {
            ExpandDecl::Param(decl) => {
                assert!(
                    !decl.variants.is_empty(),
                    "expand_param!({}) declares no variants — add .variant(fun!(...)) and/or \
                     .variant_self()",
                    decl.key.as_str()
                );
                self.param_expand_decls.push(decl);
            }
            ExpandDecl::Return(decl) => {
                assert!(
                    !decl.fields.is_empty(),
                    "expand_return!({}) declares no fields — add .field(fun!(...)) and/or \
                     .field_self()",
                    decl.key.as_str()
                );
                self.return_expand_decls.push(decl);
            }
        }
        self
    }

    /// The Kotlin name of `func` as a declared member (`.fun`/`.constructor`)
    /// of the class keyed by `key`, if it is one — the name-inheritance
    /// source for [`ExpandReturnDecl::field`].
    fn member_kotlin_name(&self, key: &TypeKey, func: &syn::Ident) -> Option<String> {
        self.class_members
            .get(key)?
            .iter()
            .find(|m| &m.rust_ident == func)
            .map(|m| self.effective_member_name(key, m))
    }

    /// The effective Kotlin name of a class member, derived at point of
    /// use (order-independent w.r.t. `set_member_name_mangle`): the
    /// per-member `.name()` override verbatim, else the member-mangle hook
    /// over the **namespace-relative** camelCase default — the class's
    /// Rust-name prefix is stripped from the fn ident first
    /// ([`strip_type_prefix`]), because a flat crate spells the type
    /// namespace inside the ident while a Kotlin member already lives in
    /// its class. No prefix match ⇒ the full ident camel-cases as before.
    pub(crate) fn effective_member_name(&self, key: &TypeKey, m: &ClassMember) -> String {
        if let Some(name) = &m.kotlin_name_override {
            return name.clone();
        }
        let ident = m.rust_ident.to_string();
        let short = rust_short_name(key);
        let base = crate::api::lang::jnigen::util::strip_type_prefix(&ident, &short)
            .unwrap_or(ident.as_str());
        self.mangle_member(&snake_to_camel(base))
    }

    /// Whether `key` was declared as a class in some package (any of the four
    /// class kinds). A boundary decl on a type without a class declaration
    /// makes it **rust-side-only**: the value is always built from
    /// ingredients / decomposed into fields at the boundary and never
    /// materializes in Kotlin — so the `_self` arms are structurally
    /// impossible for it.
    fn is_class_declared(&self, key: &TypeKey) -> bool {
        self.types.get(key).is_some_and(|c| c.class_decl)
    }

    /// Assemble the full [`Expansions`] set at the point of use: the eagerly
    /// accumulated per-fn overrides plus the raw type-level
    /// [`ExpandParamDecl`]s. Building on demand keeps declarations
    /// order-independent — a `param_expand` may precede or follow the
    /// `package` that declares its constructors (which is also why the
    /// rust-side-only `_self` check lives here and not at accept time).
    pub(crate) fn build_expansions(&self) -> crate::api::core::expand::Expansions {
        let mut exp = self.expansions.clone();
        for decl in &self.param_expand_decls {
            assert!(
                self.is_class_declared(&decl.key)
                    || !decl
                        .variants
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "expand_param!({k}).variant_self(): `{k}` has no class declaration, so there is \
                 no Kotlin object to pass — drop .variant_self() (the type is rust-side-only) \
                 or declare the type in a package",
                k = decl.key.as_str()
            );
            // Identity-only normalization: `.variant_self()` alone declares
            // the plain-handle form — exactly the default when nothing is
            // declared, so registering it would only add a degenerate
            // 1-variant selector to every param of this type.
            if matches!(decl.variants.as_slice(), [LocalVariant::SelfIdentity]) {
                continue;
            }
            exp.ensure_default_constructor(decl.key.to_type());
            for v in &decl.variants {
                match v {
                    LocalVariant::Ctor(f) => exp.add_constructor_variant(f.clone()),
                    LocalVariant::SelfIdentity => exp.add_constructor_variant_id(),
                }
            }
        }
        // Per-fn overrides: same decl shape, complete-set semantics; the
        // param-name/type cross-check and the identity-only lowering happen
        // in `core/expand.rs`'s `apply` (which sees the fn signatures).
        for (func, param, decl) in &self.fn_param_expands {
            assert!(
                self.is_class_declared(&decl.key)
                    || !decl
                        .variants
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "fun!({func}).expand_param(\"{param}\", expand_param!({k}).variant_self()): `{k}` \
                 has no class declaration, so there is no Kotlin object to pass — drop \
                 .variant_self() (the type is rust-side-only) or declare the type in a package",
                k = decl.key.as_str()
            );
            let param_ident = syn::Ident::new(param, Span::call_site());
            exp.begin_subset(func.clone(), param_ident, decl.key.to_type());
            for v in &decl.variants {
                match v {
                    LocalVariant::Ctor(f) => exp.push_subset_variant(f.clone()),
                    LocalVariant::SelfIdentity => exp.push_subset_self(),
                }
            }
        }
        exp
    }

    /// Assemble the full [`Deconstructors`] set at the point of use — the
    /// output-side peer of [`Self::build_expansions`]. Field names resolve
    /// here, against the complete declaration set: explicit `.name()` first,
    /// then the class member's Kotlin name (a getter that is both a method
    /// and a field is named once, on the member), else the camel-cased Rust
    /// name.
    pub(crate) fn build_deconstructors(&self) -> crate::api::core::unfold::Deconstructors {
        let mut dec = self.deconstructors.clone();
        for decl in &self.return_expand_decls {
            assert!(
                self.is_class_declared(&decl.key)
                    || !decl
                        .fields
                        .iter()
                        .any(|f| matches!(f, LocalField::SelfField)),
                "expand_return!({k}).field_self(): `{k}` has no class declaration, so there is \
                 no Kotlin object to deliver — drop .field_self() (the type is rust-side-only) \
                 or declare the type in a package",
                k = decl.key.as_str()
            );
            dec.ensure_default_deconstructor(decl.key.to_type());
            for f in &decl.fields {
                match f {
                    LocalField::Named(func, name_override) => {
                        let name = name_override
                            .clone()
                            .or_else(|| self.member_kotlin_name(&decl.key, func))
                            .unwrap_or_else(|| snake_to_camel(&func.to_string()));
                        dec.add_deconstructor_record(func.clone(), name);
                    }
                    LocalField::SelfField => dec.add_deconstructor_record_id(),
                }
            }
        }
        // Per-fn overrides: same decl shape and name inheritance; the
        // return-type cross-check and the identity-only lowering happen in
        // `core/unfold.rs`'s `apply` (which sees the fn signatures).
        for (func, decl) in &self.fn_return_expands {
            assert!(
                self.is_class_declared(&decl.key)
                    || !decl
                        .fields
                        .iter()
                        .any(|f| matches!(f, LocalField::SelfField)),
                "fun!({func}).expand_return(expand_return!({k}).field_self()): `{k}` has no \
                 class declaration, so there is no Kotlin object to deliver — drop \
                 .field_self() (the type is rust-side-only) or declare the type in a package",
                k = decl.key.as_str()
            );
            dec.begin_inline_output(func.clone(), decl.key.to_type());
            for f in &decl.fields {
                match f {
                    LocalField::Named(afunc, name_override) => {
                        let name = name_override
                            .clone()
                            .or_else(|| self.member_kotlin_name(&decl.key, afunc))
                            .unwrap_or_else(|| snake_to_camel(&afunc.to_string()));
                        dec.push_inline_field(afunc.clone(), name);
                    }
                    LocalField::SelfField => dec.push_inline_field_self(),
                }
            }
        }
        dec
    }

    /// Type keys of boundary decls (`expand_param!` / `expand_return!`,
    /// type-level and per-fn) whose type has no class declaration — the
    /// **rust-side-only** types. Unioned into [`Prebindgen::ignored_types`]
    /// so the registry treats them as acknowledged (no "skipping undeclared"
    /// warning, no direct converter requirement, no Kotlin emission).
    pub(crate) fn rust_side_only_types(&self) -> impl Iterator<Item = TypeKey> + '_ {
        self.param_expand_decls
            .iter()
            .map(|d| &d.key)
            .chain(self.return_expand_decls.iter().map(|d| &d.key))
            .chain(self.fn_param_expands.iter().map(|(_, _, d)| &d.key))
            .chain(self.fn_return_expands.iter().map(|(_, d)| &d.key))
            .filter(|k| !self.is_class_declared(k))
            .cloned()
    }

    /// Function idents referenced only inside boundary decls (type-level and
    /// per-fn) — `expand_return!` field accessors and `expand_param!` variant
    /// ctors. They are called Rust-side by the generated fold/unfold code and
    /// need no extern of their own; when not otherwise declared they are
    /// unioned into [`Prebindgen::ignored_functions`] so the registry's
    /// "skipping undeclared fn" warning stays quiet.
    pub(crate) fn boundary_referenced_fns(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        let ctors = self
            .param_expand_decls
            .iter()
            .map(|d| &d.variants)
            .chain(self.fn_param_expands.iter().map(|(_, _, d)| &d.variants))
            .flatten()
            .filter_map(|v| match v {
                LocalVariant::Ctor(f) => Some(f.clone()),
                LocalVariant::SelfIdentity => None,
            });
        let accessors = self
            .return_expand_decls
            .iter()
            .map(|d| &d.fields)
            .chain(self.fn_return_expands.iter().map(|(_, d)| &d.fields))
            .flatten()
            .filter_map(|f| match f {
                LocalField::Named(func, _) => Some(func.clone()),
                LocalField::SelfField => None,
            });
        ctors.chain(accessors)
    }

    /// Every function referenced as a named field in any `expand_return!`
    /// decl (type-level or per-fn) — the accessor set. Backs
    /// [`Prebindgen::accessor_functions`]: `core/unfold.rs`'s deconstructor
    /// gate requires every named record's function to be in this set
    /// (`RecordNotAccessor` otherwise), and `core/expand.rs` excludes them
    /// from parameter composition. Derived from *usage* — a function need not
    /// also be a `.fun()` class member to be referenced this way.
    pub(crate) fn field_accessor_fns(&self) -> std::collections::HashSet<syn::Ident> {
        self.return_expand_decls
            .iter()
            .map(|d| &d.fields)
            .chain(self.fn_return_expands.iter().map(|(_, d)| &d.fields))
            .flatten()
            .filter_map(|f| match f {
                LocalField::Named(func, _) => Some(func.clone()),
                LocalField::SelfField => None,
            })
            .collect()
    }
}

// ── Accepting the convert decl ───────────────────────────────────────────

impl JniGen {
    /// Declare a type's **canonical single-value conversion** (a
    /// [`ConvertDecl`], built with [`convert!`](crate::convert)): a pair of
    /// `#[prebindgen]` functions carrying one value of the type across the
    /// boundary wherever a single value is needed (params, returns,
    /// `Option`/`Vec` elements, the `Result<T, E>` success position,
    /// `data_class` fields). Applies wherever the type appears; not tied to
    /// any package. See [`ConvertDecl`] for the relation to the
    /// [`expand`](Self::expand) boundary decls.
    pub fn convert(mut self, decl: ConvertDecl) -> Self {
        assert!(
            decl.input.is_some() || decl.output.is_some(),
            "convert!({}) declares no conversions — add .input(fun!(...)) and/or \
             .output(fun!(...))",
            decl.key.as_str()
        );
        self.convert_decls.push(decl);
        self
    }

    /// Derive the rank-0 **input** converter body for a `convert!`-declared
    /// type: `(continue_ty, exc, body)` where `continue_ty` is the conversion
    /// fn's parameter type (by value) — the composed-converter machinery
    /// chains it through that type's own converter, so the wire and the
    /// Kotlin surface derive from it. Consulted by [`Self::lookup_input`]
    /// before the wrapper tables; signatures are read from the registry at
    /// lookup time (order-independent, and multi-source qualification via
    /// [`Self::fn_module`]).
    pub(crate) fn convert_input_body(
        &self,
        key: &TypeKey,
        registry: &Registry<KotlinMeta>,
    ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
        let decl = self.convert_decls.iter().find(|d| &d.key == key)?;
        let target = key.to_type();
        match decl.input.as_ref()? {
            ConvertSpec::PrebindgenFn(f) => {
                let (item_fn, _) = registry.functions.get(f).unwrap_or_else(|| {
                    panic!(
                        "convert!({}).input({f}): function not found among #[prebindgen] items",
                        key.as_str()
                    )
                });
                let (param_ty, by_ref) = convert_single_param(key, f, item_fn, "input");
                // Return: `T` (infallible) or `Result<T, E>` (fallible — E
                // routes to the caller's error handler via the exc slot).
                let ret = fn_return_type(item_fn);
                let (ok_ty, exc) = match crate::api::core::types_util::result_ok_type(&ret) {
                    Some(ok) => (
                        ok,
                        Some(
                            crate::api::core::types_util::result_err_type(&ret)
                                .expect("result_ok_type implies result_err_type"),
                        ),
                    ),
                    None => (ret, None),
                };
                assert!(
                    TypeKey::from_type(&ok_ty) == *key,
                    "convert!({k}).input({f}): the function produces `{got}`, not `{k}`",
                    k = key.as_str(),
                    got = TypeKey::from_type(&ok_ty).as_str()
                );
                let module = self.fn_module(registry, f);
                let body: syn::Expr = if by_ref {
                    syn::parse_quote!(#module::#f(&v))
                } else {
                    syn::parse_quote!(#module::#f(v))
                };
                Some((param_ty, exc, body))
            }
            // `Into`/`TryInto` impls: the repr is stated in the decl; the
            // fully-qualified call form pins both type parameters so the
            // right impl is selected regardless of what else is in scope.
            ConvertSpec::Trait { repr, fallible } => {
                if *fallible {
                    let exc: syn::Type = syn::parse_quote!(
                        <#repr as ::core::convert::TryInto<#target>>::Error
                    );
                    let body: syn::Expr = syn::parse_quote!(
                        <#repr as ::core::convert::TryInto<#target>>::try_into(v)
                    );
                    Some((repr.clone(), Some(exc), body))
                } else {
                    let body: syn::Expr = syn::parse_quote!(
                        <#repr as ::core::convert::Into<#target>>::into(v)
                    );
                    Some((repr.clone(), None, body))
                }
            }
            // Binding-local callable: emitted verbatim (multi-segment paths
            // pass the qualification visitor untouched). With a declared
            // error type the fn returns `Result<T, E>` — emitted as-is, `E`
            // riding the standard exc slot.
            ConvertSpec::LocalFn { repr, path, error } => {
                let body: syn::Expr = syn::parse_quote!(#path(v));
                Some((repr.clone(), error.clone(), body))
            }
        }
    }

    /// Output-direction peer of [`Self::convert_input_body`]: the conversion
    /// fn takes `&T` (or `T`) and returns the continue type.
    pub(crate) fn convert_output_body(
        &self,
        key: &TypeKey,
        registry: &Registry<KotlinMeta>,
    ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
        let decl = self.convert_decls.iter().find(|d| &d.key == key)?;
        let target = key.to_type();
        match decl.output.as_ref()? {
            ConvertSpec::PrebindgenFn(g) => {
                let (item_fn, _) = registry.functions.get(g).unwrap_or_else(|| {
                    panic!(
                        "convert!({}).output({g}): function not found among #[prebindgen] items",
                        key.as_str()
                    )
                });
                let (param_ty, by_ref) = convert_single_param_any(g, item_fn);
                assert!(
                    TypeKey::from_type(&param_ty) == *key,
                    "convert!({k}).output({g}): the function takes `{got}`, not `{k}`",
                    k = key.as_str(),
                    got = TypeKey::from_type(&param_ty).as_str()
                );
                let ret = fn_return_type(item_fn);
                assert!(
                    TypeKey::from_type(&ret) != *key,
                    "convert!({k}).output({g}): the function must return the converted form, \
                     not `{k}`",
                    k = key.as_str()
                );
                let module = self.fn_module(registry, g);
                let body: syn::Expr = if by_ref {
                    syn::parse_quote!(#module::#g(&v))
                } else {
                    syn::parse_quote!(#module::#g(v))
                };
                Some((ret, None, body))
            }
            ConvertSpec::Trait { repr, fallible } => {
                if *fallible {
                    let exc: syn::Type = syn::parse_quote!(
                        <#target as ::core::convert::TryInto<#repr>>::Error
                    );
                    let body: syn::Expr = syn::parse_quote!(
                        <#target as ::core::convert::TryInto<#repr>>::try_into(v)
                    );
                    Some((repr.clone(), Some(exc), body))
                } else {
                    let body: syn::Expr = syn::parse_quote!(
                        <#target as ::core::convert::Into<#repr>>::into(v)
                    );
                    Some((repr.clone(), None, body))
                }
            }
            ConvertSpec::LocalFn { repr, path, error } => {
                let body: syn::Expr = syn::parse_quote!(#path(v));
                Some((repr.clone(), error.clone(), body))
            }
        }
    }

    /// Idents of every `#[prebindgen]`-fn conversion source — scanned as
    /// helper functions ([`Prebindgen::helper_functions`]) so their extern
    /// emission is suppressed. Trait/local-fn sources have no registry item.
    pub(crate) fn convert_fns(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        self.convert_decls
            .iter()
            .flat_map(|d| d.input.iter().chain(d.output.iter()))
            .filter_map(|spec| match spec {
                ConvertSpec::PrebindgenFn(f) => Some(f.clone()),
                _ => None,
            })
    }
}

/// The single typed parameter of a conversion fn, peeled of a leading `&`;
/// asserts arity 1. Returns `(peeled_type, was_by_ref)`.
fn convert_single_param_any(f: &syn::Ident, item_fn: &syn::ItemFn) -> (syn::Type, bool) {
    let params: Vec<&syn::PatType> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            syn::FnArg::Typed(pt) => Some(pt),
            _ => None,
        })
        .collect();
    assert!(
        params.len() == 1,
        "convert fn `{f}` must take exactly one parameter, it takes {}",
        params.len()
    );
    match &*params[0].ty {
        syn::Type::Reference(r) => ((*r.elem).clone(), true),
        other => (other.clone(), false),
    }
}

/// [`convert_single_param_any`] + the direction-specific error context.
fn convert_single_param(
    key: &TypeKey,
    f: &syn::Ident,
    item_fn: &syn::ItemFn,
    dir: &str,
) -> (syn::Type, bool) {
    let (ty, by_ref) = convert_single_param_any(f, item_fn);
    assert!(
        TypeKey::from_type(&ty) != *key,
        "convert!({k}).{dir}({f}): the function must take the converted form, not `{k}` itself",
        k = key.as_str()
    );
    (ty, by_ref)
}

/// A fn's return type (`()` for none).
fn fn_return_type(item_fn: &syn::ItemFn) -> syn::Type {
    match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => (**t).clone(),
    }
}

impl JniGen {
    /// Build a `KotlinMeta` carrying just the value-context Kotlin name.
    /// Used by every built-in converter (primitives, structs, `Option<_>`,
    /// `Vec<_>`, `impl Fn(...)` lambdas). Errors are routed uniformly to the
    /// per-call `signal_error` sink by the extern emitter, so no
    /// per-converter exception metadata is carried.
    pub(crate) fn framework_meta(&self, kotlin_name: Option<kt::KtType>) -> KotlinMeta {
        KotlinMeta {
            kotlin_name,
            value_rust_key: None,
            projection: None,
        }
    }

    // ── Wrapper-table lookups (used by Prebindgen impl) ───────────

    /// Look up a registered input converter for `pat` with `args`
    /// substituted into its `_` slots. The closure's middle slot (see
    /// [`WrapperFn`]) carries the bound exception — `None` ⇒ framework
    /// `__JniErr` with an `Ok`-wrap, `Some(<Rust type>)` ⇒
    /// `Result<ty, <Rust type>>` emitted verbatim, decided in
    /// [`Self::build_input_fn`].
    ///
    /// The closure's returned type is classified by [`is_wire_type`]:
    /// * **wire** ⇒ terminal: a single converter `wire → outer`.
    /// * **rust type** ⇒ composed: that type's input converter runs
    ///   first (`wire → ty`), then this registration's body is a
    ///   value-inspecting stage `ty → outer` (built by-value via
    ///   [`Self::build_output_fn`]) prepended to the inner chain. Defer
    ///   (`None`) if the inner converter isn't resolved yet.
    ///
    /// Structurally match `ty` against every registered **input** wrapper
    /// pattern, most-specific-first (fewest wildcards win, e.g.
    /// `Result<_, ConcreteErr>` over `Result<_, _>`), and build the first hit.
    pub(crate) fn match_user_input(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        for pat in self.ordered_input_patterns() {
            if let Some(args) = crate::api::core::types_util::match_pattern(ty, &pat) {
                if let Some(c) = self.lookup_input(&pat, &args, registry) {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Output-direction peer of [`Self::match_user_input`].
    pub(crate) fn match_user_output(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        for pat in self.ordered_output_patterns() {
            if let Some(args) = crate::api::core::types_util::match_pattern(ty, &pat) {
                if let Some(c) = self.lookup_output(&pat, &args, registry) {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Registered input-wrapper patterns, ordered most-specific (fewest
    /// wildcards) first; ties keep registration-independent but stable order
    /// (by canonical key) so resolution is deterministic.
    fn ordered_input_patterns(&self) -> Vec<syn::Type> {
        ordered_patterns(&self.input_wrappers)
    }
    fn ordered_output_patterns(&self) -> Vec<syn::Type> {
        ordered_patterns(&self.output_wrappers)
    }

    pub(crate) fn lookup_input(
        &self,
        pat: &syn::Type,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let rank = args.len();
        if rank > 3 {
            return None;
        }
        let key = TypeKey::from_type(pat);
        // A `convert!`-declared conversion takes precedence at rank 0 (its
        // signature-derived body is equivalent to a rank-0 registration,
        // just computed at the point of use).
        let (ty, exc_ty, body) = match if rank == 0 {
            self.convert_input_body(&key, registry)
        } else {
            None
        } {
            Some(t) => t,
            None => {
                let f = self.input_wrappers[rank].get(&key)?;
                f(args, registry)?
            }
        };
        // The closure's middle slot carries the `Result`'s raw Rust error
        // type (or `None` for the framework `__JniErr`); it feeds the
        // converter signature `Result<_, E>` directly — no registration.
        let exc = exc_ty.as_ref();
        let outer = substitute_wildcards(pat, args);
        // Terminal vs composed: `ty` is composed iff it's a *distinct*
        // rust type with its own input converter. The self-check guards
        // the void/identity case, and the registered-converter probe
        // distinguishes a rust continue-type (compose) from a wire
        // (terminal) without forcing `()` either way. A non-wire `ty` that
        // isn't yet resolved defers.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(&outer);
        let inner = if is_self {
            None
        } else {
            registry.input_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces `outer`.
                let (niches, kotlin_name) = if rank == 0 {
                    let kn = self
                        .types
                        .get(&key)
                        .and_then(|c| c.name_spec.as_ref())
                        .map(|s| kt::KtType::cls(self.fqn_of(s)))
                        .or_else(|| kotlin_for_wire(&ty));
                    (Niches::empty(), kn)
                } else {
                    (default_niches_for_wire(&ty), None)
                };
                Some(ConverterImpl {
                    subs: vec![],
                    pre_stages: vec![],
                    function: self.build_input_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key: None,
                        // Terminal: body produces the wire directly, no inner
                        // converter composed, so no handle to carry.
                        projection: None,
                    },
                })
            }
            // Non-wire `ty` whose converter isn't resolved yet — defer.
            None => None,
            Some(inner) => {
                // Composed: `ty` is the inner source rust type. Its input
                // converter (`wire → ty`) is the wire-facing function;
                // this body is a stage `ty → outer` that runs after it.
                // The stage takes the inner-produced value BY VALUE and
                // yields `outer`, i.e. the same shape an output converter
                // has — so it's built with `build_output_fn`.
                let stage = Stage {
                    function: self.build_output_fn(&ty, &outer, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    (
                        inner.metadata.kotlin_name.clone(),
                        Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                    )
                } else {
                    (inner.metadata.kotlin_name.clone(), None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&inner.destination)
                };
                Some(ConverterImpl {
                    subs: vec![],
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Identity propagation: a composed wrapper (e.g.
                        // `Result<Handle,Error>`) projects to its inner value,
                        // so a handle inner stays a handle (same strategy).
                        projection: inner.metadata.projection.clone(),
                    },
                })
            }
        }
    }

    /// Look up a registered output converter for `pat` with `args`
    /// substituted into its `_` slots. Mirror of [`Self::lookup_input`].
    ///
    /// The closure's returned type is classified by [`is_wire_type`]:
    /// * **wire** ⇒ terminal: a single converter `outer → wire`,
    ///   returning `Result<wire, err>` (throwing iff exc is set).
    /// * **rust type** ⇒ composed: this body is a value-inspecting stage
    ///   `outer → ty` prepended to `ty`'s own output converter chain
    ///   (e.g. `ZResult<T>` returns rust `T`, so the peel stage raises
    ///   its exception and `T`'s converter marshals the wire). Defer
    ///   (`None`) if `ty`'s converter isn't resolved yet.
    pub(crate) fn lookup_output(
        &self,
        pat: &syn::Type,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let rank = args.len();
        if rank > 3 {
            return None;
        }
        let key = TypeKey::from_type(pat);
        // A `convert!`-declared conversion takes precedence at rank 0 — see
        // [`Self::lookup_input`].
        let (ty, exc_ty, body) = match if rank == 0 {
            self.convert_output_body(&key, registry)
        } else {
            None
        } {
            Some(t) => t,
            None => {
                let f = self.output_wrappers[rank].get(&key)?;
                f(args, registry)?
            }
        };
        // The closure's middle slot carries the `Result`'s raw Rust error
        // type (or `None` for the framework `__JniErr`) — see lookup_input.
        let exc = exc_ty.as_ref();
        let outer = substitute_wildcards(pat, args);
        // Terminal vs composed — see [`Self::lookup_input`] for the rule.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(&outer);
        let inner = if is_self {
            None
        } else {
            registry.output_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces it from `outer`.
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    registry
                        .output_entry(&args[0])
                        .map(|e| {
                            (
                                e.metadata.kotlin_name.clone(),
                                Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                            )
                        })
                        .unwrap_or((None, None))
                } else {
                    let kn = self
                        .types
                        .get(&key)
                        .and_then(|c| c.name_spec.as_ref())
                        .map(|s| kt::KtType::cls(self.fqn_of(s)))
                        .or_else(|| kotlin_for_wire(&ty));
                    (kn, None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&ty)
                };
                Some(ConverterImpl {
                    subs: vec![],
                    pre_stages: vec![],
                    function: self.build_output_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Terminal: body produces the wire directly, no inner
                        // converter composed, so no handle to carry.
                        projection: None,
                    },
                })
            }
            // Non-wire `ty` whose converter isn't resolved yet — defer.
            None => None,
            Some(inner) => {
                // Composed: `ty` is the continue rust type; chain its converter.
                let stage = Stage {
                    function: self.build_output_fn(&outer, &ty, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    (
                        inner.metadata.kotlin_name.clone(),
                        Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                    )
                } else {
                    (inner.metadata.kotlin_name.clone(), None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&inner.destination)
                };
                Some(ConverterImpl {
                    subs: vec![],
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Identity propagation: a composed wrapper (e.g.
                        // `Result<Handle,Error>`) projects to its inner value,
                        // so a handle inner stays a handle (same strategy).
                        projection: inner.metadata.projection.clone(),
                    },
                })
            }
        }
    }
}

/// Recognise the JNI **wire** shapes a converter body may return as a
/// terminal destination. Reuses the back-end's existing wire knowledge:
/// every `jni::sys::*` / `jni::objects::*` wire is recognised by
/// [`kotlin_for_wire`] (returns `Some`), plus
/// raw pointers structurally — so there is no separate wire-type
/// allowlist to keep in sync.
///
/// `()` is deliberately **not** treated as a wire here: it is ambiguous
/// (the void wire of a self-converter *and* the unit continue-type of
/// `ZResult<()>`). The terminal-vs-composed decision in
/// [`JniGen::lookup_input`] / [`JniGen::lookup_output`] resolves that
/// ambiguity via the self-check + registered-converter probe, so `()`
/// flows correctly without being force-classified here.
pub(crate) fn is_wire_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Ptr(_)) || kotlin_for_wire(ty).is_some()
}

/// Bare-ident type `__JniErr` — the generated file's alias for the
/// framework [`crate::api::lang::jnigen::jni::JniBindingError`]. Built-in
/// converters use this as their `Result<…, _>` error type so their bodies'
/// `<__JniErr as From<String>>::from(...)` calls keep compiling. A
/// `Result<T, E>` return instead binds its own raw `E` (see
/// [`JniGen::lookup_output`]); the extern's `Err` arm funnels both to the
/// per-call `signal_error` sink via `E: Display`.
pub(crate) fn default_err_type() -> syn::Type {
    syn::parse_quote!(__JniErr)
}

/// The actual framework error type the `__JniErr` alias resolves to: the
/// E-agnostic `JniBindingError<()>` whose failures are always `JniError`
/// (binding-layer). A `Result<T, E>` return carries its own raw `E`, surfaced
/// as `UserError` at the extern's error site.
pub(crate) fn framework_error_type() -> syn::Type {
    syn::parse_quote!(::prebindgen::lang::JniBindingError<()>)
}

/// The body expression to splice into a converter `fn` returning
/// `Result<_, E>`: with `exc = None` the `body` is a bare value, so wrap
/// it `Ok(body)`; with `exc = Some(E)` the `body` already evaluates to
/// the `Result`, so emit it verbatim.
pub(crate) fn body_for_exc(body: &syn::Expr, exc: Option<&syn::Type>) -> syn::Expr {
    if exc.is_some() {
        body.clone()
    } else {
        syn::parse_quote!(Ok(#body))
    }
}

/// Substitute the wildcard `_` slots of `pat` with `args` (left-to-right
/// depth-first), returning the concrete outer `syn::Type`. Mirrors the
/// substitution the resolver performs to derive a wildcard pattern from
/// a concrete type.
pub(crate) fn substitute_wildcards(pat: &syn::Type, args: &[syn::Type]) -> syn::Type {
    let mut idx = 0usize;
    fn walk(ty: &mut syn::Type, args: &[syn::Type], idx: &mut usize) {
        match ty {
            syn::Type::Infer(_) => {
                if let Some(replacement) = args.get(*idx) {
                    *ty = replacement.clone();
                }
                *idx += 1;
            }
            syn::Type::Path(tp) => {
                for seg in &mut tp.path.segments {
                    if let syn::PathArguments::AngleBracketed(ab) = &mut seg.arguments {
                        for arg in &mut ab.args {
                            if let syn::GenericArgument::Type(inner) = arg {
                                walk(inner, args, idx);
                            }
                        }
                    }
                }
            }
            syn::Type::Reference(r) => walk(&mut r.elem, args, idx),
            syn::Type::Tuple(t) => {
                for e in &mut t.elems {
                    walk(e, args, idx);
                }
            }
            syn::Type::Array(a) => walk(&mut a.elem, args, idx),
            syn::Type::Slice(s) => walk(&mut s.elem, args, idx),
            syn::Type::Ptr(p) => walk(&mut p.elem, args, idx),
            syn::Type::Paren(p) => walk(&mut p.elem, args, idx),
            syn::Type::Group(g) => walk(&mut g.elem, args, idx),
            _ => {}
        }
    }
    let mut out = pat.clone();
    walk(&mut out, args, &mut idx);
    out
}

/// Flatten the rank-bucketed wrapper tables into one pattern list ordered
/// most-specific-first: ascending wildcard count (so `Result<_, ConcreteErr>`
/// is tried before `Result<_, _>`), then by canonical key for a deterministic
/// tiebreak independent of `HashMap` iteration order.
fn ordered_patterns(buckets: &[HashMap<TypeKey, WrapperFn>; 4]) -> Vec<syn::Type> {
    let mut keys: Vec<(usize, String, syn::Type)> = buckets
        .iter()
        .flat_map(|m| m.keys())
        .map(|k| {
            let ty = k.to_type();
            (
                crate::api::core::types_util::wildcard_count(&ty),
                k.as_str().to_string(),
                ty,
            )
        })
        .collect();
    keys.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    keys.into_iter().map(|(_, _, ty)| ty).collect()
}
