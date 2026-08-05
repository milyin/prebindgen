//! Converter/symbol naming, path qualification of emitted types, and
//! small `syn` type probes.

use super::*;

/// Last-segment ident of a `TypeKey` — e.g. `"Publisher<'static>"` →
/// `"Publisher"`, `"AdvancedSubscriber<()>"` → `"AdvancedSubscriber"`. Used by
/// the structured builders ([`Declarations::ptr_class`],
/// [`Declarations::data_class`]) to derive a default Kotlin class name from
/// the Rust type-key. Panics for non-path types (e.g. closures, references) —
/// the per-kind `*_name_mangle` closures see only path-shaped
/// shorts. For verbatim Kotlin expressions on non-path types, use a
/// scalar / generic type wrapper.
pub(crate) fn rust_short_name(key: &TypeKey) -> String {
    rust_short_name_opt(key).unwrap_or_else(|| {
        panic!(
            "rust_short_name: cannot derive Kotlin name from type-key `{}` — \
             only path-shaped types are supported here",
            key.as_str()
        )
    })
}

/// Fallible variant of [`rust_short_name`] — returns `None` for
/// non-path types instead of panicking. Used by
/// [`Declarations::note_wrapper_registration`] which is called for rank-0
/// wrapper patterns including non-path shapes like `()` where there
/// is no Kotlin short name to derive.
pub(crate) fn rust_short_name_opt(key: &TypeKey) -> Option<String> {
    key.short_name()
}

/// `VisitMut` that prefixes every bare single-segment `Type::Path` whose
/// ident is a key of `source_names` with that name's origin module. Walks
/// the full AST — function signatures, generic args, type ascriptions,
/// casts, turbofish — so any emitted item passes through one universal pass
/// instead of each emit site having to remember to qualify.
pub(crate) struct QualifyEmittedTypes<'a> {
    /// Bare type name → the module it is reachable under (the item's origin
    /// crate, or the registry's default module).
    pub(crate) source_names: &'a std::collections::HashMap<String, syn::Path>,
    /// Every indexed source name (consts, structs, enums) → the same. Applied
    /// ONLY inside an array length — see [`QualifyLengthPaths`].
    pub(crate) length_names: &'a std::collections::HashMap<String, syn::Path>,
}

impl syn::visit_mut::VisitMut for QualifyEmittedTypes<'_> {
    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        if tp.qself.is_none() && tp.path.leading_colon.is_none() && tp.path.segments.len() == 1 {
            let ident = tp.path.segments[0].ident.to_string();
            if let Some(module) = self.source_names.get(&ident) {
                let mut qualified = module.clone();
                qualified.segments.push(tp.path.segments[0].clone());
                tp.path = qualified;
            }
        }
        syn::visit_mut::visit_type_path_mut(self, tp);
    }

    /// Qualify a `#[prebindgen]` const used as an ARRAY LENGTH (`[u8; MAX]`).
    ///
    /// `syn` models the length as an expression, so `visit_type_path_mut` never
    /// sees it and the generated file would reference a const that is not in
    /// scope. The rewrite is confined to `arr.len` on purpose: a generated
    /// converter body is full of expression paths that are LOCALS (`v`, `env`),
    /// and a source crate may legally declare `pub const env: usize` — so a
    /// whole-item expression pass would rewrite those locals to
    /// `mycrate::env` even when restricted to registered const idents. An array
    /// length cannot contain a local, which is what makes this scope safe.
    fn visit_type_array_mut(&mut self, arr: &mut syn::TypeArray) {
        syn::visit_mut::visit_type_mut(self, &mut arr.elem);
        reject_unsupported_array_length(arr);
        let mut lengths = QualifyLengthPaths {
            length_names: self.length_names,
        };
        syn::visit_mut::visit_expr_mut(&mut lengths, &mut arr.len);
    }
}

/// Refuse an array length built from anything but a small, closed set of
/// expression forms.
///
/// [`QualifyLengthPaths`] rewrites a bare path to its origin module, which is
/// sound only while every path in the length names a source ITEM. Anything that
/// can bind a name breaks that premise — a local shadowing a source item gets
/// rewritten into it:
///
/// ```ignore
/// [u8; const { let array_len = 3; array_len }]   // `array_len` is a LOCAL
/// [u8; match 3 { array_len => array_len }]       // ...so is this one
/// ```
///
/// Qualifying either yields `myflat::array_len`, a function item where a
/// `usize` was meant; a same-typed collision would compile and silently change
/// the length.
///
/// This is a WHITELIST on purpose. Listing the binding forms instead means
/// every omitted or newly added `syn::Expr` variant silently reopens the hole —
/// which is exactly how `match` and `if let` slipped past the first attempt.
/// Inverting it makes the failure mode "a legitimate length is refused", which
/// is loud and trivially worked around by hoisting the value into a named
/// `const`.
///
/// # The whitelist is the model's
///
/// It listed eight forms — adding `Binary`, `Unary`, `Paren`, `Group`, `Cast`
/// and `Call` "const arithmetic over those" — and six of them could never
/// arrive. [`lower_array_len`](prebindgen::core::flat) accepts an integer
/// literal or a **bare single-segment name of a marked const**, and nothing
/// else: `[u8; A + 1]`, `[u8; A as usize]` and `[u8; array_len()]` are all
/// `ArrayLenReason::NotLiteralOrName`, so the type never becomes a
/// `TypeKind::Array` and never reaches an emitter at all
/// (`an_extent_is_a_literal_or_a_marked_const_and_nothing_else` pins it).
///
/// So the two forms below are the two the language accepts, restated where the
/// qualifier needs them. Narrowing a whitelist to what upstream already
/// enforces cannot open a hole — it closes the gap between the two lists, which
/// is the only way they could have disagreed.
fn reject_unsupported_array_length(arr: &mut syn::TypeArray) {
    // Rendered before the mutable walk below borrows the length.
    let rendered = quote::ToTokens::to_token_stream(&*arr).to_string();
    struct Check(Option<&'static str>);
    // `VisitMut` rather than `Visit`: syn's immutable visitor is behind a
    // feature this crate does not enable, and the walk mutates nothing.
    impl syn::visit_mut::VisitMut for Check {
        fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
            // The two forms the language accepts — see the note above.
            let ok = matches!(
                e,
                // A literal length, `[u8; 4]`.
                syn::Expr::Lit(_)
                    // The name this pass exists to qualify: a marked const.
                    | syn::Expr::Path(_)
            );
            if !ok && self.0.is_none() {
                self.0 = Some("an unsupported expression form");
            }
            syn::visit_mut::visit_expr_mut(self, e);
        }
    }
    let mut check = Check(None);
    syn::visit_mut::VisitMut::visit_expr_mut(&mut check, &mut arr.len);
    if let Some(what) = check.0 {
        panic!(
            "fixed-size array `{rendered}`: the length uses {what}. Only a literal and the name \
             of a `#[prebindgen]` const are supported — anything that can bind a name \
             (`const {{ … }}`, `match`, `if let`, a closure, a loop) would let a LOCAL be \
             mistaken for a source item, because this generator qualifies the length's paths \
             against their source module. Hoist the value into a named `const` and use that as \
             the length."
        );
    }
}

