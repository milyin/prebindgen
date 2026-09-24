//! The C target, as the v2 engine calls it: the declarations its carriers need.
//!
//! Everything a C binding decides — which carrier a type crosses in, which
//! operations move it, what symbol a wrapper exports — is stated as data by
//! [`super`] when the binding is built, and the registry plans from that alone.
//! What is left here is writing: a `repr(C)` mirror of a struct, the
//! incomplete type a handle points to, the enum C sees for a Rust one, the
//! closure struct a callback arrives in — and the one operation of its own,
//! calling through that closure.

use prebindgen_registry_v2::{mirrored_i32_enum, CarrierFeed, OperationFeed, Target, Written};
use quote::{format_ident, quote};

/// The C wire types a binding's carriers are, which is what a struct's members
/// and a wrapper's parameters are allowed to be stated in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CClass {
    /// A 64-bit integer: the one scalar this target carries so far.
    I64,
    /// An address: a handle to a Rust-owned value.
    Pointer,
    /// A `repr(C)` struct passed by value.
    Aggregate,
    /// A C enum, or the storage one arrives in.
    Enum,
    /// A closure struct: a context, a function to call with it, and one to
    /// drop it.
    Closure,
}

impl CClass {
    /// Every class: what a wrapper parameter or return may be.
    pub(crate) fn all() -> [CClass; 5] {
        [
            CClass::I64,
            CClass::Pointer,
            CClass::Aggregate,
            CClass::Enum,
            CClass::Closure,
        ]
    }
}

/// What a C carrier needs declared, if anything, and under which C name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CCarrier {
    /// A type Rust and C already share: `i64` is `int64_t`.
    Builtin,
    /// A `repr(C)` struct mirroring the source struct, member for field.
    Aggregate { c_name: String },
    /// The incomplete type a handle points to: C holds a `<c_name> *` and
    /// nothing else.
    Opaque { c_name: String },
    /// A C enum of the same values as the source enum.
    Enum { c_name: String },
    /// The storage a C enum arrives in — `MaybeUninit` of it, since C lets an
    /// enum variable hold any `int`. Declared by the enum it holds.
    EnumBits,
    /// The closure struct a callback arrives in, its `call` taking the
    /// callback's arguments' carriers.
    Closure { c_name: String },
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

    type WireClass = CClass;
    type CarrierMeta = CCarrier;
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

    fn write_carrier(&self, feed: &CarrierFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
        match &feed.carrier.meta {
            CCarrier::Builtin | CCarrier::EnumBits => Vec::new(),
            // What a C caller fills in to be called back: its own context, the
            // function to call with each argument's carrier and that context,
            // and the function that frees the context once Rust drops the
            // closure. Rust may call and drop it from any thread, which is the
            // contract a C caller signs by passing one — hence the two unsafe
            // impls, which the closure Rust builds needs.
            CCarrier::Closure { c_name } => {
                let ident = format_ident!("{c_name}");
                let args = feed.members.iter().map(|(_, carrier)| &carrier.rust);
                vec![
                    quote! {
                        #[repr(C)]
                        #[allow(non_camel_case_types)]
                        pub struct #ident {
                            pub context: *mut ::core::ffi::c_void,
                            pub call: ::core::option::Option<
                                unsafe extern "C" fn(#(#args,)* *mut ::core::ffi::c_void),
                            >,
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
            CCarrier::Opaque { c_name } => {
                let ident = format_ident!("{c_name}");
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
            CCarrier::Aggregate { c_name } => {
                let ident = format_ident!("{c_name}");
                let members = feed.members.iter().map(|(part, carrier)| {
                    let name = format_ident!(
                        "{}",
                        part.name
                            .as_ref()
                            .expect("a positional field is refused when the binding is built")
                    );
                    let ty = &carrier.rust;
                    // A member mirrors a field one for one, its condition
                    // included: a field the source crate may not have must not
                    // become a member the header always declares.
                    let condition = &part.conditions;
                    for under in condition {
                        warn_undefined_condition(c_name, &name, under);
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
            CCarrier::Enum { c_name } => {
                let values = mirrored_i32_enum(feed.unit, c_name, Self::NAME)
                    .expect("an enum C cannot mirror is refused when the binding is built");
                let ident = format_ident!("{c_name}");
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
