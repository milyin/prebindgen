//! The registry-owned key for captured Rust syntax.

use std::ops::Deref;

/// The capability handed to Rust-emission callbacks.
///
/// The flat crate owns the rendering protocol because it owns the syntax. The
/// registry owns this concrete key and its private constructor, so ordinary
/// registry users receive an `Emit` only during final `write_rust` callbacks.
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
#[derive(Debug)]
pub struct Emit(());

impl Emit {
    pub(crate) fn new() -> Self {
        Self(())
    }

    /// Construct an emission key for an out-of-crate adapter test.
    ///
    /// This is absent from normal builds. Production code receives the key
    /// only in an emission callback.
    #[cfg(any(test, feature = "testing"))]
    pub fn for_test() -> Self {
        Self::new()
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

impl prebindgen_flat::RustEmitter for Emit {}

// Dereferencing to the protocol object keeps call sites ergonomic
// (`emit.spell(ty)`) without copying its method surface into this crate.
impl Deref for Emit {
    type Target = dyn prebindgen_flat::RustEmitter;

    fn deref(&self) -> &Self::Target {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{ident_component, Emit};
    use crate::{ArtifactId, Direction, OperationId};

    #[test]
    fn operation_symbols_are_stable_and_writer_scoped() {
        let operation = OperationId::shared(
            ArtifactId::new("test-codec", "owned").unwrap(),
            Direction::Construct,
        );
        let emit = Emit::for_test();

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
