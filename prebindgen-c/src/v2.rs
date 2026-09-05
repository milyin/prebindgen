//! The C binding's declarations, as the v2 engine reads them.
//!
//! [`CbindgenBuilder`] keeps accumulating declarations exactly as it does for
//! v1 — same builder, same modifiers, same manglers — and this states that
//! storage to the other engine. The engine borrows it live: nothing is copied
//! into a request object, so a closure cannot be lost on the way across and a
//! new capability grows a reader here rather than a second encoding of the
//! declarations.
//!
//! Names come from this adapter, not from the engine: what a declared element
//! is *called* in C is the frontend's manglers applied to it, which is the same
//! answer v1 would give.

use prebindgen_registry_v2::{BindingDeclarations, DeclaredElement, ElementKind};

use crate::CbindgenBuilder;

impl BindingDeclarations for CbindgenBuilder {
    fn target(&self) -> &'static str {
        "c"
    }

    fn declared_elements(&self) -> Vec<DeclaredElement> {
        let mut out = Vec::new();

        // Types, one group per declarator: which one a type came from is the
        // adapter's word for its representation, and the engine prints it back
        // rather than re-deriving it.
        for (keys, representation) in [
            (self.opaque.keys().collect::<Vec<_>>(), "opaque_ptr"),
            (self.data.keys().collect(), "data_struct"),
            (self.value_opaque.keys().collect(), "value_opaque"),
            (self.enums.keys().collect(), "enum_type"),
            (self.tagged_unions.keys().collect(), "tagged_union"),
        ] {
            for key in keys {
                // A declared type need not be a captured item: `String` crosses
                // as an opaque handle in perftest-c and the source never
                // exported it.
                out.push(
                    DeclaredElement::new(
                        ElementKind::Type,
                        key.as_str(),
                        self.c_type_name(key),
                        representation,
                    )
                    .local(),
                );
            }
        }

        // Callback signatures: no captured item names one, and its C closure
        // struct is what the target places.
        for key in self.callbacks.keys() {
            out.push(
                DeclaredElement::new(
                    ElementKind::Callback,
                    describe_callback(key),
                    self.callback_c_name(key),
                    "callback",
                )
                .local(),
            );
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        for decl in &self.convert_decls {
            out.push(
                DeclaredElement::new(
                    ElementKind::Conversion,
                    decl.key().as_str(),
                    self.c_type_name(decl.key()),
                    "convert",
                )
                .local(),
            );
        }

        // Exported functions — the one kind that must name a captured item.
        for ident in self.functions.keys() {
            out.push(DeclaredElement::new(
                ElementKind::Function,
                ident.to_string(),
                self.fn_symbol(ident).to_string(),
                "function",
            ));
        }

        out
    }

    fn ignored_elements(&self) -> Vec<DeclaredElement> {
        let mut out = Vec::new();
        for ident in &self.ignored_functions {
            out.push(
                DeclaredElement::new(
                    ElementKind::Function,
                    ident.to_string(),
                    String::new(),
                    "ignore_function",
                )
                .local(),
            );
        }
        for key in &self.ignored_types {
            out.push(
                DeclaredElement::new(
                    ElementKind::Type,
                    key.as_str(),
                    String::new(),
                    "ignore_type",
                )
                .local(),
            );
        }
        out
    }
}

/// A callback signature as its own name: `impl Fn(&Payload)`.
fn describe_callback(key: &[prebindgen_registry::TypeKey]) -> String {
    let args: Vec<&str> = key.iter().map(|k| k.as_str()).collect();
    format!("impl Fn({})", args.join(", "))
}
