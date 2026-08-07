//! The canonical identity of a type: its normalized token string.

use std::fmt;

use quote::ToTokens;

/// Canonical type-shape key: identity is the token string of the
/// **normalized** type. Normalization is a closed rule set — group/paren
/// unwrap, a `crate::`/`self::`/source-module path reduced to its final
/// segment, and a prelude path read as the bare name the language knows it
/// by (`std::vec::Vec<Foo>` ≡ `Vec<Foo>`) — and any spelling it does not
/// cover is kept verbatim.
///
/// # A key is an identity, and nothing else
///
/// It is what a table is indexed by. It is **not** a route to `syn::Type`: the
/// only way to reach a type's syntax is
/// `Conversions::reading` (in the registry layer above) followed by
/// [`TypeRef`](super::TypeRef), because a
/// reading is what pairs a spelling with the classification that vouches for
/// it.
///
/// This used to keep the parsed form beside the string and hand it out through
/// `to_type()`, which let any holder of a key produce tokens for a type the
/// model never classified — the same capability #280 sealed `TypeRef` against,
/// granted by the key itself. A caller that wants tokens now has to have gotten
/// them from somewhere that knows what they mean: the registry's reading, or
/// the declaration that wrote them (#291).
///
/// What a key can still answer about itself is what it is **called** —
/// [`Self::as_str`], [`Self::ident`], [`Self::short_name`] — because a name is
/// not syntax.
#[derive(Clone)]
pub struct TypeKey {
    /// Canonical token string — the identity `Eq`/`Hash` compare, and the whole
    /// of what a key is.
    canon: std::rc::Rc<str>,
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
    ///
    /// The parse is kept for **validation** and then discarded: a key that
    /// cannot be a type is a mistake worth reporting at the declaration, and a
    /// key that can is still only its canonical string.
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
        Self {
            canon: crate::flat::canonical_type(ty)
                .to_token_stream()
                .to_string()
                .into(),
        }
    }

    /// Build a key for a bare item ident — infallible by construction (an
    /// ident IS a single-segment path type; nothing to parse or normalize).
    pub fn from_ident(ident: &syn::Ident) -> Self {
        Self::from_type(&syn::parse_quote!(#ident))
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.canon
    }

    /// The bare item ident this key names — `Foo`, `a::Foo` → `Foo`,
    /// `a::Foo<u8>::Bar` → `Bar`; `None` when the **last** segment carries
    /// generic arguments (`Vec<u8>` names no bare item) or the key is not a
    /// path.
    /// Matches [`bare_path_ident`](crate::types_util::bare_path_ident) on the
    /// same type — a correspondence this crate's tests pin.
    ///
    /// **A name is not syntax**, which is why this is the key's business and
    /// producing a `syn::Type` is not. A caller that wants to look a declared
    /// item up by name was never asking for tokens; it was asking the key what
    /// it is called (#291).
    pub fn ident(&self) -> Option<syn::Ident> {
        let (ident, generic) = self.path_segments()?.pop()?;
        // `bare_path_ident` reads `PathArguments` on the LAST segment only, so
        // arguments earlier in the path do not disqualify the name.
        if generic {
            return None;
        }
        Some(ident)
    }

    /// The last path segment's ident, **ignoring** its generic arguments —
    /// `Publisher<'static>` → `"Publisher"`, `a::Foo<u8>::Bar` → `"Bar"`.
    /// `None` for anything that is not a path.
    ///
    /// The looser sibling of [`Self::ident`], for the callers that derive a
    /// destination-language class name from a Rust type: a declaration writes
    /// `ptr_class!(Publisher<'static>)` and means the class `Publisher`.
    pub fn short_name(&self) -> Option<String> {
        Some(self.path_segments()?.pop()?.0.to_string())
    }

    /// The **top-level** path segments of the canonical string: each segment's
    /// ident, and whether that segment carried generic arguments. `None` if the
    /// canon is not a path at all.
    ///
    /// # Read off the canonical string
    ///
    /// Deliberately, and not as a shortcut. `canon` is a token-stream
    /// rendering, so its tokens are space-separated — `Vec < u8 >`, `& Foo`,
    /// `a :: Foo` — and the structure is recoverable by tracking angle depth.
    /// Reparsing the whole type instead would make a NAME depend on a
    /// serialize-then-reparse round trip, which is the dependency #95 removed;
    /// storing the derived names on the key would put derived state back on a
    /// value whose whole point is that it carries none.
    ///
    /// **Nesting-aware, because a path segment is not the last thing before a
    /// `<`.** `a::Foo<u8>::Bar` names `Bar`, and `Vec<a::B>` names `Vec` — the
    /// `::` in the second belongs to the argument. Splitting at the first `<`
    /// got the second right and the first wrong.
    ///
    /// `syn::parse_str::<syn::Ident>` on every segment is the totality check:
    /// each non-path shape puts something in a segment that is not an ident —
    /// `& Foo`, `[u8 ; 4]`, `( )`, `* const u8`, `dyn Error`, `fn () -> u8`.
    fn path_segments(&self) -> Option<Vec<(syn::Ident, bool)>> {
        let mut rest: &str = &self.canon;
        // A qualified-self path renders its qualification first
        // (`< T as Tr > :: Item`) and syn keeps only the tail in
        // `path.segments` — so drop the group and read the rest as a plain path.
        if rest.starts_with('<') {
            rest = rest[close_angle(rest)? + 1..]
                .trim_start()
                .strip_prefix("::")?;
        }

        let bytes = rest.as_bytes();
        let mut out = Vec::new();
        let mut depth = 0usize;
        let mut start = 0usize;
        let mut ident_end: Option<usize> = None;
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'<' => {
                    if depth == 0 && ident_end.is_none() {
                        ident_end = Some(i);
                    }
                    depth += 1;
                }
                // The `>` of a bare fn's `->` is an arrow, not a bracket, and
                // miscounting it would let a `::` inside `Vec<fn() -> a::B>`
                // read as a top-level separator.
                b'>' if i > 0 && bytes[i - 1] == b'-' => {}
                b'>' => depth = depth.saturating_sub(1),
                b':' if depth == 0 && bytes.get(i + 1) == Some(&b':') => {
                    out.push(segment(rest, start, ident_end, i)?);
                    i += 2;
                    start = i;
                    ident_end = None;
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
        out.push(segment(rest, start, ident_end, bytes.len())?);
        Some(out)
    }
}

/// One path segment's ident (up to its own generic arguments, if any) and
/// whether it had them. `None` when the text is not an ident, which is how a
/// non-path canon is refused.
fn segment(
    s: &str,
    start: usize,
    ident_end: Option<usize>,
    end: usize,
) -> Option<(syn::Ident, bool)> {
    let text = s[start..ident_end.unwrap_or(end)].trim();
    Some((
        syn::parse_str::<syn::Ident>(text).ok()?,
        ident_end.is_some(),
    ))
}

/// Byte index of the `>` closing the angle group `s` opens with.
fn close_angle(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'<' => depth += 1,
            b'>' if i > 0 && bytes[i - 1] == b'-' => {}
            b'>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

impl fmt::Display for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types_util::bare_path_ident;

    /// Every shape a key can hold, as a build script or a source could spell it.
    ///
    /// Generics on a **non-final** segment (`a::Foo<u8>::Bar`) and a `::` inside
    /// an argument (`Vec<a::B>`) pull in opposite directions, and a `->` inside
    /// an argument (`Vec<fn() -> a::B>`) breaks naive angle counting — each is a
    /// way the string walk can be wrong while the easy cases still pass.
    const SHAPES: &[&str] = &[
        "Foo",
        "a::Foo",
        "a::b::Foo",
        "std::string::String",
        "Vec<u8>",
        "Vec<a::B>",
        "Vec<Vec<u8>>",
        "a::Foo<u8>::Bar",
        "Foo<u8>::Assoc",
        "<T as Tr>::Item",
        "Vec<fn() -> a::B>",
        "Publisher<'static>",
        "Option<Box<Node>>",
        "&Foo",
        "&mut Foo",
        "&[u8]",
        "[u8; 4]",
        "()",
        "(u8, u8)",
        "(a::B, c::D)",
        "*const u8",
        "dyn Error",
        "fn() -> u8",
        "fn(u8) -> a::B",
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
                bare_path_ident(&crate::flat::canonical_type(&ty)),
                "ident() disagrees with bare_path_ident on `{spec}` (canon `{key}`)"
            );

            // `rust_short_name_opt`: the last path segment's ident, generic
            // arguments and all.
            let expected_short = match &crate::flat::canonical_type(&ty) {
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

    /// `short_name` is looser than `ident` in exactly one way: generic arguments
    /// **on the last segment**.
    #[test]
    fn short_name_reads_through_last_segment_generics_and_ident_does_not() {
        let key = TypeKey::from_type(&syn::parse_quote!(Publisher<'static>));
        assert_eq!(key.short_name().as_deref(), Some("Publisher"));
        assert_eq!(key.ident(), None);

        // Arguments EARLIER in the path disqualify nothing: the segment being
        // named is `Bar`, and it has none.
        let nested = TypeKey::from_type(&syn::parse_quote!(a::Foo<u8>::Bar));
        assert_eq!(nested.short_name().as_deref(), Some("Bar"));
        assert_eq!(
            nested.ident().map(|i| i.to_string()).as_deref(),
            Some("Bar")
        );
    }

    /// A qualified-self path names its tail, like `bare_path_ident` does —
    /// syn keeps only `Item` in `path.segments`, and so does the string walk.
    #[test]
    fn qualified_self_paths_name_their_tail() {
        let key = TypeKey::from_type(&syn::parse_quote!(<T as Tr>::Item));
        assert_eq!(key.short_name().as_deref(), Some("Item"));
        assert_eq!(key.ident().map(|i| i.to_string()).as_deref(), Some("Item"));
    }

    /// A `::` inside a generic argument is not a path separator, and neither
    /// angle counting nor the `->` in a bare-fn argument may make it look like
    /// one.
    #[test]
    fn separators_inside_generic_arguments_are_not_path_separators() {
        for (spec, expected) in [
            ("Vec<a::B>", Some("Vec")),
            ("Vec<fn() -> a::B>", Some("Vec")),
            ("Vec<Vec<a::B>>", Some("Vec")),
        ] {
            let key = TypeKey::from_type(&syn::parse_str(spec).expect("test shape"));
            assert_eq!(key.short_name().as_deref(), expected, "on `{spec}`");
        }
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
