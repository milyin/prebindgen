//! Kotlin/JVM identifier mangling + the whole-artifact symbol-validation
//! pass (issue #89).
//!
//! Distinct from the sibling [`symbol`](super::symbol) module: that one
//! escapes **native `Java_…` export symbols** (the JNI ABI charset); this one
//! sanitizes and validates **Kotlin source identifiers** (class / function /
//! member / field / package names) and builds the per-package and
//! native-symbol collision tables.
//!
//! ## The "default mangler" design
//!
//! [`mangle_kotlin_ident`] is the one deterministic sanitizer that turns any
//! string into a valid Kotlin identifier. Every DEFAULT (Rust-derived) name
//! flows through it — the seven name-mangle hooks default to it (see
//! `builder.rs`) and the four non-hook derived-name sites call it directly —
//! so the emitter always produces valid names on the default path. The only
//! ways an invalid name can survive to [`validate_symbols`] are an explicit
//! `.name()` override or a custom mangle hook; both are author input, so an
//! invalid one is a hard error the author can correct in build.rs.

use super::*;

/// The whole-artifact Kotlin-identifier + top-level-name validation pass
/// (issue #89), called from [`validate_bindings`]. Returns the collected
/// errors (joined into the same message the native-symbol table produces) and
/// emits `cargo:warning=` lines directly for names the default mangler had to
/// change. Runs on the resolved registry before any file is written.
///
/// * **Invalid-name errors** — a final Kotlin name that is still not a legal
///   identifier can only come from a `.name()` override or a custom mangle
///   hook (the default path is mangled → always valid), so it is an author
///   mistake, easy to correct in build.rs.
/// * **Top-level collisions** — two class / interface / harness / const-`val`
///   names colliding in one package, including a collision the mangler
///   created.
/// * **Warnings** — where the default mangler sanitized a Rust-derived name.
pub(crate) fn validate_symbols(ext: &JniGen, registry: &Registry<KotlinMeta>) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    // (package, name) → origin, for top-level-unique Kotlin declarations.
    let mut top_level: BTreeMap<(String, String), String> = BTreeMap::new();

    let mut add_top_level =
        |package: &str, name: &str, origin: String, errors: &mut Vec<String>| {
            if let Some(prev) =
                top_level.insert((package.to_string(), name.to_string()), origin.clone())
            {
                errors.push(format!(
                    "duplicate top-level Kotlin name `{name}` in package `{package}`: \
                     declared by both {prev} and {origin}",
                ));
            }
        };

    // Classes (ptr / data / value / enum), in deterministic key order.
    let mut class_keys: Vec<&TypeKey> = ext
        .types
        .iter()
        .filter(|(_, cfg)| cfg.name_spec.is_some())
        .map(|(k, _)| k)
        .collect();
    class_keys.sort_by_key(|k| k.as_str().to_string());
    for key in class_keys {
        let cfg = &ext.types[key];
        let spec = cfg.name_spec.as_ref().expect("filtered to Some");
        let fqn = ext.fqn_of(spec);
        let (package, short) = fqn.rsplit_once('.').unwrap_or(("", fqn.as_str()));
        let origin = format!("class `{key}`");
        check_ident(short, &origin, &mut errors);
        add_top_level(package, short, origin.clone(), &mut errors);
        if cfg.interface_enabled {
            let iface = ext.interface_short_name_unchecked(
                package,
                short,
                cfg.interface_name_override.as_deref(),
            );
            let iorigin = format!("interface of class `{key}`");
            check_ident(&iface, &iorigin, &mut errors);
            add_top_level(package, &iface, iorigin, &mut errors);
        }
    }

    // The central JNINative harness object — one per base package.
    let harness = ext.jni_native_class_name();
    check_ident(&harness, "the `JNINative` harness object", &mut errors);
    add_top_level(
        &ext.package,
        &harness,
        "the `JNINative` harness object".to_string(),
        &mut errors,
    );

    // Declared package-level functions + their const `val`s, per subpackage.
    let mut subpackages: Vec<&String> = ext.packages.keys().collect();
    subpackages.sort();
    for sub in subpackages {
        let pkg_cfg = &ext.packages[sub];
        let package = ext.package_name(sub);
        for entry in &pkg_cfg.functions {
            let name = ext.effective_function_name(sub, entry);
            check_ident(
                &name,
                &format!("function `{}`", entry.rust_ident),
                &mut errors,
            );
            // Functions may overload — not added to the uniqueness table
            // (overload-signature collisions are Stage 2).
        }
        // Const `val`s ARE top-level-unique (a property, not an overloadable fn).
        for entry in pkg_cfg
            .constants
            .iter()
            .chain(pkg_cfg.constant_functions.iter())
        {
            let name = entry
                .kotlin_name_override
                .clone()
                .unwrap_or_else(|| mangle_kotlin_ident(&entry.rust_ident.to_string()));
            let origin = format!("const `{}`", entry.rust_ident);
            check_ident(&name, &origin, &mut errors);
            add_top_level(&package, &name, origin, &mut errors);
        }
        for decl in &pkg_cfg.constant_exprs {
            let origin = format!("expression constant `{}`", decl.kotlin_name);
            check_ident(&decl.kotlin_name, &origin, &mut errors);
            add_top_level(&package, &decl.kotlin_name, origin, &mut errors);
        }
    }

    // Class members (instance methods / companion factories) — validity only;
    // same-named members overload, and cross-member signature collisions are
    // Stage 2.
    let mut member_keys: Vec<&TypeKey> = ext.class_members.keys().collect();
    member_keys.sort_by_key(|k| k.as_str().to_string());
    for key in member_keys {
        for m in &ext.class_members[key] {
            let name = ext.effective_method_name(key, m);
            check_ident(&name, &format!("method `{}`", m.rust_ident), &mut errors);
        }
    }

    // Warnings: Rust-derived names the DEFAULT mangler had to sanitize (a
    // struct field / enum variant named like a Kotlin keyword). Author
    // `.name()` / custom-hook names are covered by the errors above.
    warn_derived_name_changes(ext, registry);

    errors
}

