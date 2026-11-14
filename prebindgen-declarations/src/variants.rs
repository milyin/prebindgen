//! Recursive construction and matching of source enum payloads.
use super::*;
use std::collections::BTreeMap;

/// Collect enum variant/member overrides, rejecting duplicate keys.
pub fn overrides<'a, T, Fields, Name>(
    entries: impl IntoIterator<Item = (&'a str, Fields)>,
) -> BTreeMap<String, BTreeMap<String, T>>
where
    Fields: IntoIterator<Item = (Name, T)>,
    Name: AsRef<str>,
{
    let mut variants = BTreeMap::new();
    for (name, fields) in entries {
        let mut members = BTreeMap::new();
        for (field, tree) in fields {
            assert!(
                members.insert(field.as_ref().to_owned(), tree).is_none(),
                "duplicate enum field override"
            );
        }
        assert!(
            variants.insert(name.into(), members).is_none(),
            "duplicate enum variant override"
        );
    }
    variants
}

/// Check override names against the source enum's variants and fields.
pub fn validate_overrides<V, F, T>(
    source: &prebindgen_flat::PayloadEnum<V, F>,
    entries: &BTreeMap<String, BTreeMap<String, T>>,
) -> Result<()> {
    for (variant, fields) in entries {
        let arm = source
            .variants
            .iter()
            .find(|v| v.name.to_string().trim_start_matches("r#") == variant)
            .ok_or_else(|| format!("unknown enum variant `{variant}`"))?;
        for field in fields.keys() {
            if !arm.fields.iter().any(|f| f.key() == *field) {
                return Err(format!("unknown enum field `{variant}.{field}`"));
            }
        }
    }
    Ok(())
}
