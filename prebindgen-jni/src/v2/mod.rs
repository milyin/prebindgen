//! The JNI binding under the v2 engine.
//!
//! [`Declarations`] keeps accumulating exactly as it does for v1 — same
//! `package!`/`ptr_class!`/`fun!` surface, same `set_*` settings, same
//! name-mangle closures — and this module is the only thing that reads them
//! for the other engine: it turns that storage into a list of
//! [`Declaration`]s, each paired with the [`JniChoice`] saying what the
//! binding declared it as, and a [`JniTarget`] that answers about values of a
//! type wherever they turn up. An ignore is a v1 decision about v1's
//! undeclared-item warnings, which v2 does not emit, so none reaches the
//! engine. [`generate`] hands back a [`Generation`] the frontend writes out —
//! the Rust through the engine's writer, the Kotlin through its own writer in
//! `kotlin.rs`.
//!
//! What a declaration *is* — its Kotlin class or package function, its native
//! method, its `Java_…` symbol, and which setting on it v2 does not honour
//! yet — is the choice beside it, which the engine hands back with every
//! question about that output. The declaration itself names the entity and
//! nothing more, so one function placed twice is two outputs of one
//! declaration rather than one setting overwriting the other.
//!
//! Nothing of v1 runs on this route. Kotlin names, packages and `Java_…`
//! symbols come from this adapter's settings applied to the declarations,
//! which is the same answer v1 would give; the engine never guesses one.

mod kotlin;
mod target;

use prebindgen_registry_v2::{generate, Declaration, EngineError, Generation, PlanningError};
pub use target::{ClassKind, JniChoice, JniPayload, JniTarget};

use crate::jni::{ClassMember, Declarations, FunctionEntry};

impl Declarations {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(
        &self,
        sources: prebindgen_registry::flat::FlatBuilder,
    ) -> Result<Generation<JniPayload>, EngineError> {
        let mut sources = sources;
        // What the binding defines itself enters the model as entities: a
        // helper with the signature `fun!(crate::x).sig(..)` stated, reached
        // where its path says; an opaque class the source never exported,
        // reached at the binding's root. A captured item of the same name is
        // what a class meant, and an error for a function, as under v1.
        for (ident, path, sig) in &self.local_fns {
            let mut sig = sig.clone();
            sig.ident = ident.clone();
            let prefix = prebindgen_registry::decl::local_path_prefix(path);
            let module: syn::Path = syn::parse_str(&prefix).unwrap_or_else(|_| {
                panic!("binding-local fn `{ident}` is declared at an unparseable path `{prefix}`")
            });
            sources = sources.local_function(sig, module);
        }
        // Every declared class, not only the handle ones: a data class over a
        // type the model cannot see into is then refused by the target for
        // what it is, rather than failing the build as a name nothing
        // captured. The item's name is the key's, without the arguments a key
        // may carry — `ptr_class!(Publisher<'static>)` names `Publisher`.
        for key in sorted(self.types.keys()) {
            if let Some(name) = key.short_name().and_then(|name| syn::parse_str(&name).ok()) {
                sources = sources.local_type(name);
            }
        }
        let flat = sources.build()?;
        // The module generated code reaches the source through: the first
        // source's own crate, as v1 resolves it.
        let source_module = flat
            .source_modules()
            .first()
            .and_then(|module| syn::parse_str(module).ok())
            .unwrap_or_else(|| syn::parse_quote!(crate));
        let classes = self
            .types
            .iter()
            .filter_map(|(key, config)| {
                let kind = match config.kind {
                    crate::jni::DeclaredKind::Ptr(_) => ClassKind::Handle,
                    crate::jni::DeclaredKind::Enum(_) => ClassKind::Enum,
                    _ => ClassKind::Data,
                };
                Some((key.as_str().to_string(), (self.kotlin_fqn(key)?, kind)))
            })
            .collect();
        let (target, declarations) = self.binding(&flat, classes)?;
        generate(flat, &target, declarations, source_module)
    }

