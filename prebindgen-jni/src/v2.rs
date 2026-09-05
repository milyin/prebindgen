//! The JNI binding's declarations, as the v2 engine reads them.
//!
//! [`Declarations`] keeps accumulating exactly as it does for v1 — same
//! `package!`/`ptr_class!`/`fun!` surface, same `set_*` settings, same
//! name-mangle closures — and this states that storage to the other engine.
//! The engine borrows it live: nothing is copied into a request object, so a
//! closure cannot be lost on the way across and a new capability grows a reader
//! here rather than a second encoding of the declarations.
//!
//! Kotlin names and packages come from this adapter, not from the engine: where
//! a declared element lands is the frontend's settings applied to it, which is
//! the same answer v1 would give.

use prebindgen_registry_v2::{BindingDeclarations, DeclaredElement, ElementKind};

use crate::jni::{ClassMember, Declarations, FunctionEntry};

impl BindingDeclarations for Declarations {
    fn target(&self) -> &'static str {
        "jni"
    }

    fn declared_elements(&self) -> Vec<DeclaredElement> {
        let mut out = Vec::new();

        // Declared classes. Which declarator a type came from is this
        // adapter's word for its representation, and the engine prints it back
        // rather than re-deriving it.
        for (key, config) in &self.types {
            let placement = self.kotlin_fqn(key).unwrap_or_default();
            // A declared class need not name a captured item: a target may
            // represent `String` or `Vec<u8>` without the source exporting one.
            out.push(
                DeclaredElement::new(
                    ElementKind::Type,
                    key.as_str(),
                    placement.clone(),
                    declarator(&config.kind),
                )
                .local(),
            );

            // Members are separately selected: a class can be emitted with one
            // of its methods skipped, so each is an element of its own.
            for member in self.class_members.get(key).into_iter().flatten() {
                out.push(DeclaredElement::new(
                    ElementKind::Function,
                    member.rust_ident.to_string(),
                    format!("{placement}.{}", self.effective_method_name(key, member)),
                    member_representation(member),
                ));
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
                out.push(DeclaredElement::new(
                    ElementKind::Function,
                    entry.rust_ident.to_string(),
                    placed(entry),
                    "fun",
                ));
            }
            // A `constant!(X)` names the `#[prebindgen]` const it reads.
            for entry in &config.constants {
                out.push(DeclaredElement::new(
                    ElementKind::Const,
                    entry.rust_ident.to_string(),
                    placed(entry),
                    "constant",
                ));
            }
            // A `constant!(X).fun(..)` names the nullary function behind it.
            for entry in &config.constant_functions {
                out.push(DeclaredElement::new(
                    ElementKind::Const,
                    entry.rust_ident.to_string(),
                    placed(entry),
                    "constant_fun",
                ));
            }
            // A `constant!(X).expr(..)` has no Rust item behind it at all.
            for decl in &config.constant_exprs {
                out.push(
                    DeclaredElement::new(
                        ElementKind::Const,
                        decl.kotlin_name.clone(),
                        format!("{package}.{}", decl.kotlin_name),
                        "constant_expr",
                    )
                    .local(),
                );
            }
        }

        // Declared conversions and binding-local helper functions: defined by
        // the binding rather than selected out of the source.
        for decl in &self.convert_decls {
            out.push(
                DeclaredElement::new(
                    ElementKind::Conversion,
                    decl.key().as_str(),
                    self.kotlin_fqn(decl.key()).unwrap_or_default(),
                    "convert",
                )
                .local(),
            );
        }
        for (ident, path, _) in &self.local_fns {
            out.push(
                DeclaredElement::new(
                    ElementKind::Function,
                    ident.to_string(),
                    quote::quote!(#path).to_string().replace(' ', ""),
                    "local_fn",
                )
                .local(),
            );
        }

        out
    }

    fn ignored_elements(&self) -> Vec<DeclaredElement> {
        let mut out = Vec::new();
        for ident in &self.ignored_fns {
            out.push(
                DeclaredElement::new(
                    ElementKind::Function,
                    ident.to_string(),
                    String::new(),
                    "ignore",
                )
                .local(),
            );
        }
        for key in &self.ignored_class_types {
            out.push(
                DeclaredElement::new(ElementKind::Type, key.as_str(), String::new(), "ignore")
                    .local(),
            );
        }
        for ident in &self.ignored_const_idents {
            out.push(
                DeclaredElement::new(
                    ElementKind::Const,
                    ident.to_string(),
                    String::new(),
                    "ignore_const",
                )
                .local(),
            );
        }
        out
    }
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
