//! Valid subsets of scalar representations used by custom conversions.

use std::ops::{Bound, RangeBounds};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// Scalar representations that can carry a declarative domain.
pub trait DomainScalar: Copy + 'static {
    #[doc(hidden)]
    fn domain_value(self) -> ScalarValue;
    #[doc(hidden)]
    fn domain_type() -> syn::Type;
}

macro_rules! impl_ints {
    ($(($t:ty, $v:ident)),* $(,)?) => {$(
        impl DomainScalar for $t {
            fn domain_value(self) -> ScalarValue { ScalarValue::$v(self) }
            fn domain_type() -> syn::Type { syn::parse_quote!($t) }
        }
    )*};
}
impl_ints!(
    (i8, I8),
    (i16, I16),
    (i32, I32),
    (i64, I64),
    (i128, I128),
    (u8, U8),
    (u16, U16),
    (u32, U32),
    (u64, U64),
    (u128, U128),
);

impl DomainScalar for f32 {
    fn domain_value(self) -> ScalarValue {
        ScalarValue::F32(self.to_bits())
    }
    fn domain_type() -> syn::Type {
        syn::parse_quote!(f32)
    }
}
impl DomainScalar for f64 {
    fn domain_value(self) -> ScalarValue {
        ScalarValue::F64(self.to_bits())
    }
    fn domain_type() -> syn::Type {
        syn::parse_quote!(f64)
    }
}

/// Type-erased scalar. Floats retain their raw IEEE representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalarValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(u32),
    F64(u64),
}

