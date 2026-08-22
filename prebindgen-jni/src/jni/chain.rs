//! JniGen representation bridges for registry-composed converter chains.

use prebindgen_registry::{
    chain::{self as shared, Chain as _},
    flat::TypeRef,
    write::RustFunction,
    Emit,
};

use super::*;

/// A complete legacy JNI converter or a registry-composed late plan.
#[derive(Clone)]
pub(crate) struct JFunction {
    call: shared::Call,
    body: JBody,
}

#[derive(Clone)]
enum JBody {
    Complete(Box<syn::ItemFn>),
    Optional(Box<JOptionalPlan>),
}

impl JFunction {
    pub(crate) fn complete(function: syn::ItemFn) -> Self {
        Self {
            call: shared::Call::complete(&function),
            body: JBody::Complete(Box::new(function)),
        }
    }

    pub(crate) fn optional(plan: JOptionalPlan) -> Self {
        Self {
            call: shared::Call::new(plan.ident.clone(), true, true),
            body: JBody::Optional(Box::new(plan)),
        }
    }

    pub(crate) fn call(&self) -> &shared::Call {
        &self.call
    }
    pub(crate) fn is_planned(&self) -> bool {
        matches!(self.body, JBody::Optional(_))
    }
}

impl RustFunction for JFunction {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.body {
            JBody::Complete(function) => (**function).clone(),
            JBody::Optional(plan) => plan.render(emit),
        }
    }
}

/// Source spelling plus the transparent wrappers surrounding its Optional.
#[derive(Clone)]
pub(crate) struct JSource {
    pub(crate) wrappers: Vec<&'static str>,
}

