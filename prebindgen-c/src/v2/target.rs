//! The C target, as the v2 engine calls it: the declarations its wire types need.
//!
//! Everything a C binding decides — which wire type a type crosses in, which
//! operations move it, what symbol a wrapper exports — is stated as data by
//! [`super`] when the binding is built, and the registry plans from that alone.
//! What is left here is writing: a `repr(C)` mirror of a struct, the
//! incomplete type a handle points to, the enum C sees for a Rust one, the
//! closure struct a callback arrives in — and the one operation of its own,
//! calling through that closure.

use prebindgen_registry_v2::{
    mirrored_i32_enum, OperationFeed, Target, WireKind, WireType, WireTypeFeed, Written,
};
use quote::{format_ident, quote};

/// The kinds of C wire type, which is what C's capabilities are stated in:
/// what a struct can have as members, what a callback can take.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CWireKind {
    /// A 64-bit integer: the one scalar this target carries so far.
    I64,
    /// An address: a handle to a Rust-owned value.
    Pointer,
    /// A `repr(C)` struct passed by value.
    Aggregate,
    /// A C enum.
    Enum,
    /// The storage a C enum arrives in: `MaybeUninit` of it, since C lets an
    /// enum variable hold any `int`.
    EnumBits,
    /// A closure struct: a context, a function to call with it, and one to
    /// drop it.
    Closure,
}

impl WireKind for CWireKind {
    const ALL: &'static [Self] = &[
        CWireKind::I64,
        CWireKind::Pointer,
        CWireKind::Aggregate,
        CWireKind::Enum,
        CWireKind::EnumBits,
        CWireKind::Closure,
    ];

    fn name(self) -> &'static str {
        match self {
            CWireKind::I64 => "i64",
            CWireKind::Pointer => "pointer",
            CWireKind::Aggregate => "aggregate",
            CWireKind::Enum => "enum",
            CWireKind::EnumBits => "enum_bits",
            CWireKind::Closure => "closure",
        }
    }

    /// A struct's members are scalars — not yet another struct, a handle or an
    /// enum. A callback's arguments are what leaves Rust in a register: a
    /// scalar, an address, an enum; a struct leaving Rust has no construction
    /// yet.
    fn parts(self) -> &'static [Self] {
        match self {
            CWireKind::Aggregate => &[CWireKind::I64],
            CWireKind::Closure => &[CWireKind::I64, CWireKind::Pointer, CWireKind::Enum],
            _ => &[],
        }
    }
}

/// A C wire type: its kind, and for every kind but the scalar the name of the
/// type this target declares for it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CWireType {
    /// `i64`, which C already has as `int64_t`.
    I64,
    /// `*mut <name>`: a pointer to the incomplete type a handle is.
    Pointer { name: syn::Ident },
    /// A `repr(C)` struct mirroring the source struct, member for field.
    Aggregate { name: syn::Ident },
    /// A C enum of the same values as the source enum.
    Enum { name: syn::Ident },
    /// `MaybeUninit<name>`: the storage the C enum `name` arrives in. Declared
    /// by the enum it holds.
    EnumBits { name: syn::Ident },
    /// The closure struct a callback arrives in, its `call` taking the
    /// callback's arguments' wire types.
    Closure { name: syn::Ident },
}

impl WireType for CWireType {
    type Kind = CWireKind;

    fn kind(&self) -> CWireKind {
        match self {
            CWireType::I64 => CWireKind::I64,
            CWireType::Pointer { .. } => CWireKind::Pointer,
            CWireType::Aggregate { .. } => CWireKind::Aggregate,
            CWireType::Enum { .. } => CWireKind::Enum,
            CWireType::EnumBits { .. } => CWireKind::EnumBits,
            CWireType::Closure { .. } => CWireKind::Closure,
        }
    }

