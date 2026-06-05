//! Builder API for [`JniGen`].
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;

impl JniGen {
    /// Look up the registered Kotlin FQN for a canonical Rust type key
    /// (the inverse of the `(key, fqn)` rows pushed into
    /// [`Self::kotlin_type_fqns`] by the type-declaration builders). Single
    /// home for what used to be ~10 open-coded `kotlin_type_fqns.iter().find`
    /// scans across the emitters.
    pub(crate) fn kotlin_fqn(&self, rust_canon: &str) -> Option<&str> {
        self.kotlin_type_fqns
            .iter()
            .find(|(k, _)| k == rust_canon)
            .map(|(_, v)| v.as_str())
    }

    /// Convenience constructor with sensible defaults; the paths still need
    /// to be set explicitly via the field-mutation builder methods.
    ///
    /// Pre-registers the framework's [`crate::api::lang::jnigen::jni::JniBindingError`] as
    /// `exceptions[0]` so it's always available as `throw_JniBindingError`
    /// in the generated bindings and as the default `__JniErr` alias.
    /// Its Kotlin FQN is `(empty).JniBindingError` until [`Self::package`]
    /// is called, then auto-rebases via [`Self::recompute_derived`].
    pub fn new() -> Self {
        let framework_exc = build_exception_config(
            syn::parse_quote!(::prebindgen::lang::JniBindingError),
            "",
            &[],
        );
        let base = Self {
            source_module: syn::parse_str("crate").unwrap(),
            // exceptions[0] is the framework slot (JniBindingError);
            // user `.throwable()` calls append at 1+.
            exceptions: vec![framework_exc],
            package: String::new(),
            callback_subpackage: "callbacks".to_string(),
            java_class_prefix: String::new(),
            jni_class_path: "Java_JNINative".to_string(),
            kotlin_callback_package: "callbacks".to_string(),
            kotlin_fun_name_mangle: None,
            kotlin_ptr_class_name_mangle: None,
            kotlin_data_class_name_mangle: None,
            kotlin_enum_name_mangle: None,
            kotlin_package_name_mangle: None,
            kotlin_callback_name_mangle: None,
            kotlin_wrapper_name_mangle: None,
            kotlin_harness_name_mangle: None,
            kotlin_type_fqns: Vec::new(),
            types: HashMap::new(),
            packages: BTreeMap::new(),
            into_sources_map: HashMap::new(),
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
            last_opaque_key: None,
            last_meta_key: None,
            active_subpackage: None,
            last_entry_ref: None,
            emit_handle_locks: true,
        };
        // Built-in rank-2 `Result<_, _>` peel: every Result<T, E> succeeds
        // as T and throws E on Err. E must be declared throwable via
        // `.throwable()` (chained after a class declaration); the
        // resulting peel stage is composed via `lookup_output`'s
        // exact-canonical-form match in `find_exception`. Consumers may
        // override per-binding by registering a more specific rank-1
        // `Result<_, ConcreteErr>` (rank-1 phase fires before rank-2).
        base.output_wrapper(
            syn::parse_quote!(Result<_, _>),
            |ok: &syn::Type, err: &syn::Type, _: &Registry<KotlinMeta>| {
                Some((ok.clone(), Some(err.clone()), syn::parse_quote!(v)))
            },
        )
    }
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = p;
        self
    }

    /// When `false`, generated wrappers skip the per-call
    /// `withSortedHandleLocks` scaffold (and the dispatch
    /// `as? NativeHandle` lock-adds), emitting only the raw `ptr` read +
    /// closed-handle null-check + native call. Removes per-call lock
    /// allocations / monitor entry at the cost of thread-safety (no
    /// deadlock-safe N-ary locking, no atomic consume). Default `true`.
    pub fn handle_locks(mut self, on: bool) -> Self {
        self.emit_handle_locks = on;
        self
    }

    /// Mark the most recently declared class
    /// ([`Self::data_class`], [`Self::ptr_class`], or
    /// [`Self::enum_class`]) as throwable. Two effects:
    ///
    /// 1. The emitted Kotlin class extends `Exception` — see the
    ///    `render` module for the renderer's branch.
    /// 2. A `throw_<RustShortName>` free function is generated that
    ///    constructs the JVM object via this type's own output converter
    ///    and throws it. The Result-peel stages built by the rank-2
    ///    `Result<_, _>` wrapper (`JniGen::new`) call into this fn.
    ///
    /// Chains exactly like [`Self::method`] / [`Self::suppress_kotlin_code`];
    /// panics if no class was just declared. The framework's own
    /// [`crate::api::lang::jnigen::jni::JniBindingError`] is pre-registered at
    /// `exceptions[0]` directly inside [`Self::new`] and bypasses this
    /// builder, so its stub-template Kotlin emission stays as-is.
    pub fn throwable(mut self) -> Self {
        let key = self.last_meta_key.clone().expect(
            "JniGen::throwable must be chained immediately after a \
             `data_class`, `ptr_class`, or `enum_class` call",
        );
        let rust_type = key.to_type();
        let cfg = build_exception_config(rust_type, &self.package, &self.exceptions);
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        assert!(
            !entry.value_class,
            "JniGen::throwable: `{}` was declared via `value_class` — \
             @JvmInline value classes cannot extend `Exception`. Use \
             `data_class` for throwable types.",
            key.as_str()
        );
        self.exceptions.push(cfg);
        entry.throwable = true;
        self
    }

    /// Set the JVM/Kotlin base package (dot-separated, e.g.
    /// `"io.zenoh.jni"`). All derived forms (`java_class_prefix`,
    /// `kotlin_callback_package`) are recomputed.
    pub fn package_prefix(mut self, p: impl Into<String>) -> Self {
        self.package = p.into().trim_matches('.').trim_matches('/').to_string();
        self.recompute_derived();
        self
    }
    /// Set the closure that mangles the framework "harness" class name
    /// `"Native"` (the centralized extern holder). Default = prepend
    /// `"JNI"` (yielding `JNINative`). Affects the generated Kotlin
    /// class name and, via [`Self::jni_class_path`], the JNI extern
    /// symbol path on the Rust side.
    pub fn kotlin_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_harness_name_mangle = Some(Arc::new(f));
        self.recompute_derived();
        self
    }
    /// Set the leaf appended to [`Self::package`] for the auto-emitted
    /// callback fun-interface files (e.g. `"callbacks"`). Affects
    /// `kotlin_callback_package`.
    pub fn callback_subpackage(mut self, s: impl Into<String>) -> Self {
        self.callback_subpackage = s.into().trim_matches('.').to_string();
        self.recompute_derived();
        self
    }
    /// Set the closure that mangles function names. Called for every
    /// scanned `#[prebindgen]` free function and the synthetic
    /// `freePtr` destructor; receives the camelCased Kotlin-side name
    /// and returns the final form (e.g. `"putPublisher"` →
    /// `"putPublisherViaJNI"`). Default = identity.
    pub fn kotlin_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_fun_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles Kotlin ptr-class names declared
    /// via [`Self::ptr_class`]. Receives the Rust short name.
    /// Default = identity.
    pub fn kotlin_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_ptr_class_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles Kotlin data-class names declared
    /// via [`Self::data_class`]. Receives the Rust short name.
    /// Default = identity.
    pub fn kotlin_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_data_class_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles [`Self::enum_class`]-declared
    /// enum class names. Receives the Rust short name. Default =
    /// identity.
    pub fn kotlin_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_enum_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles the package-level wrapper object
    /// name created by [`Self::package`]. Default = identity.
    pub fn kotlin_package_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_package_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles `impl Fn(...)` callback class
    /// names. Receives the auto-derived callback name
    /// ([`derive_callback_name`], always
    /// concatenated parameter type shorts + `"Callback"` suffix — e.g.
    /// `"QueryCallback"`, `"ReplyCallback"`, `"Callback"` for `Fn()`);
    /// the returned relative name is qualified against
    /// [`Self::kotlin_callback_package`]. Default = identity.
    pub fn kotlin_callback_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_callback_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles rank-0
    /// [`Self::input_wrapper`] / [`Self::output_wrapper`] pattern
    /// names (e.g. `"Encoding"`). Rank-N patterns are NOT routed
    /// through this closure — they inherit from the inner type's
    /// metadata via the existing rank-N handlers, preserving the
    /// structural invariant `Option<Encoding>` ↔ `JNIEncoding?`.
    /// Default = identity.
    pub fn kotlin_wrapper_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_wrapper_name_mangle = Some(Arc::new(f));
        self
    }

    /// Activate a subpackage context. Subsequent [`Self::function`]
    /// calls land in this subpackage, and any class declared
    /// ([`Self::ptr_class`] / [`Self::data_class`] /
    /// [`Self::enum_class`] / [`Self::value_class`]) while the
    /// subpackage is active gets an FQN of
    /// `<package>.<subpackage>.<ClassName>`.
    ///
    /// Package inheritance is **not** supported — chaining
    /// `.package("a").package("b")` does not produce
    /// `"a.b"`; each call overwrites the previous active subpackage.
    /// To nest, pass a dotted path: `.package("a.b")`.
    ///
    /// Passing an empty string clears the active subpackage (classes /
    /// functions revert to the base `<package>`).
    pub fn package(mut self, subpackage: impl Into<String>) -> Self {
        self.last_opaque_key = None;
        self.last_meta_key = None;
        self.last_entry_ref = None;
        let sub = subpackage
            .into()
            .trim_matches('.')
            .trim_matches('/')
            .to_string();
        if sub.is_empty() {
            self.active_subpackage = None;
        } else {
            self.packages.entry(sub.clone()).or_default();
            self.active_subpackage = Some(sub);
        }
        self
    }

    /// Recompute the derived caches (`java_class_prefix`,
    /// `jni_class_path`, `kotlin_callback_package`) from (`package`,
    /// `kotlin_harness_name_mangle`, `callback_subpackage`). Called by
    /// every setter that touches one of those source fields. The JNI
    /// extern symbol path resolves to the centralized Native object,
    /// whose mangled name comes from the harness mangle (default
    /// `"JNI" + n` → `JNINative`).
    fn recompute_derived(&mut self) {
        self.java_class_prefix = self.package.replace(".", "/");
        let native_class = self.mangle_harness("Native");
        self.jni_class_path = if self.package.is_empty() {
            format!("Java_{}", native_class)
        } else {
            format!("Java_{}_{}", self.package.replace(".", "_"), native_class)
        };
        self.kotlin_callback_package = if self.package.is_empty() {
            self.callback_subpackage.clone()
        } else if self.callback_subpackage.is_empty() {
            self.package.clone()
        } else {
            format!("{}.{}", self.package, self.callback_subpackage)
        };
        // Re-anchor every exception's Kotlin FQN against the (possibly
        // new) package. Each entry's `rust_short` is stable; the FQN is
        // a derived view. In practice `package` is called first in
        // every binding's build script, before any exception class is
        // declared, so the framework slot at index 0 always re-derives
        // cleanly.
        for exc in &mut self.exceptions {
            exc.kotlin_fqn = if self.package.is_empty() {
                exc.rust_short.clone()
            } else {
                format!("{}.{}", self.package, exc.rust_short)
            };
        }
    }

    /// Apply [`Self::kotlin_fun_name_mangle`] to `name`, returning the
    /// closure result or `name` verbatim when unset. Called everywhere
    /// the framework derives a function-shaped Kotlin/JNI short name —
    /// scanned `#[prebindgen]` extern symbols, the synthetic `freePtr`
    /// destructor, and the Kotlin-side `external fun` that pairs with
    /// each.
    pub(crate) fn mangle_fun(&self, name: &str) -> String {
        match &self.kotlin_fun_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_ptr_class_name_mangle`] to `name`,
    /// returning the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_ptr_class(&self, name: &str) -> String {
        match &self.kotlin_ptr_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_data_class_name_mangle`] to `name`,
    /// returning the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_data_class(&self, name: &str) -> String {
        match &self.kotlin_data_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_enum_name_mangle`] to `name`, returning the
    /// closure result or `name` verbatim when unset.
    pub(crate) fn mangle_enum(&self, name: &str) -> String {
        match &self.kotlin_enum_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_callback_name_mangle`] to `name`, returning
    /// the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_callback(&self, name: &str) -> String {
        match &self.kotlin_callback_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_wrapper_name_mangle`] to `name`, returning
    /// the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_wrapper(&self, name: &str) -> String {
        match &self.kotlin_wrapper_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_harness_name_mangle`] to `name`. The
    /// closure defaults to `|n| format!("JNI{n}")` when unset, so calling
    /// `mangle_harness("Native")` yields `"JNINative"`.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.kotlin_harness_name_mangle {
            Some(f) => f(name),
            None => format!("JNI{name}"),
        }
    }
    /// The mangled name of the centralized Native object that hosts
    /// every JNI `external fun`. Drives both the Kotlin class emission
    /// and the JNI extern symbol path on the Rust side.
    pub(crate) fn jni_native_class_name(&self) -> String {
        self.mangle_harness("Native")
    }
    /// The mangled wrapper-object class name for a given subpackage
    /// (one wrapper object per [`Self::package`] context).
    /// Derives from the subpackage's last dot-segment so
    /// `package("a.b")` yields a class named after `b`.
    pub(crate) fn jni_package_class_name(&self, subpackage: &str) -> String {
        let leaf = subpackage
            .rsplit('.')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("Package");
        match &self.kotlin_package_name_mangle {
            Some(f) => f(leaf),
            None => self.mangle_harness(leaf),
        }
    }

    /// Resolve a relative class name against [`Self::package`]. Panics
    /// if `name` contains a `.` (a check that catches accidental FQNs in
    /// the relative-name builders). The framework refuses dotted names
    /// on purpose: a binding crate owns one package and must not write
    /// classes into anyone else's namespace. Higher layers wrap or
    /// re-export — they don't get injected into.
    pub(crate) fn resolve_class_fqn(&self, name: &str) -> String {
        assert!(
            !name.contains('.'),
            "Kotlin class name `{}` must be relative (no dots) — FQNs are derived from JniGen::package",
            name
        );
        // If a `package(p)` context is active, classes declared
        // while it's active land under `<package>.<p>` instead of just
        // `<package>`. The user explicitly opts in by ordering the
        // declaration after the `package` call.
        let base: String = match (&self.package, &self.active_subpackage) {
            (p, Some(sub)) if !p.is_empty() => format!("{}.{}", p, sub),
            (p, Some(sub)) if p.is_empty() => sub.clone(),
            (p, None) => p.clone(),
            _ => String::new(),
        };
        if base.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", base, name)
        }
    }

    /// Resolve a relative callback class name against
    /// `package + "." + callback_subpackage`. Panics if `name` contains a `.`.
    pub(crate) fn resolve_callback_fqn(&self, name: &str) -> String {
        assert!(
            !name.contains('.'),
            "Kotlin callback name `{}` must be relative (no dots) — FQNs are derived from JniGen::package + callback_subpackage",
            name
        );
        if self.kotlin_callback_package.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.kotlin_callback_package, name)
        }
    }
    // ── Structured type-conversion builders ──────────────────────────

    /// Declare a typed Kotlin handle class backed by an opaque Rust
    /// type. Configures: jlong wire for both input and output,
    /// `Box::into_raw`/`Box::from_raw` lifecycle, the `instanceof`
    /// dispatch class, and the Kotlin typed-handle class FQN. By
    /// default a `.kt` shell is auto-emitted — chain
    /// [`Self::suppress_kotlin_code`] to keep the file hand-maintained,
    /// or chain one or more [`Self::method`] calls to promote
    /// `#[prebindgen]` functions onto the class as instance methods.
    pub fn ptr_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_ptr_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.opaque = Some(OpaqueConfig::default());
        // `kotlin_name` holds the typed-handle FQN for FQN-consumers
        // (typed-handle class emission, `instanceof` dispatch, return-
        // value constructor wrap). The value-context Kotlin name for
        // opaque types — `"Long"` — flows separately through
        // [`KotlinMeta::kotlin_name`] produced by the rank-0 opaque
        // handler, so wire-level mentions don't collide with the FQN.
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = Some(key.clone());
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Declare a `#[prebindgen]` function as an **instance method** on
    /// the class declared by the most recent
    /// [`Self::ptr_class`] / [`Self::data_class`] /
    /// [`Self::enum_class`] / [`Self::value_class`] call. The
    /// function's first parameter must syntactically match the class's
    /// Rust type (`&T` / `&mut T` / `T` for opaque handles; `&T` for
    /// non-opaque data/value classes); the wrapper drops it from the
    /// Kotlin signature and substitutes `this`/inherited scope at the
    /// JNI call site. Mismatch is a build-time error (caught when the
    /// wrapper is rendered).
    ///
    /// Panics if no class context is active. For free-standing functions
    /// under [`Self::package`], use [`Self::function`].
    /// For companion-object (`static`-style) methods on a class, use
    /// [`Self::companion_method`].
    pub fn class_fun(mut self, ident: syn::Ident) -> Self {
        let key = self
            .last_meta_key
            .clone()
            .or_else(|| self.last_opaque_key.clone())
            .expect(
                "JniGen::method must be chained immediately after a `ptr_class`, \
                 `data_class`, `enum_class`, or `value_class` call — \
                 for free-standing fns inside `package`, use `.function(...)`; \
                 for static-style class methods, use `.companion_method(...)`",
            );
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        let idx = entry.instance_methods.len();
        entry.instance_methods.push(MethodEntry::new(ident));
        self.last_entry_ref = Some(NamedEntryRef::Method(key, idx));
        self
    }

    /// Declare a `#[prebindgen]` function as a **companion-object method**
    /// on the class declared by the most recent class builder. No
    /// first-param constraint; the wrapper is emitted in `companion
    /// object { ... }` using the same form as a package-level wrapper
    /// (all params present). Panics if no class context is active.
    pub fn class_object_fun(mut self, ident: syn::Ident) -> Self {
        let key = self
            .last_meta_key
            .clone()
            .or_else(|| self.last_opaque_key.clone())
            .expect(
                "JniGen::companion_method must be chained immediately after a \
                 `ptr_class`, `data_class`, `enum_class`, or \
                 `value_class` call",
            );
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        let idx = entry.companion_methods.len();
        entry.companion_methods.push(MethodEntry::new(ident));
        self.last_entry_ref = Some(NamedEntryRef::Companion(key, idx));
        self
    }

    /// Declare a `#[prebindgen]` function as a free-standing wrapper
    /// under the currently-active [`Self::package`] context. If a
    /// class context is also live, calling `function` clears it — the
    /// idea being that "leak class context to package level" makes the
    /// chain unambiguous after one fn-level declaration. Panics if no
    /// `package` is active.
    pub fn package_fun(mut self, ident: syn::Ident) -> Self {
        let sub = self
            .active_subpackage
            .clone()
            .expect("JniGen::function must be chained inside a `package(...)` context");
        // Leak any class context back to package level.
        self.last_meta_key = None;
        self.last_opaque_key = None;
        let pkg = self.packages.entry(sub.clone()).or_default();
        let idx = pkg.functions.len();
        pkg.functions.push(MethodEntry::new(ident));
        self.last_entry_ref = Some(NamedEntryRef::Function(sub, idx));
        self
    }

    /// Override the Kotlin-side name for the most recent
    /// [`Self::method`] / [`Self::companion_method`] / [`Self::function`]
    /// entry. Default (without `.name(...)`) is
    /// `snake_to_camel(rust_ident)` (e.g. `z_hello_whatami` → `zHelloWhatami`).
    /// Panics if not chained immediately after a fn-level builder.
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        let r = self.last_entry_ref.clone().expect(
            "JniGen::name must be chained immediately after `.method(...)`, \
             `.companion_method(...)`, or `.function(...)`",
        );
        let name = kotlin_name.into();
        match r {
            NamedEntryRef::Method(key, idx) => {
                let entry = self.types.get_mut(&key).expect("type entry vanished");
                entry.instance_methods[idx].kotlin_name_override = Some(name);
            }
            NamedEntryRef::Companion(key, idx) => {
                let entry = self.types.get_mut(&key).expect("type entry vanished");
                entry.companion_methods[idx].kotlin_name_override = Some(name);
            }
            NamedEntryRef::Function(sub, idx) => {
                let pkg = self.packages.get_mut(&sub).expect("package entry vanished");
                pkg.functions[idx].kotlin_name_override = Some(name);
            }
        }
        self
    }

    /// Opt out of Kotlin class emission for the most recent
    /// [`Self::ptr_class`] / [`Self::enum_class`] — the `.kt` file is
    /// assumed to be hand-written. Without this, a typed-handle shell
    /// class (or an `enum class`) is auto-emitted. Panics if no
    /// `ptr_class` / `enum_class` is in scope.
    pub fn suppress_kotlin_code(mut self) -> Self {
        let key = self.last_meta_key.clone().expect(
            "JniGen::suppress_kotlin_code must be chained immediately after a \
             `ptr_class` or `enum_class` call",
        );
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        if let Some(opaque) = entry.opaque.as_mut() {
            opaque.suppress_kotlin_code = true;
        } else if let Some(enum_cfg) = entry.enum_cfg.as_mut() {
            enum_cfg.suppress_kotlin_code = true;
        } else {
            panic!(
                "JniGen::suppress_kotlin_code: type entry for `{}` has neither \
                 `opaque` nor `enum_cfg` set — chain after `ptr_class` or \
                 `enum_class`",
                key.as_str()
            );
        }
        self
    }

    /// Whether `ty` was registered via [`Self::enum_class`] — used by
    /// the Kotlin wrapper generator to decide if a parameter needs a
    /// `.value` projection between the typed enum (Kotlin signature) and
    /// the `Int` wire (JNI `external fun`).
    pub(crate) fn is_kotlin_enum(&self, ty: &syn::Type) -> bool {
        let key = TypeKey::from_type(ty);
        self.types
            .get(&key)
            .and_then(|c| c.enum_cfg.as_ref())
            .is_some()
    }

    /// Declare a `#[prebindgen]`-marked `enum` as a Kotlin `enum class`.
    /// Configures: `jni::sys::jint` wire (input + output), `TryFrom<i32>`
    /// decode on input, `as jint` encode on output, and Kotlin enum-file
    /// emission. The enum must be C-like (unit variants only) and either
    /// `#[repr(i32)]` / `#[repr(u8)]` (or similar) with explicit
    /// discriminants — the Kotlin emitter and the generated
    /// `TryFrom<i32>` decode rely on the discriminant values matching the
    /// jint wire.
    ///
    /// By default a `.kt` file is auto-emitted under [`Self::package`]; chain
    /// [`Self::suppress_kotlin_code`] to keep the file hand-maintained.
    /// The class name passes through
    /// [`Self::kotlin_enum_name_mangle`] (default = Rust short name).
    pub fn enum_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_enum(&short));
        let entry = self.types.entry(key.clone()).or_default();
        assert!(
            entry.opaque.is_none(),
            "JniGen::enum_class: `{}` is already registered as an opaque \
             handle via `ptr_class` — a type can be one or the other, \
             not both",
            short
        );
        entry.enum_cfg = Some(EnumConfig::default());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        // Clear opaque tracker so a stray `.method(...)` doesn't latch onto
        // this entry; `last_meta_key` is what `.suppress_kotlin_code` reads
        // for chained config.
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Stamp a verbatim Kotlin type expression (e.g. `"List<ByteArray>"`)
    /// onto the entry registered by the most recent type-config builder.
    /// Use this when the Kotlin type is not a class FQN (generics,
    /// primitives, container types). For class names, the per-kind
    /// `kotlin_*_name_mangle` closures (configured on [`JniGen`]) own
    /// derivation — `with_kotlin_type` is the escape hatch for verbatim
    /// expressions that don't map onto any one element kind.
    pub fn with_kotlin_type(mut self, kotlin_expr: impl Into<String>) -> Self {
        let key = self
            .last_meta_key
            .clone()
            .or_else(|| self.last_opaque_key.clone())
            .expect(
                "JniGen::with_kotlin_type must be chained immediately after a \
                 type-config builder",
            );
        let expr = kotlin_expr.into();
        let entry = self.types.get_mut(&key).expect("meta entry vanished");
        entry.kotlin_name = Some(expr.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), expr));
        self
    }

    /// Install a manual input converter for an `impl Fn(...)` callback
    /// parameter (`JObject` wire). `exc` selects the body convention,
    /// matching the unified [`Self::input_wrapper`] rule:
    ///
    /// * `exc = None` ⇒ non-throwing: emitted body is
    ///   `<dispatcher_path>(env, &v)?` (framework `?`-propagation); only
    ///   valid if the dispatcher returns the framework error.
    /// * `exc = Some(<Rust type>)` ⇒ throwing: the dispatcher is
    ///   expected to return `Result<impl Fn(...), <Rust type>>` (e.g.
    ///   `ZResult<_>`), and the emitted body is the dispatcher call
    ///   directly — no `?`/`Ok`, per the body↔exception coupling. The
    ///   type must match a [`Self::throwable`] declaration
    ///   by exact canonical-form equality (see [`Self::find_exception`]).
    ///
    /// The Kotlin FQN auto-derives via
    /// [`Self::kotlin_callback_name_mangle`] applied to the per-callback
    /// name ([`derive_callback_name`]) and
    /// then qualified against [`Self::kotlin_callback_package`]. Set
    /// the mangler closure on [`JniGen`] to control naming (default =
    /// identity).
    pub fn callback_input(
        mut self,
        impl_fn_type: syn::Type,
        exc: Option<syn::Type>,
        dispatcher_path: syn::Path,
    ) -> Self {
        let key = TypeKey::from_type(&impl_fn_type);
        let dispatcher_path_str = dispatcher_path.to_token_stream().to_string();
        let body_path = dispatcher_path_str.clone();
        // `syn::Type` holds `Rc<TokenStream>` internally and is neither
        // `Send` nor `Sync`, so we can't capture it directly in a builder
        // closure that satisfies `WrapperBuilder<Arity0>`'s `Send + Sync`
        // bounds. Serialise to its canonical token form here and re-parse
        // inside the closure — same dance the path captures use.
        let exc_str = exc.as_ref().map(|t| t.to_token_stream().to_string());
        let builder = move |_reg: &Registry<KotlinMeta>| {
            let path: syn::Path = syn::parse_str(&body_path).ok()?;
            // Throwing: dispatcher already returns `Result<_, exc>` — emit
            // the call verbatim. Non-throwing: framework `?`-propagation
            // unwraps, and the framework `Ok`-wraps later.
            let body: syn::Expr = if exc_str.is_some() {
                syn::parse_quote!(#path(env, &v))
            } else {
                syn::parse_quote!(#path(env, &v)?)
            };
            let exc_ty = exc_str
                .as_deref()
                .and_then(|s| syn::parse_str::<syn::Type>(s).ok());
            Some((syn::parse_quote!(jni::objects::JObject), exc_ty, body))
        };
        // Auto-derive the callback Kotlin FQN via
        // `kotlin_callback_name_mangle` applied to the per-callback name.
        // Stamped at registration time so downstream consumers
        // (`dispatch_fn_input`, `collect_kotlin_callback_fqns`) read a
        // resolved FQN rather than re-deriving it. The presence of
        // `callback_kotlin_fqn` also flags this entry as a callback for
        // emission paths that need to distinguish.
        let args =
            crate::api::core::registry::extract_fn_trait_args(&impl_fn_type).unwrap_or_default();
        let name = derive_callback_name(&args);
        let fqn = self.resolve_callback_fqn(&self.mangle_callback(&name));
        let entry = self.types.entry(key.clone()).or_default();
        entry.callback_kotlin_fqn = Some(fqn.clone());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.input_wrappers[0].insert(key.clone(), builder.into_wrapper_fn());
        self.note_wrapper_registration(key, 0);
        self
    }

    /// Mark an `impl Fn(...)` callback type as having a hand-written
    /// Kotlin fun-interface. The framework keeps its default Rust-side
    /// auto-dispatcher (no [`Self::callback_input`] override here) but
    /// skips emitting the Kotlin auto-stub — the binding crate provides
    /// the `<FQN>.kt` file itself. The Kotlin FQN is auto-derived via
    /// [`Self::mangle_callback`] applied to the callback's name so the
    /// hand-written file name and the JNI-side mention stay in sync.
    /// Equivalent to chaining `.suppress_kotlin_code()` after a
    /// [`Self::ptr_class`] / [`Self::enum_class`] declaration, but
    /// inline because callbacks don't have a `kotlin_callback` builder
    /// to chain off.
    pub fn suppress_kotlin_callback_code(mut self, impl_fn_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&impl_fn_type);
        let args =
            crate::api::core::registry::extract_fn_trait_args(&impl_fn_type).unwrap_or_default();
        let name = derive_callback_name(&args);
        let fqn = self.resolve_callback_fqn(&self.mangle_callback(&name));
        let entry = self.types.entry(key.clone()).or_default();
        entry.callback_kotlin_fqn = Some(fqn.clone());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = None;
        self.last_entry_ref = None;
        self
    }

    /// Declare a Rust struct that should appear in Kotlin as a data
    /// class under a derived name. The name passes through
    /// [`Self::kotlin_data_class_name_mangle`] (default = Rust short
    /// name, generics / lifetimes stripped). Only affects Kotlin
    /// emission — no Rust-side converter override.
    pub fn data_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Declare a Rust struct that should appear in Kotlin as an
    /// `@JvmInline public value class` rather than a `public data class`.
    /// The wrapped struct must have **exactly one named field** and
    /// must not be marked [`Self::throwable`] — both constraints are
    /// enforced at render time with a hard error.
    ///
    /// Why a dedicated builder rather than auto-promoting one-field
    /// data classes: value-class semantics are observable (method-name
    /// mangling, boxing on interface dispatch / generics / nullable
    /// receivers), so the decision must be explicit per-type. Naming
    /// passes through [`Self::kotlin_data_class_name_mangle`] — the
    /// same Kotlin-side namespace as `data_class`.
    ///
    /// Note: `ptr_class` deliberately does **not** support
    /// value-class emission. Typed-handle classes carry a mutable
    /// `@Volatile var ptr` slot, implement `AutoCloseable`, and use
    /// `@Synchronized` for the closed-check + JNI call. A value class
    /// can't express any of those.
    pub fn value_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.kotlin_name = Some(fqn.clone());
        entry.value_class = true;
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Declare a **`Copy` value-blob** type: a Rust type passed across the
    /// JNI boundary **by value as its raw memory bytes** in a `ByteArray`,
    /// rather than as a closeable `jlong` heap handle. The value-level peer
    /// of [`Self::ptr_class`] — `ByteArray` is to a blob what `Long` is to a
    /// handle. Use it for small `Copy` types (e.g. `ZenohId`) so they need no
    /// `close()` and so `Vec<T>` can surface as `List<ByteArray>` (a
    /// `Vec<closeable-handle>` is rejected; see the `Vec<_>` handler).
    ///
    /// The type **must be `Copy`** — the generator emits a compile-time
    /// assertion to that effect (a non-`Copy` declaration is a hard build
    /// error). Conversions reinterpret the bytes (`read_unaligned` on input,
    /// raw-bytes read on output), so the blob is valid only same-architecture
    /// in-process, exactly like an opaque handle pointer. Mutually exclusive
    /// with `ptr_class` / `enum_class` / `value_class`.
    pub fn value_blob(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        // Typed Kotlin FQN for the emitted `@JvmInline value class` — the same
        // FQN-consumer slot a `ptr_class` / `value_class` uses (typed-class
        // emission, projection-leaf lookup, `instanceof` imports). The
        // *value-level* name (`"ByteArray"`) is set separately on the rank-0
        // converter's metadata, so wire mentions stay `ByteArray` while typed
        // positions render the value class.
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.value_blob = true;
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Register `impl Into<target>` source arms. `target_key` is the
    /// canonical Rust type (e.g. `"ZKeyExpr<'static>"`); `sources` is
    /// an ordered list of [`IntoSource`] arms (dispatch order matches
    /// iteration order).
    pub fn into_sources<I>(mut self, target_type: syn::Type, sources: I) -> Self
    where
        I: IntoIterator<Item = IntoSource>,
    {
        let key = TypeKey::from_type(&target_type);
        self.into_sources_map
            .insert(key, sources.into_iter().collect());
        self.last_opaque_key = None;
        self.last_meta_key = None;
        self.last_entry_ref = None;
        self
    }

    /// Register a rank-N **input converter**. `pattern` contains 0–3
    /// `_` placeholders; the closure's arity selects the rank table.
    /// The closure returns `Some((ty, exc, body))` (see [`WrapperFn`]
    /// for the triple's full semantics) or `None` (defer to a later
    /// resolver phase). The body sees `env: &mut JNIEnv` and `v: &<wire>`
    /// in scope.
    ///
    /// * `exc = None` ⇒ non-throwing: `body` evaluates to a bare `ty`;
    ///   framework emits `-> Result<ty, __JniErr>` with an `Ok(...)`
    ///   wrap, and `?` inside propagates the framework error.
    /// * `exc = Some(<Rust type>)` ⇒ throwing: `body` evaluates to
    ///   `Result<ty, <Rust type>>`; framework emits it verbatim. The
    ///   type must match a [`Self::throwable`] declaration
    ///   by **exact canonical-form equality** with its `rust_type` (see
    ///   [`Self::find_exception`] — no short-name fallback). The match
    ///   is validated at lookup time.
    ///
    /// `ty` is auto-classified at resolve: a wire shape ⇒ terminal
    /// converter; a distinct rust type with its own converter ⇒ a
    /// value-inspecting stage composed onto that converter's chain
    /// (see [`Self::lookup_input`]).
    pub fn input_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> Self
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.input_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        s.note_wrapper_registration(key, rank);
        s
    }

    /// Output-direction counterpart of [`Self::input_wrapper`]. Same
    /// closure shape, same `exc = None` / `Some(<Rust type>)` semantics,
    /// same terminal-vs-composed classification — see that method's docs.
    /// (`Some(parse_quote!(<full path>))` with a rust-typed `ty`, e.g.
    /// `(T, Some(parse_quote!(zenoh_flat::errors::ZError)), v)` for
    /// `ZResult<T>`, gives the auto-composed peel that the deleted
    /// `output_throw_stage` used to register.)
    pub fn output_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> Self
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.output_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        s.note_wrapper_registration(key, rank);
        s
    }

    /// Shared post-registration bookkeeping for wrapper inserts. Rank-0
    /// patterns identify a concrete type — auto-stamp `kotlin_name` via
    /// [`Self::mangle_wrapper`] (skipping callback entries, whose
    /// `kotlin_name` is already stamped via
    /// [`Self::mangle_callback`] in [`Self::callback_input`], and
    /// non-path patterns like `()` where there is no sensible short
    /// name). Rank ≥1 patterns are wildcards — per-outer-type names
    /// come from inner-metadata propagation via
    /// [`Self::override_kotlin_name`].
    fn note_wrapper_registration(&mut self, key: TypeKey, rank: usize) {
        self.last_opaque_key = None;
        self.last_entry_ref = None;
        if rank == 0 {
            let entry = self.types.entry(key.clone()).or_default();
            // Skip callbacks (handled by callback_input) and any entry
            // whose kotlin_name has already been stamped (e.g. by an
            // earlier data_class / ptr_class call for the
            // same type — a wrapper layered on top should not override
            // it). Then derive the short name from the canonical
            // TypeKey; non-path patterns ($()$, references, etc.)
            // yield no Kotlin class name and are left as `None`.
            if entry.kotlin_name.is_none() && entry.callback_kotlin_fqn.is_none() {
                if let Some(short) = rust_short_name_opt(&key) {
                    let fqn = self.resolve_class_fqn(&self.mangle_wrapper(&short));
                    let entry = self.types.get_mut(&key).expect("just-inserted entry");
                    entry.kotlin_name = Some(fqn.clone());
                    self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
                }
            }
            self.last_meta_key = Some(key);
        } else {
            self.last_meta_key = None;
        }
    }

    /// [`Self::find_exception`] with a uniform fail-fast panic. `who` is
    /// the caller name for the message.
    fn find_exception_or_panic(&self, who: &str, ty: &syn::Type) -> usize {
        self.find_exception(ty).unwrap_or_else(|| {
            let needle = ty.to_token_stream().to_string();
            panic!(
                "JniGen::{who}: no exception class registered matching `{needle}` — \
                 declare it via `.data_class(<rust path>).throwable()` (or the \
                 ptr/enum equivalent) first, and bind closures to it with \
                 `Some(parse_quote!(<the same path>))`. The framework default is \
                 `::prebindgen::lang::JniBindingError` (or omit the closure's middle \
                 slot — pass `None` — for non-throwing)."
            )
        })
    }

    /// Resolve an exception type against the registered
    /// [`Self::exceptions`] by **exact canonical-form equality** with the
    /// declaration's `rust_type`. No short-name fallback — the closure /
    /// caller must spell the same full path
    /// `.throwable()` declared the type with. Returns the
    /// index into the `exceptions` vec on match.
    fn find_exception(&self, ty: &syn::Type) -> Option<usize> {
        let needle = ty.to_token_stream().to_string();
        self.exceptions
            .iter()
            .position(|e| e.rust_type.to_token_stream().to_string() == needle)
    }

    /// The framework's pre-registered [`crate::api::lang::jnigen::jni::JniBindingError`]
    /// exception. Always exists at `exceptions[0]` after [`Self::new`].
    pub(crate) fn framework_exception(&self) -> &ExceptionConfig {
        &self.exceptions[0]
    }

    /// Build a `KotlinMeta` stamped with the framework's
    /// `JniBindingError` as the converter's *thrown JVM class*. Used by
    /// every built-in fallible converter (primitives, structs,
    /// `Option<_>`, `Vec<_>`, callbacks). Both fields point at the
    /// framework exception:
    ///   * `throws` (FQN) → the Kotlin `@Throws(...)` aggregator;
    ///   * `throws_action` (`throw_JniBindingError`) → the throw fn the
    ///     function wrapper calls when this converter's `?` fails.
    /// The Rust error value flowing in is the unified `__JniErr`
    /// (domain error type), but `throw_JniBindingError` is generic over
    /// `Display`, so it surfaces that value as `JniBindingError` on the
    /// JVM regardless of the value's Rust type. (Bare-wire vs `Result`
    /// output converters are discriminated by signature inspection
    /// [`converter_returns_result`], not by `throws_action`.)
    pub(crate) fn framework_meta(&self, kotlin_name: Option<String>) -> KotlinMeta {
        let exc = self.framework_exception();
        KotlinMeta {
            kotlin_name,
            throws: Some(exc.kotlin_fqn.clone()),
            throws_action: Some(exception_throw_path(exc)),
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
        // Resolve the exception type lazily: validated here, at lookup
        // time, rather than at the `input_wrapper` call site — the
        // closure is the single source of truth for both body shape and
        // bound exception (see [`WrapperFn`]).
        let exc = exc_ty
            .as_ref()
            .map(|t| &self.exceptions[self.find_exception_or_panic("input_wrapper", t)]);
        let outer = substitute_wildcards(pat, args);
        let throw_exc = exc.unwrap_or_else(|| self.framework_exception());
        // Terminal vs composed: `ty` is composed iff it's a *distinct*
        // rust type with its own input converter. The self-check guards
        // the void/identity case (`output_wrapper("()")` returns `ty ==
        // outer`), and the registered-converter probe distinguishes a
        // rust continue-type (compose) from a wire (terminal) without
        // forcing `()` either way. A non-wire `ty` that isn't yet
        // resolved defers.
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
                        .or_else(|| {
                            kotlin_for_wire(&ty)
                        });
                    (Niches::empty(), kn)
                } else {
                    (default_niches_for_wire(&ty), None)
                };
                Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_input_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        throws: Some(throw_exc.kotlin_fqn.clone()),
                        throws_action: Some(exception_throw_path(throw_exc)),
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
                    metadata: KotlinMeta {
                        throws: Some(throw_exc.kotlin_fqn.clone()),
                        throws_action: Some(exception_throw_path(throw_exc)),
                        ..Default::default()
                    },
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
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        throws: inner.metadata.throws.clone(),
                        throws_action: inner.metadata.throws_action.clone(),
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
    ///   returning `Result<wire, err>` (throwing iff [`ConverterReg::exc`]
    ///   is set).
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
        // Resolve at lookup — see [`Self::lookup_input`] for the rationale.
        let exc = exc_ty
            .as_ref()
            .map(|t| &self.exceptions[self.find_exception_or_panic("output_wrapper", t)]);
        let outer = substitute_wildcards(pat, args);
        let throw_exc = exc.unwrap_or_else(|| self.framework_exception());
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
                        .or_else(|| {
                            kotlin_for_wire(&ty)
                        });
                    (kn, None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&ty)
                };
                Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_output_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        throws: Some(throw_exc.kotlin_fqn.clone()),
                        throws_action: Some(exception_throw_path(throw_exc)),
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
                    metadata: KotlinMeta {
                        throws: Some(throw_exc.kotlin_fqn.clone()),
                        throws_action: Some(exception_throw_path(throw_exc)),
                        ..Default::default()
                    },
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
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        throws: inner.metadata.throws.clone(),
                        throws_action: inner.metadata.throws_action.clone(),
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
/// (the void wire of the `output_wrapper("()")` self-converter *and* the
/// unit continue-type of `ZResult<()>`). The terminal-vs-composed
/// decision in [`JniGen::lookup_input`] / [`JniGen::lookup_output`]
/// resolves that ambiguity via the self-check + registered-converter
/// probe, so `()` flows correctly without being force-classified here.
pub(crate) fn is_wire_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Ptr(_))
        || kotlin_for_wire(ty).is_some()
}