impl ScalarValue {
    pub fn rust_expr(self) -> syn::Expr {
        match self {
            Self::I8(v) => syn::parse_quote!(#v),
            Self::I16(v) => syn::parse_quote!(#v),
            Self::I32(v) => syn::parse_quote!(#v),
            Self::I64(v) => syn::parse_quote!(#v),
            Self::I128(v) => syn::parse_quote!(#v),
            Self::U8(v) => syn::parse_quote!(#v),
            Self::U16(v) => syn::parse_quote!(#v),
            Self::U32(v) => syn::parse_quote!(#v),
            Self::U64(v) => syn::parse_quote!(#v),
            Self::U128(v) => syn::parse_quote!(#v),
            Self::F32(v) => syn::parse_quote!(f32::from_bits(#v)),
            Self::F64(v) => syn::parse_quote!(f64::from_bits(#v)),
        }
    }

    /// A literal expression suitable for C header generation. Arbitrary NaN
    /// payloads and infinities have no portable C constant spelling.
    #[cfg(feature = "unstable-cbindgen")]
    pub(crate) fn portable_expr(self) -> Option<syn::Expr> {
        match self {
            Self::F32(bits) => {
                let value = f32::from_bits(bits);
                value
                    .is_finite()
                    .then(|| syn::parse_str(&format!("{value:?}f32")).expect("finite f32 literal"))
            }
            Self::F64(bits) => {
                let value = f64::from_bits(bits);
                value
                    .is_finite()
                    .then(|| syn::parse_str(&format!("{value:?}f64")).expect("finite f64 literal"))
            }
            _ => Some(self.rust_expr()),
        }
    }

    pub fn ty(self) -> syn::Type {
        match self {
            Self::I8(_) => syn::parse_quote!(i8),
            Self::I16(_) => syn::parse_quote!(i16),
            Self::I32(_) => syn::parse_quote!(i32),
            Self::I64(_) => syn::parse_quote!(i64),
            Self::I128(_) => syn::parse_quote!(i128),
            Self::U8(_) => syn::parse_quote!(u8),
            Self::U16(_) => syn::parse_quote!(u16),
            Self::U32(_) => syn::parse_quote!(u32),
            Self::U64(_) => syn::parse_quote!(u64),
            Self::U128(_) => syn::parse_quote!(u128),
            Self::F32(_) => syn::parse_quote!(f32),
            Self::F64(_) => syn::parse_quote!(f64),
        }
    }

    fn raw_eq(self, value: &TokenStream) -> TokenStream {
        match self {
            Self::F32(bits) => quote!((#value).to_bits() == #bits),
            Self::F64(bits) => quote!((#value).to_bits() == #bits),
            _ => {
                let v = self.rust_expr();
                quote!((#value) == #v)
            }
        }
    }

    fn cmp_expr(self, value: &TokenStream, op: &str) -> TokenStream {
        let v = self.rust_expr();
        match op {
            ">" => quote!((#value) > #v),
            ">=" => quote!((#value) >= #v),
            "<" => quote!((#value) < #v),
            "<=" => quote!((#value) <= #v),
            _ => unreachable!(),
        }
    }

    fn is_integer_min(self) -> bool {
        matches!(
            self,
            Self::I8(i8::MIN)
                | Self::I16(i16::MIN)
                | Self::I32(i32::MIN)
                | Self::I64(i64::MIN)
                | Self::I128(i128::MIN)
                | Self::U8(u8::MIN)
                | Self::U16(u16::MIN)
                | Self::U32(u32::MIN)
                | Self::U64(u64::MIN)
                | Self::U128(u128::MIN)
        )
    }

    fn is_integer_max(self) -> bool {
        matches!(
            self,
            Self::I8(i8::MAX)
                | Self::I16(i16::MAX)
                | Self::I32(i32::MAX)
                | Self::I64(i64::MAX)
                | Self::I128(i128::MAX)
                | Self::U8(u8::MAX)
                | Self::U16(u16::MAX)
                | Self::U32(u32::MAX)
                | Self::U64(u64::MAX)
                | Self::U128(u128::MAX)
        )
    }
}

#[derive(Clone)]
enum Base {
    Range {
        start: Bound<ScalarValue>,
        end: Bound<ScalarValue>,
    },
    Values(Vec<ScalarValue>),
}

/// Legal values of a custom conversion's scalar representation.
#[derive(Clone)]
pub struct RepresentationDomain {
    ty: syn::Type,
    base: Base,
    excluded: Vec<ScalarValue>,
}

impl RepresentationDomain {
    pub fn range<T: DomainScalar, R: RangeBounds<T>>(range: R) -> Self {
        let cvt = |b: Bound<&T>| match b {
            Bound::Included(v) => Bound::Included((*v).domain_value()),
            Bound::Excluded(v) => Bound::Excluded((*v).domain_value()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let start = cvt(range.start_bound());
        let end = cvt(range.end_bound());
        assert!(
            !bound_is_nan(&start) && !bound_is_nan(&end),
            "representation-domain range bounds cannot be NaN"
        );
        assert!(
            range_is_nonempty(&start, &end),
            "representation-domain range cannot be empty"
        );
        Self {
            ty: T::domain_type(),
            base: Base::Range { start, end },
            excluded: vec![],
        }
    }

    pub fn values<T: DomainScalar>(values: impl IntoIterator<Item = T>) -> Self {
        let values = dedup(values.into_iter().map(T::domain_value).collect());
        assert!(
            !values.is_empty(),
            "representation-domain valid set cannot be empty"
        );
        Self {
            ty: T::domain_type(),
            base: Base::Values(values),
            excluded: vec![],
        }
    }

    pub fn exclude<T: DomainScalar>(&mut self, values: impl IntoIterator<Item = T>) {
        assert_eq!(
            crate::api::core::registry::TypeKey::from_type(&self.ty),
            crate::api::core::registry::TypeKey::from_type(&T::domain_type()),
            "representation-domain exclusions must use the base domain's scalar type"
        );
        self.excluded
            .extend(values.into_iter().map(T::domain_value));
        self.excluded = dedup(std::mem::take(&mut self.excluded));
    }

    pub fn ty(&self) -> &syn::Type {
        &self.ty
    }

    pub(crate) fn contains_expr(&self, value: TokenStream) -> TokenStream {
        let base = match &self.base {
            Base::Range { start, end } => {
                let lo = bound_expr(start, &value, true);
                let hi = bound_expr(end, &value, false);
                let not_nan = if matches!(
                    self.ty.to_token_stream().to_string().as_str(),
                    "f32" | "f64"
                ) {
                    quote!(!(#value).is_nan())
                } else {
                    quote!(true)
                };
                quote!(#not_nan && #lo && #hi)
            }
            Base::Values(values) => {
                let checks = values.iter().map(|v| v.raw_eq(&value));
                quote!(false #(|| #checks)*)
            }
        };
        let excluded = self.excluded.iter().map(|v| v.raw_eq(&value));
        quote!((#base) && !(false #(|| #excluded)*))
    }

    /// Derive a bounded number of stable values outside the legal domain.
    pub(crate) fn niche_values(&self, limit: usize) -> Vec<ScalarValue> {
        let mut out = Vec::new();
        let extra = match &self.base {
            Base::Values(v) => v.len(),
            _ => 0,
        };
        let budget = limit.saturating_add(extra).max(1);
        match self.ty.to_token_stream().to_string().as_str() {
            "i8" => ints!(out, self, i8, I8, budget),
            "i16" => ints!(out, self, i16, I16, budget),
            "i32" => ints!(out, self, i32, I32, budget),
            "i64" => ints!(out, self, i64, I64, budget),
            "i128" => ints!(out, self, i128, I128, budget),
            "u8" => ints!(out, self, u8, U8, budget),
            "u16" => ints!(out, self, u16, U16, budget),
            "u32" => ints!(out, self, u32, U32, budget),
            "u64" => ints!(out, self, u64, U64, budget),
            "u128" => ints!(out, self, u128, U128, budget),
            "f32" => float32_candidates(&mut out, budget),
            "f64" => float64_candidates(&mut out, budget),
            _ => unreachable!(),
        }
        out.extend(self.excluded.iter().copied());
        let mut selected = Vec::new();
        for value in out {
            if !self.contains(value) && !selected.contains(&value) {
                selected.push(value);
                if selected.len() == limit {
                    break;
                }
            }
        }
        selected
    }

    fn contains(&self, value: ScalarValue) -> bool {
        let base = match &self.base {
            Base::Range { start, end } => {
                !is_nan(value) && lower_ok(value, start) && upper_ok(value, end)
            }
            Base::Values(values) => values.contains(&value),
        };
        base && !self.excluded.contains(&value)
    }
}

macro_rules! ints {
    ($out:expr, $d:expr, $ty:ty, $variant:ident, $budget:expr) => {{
        let mut hi = <$ty>::MAX;
        let mut lo = <$ty>::MIN;
        for _ in 0..$budget {
            let h = ScalarValue::$variant(hi);
            if !$d.contains(h) {
                $out.push(h);
            }
            hi = hi.saturating_sub(1);
            let l = ScalarValue::$variant(lo);
            if !$d.contains(l) {
                $out.push(l);
            }
            lo = lo.saturating_add(1);
        }
    }};
}
use ints;

fn float32_candidates(out: &mut Vec<ScalarValue>, n: usize) {
    for i in 0..n {
        out.push(ScalarValue::F32(f32::MAX.to_bits() - i as u32));
        out.push(ScalarValue::F32((-f32::MAX).to_bits() - i as u32));
    }
    out.push(ScalarValue::F32(f32::INFINITY.to_bits()));
    out.push(ScalarValue::F32(f32::NEG_INFINITY.to_bits()));
    for i in 0..n {
        out.push(ScalarValue::F32(0x7fc0_0000 + i as u32));
    }
}
fn float64_candidates(out: &mut Vec<ScalarValue>, n: usize) {
    for i in 0..n {
        out.push(ScalarValue::F64(f64::MAX.to_bits() - i as u64));
        out.push(ScalarValue::F64((-f64::MAX).to_bits() - i as u64));
    }
    out.push(ScalarValue::F64(f64::INFINITY.to_bits()));
    out.push(ScalarValue::F64(f64::NEG_INFINITY.to_bits()));
    for i in 0..n {
        out.push(ScalarValue::F64(0x7ff8_0000_0000_0000 + i as u64));
    }
}

fn bound_expr(bound: &Bound<ScalarValue>, value: &TokenStream, lower: bool) -> TokenStream {
    match bound {
        Bound::Unbounded => quote!(true),
        Bound::Included(v) if lower && v.is_integer_min() => quote!(true),
        Bound::Included(v) if !lower && v.is_integer_max() => quote!(true),
        Bound::Included(v) if lower => v.cmp_expr(value, ">="),
        Bound::Excluded(v) if lower => v.cmp_expr(value, ">"),
        Bound::Included(v) => v.cmp_expr(value, "<="),
        Bound::Excluded(v) => v.cmp_expr(value, "<"),
    }
}
fn lower_ok(v: ScalarValue, b: &Bound<ScalarValue>) -> bool {
    match b {
        Bound::Unbounded => true,
        Bound::Included(x) => cmp(v, *x).is_some_and(|v| v >= 0),
        Bound::Excluded(x) => cmp(v, *x).is_some_and(|v| v > 0),
    }
}
fn upper_ok(v: ScalarValue, b: &Bound<ScalarValue>) -> bool {
    match b {
        Bound::Unbounded => true,
        Bound::Included(x) => cmp(v, *x).is_some_and(|v| v <= 0),
        Bound::Excluded(x) => cmp(v, *x).is_some_and(|v| v < 0),
    }
}
fn cmp(a: ScalarValue, b: ScalarValue) -> Option<i8> {
    macro_rules! c {
        ($a:expr, $b:expr) => {
            Some(if $a < $b {
                -1
            } else if $a > $b {
                1
            } else {
                0
            })
        };
    }
    match (a, b) {
        (ScalarValue::I8(a), ScalarValue::I8(b)) => c!(a, b),
        (ScalarValue::I16(a), ScalarValue::I16(b)) => c!(a, b),
        (ScalarValue::I32(a), ScalarValue::I32(b)) => c!(a, b),
        (ScalarValue::I64(a), ScalarValue::I64(b)) => c!(a, b),
        (ScalarValue::I128(a), ScalarValue::I128(b)) => c!(a, b),
        (ScalarValue::U8(a), ScalarValue::U8(b)) => c!(a, b),
        (ScalarValue::U16(a), ScalarValue::U16(b)) => c!(a, b),
        (ScalarValue::U32(a), ScalarValue::U32(b)) => c!(a, b),
        (ScalarValue::U64(a), ScalarValue::U64(b)) => c!(a, b),
        (ScalarValue::U128(a), ScalarValue::U128(b)) => c!(a, b),
        (ScalarValue::F32(a), ScalarValue::F32(b)) => {
            f32::from_bits(a).partial_cmp(&f32::from_bits(b)).map(ord)
        }
        (ScalarValue::F64(a), ScalarValue::F64(b)) => {
            f64::from_bits(a).partial_cmp(&f64::from_bits(b)).map(ord)
        }
        _ => None,
    }
}
fn ord(v: std::cmp::Ordering) -> i8 {
    match v {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}
fn is_nan(v: ScalarValue) -> bool {
    match v {
        ScalarValue::F32(v) => f32::from_bits(v).is_nan(),
        ScalarValue::F64(v) => f64::from_bits(v).is_nan(),
        _ => false,
    }
}
fn bound_is_nan(v: &Bound<ScalarValue>) -> bool {
    match v {
        Bound::Included(v) | Bound::Excluded(v) => is_nan(*v),
        Bound::Unbounded => false,
    }
}
fn range_is_nonempty(start: &Bound<ScalarValue>, end: &Bound<ScalarValue>) -> bool {
    let (start_value, start_included) = match start {
        Bound::Unbounded => return true,
        Bound::Included(value) => (*value, true),
        Bound::Excluded(value) => (*value, false),
    };
    let (end_value, end_included) = match end {
        Bound::Unbounded => return true,
        Bound::Included(value) => (*value, true),
        Bound::Excluded(value) => (*value, false),
    };
    cmp(start_value, end_value)
        .is_some_and(|ordering| ordering < 0 || (ordering == 0 && start_included && end_included))
}
fn dedup(values: Vec<ScalarValue>) -> Vec<ScalarValue> {
    let mut out = Vec::new();
    for v in values {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_range_derives_extreme_niches() {
        let domain = RepresentationDomain::range(0u64..=1_000_000);
        assert_eq!(
            domain.niche_values(3),
            vec![
                ScalarValue::U64(u64::MAX),
                ScalarValue::U64(u64::MAX - 1),
                ScalarValue::U64(u64::MAX - 2),
            ]
        );
    }

    #[test]
    fn integer_extreme_bounds_do_not_emit_useless_comparisons() {
        let domain = RepresentationDomain::range(0u64..=1_000_000);
        let expr = domain.contains_expr(quote!(value)).to_string();
        assert!(!expr.contains(">= 0u64"), "{expr}");
        assert!(expr.contains("<= 1000000u64"), "{expr}");

        let full = RepresentationDomain::range(i32::MIN..=i32::MAX);
        let expr = full.contains_expr(quote!(value)).to_string();
        assert!(!expr.contains(">="), "{expr}");
        assert!(!expr.contains("<="), "{expr}");
    }

    #[test]
    fn valid_set_and_exclusions_use_raw_float_bits() {
        let domain = RepresentationDomain::values([0.0f64, -0.0f64, 1.0]);
        assert!(domain.contains(ScalarValue::F64(0.0f64.to_bits())));
        assert!(domain.contains(ScalarValue::F64((-0.0f64).to_bits())));

        let mut range = RepresentationDomain::range(-1.0f64..=1.0f64);
        range.exclude([0.5f64]);
        assert!(!range.contains(ScalarValue::F64(0.5f64.to_bits())));
        assert!(!range.contains(ScalarValue::F64(f64::NAN.to_bits())));
    }

    #[test]
    #[should_panic(expected = "cannot be NaN")]
    fn nan_range_bound_is_rejected() {
        let _ = RepresentationDomain::range(f64::NAN..=1.0);
    }

    #[test]
    #[should_panic(expected = "range cannot be empty")]
    fn empty_range_is_rejected() {
        let _ = RepresentationDomain::range(2u8..2u8);
    }
}
