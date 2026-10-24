//! One C boundary value and its remaining niche availability.
use super::*;

/// One self-sufficient C value and its conversion information.
///
/// Constructors initialize niche availability once. An enclosing optional node
/// reserves a sentinel and advances or clears that state; reading it must never
/// restore a consumed niche. Use [`Self::set_niche`] to check updates.
#[derive(Clone, Debug)]
pub enum CWire {
    /// One pointer/length carrier of generated ABI rows. Its element layout lives
    /// in the enclosing sequence chain; empty rows may use a null pointer.
    Rows,
    /// A fieldless Rust enum represented by checked `int32_t` values.
    Enum {
        /// Unprefixed C typedef name.
        name: String,
        /// Valid source variants and their integer representations.
        mapping: Box<prebindgen_declarations::EnumMapping>,
        /// Next unused integer representation, or `None` when unavailable.
        next_integer_niche: Option<i32>,
    },
    /// A primitive in its C scalar type.
    Scalar {
        /// Source primitive that determines the Rust/C conversion.
        primitive: Primitive,
        /// Next excluded integer representation, or `None` when unavailable.
        next_integer_niche: Option<i64>,
    },
    /// Text or bytes as one pointer/length structure.
    /// A null pointer with zero length is a valid empty value, so no null niche
    /// is available for this representation.
    Buffer,
    /// A typed opaque pointer to a boxed source value.
    Pointer {
        /// Pointee type and mutability.
        pointer: Box<CPointer>,
        /// Whether null is still available to an enclosing option.
        null_niche_available: bool,
    },
}
impl CWire {
    /// Initialize a scalar's niche availability from its primitive domain.
    pub fn scalar(primitive: Primitive) -> Self {
        Self::Scalar {
            primitive,
            next_integer_niche: matches!(primitive, Primitive::Char).then_some(0x110000),
        }
    }
    /// Describe an enum with its first unused negative value available.
    pub fn enumeration(
        name: impl Into<String>,
        mapping: prebindgen_declarations::EnumMapping,
    ) -> Self {
        Self::Enum {
            name: name.into(),
            next_integer_niche: mapping.initial_niche(),
            mapping: Box::new(mapping),
        }
    }
    /// Describe a typed pointer with its null niche initially available.
    pub fn pointer(pointer: CPointer) -> Self {
        Self::Pointer {
            pointer: Box::new(pointer),
            null_niche_available: true,
        }
    }
    /// Unused representation still available to an enclosing option.
    /// This reads stored state rather than recomputing the initial niche.
    pub fn available_niche(&self) -> Option<CNiche> {
        match self {
            Self::Scalar {
                next_integer_niche, ..
            } => next_integer_niche.map(CNiche::Integer),
            Self::Enum {
                next_integer_niche, ..
            } => next_integer_niche.map(|n| CNiche::Integer(n.into())),
            Self::Pointer {
                null_niche_available: true,
                ..
            } => Some(CNiche::Null),
            _ => None,
        }
    }
    /// Update availability, rejecting representations incompatible with this variant.
    /// Invalid updates leave the previous state unchanged. `None` disables niches.
    pub fn set_niche(&mut self, niche: Option<CNiche>) -> Result<()> {
        if let Some(niche) = &niche {
            self.validate_niche(niche)?;
        }
        match (self, niche) {
            (
                Self::Scalar {
                    next_integer_niche, ..
                },
                Some(CNiche::Integer(n)),
            ) => *next_integer_niche = Some(n),
            (
                Self::Enum {
                    next_integer_niche, ..
                },
                Some(CNiche::Integer(n)),
            ) => *next_integer_niche = Some(n as i32),
            (
                Self::Pointer {
                    null_niche_available,
                    ..
                },
                Some(CNiche::Null),
            ) => *null_niche_available = true,
            (wire, None) => wire.clear_niche(),
            _ => unreachable!("niche compatibility was checked above"),
        }
        Ok(())
    }
    /// Disable niche availability, including for conditionally active payloads.
    pub fn clear_niche(&mut self) {
        match self {
            Self::Scalar {
                next_integer_niche, ..
            } => *next_integer_niche = None,
            Self::Enum {
                next_integer_niche, ..
            } => *next_integer_niche = None,
            Self::Pointer {
                null_niche_available,
                ..
            } => *null_niche_available = false,
            Self::Buffer | Self::Rows => {}
        }
    }
    // A reserved sentinel remains valid even after availability advances or clears.
    pub(crate) fn validate_niche(&self, niche: &CNiche) -> Result<()> {
        match (self, niche) {
            (
                Self::Scalar {
                    primitive: Primitive::Char,
                    ..
                },
                CNiche::Integer(n),
            ) if u32::try_from(*n).is_ok_and(|n| char::from_u32(n).is_none()) => Ok(()),
            (Self::Enum { mapping, .. }, CNiche::Integer(n)) if mapping.is_niche(*n) => Ok(()),
            (Self::Pointer { .. }, CNiche::Null) => Ok(()),
            _ => Err("niche is incompatible with the C wire variant or primitive domain".into()),
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        if let Self::Enum { mapping, .. } = self {
            mapping.validate()?;
        }
        match self.available_niche() {
            Some(niche) => self.validate_niche(&niche),
            None => Ok(()),
        }
    }
    pub(crate) fn remaining_niche(&self, niche: &CNiche) -> Option<CNiche> {
        self.validate_niche(niche).ok()?;
        match (self, niche) {
            (Self::Enum { mapping, .. }, CNiche::Integer(n)) => (*n as i32)
                .checked_sub(1)
                .and_then(|n| mapping.unused_at_or_below(n))
                .map(|n| CNiche::Integer(n.into())),
            (
                Self::Scalar {
                    primitive: Primitive::Char,
                    ..
                },
                CNiche::Integer(n),
            ) => {
                let next = CNiche::Integer(n + 1);
                self.validate_niche(&next).ok().map(|()| next)
            }
            _ => None,
        }
    }
}

impl WireKind for CWire {
    fn rust_type(&self) -> syn::Type {
        match self {
            CWire::Enum { .. } => syn::parse_quote!(i32),
            CWire::Scalar { primitive: p, .. } => match p {
                Primitive::Bool => syn::parse_quote!(u8),
                Primitive::Char => syn::parse_quote!(u32),
                Primitive::I128 | Primitive::U128 => syn::parse_quote!(__Pb128),
                p => p.rust_type(),
            },
            CWire::Pointer { pointer: p, .. } => {
                let ty = &p.ty;
                if !p.mutable {
                    syn::parse_quote!(*const #ty)
                } else {
                    syn::parse_quote!(*mut #ty)
                }
            }
            CWire::Buffer => syn::parse_quote!(__PbBuffer),
            CWire::Rows => syn::parse_quote!(__PbRows),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer(mutable: bool) -> CWire {
        CWire::pointer(CPointer {
            ty: syn::parse_quote!(Widget),
            mutable,
        })
    }
    fn enumeration() -> CWire {
        CWire::enumeration(
            "code",
            prebindgen_declarations::EnumMapping {
                ty: syn::parse_quote!(Code),
                variants: vec![
                    (syn::parse_quote!(A), -1),
                    (syn::parse_quote!(B), 0),
                    (syn::parse_quote!(C), 2),
                ],
            },
        )
    }
    #[test]
    fn constructors_initialize_only_supported_niches() {
        assert_eq!(
            CWire::scalar(Primitive::Char).available_niche(),
            Some(CNiche::Integer(0x110000))
        );
        for primitive in [
            Primitive::Bool,
            Primitive::I8,
            Primitive::I16,
            Primitive::I32,
            Primitive::I64,
            Primitive::I128,
            Primitive::Isize,
            Primitive::U8,
            Primitive::U16,
            Primitive::U32,
            Primitive::U64,
            Primitive::U128,
            Primitive::Usize,
            Primitive::F32,
            Primitive::F64,
        ] {
            let mut wire = CWire::scalar(primitive);
            assert_eq!(wire.available_niche(), None);
            for niche in [CNiche::Null, CNiche::Integer(-1)] {
                assert!(wire.set_niche(Some(niche)).is_err());
                assert_eq!(wire.available_niche(), None);
            }
            wire.validate().unwrap();
        }
        let mut buffer = CWire::Buffer;
        for niche in [CNiche::Null, CNiche::Integer(-1)] {
            assert!(buffer.set_niche(Some(niche)).is_err());
        }
        buffer.set_niche(None).unwrap();
        assert_eq!(buffer.available_niche(), None);
        assert_eq!(pointer(false).available_niche(), Some(CNiche::Null));
        assert_eq!(enumeration().available_niche(), Some(CNiche::Integer(-2)));
    }
    #[test]
    fn character_domain_updates_are_checked_atomically_and_exhaust_at_boundaries() {
        let mut wire = CWire::scalar(Primitive::Char);
        for bad in [
            CNiche::Null,
            CNiche::Integer(-1),
            CNiche::Integer(0),
            CNiche::Integer(0xd7ff),
            CNiche::Integer(0xe000),
            CNiche::Integer(0x10ffff),
            CNiche::Integer(u32::MAX as i64 + 1),
            CNiche::Integer(i64::MAX),
        ] {
            assert!(wire.set_niche(Some(bad)).is_err());
            assert_eq!(wire.available_niche(), Some(CNiche::Integer(0x110000)));
        }
        for (value, next) in [
            (0xd800, Some(0xd801)),
            (0xdfff, None),
            (0x110000, Some(0x110001)),
            (u32::MAX as i64, None),
        ] {
            let niche = CNiche::Integer(value);
            wire.set_niche(Some(niche.clone())).unwrap();
            assert_eq!(wire.remaining_niche(&niche), next.map(CNiche::Integer));
            wire.clear_niche();
            assert_eq!(wire.available_niche(), None);
            wire.validate_niche(&niche).unwrap();
        }
        assert!(wire.remaining_niche(&CNiche::Integer(-1)).is_none());
    }
    #[test]
    fn enums_skip_valid_tags_and_preserve_suppressed_availability() {
        let mut wire = enumeration();
        for bad in [
            CNiche::Null,
            CNiche::Integer(-1),
            CNiche::Integer(0),
            CNiche::Integer(2),
            CNiche::Integer(i32::MIN as i64 - 1),
            CNiche::Integer(i32::MAX as i64 + 1),
        ] {
            assert!(wire.set_niche(Some(bad)).is_err());
            assert_eq!(wire.available_niche(), Some(CNiche::Integer(-2)));
        }
        wire.set_niche(Some(CNiche::Integer(1))).unwrap();
        assert_eq!(
            wire.remaining_niche(&CNiche::Integer(1)),
            Some(CNiche::Integer(-2))
        );
        wire.set_niche(Some(CNiche::Integer(i32::MIN as i64)))
            .unwrap();
        assert_eq!(
            wire.remaining_niche(&CNiche::Integer(i32::MIN as i64)),
            None
        );
        wire.set_niche(None).unwrap();
        wire.validate_niche(&CNiche::Integer(1)).unwrap();
        assert_eq!(wire.available_niche(), None);
        let CWire::Enum { mapping, name, .. } = &wire else {
            panic!()
        };
        assert_eq!(name, "code");
        assert_eq!(mapping.variants.len(), 3);
    }
    #[test]
    fn pointers_keep_their_abi_after_consuming_null() {
        for mutable in [false, true] {
            let mut wire = pointer(mutable);
            let before = wire.rust_type();
            assert!(wire.set_niche(Some(CNiche::Integer(0))).is_err());
            assert_eq!(wire.available_niche(), Some(CNiche::Null));
            assert_eq!(wire.remaining_niche(&CNiche::Null), None);
            wire.clear_niche();
            wire.set_niche(None).unwrap();
            assert_eq!(wire.available_niche(), None);
            wire.validate_niche(&CNiche::Null).unwrap();
            assert!(prebindgen_flat::same_type(&before, &wire.rust_type()));
            let CWire::Pointer { pointer, .. } = &wire else {
                panic!()
            };
            assert_eq!(pointer.mutable, mutable);
            assert!(prebindgen_flat::same_type(
                &pointer.ty,
                &syn::parse_quote!(Widget)
            ));
        }
    }
}
