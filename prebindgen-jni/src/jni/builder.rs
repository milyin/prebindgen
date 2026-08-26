//! Builder API for [`JniGenBuilder`].
//!
//! [`JniGenBuilder::new`] starts from defaults; global settings are applied with
//! the `set_*` methods (`config.rs`) and declarations are *accepted* as
//! pre-built objects (`decl.rs`) via [`JniGenBuilder::package`], [`JniGenBuilder::expand`],
//! and [`JniGenBuilder::convert`] — there is no fluent typestate cursor. Carved from the former monolithic
//! JNI module; shares the `jni` namespace via `use super::*`.

// `flat` as a module, not `flat::TypeKind` directly: the bare `TypeKind` in this
// file is jnigen's OWN classifier (`classify.rs`, reached through `use super::*`
// above), and an explicit import beats a glob — importing the model's would
// silently retarget the `TypeKind::Sum` / `TypeKind::DataStruct` matches below.
// One qualifier keeps both names short and says which of the two it is.
use kotlin_codegen::KtType;
use prebindgen_registry::{
    flat::{self, TypeRef},
    Conversions,
};

use super::*;

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

impl Declarations {
    /// The module path a generated call to `#[prebindgen]` fn `ident` must be
    /// qualified with: the fn's **origin crate** as recorded from its
    /// stream's `SourceLocation` stamp (multi-source bindings — helper
    /// crates layered on the flat crate), else the registry's default
    /// module (first-seen stream origin), else `crate`.
    pub(crate) fn fn_module(&self, registry: &impl Conversions, ident: &syn::Ident) -> syn::Path {
        registry
            .origin_module(ident)
            .or_else(|| registry.default_module())
            .unwrap_or_else(|| syn::parse_quote!(crate))
    }

    /// The module for source references with no per-item origin (declared
    /// types with no `#[prebindgen]` item, glob imports): the registry's
    /// default module (first source), `crate` for an origin-less registry.
    pub(crate) fn default_module(&self, registry: &Registry) -> syn::Path {
        registry
            .default_module()
            .unwrap_or_else(|| syn::parse_quote!(crate))
    }

    /// Whether this exact type identity was registered as an `EnumClassDecl`.
    /// See [`Self::is_kotlin_enum_reading`] for the question that peels model
    /// layers before probing the identity.
    pub(crate) fn is_kotlin_enum_key(&self, key: &TypeKey) -> bool {
        self.types.get(key).is_some_and(|c| c.is_enum_class())
    }

    /// Whether this value's core is a type registered via an `EnumClassDecl`,
    /// asked of the **reading**: [`enum_probe`] peels the borrow/optional
    /// layers off the model, and the name comes off the classification.
    ///
    /// A declaration names a type (`enum_class!(Priority)` keys `Priority`),
    /// and the model erases the wrappers no destination language can see — so
    /// `Box<Option<&Priority>>` reaches the same declaration `Priority` does,
    /// where taking the spelling apart finds `Box` and answers about it.
    pub(crate) fn is_kotlin_enum_reading(&self, reading: &TypeRef) -> bool {
        let flat::TypeKind::Named { id, .. } = enum_probe(reading).unwrapped().kind() else {
            return false;
        };
        id.ident()
            .and_then(|i| self.types.get(&TypeKey::from_ident(&i)))
            .is_some_and(|c| c.is_enum_class())
    }
}

impl Default for Declarations {
    fn default() -> Self {
        Self {
            tables: None,
            compiled: Default::default(),
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
            struct_plans: Default::default(),
            sum_plans: Default::default(),
            vec_build_plans: Default::default(),
            generation: None,
        }
    }
}

impl JniGenBuilder {
    /// Start a binding generator with default settings: empty base
    /// package, no `JNINative` init block, identity
    /// name-mangling, handle locks enabled. Adjust settings with the `set_*`
    /// methods, add declarations with [`package`](Self::package),
    /// [`expand`](Self::expand), [`convert`](Self::convert), etc., then run the
    /// result through [`build`](Self::build) → `JniGen::write_rust` /
    /// `write_kotlin`. Settings and
    /// declarations may be interleaved in any order — the builder stores
    /// only raw inputs, and every setting-derived name is computed at the
    /// point of use.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Declarations {
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