/// Bare-ident path to the generated `throw_<short>` free function for
/// `exc` (e.g. `throw_ZError`). Spliced into wrapper code as a direct
/// call — `<path>(env, &err)` — so the trait/macro dance the legacy
/// `throw_exception!` indirection performed is replaced with a plain
/// function call. The path is unqualified because the throw fn lands
/// in the same generated file as every wrapper (emitted from
/// [`Prebindgen::prerequisites`]); same-module name resolution
/// finds it.
pub(crate) fn exception_throw_path(exc: &ExceptionConfig) -> syn::Path {
    let ident = exc.throw_fn_name.clone();
    syn::Path::from(ident)
}

/// Bare-ident type `__JniErr` — the generated file's alias for the
/// framework `JniBindingError`. Non-throwing converters use this as
/// their `Result<…, _>` error type so their bodies' `<__JniErr as
/// From<String>>::from(...)` calls keep compiling, and so a
/// `?`-propagated framework failure surfaces as the framework
/// exception on the JVM. Throwing converters bypass this in favour of
/// their bound exception's own type (see [`JniGen::lookup_input`] /
/// [`JniGen::lookup_output`]). Returned as `syn::Type` so it shares the
/// `err_type` binding with [`ExceptionConfig::rust_type`].
pub(crate) fn default_err_type() -> syn::Type {
    syn::parse_quote!(__JniErr)
}

