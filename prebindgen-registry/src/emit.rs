//! Final, model-driven Rust generation.

use std::collections::HashMap;

/// The zero-sized capability proving final Rust emission has begun.
///
/// Flat owns the generate-only protocol; the registry owns this concrete key and
/// its private constructor. `RustWriter` holds it only during final file assembly.
///
/// ```compile_fail
/// use prebindgen_registry::Emit;
/// let emit = Emit::new();
/// ```
///
/// ```compile_fail
/// use prebindgen_registry::Emit;
/// let emit = Emit(());
/// ```
///
/// A registry-only adapter cannot name the flat rendering protocol through
/// the registry's model re-export:
///
/// ```compile_fail
/// use prebindgen_registry::flat::emit::RustEmitter;
/// ```
///
/// Even code which can name the token cannot recover retained spelling:
///
/// ```compile_fail
/// # use prebindgen_registry::{Emit, flat::TypeRef};
/// use prebindgen_flat::RustEmitter;
/// # fn leak(emit: &Emit, ty: &TypeRef) {
/// let _ = emit.spell(ty);
/// # }
/// ```
///
/// The stateful writer facade exposes no spelling escape either:
///
/// ```compile_fail
/// # use prebindgen_registry::{RustWriter, flat::TypeRef};
/// # fn leak(writer: &RustWriter, ty: &TypeRef) {
/// let _ = writer.spell(ty);
/// # }
/// ```
#[derive(Debug)]
pub struct Emit(());

impl prebindgen_flat::RustEmitter for Emit {}

/// Writer-owned context for final Rust emission.
///
/// Unlike [`Emit`], this is deliberately stateful: qualifying a modeled
/// nominal type requires the frozen registry mapping from Flat names to source
/// modules. Keeping that state here preserves the distinction between the
/// phase token and the writer which consumes model facts.
#[derive(Debug)]
pub struct RustWriter {
    emit: Emit,
    source_modules: HashMap<String, syn::Path>,
    default_module: syn::Path,
}

impl Emit {
    fn new() -> Self {
        Self(())
    }
}

impl RustWriter {
    pub(crate) fn new(registry: &crate::Registry, source_module: Option<&syn::Path>) -> Self {
        let default_module = source_module
            .cloned()
            .or_else(|| registry.default_module())
            .unwrap_or_else(|| syn::parse_quote!(crate));
        let source_modules = registry
            .named_item_idents()
            .map(|ident| {
                let module = registry
                    .origin_module(ident)
                    .unwrap_or_else(|| default_module.clone());
                (ident.to_string(), module)
            })
            .collect();
        Self {
            emit: Emit::new(),
            source_modules,
            default_module,
        }
    }

    /// Construct a final writer for an out-of-crate adapter test.
    ///
    /// This is absent from normal builds. Production code receives the writer
    /// only in an emission callback.
    #[cfg(any(test, feature = "testing"))]
    pub fn for_test() -> Self {
        Self {
            emit: Emit::new(),
            source_modules: HashMap::new(),
            default_module: syn::parse_quote!(crate),
        }
    }

    /// Construct the production writer context for a test that renders a
    /// frozen plan directly instead of going through `write_rust`.
    #[cfg(any(test, feature = "testing"))]
    pub fn for_registry_test(registry: &crate::Registry) -> Self {
        Self::new(registry, None)
    }

    /// Emit one source-type fragment from Flat model facts.
    ///
    /// Qualification is driven by Flat nominal/extent facts and the registry's
    /// declaration-module map. No token text is inspected to decide what the
    /// type means.
    pub fn emit_source_type(
        &self,
        ty: &prebindgen_flat::flat::TypeRef,
    ) -> proc_macro2::TokenStream {
        prebindgen_flat::RustEmitter::emit_source_type(
            &self.emit,
            ty,
            &self.source_modules,
            &self.default_module,
        )
    }

