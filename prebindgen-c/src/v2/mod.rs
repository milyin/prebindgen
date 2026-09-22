//! The C binding under the v2 engine.
//!
//! [`CbindgenBuilder`] keeps accumulating declarations exactly as it does for
//! v1 — same builder, same modifiers, same manglers — and this module is the
//! only thing that reads them for the other engine: it turns that storage into
//! a list of [`Declaration`]s, each paired with the [`CChoice`] saying what
//! the binding declared it as, and a [`CTarget`] that answers about values of
//! a type wherever they turn up. An ignore is a v1 decision about v1's
//! undeclared-item warnings, which v2 does not emit, so none reaches the
//! engine. [`generate`] hands back a [`Generation`] the frontend writes out.
//!
//! What a declaration *is* — its C name, which declarator produced it, which
//! destructor frees it — is the choice beside it, which the engine hands back
//! with every question about that output. The declaration itself names the
//! entity and nothing more.
//!
//! Nothing of v1 runs on this route. The engine reads the same sources and the
//! same declarations and owns everything after that; the frontend's part is
//! naming — which C name a type or a symbol gets is its manglers applied, the
//! same answer v1 would give — and saying, in each declaration, which
//! declarator produced it, so a skip can name the capability it waits for and
//! the report can print the word back.

mod target;

use prebindgen_registry_v2::{generate, Declaration, EngineError, Generation};
pub use target::{CChoice, CPayload, CTarget};

use crate::CbindgenBuilder;

impl CbindgenBuilder {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(&self) -> Result<Generation<CPayload>, EngineError> {
        // A declared type the source never exported — `String` as a handle —
        // is an item the binding defines: the model gets an extern for it, and
        // steps aside where the source captured a type of that name. Every
        // declared type is registered, not only the opaque ones: a data
        // declaration over a type the model cannot see into is then refused
        // by the target for what it is, rather than failing the build as a
        // name nothing captured. The item's name is the key's, without the
        // arguments a key may carry — `ptr_type!(Publisher<'static>)` names
        // the item `Publisher`.
        let mut sources = self.sources.clone();
        let declared_types = self
            .data
            .keys()
            .chain(self.opaque.keys())
            .chain(self.value_opaque.keys())
            .chain(self.enums.keys())
            .chain(self.tagged_unions.keys());
        for key in sorted(declared_types) {
            if let Some(name) = key.short_name().and_then(|name| syn::parse_str(&name).ok()) {
                sources = sources.local_type(name);
            }
        }
        let flat = sources.build()?;
        // The module generated code reaches the source through: the one the
        // build script set, else the first source's own crate.
        let source_module = self.source_module.clone().unwrap_or_else(|| {
            flat.source_modules()
                .first()
                .and_then(|module| syn::parse_str(module).ok())
                .unwrap_or_else(|| syn::parse_quote!(crate))
        });
        let (target, declarations) = self.binding();
        generate(flat, &target, declarations, source_module)
    }

    /// Everything this binding declared: each declaration paired with what
    /// this target recorded it as, which together are what the engine plans
    /// and accounts for.
    ///
    /// One entry per declaration, in any order — the report sorts. Stating the
    /// two halves together is what keeps them in step: a declaration cannot be
    /// planned without saying what it is.
    fn binding(&self) -> (CTarget, Vec<(Declaration, CChoice)>) {
        let mut target = CTarget::default();
        let mut declarations = Vec::new();
        let mut declare = |declaration: Declaration, choice: CChoice| {
            target.declare(&declaration, &choice);
            declarations.push((declaration, choice));
        };

        // By-value data structs: the one type representation v2 lowers. The
        // per-type entry makes every value of the type cross this way,
        // wherever it appears.
        for key in sorted(self.data.keys()) {
            declare(
                Declaration::Type(key.clone()),
                CChoice::DataStruct {
                    c_name: self.c_type_name(key),
                },
            );
        }

        // Opaque handles: `<c_name> *` to a Rust-owned value, freed through
        // the typed destructor the manglers name.
        for key in sorted(self.opaque.keys()) {
            declare(
                Declaration::Type(key.clone()),
                CChoice::OpaquePtr {
                    c_name: self.c_type_name(key),
                    release: self.destructor_symbol(key).to_string(),
                },
            );
        }

        // A declared enum: C sees a `repr(C)` enum of the same values, and a
        // value of the type crosses as one of them.
        for key in sorted(self.enums.keys()) {
            declare(
                Declaration::Type(key.clone()),
                CChoice::Enum {
                    c_name: self.c_type_name(key),
                },
            );
        }

        // Every other declarator is accounted for and refused by name. A
        // declared type need not be a captured item: `String` crosses as an
        // opaque handle in perftest-c and the source never exported it.
        for (keys, declarator) in [
            (sorted(self.value_opaque.keys()), "value_opaque"),
            (sorted(self.tagged_unions.keys()), "tagged_union"),
        ] {
            for key in keys {
                declare(
                    Declaration::Type(key.clone()),
                    CChoice::Unimplemented {
                        declarator,
                        c_name: self.c_type_name(key),
                    },
                );
            }
        }

        // Callback signatures: no captured item names one, and its C closure
        // struct is what the target places.
        for key in sorted(self.callbacks.keys()) {
            declare(
                Declaration::Callback(describe_callback(key)),
                CChoice::Unimplemented {
                    declarator: "callback",
                    c_name: self.callback_c_name(key),
                },
            );
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        for decl in &self.convert_decls {
            declare(
                Declaration::Conversion(decl.key().clone()),
                CChoice::Unimplemented {
                    declarator: "convert",
                    c_name: self.c_type_name(decl.key()),
                },
            );
        }

        // Exported functions — the one kind that must name a captured item.
        for ident in sorted(self.functions.keys()) {
            declare(
                Declaration::Function(ident.clone()),
                CChoice::Function {
                    symbol: self.fn_symbol(ident).to_string(),
                },
            );
        }

        (target, declarations)
    }
}

/// The builder's hash-keyed storage, in a stable order: the engine emits in
/// request order, and a run over unchanged input has to write the same file.
fn sorted<T: Ord>(keys: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut keys: Vec<T> = keys.into_iter().collect();
    keys.sort();
    keys
}

/// A callback signature as its own name: `impl Fn(&Payload)`.
///
/// A canonical type key spells a generic with spaces around its brackets
/// (`Option < Grade >`), which is right for a key and unreadable in a report, so
/// the argument list is closed up here. The result is still derived only from
/// the keys, and so is still stable across runs.
fn describe_callback(key: &[prebindgen_registry::TypeKey]) -> String {
    let args: Vec<String> = key
        .iter()
        .map(|k| prebindgen_registry::close_up(k.as_str()))
        .collect();
    format!("impl Fn({})", args.join(", "))
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