    /// The `@RequiresOptIn` marker guarding every generated entry point that
    /// takes or hands out a raw native pointer. Lives in the base package next
    /// to [`Self::jni_native_class_name`], and is referenced fully qualified
    /// from every generated file.
    ///
    /// With no base package that qualification is just the short name, which
    /// only resolves from the root package itself — Kotlin cannot import from
    /// the root package. `write_kotlin` rejects that configuration when any
    /// generated file would land in a subpackage.
    pub(crate) fn unsafe_marker_fqn(&self) -> String {
        match self.package.is_empty() {
            true => UNSAFE_MARKER.to_string(),
            false => format!("{}.{UNSAFE_MARKER}", self.package),
        }
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

// ── Accepting a `PackageDecl` ────────────────────────────────────────────

/// The describing surface: everything that *adds* to [`Declarations`].
///
/// Every method here takes `self` or `&mut self`; not one of them exists on
/// `Declarations`, which is the whole point of the two types.
impl JniGenBuilder {
    /// Register a package's worth of classes, functions and consts (a
    /// [`PackageDecl`], built with [`package!`](crate::package)). Call it once
    /// per package, or several times for the same package name — the
    /// declarations merge, so you can split a large package across calls.
    /// Every `#[prebindgen]` item captured in `dir` — pass
    /// `<source_crate>::PREBINDGEN_OUT_DIR`.
    ///
    /// The same feeder [`FlatBuilder::source`](prebindgen_registry::flat::FlatBuilder::source)
    /// has, because it is that feeder: the binding says where its source is,
    /// and the model is built from it at [`Self::build`].
    pub fn source<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.sources = std::mem::take(&mut self.sources).source(dir);
        self
    }

    /// The same, for a dependency this crate **renames** in `Cargo.toml`.
    ///
    /// The origin recorded at capture time is the dependency's real package
    /// name, which will not resolve from a crate that refers to it by another
    /// name. `crate_name` is the name *this* crate uses. Per directory,
    /// deliberately: a binding may layer several sources.
    pub fn source_named<P: AsRef<std::path::Path>>(
        mut self,
        dir: P,
        crate_name: impl Into<String>,
    ) -> Self {
        self.sources = std::mem::take(&mut self.sources).source_named(dir, crate_name);
        self
    }

    /// Add a captured item stream — a group selection, an otherwise-configured
    /// [`Source`](prebindgen::Source), or synthetic items in a test.
    ///
    /// Accumulates, so it mixes freely with [`Self::source`].
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (syn::Item, prebindgen::SourceLocation)>,
    {
        self.sources = std::mem::take(&mut self.sources).items(items);
        self
    }

    pub fn package(mut self, decl: PackageDecl) -> Self {
        let PackageDecl {
            name,
            classes,
            functions,
            constants,
        } = decl;
        self.decls.packages.entry(name.clone()).or_default();
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
            let pkg = self.decls.packages.entry(name.clone()).or_default();
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
    /// items, [`matching`] for a name-family
    /// predicate over ANY item kind.
    pub fn ignore(mut self, decl: impl Into<IgnoreDecl>) -> Self {
        match decl.into().0 {
            super::decl::IgnoreKind::Fun(ident) => {
                self.decls.ignored_fns.insert(ident);
            }
            super::decl::IgnoreKind::Type(key) => {
                self.decls.ignored_class_types.insert(key);
            }
            super::decl::IgnoreKind::Const(ident) => {
                self.decls.ignored_const_idents.insert(ident);
            }
            super::decl::IgnoreKind::Matching(pred) => {
                self.decls.ignored_name_predicates.push(pred);
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
    /// [`JniGenBuilder::fqn_of`], against whatever the settings are then.
    ///
    /// Returns the stored config so the caller can fold in its cross-kind
    /// options (`jobject_input`, interfaces).
    ///
    /// `rust_type` is the declaration's own spelling; `key` is the identity
    /// derived from it. They cannot disagree — every `*ClassDecl::new` builds
    /// both from the one `syn::Type` it was handed.
    fn register_class(
        &mut self,
        key: &TypeKey,
        rust_type: Origin<syn::Type>,
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
        match self.decls.types.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(e) => {
                // A reopened declarator keeps the first spelling: the two agree
                // on identity by construction, and the model indexes types
                // first-mention-wins for the same reason.
                let cfg = e.into_mut();
                cfg.kind.merge(kind, &short);
                cfg.name_spec = Some(spec);
                cfg
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(TypeConfig::new(kind, spec, rust_type))
            }
        }
    }

    /// Merge a decl's interface options into the type's [`TypeConfig`]
    /// (reopened decls merge; the `.interface()` switch and name override are
    /// sticky-OR / last-wins, a repeated `.implements` interface is
    /// idempotent).
    fn store_iface_opts(&mut self, key: &TypeKey, iface: IfaceOpts) {
        let cfg = self
            .decls
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
            decl.rust_type,
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
            decl.rust_type,
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
        let rust_type = decl.rust_type;
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
            rust_type,
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

    fn data_name_spec(subpackage: &str, short: String, name_override: Option<String>) -> NameSpec {
        NameSpec {
            subpackage: subpackage.to_string(),
            short,
            name_override,
            kind: NameKind::Data,
        }
    }

    fn accept_data_class(&mut self, subpackage: &str, decl: DataClassDecl) {
        let short = rust_short_name(&decl.key);
        let key = decl.key;
        let spec = Self::data_name_spec(subpackage, short, decl.name_override);
        self.register_class(&key, decl.rust_type, DeclaredKind::Data, spec)
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
            let rust_ident = decl.rust_ident().clone();
            let kotlin_name_override = decl.kotlin_name_override().clone();
            self.accept_fn_expands(decl);
            // A constructor member's return is a factory, never
            // output-flattened — derived from `class_members` in
            // `build_deconstructors` (`skip_output`), not stored separately.
            self.decls
                .class_members
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
        let mut entry = FunctionEntry::new(decl.rust_ident().clone());
        entry.kotlin_name_override = decl.kotlin_name_override().clone();
        self.decls
            .packages
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
        let (
            rust_ident,
            _kotlin_name_override,
            param_expands,
            return_expand,
            split_on_params,
            local,
        ) = decl.into_parts();
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
            self.decls.local_fns.push((rust_ident.clone(), path, sig));
        }
        for (param, pdecl) in param_expands {
            self.decls
                .fn_param_expands
                .push((rust_ident.clone(), param, pdecl));
        }
        if let Some(rdecl) = return_expand {
            self.decls
                .fn_return_expands
                .push((rust_ident.clone(), rdecl));
        }
        for param in split_on_params {
            self.decls.fn_split_params.push((rust_ident.clone(), param));
        }
    }
}

// ── Accepting boundary decls ─────────────────────────────────────────────

impl JniGenBuilder {
    /// Declare a type's **default boundary behavior** — either of the two
    /// [`ExpandDecl`] directions, the direction carried by the decl object
    /// (the boundary-decl peer of [`PackageDecl::class`]):
    ///
    /// * [`expand_param!`](prebindgen_registry::expand_param) — the input side: how a
    ///   parameter of the type may be supplied, as an OR-list of build
    ///   variants.
    /// * [`expand_return!`](prebindgen_registry::expand_return) — the output side: the
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
                    !decl.variants().is_empty(),
                    "expand_param!({}) declares no variants — add .variant(fun!(...)) and/or \
                     .variant_self()",
                    decl.key().as_str()
                );
                self.decls.param_expand_decls.push(decl);
            }
            ExpandDecl::Return(decl) => {
                assert!(
                    !decl.field_list().is_empty(),
                    "expand_return!({}) declares no fields — add .field(fun!(...)) and/or \
                     .field_self()",
                    decl.key().as_str()
                );
                self.decls.return_expand_decls.push(decl);
            }
        }
        self
    }
}

