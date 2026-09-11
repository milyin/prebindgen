//! The JNI binding under the v2 engine.
//!
//! [`Declarations`] keeps accumulating exactly as it does for v1 — same
//! `package!`/`ptr_class!`/`fun!` surface, same `set_*` settings, same
//! name-mangle closures — and this module is the only thing that reads them
//! for the other engine: it turns that storage into the [`BindingRequests`]
//! the registry plans, hands them to [`generate`] with the [`JniTarget`], and
//! gets back a [`Generation`] the frontend writes out — the Rust through the
//! engine's writer, the Kotlin through its own writer in `kotlin.rs`.
//!
//! Nothing of v1 runs on this route. Kotlin names, packages and `Java_…`
//! symbols come from this adapter's settings applied to the declarations,
//! which is the same answer v1 would give; the engine never guesses one.

mod kotlin;
mod target;

use prebindgen_registry_v2::{
    generate, BindingRequests, DeclaredElement, ElementKind, EngineError, Generation, SourceKind,
};
pub use target::{JniPayload, JniPolicy, JniTarget};

use crate::jni::{ClassMember, Declarations, FunctionEntry};

impl Declarations {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(
        &self,
        sources: prebindgen_registry::flat::FlatBuilder,
        declaring_crate: impl Into<String>,
    ) -> Result<Generation<JniPayload>, EngineError> {
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
            .keys()
            .filter_map(|key| Some((key.as_str().to_string(), self.kotlin_fqn(key)?)))
            .collect();
        let requests = self.requests(source_module);
        generate(flat, &JniTarget::new(classes), requests, declaring_crate)
    }