/// The body expression to splice into a converter `fn` returning
/// `Result<_, E>`, per the body↔exception coupling: a non-throwing
/// converter's `body` is a bare value, so wrap it `Ok(body)`; a throwing
/// converter's `body` already evaluates to the `Result`, so emit it
/// verbatim. (Replaces the old "always-`Ok`-wrap then strip" dance.)
pub(crate) fn body_for_exc(body: &syn::Expr, exc: Option<&ExceptionConfig>) -> syn::Expr {
    if exc.is_some() {
        body.clone()
    } else {
        syn::parse_quote!(Ok(#body))
    }
}

/// Construct an [`ExceptionConfig`] from a path-shaped `syn::Type` and
/// the current Kotlin package. Shared by [`JniGen::new`] (framework
/// `JniBindingError` slot) and [`JniGen::throwable`] (user-declared slots).
/// (user-declared slots). Panics if `rust_type` isn't a `Type::Path`
/// or if its short-name collides with an already-registered exception.
pub(crate) fn build_exception_config(
    rust_type: syn::Type,
    package: &str,
    existing: &[ExceptionConfig],
) -> ExceptionConfig {
    let segs = match &rust_type {
        syn::Type::Path(tp) => &tp.path.segments,
        _ => panic!(
            "throwable: expected a path-shaped type, got `{}`",
            rust_type.to_token_stream()
        ),
    };
    let short = segs.last().map(|s| s.ident.to_string()).unwrap_or_else(|| {
        panic!(
            "throwable: rust type `{}` has no path segments",
            rust_type.to_token_stream()
        )
    });
    let kotlin_fqn = if package.is_empty() {
        short.clone()
    } else {
        format!("{}.{}", package, short)
    };
    let throw_fn_name = format_ident!("throw_{}", short);
    if existing.iter().any(|e| e.throw_fn_name == throw_fn_name) {
        panic!(
            "throwable: another exception is already \
             registered with Rust short name `{}` — rename the Rust \
             type to disambiguate",
            short
        );
    }
    ExceptionConfig {
        rust_type,
        rust_short: short,
        kotlin_fqn,
        throw_fn_name,
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

impl Default for JniGen {
    fn default() -> Self {
        Self::new()
    }
}
