//! Syntax-free plans for converters composed from recipe shapes.
//!
//! A plan keeps Flat [`TypeRef`]s opaque and records only shape operations,
//! wire-side types and child converter contracts. [`RustFunction::render`]
//! receives the writer-owned [`Emit`] after resolution and validation, which is
//! the first point at which the captured Rust types and function bodies are
//! materialized.

use prebindgen_registry::{
    chain::{self, Chain as _},
    flat::{Alternative, TypeRef},
    recipe::Mode,
    write::RustFunction,
    Emit,
};

use super::{builder::qualify_source_type, *};

/// The callable facts a parent chain needs from a child converter.
#[derive(Clone)]
pub(crate) struct CCall(chain::Call);

impl CCall {
    pub(crate) fn ident(&self) -> &syn::Ident {
        self.0.ident()
    }

    pub(crate) fn fallible(&self) -> bool {
        self.0.fallible()
    }

    pub(crate) fn unsafe_(&self) -> bool {
        self.0.unsafe_()
    }
}

impl chain::Child for CCall {
    fn call(&self) -> &chain::Call {
        &self.0
    }

    fn invoke(&self, value: TokenStream) -> TokenStream {
        let ident = self.ident();
        quote!(#ident(#value))
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
    Sequence(SequencePlan),
    Choice(ChoicePlan),
}

impl CFunction {
    pub(crate) fn complete(function: syn::ItemFn) -> Self {
        let call = CCall(chain::Call::complete(&function));
        Self {
            call,
            body: CBody::Complete(function),
        }
    }

    pub(crate) fn product(plan: ProductPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.fields.iter().any(|field| field.converter.fallible()),
            plan.direction == Direction::Construct,
        ));
        Self {
            call,
            body: CBody::Product(plan),
        }
    }

    pub(crate) fn optional(plan: OptionalPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.converter.fallible(),
            true,
        ));
        Self {
            call,
            body: CBody::Optional(plan),
        }
    }

    pub(crate) fn sequence(plan: SequencePlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.child.fallible(),
            plan.child.unsafe_(),
        ));
        Self {
            call,
            body: CBody::Sequence(plan),
        }
    }

    pub(crate) fn choice(plan: ChoicePlan) -> Self {
        let fallible = plan.direction == Direction::Construct
            || plan
                .arms
                .iter()
                .flat_map(|arm| &arm.parts)
                .any(|part| part.child.fallible());
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            fallible,
            plan.direction == Direction::Construct,
        ));
        Self {
            call,
            body: CBody::Choice(plan),
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
            CBody::Choice(plan) => plan.render(emit),
            CBody::Sequence(plan) => plan.render(emit),
            CBody::Optional(plan) => plan.render(emit),
        }
    }
}

/// One field step in a product converter chain.
#[derive(Clone)]
pub(crate) struct ProductField {
    pub(crate) name: syn::Ident,
    pub(crate) converter: CCall,
    pub(crate) mode: Mode,
    pub(crate) hold_uninit: bool,
}

#[derive(Clone)]
struct CSource {
    module: Option<syn::Path>,
}

impl chain::Source for CSource {
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
        qualify_source_type(&emit.spell_ty(source), self.module.as_ref())
    }
}

#[derive(Clone)]
struct CProductBridge {
    wire: syn::Type,
}

impl chain::ProductBridge for CProductBridge {
    fn intermediate(&self) -> syn::Type {
        self.wire.clone()
    }