/// Error when `name` is not a legal Kotlin identifier — reachable only from a
/// `.name()` override or a custom mangle hook (see [`validate_symbols`]).
fn check_ident(name: &str, origin: &str, errors: &mut Vec<String>) {
    if !is_valid_kotlin_ident(name) {
        errors.push(format!(
            "`{name}` ({origin}) is not a valid Kotlin identifier — fix the `.name(...)` \
             override or the name mangle hook that produced it",
        ));
    }
}

/// Emit a `cargo:warning` for each Rust struct field (data-class property) or
/// enum variant whose Kotlin name the default mangler had to change.
fn warn_derived_name_changes(ext: &JniGen, registry: &Registry<KotlinMeta>) {
    let warn = |raw: &str, mangled: &str, what: &str, owner: &str| {
        if raw != mangled {
            println!(
                "cargo:warning=prebindgen: {what} `{raw}` of `{owner}` emitted as `{mangled}` \
                 (invalid Kotlin identifier sanitized)"
            );
        }
    };
    let mut class_keys: Vec<&TypeKey> = ext
        .types
        .iter()
        .filter(|(_, cfg)| cfg.name_spec.is_some())
        .map(|(k, _)| k)
        .collect();
    class_keys.sort_by_key(|k| k.as_str().to_string());
    for key in class_keys {
        let ident = match bare_path_ident(&key.to_type()) {
            Some(i) => i,
            None => continue,
        };
        if let Some((s, _)) = registry.structs.get(&ident) {
            for f in &s.fields {
                if let Some(fname) = &f.ident {
                    let camel = kt_snake_to_camel(&fname.to_string());
                    warn(
                        &camel,
                        &mangle_kotlin_ident(&camel),
                        "field",
                        &ident.to_string(),
                    );
                }
            }
        }
        if let Some((e, _)) = registry.enums.get(&ident) {
            for v in &e.variants {
                let screaming =
                    crate::api::lang::jnigen::util::camel_to_screaming_snake(&v.ident.to_string());
                warn(
                    &screaming,
                    &mangle_kotlin_ident(&screaming),
                    "enum variant",
                    &ident.to_string(),
                );
            }
        }
    }
}

/// Kotlin **hard keywords** — reserved words that cannot be used as an
/// identifier even back-ticked. The single source of truth: [`kt_param_name`]
/// (params) and [`mangle_kotlin_ident`] / [`is_valid_kotlin_ident`] (every
/// other position) all consult this list.
pub(crate) const HARD_KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

/// True when `s` is a legal, non-keyword Kotlin identifier: non-empty, first
/// char a Unicode letter or `_`, the rest Unicode letters / digits / `_`, and
/// not a [hard keyword](HARD_KEYWORDS). (Kotlin also permits back-ticked
/// identifiers with arbitrary content, but those can't be native-symbol
/// components and don't round-trip everywhere, so the generator does not emit
/// them.)
pub(crate) fn is_valid_kotlin_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_alphabetic()) {
        return false;
    }
    if !chars.all(|c| c == '_' || c.is_alphanumeric()) {
        return false;
    }
    !HARD_KEYWORDS.contains(&s)
}

