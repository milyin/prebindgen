//! Leaf conversions for values the adapter copies across the boundary.
//! [`Value::of`](crate::value::Value::of) classifies a source type;
//! [`input`](crate::value::input) and [`output`](crate::value::output) build
//! the conversion around the adapter's own decode and encode expressions.
use crate::{Condition, InputConversion, OutputConversion, Wire};
use prebindgen_flat::{type_key, Primitive, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// A source type that has a by-value carrier: owned, or behind a shared reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    /// A Rust primitive.
    Scalar(Primitive),
    /// `String`, `&String` or `&str`.
    Text,
    /// `Vec<u8>`, `&Vec<u8>` or `&[u8]`.
    Bytes,
}
impl Value {
    /// Classify `ty`; `None` means it is not a value type, including `&mut` views.
    ///
    /// ```
    /// use prebindgen_tools::value::Value;
    /// assert_eq!(Value::of(&syn::parse_quote!(&str)), Some(Value::Text));
    /// assert_eq!(Value::of(&syn::parse_quote!(&mut String)), None);
    /// ```
    pub fn of(ty: &Type) -> Option<Self> {
        let (core, borrowed) = match ty {
            Type::Reference(r) if r.mutability.is_none() => (&*r.elem, true),
            Type::Reference(_) => return None,
            _ => (ty, false),
        };
        if let Some(p) = Primitive::from_type(core) {
            return Some(Self::Scalar(p));
        }
        match type_key(core).as_str() {
            "String" => Some(Self::Text),
            "str" if borrowed => Some(Self::Text),
            "Vec < u8 >" => Some(Self::Bytes),
            "[u8]" if borrowed => Some(Self::Bytes),
            _ => None,
        }
    }
}

/// Decode one wire into the value type `ty` during validation.
///
/// `decode` runs only on the active path and may use `?`. It may yield the owned
/// value or, for text and bytes, a view (`&str`, `&[u8]`): the generated code
/// converts it with `Into` for owned targets and `AsRef` for views, so a view
/// target borrows without copying. `ty` must satisfy [`Value::of`].
///
/// ```
/// use prebindgen_tools::{value, Condition};
/// let input = value::input(
///     &syn::parse_quote!(&str), syn::parse_quote!(raw), syn::parse_quote!(text),
///     &Condition::default(), quote::quote!(read(raw)?), "buffer",
/// )?;
/// assert_eq!(input.wires[0].kind, "buffer");
/// assert!(input.acquire.is_empty() && input.resources.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn input<W>(
    ty: &Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
    decode: TokenStream,
    kind: W,
) -> Result<InputConversion<W>> {
    if Value::of(ty).is_none() {
        return Err(format!("`{}` is not a value type", type_key(ty)));
    }
    let condition = active.rust();
    let taken = quote!(#v.expect("active value"));
    let (statements, argument) = match ty {
        Type::Reference(r) if matches!(type_key(&r.elem).as_str(), "str" | "[u8]") => {
            let target = &r.elem;
            (
                quote!(let #v = #taken;),
                quote!(::core::convert::AsRef::<#target>::as_ref(&#v)),
            )
        }
        Type::Reference(r) => {
            let owned = &r.elem;
            (
                quote!(let #v: #owned = ::core::convert::Into::into(#taken);),
                quote!(&#v),
            )
        }
        _ => (
            quote!(let #v: #ty = ::core::convert::Into::into(#taken);),
            quote!(#v),
        ),
    };
    Ok(InputConversion {
        wires: vec![Wire {
            name: w,
            kind,
            active: active.clone(),
        }],
        validation: quote!(let #v = if #condition {Some(#decode)} else {None};),
        acquire: quote!(),
        resources: vec![],
        statements,
        argument,
    })
}

/// Deliver an owned value type. A borrowed `value` is copied first: scalars by
/// dereference, text and bytes by `Clone`.
///
/// `encode` maps the owned Rust value to the delivered expression. The adapter
/// may adjust the result, for example to defer an allocation's transfer to `commit`.
pub fn output<W>(
    ty: &Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    encode: impl FnOnce(TokenStream) -> TokenStream,
    kind: W,
) -> Result<OutputConversion<W>> {
    let Some(class) = Value::of(ty).filter(|_| !matches!(ty, Type::Reference(_))) else {
        return Err(format!("`{}` is not an owned value type", type_key(ty)));
    };
    let mut statements = TokenStream::new();
    let value = match (borrowed, class) {
        (false, _) => value,
        // Bound first: dereferencing `&place` in place is `clippy::deref_addrof`.
        (true, Value::Scalar(_)) => {
            statements.extend(quote!(let #w: &#ty = #value;));
            quote!(*#w)
        }
        (true, _) => quote!(::core::clone::Clone::clone(#value)),
    };
    let expr = encode(value);
    statements.extend(quote!(let #w = #expr;));
    Ok(OutputConversion {
        wires: vec![Wire {
            name: w.clone(),
            kind,
            active: Condition::default(),
        }],
        statements,
        values: vec![quote!(#w)],
        commit: quote!(),
    })
}
