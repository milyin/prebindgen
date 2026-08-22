//! JniGen representation bridges for registry-composed converter chains.

use prebindgen_registry::{
    chain::{self as shared, Chain as _},
    flat::TypeRef,
    recipe::Mode,
    write::RustFunction,
    Emit,
};

use super::*;

/// A complete legacy JNI converter or a registry-composed late plan.
#[derive(Clone)]
pub(crate) struct JFunction(JBody);

#[derive(Clone)]
enum JBody {
    Complete(Box<syn::ItemFn>),
    OwnedHandle(Box<JOwnedHandlePlan>),
    Product(Box<JProductPlan>),
    Optional(Box<JOptionalPlan>),
}

impl JFunction {
    pub(crate) fn complete(function: syn::ItemFn) -> Self {
        Self(JBody::Complete(Box::new(function)))
    }

    pub(crate) fn owned_handle(plan: JOwnedHandlePlan) -> Self {
        Self(JBody::OwnedHandle(Box::new(plan)))
    }

    pub(crate) fn product(plan: JProductPlan) -> Self {
        Self(JBody::Product(Box::new(plan)))
    }

    pub(crate) fn optional(plan: JOptionalPlan) -> Self {
        Self(JBody::Optional(Box::new(plan)))
    }
    pub(crate) fn mark_reachable(&self) {
        if let JBody::Product(plan) = &self.0 {
            plan.reachable.set(true);
            for dependency in &plan.dependencies {
                dependency.mark_reachable();
            }
        }
    }
}

impl RustFunction for JFunction {
    fn should_emit(&self) -> bool {
        match &self.0 {
            JBody::Product(plan) => plan.reachable.get(),
            _ => true,
        }
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.0 {
            JBody::Complete(function) => (**function).clone(),
            JBody::OwnedHandle(plan) => plan.render(emit),
            JBody::Product(plan) => plan.render(emit),
            JBody::Optional(plan) => plan.render(emit),
        }
    }
}

/// Source spelling policy plus the transparent wrappers around its model type.
#[derive(Clone)]
pub(crate) struct JSource {
    pub(crate) wrappers: Vec<&'static str>,
    pub(crate) module: Option<syn::Path>,
}

struct QualifySource<'a> {
    module: &'a syn::Path,
    target: syn::Ident,
}

impl syn::visit_mut::VisitMut for QualifySource<'_> {
    fn visit_type_path_mut(&mut self, ty: &mut syn::TypePath) {
        if ty.qself.is_none()
            && ty.path.leading_colon.is_none()
            && ty.path.segments.len() == 1
            && ty.path.segments[0].ident == self.target
        {
            let mut qualified = self.module.clone();
            qualified.segments.push(ty.path.segments[0].clone());
            ty.path = qualified;
        }
        syn::visit_mut::visit_type_path_mut(self, ty);
    }
}

