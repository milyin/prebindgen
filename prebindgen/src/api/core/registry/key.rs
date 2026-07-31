//! The canonical identity of a type: its normalized token string.

use std::fmt;

use quote::ToTokens;

/// Canonical type-shape key: identity is the token string of the
/// **normalized** type ([`crate::api::core::types_util::normalize_type`] —
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
        let t = crate::api::core::types_util::canonical_type(ty);
        Self {
            canon: t.to_token_stream().to_string().into(),
            ty: std::rc::Rc::new(t),
        }
    }

    /// Build a key for a bare item ident — infallible by construction (an
    /// ident IS a single-segment path type; nothing to parse or normalize).
    pub fn from_ident(ident: &syn::Ident) -> Self {
        Self::from_type(&crate::api::core::types_util::type_from_ident(ident))
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
}

impl fmt::Display for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canon)
    }
}
