//! The canonical identity of a type: its normalized token string.

use std::fmt;

use quote::ToTokens;

/// Canonical type-shape key: identity is the token string of the
/// **normalized** type ([`crate::api::core::flat::spelling::normalize_type`] —
/// group/paren unwrap, `crate::`/`self::` and std-prelude path reduction;
/// the complete equivalence rule set is documented there). The normalized
/// parsed form is kept alongside the string, so [`Self::to_type`] is an
/// infallible clone — no core invariant depends on serialize-then-reparse
/// round trips (issue #95).
#[derive(Clone)]
pub struct TypeKey {
    /// Canonical token string — the identity `Eq`/`Hash` compare.
    canon: std::rc::Rc<str>,
    /// The normalized parsed form the string was rendered from.
    ty: std::rc::Rc<syn::Type>,
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        self.canon == other.canon
    }
}
impl Eq for TypeKey {}
impl std::hash::Hash for TypeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.canon.hash(state)
    }
}
impl PartialOrd for TypeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TypeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canon.cmp(&other.canon)
    }
}
// Keep the historical single-field tuple rendering (`TypeKey("Vec < u8 >")`)
// — error text and test expectations format keys through it.
impl fmt::Debug for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypeKey").field(&&*self.canon).finish()
    }
}

/// Structured failure of [`TypeKey::parse`]: the offending input plus the
/// underlying `syn` parse error.
#[derive(Debug)]
pub struct TypeKeyParseError {
    pub input: String,
    pub error: syn::Error,
}

impl fmt::Display for TypeKeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid type `{}`: {}", self.input, self.error)
    }
}

impl std::error::Error for TypeKeyParseError {}

impl TypeKey {
    /// Build a key by parsing the input as a type and normalizing.
    pub fn parse(s: &str) -> Result<Self, TypeKeyParseError> {
        let ty: syn::Type = syn::parse_str(s).map_err(|error| TypeKeyParseError {
            input: s.to_string(),
            error,
        })?;
        Ok(Self::from_type(&ty))
    }

    /// Build a key directly from a `syn::Type` (normalizing a clone; the
    /// input is not modified).
    pub fn from_type(ty: &syn::Type) -> Self {
        // Off the shared reduction, so this key and the model's type index
        // cannot drift apart about what a type is called.
        let t = crate::api::core::flat::canonical_type(ty);
        Self {
            canon: t.to_token_stream().to_string().into(),
            ty: std::rc::Rc::new(t),
        }
    }

    /// Build a key for a bare item ident — infallible by construction (an
    /// ident IS a single-segment path type; nothing to parse or normalize).
    pub fn from_ident(ident: &syn::Ident) -> Self {
        Self::from_type(&crate::api::core::flat::type_from_ident(ident))
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.canon
    }

    /// The normalized parsed form. Infallible — a clone of the stored type,
    /// never a reparse.
    pub fn to_type(&self) -> syn::Type {
        (*self.ty).clone()
    }

    /// The bare item ident this key names — `Foo`, `a::Foo` → `Foo`; `None`
    /// when the type carries generic arguments or is not a path.
    ///
    /// Matches [`bare_path_ident`](crate::api::core::types_util::bare_path_ident)
    /// on the same type, which is the property
    /// `key_name_accessors_match_the_syn_walks` pins.
    ///
    /// **A name is not syntax**, which is why this is the key's business and
    /// producing a `syn::Type` is not. A caller that wants to look a declared
    /// item up by name was never asking for tokens; it was asking the key what
    /// it is called (#291).
    pub fn ident(&self) -> Option<syn::Ident> {
        let short = self.short_name()?;
        // The generic-argument rule the syn walk has: `Vec<u8>` names no bare
        // item, so it answers `None` where `short_name` still says "Vec".
        if self.canon.contains('<') {
            return None;
        }
        syn::parse_str::<syn::Ident>(&short).ok()
    }