impl shared::Source for JSource {
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
        emit.spell_ty(source)
    }

    fn build(&self, canonical: TokenStream) -> TokenStream {
        self.wrappers
            .iter()
            .rev()
            .fold(canonical, |value, wrapper| match *wrapper {
                "Box" => quote!(::std::boxed::Box::new(#value)),
                _ => unreachable!("unsupported wrapper was rejected while planning"),
            })
    }

    fn read(&self, source: TokenStream) -> TokenStream {
        self.wrappers
            .iter()
            .fold(source, |value, wrapper| match *wrapper {
                "Box" => quote!(*(#value)),
                _ => unreachable!("unsupported wrapper was rejected while planning"),
            })
    }
}

/// Whether the first JNI child converter consumes an existing reference or a
/// freshly produced intermediate value.
#[derive(Clone, Copy)]
pub(crate) enum JValueUse {
    Direct,
    SharedRef,
}

/// A JNI child's complete converter pipeline, including semantic pre-stages.
#[derive(Clone)]
pub(crate) struct JChild {
    call: shared::Call,
    direction: Direction,
    stages: Vec<syn::Ident>,
    value_use: JValueUse,
}

impl JChild {
    pub(crate) fn input(
        converter: syn::Ident,
        stages: Vec<syn::Ident>,
        value_use: JValueUse,
    ) -> Self {
        Self {
            call: shared::Call::new(converter, true, true),
            direction: Direction::Construct,
            stages,
            value_use,
        }
    }

    pub(crate) fn output(converter: syn::Ident, stages: Vec<syn::Ident>) -> Self {
        Self {
            call: shared::Call::new(converter, true, true),
            direction: Direction::Deconstruct,
            stages,
            value_use: JValueUse::Direct,
        }
    }
}

impl shared::Child for JChild {
    fn call(&self) -> &shared::Call {
        &self.call
    }

    fn invoke(&self, value: TokenStream) -> TokenStream {
        let converter = self.call.ident();
        match self.direction {
            Direction::Construct => {
                let value = match self.value_use {
                    JValueUse::Direct => value,
                    JValueUse::SharedRef => quote!(&(#value)),
                };
                let first = quote!(#converter(env, #value));
                if self.stages.is_empty() {
                    return first;
                }
                let mut body = quote!(let __chain_s0 = #first?;);
                let mut previous = format_ident!("__chain_s0");
                for (index, stage) in self.stages.iter().enumerate() {
                    let next = format_ident!("__chain_s{}", index + 1);
                    body.extend(quote!(
                        let #next = #stage(env, #previous)
                            .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                                __e.to_string()
                            ))?;
                    ));
                    previous = next;
                }
                quote!({ #body ::core::result::Result::<_, __JniErr>::Ok(#previous) })
            }
            Direction::Deconstruct => {
                if self.stages.is_empty() {
                    return quote!(#converter(env, #value));
                }
                let mut body = TokenStream::new();
                let mut previous = value;
                for (index, stage) in self.stages.iter().enumerate() {
                    let next = format_ident!("__chain_s{index}");
                    body.extend(quote!(
                        let #next = #stage(env, #previous)
                            .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                                __e.to_string()
                            ))?;
                    ));
                    previous = quote!(#next);
                }
                quote!({ #body #converter(env, #previous) })
            }
        }
    }
}

/// JNI's single-intermediate Optional representation operations.
#[derive(Clone)]
pub(crate) enum JOptionalBridge {
    InputNiche {
        wire: syn::Type,
        absent: syn::Expr,
    },
    InputBoxed {
        inner_wire: syn::Type,
        method: &'static str,
        signature: &'static str,
        getter: syn::Ident,
    },
    OutputNiche {
        wire: syn::Type,
        absent: syn::Expr,
    },
    OutputBoxed {
        inner_wire: syn::Type,
        helper: syn::Ident,
    },
}

impl shared::OptionalBridge for JOptionalBridge {
    fn intermediate(&self) -> syn::Type {
        match self {
            Self::InputNiche { wire, .. } | Self::OutputNiche { wire, .. } => wire.clone(),
            Self::InputBoxed { .. } | Self::OutputBoxed { .. } => {
                syn::parse_quote!(jni::objects::JObject)
            }
        }
    }

    fn is_absent(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::InputNiche { absent, .. } => quote!(#absent),
            Self::InputBoxed { .. } => quote!((#value).is_null()),
            Self::OutputNiche { .. } | Self::OutputBoxed { .. } => {
                quote!(::core::unreachable!())
            }
        }
    }

    fn present(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::InputNiche { .. } => value,
            Self::InputBoxed {
                inner_wire,
                method,
                signature,
                getter,
            } => quote!({
                env.call_method(&#value, #method, #signature, &[])
                    .and_then(|__value| __value.#getter())
                    .map(|__value| __value as #inner_wire)
                    .map_err(|__error| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Option unbox: {}", __error)
                    ))?
            }),
            Self::OutputNiche { .. } | Self::OutputBoxed { .. } => {
                quote!(::core::unreachable!())
            }
        }
    }

    fn build_absent(&self) -> TokenStream {
        match self {
            Self::OutputNiche { absent, .. } => quote!(#absent),
            Self::OutputBoxed { .. } => quote!(jni::objects::JObject::null()),
            Self::InputNiche { .. } | Self::InputBoxed { .. } => {
                quote!(::core::unreachable!())
            }
        }
    }

    fn build_present(&self, child: TokenStream) -> TokenStream {
        match self {
            Self::OutputNiche { .. } => child,
            Self::OutputBoxed { inner_wire, helper } => quote!({
                let __raw: #inner_wire = #child;
                ::prebindgen_jni_runtime::#helper(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Option box: {}", __error)
                    ))?
            }),
            Self::InputNiche { .. } | Self::InputBoxed { .. } => {
                quote!(::core::unreachable!())
            }
        }
    }
}

/// One late-rendered JniGen Optional converter.
#[derive(Clone)]
pub(crate) struct JOptionalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) chain: shared::Optional<JSource, JOptionalBridge, JChild>,
}

impl JOptionalPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.chain.direction {
            Direction::Construct => {
                let intermediate = annotate_jobject_with_lifetime(intermediate, "v");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#intermediate,
                    ) -> ::core::result::Result<#source, __JniErr> {
                        ::core::result::Result::Ok(#body)
                    }
                )
            }
            Direction::Deconstruct => {
                let intermediate = annotate_jobject_with_lifetime(intermediate, "a");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'a>(
                        env: &mut jni::JNIEnv<'a>,
                        v: #source,
                    ) -> ::core::result::Result<#intermediate, __JniErr> {
                        ::core::result::Result::Ok(#body)
                    }
                )
            }
        }
    }
}

/// Stable private name for a converter whose source spelling is deliberately
/// unavailable until final rendering.
pub(crate) fn planned_name(
    direction: Direction,
    source: &TypeRef,
    intermediate: &syn::Type,
) -> syn::Ident {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let key = source.key();
    let source_key = key.as_str();
    let source_id = crate::jni::emit::sanitize_for_ident(source_key);
    let wire_id = crate::jni::emit::wire_short(intermediate);
    let mut hash = DefaultHasher::new();
    source_key.hash(&mut hash);
    "::".hash(&mut hash);
    intermediate.to_token_stream().to_string().hash(&mut hash);
    let suffix = hash.finish() & 0xffff_ffff;
    match direction {
        Direction::Construct => format_ident!("{wire_id}_to_{source_id}_{suffix:08x}"),
        Direction::Deconstruct => format_ident!("{source_id}_to_{wire_id}_{suffix:08x}"),
    }
}
