//! Opaque references issued for entries already present in a Flat type table.
use crate::{same_type, type_key, Flat, Result, Schema, TypeInfo};
use std::sync::Arc;
use syn::Type;

/// A reference to an existing entry in [`Flat::types`].
///
/// Only [`Flat::type_ref`] creates references. Cloning a reference preserves its
/// table key; it does not copy the source definition or its metadata. References
/// remain usable after cloning a Flat or mapping its attachments, provided the
/// type-table keys are preserved. Removing or renaming an entry makes lookup fail.
/// These are table references, not globally unique Rust type identities.
///
/// ```
/// use prebindgen_flat::Flat;
/// let source = syn::parse_file("pub struct Point { pub x: i32 }")?;
/// let flat = Flat::<()>::parse(source.items.into_iter().map(|item| (item, Default::default())))?;
/// let point = flat.type_ref(&syn::parse_quote!(Point))?;
/// assert_eq!(flat.get_type(&point)?.ty, syn::parse_quote!(crate::Point));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Fields are private; a reference cannot be constructed from a table key.
///
/// ```compile_fail
/// use prebindgen_flat::TypeRef;
/// let forged = TypeRef { key: "Point".into() };
/// ```
///
/// There is no unchecked string conversion.
///
/// ```compile_fail
/// use prebindgen_flat::TypeRef;
/// let forged: TypeRef = "Point".into();
/// ```
///
/// Nor is there a default reference.
///
/// ```compile_fail
/// use prebindgen_flat::TypeRef;
/// let forged = TypeRef::default();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeRef {
    key: Arc<str>,
}

impl TypeRef {
    /// Borrow the actual table key, for consumers borrowing [`Flat::types`].
    /// This permits lookup without exposing a way to construct a reference.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl<M: Schema> Flat<M> {
    /// Issue a reference to an existing canonical type entry.
    ///
    /// Declared short names and supported type spellings are resolved first.
    /// This does not register a type: even built-ins and composites require an
    /// existing entry, created for example by [`Flat::type_metadata_mut`].
    pub fn type_ref(&self, ty: &Type) -> Result<TypeRef> {
        let ty = if matches!(ty, Type::ImplTrait(_)) {
            self.resolve_parameter_type(ty)?
        } else {
            self.resolve_type(ty)?
        };
        let (key, _) = self
            .types
            .iter()
            .find(|(_, entry)| same_type(&entry.ty, &ty))
            .ok_or_else(|| format!("unregistered source type `{}`", type_key(&ty)))?;
        Ok(TypeRef {
            key: Arc::from(key.as_str()),
        })
    }

    /// Borrow an entry directly by a Flat-issued reference.
    #[allow(clippy::type_complexity)] // Keep the schema attachment roles explicit.
    pub fn get_type(
        &self,
        reference: &TypeRef,
    ) -> Result<&TypeInfo<M::Type, M::Field, M::Variant, M::Record, M::Enum, M::PayloadEnum>> {
        self.types
            .get(reference.key())
            .ok_or_else(|| format!("missing source type entry `{}`", reference.key()))
    }

    /// Mutably borrow an entry directly by a Flat-issued reference.
    #[allow(clippy::type_complexity)] // Keep the schema attachment roles explicit.
    pub fn get_type_mut(
        &mut self,
        reference: &TypeRef,
    ) -> Result<&mut TypeInfo<M::Type, M::Field, M::Variant, M::Record, M::Enum, M::PayloadEnum>>
    {
        self.types
            .get_mut(reference.key())
            .ok_or_else(|| format!("missing source type entry `{}`", reference.key()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Flat<()> {
        let items = syn::parse_file("pub struct Point {pub x:i32}")
            .unwrap()
            .items;
        Flat::parse(items.into_iter().map(|item| (item, Default::default()))).unwrap()
    }

    #[test]
    fn references_identify_the_actual_entry_across_clones_and_mutation() {
        let mut flat = source();
        let reference = flat.type_ref(&syn::parse_quote!(Point)).unwrap();
        assert_eq!(
            reference,
            flat.type_ref(&syn::parse_quote!(crate::Point)).unwrap()
        );
        assert!(std::ptr::eq(
            flat.get_type(&reference).unwrap(),
            &flat.types["Point"]
        ));
        flat.get_type_mut(&reference).unwrap().definition = crate::TypeDefinition::Opaque;
        let copied = flat.clone();
        assert!(matches!(
            copied.get_type(&reference).unwrap().definition,
            crate::TypeDefinition::Opaque
        ));
    }

    #[test]
    fn issuance_requires_registration_and_deleted_references_fail_lookup() {
        let mut flat = source();
        assert!(flat.type_ref(&syn::parse_quote!(Missing)).is_err());
        assert!(flat.type_ref(&syn::parse_quote!(u64)).is_err());
        assert!(flat.type_ref(&syn::parse_quote!(Vec<Point>)).is_err());
        assert_eq!(flat.types.len(), 1);
        flat.type_metadata_mut(&syn::parse_quote!(Vec<Point>))
            .unwrap();
        let reference = flat.type_ref(&syn::parse_quote!(Vec<Point>)).unwrap();
        assert_eq!(
            flat.get_type(&reference).unwrap().ty,
            syn::parse_quote!(Vec<crate::Point>)
        );
        flat.types.remove(reference.key()).unwrap();
        assert!(flat.get_type(&reference).is_err());
        assert!(flat.get_type_mut(&reference).is_err());
    }
}