    /// The last path segment's ident, **ignoring** generic arguments —
    /// `Publisher<'static>` → `"Publisher"`, `a::Foo` → `"Foo"`. `None` for
    /// anything that is not a path.
    ///
    /// The looser sibling of [`Self::ident`], for the callers that derive a
    /// destination-language class name from a Rust type: a declaration writes
    /// `ptr_class!(Publisher<'static>)` and means the class `Publisher`.
    ///
    /// # Read off the canonical string
    ///
    /// Deliberately, and not as a shortcut. `canon` is a token-stream
    /// rendering, so its tokens are space-separated — `Vec < u8 >`, `& Foo`,
    /// `a :: Foo`, `[u8 ; 4]` — which makes a path's head everything before the
    /// first `<`, and its last segment everything after the last `::`.
    /// `syn::parse_str::<syn::Ident>` is the total validator on the far end:
    /// every non-path shape fails it. Reparsing the whole type instead would
    /// make a name depend on a serialize-then-reparse round trip, which is the
    /// dependency #95 removed.
    ///
    /// One deliberate limit: a qualified-self path (`<T as Tr>::Item`) answers
    /// `None` rather than `Item`. It cannot reach here — `scan_declared_items`
    /// refuses a `qself` declaration — and refusing is the safe direction for a
    /// shape this cannot read.
    pub fn short_name(&self) -> Option<String> {
        // Strip generic arguments FIRST: in `Vec < a :: B >` the `::` belongs to
        // the argument, and the segment being named is still `Vec`.
        let head = self.canon.split('<').next()?;
        let tail = head.rsplit("::").next()?.trim();
        syn::parse_str::<syn::Ident>(tail)
            .ok()
            .map(|i| i.to_string())
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::core::types_util::bare_path_ident;

    /// Every shape a key can hold, as a build script or a source could spell it.
    ///
    /// A `qself` path is absent on purpose: `scan_declared_items` refuses one,
    /// and `short_name` documents answering `None` for it rather than reading
    /// through the qualification.
    const SHAPES: &[&str] = &[
        "Foo",
        "a::Foo",
        "std::string::String",
        "Vec<u8>",
        "Vec<a::B>",
        "Publisher<'static>",
        "Option<Box<Node>>",
        "&Foo",
        "&mut Foo",
        "&[u8]",
        "[u8; 4]",
        "()",
        "(u8, u8)",
        "*const u8",
        "dyn Error",
        "fn() -> u8",
    ];

    /// The accessors and the `syn` walks they replace answer identically.
    ///
    /// This is the whole warrant for reading names off the canonical string
    /// instead of off a parsed type. Both walks are the incumbent definition —
    /// `bare_path_ident` for [`TypeKey::ident`], and `rust_short_name_opt`'s
    /// last-segment rule (spelled out here rather than imported, since it lives
    /// under a language adapter) for [`TypeKey::short_name`].
    #[test]
    fn key_name_accessors_match_the_syn_walks() {
        for spec in SHAPES {
            let ty: syn::Type = syn::parse_str(spec).expect("test shape parses");
            let key = TypeKey::from_type(&ty);

            assert_eq!(
                key.ident(),
                bare_path_ident(&crate::api::core::flat::canonical_type(&ty)),
                "ident() disagrees with bare_path_ident on `{spec}` (canon `{key}`)"
            );

            // `rust_short_name_opt`: the last path segment's ident, generic
            // arguments and all.
            let expected_short = match &crate::api::core::flat::canonical_type(&ty) {
                syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
                _ => None,
            };
            assert_eq!(
                key.short_name(),
                expected_short,
                "short_name() disagrees with the last-segment rule on `{spec}` (canon `{key}`)"
            );
        }
    }

    /// `short_name` is looser than `ident` in exactly one way: generic arguments.
    #[test]
    fn short_name_reads_through_generics_and_ident_does_not() {
        let key = TypeKey::from_type(&syn::parse_quote!(Publisher<'static>));
        assert_eq!(key.short_name().as_deref(), Some("Publisher"));
        assert_eq!(key.ident(), None);
    }

    /// A name comes back out as the ident it names — `from_ident` is the inverse.
    #[test]
    fn ident_round_trips_through_from_ident() {
        let ident = syn::Ident::new("ZKeyExpr", proc_macro2::Span::call_site());
        let key = TypeKey::from_ident(&ident);
        assert_eq!(key.ident().as_ref(), Some(&ident));
        assert_eq!(key.short_name().as_deref(), Some("ZKeyExpr"));
    }
}