/// Qualifies the source-crate paths in an array's LENGTH expression, run only
/// by [`QualifyEmittedTypes::visit_type_array_mut`]. Separate from the type
/// visitor so it can never reach a converter body's locals.
///
/// A length is an ordinary Rust const expression, so it reaches the source
/// crate two ways and both need the origin module prefixed:
///
/// * a **free const**, `[u8; MAX]` — the whole path is the name;
/// * an **associated const**, `[u8; Holder::N]` — the LEADING segment is the
///   owning type. Only that segment is rewritten; the rest (`::N`, and any
///   further associated item) is relative to it and must be left alone.
///
/// Both look up the same registry-wide map, so an owner that exists only as a
/// compile-time namespace does not have to be declared to the binding: forcing
/// that would emit a dead Kotlin class purely to make the generated Rust
/// compile.
struct QualifyLengthPaths<'a> {
    length_names: &'a std::collections::HashMap<String, syn::Path>,
}

impl syn::visit_mut::VisitMut for QualifyLengthPaths<'_> {
    fn visit_expr_path_mut(&mut self, ep: &mut syn::ExprPath) {
        if ep.qself.is_none() && ep.path.leading_colon.is_none() {
            // One segment names the const itself; more than one means the
            // leading segment is the type that owns it. Either way it is the
            // leading segment that carries the origin module.
            let ident = ep.path.segments[0].ident.to_string();
            if let Some(module) = self.length_names.get(&ident) {
                let mut qualified = module.clone();
                qualified.segments.extend(ep.path.segments.iter().cloned());
                ep.path = qualified;
            }
        }
        syn::visit_mut::visit_expr_path_mut(self, ep);
    }
}

/// If `ty` is `JObject` / `JString` / `JByteArray` (no explicit angle args),
/// splice in `<'<life>>`. Otherwise return `ty` unchanged.
pub(crate) fn annotate_jobject_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if crate::jni::wire_access::is_jni_reference_wire(ty)
                && matches!(last.arguments, syn::PathArguments::None)
            {
                let mut new = tp.clone();
                if let Some(last) = new.path.segments.last_mut() {
                    let lt =
                        syn::Lifetime::new(&format!("'{}", life), proc_macro2::Span::call_site());
                    last.arguments =
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: syn::token::Lt::default(),
                            args: syn::punctuated::Punctuated::from_iter(std::iter::once(
                                syn::GenericArgument::Lifetime(lt),
                            )),
                            gt_token: syn::token::Gt::default(),
                        });
                }
                return syn::Type::Path(new);
            }
        }
    }
    ty.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

// `pat_match` lived here — `ty.to_token_stream().to_string() == pat` — and was
// how the converter selector decided what a type WAS: rebuild a wildcard
// pattern from the spelling, render it to a string, compare. That made the
// answer depend on how Rust happened to spell the type, so `Box<Option<T>>`
// reconstructed as `Box<_>`, matched nothing, and got no converter at all
// (#270). Dispatch reads `TypeKind` now; nothing needs it.

/// INPUT: wire → rust. Format `<wire_id>_to_<rust_id>_<hash>` (including
/// `impl Fn(...)` lambda converters — the legacy
/// `process_kotlin_<Name>_callback` naming is gone with the fun-interface
/// subsystem).
pub(crate) fn input_name(rust: &TokenStream, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", wire_id, rust_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

/// OUTPUT: rust → wire. Format `<rust_id>_to_<wire_id>_<hash>`.
pub(crate) fn output_name(rust: &TokenStream, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", rust_id, wire_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

pub(crate) fn sanitize_for_ident(s: &str) -> String {
    // Special-case the empty tuple — the all-punctuation token stream
    // would sanitize to a meaningless fallback. `unit` is recognisable.
    if s.trim() == "()" {
        return "unit".to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("ty");
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn wire_short(wire: &syn::Type) -> String {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return sanitize_for_ident(&last.ident.to_string());
        }
    }
    sanitize_for_ident(&wire.to_token_stream().to_string())
}

pub(crate) fn hash_pair(rust: &TokenStream, wire: &syn::Type) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let mut h = DefaultHasher::new();
    rust.to_string().hash(&mut h);
    "::".hash(&mut h);
    wire.to_token_stream().to_string().hash(&mut h);
    h.finish()
}