    /// Write the Kotlin side of a v2 generation under `kotlin_root`.
    pub(crate) fn write_kotlin_v2(
        &self,
        generation: &Generation<JniPayload>,
        kotlin_root: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, kotlin_codegen::WriteKotlinError> {
        kotlin::write(self, generation, kotlin_root)
    }

    /// Everything this binding declared: each declaration paired with what
    /// this target recorded it as, which together are what the engine plans
    /// and accounts for.
    ///
    /// One entry per declaration, in any order. Stating the two halves
    /// together is what keeps them in step: a declaration cannot be planned
    /// without saying what it is.
    ///
    /// Fails when the binding contradicts itself — two declarations claiming
    /// one native method on the harness — which is the frontend's own
    /// validation and not a capability the engine lacks.
    fn binding(
        &self,
        flat: &prebindgen_registry::flat::Flat,
        classes: std::collections::BTreeMap<String, (String, ClassKind)>,
    ) -> Result<(JniTarget, Vec<(Declaration, JniChoice)>), EngineError> {
        let mut target = JniTarget::new(classes);
        let mut declarations = Vec::new();
        // Every wrapper hangs off one harness object, so its native methods
        // share a namespace: two declarations naming the same one would be two
        // definitions of one `Java_…` symbol, which the generated code cannot
        // compile. The binding is what decides the names, so it is told here.
        let mut natives: std::collections::HashMap<String, Declaration> =
            std::collections::HashMap::new();
        let mut collision = None;
        let mut declare = |declaration: Declaration, choice: JniChoice| {
            if let Some(native) = choice.native() {
                if let Some(taken) = natives.insert(native.to_string(), declaration.clone()) {
                    collision.get_or_insert_with(|| {
                        format!(
                            "`{taken}` and `{declaration}` would both be the native method \
                             `{native}` on the JNI harness; give one of them another Kotlin name"
                        )
                    });
                }
            }
            target.declare(&declaration, &choice);
            declarations.push((declaration, choice));
        };

        // A function is declared wherever it is placed — as a class member, as
        // a package function, as the `val` a `constant!(X).fun(..)` reads
        // through — and one function may be placed more than once. Each
        // placement is declared and accounted for on its own, and each needs
        // its own native method, because the harness has one namespace: the
        // name a single placement takes is the Rust identifier's, as v1 names
        // it, and a further placement is named after where it is placed.
        let placements: std::collections::HashMap<&syn::Ident, usize> = self
            .class_members
            .values()
            .flatten()
            .map(|member| &member.rust_ident)
            .chain(self.packages.values().flat_map(|config| {
                config
                    .functions
                    .iter()
                    .chain(&config.constant_functions)
                    .map(|entry| &entry.rust_ident)
            }))
            .fold(std::collections::HashMap::new(), |mut count, ident| {
                *count.entry(ident).or_default() += 1;
                count
            });
        let placed_more_than_once =
            |ident: &syn::Ident| placements.get(ident).is_some_and(|n| *n > 1);

        // Declared classes. A data class is the one representation v2 lowers;
        // the per-type entry makes every value of the type cross that way,
        // wherever it appears. A declared class need not name a captured item:
        // a target may represent `String` or `Vec<u8>` without the source
        // exporting one.
        for key in sorted(self.types.keys()) {
            let config = &self.types[key];
            let placement = self.kotlin_fqn(key).unwrap_or_default();
            let declarator = declarator(&config.kind);
            declare(
                Declaration::Type(key.clone()),
                match config.kind {
                    crate::jni::DeclaredKind::Data => JniChoice::DataClass {
                        class: placement.clone(),
                    },
                    // A Kotlin `enum class` of the same values: what crosses
                    // is the number each value carries.
                    crate::jni::DeclaredKind::Enum(_) => JniChoice::EnumClass {
                        class: placement.clone(),
                    },
                    // The release is a native method on the harness like any
                    // other, named after the class: `freeLedger`.
                    crate::jni::DeclaredKind::Ptr(_) => {
                        let short = placement.rsplit('.').next().unwrap_or_default();
                        let native = self.mangle_jni_method(&format!("free{short}"));
                        JniChoice::PtrClass {
                            class: placement.clone(),
                            symbol: self.native_method_symbol(&native),
                            native,
                        }
                    }
                    _ => JniChoice::unimplemented(declarator, placement.clone()),
                },
            );

            // Members are separately selected: a class can be emitted with one
            // of its methods skipped, so each is a declaration of its own. None is
            // lowered yet: a method's receiver is a handle.
            for member in self.class_members.get(key).into_iter().flatten() {
                let placed = format!("{placement}.{}", self.effective_method_name(key, member));
                declare(
                    Declaration::Function(member.rust_ident.clone()),
                    JniChoice::unimplemented(member_representation(member), placed),
                );
            }
        }

        // Free-standing package functions and constants.
        for (subpackage, config) in &self.packages {
            let package = self.package_name(subpackage);
            let placed = |entry: &FunctionEntry| {
                format!(
                    "{package}.{}",
                    self.effective_function_name(subpackage, entry)
                )
            };
            for entry in &config.functions {
                let method = self.effective_function_name(subpackage, entry);
                // The native method is named from the Rust identifier, through
                // the method-name hook, as v1 names it — never from the public
                // function's `.name()`: two packages may each export a `value`,
                // and the harness has one namespace. A function placed more
                // than once is the exception: each placement is a wrapper of
                // its own, so each is named after where it is placed, which is
                // what tells the placements apart.
                let native_name = match placed_more_than_once(&entry.rust_ident) {
                    true if subpackage.is_empty() => method.clone(),
                    true => format!("{}_{method}", subpackage.replace('.', "_")),
                    false => entry.rust_ident.to_string(),
                };
                let native = self.mangle_jni_method(&crate::util::snake_to_camel(&native_name));
                // A function under a setting v2 does not honour is still a
                // `fun` in the report; the setting is the capability missing.
                // It is refused rather than emitted with the setting dropped:
                // a lookup that fell back to the default here would generate
                // an interface the binding did not ask for.
                declare(
                    Declaration::Function(entry.rust_ident.clone()),
                    match self.unimplemented_setting(flat, &entry.rust_ident) {
                        Some(setting) => JniChoice::Unimplemented {
                            declarator: "fun",
                            capability: setting,
                            placement: placed(entry),
                        },
                        None => JniChoice::Function {
                            package: package.clone(),
                            symbol: self.native_method_symbol(&native),
                            native,
                            method,
                        },
                    },
                );
            }
            // A `constant!(X)` names the `#[prebindgen]` const it reads. The
            // Kotlin `val` keeps the const's own name — it is not a function
            // and takes neither the camel-casing nor the function-name hook.
            for entry in &config.constants {
                let placed = format!(
                    "{package}.{}",
                    entry
                        .kotlin_name_override
                        .clone()
                        .unwrap_or_else(|| entry.rust_ident.to_string())
                );
                declare(
                    Declaration::Const(entry.rust_ident.clone()),
                    JniChoice::unimplemented("constant", placed),
                );
            }
            // A `constant!(X).fun(..)` is a Kotlin `val` read through a nullary
            // function: the declaration is the function's, and the `val` is
            // what this target chooses to show the call as.
            for entry in &config.constant_functions {
                declare(
                    Declaration::Function(entry.rust_ident.clone()),
                    JniChoice::unimplemented("constant_fun", placed(entry)),
                );
            }
            // A `constant!(X).expr(..)` has no Rust item behind it at all.
            for decl in &config.constant_exprs {
                declare(
                    Declaration::ComputedConst(decl.kotlin_name.clone()),
                    JniChoice::unimplemented(
                        "constant_expr",
                        format!("{package}.{}", decl.kotlin_name),
                    ),
                );
            }
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        //
        // A binding-local fn is NOT a declaration of its own. It is a helper the
        // binding defines, and what the target exports is the member or the
        // package function it was bound to — already stated above. Listing it
        // twice would give one id to two entries.
        for decl in &self.convert_decls {
            declare(
                Declaration::Conversion(decl.key().clone()),
                JniChoice::unimplemented(
                    "convert",
                    self.kotlin_fqn(decl.key()).unwrap_or_default(),
                ),
            );
        }

        if let Some(collision) = collision {
            return Err(EngineError::Planning(PlanningError::InvalidInput(
                collision,
            )));
        }
        Ok((target, declarations))
    }
}

impl Declarations {
    /// A setting v2 does not lower yet that applies to this function, if any.
    ///
    /// Either the function's own — a per-function `expand_param`,
    /// `expand_return` or `split_on_param` — or a type-level boundary
    /// declaration for the type of one of its parameters or of its result,
    /// which is where such a declaration takes effect (a field of that type is
    /// not a boundary). A function under such a setting is refused rather than
    /// emitted with the setting silently dropped: the default interface is not
    /// the one the binding asked for.
    fn unimplemented_setting(
        &self,
        flat: &prebindgen_registry::flat::Flat,
        ident: &syn::Ident,
    ) -> Option<&'static str> {
        if self.fn_param_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_param");
        }
        if self.fn_return_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_return");
        }
        if self.fn_split_params.iter().any(|(fun, ..)| fun == ident) {
            return Some("split_on_param");
        }
        // A binding-local function is not in the model; the engine refuses it
        // on its own account.
        let function = flat.function(&ident.to_string())?;
        if function.params.iter().any(|param| {
            self.param_expand_decls
                .iter()
                .any(|decl| *decl.key() == param.ty.key())
        }) {
            return Some("expand_param");
        }
        if self
            .return_expand_decls
            .iter()
            .any(|decl| *decl.key() == function.ret.key())
        {
            return Some("expand_return");
        }
        None
    }
}

