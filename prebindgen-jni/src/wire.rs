//! One JNI boundary value and its remaining niche availability.
use super::*;
pub(crate) mod jvm;
mod transport;
pub use transport::JniArgument;

/// One self-sufficient JNI value and its conversion information.
///
/// Niche availability is mutable state for the current slot. Consuming or
/// suppressing a niche must not restore the variant's initial availability.
/// Use the variant constructors to initialize it, and [`Self::set_niche`] to
/// check updates.
#[derive(Clone, Debug)]
pub enum JniWire {
    /// One complete bulk frame in a byte array. Input length is a separate control.
    BulkBuffer {
        /// Kotlin expression selecting this frame.
        condition: String,
        /// Whether null is available for an enclosing option.
        null_niche_available: bool,
    },
    /// Native byte/text view inside a bulk frame; never a JNI reference.
    NativeBuffer {
        /// Whether the payload is validated UTF-8 text.
        text: bool,
    },
    /// Native nullable offset/count descriptor for collection rows.
    NativeRows {
        /// Kotlin expression selecting these rows.
        condition: String,
    },
    /// Fixed-width big-endian bytes for a native 128-bit scalar.
    Native128 {
        /// Kotlin expression selecting the active input alternative.
        condition: String,
        /// Signed or unsigned 128-bit source primitive.
        primitive: Primitive,
    },
    /// A retained callback object with its independently prepared argument boundary.
    Callback {
        /// Signature and output conversions used when Rust invokes this callback.
        callback: std::rc::Rc<crate::JniCallback>,
    },
    /// One JNI object-array carrier. Buffered sequences store raw rows here;
    /// callback delivery uses an empty array as its present marker. The
    /// sequence container owns the element layout in both cases.
    Rows {
        /// Kotlin expression selecting this array's active input alternative.
        condition: String,
        /// Whether a null array remains available to an enclosing option.
        null_niche_available: bool,
    },
    /// A fieldless Rust enum carried as a checked JVM `int`.
    Enum {
        /// Valid source variants and generated Kotlin class information.
        enum_type: std::rc::Rc<JniEnum>,
        /// Next unused integer representation, or `None` when unavailable.
        next_integer_niche: Option<i32>,
    },
    /// A primitive in its JVM carrier.
    Scalar {
        /// Source primitive that determines the Rust/JVM conversion.
        primitive: Primitive,
        /// Next excluded integer representation, or `None` when unavailable.
        next_integer_niche: Option<i64>,
    },
    /// A boxed allocation address in a `long`, owned by a Kotlin handle class.
    Handle {
        /// Allocation type and access mode.
        handle: Box<BoxedWire>,
        /// Kotlin condition under which the handle is active.
        condition: String,
        /// Whether null is still available to an enclosing option.
        null_niche_available: bool,
    },
    /// A non-null `java.lang.String` when present.
    Text {
        /// Whether a null reference is still available to an enclosing option.
        null_niche_available: bool,
    },
    /// A non-null `byte[]` when present, including an empty array.
    Bytes {
        /// Whether a null reference is still available to an enclosing option.
        null_niche_available: bool,
    },
    /// An `int` selector with nonnegative valid tags (including 0/1 presence flags).
    Selector {
        /// Next unused negative tag, or `None` when unavailable.
        /// Positive unused tags are not advertised because arm counts vary.
        next_integer_niche: Option<i32>,
    },
}
impl JniWire {
    /// Describe a string with its null-reference niche initially available.
    pub fn text() -> Self {
        Self::Text {
            null_niche_available: true,
        }
    }
    /// Describe bytes with their null-reference niche initially available.
    pub fn bytes() -> Self {
        Self::Bytes {
            null_niche_available: true,
        }
    }
    /// Describe nonnegative selector tags with negative niches starting at -1.
    pub fn selector() -> Self {
        Self::Selector {
            next_integer_niche: Some(-1),
        }
    }
    /// Initialize a scalar's niche availability from its primitive domain.
    pub fn scalar(primitive: Primitive) -> Self {
        let next_integer_niche = integer_niche_min(primitive).map(|_| -1);
        Self::Scalar {
            primitive,
            next_integer_niche,
        }
    }
    /// Describe a handle with its null niche initially available.
    pub fn handle(handle: BoxedWire, condition: impl Into<String>) -> Self {
        Self::Handle {
            handle: Box::new(handle),
            condition: condition.into(),
            null_niche_available: true,
        }
    }
    /// Unused representation still available to an enclosing option.
    /// This reads stored state rather than recomputing the initial niche.
    pub fn available_niche(&self) -> Option<JniNiche> {
        match self {
            Self::Callback { .. }
            | Self::NativeBuffer { .. }
            | Self::NativeRows { .. }
            | Self::Native128 { .. } => None,
            Self::Scalar {
                next_integer_niche, ..
            } => next_integer_niche.map(JniNiche::Integer),
            Self::Handle {
                null_niche_available: true,
                ..
            }
            | Self::BulkBuffer {
                null_niche_available: true,
                ..
            }
            | Self::Rows {
                null_niche_available: true,
                ..
            }
            | Self::Text {
                null_niche_available: true,
            }
            | Self::Bytes {
                null_niche_available: true,
            } => Some(JniNiche::Null),
            Self::Selector { next_integer_niche }
            | Self::Enum {
                next_integer_niche, ..
            } => next_integer_niche.map(|n| JniNiche::Integer(n.into())),
            _ => None,
        }
    }
    /// Update availability, rejecting representations incompatible with this variant.
    /// Invalid updates return an error without changing the previous state.
    /// `None` suppresses or exhausts the niche of any variant.
    pub fn set_niche(&mut self, niche: Option<JniNiche>) -> Result<()> {
        match (self, niche) {
            (
                Self::Scalar {
                    primitive,
                    next_integer_niche,
                },
                Some(JniNiche::Integer(n)),
            ) if is_integer_niche(*primitive, n) => *next_integer_niche = Some(n),
            (
                Self::Handle {
                    null_niche_available,
                    ..
                }
                | Self::BulkBuffer {
                    null_niche_available,
                    ..
                }
                | Self::Rows {
                    null_niche_available,
                    ..
                }
                | Self::Text {
                    null_niche_available,
                }
                | Self::Bytes {
                    null_niche_available,
                },
                Some(JniNiche::Null),
            ) => *null_niche_available = true,
            (
                Self::Enum {
                    enum_type,
                    next_integer_niche,
                },
                Some(JniNiche::Integer(n)),
            ) if enum_type.mapping.is_niche(n) => {
                *next_integer_niche = Some(n as i32);
            }
            (Self::Selector { next_integer_niche }, Some(JniNiche::Integer(n)))
                if n < 0 && i32::try_from(n).is_ok() =>
            {
                *next_integer_niche = Some(n as i32)
            }
            (wire, None) => wire.clear_niche(),
            _ => {
                return Err(
                    "niche is incompatible with the JNI wire variant or primitive domain".into(),
                )
            }
        }
        Ok(())
    }
    /// Disable niche availability, including for conditionally active payloads.
    pub fn clear_niche(&mut self) {
        match self {
            Self::Callback { .. }
            | Self::NativeBuffer { .. }
            | Self::NativeRows { .. }
            | Self::Native128 { .. } => {}
            Self::Scalar {
                next_integer_niche, ..
            } => *next_integer_niche = None,
            Self::Handle {
                null_niche_available,
                ..
            }
            | Self::BulkBuffer {
                null_niche_available,
                ..
            }
            | Self::Rows {
                null_niche_available,
                ..
            }
            | Self::Text {
                null_niche_available,
            }
            | Self::Bytes {
                null_niche_available,
            } => *null_niche_available = false,
            Self::Selector { next_integer_niche }
            | Self::Enum {
                next_integer_niche, ..
            } => *next_integer_niche = None,
        }
    }
    // Reserved niches live on optional nodes after availability has advanced or
    // been cleared. Validate compatibility independently of the current state.
    pub(crate) fn validate_niche(&self, niche: &JniNiche) -> Result<()> {
        match (self, niche) {
            (Self::Scalar { primitive, .. }, JniNiche::Integer(n))
                if is_integer_niche(*primitive, *n) =>
            {
                Ok(())
            }
            (
                Self::Handle { .. }
                | Self::BulkBuffer { .. }
                | Self::Rows { .. }
                | Self::Text { .. }
                | Self::Bytes { .. },
                JniNiche::Null,
            ) => Ok(()),
            (Self::Enum { enum_type, .. }, JniNiche::Integer(n))
                if enum_type.mapping.is_niche(*n) =>
            {
                Ok(())
            }
            (Self::Selector { .. }, JniNiche::Integer(n))
                if *n < 0 && i32::try_from(*n).is_ok() =>
            {
                Ok(())
            }
            _ => Err("niche is incompatible with the JNI wire variant or primitive domain".into()),
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        if let Self::Enum { enum_type, .. } = self {
            enum_type.validate()?;
        }
        match self.available_niche() {
            Some(niche) => self.validate_niche(&niche),
            None => Ok(()),
        }
    }
}
pub(crate) fn integer_niche_min(primitive: Primitive) -> Option<i64> {
    match primitive {
        Primitive::U32 => Some(i64::MIN),
        Primitive::Char | Primitive::U8 | Primitive::U16 => Some(i32::MIN as i64),
        _ => None,
    }
}

fn is_integer_niche(primitive: Primitive, value: i64) -> bool {
    let Some(min) = integer_niche_min(primitive) else {
        return false;
    };
    let max = if matches!(primitive, Primitive::U32) {
        i64::MAX
    } else {
        i32::MAX as i64
    };
    if !(min..=max).contains(&value) {
        return false;
    }
    match primitive {
        Primitive::Char => value < 0 || char::from_u32(value as u32).is_none(),
        Primitive::U8 => u8::try_from(value).is_err(),
        Primitive::U16 => u16::try_from(value).is_err(),
        Primitive::U32 => u32::try_from(value).is_err(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle() -> JniWire {
        JniWire::handle(
            BoxedWire {
                ty: syn::parse_quote!(Counter),
                access: Access::Move,
            },
            "true",
        )
    }
    #[test]
    fn wire_constructors_initialize_only_excluded_representations() {
        for primitive in [
            Primitive::Char,
            Primitive::U8,
            Primitive::U16,
            Primitive::U32,
        ] {
            let wire = JniWire::scalar(primitive);
            assert!(matches!(
                wire.available_niche(),
                Some(JniNiche::Integer(-1))
            ));
            wire.validate().unwrap();
        }
        for primitive in [
            Primitive::Bool,
            Primitive::I8,
            Primitive::I16,
            Primitive::I32,
            Primitive::I64,
            Primitive::I128,
            Primitive::Isize,
            Primitive::U64,
            Primitive::U128,
            Primitive::Usize,
            Primitive::F32,
            Primitive::F64,
        ] {
            assert!(JniWire::scalar(primitive).available_niche().is_none());
        }
        assert!(matches!(handle().available_niche(), Some(JniNiche::Null)));
        for mut wire in [JniWire::text(), JniWire::bytes()] {
            assert_eq!(wire.available_niche(), Some(JniNiche::Null));
            assert!(wire.set_niche(Some(JniNiche::Integer(-1))).is_err());
            assert_eq!(wire.available_niche(), Some(JniNiche::Null));
            wire.clear_niche();
            assert!(wire.available_niche().is_none());
            wire.validate_niche(&JniNiche::Null).unwrap();
        }
    }
    #[test]
    fn selector_niches_advance_exhaust_and_validate_without_reinitialization() {
        let mut wire = JniWire::selector();
        assert_eq!(wire.available_niche(), Some(JniNiche::Integer(-1)));
        for n in [-1, -2, -12345, i32::MIN as i64] {
            wire.set_niche(Some(JniNiche::Integer(n))).unwrap();
            assert_eq!(wire.available_niche(), Some(JniNiche::Integer(n)));
            let next = optional::remaining(&wire, &JniNiche::Integer(n));
            assert_eq!(
                next,
                (n > i32::MIN as i64).then_some(JniNiche::Integer(n - 1))
            );
        }
        for bad in [
            JniNiche::Null,
            JniNiche::Integer(0),
            JniNiche::Integer(1),
            JniNiche::Integer(i32::MIN as i64 - 1),
            JniNiche::Integer(i64::MAX),
        ] {
            assert!(wire.set_niche(Some(bad)).is_err());
            assert_eq!(
                wire.available_niche(),
                Some(JniNiche::Integer(i32::MIN as i64))
            );
        }
        wire.clear_niche();
        assert!(wire.available_niche().is_none());
        wire.validate_niche(&JniNiche::Integer(-1)).unwrap();
        assert!(JniWire::Selector {
            next_integer_niche: Some(0)
        }
        .validate()
        .is_err());
    }
    #[test]
    fn scalar_niches_advance_exhaust_and_reject_invalid_updates_atomically() {
        let mut wire = JniWire::scalar(Primitive::Char);
        wire.set_niche(Some(JniNiche::Integer(-2))).unwrap();
        for niche in [
            JniNiche::Null,
            JniNiche::Integer(0),
            JniNiche::Integer(i32::MIN as i64 - 1),
        ] {
            assert!(wire.set_niche(Some(niche)).is_err());
            assert!(matches!(
                wire.available_niche(),
                Some(JniNiche::Integer(-2))
            ));
        }
        wire.set_niche(Some(JniNiche::Integer(i32::MIN as i64)))
            .unwrap();
        let reserved = wire.available_niche().unwrap();
        assert!(optional::remaining(&wire, &reserved).is_none());
        wire.set_niche(None).unwrap();
        assert!(wire.available_niche().is_none());
        // Reserved representations remain meaningful after availability is gone.
        wire.validate_niche(&reserved).unwrap();
        let mut widened = JniWire::scalar(Primitive::U32);
        widened
            .set_niche(Some(JniNiche::Integer(i64::MIN)))
            .unwrap();
        assert!(optional::remaining(&widened, &JniNiche::Integer(i64::MIN)).is_none());
        widened.clear_niche();
        assert!(widened.available_niche().is_none());
        assert!(JniWire::scalar(Primitive::I64)
            .set_niche(Some(JniNiche::Integer(-1)))
            .is_err());
    }
    #[test]
    fn consumed_handle_niche_stays_disabled_without_losing_handle_metadata() {
        let mut wire = handle();
        assert!(wire.set_niche(Some(JniNiche::Integer(-1))).is_err());
        assert!(matches!(wire.available_niche(), Some(JniNiche::Null)));
        wire.clear_niche();
        wire.set_niche(None).unwrap();
        assert!(wire.available_niche().is_none());
        let JniWire::Handle {
            handle,
            condition,
            null_niche_available,
        } = &wire
        else {
            panic!()
        };
        assert_eq!(handle.access, Access::Move);
        assert_eq!(condition, "true");
        assert!(!null_niche_available);
        wire.validate_niche(&JniNiche::Null).unwrap();
    }
    #[test]
    fn direct_enum_construction_is_checked_before_emission() {
        let wire = JniWire::Scalar {
            primitive: Primitive::I64,
            next_integer_niche: Some(-1),
        };
        assert!(wire.validate().unwrap_err().contains("incompatible"));
    }
    #[test]
    fn explicit_integer_niches_cover_excluded_positive_and_unicode_domains() {
        for (primitive, last_in_domain) in [
            (Primitive::U8, 256),
            (Primitive::U16, 65536),
            (Primitive::U32, u32::MAX as i64 + 1),
            (Primitive::Char, 0x110000),
            (Primitive::Char, 0xd800),
        ] {
            let mut wire = JniWire::scalar(primitive);
            wire.set_niche(Some(JniNiche::Integer(last_in_domain + 1)))
                .unwrap();
            assert!(
                matches!(optional::remaining(&wire, &JniNiche::Integer(last_in_domain + 1)), Some(JniNiche::Integer(n)) if n == last_in_domain)
            );
            wire.set_niche(Some(JniNiche::Integer(last_in_domain)))
                .unwrap();
            assert!(optional::remaining(&wire, &JniNiche::Integer(last_in_domain)).is_none());
        }
        assert!(JniWire::scalar(Primitive::Char)
            .set_niche(Some(JniNiche::Integer(i32::MAX as i64 + 1)))
            .is_err());
    }
}
