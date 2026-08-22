//! Syntax-free plans for converters composed from recipe shapes.
//!
//! A plan keeps Flat [`TypeRef`]s opaque and records only shape operations,
//! wire-side types and child converter contracts. [`RustFunction::render`]
//! receives the writer-owned [`Emit`] after resolution and validation, which is
//! the first point at which the captured Rust types and function bodies are
//! materialized.

use prebindgen_registry::{flat::TypeRef, write::RustFunction, Emit};

use super::{builder::qualify_source_type, *};

/// The callable facts a parent chain needs from a child converter.
#[derive(Clone)]
pub(crate) struct CCall {
    ident: syn::Ident,
    fallible: bool,
    unsafe_: bool,
}

impl CCall {
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub(crate) fn fallible(&self) -> bool {
        self.fallible
    }

    pub(crate) fn unsafe_(&self) -> bool {
        self.unsafe_
    }
}

/// A complete legacy function or a chain waiting for final rendering.
#[derive(Clone)]
pub(crate) struct CFunction {
    call: CCall,
    body: CBody,
}

#[derive(Clone)]
enum CBody {
    Complete(syn::ItemFn),
    Product(ProductPlan),
    Optional(OptionalPlan),
}

impl CFunction {
    pub(crate) fn complete(function: syn::ItemFn) -> Self {
        let call = CCall {
            ident: function.sig.ident.clone(),
            fallible: returns_result(&function.sig.output),
            unsafe_: function.sig.unsafety.is_some(),
        };
        Self {
            call,
            body: CBody::Complete(function),
        }
    }

    pub(crate) fn product(plan: ProductPlan) -> Self {
        let call = CCall {
            ident: plan.ident.clone(),
            fallible: plan.fields.iter().any(|field| field.converter.fallible),
            unsafe_: plan.direction == Direction::Construct,
        };
        Self {
            call,
            body: CBody::Product(plan),
        }
    }

    pub(crate) fn optional(plan: OptionalPlan) -> Self {
        let call = CCall {
            ident: plan.ident.clone(),
            fallible: plan.converter.fallible,
            unsafe_: true,
        };
        Self {
            call,
            body: CBody::Optional(plan),
        }
    }

    pub(crate) fn call(&self) -> &CCall {
        &self.call
    }
}

impl RustFunction for CFunction {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.body {
            CBody::Complete(function) => function.clone(),
            CBody::Product(plan) => plan.render(emit),
            CBody::Optional(plan) => plan.render(emit),
        }
    }
}

/// One field step in a product converter chain.
#[derive(Clone)]
pub(crate) struct ProductField {
    pub(crate) name: syn::Ident,
    pub(crate) converter: CCall,
    pub(crate) hold_uninit: bool,
}

/// A product chain. Source syntax stays behind `source` until `render`.
#[derive(Clone)]
pub(crate) struct ProductPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) direction: Direction,
    pub(crate) fields: Vec<ProductField>,
}

impl ProductPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let wire = &self.wire;
        let names: Vec<_> = self.fields.iter().map(|field| &field.name).collect();
        let calls: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let fname = &field.name;
                let converter = field.converter.ident();
                let mut call = quote!(#converter(v.#fname));
                if field.converter.fallible() {
                    call = quote!(#call?);
                }
                if field.hold_uninit {
                    call = quote!(::core::mem::MaybeUninit::new(#call));
                }
                call
            })
            .collect();
        let fallible = self.fields.iter().any(|field| field.converter.fallible());

        match (self.direction, fallible) {
            (Direction::Construct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #wire,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    ::core::result::Result::Ok(#source { #(#names: #calls),* })
                }
            ),
            (Direction::Construct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #wire) -> #source {
                    #source { #(#names: #calls),* }
                }
            ),
            (Direction::Deconstruct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(
                    v: #source,
                ) -> ::core::result::Result<#wire, ::std::string::String> {
                    ::core::result::Result::Ok(#wire { #(#names: #calls),* })
                }
            ),
            (Direction::Deconstruct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    #wire { #(#names: #calls),* }
                }
            ),
        }
    }
}

/// How an inbound optional distinguishes absence from a present child.
#[derive(Clone)]
pub(crate) enum OptionalRepr {
    Niche { absent: syn::Expr },
    Nullable { read_direct: bool },
}

/// An optional chain. Its source `Option<T>` remains opaque until rendering.
#[derive(Clone)]
pub(crate) struct OptionalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) converter: CCall,
    pub(crate) repr: OptionalRepr,
    pub(crate) borrowed: bool,
}

impl OptionalPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let wire = &self.wire;
        let converter = self.converter.ident();
        let lifetime = self.borrowed.then(|| quote!(<'a>));

        match (&self.repr, self.converter.fallible()) {
            (OptionalRepr::Niche { absent }, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #wire)
                    -> ::core::result::Result<#source, ::std::string::String>
                {
                    if #absent {
                        ::core::result::Result::Ok(::core::option::Option::None)
                    } else {
                        #converter(v).map(::core::option::Option::Some)
                    }
                }
            ),
            (OptionalRepr::Niche { absent }, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #wire) -> #source {
                    if #absent {
                        ::core::option::Option::None
                    } else {
                        ::core::option::Option::Some(#converter(v))
                    }
                }
            ),
            (OptionalRepr::Nullable { read_direct }, true) => {
                let read = if *read_direct {
                    quote!(v)
                } else {
                    quote!(::core::ptr::read(v))
                };
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lifetime(v: #wire)
                        -> ::core::result::Result<#source, ::std::string::String>
                    {
                        if v.is_null() {
                            return ::core::result::Result::Ok(::core::option::Option::None);
                        }
                        match #converter(#read) {
                            ::core::result::Result::Ok(__x) => {
                                ::core::result::Result::Ok(::core::option::Option::Some(__x))
                            }
                            ::core::result::Result::Err(__e) => ::core::result::Result::Err(__e),
                        }
                    }
                )
            }
            (OptionalRepr::Nullable { read_direct }, false) => {
                let read = if *read_direct {
                    quote!(v)
                } else {
                    quote!(::core::ptr::read(v))
                };
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lifetime(v: #wire) -> #source {
                        if v.is_null() {
                            ::core::option::Option::None
                        } else {
                            ::core::option::Option::Some(#converter(#read))
                        }
                    }
                )
            }
        }
    }
}
