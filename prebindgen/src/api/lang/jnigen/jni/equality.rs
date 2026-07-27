//! Content-based `equals` / `hashCode` / `toString` for generated Kotlin
//! classes with an **array-backed** property.
//!
//! A generated class mirrors a Rust type that derives `PartialEq`/`Eq`, so two
//! values with equal contents must compare equal. Kotlin arrays compare by
//! IDENTITY, which breaks that for every `Vec<u8>` field (`ByteArray`) and for
//! the `ByteArray` a value blob carries:
//!
//! ```text
//! Timestamp(1uL, byteArrayOf(1,2,3)) == Timestamp(1uL, byteArrayOf(1,2,3))  // false
//! ```
//!
//! Kotlin is inconsistent here rather than uniformly identity-based: a `data
//! class`'s generated `hashCode`/`toString` DO special-case arrays
//! (`contentHashCode` / `contentToString`), while its `equals` does not. So a
//! broken value has an equal hash and an unequal `equals` — it lands in the
//! right `HashMap` bucket and is then rejected. Both members are emitted here
//! anyway, so the behavior is stated by the generated source rather than
//! inherited from a compiler special case that only covers two of the three.
//!
//! ## Why value blobs are not `@JvmInline`
//!
//! Kotlin 1.9 (the version this generator targets downstream) rejects
//! `equals`/`hashCode` members on a value class outright — *"Member with the
//! name 'equals' is reserved for future releases"* — and its typed-equals
//! replacement (`operator fun equals(other: T)`) is experimental, needing an
//! opt-in flag every consumer would have to set. A `@JvmInline value class`
//! therefore CANNOT be given value equality at this language level, so
//! [`super::kotlin_emit`] emits value blobs as a plain `data class`. That costs
//! one small allocation per crossing at the wrapper tier (the JNI ABI is
//! unaffected — externs declare `ByteArray` directly and the wrapper passes
//! `.bytes`), which is the price of the type behaving like the value it is.

use super::*;

/// A property whose Kotlin type compares by identity and therefore needs
/// content-aware operators.
enum ArrayProp {
    /// `ByteArray` / `ByteArray?`.
    Bytes { nullable: bool },
}

/// Classify one constructor property. `None` for everything that already
/// compares by value (scalars, `String`, enums, nested generated classes).
fn array_prop(ty: &kt::KtType) -> Option<ArrayProp> {
    match ty.simple_name() {
        Some("ByteArray") => Some(ArrayProp::Bytes {
            nullable: ty.is_nullable(),
        }),
        _ => None,
    }
}

/// The `equals` / `hashCode` / `toString` trio a class needs when any
/// constructor property is array-backed.
///
/// `None` when none is — the compiler's own generation is already correct
/// there, and emitting these would be pure churn on every existing class.
///
/// `props` is the class's constructor properties in declaration order, as
/// `(kotlin_name, kotlin_type)`.
pub(crate) fn content_equality_members(
    class_name: &str,
    props: &[(String, kt::KtType)],
) -> Option<Vec<kt::KtFun>> {
    if !props.iter().any(|(_, ty)| array_prop(ty).is_some()) {
        return None;
    }

    // `equals`: identity short-circuit, type check, then per-property
    // comparison — `contentEquals` for arrays (its nullable-receiver overload
    // covers `ByteArray?`), `==` for everything else.
    let comparisons: Vec<String> = props
        .iter()
        .map(|(name, ty)| match array_prop(ty) {
            Some(ArrayProp::Bytes { .. }) => format!("{name}.contentEquals(other.{name})"),
            None => format!("{name} == other.{name}"),
        })
        .collect();
    let mut equals_body = kt::Code::new()
        .line("if (this === other) return true")
        .line(format!("if (other !is {class_name}) return false"));
    equals_body = equals_body.line(format!("return {}", comparisons.join(" && ")));
    let equals = kt::KtFun::new("equals")
        .modifier("override")
        .param(kt::KtParam::new("other", kt::KtType::any().nullable()))
        .returns(kt::KtType::boolean())
        .body(equals_body);

    // `hashCode`: the standard 31-multiplier fold, with `contentHashCode` for
    // arrays (`?: 0` for an absent one, matching how Kotlin hashes a null).
    let hash_of = |name: &str, ty: &kt::KtType| -> String {
        match array_prop(ty) {
            Some(ArrayProp::Bytes { nullable: true }) => {
                format!("({name}?.contentHashCode() ?: 0)")
            }
            Some(ArrayProp::Bytes { nullable: false }) => format!("{name}.contentHashCode()"),
            None if ty.is_nullable() => format!("({name}?.hashCode() ?: 0)"),
            None => format!("{name}.hashCode()"),
        }
    };
    let first = hash_of(&props[0].0, &props[0].1);
    let hash_body = if props.len() == 1 {
        // A single property needs no accumulator — `var result` would draw a
        // "never reassigned" warning in the generated source.
        kt::Code::new().line(format!("return {first}"))
    } else {
        let mut b = kt::Code::new().line(format!("var result = {first}"));
        for (name, ty) in &props[1..] {
            b = b.line(format!("result = 31 * result + {}", hash_of(name, ty)));
        }
        b.line("return result")
    };
    let hash_code = kt::KtFun::new("hashCode")
        .modifier("override")
        .returns(kt::KtType::int())
        .body(hash_body);

    // `toString`: an array would otherwise render as `[B@1a2b3c`.
    let rendered: Vec<String> = props
        .iter()
        .map(|(name, ty)| match array_prop(ty) {
            Some(ArrayProp::Bytes { nullable: true }) => {
                format!("{name}=${{{name}?.contentToString()}}")
            }
            Some(ArrayProp::Bytes { nullable: false }) => {
                format!("{name}=${{{name}.contentToString()}}")
            }
            None => format!("{name}=${name}"),
        })
        .collect();
    let to_string = kt::KtFun::new("toString")
        .modifier("override")
        .returns(kt::KtType::string())
        .expr_body(kt::Code::new().line(format!("\"{class_name}({})\"", rendered.join(", "))));

    Some(vec![equals, hash_code, to_string])
}
