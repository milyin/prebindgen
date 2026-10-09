use std::fmt;

use proc_macro2::TokenStream;
use quote::quote;

/// The adapter's closed set of boundary types — one enum per adapter.
///
/// A variant may say more than its Rust type does: JNI's `jlong` is a Kotlin
/// `Long`, a `ULong` or a handle of some class, and each is its own variant.
/// The foreign-side writer reads that meaning off the wire, so it never has to
/// repeat a decision that an override or a default made for it.
pub trait WireType: Clone + fmt::Debug {
    /// The Rust type of a slot of this kind in the generated file.
    fn rust(&self) -> TokenStream;
    /// The value a slot holds when it carries nothing: an absent optional,
    /// an alternative of a sum that was not taken.
    fn placeholder(&self) -> TokenStream;
}

/// One named slot of the generated boundary.
#[derive(Clone, Debug)]
pub struct Wire<W> {
    pub name: syn::Ident,
    pub ty: W,
}

impl<W: WireType> Wire<W> {
    pub fn new(name: syn::Ident, ty: W) -> Self {
        Self { name, ty }
    }

    /// `name: RustType`, for a parameter list or a struct body.
    pub fn decl(&self) -> TokenStream {
        let (n, t) = (&self.name, self.ty.rust());
        quote!(#n: #t)
    }
}