    /// Generate a public const alias to the source declaration.
    pub fn const_alias(
        &self,
        item: &prebindgen_flat::flat::Constant,
        source_module: &syn::Path,
    ) -> syn::ItemConst {
        let ident = &item.name;
        let ty = self.emit_source_type(&item.ty);
        let docs: Vec<syn::Attribute> = item
            .docs()
            .into_iter()
            .flat_map(|docs| {
                docs.lines()
                    .map(|line| {
                        let doc = format!(" {line}");
                        syn::parse_quote!(#[doc = #doc])
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        syn::parse_quote!(#(#docs)* pub const #ident: #ty = #source_module::#ident;)
    }

    /// Copy the proc-macro's anonymous feature guard into the final file.
    pub(crate) fn guard(&self, guard: &prebindgen_flat::flat::Guard) -> syn::ItemConst {
        prebindgen_flat::RustEmitter::guard(&self.emit, guard)
    }

    /// Emit a fieldless enum's modeled discriminant spelling.
    pub fn discriminant(
        &self,
        value: &prebindgen_flat::flat::EnumValue,
    ) -> Option<proc_macro2::TokenStream> {
        prebindgen_flat::RustEmitter::discriminant(&self.emit, value)
    }

    /// Generate a struct constructor or pattern with its modeled delimiter shape.
    pub fn shape_struct(
        &self,
        item: &prebindgen_flat::flat::Struct,
        head: proc_macro2::TokenStream,
        parts: &[proc_macro2::TokenStream],
    ) -> proc_macro2::TokenStream {
        prebindgen_flat::RustEmitter::shape_struct(&self.emit, item, head, parts)
    }

    /// Generate an enum-alternative constructor or pattern with its modeled shape.
    pub fn shape_alternative(
        &self,
        item: &prebindgen_flat::flat::Alternative,
        head: proc_macro2::TokenStream,
        parts: &[proc_macro2::TokenStream],
    ) -> proc_macro2::TokenStream {
        prebindgen_flat::RustEmitter::shape_alternative(&self.emit, item, head, parts)
    }

    /// Allocate the final private Rust symbol for a registry-owned operation.
    ///
    /// `namespace` is adapter vocabulary (for example `"jni"`), not a Rust
    /// type spelling. This method lives on the emission capability so neither
    /// the registry plan nor a language adapter can turn model identity into a
    /// Rust identifier before final file assembly. The emitted name keeps a
    /// bounded semantic stem for readability and a stable hash for uniqueness.
    pub fn operation_ident(
        &self,
        namespace: &str,
        operation: &crate::generation::OperationId,
    ) -> syn::Ident {
        let semantic = ident_component(&operation.semantic_label());
        let (direction, semantic) = match operation.direction() {
            crate::recipe::Direction::Construct => ("in", format!("wire_to_{semantic}")),
            crate::recipe::Direction::Deconstruct => ("out", format!("{semantic}_to_wire")),
        };
        let role = match operation.role() {
            crate::generation::OperationRole::Converter => "convert".to_owned(),
            crate::generation::OperationRole::Stage(index) => format!("stage_{index}"),
        };
        quote::format_ident!(
            "__{namespace}_{direction}_{role}_{semantic}_{:016x}",
            operation.stable_fingerprint()
        )
    }
}

/// Turn diagnostic model vocabulary into a readable, bounded identifier
/// component. This formats identity during final emission; it does not parse
/// the text or use it to make a generation decision.
fn ident_component(label: &str) -> String {
    const MAX_CHARS: usize = 96;
    if label.trim() == "()" {
        return "unit".to_owned();
    }
    let mut out = String::new();
    let mut separator = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            separator = false;
        } else if !out.is_empty() && !separator {
            out.push('_');
            separator = true;
        }
        if out.len() >= MAX_CHARS {
            break;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "operation".to_owned()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{ident_component, Emit, RustWriter};
    use crate::{ArtifactId, Direction, OperationId};

    #[test]
    fn emit_remains_only_a_zero_sized_phase_token() {
        assert_eq!(std::mem::size_of::<Emit>(), 0);
    }

    #[test]
    fn operation_symbols_are_stable_and_writer_scoped() {
        let operation = OperationId::shared(
            ArtifactId::new("test-codec", "owned").unwrap(),
            Direction::Construct,
        );
        let emit = RustWriter::for_test();

        let first = emit.operation_ident("test", &operation);
        let second = emit.operation_ident("test", &operation);

        assert_eq!(first, second);
        assert!(first
            .to_string()
            .starts_with("__test_in_convert_wire_to_test_codec_owned_"));
    }

    #[test]
    fn operation_symbol_stems_are_readable_and_bounded() {
        assert_eq!(
            ident_component("impl Fn(ZSample) + Send + Sync + 'static"),
            "impl_Fn_ZSample_Send_Sync_static"
        );
        assert_eq!(ident_component("()"), "unit");
        assert!(ident_component(&"word-".repeat(100)).len() <= 96);
    }
}
