//! Builder API for [`JniGen`].
//!
//! [`JniGen::new`] starts from defaults; global settings are applied with
//! the `set_*` methods (`config.rs`) and declarations are *accepted* as
//! pre-built objects (`decl.rs`) via [`JniGen::package`], [`JniGen::expand`],
//! and [`JniGen::convert`] — there is no fluent typestate cursor. Carved from the former monolithic
//! JNI module; shares the `jni` namespace via `use super::*`.

use super::*;
use crate::api::core::registry::Conversions;

impl DeclaredKind {
    /// The declaring macro's name, for the conflict message.
    pub(crate) fn macro_name(&self) -> &'static str {
        match self {
            DeclaredKind::Ptr(_) => "ptr_class",
            DeclaredKind::Enum(_) => "enum_class",
            DeclaredKind::Sealed(_) => "sealed_class",
            DeclaredKind::Data => "data_class",
        }
    }

    /// Fold a **reopened** declaration of the same kind into this one, or
    /// reject a *different* kind. Both halves of "a type gets exactly one
    /// class declarator" live here, once, for every kind:
    ///
    /// * a different variant is rejected — two declarators would emit two
    ///   Kotlin declarations for the same FQN and leave the type table
    ///   ambiguous about the kind. The check is one discriminant comparison,
    ///   kind-agnostic and symmetric in declaration order, so a new kind is
    ///   covered by it without touching this function;
    /// * the same variant merges that kind's own payload, each rule written
    ///   next to the payload it merges.
    ///
    /// `type_name` is the short Rust name of the type being declared, for the
    /// message.
    fn merge(&mut self, incoming: DeclaredKind, type_name: &str) {
        assert!(
            std::mem::discriminant(&*self) == std::mem::discriminant(&incoming),
            "`{}` is already declared with `{}!(...)`; it cannot also be declared with \
             `{}!(...)` — a type gets exactly one class declarator",
            type_name,
            self.macro_name(),
            incoming.macro_name(),
        );
        match (self, incoming) {
            // `.gc_managed()` is sticky-OR: once any declaration asks for
            // Cleaner-backed release, the handle keeps it.
            (DeclaredKind::Ptr(have), DeclaredKind::Ptr(add)) => {
                have.gc_managed |= add.gc_managed;
            }
            // Per-variant last-wins: a second `sealed_class!(E)` adding one
            // `.variant(...)` must not drop the renames the first one set.
            (DeclaredKind::Sealed(have), DeclaredKind::Sealed(add)) => {
                have.variant_names.extend(add.variant_names);
            }
            // Kinds with no payload of their own: nothing to merge.
            (DeclaredKind::Enum(_), _) | (DeclaredKind::Data, _) => {}
            // The discriminants were just checked equal, so no mixed pair
            // reaches this arm — landing here means a kind carrying options
            // was added above without a merge rule.
            (existing, _) => unreachable!(
                "`{}!(...)` has no merge rule for a reopened declaration",
                existing.macro_name()
            ),
        }
    }
}

impl JniGen {
    /// The module path a generated call to `#[prebindgen]` fn `ident` must be
    /// qualified with: the fn's **origin crate** as recorded from its
    /// stream's `SourceLocation` stamp (multi-source bindings — helper
    /// crates layered on the flat crate), else the registry's default
    /// module (first-seen stream origin), else `crate`.
    pub(crate) fn fn_module(
        &self,
        registry: &impl Conversions<KotlinMeta>,
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
        self.types.get(&key).is_some_and(|c| c.is_enum_class())
    }
}

impl JniGen {
    /// Start a binding generator with default settings: empty base
    /// package, no `JNINative` init block, identity
    /// name-mangling, handle locks enabled. Adjust settings with the `set_*`
    /// methods, add declarations with [`package`](Self::package),
    /// [`expand`](Self::expand), [`convert`](Self::convert), etc., then run the
    /// result through `JniGen::resolve` → `Generation::write_rust` /
    /// `write_kotlin`. Settings and
    /// declarations may be interleaved in any order — the builder stores
    /// only raw inputs, and every setting-derived name is computed at the
    /// point of use.
    pub fn new() -> Self {
        Self {
            package: String::new(),
            fun_name_mangle: None,
            ptr_class_name_mangle: None,
            data_class_name_mangle: None,
            enum_name_mangle: None,
            method_name_mangle: None,
            harness_name_mangle: None,
            interface_name_mangle: None,
            types: HashMap::new(),
            packages: BTreeMap::new(),
            emit_handle_locks: true,
            jni_native_init: None,
            convert_decls: Vec::new(),
            param_expand_decls: Vec::new(),
            return_expand_decls: Vec::new(),
            fn_param_expands: Vec::new(),
            fn_return_expands: Vec::new(),
            fn_split_params: Vec::new(),
            class_members: HashMap::new(),
            ignored_fns: std::collections::HashSet::new(),
            ignored_name_predicates: Vec::new(),
            ignored_class_types: std::collections::HashSet::new(),
            ignored_const_idents: std::collections::HashSet::new(),
            local_fns: Vec::new(),
            iface_specs: Default::default(),
            fn_plans: Default::default(),
        }
    }

    /// Apply the package-level function-name mangle closure to `name`.
    pub(crate) fn mangle_fun(&self, package: &str, name: &str) -> String {
        match &self.fun_name_mangle {
            Some(f) => f(package, name),
            None => mangle_kotlin_ident(name),
        }
    }
    /// Apply the method-name mangle closure to `name`, providing the package
    /// and final class name that contain the method.
    pub(crate) fn mangle_method(&self, package: &str, class: &str, name: &str) -> String {
        match &self.method_name_mangle {
            Some(f) => f(package, class, name),
            None => mangle_kotlin_ident(name),
        }
    }
    /// Apply the ptr-class mangle closure to `name`, returning the closure
    /// result or the sanitized `name` (issue #89) when unset.
    pub(crate) fn mangle_ptr_class(&self, package: &str, name: &str) -> String {
        match &self.ptr_class_name_mangle {
            Some(f) => f(package, name),
            None => mangle_kotlin_ident(name),
        }
    }
    /// Apply the data-class mangle closure to `name`, returning the closure
    /// result or the sanitized `name` (issue #89) when unset.
    pub(crate) fn mangle_data_class(&self, package: &str, name: &str) -> String {
        match &self.data_class_name_mangle {
            Some(f) => f(package, name),
            None => mangle_kotlin_ident(name),
        }
    }
    /// Apply the enum mangle closure to `name`, returning the closure result
    /// or the sanitized `name` (issue #89) when unset.
    pub(crate) fn mangle_enum(&self, package: &str, name: &str) -> String {
        match &self.enum_name_mangle {
            Some(f) => f(package, name),
            None => mangle_kotlin_ident(name),
        }
    }
    /// Apply the harness mangle closure to `name`, returning the closure
    /// result or the sanitized `name` (issue #89) when unset.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.harness_name_mangle {
            Some(f) => f(name),
            None => mangle_kotlin_ident(name),
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