    fn part(&self, value: TokenStream, _index: usize, name: &syn::Ident) -> TokenStream {
        quote!(#value.#name)
    }

    fn build(&self, parts: &[(syn::Ident, TokenStream)]) -> TokenStream {
        let wire = &self.wire;
        let names = parts.iter().map(|(name, _)| name);
        let values = parts.iter().map(|(_, value)| value);
        quote!(#wire { #(#names: #values),* })
    }
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
        let chain = chain::Product {
            source: self.source.clone(),
            direction: self.direction,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CProductBridge {
                wire: self.wire.clone(),
            },
            parts: self
                .fields
                .iter()
                .map(|field| chain::ProductPart {
                    name: field.name.clone(),
                    child: field.converter.clone(),
                    mode: field.mode,
                    hold_uninit: field.hold_uninit,
                })
                .collect(),
        };
        let rendered = chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;

        match (self.direction, rendered.fallible) {
            (Direction::Construct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #intermediate,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            (Direction::Construct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #intermediate) -> #source {
                    #body
                }
            ),
            (Direction::Deconstruct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(
                    v: #source,
                ) -> ::core::result::Result<#intermediate, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            (Direction::Deconstruct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #intermediate {
                    #body
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

#[derive(Clone)]
struct COptionalBridge {
    wire: syn::Type,
    repr: OptionalRepr,
}

impl chain::OptionalBridge for COptionalBridge {
    fn intermediate(&self) -> syn::Type {
        self.wire.clone()
    }

    fn is_absent(&self) -> TokenStream {
        match &self.repr {
            OptionalRepr::Niche { absent } => quote!(#absent),
            OptionalRepr::Nullable { .. } => quote!((v).is_null()),
        }
    }

    fn present(&self, value: TokenStream) -> TokenStream {
        match self.repr {
            OptionalRepr::Niche { .. } | OptionalRepr::Nullable { read_direct: true } => value,
            OptionalRepr::Nullable { read_direct: false } => quote!(::core::ptr::read(#value)),
        }
    }

    fn build_absent(&self) -> TokenStream {
        unreachable!("optional bridge operation does not match its planned direction")
    }

    fn build_present(&self, _child: TokenStream) -> TokenStream {
        unreachable!("optional bridge operation does not match its planned direction")
    }
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
        let chain = chain::Optional {
            source: self.source.clone(),
            direction: Direction::Construct,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: COptionalBridge {
                wire: self.wire.clone(),
                repr: self.repr.clone(),
            },
            child: self.converter.clone(),
        };
        let rendered = chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let lifetime = self.borrowed.then(|| quote!(<'a>));

        if rendered.fallible {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name #lifetime(v: #intermediate)
                    -> ::core::result::Result<#source, ::std::string::String>
                {
                    ::core::result::Result::Ok(#body)
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name #lifetime(v: #intermediate) -> #source {
                    #body
                }
            )
        }
    }
}

/// A Vec of one child wire for one registry-owned Sequence loop.
#[derive(Clone)]
struct CSequenceBridge {
    child: syn::Type,
}

impl chain::SequenceBridge for CSequenceBridge {
    fn intermediate(&self) -> syn::Type {
        let child = &self.child;
        syn::parse_quote!(::std::vec::Vec<#child>)
    }

    fn begin(&self, _value: TokenStream) -> TokenStream {
        unreachable!("C Sequence input is represented by the specialized slice path")
    }

    fn next(&self) -> TokenStream {
        unreachable!("C Sequence input is represented by the specialized slice path")
    }

    fn begin_output(&self, source: TokenStream) -> TokenStream {
        let child = &self.child;
        quote!(
            let mut __sequence_output: ::std::vec::Vec<#child> =
                ::std::vec::Vec::with_capacity((#source).len());
        )
    }

    fn push(&self, value: TokenStream) -> TokenStream {
        quote!(__sequence_output.push(#value);)
    }

    fn finish(&self) -> TokenStream {
        quote!(__sequence_output)
    }

    fn fallible(&self) -> bool {
        false
    }
}

/// One C Vec-output converter composed by the registry.
#[derive(Clone)]
pub(crate) struct SequencePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) element: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) child_wire: syn::Type,
    pub(crate) child: CCall,
}

impl SequencePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let composed = chain::Sequence {
            source: self.source.clone(),
            element: self.element.clone(),
            direction: Direction::Deconstruct,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CSequenceBridge {
                child: self.child_wire.clone(),
            },
            child: self.child.clone(),
        };
        let rendered = composed.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let unsafe_ = self.child.unsafe_().then(|| quote!(unsafe));
        if rendered.fallible {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                #[inline(always)]
                pub(crate) #unsafe_ fn #name(v: #source)
                    -> ::core::result::Result<#intermediate, ::std::string::String>
                {
                    ::core::result::Result::Ok(#body)
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                #[inline(always)]
                pub(crate) #unsafe_ fn #name(v: #source) -> #intermediate {
                    #body
                }
            )
        }
    }
}
#[derive(Clone)]
struct CChoiceBridge {
    wire: syn::Ident,
    alternatives: Vec<Alternative>,
}

impl chain::ChoiceBridge for CChoiceBridge {
    fn intermediate(&self) -> syn::Type {
        let wire = &self.wire;
        syn::parse_quote!(::core::mem::MaybeUninit<#wire>)
    }

    fn tag(&self, value: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let bounds_msg = format!(
            "`{wire}`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
        quote!({
            const _: () = {
                assert!(
                    ::core::mem::size_of::<#wire>()
                        >= ::core::mem::size_of::<::core::ffi::c_int>(),
                    #bounds_msg
                );
            };
            unsafe {
                ::core::ptr::read((#value).as_ptr() as *const ::core::ffi::c_int)
            }
        })
    }

    fn prepare(&self, value: TokenStream) -> TokenStream {
        quote!(unsafe { (#value).assume_init() })
    }

    fn arm(&self, emit: &Emit, value: TokenStream, index: usize) -> TokenStream {
        let wire = &self.wire;
        let alternative = &self.alternatives[index];
        let variant = &alternative.name;
        let bindings: Vec<_> = alternative
            .fields
            .iter()
            .enumerate()
            .map(|(index, _)| format_ident!("__wire_part{index}"))
            .collect();
        let bound: Vec<_> = alternative
            .fields
            .iter()
            .zip(&bindings)
            .map(|(field, binding)| field.bind(binding))
            .collect();
        let pattern = emit.shape_alternative(alternative, quote!(#wire::#variant), &bound);
        quote!({
            match #value {
                #pattern => (#(#bindings,)*),
                _ => unreachable!("validated Choice tag selected a different arm"),
            }
        })
    }

    fn build(&self, emit: &Emit, active: usize, value: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let alternative = &self.alternatives[active];
        let variant = &alternative.name;
        if alternative.fields.is_empty() {
            return emit.shape_alternative(alternative, quote!(#wire::#variant), &[]);
        }
        let fields: Vec<_> = alternative
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let index = syn::Index::from(index);
                field.bind(&quote!(__built_arm.#index))
            })
            .collect();
        let built = emit.shape_alternative(alternative, quote!(#wire::#variant), &fields);
        quote!({
            let __built_arm = #value;
            #built
        })
    }

    fn finish(&self, value: TokenStream) -> TokenStream {
        quote!(::core::mem::MaybeUninit::new(#value))
    }

    fn invalid_tag(&self, tag: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let variants = self.alternatives.len();
        quote!(::std::format!(
            "invalid tag {} for `{}` (expected 0..{})",
            #tag,
            stringify!(#wire),
            #variants,
        ))
    }
}

/// A tagged-union Choice chain. Raw C storage is represented by its bridge;
/// source syntax remains behind `source` until `render`.
#[derive(Clone)]
pub(crate) struct ChoicePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Ident,
    pub(crate) direction: Direction,
    pub(crate) arms: Vec<chain::ChoiceArm<chain::TupleProduct, CCall>>,
}

impl ChoicePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let composed = chain::Choice {
            source: self.source.clone(),
            direction: self.direction,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CChoiceBridge {
                wire: self.wire.clone(),
                alternatives: self
                    .arms
                    .iter()
                    .map(|arm| arm.alternative.clone())
                    .collect(),
            },
            arms: self.arms.clone(),
        };
        let rendered = composed.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;

        match self.direction {
            Direction::Construct => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #intermediate,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            Direction::Deconstruct => {
                assert!(
                    !rendered.fallible,
                    "fallible C Choice output must be refused while planning"
                );
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source) -> #intermediate {
                        #body
                    }
                )
            }
        }
    }
}