impl Declarations {
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
    pub(crate) fn build_expansions(&self) -> prebindgen_registry::expand::Expansions {
        use prebindgen_registry::expand::{ExpandDecl, ExpandSel, Expansions, Variant};
        let lower = |v: &LocalVariant| match v {
            LocalVariant::Ctor(f) => Variant::Ctor(f.clone()),
            LocalVariant::SelfIdentity => Variant::Identity,
        };
        let mut exp = Expansions::default();
        for decl in &self.param_expand_decls {
            assert!(
                self.is_class_declared(decl.key())
                    || !decl
                        .variants()
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "expand_param!({k}).variant_self(): `{k}` has no class declaration, so there is \
                 no Kotlin object to pass — drop .variant_self() (the type is rust-side-only) \
                 or declare the type in a package",
                k = decl.key().as_str()
            );
            // Identity-only normalization: `.variant_self()` alone declares
            // the plain-handle form — exactly the default when nothing is
            // declared, so registering it would only add a degenerate
            // 1-variant selector to every param of this type.
            if matches!(decl.variants(), [LocalVariant::SelfIdentity]) {
                continue;
            }
            exp.constructors
                .push(prebindgen_registry::expand::ConstructorDecl {
                    target: decl.rust_type().key(),
                    variants: decl.variants().iter().map(lower).collect(),
                    default: true,
                });
        }
        // Per-fn overrides: same decl shape, complete-set semantics; the
        // param-name/type cross-check and the identity-only lowering happen
        // in `core/expand.rs`'s `apply` (which sees the fn signatures).
        for (func, param, decl) in &self.fn_param_expands {
            assert!(
                self.is_class_declared(decl.key())
                    || !decl
                        .variants()
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "fun!({func}).expand_param(\"{param}\", expand_param!({k}).variant_self()): `{k}` \
                 has no class declaration, so there is no Kotlin object to pass — drop \
                 .variant_self() (the type is rust-side-only) or declare the type in a package",
                k = decl.key().as_str()
            );
            exp.expands.push(ExpandDecl {
                func: func.clone(),
                param: syn::Ident::new(param, Span::call_site()),
                declared_target: Some(decl.rust_type().key()),
                sel: ExpandSel::Subset(decl.variants().iter().map(lower).collect()),
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
        registry: &impl Conversions,
        key: &TypeKey,
        fields: &[LocalField],
    ) -> Vec<prebindgen_registry::unfold::DeconRecord> {
        use prebindgen_registry::unfold::DeconRecord;
        fields
            .iter()
            .map(|f| match f {
                LocalField::Fields(decl) => DeconRecord::Fields {
                    func: decl.func().clone(),
                    consuming: decl.is_consuming(),
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
    /// [`FieldRecord`](prebindgen_registry::unfold::FieldRecord) per field of the
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
    pub(crate) fn lower_value_form(
        &self,
        registry: &impl Conversions,
        key: &TypeKey,
        decl: &FieldsDecl,
    ) -> Vec<prebindgen_registry::unfold::FieldRecord> {
        let func = decl.func();
        let accessor = registry.flat().function(&func).unwrap_or_else(|| {
            panic!(
                "expand_return!({}).fields(fields!({func})): no `#[prebindgen]` function \
                 `{func}` — a value form is an accessor `fn {func}(v: &{}) -> {}Struct`",
                key.as_str(),
                key.as_str(),
                key.as_str(),
            )
        });
        // The accessor's return as the model read it, peeled of a leading `&`.
        // An elided return and a written `-> ()` are one thing here, because the
        // model already normalized them.
        let ret = accessor.ret.borrow_target().unwrap_or(&accessor.ret);
        assert!(
            !matches!(ret.unwrapped().kind(), flat::TypeKind::Unit),
            "expand_return!({}).fields(fields!({func})): `{func}` returns nothing — a \
             value form returns the struct holding this type's fields",
            key.as_str(),
        );
        let TypeKind::DataStruct { st, .. } = self.type_kind(registry, &ret.key()) else {
            panic!(
                "expand_return!({}).fields(fields!({func})): `{func}` returns `{}`, which is \
                 not a struct — a value form returns a struct whose fields become the leaves",
                key.as_str(),
                ret,
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
            .map(|r: &prebindgen_registry::unfold::FieldRecord| {
                r.members
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .collect();
        for (field, _) in decl.overrides().iter() {
            assert!(
                named.contains(field),
                "fields!({func}).field(\"{field}\", ...): `{}` has no field `{field}` \
                 (fields: {})",
                st.name,
                named.iter().cloned().collect::<Vec<_>>().join(", "),
            );
        }
        for (field, _) in decl.names().iter() {
            assert!(
                named.contains(field),
                "fields!({func}).name(\"{field}\", ...): `{}` has no field `{field}` \
                 (fields: {})",
                st.name,
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
        registry: &impl Conversions,
        key: &TypeKey,
        decl: &FieldsDecl,
        st: &flat::Struct,
        members: &[syn::Ident],
        name_prefix: &str,
        depth: usize,
        out: &mut Vec<prebindgen_registry::unfold::FieldRecord>,
    ) {
        use prebindgen_registry::unfold::{FieldDecon, FieldRecord};
        // A value form holding itself would expand forever; the cycle rule for
        // everything reachable BELOW a field is core's `visited` check.
        assert!(
            depth <= 16,
            "expand_return!({}).fields(fields!({})): `{}` nests data classes more than 16 \
             deep — is a value form holding itself?",
            key.as_str(),
            decl.func(),
            st.name,
        );
        // A tuple struct is an `Extern` rather than a `Struct`, so it never
        // reaches here — `lower_value_form`'s "not a struct" diagnosis catches
        // it at the return type, which is where the author wrote it.
        for field in &st.fields {
            let Some(fname) = field.name.as_ref() else {
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
                .names()
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
            if let Some((_, ovr)) = decl.overrides().iter().find(|(f, _)| *f == dotted) {
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
                let under_opt = field.ty.optional_inner().unwrap_or(&field.ty);
                let peeled = under_opt.borrow_target().unwrap_or(under_opt);
                let actual = peeled.key();
                assert!(
                    actual == *ovr.key(),
                    "fields!({}).field(\"{dotted}\", expand_return!({})): `{}.{dotted}` is \
                     `{}`, not `{}` — a per-field override names the field's own type",
                    decl.func(),
                    ovr.key().as_str(),
                    st.name,
                    actual.as_str(),
                    ovr.key().as_str(),
                );
                out.push(FieldRecord {
                    members: member_path,
                    name,
                    ty: field.ty.clone(),
                    decon: FieldDecon::Records(self.lower_fields(
                        registry,
                        ovr.key(),
                        ovr.field_list(),
                    )),
                });
                continue;
            }

            // A nested `data_class!` inlines when it is reached directly; behind
            // `Option` / `Vec` it stays one leaf, whose own converter builds the
            // object (the rule `synth_value_struct_leaves` already follows).
            // A `sealed_class!` field has no whole-value converter at all, so it
            // must decompose into its selector and groups wherever it appears.
            let bare = field.ty.optional_inner().unwrap_or(&field.ty);
            let probe = bare.sequence_elem().unwrap_or(bare);
            match self.type_kind(registry, &probe.key()) {
                TypeKind::DataStruct { st, cfg: Some(_) }
                    if field.ty.optional_inner().is_none()
                        && field.ty.sequence_elem().is_none() =>
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
                    // A `Vec` of them has variable arity, so there is no fixed
                    // layout to lay out — that one stays refused.
                    //
                    // `Option<sum>` does NOT: absence is the selector leaf's own
                    // nullability, the same mechanism a sum under a conditional
                    // value form already crosses by (#220). The refusal that
                    // stood here predated it.
                    assert!(
                        bare.sequence_elem().is_none(),
                        "expand_return!({}).fields(fields!({})): field `{}.{}` is a \
                         `Vec<{}>` — a sequence of tag-gated groups has variable arity and \
                         cannot be laid out in a fixed leaf list",
                        key.as_str(),
                        decl.func(),
                        st.name,
                        dotted,
                        probe,
                    );
                    // The name is the reading's, not a path taken apart to
                    // re-derive one.
                    let flat::TypeKind::Named { id, .. } = probe.unwrapped().kind() else {
                        panic!("a sum type is a named type")
                    };
                    let flat::Type::Variant(sum) = registry
                        .flat()
                        .declared_type(&id.name)
                        .expect("TypeKind::Sum implies an indexed enum")
                    else {
                        panic!("TypeKind::Sum implies a payload-carrying enum")
                    };
                    // The declaration has to exist for the composition below
                    // to name the alternatives' classes, and `TypeKind::Sum`
                    // means it does — asserted here rather than trusted,
                    // because the composition answers `None` for a missing one
                    // and an empty leaf list would silently drop the field.
                    assert!(
                        self.types[&probe.key()].sum().is_some(),
                        "TypeKind::Sum implies a sealed-class config",
                    );
                    out.push(FieldRecord {
                        members: member_path,
                        name,
                        ty: field.ty.clone(),
                        decon: FieldDecon::Leaves(crate::jni::synth_sum_leaves(
                            self,
                            registry,
                            &id.ident().expect("a sum type is one identifier"),
                            sum,
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
        registry: &impl Conversions,
    ) -> prebindgen_registry::unfold::Deconstructors {
        use prebindgen_registry::unfold::{
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
                self.is_class_declared(decl.key())
                    || !decl
                        .field_list()
                        .iter()
                        .any(|f| matches!(f, LocalField::SelfField)),
                "expand_return!({k}).field_self(): `{k}` has no class declaration, so there is \
                 no Kotlin object to deliver — drop .field_self() (the type is rust-side-only) \
                 or declare the type in a package",
                k = decl.key().as_str()
            );
            dec.deconstructors.push(DeconstructorDecl {
                target: decl.rust_type().key(),
                records: self.lower_fields(registry, decl.key(), decl.field_list()),
                default: Some((DeconTarget::Output, Delivery::Callback)),
            });
        }
        // Per-fn overrides: same decl shape and name inheritance; the
        // return-type cross-check and the identity-only lowering happen in
        // `core/unfold.rs`'s `apply` (which sees the fn signatures).
        for (func, decl) in &self.fn_return_expands {
            assert!(
                self.is_class_declared(decl.key())
                    || !decl
                        .field_list()
                        .iter()
                        .any(|f| matches!(f, LocalField::SelfField)),
                "fun!({func}).expand_return(expand_return!({k}).field_self()): `{k}` has no \
                 class declaration, so there is no Kotlin object to deliver — drop \
                 .field_self() (the type is rust-side-only) or declare the type in a package",
                k = decl.key().as_str()
            );
            dec.outputs.push(OutputDecl {
                func: func.clone(),
                sel: DeconSel::Inline(self.lower_fields(registry, decl.key(), decl.field_list())),
                target: DeconTarget::Output,
                delivery: Delivery::Callback,
                declared_source: Some(decl.rust_type().key()),
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
            .map(|d| (d.key().clone(), d.field_list()));
        let per_fn = self
            .fn_return_expands
            .iter()
            .map(|(_, d)| (d.key().clone(), d.field_list()));
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
    ///
    /// Yields each decl's own `syn::Type` beside its key: these are types a
    /// build script wrote, and the scan diagnoses their spelling before
    /// anything has classified them (#291).
    pub(crate) fn rust_side_only_types(
        &self,
    ) -> impl Iterator<Item = (TypeKey, Origin<syn::Type>)> + '_ {
        self.param_expand_decls
            .iter()
            .map(|d| (d.key(), d.rust_type()))
            .chain(
                self.return_expand_decls
                    .iter()
                    .map(|d| (d.key(), d.rust_type())),
            )
            .chain(
                self.fn_param_expands
                    .iter()
                    .map(|(_, _, d)| (d.key(), d.rust_type())),
            )
            .chain(
                self.fn_return_expands
                    .iter()
                    .map(|(_, d)| (d.key(), d.rust_type())),
            )
            .filter(|(k, _)| !self.is_class_declared(k))
            .map(|(k, t)| (k.clone(), t.clone()))
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
            .map(|d| d.variants())
            .chain(self.fn_param_expands.iter().map(|(_, _, d)| d.variants()))
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
                        out.push(d.func().clone());
                        for (_, ovr) in d.overrides() {
                            walk(ovr.field_list(), out);
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        for fields in self
            .return_expand_decls
            .iter()
            .map(|d| d.field_list())
            .chain(self.fn_return_expands.iter().map(|(_, d)| d.field_list()))
        {
            walk(fields, &mut out);
        }
        out
    }
}

// ── Accepting the convert decl ───────────────────────────────────────────

impl JniGenBuilder {
    /// Declare a type's **canonical single-value conversion** (a
    /// [`ConvertDecl`], built with [`convert!`](prebindgen_registry::convert)): a pair of
    /// `#[prebindgen]` functions carrying one value of the type across the
    /// boundary wherever a single value is needed (params, returns,
    /// `Option`/`Vec` elements, the `Result<T, E>` success position,
    /// `data_class` fields). Applies wherever the type appears; not tied to
    /// any package. See [`ConvertDecl`] for the relation to the
    /// [`expand`](Self::expand) boundary decls.
    pub fn convert(mut self, mut decl: ConvertDecl) -> Self {
        assert!(
            decl.input_spec().is_some() || decl.output_spec().is_some(),
            "convert!({}) declares no conversions — add .input(fun!(...)) and/or \
             .output(fun!(...))",
            decl.key().as_str()
        );
        // Binding-local fn sources (`fun!(crate::f).sig(…)`) join the same
        // synthesis list as fun/method/constructor sites — after the
        // pre-pass they lower exactly like `#[prebindgen]` fn sources.
        self.decls.local_fns.append(decl.locals_mut());
        self.decls.convert_decls.push(decl);
        self
    }
}

impl Declarations {
    pub(crate) fn convert_fns(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        self.convert_decls
            .iter()
            .flat_map(|d| d.input_spec().iter().chain(d.output_spec().iter()))
            .filter_map(|spec| match spec {
                ConvertSpec::PrebindgenFn(f) => Some(f.clone()),
                _ => None,
            })
    }
}

/// The single typed parameter of a conversion fn, peeled of a leading `&`;
/// asserts arity 1. Returns `(peeled_type, was_by_ref)`.
fn convert_single_param_any<'f>(
    f: &syn::Ident,
    item_fn: &'f prebindgen_registry::flat::Function,
) -> (&'f TypeRef, bool) {
    assert!(
        item_fn.params.len() == 1,
        "convert fn `{f}` must take exactly one parameter, it takes {}",
        item_fn.params.len()
    );
    let ty = &item_fn.params[0].ty;
    match ty.kind() {
        flat::TypeKind::Ref { inner, .. } => (inner, true),
        _ => (ty, false),
    }
}

impl Declarations {
    /// Build a `KotlinMeta` carrying just the value-context Kotlin name.
    /// Used by every built-in converter (primitives, structs, `Option<_>`,
    /// `Vec<_>`, `impl Fn(...)` lambdas). Errors are routed uniformly to the
    /// per-call `signal_error` sink by the extern emitter, so no
    /// per-converter exception metadata is carried.
    pub(crate) fn framework_meta(&self, kotlin_name: Option<KtType>) -> KotlinMeta {
        KotlinMeta {
            kotlin_name,
            value_reading: None,
            projection: None,
            niche_sentinels: Vec::new(),
        }
    }

    /// Maximum number of `Option` layers placed over `key` in one crossing
    /// direction. This is representation demand, not a spelling probe: the
    /// registry's readings already account for transparent wrappers.
    pub(crate) fn optional_niche_demand(
        &self,
        key: &TypeKey,
        registry: &impl Conversions,
        direction: Direction,
    ) -> usize {
        registry
            .crossing_keys(direction)
            .iter()
            .map(|candidate| {
                let Some(mut reading) = registry.reading(candidate) else {
                    return 0;
                };
                let mut depth = 0;
                while let Some(inner) = reading.optional_inner().cloned() {
                    reading = inner;
                    depth += 1;
                }
                if reading.key() == *key {
                    depth
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Allocate unused `jint` discriminants for an `enum_class` terminal.
    /// One slot is exposed per Optional layer that the registry will compose;
    /// the ordered slots let nested options remain a single primitive wire.
    pub(crate) fn enum_niches(
        &self,
        e: &prebindgen_registry::flat::Enum,
        registry: &impl Conversions,
        direction: Direction,
    ) -> (Niches, Vec<String>) {
        let key = TypeKey::from_ident(&e.name);
        let demand = self.optional_niche_demand(&key, registry, direction);
        if demand == 0 {
            return (Niches::empty(), Vec::new());
        }
        let used: std::collections::BTreeSet<i64> = e
            .discriminant_values()
            .unwrap_or_else(|name| {
                panic!(
                    "enum `{}` variant `{name}` has a non-literal discriminant; use a literal \
                     integer value (e.g. `= 1`) or an implicit discriminant",
                    e.name
                )
            })
            .into_iter()
            .map(|(_, value)| value)
            .collect();
        let mut raw = i32::MIN;
        let mut slots = Vec::with_capacity(demand);
        let mut kotlin = Vec::with_capacity(demand);
        while slots.len() < demand {
            if !used.contains(&i64::from(raw)) {
                slots.push(NicheSlot {
                    value: syn::parse_quote!(#raw),
                    matches: syn::parse_quote!(*v == #raw),
                });
                kotlin.push(if raw == i32::MIN {
                    "Int.MIN_VALUE".to_string()
                } else {
                    raw.to_string()
                });
                if slots.len() == demand {
                    break;
                }
            }
            raw = raw.checked_add(1).unwrap_or_else(|| {
                panic!(
                    "enum `{}` does not leave {demand} free jint discriminants for Optional \
                     composition",
                    e.name
                )
            });
        }
        (Niches::from_slots(slots), kotlin)
    }

    pub(crate) fn conversion_domain_niches(
        &self,
        key: &TypeKey,
        registry: &impl Conversions,
        direction: Direction,
        wire: &syn::Type,
    ) -> (Niches, Vec<String>) {
        let Some(domain) = self
            .convert_decls
            .iter()
            .find(|d| d.key() == key)
            .and_then(|d| d.domain().as_ref())
        else {
            return (Niches::empty(), Vec::new());
        };
        if TypeKey::from_type(domain.ty()).as_str() != "u64"
            || prebindgen_registry::types_util::path_tail_ident(wire)
                .is_none_or(|ident| ident != "jlong")
        {
            return (Niches::empty(), Vec::new());
        }
        // How many `Option` layers this crossing puts over `key` — the
        // model's count, so a wrapped spelling contributes the same demand a
        // bare one does (#273).
        let demand = self.optional_niche_demand(key, registry, direction);
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

    pub(crate) fn attach_domain_sentinels(metadata: &mut KotlinMeta, sentinels: Vec<String>) {
        if let Some(projection) = metadata.projection.as_mut() {
            projection.niche_sentinels = sentinels;
        }
    }

    /// The representation a `convert!` declaration depends on.
    ///
    /// This declaration-only bridge exists because `RegistryBuilder::cross`
    /// still accepts syntax. Function-backed conversions already provide a
    /// Flat `TypeRef`; converting its key back to syntax is the leak tracked
    /// by #558. Recipe compilation does not use this result: it retains the
    /// reading itself and final rendering alone spells it.
    pub(crate) fn convert_target(
        &self,
        key: &TypeKey,
        registry: &impl Conversions,
        dir: Direction,
    ) -> Option<syn::Type> {
        let decl = self.convert_decls.iter().find(|d| d.key() == key)?;
        let spec = match dir {
            Direction::Construct => decl.input_spec().as_ref()?,
            Direction::Deconstruct => decl.output_spec().as_ref()?,
        };
        match spec {
            ConvertSpec::Trait { repr, .. } => Some(repr.clone()),
            ConvertSpec::PrebindgenFn(f) => {
                let item_fn = registry.flat().function(f)?;
                let reading = match dir {
                    Direction::Construct => convert_single_param_any(f, item_fn).0,
                    Direction::Deconstruct => item_fn
                        .ret
                        .fallible_parts()
                        .map_or(&item_fn.ret, |(ok, _)| ok),
                };
                syn::parse_str(reading.key().as_str()).ok()
            }
        }
    }
}

/// The actual framework error type the `__JniErr` alias resolves to: the
/// E-agnostic `JniBindingError<()>` whose failures are always `JniError`
/// (binding-layer). A `Result<T, E>` return carries its own raw `E`, surfaced
/// as `UserError` at the extern's error site.
pub(crate) fn framework_error_type() -> syn::Type {
    syn::parse_quote!(::prebindgen_jni_runtime::JniBindingError<()>)
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