    /// Write the Kotlin side of a v2 generation under `kotlin_root`.
    pub(crate) fn write_kotlin_v2(
        &self,
        generation: &Generation<JniPayload>,
        kotlin_root: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, kotlin_codegen::WriteKotlinError> {
        kotlin::write(self, generation, kotlin_root)
    }

    /// Everything this binding declared, as the engine plans it.
    ///
    /// One entry per declaration, in any order — the report sorts. Which
    /// declarator a thing came from is this adapter's word for its
    /// representation, printed back by the report rather than re-derived.
    fn requests(&self, source_module: syn::Path) -> BindingRequests<JniPolicy> {
        let mut requests = BindingRequests::new("jni", source_module, JniPolicy::Scalar);

        // A binding-local fn — `fun!(crate::x).sig(..)` — is declared like any
        // other, as a class member or as a package function, but the binding
        // defines it, so the source never captured it. It is stated where it is
        // bound and nowhere else: a second entry for the helper itself would
        // give one id to two elements.
        let is_local = |ident: &syn::Ident| self.local_fns.iter().any(|(local, ..)| local == ident);
        let stated = |element: DeclaredElement, ident: &syn::Ident| {
            if is_local(ident) {
                element.local()
            } else {
                element
            }
        };

        // Declared classes. A data class is the one representation v2 lowers;
        // the type policy makes every value of the type cross that way,
        // wherever it appears. A declared class need not name a captured item:
        // a target may represent `String` or `Vec<u8>` without the source
        // exporting one.
        for key in sorted(self.types.keys()) {
            let config = &self.types[key];
            let placement = self.kotlin_fqn(key).unwrap_or_default();
            let declarator = declarator(&config.kind);
            // A type-level boundary declaration changes how every value of the
            // type crosses, and v2 has no lowering for it: the type is refused
            // under that declarator, whatever class it was declared as.
            let expanded = self
                .param_expand_decls
                .iter()
                .any(|decl| decl.key() == key)
                .then_some("expand_param")
                .or_else(|| {
                    self.return_expand_decls
                        .iter()
                        .any(|decl| decl.key() == key)
                        .then_some("expand_return")
                });
            let policy = requests.policy(match (&config.kind, expanded) {
                (_, Some(declarator)) => JniPolicy::Unimplemented { declarator },
                (crate::jni::DeclaredKind::Data, None) => JniPolicy::DataClass {
                    class: placement.clone(),
                },
                _ => JniPolicy::Unimplemented { declarator },
            });
            requests
                .type_policies
                .insert(key.as_str().to_string(), policy);
            requests.output(
                DeclaredElement::new(
                    ElementKind::Type,
                    key.as_str(),
                    placement.clone(),
                    declarator,
                )
                .local(),
                policy,
            );

            // Members are separately selected: a class can be emitted with one
            // of its methods skipped, so each is an element of its own. None is
            // lowered yet: a method's receiver is a handle.
            for member in self.class_members.get(key).into_iter().flatten() {
                let declarator = member_representation(member);
                let policy = requests.policy(JniPolicy::Unimplemented { declarator });
                requests.output(
                    stated(
                        DeclaredElement::new(
                            ElementKind::Function,
                            member.rust_ident.to_string(),
                            format!("{placement}.{}", self.effective_method_name(key, member)),
                            declarator,
                        ),
                        &member.rust_ident,
                    ),
                    policy,
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
                // and the harness has one namespace.
                let native = self
                    .mangle_jni_method(&crate::util::snake_to_camel(&entry.rust_ident.to_string()));
                let policy = requests.policy(match self.unimplemented_setting(&entry.rust_ident) {
                    Some(declarator) => JniPolicy::Unimplemented { declarator },
                    None => JniPolicy::Function {
                        package: package.clone(),
                        symbol: self.native_method_symbol(&native),
                        native,
                        method,
                    },
                });
                requests.output(
                    stated(
                        DeclaredElement::new(
                            ElementKind::Function,
                            entry.rust_ident.to_string(),
                            placed(entry),
                            "fun",
                        ),
                        &entry.rust_ident,
                    ),
                    policy,
                );
            }
            // A `constant!(X)` names the `#[prebindgen]` const it reads.
            let unimplemented = requests.policy(JniPolicy::Unimplemented {
                declarator: "constant",
            });
            for entry in &config.constants {
                requests.output(
                    DeclaredElement::new(
                        ElementKind::Const,
                        entry.rust_ident.to_string(),
                        placed(entry),
                        "constant",
                    ),
                    unimplemented,
                );
            }
            // A `constant!(X).fun(..)` is a Kotlin `val` backed by a nullary
            // captured **function**, so its target kind and its source kind
            // differ — and a binding-local one names no captured item at all.
            for entry in &config.constant_functions {
                let element = DeclaredElement::new(
                    ElementKind::Const,
                    entry.rust_ident.to_string(),
                    placed(entry),
                    "constant_fun",
                );
                requests.output(
                    stated(element.sourced_as(SourceKind::Function), &entry.rust_ident),
                    unimplemented,
                );
            }
            // A `constant!(X).expr(..)` has no Rust item behind it at all.
            for decl in &config.constant_exprs {
                requests.output(
                    DeclaredElement::new(
                        ElementKind::Const,
                        decl.kotlin_name.clone(),
                        format!("{package}.{}", decl.kotlin_name),
                        "constant_expr",
                    )
                    .local(),
                    unimplemented,
                );
            }
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        //
        // A binding-local fn is NOT an element of its own. It is a helper the
        // binding defines, and what the target exports is the member or the
        // package function it was bound to — already stated above. Listing it
        // twice would give one id to two entries.
        let unimplemented = requests.policy(JniPolicy::Unimplemented {
            declarator: "convert",
        });
        for decl in &self.convert_decls {
            requests.output(
                DeclaredElement::new(
                    ElementKind::Conversion,
                    decl.key().as_str(),
                    self.kotlin_fqn(decl.key()).unwrap_or_default(),
                    "convert",
                )
                .local(),
                unimplemented,
            );
        }

        // Ignores are decisions, accounted apart from the gaps.
        for ident in sorted(&self.ignored_fns) {
            requests.ignored.push(
                DeclaredElement::new(
                    ElementKind::Function,
                    ident.to_string(),
                    String::new(),
                    "ignore",
                )
                .local(),
            );
        }
        for key in sorted(&self.ignored_class_types) {
            requests.ignored.push(
                DeclaredElement::new(ElementKind::Type, key.as_str(), String::new(), "ignore")
                    .local(),
            );
        }
        for ident in sorted(&self.ignored_const_idents) {
            requests.ignored.push(
                DeclaredElement::new(
                    ElementKind::Const,
                    ident.to_string(),
                    String::new(),
                    "ignore_const",
                )
                .local(),
            );
        }
        requests
    }
}

impl Declarations {
    /// A per-function setting v2 does not lower yet, if the function has one.
    ///
    /// A function declared with such a setting is refused under it rather than
    /// emitted with the setting silently dropped: the default interface is not
    /// the one the binding asked for.
    fn unimplemented_setting(&self, ident: &syn::Ident) -> Option<&'static str> {
        if self.fn_param_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_param");
        }
        if self.fn_return_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_return");
        }
        if self.fn_split_params.iter().any(|(fun, ..)| fun == ident) {
            return Some("split_on_param");
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