/// The declarations' hash-keyed storage, in a stable order: the engine emits in
/// request order, and a run over unchanged input has to write the same file.
fn sorted<T: Ord>(keys: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut keys: Vec<T> = keys.into_iter().collect();
    keys.sort();
    keys
}

/// A declared class's declarator, as the report names it.
fn declarator(kind: &crate::jni::DeclaredKind) -> &'static str {
    match kind {
        crate::jni::DeclaredKind::Ptr(_) => "ptr_class",
        crate::jni::DeclaredKind::Enum(_) => "enum_class",
        crate::jni::DeclaredKind::Sealed(_) => "sealed_class",
        crate::jni::DeclaredKind::Data => "data_class",
    }
}

/// A class member's declarator, as the report names it.
fn member_representation(member: &ClassMember) -> &'static str {
    match member.kind {
        crate::jni::MemberKind::Method => "method",
        crate::jni::MemberKind::Constructor => "constructor",
    }
}

/// Print one cargo warning per capability a skip named, with the declarations
/// it took down.
///
/// A build log is not a list of everything: at most five declarations per
/// capability, and a count of the rest. What a build script needs from it is
/// which capability to ask for next, not which of forty declarations waits on
/// it.
pub(crate) fn warn_skipped(skipped: &[(Declaration, prebindgen_registry_v2::Skip)]) {
    let mut by_capability: std::collections::BTreeMap<&str, Vec<String>> =
        std::collections::BTreeMap::new();
    for (declaration, skip) in skipped {
        by_capability
            .entry(skip.capability.as_str())
            .or_default()
            .push(declaration.to_string());
    }
    for (capability, mut roots) in by_capability {
        roots.sort();
        roots.dedup();
        let shown = roots.len().min(5);
        let more = match roots.len() - shown {
            0 => String::new(),
            rest => format!(" (+{rest} more)"),
        };
        println!(
            "cargo:warning=SKIP {capability}: {}{more}",
            roots[..shown].join(", ")
        );
    }
}
