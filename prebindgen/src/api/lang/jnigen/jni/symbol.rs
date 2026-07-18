//! The one spec-compliant JNI native-symbol encoder (#86).
//!
//! Every exported `Java_…` symbol — function wrappers, const getters, typed
//! handle destructors, vec build helpers — is assembled here; no emitter
//! concatenates symbol strings ad hoc. The encoding implements "Resolving
//! Native Method Names" of the official JNI specification:
//! <https://docs.oracle.com/en/java/javase/21/docs/specs/jni/design.html#resolving-native-method-names>
//!
//! A JVM computes the expected export name from the Java/Kotlin declaration
//! with that algorithm; a symbol assembled any other way (e.g. plain
//! dot-to-underscore replacement) will not resolve whenever a package
//! segment, class name, or method name contains an underscore or a
//! non-ASCII-alphanumeric character. Kotlin/JVM names are **not** touched —
//! the encoder replicates the JVM's derivation on the Rust side only.

/// JNI-escape one symbol component (a package segment, class name, method
/// name, or argument-signature fragment) per the spec table
/// (<https://docs.oracle.com/en/java/javase/21/docs/specs/jni/design.html#resolving-native-method-names>):
///
/// | source              | encoded  |
/// |---------------------|----------|
/// | `/` (separator)     | `_`      |
/// | `_`                 | `_1`     |
/// | `;` (in signatures) | `_2`     |
/// | `[` (in signatures) | `_3`     |
/// | other non-`[A-Za-z0-9]` | `_0xxxx` (lowercase hex, per UTF-16 code unit) |
///
/// ASCII-alphanumeric input passes through verbatim, so escape-free names
/// produce exactly the symbols the pre-#86 concatenation produced.
fn escape(component: &str) -> String {
    let mut out = String::with_capacity(component.len());
    for c in component.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' => out.push(c),
            '/' => out.push('_'),
            '_' => out.push_str("_1"),
            ';' => out.push_str("_2"),
            '[' => out.push_str("_3"),
            other => {
                let mut units = [0u16; 2];
                for unit in other.encode_utf16(&mut units) {
                    out.push_str(&format!("_0{unit:04x}"));
                }
            }
        }
    }
    out
}

/// The short native symbol `Java_<pkg…>_<class>_<method>`. `package` is the
/// dot-separated Java package (empty ⇒ the package segments are omitted);
/// every component is escaped by [`escape`]. This is the name the JVM looks
/// up first for a non-overloaded `native` method.
pub(crate) fn native_symbol(package: &str, class: &str, method: &str) -> String {
    let mut out = String::from("Java");
    if !package.is_empty() {
        for segment in package.split('.') {
            out.push('_');
            out.push_str(&escape(segment));
        }
    }
    out.push('_');
    out.push_str(&escape(class));
    out.push('_');
    out.push_str(&escape(method));
    out
}

/// The long native symbol for **overloaded** natives: the short name plus
/// `__` and the escaped argument signature (the descriptor between `(` and
/// `)`, e.g. `ILjava/lang/String;` — `/`→`_`, `;`→`_2`, `[`→`_3`). Nothing
/// JniGen emits today is overloaded at the extern level (every `JNINative`
/// method is uniquely named), so this is provided-but-unwired per #86's
/// direction: if overloaded natives are ever emitted, they must come from
/// this same abstraction.
#[allow(dead_code)]
pub(crate) fn native_symbol_overloaded(
    package: &str,
    class: &str,
    method: &str,
    arg_sig: &str,
) -> String {
    format!(
        "{}__{}",
        native_symbol(package, class, method),
        escape(arg_sig)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Golden values produced by `javac -h` (javac 21.0.5) for
    /// `package io.example.my_pkg; public class Native_Harness` — the
    /// acceptance fixture of #86. Underscores in every component escape to
    /// `_1`.
    #[test]
    fn javac_golden_underscores() {
        assert_eq!(
            native_symbol("io.example.my_pkg", "Native_Harness", "do_work"),
            "Java_io_example_my_1pkg_Native_1Harness_do_1work"
        );
    }

    /// `javac -h` golden: non-ASCII (`café`, é = U+00E9) escapes to
    /// `_0xxxx` with four lowercase hex digits.
    #[test]
    fn javac_golden_unicode() {
        assert_eq!(
            native_symbol("io.example.my_pkg", "Native_Harness", "café"),
            "Java_io_example_my_1pkg_Native_1Harness_caf_000e9"
        );
    }

    /// `javac -h` goldens for the overloaded pair `g(int)` /
    /// `g(String, int[])`: long names carry the `__`-separated escaped
    /// argument signature (`;`→`_2`, `[`→`_3`, `/`→`_`).
    #[test]
    fn javac_golden_overloaded_long_names() {
        assert_eq!(
            native_symbol_overloaded("io.example.my_pkg", "Native_Harness", "g", "I"),
            "Java_io_example_my_1pkg_Native_1Harness_g__I"
        );
        assert_eq!(
            native_symbol_overloaded(
                "io.example.my_pkg",
                "Native_Harness",
                "g",
                "Ljava/lang/String;[I"
            ),
            "Java_io_example_my_1pkg_Native_1Harness_g__Ljava_lang_String_2_3I"
        );
    }

    /// Escape-free camelCase names (the entire pre-#86 surface) pass through
    /// verbatim — the encoder is identity-equivalent to the old
    /// concatenation, keeping every existing generated symbol byte-identical.
    #[test]
    fn identity_on_escape_free_names() {
        assert_eq!(
            native_symbol("io.prebindgen.covertest", "CovNative", "storageSummary"),
            "Java_io_prebindgen_covertest_CovNative_storageSummary"
        );
    }

    /// An empty package omits the package segments entirely.
    #[test]
    fn empty_package() {
        assert_eq!(
            native_symbol("", "Native_Harness", "do_work"),
            "Java_Native_1Harness_do_1work"
        );
    }

    /// A supplementary-plane character (outside the BMP) escapes per UTF-16
    /// code unit — two `_0xxxx` groups (the surrogate pair).
    #[test]
    fn supplementary_plane_escapes_per_utf16_unit() {
        // U+10400 (𐐀) = surrogate pair D801 DC00.
        assert_eq!(escape("𐐀"), "_0d801_0dc00");
    }
}