/// Deterministically sanitize `s` into a valid Kotlin identifier
/// ([`is_valid_kotlin_ident`] holds on the result), idempotently:
///
/// * a hard keyword gets a trailing `_` (`object` → `object_`);
/// * every char that isn't a Kotlin identifier char (Unicode letter / digit /
///   `_`) becomes `_` (`my-name` → `my_name`);
/// * a leading digit gets a `_` prefix (`1x` → `_1x`);
/// * an empty string becomes `_`.
///
/// This is the one primitive the whole "default mangler" design rests on.
pub(crate) fn mangle_kotlin_ident(s: &str) -> String {
    if is_valid_kotlin_ident(s) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 1);
    for (i, c) in s.chars().enumerate() {
        if c == '_' || c.is_alphanumeric() {
            if i == 0 && c.is_numeric() {
                out.push('_');
            }
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    if HARD_KEYWORDS.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

/// Sanitize a dot-separated Kotlin package path by [mangling](mangle_kotlin_ident)
/// each non-empty segment (`fun.my-pkg` → `fun_.my_pkg`). Empty segments are
/// dropped (a leading/trailing/double dot). Idempotent.
pub(crate) fn mangle_package(path: &str) -> String {
    path.split('.')
        .filter(|s| !s.is_empty())
        .map(mangle_kotlin_ident)
        .collect::<Vec<_>>()
        .join(".")
}

/// A native `Java_…` export symbol — charset-guaranteed valid by
/// [`symbol::native_symbol`](super::symbol::native_symbol). A newtype so the
/// collision table's key type documents itself and can't be confused with a
/// Kotlin name string. (The typed Kotlin-side carriers `KotlinIdent` /
/// `KotlinFqn` / `JvmSignature` arrive in Stage 2, where the JVM-erasure
/// overload model needs them.)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct NativeSymbol(String);

impl NativeSymbol {
    pub fn new(sym: impl Into<String>) -> Self {
        NativeSymbol(sym.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid_kotlin_ident, mangle_kotlin_ident, mangle_package};

    #[test]
    fn validity_predicate() {
        assert!(is_valid_kotlin_ident("foo"));
        assert!(is_valid_kotlin_ident("_foo"));
        assert!(is_valid_kotlin_ident("fooBar1"));
        assert!(is_valid_kotlin_ident("Δelta")); // Unicode letter
        assert!(!is_valid_kotlin_ident("")); // empty
        assert!(!is_valid_kotlin_ident("1x")); // leading digit
        assert!(!is_valid_kotlin_ident("my name")); // space
        assert!(!is_valid_kotlin_ident("my-name")); // dash
        assert!(!is_valid_kotlin_ident("object")); // hard keyword
        assert!(!is_valid_kotlin_ident("when")); // hard keyword
    }

    #[test]
    fn mangling() {
        // Valid names pass through unchanged (byte-identity depends on this).
        assert_eq!(mangle_kotlin_ident("fooBar"), "fooBar");
        assert_eq!(mangle_kotlin_ident("_x"), "_x");
        // Keyword → trailing underscore.
        assert_eq!(mangle_kotlin_ident("object"), "object_");
        assert_eq!(mangle_kotlin_ident("when"), "when_");
        // Leading digit → prefix underscore.
        assert_eq!(mangle_kotlin_ident("1x"), "_1x");
        // Illegal char → underscore.
        assert_eq!(mangle_kotlin_ident("my-name"), "my_name");
        assert_eq!(mangle_kotlin_ident("a b"), "a_b");
        // Empty → placeholder.
        assert_eq!(mangle_kotlin_ident(""), "_");
    }

    #[test]
    fn mangling_is_idempotent() {
        for s in ["object", "1x", "my-name", "when", "", "a b", "fooBar"] {
            let once = mangle_kotlin_ident(s);
            assert_eq!(mangle_kotlin_ident(&once), once, "not idempotent for {s:?}");
            assert!(
                is_valid_kotlin_ident(&once),
                "mangle produced invalid: {once:?}"
            );
        }
    }

    #[test]
    fn package_mangles_each_segment() {
        assert_eq!(mangle_package("io.zenoh.jni"), "io.zenoh.jni");
        assert_eq!(mangle_package("fun.my-pkg"), "fun_.my_pkg");
        assert_eq!(mangle_package("a..b"), "a.b"); // empty segments dropped
        assert_eq!(mangle_package(""), "");
    }
}