    /// Mangle a method emitted on the centralized JNI extern harness.
    pub(crate) fn mangle_jni_method(&self, name: &str) -> String {
        self.mangle_method(&self.package, &self.jni_native_class_name(), name)
    }

    /// Resolve a relative subpackage against the configured base package.
    pub(crate) fn package_name(&self, subpackage: &str) -> String {
        match (&self.package, subpackage) {
            (p, sub) if !sub.is_empty() && !p.is_empty() => format!("{p}.{sub}"),
            (_, sub) if !sub.is_empty() => sub.to_string(),
            (p, _) => p.clone(),
        }
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
        let base = self.package_name(subpackage);
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
                    let mut entry = FunctionEntry::new(c.rust_ident);
                    entry.kotlin_name_override = c.kotlin_name_override;
                    pkg.constants.push(entry);
                }
                super::decl::ConstSource::Fun(ref fn_ident) => {
                    let mut entry = FunctionEntry::new(fn_ident.clone());
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
            ClassDecl::Sealed(d) => self.accept_sealed_class(subpackage, d),
            ClassDecl::Data(d) => self.accept_data_class(subpackage, d),
        }
    }

    /// Register one class declaration: its [`DeclaredKind`] (kind + that
    /// kind's own options) and its raw [`NameSpec`]. The single entry point
    /// into [`Self::types`] — every acceptor goes through it, so the
    /// one-declarator-per-type rule is enforced for all kinds by
    /// [`DeclaredKind::merge`] and cannot be forgotten by a new one.
    /// No FQN is derived here — names materialize at read time via
    /// [`JniGen::fqn_of`], against whatever the settings are then.
    ///
    /// Returns the stored config so the caller can fold in its cross-kind
    /// options (`jobject_input`, interfaces).
    fn register_class(
        &mut self,
        key: &TypeKey,
        kind: DeclaredKind,
        spec: NameSpec,
    ) -> &mut TypeConfig {
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
        let short = rust_short_name(key);
        match self.types.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let cfg = e.into_mut();
                cfg.kind.merge(kind, &short);
                cfg.name_spec = Some(spec);
                cfg
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(TypeConfig::new(kind, spec)),
        }
    }

    /// Merge a decl's interface options into the type's [`TypeConfig`]
    /// (reopened decls merge; the `.interface()` switch and name override are
    /// sticky-OR / last-wins, a repeated `.implements` interface is
    /// idempotent).
    fn store_iface_opts(&mut self, key: &TypeKey, iface: IfaceOpts) {
        let cfg = self
            .types
            .get_mut(key)
            .expect("register_class created the entry");
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
        self.register_class(
            &key,
            DeclaredKind::Ptr(OpaqueConfig {
                gc_managed: decl.gc_managed,
            }),
            NameSpec {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Ptr,
            },
        );
        self.store_iface_opts(&key, decl.iface);
        self.accept_members(&key, decl.members);
    }

    fn accept_enum_class(&mut self, subpackage: &str, decl: EnumClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        self.register_class(
            &key,
            DeclaredKind::Enum(EnumConfig::default()),
            NameSpec {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Enum,
            },
        );
        self.store_iface_opts(&key, decl.iface);
    }

    /// A `sealed_class!` declaration. The Kotlin name routes through the
    /// **enum** mangle hook: a sum is a declared enum, and its interface is
    /// the Kotlin name of that enum — one hook per Rust item kind, not one
    /// per emitted shape.
    fn accept_sealed_class(&mut self, subpackage: &str, decl: SealedClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        // Reopened decls merge — `DeclaredKind::merge` owns that rule for
        // every kind, so this acceptor only builds its own payload.
        let mut sum = SumConfig::default();
        for v in decl.variants {
            if let Some(name) = v.name_override {
                sum.variant_names.insert(v.rust_ident, name);
            }
        }
        self.register_class(
            &key,
            DeclaredKind::Sealed(sum),
            NameSpec {
                subpackage: subpackage.to_string(),
                short,
                name_override: decl.name_override,
                kind: NameKind::Enum,
            },
        );
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
        self.register_class(&key, DeclaredKind::Data, spec)
            .jobject_input |= decl.jobject_input;
        self.store_iface_opts(&key, decl.iface);
        self.accept_members(&key, decl.members);
    }

    /// Shared tail of the member-bearing class kinds (`ptr` / `data` —
    /// every kind whose instance can re-enter Rust): each member's
    /// per-fn expand overrides apply exactly as a free function's would; a
    /// constructor member's return is additionally never output-flattened
    /// (it's a factory); then the members join the class's registered set.
    fn accept_members(&mut self, key: &TypeKey, members: Vec<(FunctionDecl, MemberKind)>) {
        for (decl, kind) in members {
            let rust_ident = decl.rust_ident.clone();
            let kotlin_name_override = decl.kotlin_name_override.clone();
            self.accept_fn_expands(decl);
            // A constructor member's return is a factory, never
            // output-flattened — derived from `class_members` in
            // `build_deconstructors` (`skip_output`), not stored separately.
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
        let mut entry = FunctionEntry::new(decl.rust_ident.clone());
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
            split_on_params,
            local,
        } = decl;
        // A path-built decl (`fun!(crate::f)`) declares a BINDING-LOCAL fn:
        // record its stated signature for the synthesis pre-pass
        // ([`Self::local_functions`]). The signature is mandatory — a path
        // carries nothing to read.
        if let Some((path, sig)) = local {
            let Some(sig) = sig else {
                panic!(
                    "fun!({p}): a binding-local fn states its signature — chain \
                     .sig(sig!((params) -> Ret))",
                    p = quote::quote!(#path)
                );
            };
            self.local_fns.push((rust_ident.clone(), path, sig));
        }
        for (param, pdecl) in param_expands {
            self.fn_param_expands
                .push((rust_ident.clone(), param, pdecl));
        }
        if let Some(rdecl) = return_expand {
            self.fn_return_expands.push((rust_ident.clone(), rdecl));
        }
        for param in split_on_params {
            self.fn_split_params.push((rust_ident.clone(), param));
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

    /// The Kotlin name of `func` as a declared member (`.method`/`.constructor`)
    /// of the class keyed by `key`, if it is one — the name-inheritance
    /// source for [`ExpandReturnDecl::field`].
    fn class_method_kotlin_name(&self, key: &TypeKey, func: &syn::Ident) -> Option<String> {
        self.class_members
            .get(key)?
            .iter()
            .find(|m| &m.rust_ident == func)
            .map(|m| self.effective_method_name(key, m))
    }

    /// The effective Kotlin name of a class method/factory, derived at point
    /// of use: a per-method `.name()` override verbatim, else the method
    /// hook over the full camelCase Rust identifier with its final package
    /// and class context. Consumers can therefore remove flat namespace
    /// prefixes without the generator guessing their source convention.
    pub(crate) fn effective_method_name(&self, key: &TypeKey, m: &ClassMember) -> String {
        if let Some(name) = &m.kotlin_name_override {
            return name.clone();
        }
        let spec = self
            .types
            .get(key)
            .and_then(|cfg| cfg.name_spec.as_ref())
            .unwrap_or_else(|| panic!("class member `{}` has no class name", m.rust_ident));
        let fqn = self.fqn_of(spec);
        let (package, class) = fqn.rsplit_once('.').unwrap_or(("", fqn.as_str()));
        self.mangle_method(package, class, &snake_to_camel(&m.rust_ident.to_string()))
    }

    /// The effective Kotlin name of a package-level function. Explicit
    /// `.name()` wins; otherwise the package-aware function hook receives
    /// the full camelCase Rust identifier.
    pub(crate) fn effective_function_name(
        &self,
        subpackage: &str,
        entry: &FunctionEntry,
    ) -> String {
        entry.kotlin_name_override.clone().unwrap_or_else(|| {
            self.mangle_fun(
                &self.package_name(subpackage),
                &snake_to_camel(&entry.rust_ident.to_string()),
            )
        })
    }

    /// Whether `key` was declared as a class in some package (any
    /// [`DeclaredKind`]). Presence in [`Self::types`] *is* the answer —
    /// [`Self::register_class`] is the table's only writer. A boundary decl on
    /// a type without a class declaration makes it **rust-side-only**: the
    /// value is always built from ingredients / decomposed into fields at the
    /// boundary and never materializes in Kotlin — so the `_self` arms are
    /// structurally impossible for it.
    fn is_class_declared(&self, key: &TypeKey) -> bool {
        self.types.contains_key(key)
    }

    /// Lower the raw [`ExpandParamDecl`]s into the core's immutable
    /// [`Expansions`] record set at the point of use — a pure declaration →
    /// record mapping. Building on demand keeps declarations
    /// order-independent — a `param_expand` may precede or follow the
    /// `package` that declares its constructors (which is also why the
    /// rust-side-only `_self` check lives here and not at accept time).
    /// Duplicate targets pass through unmerged; core `apply` diagnoses them.
    pub(crate) fn build_expansions(&self) -> crate::api::core::expand::Expansions {
        use crate::api::core::expand::{ExpandDecl, ExpandSel, Expansions, Variant};
        let lower = |v: &LocalVariant| match v {
            LocalVariant::Ctor(f) => Variant::Ctor(f.clone()),
            LocalVariant::SelfIdentity => Variant::Identity,
        };
        let mut exp = Expansions::default();
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
            exp.constructors
                .push(crate::api::core::expand::ConstructorDecl {
                    target: decl.key.to_type(),
                    variants: decl.variants.iter().map(lower).collect(),
                    default: true,
                });
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
            exp.expands.push(ExpandDecl {
                func: func.clone(),
                param: syn::Ident::new(param, Span::call_site()),
                declared_target: Some(decl.key.to_type()),
                sel: ExpandSel::Subset(decl.variants.iter().map(lower).collect()),
            });
        }
        exp
    }

    /// Lower one raw [`LocalField`] list into core [`DeconRecord`]s with the
    /// UNIFORM field-name precedence resolved against the complete
    /// declaration set: explicit `.name()` first, then the class member's
    /// Kotlin name (a getter that is both a method and a field is named
    /// once, on the member), else the camel-cased Rust name.
    fn lower_fields(
        &self,
        registry: &impl Conversions<KotlinMeta>,
        key: &TypeKey,
        fields: &[LocalField],
    ) -> Vec<crate::api::core::unfold::DeconRecord> {
        use crate::api::core::unfold::DeconRecord;
        fields
            .iter()
            .map(|f| match f {
                LocalField::Fields(decl) => DeconRecord::Fields {
                    func: decl.func.clone(),
                    consuming: decl.consuming,
                    fields: self.lower_value_form(registry, key, decl),
                },
                LocalField::Named(func, name_override) => {
                    let name = name_override
                        .clone()
                        .or_else(|| self.class_method_kotlin_name(key, func))
                        .unwrap_or_else(|| snake_to_camel(&func.to_string()));
                    DeconRecord::Acc {
                        func: func.clone(),
                        name,
                    }
                }
                LocalField::SelfField => DeconRecord::Identity,
                LocalField::Local {
                    path,
                    sig,
                    name_override,
                } => {
                    let name = self.local_field_name(key, path, name_override);
                    self.check_local_field_ty(key, &name, sig);
                    DeconRecord::LocalAcc {
                        path: path.clone(),
                        name,
                    }
                }
            })
            .collect()
    }

    /// Expand a `.fields(fields!(f))` declaration into one
    /// [`FieldRecord`](crate::api::core::unfold::FieldRecord) per field of the
    /// struct `f` returns — the value form.
    ///
    /// The walk is the adapter's job because only it knows which structs are
    /// declared `data_class!`es: a **non-optional** nested one is inlined (its
    /// own fields become records with `__`-joined names), matching what
    /// `synth_value_struct_leaves` does for a by-value data class; everything
    /// else is one record and core decides whether that record's type splices
    /// its own `expand_return!`.
    ///
    /// Per-field `.field(...)` overrides and `.name(...)` renames key on the
    /// **Rust field ident**, and both are checked against the struct: naming a
    /// field the value form doesn't have is a hard error, which is the point —
    /// a field renamed upstream must not silently lose its adjustment.
    fn lower_value_form(
        &self,
        registry: &impl Conversions<KotlinMeta>,
        key: &TypeKey,
        decl: &FieldsDecl,
    ) -> Vec<crate::api::core::unfold::FieldRecord> {
        let func = &decl.func;
        let item_fn = registry
            .flat()
            .function(&func)
            .map(|func| &func.origin.syntax)
            .unwrap_or_else(|| {
                panic!(
                    "expand_return!({}).fields(fields!({func})): no `#[prebindgen]` function \
                 `{func}` — a value form is an accessor `fn {func}(v: &{}) -> {}Struct`",
                    key.as_str(),
                    key.as_str(),
                    key.as_str(),
                )
            });
        let ret: syn::Type = match &item_fn.sig.output {
            syn::ReturnType::Type(_, t) => crate::api::core::unfold::peel_ref(t),
            syn::ReturnType::Default => panic!(
                "expand_return!({}).fields(fields!({func})): `{func}` returns nothing — a \
                 value form returns the struct holding this type's fields",
                key.as_str()
            ),
        };
        let TypeKind::DataStruct { st, .. } = self.type_kind(registry, &ret) else {
            panic!(
                "expand_return!({}).fields(fields!({func})): `{func}` returns `{}`, which is \
                 not a struct — a value form returns a struct whose fields become the leaves",
                key.as_str(),
                ret.to_token_stream(),
            )
        };
        let st = st.clone();

        let mut out = Vec::new();
        self.walk_value_form(registry, key, decl, &st, &[], "", 0, &mut out);

        // Every adjustment must have found its field. An unknown name is the
        // drift this whole declarator exists to catch, so it is an error rather
        // than a no-op.
        let named: std::collections::HashSet<String> = out
            .iter()
            .map(|r: &crate::api::core::unfold::FieldRecord| {
                r.members
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .collect();
        for (field, _) in decl.overrides.iter() {
            assert!(
                named.contains(field),
                "fields!({func}).field(\"{field}\", ...): `{}` has no field `{field}` \
                 (fields: {})",
                st.ident,
                named.iter().cloned().collect::<Vec<_>>().join(", "),
            );
        }
        for (field, _) in decl.names.iter() {
            assert!(
                named.contains(field),
                "fields!({func}).name(\"{field}\", ...): `{}` has no field `{field}` \
                 (fields: {})",
                st.ident,
                named.iter().cloned().collect::<Vec<_>>().join(", "),
            );
        }
        out
    }

    /// One level of [`Self::lower_value_form`]'s struct walk. `members` /
    /// `name_prefix` accumulate through inlined nested data classes; an
    /// override or rename keys on the dotted member path, so a nested field is
    /// addressed as `"outer.inner"`.
    #[allow(clippy::too_many_arguments)]
    fn walk_value_form(
        &self,
        registry: &impl Conversions<KotlinMeta>,
        key: &TypeKey,
        decl: &FieldsDecl,
        st: &syn::ItemStruct,
        members: &[syn::Ident],
        name_prefix: &str,
        depth: usize,
        out: &mut Vec<crate::api::core::unfold::FieldRecord>,
    ) {
        use crate::api::core::unfold::{FieldDecon, FieldRecord};
        let syn::Fields::Named(named) = &st.fields else {
            panic!(
                "expand_return!({}).fields(fields!({})): `{}` has no named fields — a value \
                 form is a plain struct whose fields become the leaves",
                key.as_str(),
                decl.func,
                st.ident,
            )
        };
        // A value form holding itself would expand forever; the cycle rule for
        // everything reachable BELOW a field is core's `visited` check.
        assert!(
            depth <= 16,
            "expand_return!({}).fields(fields!({})): `{}` nests data classes more than 16 \
             deep — is a value form holding itself?",
            key.as_str(),
            decl.func,
            st.ident,
        );
        for field in &named.named {
            let Some(fname) = field.ident.as_ref() else {
                continue;
            };
            let mut member_path = members.to_vec();
            member_path.push(fname.clone());
            let dotted = member_path
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(".");
            let camel = mangle_kotlin_ident(&kt_snake_to_camel(&fname.to_string()));
            let name = decl
                .names
                .iter()
                .find(|(f, _)| *f == dotted)
                .map(|(_, n)| n.clone())
                .unwrap_or(camel);
            let name = if name_prefix.is_empty() {
                name
            } else {
                format!("{name_prefix}__{name}")
            };

            // An explicit override replaces the field type's default
            // decomposition wholesale — including any nesting it would have had.
            if let Some((_, ovr)) = decl.overrides.iter().find(|(f, _)| *f == dotted) {
                // The override states the field's type, so it is cross-checked
                // against the field the same way a per-fn `.expand_param` /
                // `.expand_return` decl is checked against its parameter or
                // return. Without this an override outlives an upstream
                // field-type change — the very drift `.fields()` exists to
                // catch — and two same-shaped handle types silently swap.
                // Core applies override records to the whole field after
                // peeling only an outer `Option`: a `Vec<T>` remains `Vec<T>`.
                // Mirror that exact normalization here; peeling `Vec` would
                // accept `expand_return!(T)` and only fail later when core
                // applies its records to `Vec<T>`.
                let peeled = option_inner_type(&field.ty)
                    .map(|t| crate::api::core::unfold::peel_ref(&t))
                    .unwrap_or_else(|| crate::api::core::unfold::peel_ref(&field.ty));
                let actual = TypeKey::from_type(&peeled);
                assert!(
                    actual == ovr.key,
                    "fields!({}).field(\"{dotted}\", expand_return!({})): `{}.{dotted}` is \
                     `{}`, not `{}` — a per-field override names the field's own type",
                    decl.func,
                    ovr.key.as_str(),
                    st.ident,
                    actual.as_str(),
                    ovr.key.as_str(),
                );
                out.push(FieldRecord {
                    members: member_path,
                    name,
                    ty: field.ty.clone(),
                    decon: FieldDecon::Records(self.lower_fields(registry, &ovr.key, &ovr.fields)),
                });
                continue;
            }

            // A nested `data_class!` inlines when it is reached directly; behind
            // `Option` / `Vec` it stays one leaf, whose own converter builds the
            // object (the rule `synth_value_struct_leaves` already follows).
            // A `sealed_class!` field has no whole-value converter at all, so it
            // must decompose into its selector and groups wherever it appears.
            let bare = option_inner_type(&field.ty).unwrap_or_else(|| field.ty.clone());
            let probe = vec_inner_type(&bare).unwrap_or_else(|| bare.clone());
            match self.type_kind(registry, &probe) {
                TypeKind::DataStruct { st, cfg: Some(_) }
                    if option_inner_type(&field.ty).is_none()
                        && vec_inner_type(&field.ty).is_none() =>
                {
                    let child = st.clone();
                    self.walk_value_form(
                        registry,
                        key,
                        decl,
                        &child,
                        &member_path,
                        &name,
                        depth + 1,
                        out,
                    );
                    continue;
                }
                TypeKind::Sum => {
                    // A sum's leaves are a selector plus one group per
                    // alternative, laid out side by side at a FIXED position.
                    // A `Vec` of them has variable arity; an `Option` of one
                    // needs a present flag the unfold leaf list has no notion of
                    // (the `fromParts` bridge's `PlanFieldKind::Sum` does — a
                    // data-class field can be `Option<sum>`).
                    assert!(
                        vec_inner_type(&bare).is_none(),
                        "expand_return!({}).fields(fields!({})): field `{}.{}` is a \
                         `Vec<{}>` — a sequence of tag-gated groups has variable arity and \
                         cannot be laid out in a fixed leaf list",
                        key.as_str(),
                        decl.func,
                        st.ident,
                        dotted,
                        probe.to_token_stream(),
                    );
                    assert!(
                        option_inner_type(&field.ty).is_none(),
                        "expand_return!({}).fields(fields!({})): field `{}.{}` is an \
                         `Option<{}>` — an optional sum would need a present flag beside its \
                         tag, which an output leaf list cannot carry. Give the field a \
                         payload-less alternative instead of wrapping the sum in `Option`, \
                         or override it with .field(\"{}\", ...)",
                        key.as_str(),
                        decl.func,
                        st.ident,
                        dotted,
                        probe.to_token_stream(),
                        dotted,
                    );
                    let ident = bare_path_ident(&probe).expect("a sum type is a path type");
                    let item_enum = registry
                        .flat()
                        .enum_item(&ident)
                        .expect("TypeKind::Sum implies an indexed enum");
                    let sum_cfg = self.types[&TypeKey::from_type(&probe)]
                        .sum()
                        .expect("TypeKind::Sum implies a sealed-class config");
                    out.push(FieldRecord {
                        members: member_path,
                        name,
                        ty: field.ty.clone(),
                        decon: FieldDecon::Leaves(crate::api::lang::jnigen::jni::synth_sum_leaves(
                            self, sum_cfg, item_enum,
                        )),
                    });
                    continue;
                }
                _ => {}
            }

            out.push(FieldRecord {
                members: member_path,
                name,
                ty: field.ty.clone(),
                decon: FieldDecon::Default,
            });
        }
    }

    /// Lower the raw [`ExpandReturnDecl`]s into the core's immutable
    /// [`Deconstructors`] record set — the output-side peer of
    /// [`Self::build_expansions`], a pure declaration → record mapping.
    /// Duplicate targets pass through unmerged; core `apply` diagnoses
    /// them. `skip_output` is derived from the class members: a
    /// `.constructor()` member's return is a factory, never
    /// output-flattened.
    pub(crate) fn build_deconstructors(
        &self,
        registry: &impl Conversions<KotlinMeta>,
    ) -> crate::api::core::unfold::Deconstructors {
        use crate::api::core::unfold::{
            DeconSel, DeconTarget, DeconstructorDecl, Deconstructors, Delivery, OutputDecl,
        };
        let mut dec = Deconstructors {
            skip_output: self
                .class_members
                .values()
                .flatten()
                .filter(|m| m.kind == MemberKind::Constructor)
                .map(|m| m.rust_ident.clone())
                .collect(),
            ..Deconstructors::default()
        };
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
            dec.deconstructors.push(DeconstructorDecl {
                target: decl.key.to_type(),
                records: self.lower_fields(registry, &decl.key, &decl.fields),
                default: Some((DeconTarget::Output, Delivery::Callback)),
            });
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
            dec.outputs.push(OutputDecl {
                func: func.clone(),
                sel: DeconSel::Inline(self.lower_fields(registry, &decl.key, &decl.fields)),
                target: DeconTarget::Output,
                delivery: Delivery::Callback,
                declared_source: Some(decl.key.to_type()),
            });
        }
        dec
    }

    /// The resolved Kotlin name of a binding-local field — the UNIFORM
    /// field-name precedence over the path's LAST segment: explicit
    /// `.name()`; else the class member's Kotlin name if the same fn is a
    /// `.method()` of the type; else the camel-cased fn ident.
    fn local_field_name(
        &self,
        key: &TypeKey,
        path: &syn::Path,
        name_override: &Option<String>,
    ) -> String {
        let ident = &path.segments.last().expect("non-empty path").ident;
        name_override
            .clone()
            .or_else(|| self.class_method_kotlin_name(key, ident))
            .unwrap_or_else(|| snake_to_camel(&ident.to_string()))
    }

    /// Synthesize the registry items for every binding-local fn declared on
    /// this generator (see [`Prebindgen::local_functions`]): path-built
    /// `fun!(crate::f).sig(…)` decls contribute their full stated signature;
    /// `field!("name").with(ty, path)` output fields contribute
    /// `fn <ident>(v: &Target) -> Ty`. The item body is `unimplemented!()`
    /// — never emitted, only the signature is read; the origin is the path's
    /// module prefix, so generated calls qualify exactly as declared. One fn
    /// ident may back several declarations only with an IDENTICAL
    /// synthesized signature (panic otherwise — the emitted call could not
    /// distinguish them).
    pub(crate) fn collect_local_functions(&self) -> Vec<(syn::ItemFn, String)> {
        use quote::ToTokens;
        let mut out: Vec<(syn::ItemFn, String)> = Vec::new();
        let mut seen: HashMap<syn::Ident, String> = HashMap::new();
        let mut push = |item_fn: syn::ItemFn, origin: String, out: &mut Vec<_>| {
            let ident = item_fn.sig.ident.clone();
            let sig_str = format!("{origin}::{}", item_fn.sig.to_token_stream());
            match seen.get(&ident) {
                Some(prev) if *prev == sig_str => {} // same fn, same shape
                Some(_) => panic!(
                    "binding-local fn `{ident}` is declared with two different signatures — \
                     the emitted call is `<origin>::{ident}`, so one fn = one signature"
                ),
                None => {
                    seen.insert(ident, sig_str);
                    out.push((item_fn, origin));
                }
            }
        };
        // Path-built fun! decls: the stated signature, renamed to the ident.
        for (ident, path, sig) in &self.local_fns {
            let origin = local_path_prefix(path);
            let mut sig = sig.clone();
            sig.ident = ident.clone();
            let item_fn: syn::ItemFn = syn::parse_quote! {
                #sig {
                    unimplemented!()
                }
            };
            push(item_fn, origin, &mut out);
        }
        // field! output fields: `fn <ident>(v: &Target) -> Ty`.
        let type_level = self
            .return_expand_decls
            .iter()
            .map(|d| (d.key.clone(), &d.fields));
        let per_fn = self
            .fn_return_expands
            .iter()
            .map(|(_, d)| (d.key.clone(), &d.fields));
        for (_key, fields) in type_level.chain(per_fn) {
            for f in fields {
                let LocalField::Local { path, sig, .. } = f else {
                    continue;
                };
                let origin = local_path_prefix(path);
                let ident = path.segments.last().expect("non-empty path").ident.clone();
                let mut sig = sig.clone();
                sig.ident = ident;
                let item_fn: syn::ItemFn = syn::parse_quote! {
                    #sig {
                        unimplemented!()
                    }
                };
                push(item_fn, origin, &mut out);
            }
        }
        out
    }

    /// Guard for binding-local fields returning an OPTIONAL BORROW: the
    /// `Option<&T>` conditional-delivery leaf rides the opaque-handle
    /// projection (nullable typed handle / boxed `Long?` on the wire), so `T`
    /// must be a declared `ptr_class`. Owned returns — `Option<String>`,
    /// scalars, handles — carry their nullability in their own converters and
    /// pass through unchecked.
    fn check_local_field_ty(&self, decl_key: &TypeKey, name: &str, sig: &syn::Signature) {
        let syn::ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        let syn::Type::Path(p) = &**ty else { return };
        if p.path.segments.last().is_none_or(|s| s.ident != "Option") {
            return;
        }
        let Some(syn::PathArguments::AngleBracketed(args)) =
            p.path.segments.last().map(|s| &s.arguments)
        else {
            return;
        };
        let Some(syn::GenericArgument::Type(syn::Type::Reference(r))) = args.args.first() else {
            return;
        };
        let inner_key = TypeKey::from_type(&r.elem);
        assert!(
            self.types.get(&inner_key).is_some_and(|c| c.is_opaque()),
            "expand_return!({}).field(… .name(\"{name}\")): an `Option<&T>` binding-local field \
             delivers a nullable typed HANDLE, so `T` must be a declared ptr_class — `{}` is \
             not; return an owned `Option<{}>` instead",
            decl_key.as_str(),
            inner_key.as_str(),
            inner_key.as_str(),
        );
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
        // Includes a binding-local field's synthesized fn (called by the
        // generated code, never externed, so its synthesized registry entry
        // must not trip the warning) and a value form's accessor.
        let accessors = self.field_referenced_fns().into_iter();
        // Synthesized binding-local fns from every entry form (path-built
        // fun! at fun/method/constructor/convert sites): their registry
        // entries exist only for signature reads — helper-only unless also
        // declared (the declared set is subtracted by the caller).
        let locals = self.local_fns.iter().map(|(ident, _, _)| ident.clone());
        ctors.chain(accessors).chain(locals)
    }

    /// Every function referenced as a named field in any `expand_return!`
    /// decl (type-level or per-fn) — the accessor set. Backs
    /// [`Prebindgen::accessor_functions`]: `core/unfold.rs`'s deconstructor
    /// gate requires every named record's function to be in this set
    /// (`RecordNotAccessor` otherwise), and `core/expand.rs` excludes them
    /// from parameter composition. Derived from *usage* — a function need not
    /// also be a `.method()` class member to be referenced this way.
    pub(crate) fn field_accessor_fns(&self) -> std::collections::HashSet<syn::Ident> {
        self.field_referenced_fns().into_iter().collect()
    }

    /// Every function ident referenced as a field by any `expand_return!` decl
    /// (type-level or per-fn), recursing into a value form's per-field
    /// overrides. The one walk behind both [`Self::field_accessor_fns`] and the
    /// helper-only set in [`Self::boundary_referenced_fns`] — they ask the same
    /// question of the same declarations, so a new field kind is taught to both
    /// at once.
    fn field_referenced_fns(&self) -> Vec<syn::Ident> {
        fn walk(fields: &[LocalField], out: &mut Vec<syn::Ident>) {
            for f in fields {
                match f {
                    LocalField::Named(func, _) => out.push(func.clone()),
                    // A binding-local field's synthesized fn IS an accessor —
                    // excluded from parameter composition, and acknowledged so
                    // the registry's "skipping undeclared" warning stays quiet.
                    LocalField::Local { path, .. } => out.push(
                        path.segments
                            .last()
                            .expect("validated non-empty at decl time")
                            .ident
                            .clone(),
                    ),
                    LocalField::SelfField => {}
                    // The value form's own accessor, plus whatever its
                    // per-field overrides reference.
                    LocalField::Fields(d) => {
                        out.push(d.func.clone());
                        for (_, ovr) in &d.overrides {
                            walk(&ovr.fields, out);
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        for fields in self
            .return_expand_decls
            .iter()
            .map(|d| &d.fields)
            .chain(self.fn_return_expands.iter().map(|(_, d)| &d.fields))
        {
            walk(fields, &mut out);
        }
        out
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
    pub fn convert(mut self, mut decl: ConvertDecl) -> Self {
        assert!(
            decl.input.is_some() || decl.output.is_some(),
            "convert!({}) declares no conversions — add .input(fun!(...)) and/or \
             .output(fun!(...))",
            decl.key.as_str()
        );
        // Binding-local fn sources (`fun!(crate::f).sig(…)`) join the same
        // synthesis list as fun/method/constructor sites — after the
        // pre-pass they lower exactly like `#[prebindgen]` fn sources.
        self.local_fns.append(&mut decl.locals);
        self.convert_decls.push(decl);
        self
    }

    /// Derive the rank-0 **input** converter body for a `convert!`-declared
    /// type: `(continue_ty, exc, body)` where `continue_ty` is the conversion
    /// fn's parameter type (by value) — the composed-converter machinery
    /// chains it through that type's own converter, so the wire and the
    /// Kotlin surface derive from it. It is what [`Self::lookup_input`]
    /// answers with; signatures are read from the registry at
    /// lookup time (order-independent, and multi-source qualification via
    /// [`Self::fn_module`]).
    pub(crate) fn convert_input_body(
        &self,
        key: &TypeKey,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
        let decl = self.convert_decls.iter().find(|d| &d.key == key)?;
        let target = key.to_type();
        let result = match decl.input.as_ref()? {
            ConvertSpec::PrebindgenFn(f) => {
                let item_fn = registry
                    .flat()
                    .function(&f)
                    .map(|func| &func.origin.syntax)
                    .unwrap_or_else(|| {
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
            } // Binding-local callable: emitted verbatim (multi-segment paths
              // pass the qualification visitor untouched). With a declared
              // error type the fn returns `Result<T, E>` — emitted as-is, `E`
              // riding the standard exc slot.
        };
        let (repr, exc, body) = result?;
        Some(self.apply_input_domain(decl, repr, exc, body))
    }

    /// Output-direction peer of [`Self::convert_input_body`]: the conversion
    /// fn takes `&T` (or `T`) and returns the continue type.
    pub(crate) fn convert_output_body(
        &self,
        key: &TypeKey,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
        let decl = self.convert_decls.iter().find(|d| &d.key == key)?;
        let target = key.to_type();
        let result = match decl.output.as_ref()? {
            ConvertSpec::PrebindgenFn(g) => {
                let item_fn = registry
                    .flat()
                    .function(&g)
                    .map(|func| &func.origin.syntax)
                    .unwrap_or_else(|| {
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
                let (repr, exc) = match crate::api::core::types_util::result_ok_type(&ret) {
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
                    TypeKey::from_type(&repr) != *key,
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
                Some((repr, exc, body))
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
        };
        let (repr, exc, body) = result?;
        Some(self.apply_output_domain(decl, repr, exc, body))
    }

    /// Idents of every `#[prebindgen]`-fn conversion source — scanned as
    /// helper functions ([`Prebindgen::helper_functions`]) so their extern
    /// emission is suppressed. Trait/local-fn sources have no registry item.
    fn apply_input_domain(
        &self,
        decl: &ConvertDecl,
        repr: syn::Type,
        exc: Option<syn::Type>,
        body: syn::Expr,
    ) -> (syn::Type, Option<syn::Type>, syn::Expr) {
        let Some(domain) = &decl.domain else {
            return (repr, exc, body);
        };
        assert_eq!(
            TypeKey::from_type(domain.ty()),
            TypeKey::from_type(&repr),
            "convert!({}): domain type {} does not match input representation {}",
            decl.key.as_str(),
            TypeKey::from_type(domain.ty()),
            TypeKey::from_type(&repr),
        );
        let valid = domain.contains_expr(quote!(v));
        let key = decl.key.as_str();
        let converted = if exc.is_some() {
            quote!((#body).map_err(|__e| {
                <__JniErr as ::core::convert::From<String>>::from(__e.to_string())
            }))
        } else {
            quote!(::core::result::Result::Ok(#body))
        };
        let body = syn::parse_quote!({
            if #valid {
                #converted
            } else {
                ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(
                        format!("{} representation is outside its declared domain", #key)
                    )
                )
            }
        });
        (repr, Some(syn::parse_quote!(__JniErr)), body)
    }

    fn apply_output_domain(
        &self,
        decl: &ConvertDecl,
        repr: syn::Type,
        exc: Option<syn::Type>,
        body: syn::Expr,
    ) -> (syn::Type, Option<syn::Type>, syn::Expr) {
        let Some(domain) = &decl.domain else {
            return (repr, exc, body);
        };
        assert_eq!(
            TypeKey::from_type(domain.ty()),
            TypeKey::from_type(&repr),
            "convert!({}): domain type {} does not match output representation {}",
            decl.key.as_str(),
            TypeKey::from_type(domain.ty()),
            TypeKey::from_type(&repr),
        );
        let valid = domain.contains_expr(quote!(__repr));
        let key = decl.key.as_str();
        let converted = if exc.is_some() {
            quote!((#body).map_err(|__e| {
                <__JniErr as ::core::convert::From<String>>::from(__e.to_string())
            }))
        } else {
            quote!(::core::result::Result::Ok(#body))
        };
        let body = syn::parse_quote!({
            match #converted {
                ::core::result::Result::Ok(__repr) if #valid => {
                    ::core::result::Result::Ok(__repr)
                }
                ::core::result::Result::Ok(_) => {
                    ::core::result::Result::Err(
                        <__JniErr as ::core::convert::From<String>>::from(
                            format!("{} representation is outside its declared domain", #key)
                        )
                    )
                }
                ::core::result::Result::Err(__e) => {
                    ::core::result::Result::Err(__e)
                }
            }
        });
        (repr, Some(syn::parse_quote!(__JniErr)), body)
    }

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

    fn conversion_domain_niches(
        &self,
        key: &TypeKey,
        registry: &impl Conversions<KotlinMeta>,
        direction: Direction,
        wire: &syn::Type,
    ) -> (Niches, Vec<String>) {
        let Some(domain) = self
            .convert_decls
            .iter()
            .find(|d| &d.key == key)
            .and_then(|d| d.domain.as_ref())
        else {
            return (Niches::empty(), Vec::new());
        };
        if TypeKey::from_type(domain.ty()).as_str() != "u64"
            || crate::api::core::types_util::path_tail_ident(wire)
                .is_none_or(|ident| ident != "jlong")
        {
            return (Niches::empty(), Vec::new());
        }
        let demand = registry
            .crossing_keys(direction)
            .iter()
            .map(|candidate| {
                let mut ty = candidate.to_type();
                let mut depth = 0;
                while crate::api::core::types_util::is_option_type(&ty) {
                    let Some(inner) = option_inner_type(&ty) else {
                        return 0;
                    };
                    ty = inner;
                    depth += 1;
                }
                if TypeKey::from_type(&ty) == *key {
                    depth
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);
        let mut slots = Vec::new();
        let mut kotlin = Vec::new();
        for value in domain.niche_values(demand) {
            let ScalarValue::U64(value) = value else {
                continue;
            };
            let raw = value as i64;
            let literal = if raw == i64::MIN {
                "Long.MIN_VALUE".to_string()
            } else {
                format!("{raw}L")
            };
            slots.push(NicheSlot {
                value: syn::parse_quote!(#raw),
                matches: syn::parse_quote!(*v == #raw),
            });
            kotlin.push(literal);
        }
        (Niches::from_slots(slots), kotlin)
    }

    fn attach_domain_sentinels(metadata: &mut KotlinMeta, sentinels: Vec<String>) {
        if let Some(projection) = metadata.projection.as_mut() {
            projection.niche_sentinels = sentinels;
        }
    }

    // ── Converter lookups (used by the Prebindgen impl) ───────────

    /// The input converter a `convert!` declaration supplies for `outer`.
    ///
    /// The body triple's middle slot carries the bound exception — `None` ⇒
    /// framework `__JniErr` with an `Ok`-wrap, `Some(<Rust type>)` ⇒
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
    pub(crate) fn lookup_input(
        &self,
        outer: &syn::Type,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // A `convert!`-declared conversion is the only thing that answers here.
        // There was a wildcard-pattern table beside it; nothing ever wrote to
        // the input half, so every lookup through it returned `None`.
        let key = TypeKey::from_type(outer);
        let (ty, exc_ty, body) = self.convert_input_body(&key, registry)?;
        // The closure's middle slot carries the `Result`'s raw Rust error
        // type (or `None` for the framework `__JniErr`); it feeds the
        // converter signature `Result<_, E>` directly — no registration.
        let exc = exc_ty.as_ref();
        // Terminal vs composed: `ty` is composed iff it's a *distinct*
        // rust type with its own input converter. The self-check guards
        // the void/identity case, and the registered-converter probe
        // distinguishes a rust continue-type (compose) from a wire
        // (terminal) without forcing `()` either way. A non-wire `ty` that
        // isn't yet resolved defers.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(outer);
        let inner = if is_self {
            None
        } else {
            registry.input_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces `outer`.
                let kotlin_name = self
                    .types
                    .get(&key)
                    .and_then(|c| c.name_spec.as_ref())
                    .map(|s| kt::KtType::cls(self.fqn_of(s)))
                    .or_else(|| kotlin_for_wire(&ty));
                let niches = Niches::empty();
                Some(ConverterImpl {
                    subs: vec![],
                    pre_stages: vec![],
                    function: self.build_input_fn(outer, &ty, &body, exc),
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
                    function: self.build_output_fn(&ty, outer, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let kotlin_name = inner.metadata.kotlin_name.clone();
                let value_rust_key = None;
                let (niches, sentinels) = self.conversion_domain_niches(
                    &key,
                    registry,
                    Direction::Input,
                    &inner.destination,
                );
                let mut metadata = KotlinMeta {
                    kotlin_name,
                    value_rust_key,
                    projection: inner.metadata.projection.clone(),
                };
                Self::attach_domain_sentinels(&mut metadata, sentinels);
                Some(ConverterImpl {
                    subs: vec![],
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata,
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
        outer: &syn::Type,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let key = TypeKey::from_type(outer);
        let (ty, exc_ty, body) = self.convert_output_body(&key, registry)?;
        self.build_output_converter(outer, None, ty, exc_ty, body, registry)
    }

    /// The `Result<T, E>` output peel: the value succeeds as `T`, and `E` routes
    /// to the error sink on `Err`.
    ///
    /// This was the sole entry in a four-rank wildcard-pattern table, reached
    /// through a general unification engine. The model already calls this shape
    /// [`TypeKind::Fallible`](crate::core::flat::TypeKind::Fallible), so the
    /// engine expressed one fact the frontend states outright.
    pub(crate) fn result_peel(
        &self,
        outer: &syn::Type,
        ok: &syn::Type,
        err: &syn::Type,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.build_output_converter(
            outer,
            Some(ok),
            ok.clone(),
            Some(err.clone()),
            syn::parse_quote!(v),
            registry,
        )
    }

    /// Assemble the output `ConverterImpl` from a body triple.
    ///
    /// `arg0` is the peeled inner type for a shape peel, `None` for a
    /// `convert!`-declared conversion — which is what the old `rank == 0`
    /// tested.
    fn build_output_converter(
        &self,
        outer: &syn::Type,
        arg0: Option<&syn::Type>,
        ty: syn::Type,
        exc_ty: Option<syn::Type>,
        body: syn::Expr,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let key = TypeKey::from_type(outer);
        // The middle slot carries the `Result`'s raw Rust error type (or `None`
        // for the framework `__JniErr`).
        let exc = exc_ty.as_ref();
        // Terminal vs composed — see [`Self::lookup_input`] for the rule.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(outer);
        let inner = if is_self {
            None
        } else {
            registry.output_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces it from `outer`.
                let (kotlin_name, value_rust_key) = if let Some(a0) = arg0 {
                    registry
                        .output_entry(a0)
                        .map(|e| (e.metadata.kotlin_name.clone(), Some(TypeKey::from_type(a0))))
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
                let niches = match arg0 {
                    None => Niches::empty(),
                    Some(_) => default_niches_for_wire(&ty),
                };
                Some(ConverterImpl {
                    subs: vec![],
                    pre_stages: vec![],
                    function: self.build_output_fn(outer, &ty, &body, exc),
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
                    function: self.build_output_fn(outer, &ty, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let kotlin_name = inner.metadata.kotlin_name.clone();
                let value_rust_key = arg0.map(TypeKey::from_type);
                let (niches, sentinels) = match arg0 {
                    None => self.conversion_domain_niches(
                        &key,
                        registry,
                        Direction::Output,
                        &inner.destination,
                    ),
                    Some(_) => (default_niches_for_wire(&inner.destination), Vec::new()),
                };
                let mut metadata = KotlinMeta {
                    kotlin_name,
                    value_rust_key,
                    projection: inner.metadata.projection.clone(),
                };
                Self::attach_domain_sentinels(&mut metadata, sentinels);
                Some(ConverterImpl {
                    subs: vec![],
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata,
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
/// The origin-module prefix of a binding-local fn's declared path
/// (`crate::sub::f` → `"crate::sub"`). Paths are validated ≥2 segments at
/// decl time (`fun!` path arm / `FieldDecl::with`), so the prefix is
/// always non-empty.
pub(crate) fn local_path_prefix(path: &syn::Path) -> String {
    path.segments
        .iter()
        .take(path.segments.len() - 1)
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

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
