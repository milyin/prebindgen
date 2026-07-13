//! Builder API for [`JniGen`].
//!
//! [`JniGen::new`] starts from defaults; global settings are applied with
//! the `set_*` methods (`config.rs`) and declarations are *accepted* as
//! pre-built objects (`decl.rs`) via [`JniGen::package`],
//! [`JniGen::param_expand`], [`JniGen::return_expand`],
//! [`JniGen::scalar_type_wrapper`], and [`JniGen::generic_type_wrapper`] —
//! there is no fluent typestate cursor. Carved from the former monolithic
//! JNI module; shares the `jni` namespace via `use super::*`.

use super::*;

impl JniGen {
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
    /// Start a binding generator with default settings: `source_module =
    /// crate`, empty base package, no `JNINative` init block, identity
    /// name-mangling, handle locks enabled. Adjust settings with the `set_*`
    /// methods, add declarations with [`package`](Self::package),
    /// [`scalar_type_wrapper`](Self::scalar_type_wrapper), etc., then run the
    /// result through `Registry::write_rust` / `write_kotlin`. Settings and
    /// declarations may be interleaved in any order — the builder stores
    /// only raw inputs, and every setting-derived name is computed at the
    /// point of use.
    pub fn new() -> Self {
        let mut jni = Self {
            source_module: syn::parse_str("crate").unwrap(),
            package: String::new(),
            fun_name_mangle: None,
            ptr_class_name_mangle: None,
            data_class_name_mangle: None,
            enum_name_mangle: None,
            harness_name_mangle: None,
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
            param_expand_decls: Vec::new(),
            return_expand_decls: Vec::new(),
            class_members: HashMap::new(),
            ignored_fns: std::collections::HashSet::new(),
            ignored_class_types: std::collections::HashSet::new(),
            ignored_const_idents: std::collections::HashSet::new(),
            accessor_record_fns: std::collections::HashSet::new(),
        };
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
    /// Apply the harness mangle closure to `name`. Defaults to
    /// `|n| format!("JNI{n}")` when unset, so `mangle_harness("Native")`
    /// yields `"JNINative"`.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.harness_name_mangle {
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
            constant_functions,
            constant_exprs,
        } = decl;
        self.packages.entry(name.clone()).or_default();
        for class in classes {
            self.accept_class(&name, class);
        }
        for func in functions {
            self.accept_function(&name, func);
        }
        for c in constants {
            let mut entry = MethodEntry::new(c.rust_ident);
            entry.kotlin_name_override = c.kotlin_name_override;
            self.packages
                .entry(name.clone())
                .or_default()
                .constants
                .push(entry);
        }
        for f in constant_functions {
            let mut entry = MethodEntry::new(f.rust_ident);
            entry.kotlin_name_override = f.kotlin_name_override;
            self.packages
                .entry(name.clone())
                .or_default()
                .constant_functions
                .push(entry);
        }
        for e in constant_exprs {
            self.packages
                .entry(name.clone())
                .or_default()
                .constant_exprs
                .push(e);
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

    /// Acknowledge a `#[prebindgen]` const this binding deliberately does
    /// NOT expose — the const-level dual of [`Self::ignore_fun`]. E.g.
    /// `.ignore_const(constant!(INTERNAL_MAGIC))`.
    pub fn ignore_const(mut self, decl: ConstDecl) -> Self {
        self.ignored_const_idents.insert(decl.rust_ident);
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
        if let NameSpec::Class {
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

    fn accept_ptr_class(&mut self, subpackage: &str, decl: PtrClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        self.register_class_name(
            &key,
            NameSpec::Class {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Ptr,
            },
        );
        self.types
            .get_mut(&key)
            .expect("register_class_name created the entry")
            .opaque = Some(OpaqueConfig::default());
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
            NameSpec::Class {
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
    }

    /// A data/value class's explicit `kotlin_type` expression is a verbatim
    /// Kotlin type and wins over everything; otherwise the name derives from
    /// settings at read time like any other declared class.
    fn data_value_name_spec(
        subpackage: &str,
        short: String,
        name_override: Option<String>,
        kotlin_type: Option<String>,
    ) -> NameSpec {
        match kotlin_type {
            Some(expr) => NameSpec::Verbatim(expr),
            None => NameSpec::Class {
                subpackage: subpackage.to_string(),
                short,
                name_override,
                kind: NameKind::DataOrValue,
            },
        }
    }

    fn accept_data_class(&mut self, subpackage: &str, decl: DataClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        let spec =
            Self::data_value_name_spec(subpackage, short, decl.name_override, decl.kotlin_type);
        self.register_class_name(&key, spec);
    }

    fn accept_value_class(&mut self, subpackage: &str, decl: ValueClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        let spec =
            Self::data_value_name_spec(subpackage, short, decl.name_override, decl.kotlin_type);
        self.register_class_name(&key, spec);
        self.types
            .get_mut(&key)
            .expect("register_class_name created the entry")
            .value_blob = true;
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
                        let name = name.unwrap_or_else(|| snake_to_camel(&func.to_string()));
                        self.deconstructors.push_inline_field(func, name)
                    }
                    LocalField::SelfField => self.deconstructors.push_inline_field_self(),
                }
            }
        }
    }
}