impl shared::Source for JSource {
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
        let mut ty = emit.spell_ty(source);
        let (Some(module), prebindgen_registry::flat::TypeKind::Named { id, .. }) =
            (&self.module, source.unwrapped().kind())
        else {
            return ty;
        };
        let Some(target) = id.ident() else {
            return ty;
        };
        let mut qualifier = QualifySource { module, target };
        syn::visit_mut::VisitMut::visit_type_mut(&mut qualifier, &mut ty);
        ty
    }

    fn build(&self, canonical: TokenStream) -> TokenStream {
        self.wrappers
            .iter()
            .rev()
            .fold(canonical, |value, wrapper| {
                let build = super::trait_impl::wrapper_ops(wrapper)
                    .and_then(|ops| ops.build)
                    .expect("unsupported wrapper was rejected while planning");
                build(value)
            })
    }

    fn read(&self, source: TokenStream) -> TokenStream {
        self.wrappers.iter().fold(source, |value, wrapper| {
            let read = super::trait_impl::wrapper_ops(wrapper)
                .and_then(|ops| ops.read)
                .expect("unsupported wrapper was rejected while planning");
            read(value)
        })
    }

    fn field(&self, source: TokenStream, name: &syn::Ident) -> TokenStream {
        if self.wrappers.is_empty() {
            quote!(#source.#name)
        } else {
            let value = self.read(source);
            quote!((#value).#name)
        }
    }
}

/// Whether the first JNI child converter consumes an existing reference or a
/// freshly produced intermediate value.
#[derive(Clone, Copy)]
pub(crate) enum JValueUse {
    Direct,
    SharedRef,
    Cloned,
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

    pub(crate) fn output(
        converter: syn::Ident,
        stages: Vec<syn::Ident>,
        value_use: JValueUse,
    ) -> Self {
        Self {
            call: shared::Call::new(converter, true, true),
            direction: Direction::Deconstruct,
            stages,
            value_use,
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
                    JValueUse::Cloned => quote!((*#value).clone()),
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
                let value = match self.value_use {
                    JValueUse::Direct => value,
                    JValueUse::SharedRef => quote!(&(#value)),
                    JValueUse::Cloned => quote!((*#value).clone()),
                };
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

/// Owned opaque-handle input, kept syntax-free until final emission.
#[derive(Clone)]
pub(crate) struct JOwnedHandlePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) module: syn::Path,
}

impl JOwnedHandlePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = shared::Source::spell(
            &JSource {
                wrappers: Vec::new(),
                module: Some(self.module.clone()),
            },
            &self.source,
            emit,
        );
        let allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::sys::jlong,
            ) -> ::core::result::Result<#source, __JniErr> {
                if *v == 0 || (*v & 1) == 1 {
                    return ::core::result::Result::Err(
                        <__JniErr as ::core::convert::From<String>>::from(
                            "Operation on a closed native handle.".to_string(),
                        ),
                    );
                }
                ::core::result::Result::Ok(unsafe {
                    *::std::boxed::Box::from_raw(*v as *mut #source)
                })
            }
        )
    }
}
/// One JNI Product converter assembled by the registry from child chains.
#[derive(Clone)]
pub(crate) struct JProductPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) dependencies: Vec<JFunction>,
    pub(crate) mode: Mode,
    pub(crate) chain: shared::Product<JSource, shared::TupleProduct, JChild>,
}

impl JProductPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.chain.direction {
            Direction::Construct => {
                let intermediate = annotate_jobject_with_lifetime(intermediate, "a");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'a>(
                        env: &mut jni::JNIEnv<'env>,
                        v: #intermediate,
                    ) -> ::core::result::Result<#source, __JniErr> {
                        ::core::result::Result::Ok(#body)
                    }
                )
            }
            Direction::Deconstruct => {
                let intermediate = annotate_jobject_with_lifetime(intermediate, "a");
                let input = match self.mode {
                    Mode::Owned => quote!(v: #source),
                    Mode::Shared => quote!(v: &#source),
                    Mode::Exclusive => quote!(v: &mut #source),
                };
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'a>(
                        env: &mut jni::JNIEnv<'a>,
                        #input,
                    ) -> ::core::result::Result<#intermediate, __JniErr> {
                        ::core::result::Result::Ok(#body)
                    }
                )
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
                unreachable!("optional bridge operation does not match its planned direction")
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
                unreachable!("optional bridge operation does not match its planned direction")
            }
        }
    }

    fn build_absent(&self) -> TokenStream {
        match self {
            Self::OutputNiche { absent, .. } => quote!(#absent),
            Self::OutputBoxed { .. } => quote!(jni::objects::JObject::null()),
            Self::InputNiche { .. } | Self::InputBoxed { .. } => {
                unreachable!("optional bridge operation does not match its planned direction")
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
                unreachable!("optional bridge operation does not match its planned direction")
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

/// Build the name carrier required by the legacy \`ConverterImpl\` index.
///
/// Planned converters do not use this function's signature or body: their
/// complete Rust function is rendered from \`JFunction\` only after the model is
/// fully resolved. Keeping this compatibility seam in one visibly fake helper
/// prevents callers from mistaking the marker for executable generated code.
pub(crate) fn planned_marker(ident: &syn::Ident) -> syn::ItemFn {
    syn::parse_quote!(fn #ident() {})
}

/// Stable private name for a converter whose source spelling is deliberately
/// unavailable until final rendering.
pub(crate) fn planned_name(
    direction: Direction,
    source: &TypeRef,
    intermediate: &syn::Type,
) -> syn::Ident {
    let key = source.key();
    let source_key = key.as_str();
    let source_id = crate::jni::emit::sanitize_for_ident(source_key);
    let wire_id = crate::jni::emit::wire_short(intermediate);
    let suffix = crate::jni::emit::hash_name_pair(source_key, intermediate) & 0xffff_ffff;
    match direction {
        Direction::Construct => format_ident!("{wire_id}_to_{source_id}_{suffix:08x}"),
        Direction::Deconstruct => format_ident!("{source_id}_to_{wire_id}_{suffix:08x}"),
    }
}
