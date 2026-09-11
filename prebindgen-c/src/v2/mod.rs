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
//! same answer v1 would give — and saying which declarator produced each
//! element, so a skip can name the capability it waits for.

mod target;

use prebindgen_registry_v2::{
    generate, BindingRequests, DeclaredElement, ElementKind, EngineError, Generation,
};
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
    /// One entry per declaration, in any order — the report sorts. Which
    /// declarator a type came from is this adapter's word for its
    /// representation, printed back by the report rather than re-derived.
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
            requests.output(
                DeclaredElement::new(ElementKind::Type, key.as_str(), c_name, "data_struct")
                    .local(),
                policy,
            );
        }

        // Every other declarator is accounted for and refused by name. A
        // declared type need not be a captured item: `String` crosses as an
        // opaque handle in perftest-c and the source never exported it.
        for (keys, declarator) in [
            (sorted(self.opaque.keys()), "opaque_ptr"),
            (sorted(self.value_opaque.keys()), "value_opaque"),
            (sorted(self.enums.keys()), "enum_type"),
            (sorted(self.tagged_unions.keys()), "tagged_union"),
        ] {
            for key in keys {
                let policy = requests.policy(CPolicy::Unimplemented { declarator });
                requests
                    .type_policies
                    .insert(key.as_str().to_string(), policy);
                requests.output(
                    DeclaredElement::new(
                        ElementKind::Type,
                        key.as_str(),
                        self.c_type_name(key),
                        declarator,
                    )
                    .local(),
                    policy,
                );
            }
        }

        // Callback signatures: no captured item names one, and its C closure
        // struct is what the target places.
        let unimplemented = requests.policy(CPolicy::Unimplemented {
            declarator: "callback",
        });
        for key in sorted(self.callbacks.keys()) {
            requests.output(
                DeclaredElement::new(
                    ElementKind::Callback,
                    describe_callback(key),
                    self.callback_c_name(key),
                    "callback",
                )
                .local(),
                unimplemented,
            );
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        let unimplemented = requests.policy(CPolicy::Unimplemented {
            declarator: "convert",
        });
        for decl in &self.convert_decls {
            requests.output(
                DeclaredElement::new(
                    ElementKind::Conversion,
                    decl.key().as_str(),
                    self.c_type_name(decl.key()),
                    "convert",
                )
                .local(),
                unimplemented,
            );
        }

        // Exported functions — the one kind that must name a captured item.
        for ident in sorted(self.functions.keys()) {
            let symbol = self.fn_symbol(ident).to_string();
            let policy = requests.policy(CPolicy::Function {
                symbol: symbol.clone(),
            });
            requests.output(
                DeclaredElement::new(ElementKind::Function, ident.to_string(), symbol, "function"),
                policy,
            );
        }

        // Ignores are decisions, accounted apart from the gaps.
        for ident in sorted(&self.ignored_functions) {
            requests.ignored.push(
                DeclaredElement::new(
                    ElementKind::Function,
                    ident.to_string(),
                    String::new(),
                    "ignore_function",
                )
                .local(),
            );
        }
        for key in sorted(&self.ignored_types) {
            requests.ignored.push(
                DeclaredElement::new(
                    ElementKind::Type,
                    key.as_str(),
                    String::new(),
                    "ignore_type",
                )
                .local(),
            );
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
    let args: Vec<String> = key.iter().map(|k| close_up(k.as_str())).collect();
    format!("impl Fn({})", args.join(", "))
}

/// A canonical key with the spaces a reader does not want: `Option < Grade >`
/// becomes `Option<Grade>` and `& [Payload]` becomes `&[Payload]`, while the
/// space in `dyn Error` — the only kind that separates two words — stays.
fn close_up(key: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let characters: Vec<char> = key.chars().collect();
    let mut out = String::with_capacity(key.len());
    for (index, &character) in characters.iter().enumerate() {
        if character == ' ' {
            let before = index.checked_sub(1).map(|i| characters[i]);
            let after = characters.get(index + 1).copied();
            let separates_words = before.is_some_and(word) && after.is_some_and(word);
            if !separates_words {
                continue;
            }
        }
        out.push(character);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn punctuation_closes_up_and_words_stay_apart() {
        for (key, expected) in [
            ("Option < Grade >", "Option<Grade>"),
            ("& [Payload]", "&[Payload]"),
            ("dyn Error", "dyn Error"),
            (
                "Result < Box < dyn Error > , u8 >",
                "Result<Box<dyn Error>,u8>",
            ),
        ] {
            assert_eq!(super::close_up(key), expected, "{key}");
        }
    }
}