// ── Accepting boundary decls ─────────────────────────────────────────────

impl JniGen {
    /// Declare a type's **default input boundary** (a [`ParamExpandDecl`],
    /// built with [`param_expand!`](crate::param_expand)): how a parameter of
    /// that type may be supplied, as an OR-list of build variants. Applies to
    /// every function with a parameter of the type, in any package; a single
    /// function overrides via [`FunctionDecl::param_expand`] /
    /// [`FunctionDecl::param_expand_self`].
    pub fn param_expand(mut self, decl: ParamExpandDecl) -> Self {
        assert!(
            !decl.variants.is_empty(),
            "param_expand!({}) declares no variants — add .variant(fun!(...)) and/or \
             .variant_self()",
            decl.key.as_str()
        );
        self.param_expand_decls.push(decl);
        self
    }

    /// Declare a type's **default output boundary** (a [`ReturnExpandDecl`],
    /// built with [`return_expand!`](crate::return_expand)): the AND-set of
    /// fields a returned / callback-delivered value of that type decomposes
    /// into. Applies to every function returning the type, in any package; a
    /// single function overrides via [`FunctionDecl::return_expand`] /
    /// [`FunctionDecl::return_expand_self`].
    pub fn return_expand(mut self, decl: ReturnExpandDecl) -> Self {
        assert!(
            !decl.fields.is_empty(),
            "return_expand!({}) declares no fields — add .field(fun!(...)) and/or \
             .field_self()",
            decl.key.as_str()
        );
        self.return_expand_decls.push(decl);
        self
    }

    /// The Kotlin name of `func` as a declared member (`.fun`/`.constructor`)
    /// of the class keyed by `key`, if it is one — the name-inheritance
    /// source for [`ReturnExpandDecl::field`].
    fn member_kotlin_name(&self, key: &TypeKey, func: &syn::Ident) -> Option<String> {
        self.class_members
            .get(key)?
            .iter()
            .find(|m| &m.rust_ident == func)
            .map(|m| m.kotlin_name.clone())
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
    /// [`ParamExpandDecl`]s. Building on demand keeps declarations
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
                "param_expand!({k}).variant_self(): `{k}` has no class declaration, so there is \
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
                "return_expand!({k}).field_self(): `{k}` has no class declaration, so there is \
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
        dec
    }

    /// Type keys of boundary decls (`param_expand!` / `return_expand!`) whose
    /// type has no class declaration — the **rust-side-only** types. Unioned
    /// into [`Prebindgen::ignored_types`] so the registry treats them as
    /// acknowledged (no "skipping undeclared" warning, no direct converter
    /// requirement, no Kotlin emission).
    pub(crate) fn rust_side_only_types(&self) -> impl Iterator<Item = TypeKey> + '_ {
        self.param_expand_decls
            .iter()
            .map(|d| &d.key)
            .chain(self.return_expand_decls.iter().map(|d| &d.key))
            .filter(|k| !self.is_class_declared(k))
            .cloned()
    }

    /// Function idents referenced only inside boundary decls — `return_expand!`
    /// field accessors and `param_expand!` variant ctors. They are called
    /// Rust-side by the generated fold/unfold code and need no extern of
    /// their own; when not otherwise declared they are unioned into
    /// [`Prebindgen::ignored_functions`] so the registry's "skipping
    /// undeclared fn" warning stays quiet.
    pub(crate) fn boundary_referenced_fns(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        let ctors = self.param_expand_decls.iter().flat_map(|d| {
            d.variants.iter().filter_map(|v| match v {
                LocalVariant::Ctor(f) => Some(f.clone()),
                LocalVariant::SelfIdentity => None,
            })
        });
        let accessors = self.return_expand_decls.iter().flat_map(|d| {
            d.fields.iter().filter_map(|f| match f {
                LocalField::Named(func, _) => Some(func.clone()),
                LocalField::SelfField => None,
            })
        });
        ctors.chain(accessors)
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
                Arc::new(
                    move |_args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                        let wire: syn::Type =
                            syn::parse_str(&wire_src).expect("stored wire type re-parses");
                        Some((wire, None, input(&wrapper_value_ident())))
                    },
                ),
            );
        }
        if let Some(output) = decl.output {
            let wire_src = decl.wire.clone();
            self.output_wrappers[0].insert(
                key.clone(),
                Arc::new(
                    move |_args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                        let wire: syn::Type =
                            syn::parse_str(&wire_src).expect("stored wire type re-parses");
                        Some((wire, None, output(&wrapper_value_ident())))
                    },
                ),
            );
        }
        let entry = self.types.entry(key.clone()).or_default();
        entry.name_spec = Some(NameSpec::Verbatim(decl.kotlin_type));
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