    fn rust(&self) -> syn::Type {
        match self {
            CWireType::I64 => syn::parse_quote!(i64),
            CWireType::Pointer { name } => syn::parse_quote!(*mut #name),
            CWireType::Aggregate { name }
            | CWireType::Enum { name }
            | CWireType::Closure { name } => syn::parse_quote!(#name),
            CWireType::EnumBits { name } => syn::parse_quote!(::core::mem::MaybeUninit<#name>),
        }
    }
}

/// The one operation C writes itself. Reading an aggregate member, and going
/// between a source enum and the C one declared for it, are standard
/// operations the registry writes — it is the registry that can spell a source
/// path.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum COp {
    /// Call a closure struct's `call` with the arguments and its context. A
    /// closure with no `call` is one C asked to be called and not told about,
    /// so nothing happens.
    Call,
}

/// The C target: its writers.
#[derive(Default)]
pub struct CTarget;

impl Target for CTarget {
    const NAME: &'static str = "c";

    type WireType = CWireType;
    type Op = COp;
    /// A C declaration is the Rust cbindgen reads, so nothing about an output
    /// is for a foreign writer alone.
    type OutputMeta = ();

    fn write_operation(&self, op: &COp, feed: &OperationFeed<'_, Self>) -> Written {
        match op {
            COp::Call => {
                let (closure, _) = feed
                    .value
                    .as_ref()
                    .expect("a call is applied to the closure struct");
                let args = feed.args.iter().map(|(name, _)| name);
                // Borrowed whole before its fields are read: a closure
                // capturing `.context` alone would hold a raw pointer, which
                // is neither `Send` nor `Sync`, where the struct is both.
                Written::new(quote! {
                    {
                        let closure = &#closure;
                        if let ::core::option::Option::Some(call) = closure.call {
                            unsafe { call(#(#args,)* closure.context) }
                        }
                    }
                })
            }
        }
    }

    fn write_wire_type(&self, feed: &WireTypeFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
        match feed.wire_type {
            CWireType::I64 | CWireType::EnumBits { .. } => Vec::new(),
            // What a C caller fills in to be called back: its own context, the
            // function to call with each argument's wire type and that context,
            // and the function that frees the context once Rust drops the
            // closure. Rust may call and drop it from any thread, which is the
            // contract a C caller signs by passing one — hence the two unsafe
            // impls, which the closure Rust builds needs.
            CWireType::Closure { name: ident } => {
                let args = feed.parts.iter().map(|(_, wire_type)| wire_type.rust());
                vec![
                    // What each member means is said on the member, which is
                    // where `cbindgen` puts it in the header: a C caller never
                    // reads this Rust.
                    quote! {
                        #[repr(C)]
                        #[allow(non_camel_case_types)]
                        pub struct #ident {
                            /// The caller's own state, handed to `call` and to `drop`.
                            pub context: *mut ::core::ffi::c_void,
                            /// Called on every call of the callback, with its arguments
                            /// and `context`, from whichever thread Rust calls it on. When
                            /// null, a call does nothing.
                            pub call: ::core::option::Option<
                                unsafe extern "C" fn(#(#args,)* *mut ::core::ffi::c_void),
                            >,
                            /// Called once with `context` when Rust lets go of the
                            /// callback. When null, nothing frees `context`.
                            pub drop: ::core::option::Option<
                                unsafe extern "C" fn(*mut ::core::ffi::c_void),
                            >,
                        }
                    },
                    quote!(unsafe impl ::core::marker::Send for #ident {}),
                    quote!(unsafe impl ::core::marker::Sync for #ident {}),
                    quote! {
                        impl ::core::ops::Drop for #ident {
                            fn drop(&mut self) {
                                if let ::core::option::Option::Some(drop) = self.drop {
                                    unsafe { drop(self.context) }
                                }
                            }
                        }
                    },
                ]
            }
            // A struct whose only member is a zero-length array is what
            // `cbindgen` renders as an incomplete type: a C caller can hold a
            // pointer to one and nothing else. The same declaration v1 emits,
            // case lint included.
            CWireType::Pointer { name: ident } => {
                vec![quote! {
                    #[repr(C)]
                    #[allow(non_camel_case_types)]
                    pub struct #ident {
                        _private: [u8; 0],
                    }
                }]
            }
            // `repr(C)` is required: without it the layout the header promises
            // is not the layout the wrapper reads. The C name is the mangler's —
            // `foo_t` — so the case lint is silenced as v1 silences it.
            CWireType::Aggregate { name: ident } => {
                let c_name = ident.to_string();
                let members = feed.parts.iter().map(|(part, wire_type)| {
                    let name = format_ident!(
                        "{}",
                        part.name
                            .as_ref()
                            .expect("a positional field is refused when the binding is built")
                    );
                    let ty = wire_type.rust();
                    // A member mirrors a field one for one, its condition
                    // included: a field the source crate may not have must not
                    // become a member the header always declares.
                    let condition = &part.conditions;
                    for under in condition {
                        warn_undefined_condition(&c_name, &name, under);
                    }
                    quote!(#(#condition)* pub #name: #ty)
                });
                vec![quote! {
                    #[repr(C)]
                    #[allow(non_camel_case_types)]
                    pub struct #ident { #(#members),* }
                }]
            }
            // The C enum itself: the same values under the same names, and the
            // numbers Rust assigns, so a C caller reading the header sees what a
            // Rust caller sees.
            CWireType::Enum { name: ident } => {
                let c_name = ident.to_string();
                let values = mirrored_i32_enum(feed.unit, &c_name, Self::NAME)
                    .expect("an enum C cannot mirror is refused when the binding is built");
                let size_message = format!("`{c_name}` is not the size of a C `int`");
                let values = values.iter().map(|(value, number)| {
                    let name = &value.name;
                    let number = proc_macro2::Literal::i32_unsuffixed(*number);
                    quote!(#name = #number)
                });
                vec![
                    quote! {
                        #[repr(C)]
                        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
                        #[allow(non_camel_case_types)]
                        pub enum #ident {
                            #(#values),*
                        }
                    },
                    // A value coming into Rust is read as a C `int`, which is
                    // only its bits where the two are one size — not on a
                    // target whose C enums are narrower. An item of its own, so
                    // the enum's condition reaches it too.
                    quote! {
                        const _: () = assert!(
                            ::core::mem::size_of::<#ident>()
                                == ::core::mem::size_of::<::core::ffi::c_int>(),
                            #size_message
                        );
                    },
                ]
            }
        }
    }
}

/// Say, in the build log, that a member written under a condition needs the
/// consumer's cbindgen configuration to know that condition.
///
/// cbindgen guards a member only for a condition its `[defines]` table names;
/// with no entry it writes the member unguarded and says nothing, because its
/// warning goes through `log` and a build script driving its library API
/// installs no logger. The header then declares a member the library may not
/// have, which no compiler or linker catches — the caller and the library
/// simply disagree about the struct's size. This line is the only output such a
/// build produces, so it names the fix; it cannot tell whether the fix is
/// already in place, since reading the consumer's cbindgen configuration would
/// cost a dependency for a warning.
fn warn_undefined_condition(c_name: &str, member: &syn::Ident, under: &proc_macro2::TokenStream) {
    // Tokens print with a space between each pair, which `close_up` removes
    // where it separates no two words — the same treatment a type key gets in
    // the report.
    let under = prebindgen_registry::close_up(&under.to_string());
    println!(
        "cargo:warning=prebindgen: `{c_name}.{member}` is emitted under {under}; unless your \
         cbindgen configuration already maps that condition in [defines], the header declares \
         the member unconditionally and a C caller disagrees with the library about the layout \
         of `{c_name}`"
    );
}
