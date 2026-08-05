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

/// Whether a Kotlin type carries an array anywhere inside it, and therefore
/// compares by identity unless the generated operators dig in.
///
/// Recursive, because a container of arrays is just as identity-compared as a
/// bare one: `List<ByteArray>` (from `Vec<Vec<u8>>`) inherits `ByteArray`'s
/// `equals`, so two lists of equal chunks are unequal and `toString` renders
/// `[[B@3830f1c0]`. A container of *classes* is fine — those already compare
/// by value — so only an array at the bottom makes a property array-bearing.
fn array_bearing(ty: &kt::KtType) -> bool {
    match ty {
        kt::KtType::Named { fqn, args, .. } => {
            is_kotlin_array(fqn.rsplit('.').next().unwrap_or(fqn)) || args.iter().any(array_bearing)
        }
        kt::KtType::Function { .. } => false,
    }
}

/// Every Kotlin primitive array. All of them compare by identity, so a
/// fixed-size Rust array field needs the content operators whatever its element
/// type is — not only `[u8; N]`.
fn is_kotlin_array(name: &str) -> bool {
    crate::jni::wire_access::kotlin_array_descriptor(name).is_some()
}

/// The element type of a single-argument container (`List<T>` -> `T`).
fn element_of(ty: &kt::KtType) -> Option<&kt::KtType> {
    match ty {
        kt::KtType::Named { args, .. } if args.len() == 1 => Some(&args[0]),
        _ => None,
    }
}

/// `a == b` for one value of type `ty`, digging through containers.
///
/// `a`/`b` are Kotlin expressions. Nullability is handled per level: a
/// `ByteArray?` rides `contentEquals`'s nullable-receiver overload, while a
/// nullable container needs an explicit both-null / both-present test.
fn eq_expr(a: &str, b: &str, ty: &kt::KtType) -> String {
    if !array_bearing(ty) {
        return format!("{a} == {b}");
    }
    if element_of(ty).is_none() {
        // The array itself.
        return format!("{a}.contentEquals({b})");
    }
    let elem = element_of(ty).expect("checked above");
    let inner = eq_expr("__x", "__y", elem);
    let cmp = format!(
        "{a}.size == {b}.size && {a}.indices.all {{ __i -> \
         val __x = {a}[__i]; val __y = {b}[__i]; {inner} }}"
    );
    if ty.is_nullable() {
        format!("(({a} == null && {b} == null) || ({a} != null && {b} != null && {cmp}))")
    } else {
        format!("({cmp})")
    }
}

/// `hashCode` for one value of type `ty`, digging through containers. The
/// container fold mirrors `Arrays.hashCode`'s 31-multiplier so a `List` and the
/// array it came from agree.
fn hash_expr(x: &str, ty: &kt::KtType) -> String {
    if !array_bearing(ty) {
        return if ty.is_nullable() {
            format!("({x}?.hashCode() ?: 0)")
        } else {
            format!("{x}.hashCode()")
        };
    }
    match element_of(ty) {
        None if ty.is_nullable() => format!("({x}?.contentHashCode() ?: 0)"),
        None => format!("{x}.contentHashCode()"),
        Some(elem) => {
            let inner = hash_expr("__e", elem);
            let fold = format!("{x}.fold(1) {{ __acc, __e -> 31 * __acc + {inner} }}");
            if ty.is_nullable() {
                format!("({x}?.let {{ __l -> {} }} ?: 0)", fold.replace(x, "__l"))
            } else {
                format!("({fold})")
            }
        }
    }
}

/// `toString` rendering for one value of type `ty`, digging through containers
/// so a nested array never prints as `[B@1a2b3c`.
fn str_expr(x: &str, ty: &kt::KtType) -> String {
    if !array_bearing(ty) {
        return format!("${{{x}}}");
    }
    match element_of(ty) {
        None if ty.is_nullable() => format!("${{{x}?.contentToString()}}"),
        None => format!("${{{x}.contentToString()}}"),
        Some(elem) => {
            let inner = str_expr("__e", elem);
            let join = format!("{x}.joinToString(\", \", \"[\", \"]\") {{ __e -> \"{inner}\" }}");
            if ty.is_nullable() {
                format!("${{{x}?.let {{ __l -> {} }}}}", join.replace(x, "__l"))
            } else {
                format!("${{{join}}}")
            }
        }
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
    if !props.iter().any(|(_, ty)| array_bearing(ty)) {
        return None;
    }

    // `equals`: identity short-circuit, type check, then per-property
    // comparison that digs through any container down to the arrays.
    let comparisons: Vec<String> = props
        .iter()
        .map(|(name, ty)| eq_expr(name, &format!("other.{name}"), ty))
        .collect();
    let equals_body = kt::Code::new()
        .line("if (this === other) return true")
        .line(format!("if (other !is {class_name}) return false"))
        .line(format!("return {}", comparisons.join(" && ")));
    let equals = kt::KtFun::new("equals")
        .modifier("override")
        .param(kt::KtParam::new("other", kt::KtType::any().nullable()))
        .returns(kt::KtType::boolean())
        .body(equals_body);

    // `hashCode`: the standard 31-multiplier fold over the same per-property
    // expressions, so equal values always agree.
    let first = hash_expr(&props[0].0, &props[0].1);
    let hash_body = if props.len() == 1 {
        // A single property needs no accumulator — `var result` would draw a
        // "never reassigned" warning in the generated source.
        kt::Code::new().line(format!("return {first}"))
    } else {
        let mut b = kt::Code::new().line(format!("var result = {first}"));
        for (name, ty) in &props[1..] {
            b = b.line(format!("result = 31 * result + {}", hash_expr(name, ty)));
        }
        b.line("return result")
    };
    let hash_code = kt::KtFun::new("hashCode")
        .modifier("override")
        .returns(kt::KtType::int())
        .body(hash_body);

    // `toString`: an array at any depth would otherwise render as `[B@1a2b3c`.
    let rendered: Vec<String> = props
        .iter()
        .map(|(name, ty)| format!("{name}={}", str_expr(name, ty)))
        .collect();
    let to_string = kt::KtFun::new("toString")
        .modifier("override")
        .returns(kt::KtType::string())
        .expr_body(kt::Code::new().line(format!("\"{class_name}({})\"", rendered.join(", "))));

    Some(vec![equals, hash_code, to_string])
}
