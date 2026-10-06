//! The C adapter's wire types: everything a generated C function, struct
//! field or callback argument can be.

use prebindgen_flat::flat::ScalarKind;
use prebindgen_tools::{names, WireType};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug)]
pub(crate) enum CWire {
    /// A Rust scalar C shares as is (`i32`, `f64`, `usize`, …).
    Scalar(ScalarKind),
    /// A `bool` C receives: Rust writes it, so it is valid.
    Bool,
    /// A `bool` C writes: held as `MaybeUninit<bool>`, its byte checked.
    BoolSlot,
    /// A C `char`, behind a pointer: a NUL-terminated string.
    Char,
    /// A generated struct by value: a mirror, a `repr_c_struct`, a closure
    /// struct, or the opaque struct a handle points to.
    Struct(syn::Ident),
    /// A generated enum C writes: held as `MaybeUninit`, its tag checked.
    Uninit(syn::Ident),
    /// A pointer to another wire.
    Ptr { mutable: bool, to: Box<CWire> },
}

impl CWire {
    pub(crate) fn ptr(mutable: bool, to: CWire) -> Self {
        CWire::Ptr {
            mutable,
            to: Box::new(to),
        }
    }

    /// `*const c_char` / `*mut c_char`.
    pub(crate) fn c_str(mutable: bool) -> Self {
        Self::ptr(mutable, CWire::Char)
    }
}

impl WireType for CWire {
    fn rust(&self) -> TokenStream {
        match self {
            CWire::Scalar(k) => {
                let id = names::ident(k.as_str());
                quote!(#id)
            }
            CWire::Bool => quote!(bool),
            CWire::BoolSlot => quote!(::core::mem::MaybeUninit<bool>),
            CWire::Char => quote!(::core::ffi::c_char),
            CWire::Struct(c) => quote!(#c),
            CWire::Uninit(c) => quote!(::core::mem::MaybeUninit<#c>),
            CWire::Ptr { mutable, to } => {
                let to = to.rust();
                if *mutable {
                    quote!(*mut #to)
                } else {
                    quote!(*const #to)
                }
            }
        }
    }

    fn placeholder(&self) -> TokenStream {
        match self {
            CWire::Scalar(ScalarKind::Bool) | CWire::Bool => quote!(false),
            CWire::Scalar(ScalarKind::F32 | ScalarKind::F64) => quote!(0.0),
            CWire::Scalar(_) | CWire::Char => quote!(0),
            CWire::BoolSlot => quote!(::core::mem::MaybeUninit::new(false)),
            CWire::Uninit(_) => quote!(::core::mem::MaybeUninit::uninit()),
            CWire::Struct(_) => quote!(unsafe { ::core::mem::zeroed() }),
            CWire::Ptr { mutable: true, .. } => quote!(::core::ptr::null_mut()),
            CWire::Ptr { mutable: false, .. } => quote!(::core::ptr::null()),
        }
    }
}
