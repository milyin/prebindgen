//! Builder API for [`JniGen`].
//!
//! [`JniGen::new`] takes a finished [`JniGenConfig`]; from then on `JniGen`
//! only *accepts* pre-built declaration objects (`decl.rs`) via
//! [`JniGen::package`], [`JniGen::scalar_type_wrapper`], and
//! [`JniGen::generic_type_wrapper`] — there is no fluent typestate cursor.
//! Carved from the former monolithic JNI module; shares the `jni` namespace
//! via `use super::*`.

use super::*;

impl JniGen {
    /// Look up the registered Kotlin FQN for a canonical Rust type key
    /// (the inverse of the `(key, fqn)` rows pushed into
    /// [`Self::kotlin_type_fqns`] by the class-decl accept logic).
    pub(crate) fn kotlin_fqn(&self, rust_canon: &str) -> Option<&str> {
        self.kotlin_type_fqns
            .iter()
            .find(|(k, _)| k == rust_canon)
            .map(|(_, v)| v.as_str())
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
    /// Start a binding generator from a finished [`JniGenConfig`] (the
    /// package prefix, native-init hook and name-mangle rules). From here you
    /// add declarations with [`package`](Self::package),
    /// [`scalar_type_wrapper`](Self::scalar_type_wrapper), etc., then run the
    /// result through `Registry::write_rust` / `write_kotlin`. Taking the
    /// whole config up front means it can't change once declarations exist.
    pub fn new(config: JniGenConfig) -> Self {
        let mut jni = Self {
            source_module: config.source_module,
            package: config.package_prefix,
            java_class_prefix: String::new(),
            jni_class_path: String::new(),
            kotlin_fun_name_mangle: config.kotlin_fun_name_mangle,
            kotlin_ptr_class_name_mangle: config.kotlin_ptr_class_name_mangle,
            kotlin_data_class_name_mangle: config.kotlin_data_class_name_mangle,
            kotlin_enum_name_mangle: config.kotlin_enum_name_mangle,
            kotlin_harness_name_mangle: config.kotlin_harness_name_mangle,
            kotlin_type_fqns: Vec::new(),
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
            emit_handle_locks: config.emit_handle_locks,
            jni_native_init: config.jni_native_init,
            expansions: crate::api::core::expand::Expansions::default(),
            deconstructors: crate::api::core::unfold::Deconstructors::default(),
            class_members: HashMap::new(),
            ignored_fns: std::collections::HashSet::new(),
            ignored_class_types: std::collections::HashSet::new(),
            accessor_record_fns: std::collections::HashSet::new(),
        };
        jni.recompute_derived();
        // Built-in rank-2 `Result<_, _>` peel: every Result<T, E> succeeds
        // as T and routes E to the error-sink on Err. Consumers may override
        // per-binding by registering a more specific rank-1
        // `GenericTypeWrapperDecl::new(pq!(Result<_, ConcreteErr>))` (rank-1
        // fires before rank-2 in resolve and short-circuits this).
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

    /// Recompute the derived caches (`java_class_prefix`, `jni_class_path`)
    /// from (`package`, `kotlin_harness_name_mangle`). The JNI extern symbol
    /// path resolves to the centralized Native object, whose mangled name
    /// comes from the harness mangle (default `"JNI" + n` → `JNINative`).
    fn recompute_derived(&mut self) {
        self.java_class_prefix = self.package.replace(".", "/");
        let native_class = self.mangle_harness("Native");
        self.jni_class_path = if self.package.is_empty() {
            format!("Java_{}", native_class)
        } else {
            format!("Java_{}_{}", self.package.replace(".", "_"), native_class)
        };
    }

    /// Apply the fun-name mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset. Called everywhere the framework
    /// derives a function-shaped Kotlin/JNI short name — scanned
    /// `#[prebindgen]` extern symbols, the synthetic `freePtr` destructor,
    /// and the Kotlin-side `external fun` that pairs with each.
    pub(crate) fn mangle_fun(&self, name: &str) -> String {
        match &self.kotlin_fun_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the ptr-class mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset.
    pub(crate) fn mangle_ptr_class(&self, name: &str) -> String {
        match &self.kotlin_ptr_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the data-class mangle closure to `name`, returning the closure
    /// result or `name` verbatim when unset.
    pub(crate) fn mangle_data_class(&self, name: &str) -> String {
        match &self.kotlin_data_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the enum mangle closure to `name`, returning the closure result
    /// or `name` verbatim when unset.
    pub(crate) fn mangle_enum(&self, name: &str) -> String {
        match &self.kotlin_enum_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply the harness mangle closure to `name`. Defaults to
    /// `|n| format!("JNI{n}")` when unset, so `mangle_harness("Native")`
    /// yields `"JNINative"`.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.kotlin_harness_name_mangle {
            Some(f) => f(name),
            None => format!("JNI{name}"),
        }
    }
    /// The mangled name of the centralized Native object that hosts every
    /// JNI `external fun`. Drives both the Kotlin class emission and the
    /// JNI extern symbol path on the Rust side.
    pub(crate) fn jni_native_class_name(&self) -> String {
        self.mangle_harness("Native")
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

// ── Accepting a `PackageDecl` ────────────────────────────────────────────

impl JniGen {
    /// Register a package's worth of classes and functions (a
    /// [`PackageDecl`], built with [`package!`](crate::package)). Call it once
    /// per package, or several times for the same package name — the
    /// declarations merge, so you can split a large package across calls.
    pub fn package(mut self, decl: PackageDecl) -> Self {
        let PackageDecl {
            name,
            classes,
            functions,
        } = decl;
        self.packages.entry(name.clone()).or_default();
        for class in classes {
            self.accept_class(&name, class);
        }
        for func in functions {
            self.accept_function(&name, func);
        }
        self
    }

    /// Acknowledge a `#[prebindgen]` function this binding deliberately does
    /// NOT wrap: nothing is emitted for it and the registry's per-item
    /// "skipping undeclared" warning is suppressed. Global — an ignored fn
    /// belongs to no package. E.g. `.ignore_fun(fun!(string_len))`.
    pub fn ignore_fun(mut self, decl: FunctionDecl) -> Self {
        self.ignored_fns.insert(decl.rust_ident);
        self
    }

    /// Acknowledge a `#[prebindgen]` type this binding deliberately does NOT
    /// declare as a class — the type-level dual of [`Self::ignore_fun`].
    pub fn ignore_class(mut self, rust_type: syn::Type) -> Self {
        self.ignored_class_types
            .insert(TypeKey::from_type(&rust_type));
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

    fn accept_ptr_class(&mut self, subpackage: &str, decl: PtrClassDecl) {
        let short = rust_short_name(&decl.key);
        let fqn = match decl.name_override {
            Some(n) => self.resolve_class_fqn(subpackage, &n),
            None => {
                let mangled = self.mangle_ptr_class(&short);
                self.resolve_class_fqn(subpackage, &mangled)
            }
        };
        let key = decl.key;
        let entry = self.types.entry(key.clone()).or_default();
        entry.class_decl = true;
        entry.opaque = Some(OpaqueConfig::default());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.accept_members(&key, decl.members);

        if let Some(variants) = decl.input_variants {
            // Identity-only normalization: `.default_param_expand_self()`
            // alone declares the plain-handle form — exactly the default
            // when nothing is declared, so registering it would only add a
            // degenerate 1-variant selector to every param of this type.
            if !matches!(variants.as_slice(), [LocalVariant::SelfIdentity]) {
                self.expansions.ensure_default_constructor(key.to_type());
                for v in variants {
                    match v {
                        LocalVariant::Ctor(f) => self.expansions.add_constructor_variant(f),
                        LocalVariant::SelfIdentity => {
                            self.expansions.add_constructor_variant_id()
                        }
                    }
                }
            }
        }
        if let Some(fields) = decl.output_fields {
            self.deconstructors
                .ensure_default_deconstructor(key.to_type());
            for f in fields {
                match f {
                    LocalField::Named(func, name) => {
                        self.accessor_record_fns.insert(func.clone());
                        self.deconstructors.add_deconstructor_record(func, name)
                    }
                    LocalField::SelfField => self.deconstructors.add_deconstructor_record_id(),
                }
            }
        }
    }

    fn accept_enum_class(&mut self, subpackage: &str, decl: EnumClassDecl) {
        let short = rust_short_name(&decl.key);
        let fqn = match decl.name_override {
            Some(n) => self.resolve_class_fqn(subpackage, &n),
            None => {
                let mangled = self.mangle_enum(&short);
                self.resolve_class_fqn(subpackage, &mangled)
            }
        };
        let key = decl.key;
        let entry = self.types.entry(key.clone()).or_default();
        assert!(
            entry.opaque.is_none(),
            "EnumClassDecl: `{}` is already registered as an opaque handle via a PtrClassDecl — \
             a type can be one or the other, not both",
            short
        );
        entry.class_decl = true;
        entry.enum_cfg = Some(EnumConfig::default());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
    }

    fn accept_data_class(&mut self, subpackage: &str, decl: DataClassDecl) {
        let short = rust_short_name(&decl.key);
        let fqn = match (decl.kotlin_type, decl.name_override) {
            (Some(expr), _) => expr,
            (None, Some(n)) => self.resolve_class_fqn(subpackage, &n),
            (None, None) => {
                let mangled = self.mangle_data_class(&short);
                self.resolve_class_fqn(subpackage, &mangled)
            }
        };
        let key = decl.key;
        let entry = self.types.entry(key.clone()).or_default();
        entry.class_decl = true;
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
    }

    fn accept_value_class(&mut self, subpackage: &str, decl: ValueClassDecl) {
        let short = rust_short_name(&decl.key);
        let fqn = match (decl.kotlin_type, decl.name_override) {
            (Some(expr), _) => expr,
            (None, Some(n)) => self.resolve_class_fqn(subpackage, &n),
            (None, None) => {
                let mangled = self.mangle_data_class(&short);
                self.resolve_class_fqn(subpackage, &mangled)
            }
        };
        let key = decl.key;
        let entry = self.types.entry(key.clone()).or_default();
        entry.class_decl = true;
        entry.value_blob = true;
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.accept_members(&key, decl.members);
    }

    /// Shared tail of `accept_ptr_class`/`accept_value_class` (the two class
    /// kinds whose members are emitted): each member's per-fn flatten
    /// modifiers apply exactly as a free function's would; a constructor
    /// member's return is additionally never output-flattened (it's a
    /// factory); then the members join the class's registered set.
    fn accept_members(&mut self, key: &TypeKey, members: Vec<(FunctionDecl, MemberKind)>) {
        for (decl, kind) in members {
            let rust_ident = decl.rust_ident.clone();
            let kotlin_name = decl
                .kotlin_name_override
                .clone()
                .unwrap_or_else(|| snake_to_camel(&rust_ident.to_string()));
            self.apply_fn_flatten_modifiers(decl);
            if kind == MemberKind::Constructor {
                self.deconstructors
                    .add_skip_default_output(rust_ident.clone());
            }
            self.class_members
                .entry(key.clone())
                .or_default()
                .push(ClassMember {
                    rust_ident,
                    kotlin_name,
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
        self.apply_fn_flatten_modifiers(decl);
    }

    /// Replay a [`FunctionDecl`]'s per-fn overrides (per-param
    /// `.param_expand*` / `.return_expand*`) into the
    /// expansion/deconstruction bookkeeping. Shared by [`Self::accept_function`]
    /// (free package fns) and [`Self::accept_members`] (class members) — the
    /// overrides mean the same thing in both positions.
    ///
    /// Identity-only normalization: an override consisting of only the
    /// `_self` form means "the plain handle, nothing else" and lowers to the
    /// skip-default opt-out — no selector on the input side, the raw
    /// whole-handle return (borrowed-`&T`-capable) on the output side.
    fn apply_fn_flatten_modifiers(&mut self, decl: FunctionDecl) {
        let FunctionDecl {
            rust_ident,
            kotlin_name_override: _,
            input_overrides,
            output_override,
        } = decl;

        for (param, variants) in input_overrides {
            if matches!(variants.as_slice(), [LocalVariant::SelfIdentity]) {
                self.expansions
                    .add_skip_default_construct(rust_ident.clone(), param);
                continue;
            }
            self.expansions.begin_subset(rust_ident.clone(), param);
            for v in variants {
                match v {
                    LocalVariant::Ctor(f) => self.expansions.push_subset_variant(f),
                    LocalVariant::SelfIdentity => self.expansions.push_subset_self(),
                }
            }
        }
        if let Some(fields) = output_override {
            if matches!(fields.as_slice(), [LocalField::SelfField]) {
                self.deconstructors
                    .add_skip_default_output(rust_ident.clone());
                return;
            }
            self.deconstructors.begin_inline_output(rust_ident.clone());
            for f in fields {
                match f {
                    LocalField::Named(func, name) => {
                        self.accessor_record_fns.insert(func.clone());
                        self.deconstructors.push_inline_field(func, name)
                    }
                    LocalField::SelfField => self.deconstructors.push_inline_field_self(),
                }
            }
        }
    }
}

// ── Accepting wrapper decls ──────────────────────────────────────────────

impl JniGen {
    /// Teach the generator how a **custom scalar type** crosses the boundary
    /// — e.g. a newtype like `Millis(u64)` that should travel as a plain
    /// `Long` rather than as a class. You supply the wire type and the
    /// convert-in / convert-out expressions (see [`ScalarTypeWrapperDecl`]).
    /// Applies wherever that type appears; not tied to any package.
    pub fn scalar_type_wrapper(mut self, decl: ScalarTypeWrapperDecl) -> Self {
        let key = TypeKey::from_type(&decl.pattern);
        if let Some(input) = decl.input {
            let wire_src = decl.wire.clone();
            self.input_wrappers[0].insert(
                key.clone(),
                Arc::new(move |_args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                    let wire: syn::Type =
                        syn::parse_str(&wire_src).expect("stored wire type re-parses");
                    Some((wire, None, input(&wrapper_value_ident())))
                }),
            );
        }
        if let Some(output) = decl.output {
            let wire_src = decl.wire.clone();
            self.output_wrappers[0].insert(
                key.clone(),
                Arc::new(move |_args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                    let wire: syn::Type =
                        syn::parse_str(&wire_src).expect("stored wire type re-parses");
                    Some((wire, None, output(&wrapper_value_ident())))
                }),
            );
        }
        let entry = self.types.entry(key.clone()).or_default();
        entry.kotlin_name = Some(decl.kotlin_type.clone());
        self.kotlin_type_fqns
            .push((key.as_str().to_string(), decl.kotlin_type));
        self
    }

    /// Override how a **generic wrapper type** is unwrapped for a specific
    /// inner type — e.g. peel `Result<_, MyError>` your own way rather than
    /// through the built-in `Result` handling. You give a pattern with
    /// wildcards and the convert bodies (see [`GenericTypeWrapperDecl`]).
    /// Not tied to any package.
    pub fn generic_type_wrapper(mut self, decl: GenericTypeWrapperDecl) -> Self {
        let key = TypeKey::from_type(&decl.pattern);
        if let Some((rank, f)) = decl.input {
            self.input_wrappers[rank].insert(key.clone(), f);
        }
        if let Some((rank, f)) = decl.output {
            self.output_wrappers[rank].insert(key, f);
        }
        self
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
        let f = self.input_wrappers[rank].get(&key)?;
        let (ty, exc_ty, body) = f(args, registry)?;
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
                        .and_then(|c| c.kotlin_name.clone())
                        .map(kt::KtType::cls)
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
        let f = self.output_wrappers[rank].get(&key)?;
        let (ty, exc_ty, body) = f(args, registry)?;
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
                        .and_then(|c| c.kotlin_name.clone())
                        .map(kt::KtType::cls)
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
