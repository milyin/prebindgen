//! The C binding under the v2 engine.
//!
//! [`CbindgenBuilder`] keeps accumulating declarations exactly as it does for
//! v1 — same builder, same modifiers, same manglers — and this module is the
//! only thing that reads them for the other engine: it turns that storage into
//! the [`BindingRequests`] the registry plans, hands them to [`generate`] with
//! the [`CTarget`], and gets back a [`Generation`] the frontend writes out.
//!
//! Nothing of v1 runs on this route. The engine reads the same sources and the
//! same declarations and owns everything after that; the frontend's part is
//! naming — which C name a type or a symbol gets is its manglers applied, the
//! same answer v1 would give — and saying, in each policy, which declarator
//! produced the declaration, so a skip can name the capability it waits for
//! and the report can print the word back.

mod target;

use prebindgen_registry_v2::{generate, BindingRequests, Declaration, EngineError, Generation};
pub use target::{CPayload, CPolicy, CTarget};

use crate::CbindgenBuilder;

impl CbindgenBuilder {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(
        &self,
        declaring_crate: impl Into<String>,
    ) -> Result<Generation<CPayload>, EngineError> {
        let flat = self.sources.clone().build()?;
        // The module generated code reaches the source through: the one the
        // build script set, else the first source's own crate.
        let source_module = self.source_module.clone().unwrap_or_else(|| {
            flat.source_modules()
                .first()
                .and_then(|module| syn::parse_str(module).ok())
                .unwrap_or_else(|| syn::parse_quote!(crate))
        });
        let requests = self.requests(source_module);
        generate(flat, &CTarget, requests, declaring_crate)
    }

    /// Everything this binding declared, as the engine plans it.
    ///
    /// One entry per declaration, in any order — the report sorts. Each
    /// declaration's policy carries the C name it gets and which declarator it
    /// came from: what the target generates from, and what the report prints.
    fn requests(&self, source_module: syn::Path) -> BindingRequests<CPolicy> {
        let mut requests = BindingRequests::new("c", source_module, CPolicy::Scalar);

        // By-value data structs: the one type representation v2 lowers. The
        // type policy makes every value of the type cross this way, wherever
        // it appears.
        for key in sorted(self.data.keys()) {
            let c_name = self.c_type_name(key);
            let policy = requests.policy(CPolicy::DataStruct {
                c_name: c_name.clone(),
            });
            requests
                .type_policies
                .insert(key.as_str().to_string(), policy);
            requests.output(Declaration::LocalType(key.clone()), policy);
        }

        // Opaque handles: `<c_name> *` to a Rust-owned value, freed through
        // the typed destructor the manglers name.
        for key in sorted(self.opaque.keys()) {
            let c_name = self.c_type_name(key);
            let policy = requests.policy(CPolicy::OpaquePtr {
                c_name: c_name.clone(),
                release: self.destructor_symbol(key).to_string(),
            });
            requests
                .type_policies
                .insert(key.as_str().to_string(), policy);
            requests.output(Declaration::LocalType(key.clone()), policy);
        }

        // Every other declarator is accounted for and refused by name. A
        // declared type need not be a captured item: `String` crosses as an
        // opaque handle in perftest-c and the source never exported it.
        for (keys, declarator) in [
            (sorted(self.value_opaque.keys()), "value_opaque"),
            (sorted(self.enums.keys()), "enum_type"),
            (sorted(self.tagged_unions.keys()), "tagged_union"),
        ] {
            for key in keys {
                let policy = requests.policy(CPolicy::Unimplemented {
                    declarator,
                    c_name: self.c_type_name(key),
                });
                requests
                    .type_policies
                    .insert(key.as_str().to_string(), policy);
                requests.output(Declaration::LocalType(key.clone()), policy);
            }
        }

        // Callback signatures: no captured item names one, and its C closure
        // struct is what the target places.
        for key in sorted(self.callbacks.keys()) {
            let policy = requests.policy(CPolicy::Unimplemented {
                declarator: "callback",
                c_name: self.callback_c_name(key),
            });
            requests.output(Declaration::Callback(describe_callback(key)), policy);
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        for decl in &self.convert_decls {
            let policy = requests.policy(CPolicy::Unimplemented {
                declarator: "convert",
                c_name: self.c_type_name(decl.key()),
            });
            requests.output(Declaration::Conversion(decl.key().clone()), policy);
        }

        // Exported functions — the one kind that must name a captured item.
        for ident in sorted(self.functions.keys()) {
            let symbol = self.fn_symbol(ident).to_string();
            let policy = requests.policy(CPolicy::Function {
                symbol: symbol.clone(),
            });
            requests.output(Declaration::Function(ident.clone()), policy);
        }

        // Ignores are decisions, accounted apart from the gaps.
        for ident in sorted(&self.ignored_functions) {
            requests
                .ignored
                .push(Declaration::LocalFunction(ident.to_string()));
        }
        for key in sorted(&self.ignored_types) {
            requests.ignored.push(Declaration::LocalType(key.clone()));
        }
        requests
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
