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
    Choice(Box<JChoicePlan>),
    Sequence(Box<JSequencePlan>),
    Optional(Box<JOptionalPlan>),
    Invoke(Box<crate::jni::emit::JInvokePlan>),
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

    pub(crate) fn sequence(plan: JSequencePlan) -> Self {
        Self(JBody::Sequence(Box::new(plan)))
    }

    pub(crate) fn choice(plan: JChoicePlan) -> Self {
        Self(JBody::Choice(Box::new(plan)))
    }

    pub(crate) fn invoke(plan: crate::jni::emit::JInvokePlan) -> Self {
        Self(JBody::Invoke(Box::new(plan)))
    }

    #[cfg(test)]
    pub(crate) fn is_invoke(&self) -> bool {
        matches!(self.0, JBody::Invoke(_))
    }

    pub(crate) fn mark_reachable(&self) {
        match &self.0 {
            JBody::Complete(_) => {}
            JBody::OwnedHandle(plan) => plan.reachable.set(true),
            JBody::Product(plan) => {
                plan.reachable.set(true);
                for dependency in &plan.dependencies {
                    dependency.mark_reachable();
                }
            }
            JBody::Choice(plan) => {
                plan.reachable.set(true);
                for dependency in &plan.dependencies {
                    dependency.mark_reachable();
                }
            }
            JBody::Sequence(plan) => {
                plan.reachable.set(true);
                for dependency in &plan.dependencies {
                    dependency.mark_reachable();
                }
            }
            JBody::Optional(plan) => {
                plan.reachable.set(true);
                for dependency in &plan.dependencies {
                    dependency.mark_reachable();
                }
            }
            // An Invoke plan exists only for a declared callback crossing and
            // is called directly by that crossing's wrapper. Unlike reusable
            // child converters, it is reachable by construction and needs no
            // later activation pass.
            JBody::Invoke(_) => {}
        }
    }
}

impl RustFunction for JFunction {
    fn should_emit(&self) -> bool {
        match &self.0 {
            JBody::Complete(_) => true,
            JBody::OwnedHandle(plan) => plan.reachable.get(),
            JBody::Product(plan) => plan.reachable.get(),
            JBody::Optional(plan) => plan.reachable.get(),
            JBody::Sequence(plan) => plan.reachable.get(),
            JBody::Choice(plan) => plan.reachable.get(),
            // See `mark_reachable`: callback converters are roots, never
            // dormant dependencies waiting for a parent to activate them.
            JBody::Invoke(_) => true,
        }
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.0 {
            JBody::Complete(function) => (**function).clone(),
            JBody::OwnedHandle(plan) => plan.render(emit),
            JBody::Product(plan) => plan.render(emit),
            JBody::Optional(plan) => plan.render(emit),
            JBody::Choice(plan) => plan.render(emit),
            JBody::Sequence(plan) => plan.render(emit),
            JBody::Invoke(plan) => plan.render(emit),
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

/// Borrow a rendered value without adding noise to the common identifier case.
/// Parentheses remain mandatory for compound expressions so `&` applies to the
/// whole planned value rather than only its first syntactic component.
fn shared_ref(value: TokenStream) -> TokenStream {
    let mut tokens = value.clone().into_iter();
    if matches!(tokens.next(), Some(proc_macro2::TokenTree::Ident(_))) && tokens.next().is_none() {
        quote!(&#value)
    } else {
        quote!(&(#value))
    }
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

/// One frozen wire-to-Rust or Rust-to-wire converter pipeline.
///
/// The registry fragment has already selected the terminal converter and
/// ordered every semantic stage. Ordinary site renderers consume this payload
/// as one operation; they do not inspect `ConverterImpl::pre_stages` or look the
/// crossing up again.
#[derive(Clone)]
pub(crate) struct JPipeline {
    wire: syn::Type,
    child: JChild,
}

impl JPipeline {
    pub(crate) fn new(wire: syn::Type, child: JChild) -> Self {
        Self { wire, child }
    }

    pub(crate) fn wire(&self) -> &syn::Type {
        &self.wire
    }

    /// Render the already-planned call graph around `value`.
    pub(crate) fn invoke(&self, value: TokenStream) -> TokenStream {
        let call = self.child.invoke_with_env(quote!(&mut env), value);
        if self.child.stages.is_empty() {
            call
        } else {
            quote!((|| -> ::core::result::Result<_, __JniErr> { #call })())
        }
    }
}

impl JChild {
    /// Render this child in a context that supplies its own `JNIEnv` expression.
    /// Registry-composed converter bodies receive an `&mut JNIEnv` named
    /// `env`; exported wrappers own a mutable `JNIEnv` and pass `&mut env`.
    fn invoke_with_env(&self, env: TokenStream, value: TokenStream) -> TokenStream {
        let converter = self.call.ident();
        match self.direction {
            Direction::Construct => {
                let value = match self.value_use {
                    JValueUse::Direct => value,
                    JValueUse::SharedRef => shared_ref(value),
                    JValueUse::Cloned => quote!((*#value).clone()),
                };
                let first = quote!(#converter(#env, #value));
                if self.stages.is_empty() {
                    return first;
                }
                let mut body = quote!(let __chain_s0 = #first?;);
                let mut previous = format_ident!("__chain_s0");
                for (index, stage) in self.stages.iter().enumerate() {
                    let next = format_ident!("__chain_s{}", index + 1);
                    body.extend(quote!(
                        let #next = #stage(#env, #previous)
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
                    JValueUse::SharedRef => shared_ref(value),
                    JValueUse::Cloned => quote!((*#value).clone()),
                };
                if self.stages.is_empty() {
                    return quote!(#converter(#env, #value));
                }
                let mut body = TokenStream::new();
                let mut previous = value;
                for (index, stage) in self.stages.iter().enumerate() {
                    let next = format_ident!("__chain_s{index}");
                    body.extend(quote!(
                        let #next = #stage(#env, #previous)
                            .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                                __e.to_string()
                            ))?;
                    ));
                    previous = quote!(#next);
                }
                quote!({ #body #converter(#env, #previous) })
            }
        }
    }
}

impl shared::Child for JChild {
    fn call(&self) -> &shared::Call {
        &self.call
    }

    fn invoke(&self, value: TokenStream) -> TokenStream {
        self.invoke_with_env(quote!(env), value)
    }
}

/// Owned opaque-handle input, kept syntax-free until final emission.
#[derive(Clone)]
pub(crate) struct JOwnedHandlePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
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
                    #[inline(always)]
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
                    #[inline(always)]
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

#[derive(Clone)]
pub(crate) struct JChoicePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) dependencies: Vec<JFunction>,
    pub(crate) mode: Mode,
    pub(crate) chain: shared::Choice<JSource, shared::TupleChoice, shared::TupleProduct, JChild>,
}

impl JChoicePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = annotate_jobject_with_lifetime(&rendered.intermediate, "a");
        let body = &rendered.body;
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.chain.direction {
            Direction::Construct => syn::parse_quote!(
                #allow
                #[inline(always)]
                pub(crate) unsafe fn #name<'env, 'a>(
                    env: &mut jni::JNIEnv<'env>,
                    v: #intermediate,
                ) -> ::core::result::Result<#source, __JniErr> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            Direction::Deconstruct => {
                let input = match self.mode {
                    Mode::Owned => quote!(v: #source),
                    Mode::Shared => quote!(v: &#source),
                    Mode::Exclusive => quote!(v: &mut #source),
                };
                syn::parse_quote!(
                    #allow
                    #[inline(always)]
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

/// Java List operations for one registry-owned Sequence loop.
#[derive(Clone)]
pub(crate) enum JSequenceBridge {
    Input { child: Box<syn::Type> },
    Output,
}

impl shared::SequenceBridge for JSequenceBridge {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn begin(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Input { .. } => quote! {
                let __sequence_list = jni::objects::JList::from_env(env, #value)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: list-from-env: {}", e)
                    ))?;
                let mut __sequence_iter = __sequence_list.iter(env)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: list-iter: {}", e)
                    ))?;
            },
            Self::Output => {
                unreachable!("sequence bridge operation does not match its planned direction")
            }
        }
    }

    fn next(&self) -> TokenStream {
        match self {
            Self::Input { child } => quote! {
                match __sequence_iter.next(env)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: list-next: {}", e)
                    ))?
                {
                    ::core::option::Option::Some(__sequence_object) => {
                        let __sequence_part: #child = __sequence_object.into();
                        ::core::option::Option::Some(__sequence_part)
                    }
                    ::core::option::Option::None => ::core::option::Option::None,
                }
            },
            Self::Output => {
                unreachable!("sequence bridge operation does not match its planned direction")
            }
        }
    }

    fn begin_output(&self, _source: TokenStream) -> TokenStream {
        match self {
            Self::Output => quote! {
                let __sequence_output = env
                    .new_object("java/util/ArrayList", "()V", &[])
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: new ArrayList: {}", e)
                    ))?;
                let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: list-from-env: {}", e)
                    ))?;
            },
            Self::Input { .. } => {
                unreachable!("sequence bridge operation does not match its planned direction")
            }
        }
    }

    fn push(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Output => quote! {
                let __sequence_object: jni::objects::JObject = #value.into();
                __sequence_list.add(env, &__sequence_object)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Vec<_>: list-add: {}", e)
                    ))?;
            },
            Self::Input { .. } => {
                unreachable!("sequence bridge operation does not match its planned direction")
            }
        }
    }

    fn finish(&self) -> TokenStream {
        match self {
            Self::Output => quote!(__sequence_output),
            Self::Input { .. } => {
                unreachable!("sequence bridge operation does not match its planned direction")
            }
        }
    }

    fn fallible(&self) -> bool {
        true
    }
}

/// One late-rendered JniGen Sequence converter.
#[derive(Clone)]
pub(crate) struct JSequencePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) dependencies: Vec<JFunction>,
    pub(crate) mode: Mode,
    pub(crate) chain: shared::Sequence<JSource, JSequenceBridge, JChild>,
}

impl JSequencePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = annotate_jobject_with_lifetime(&rendered.intermediate, "a");
        let body = &rendered.body;
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.chain.direction {
            Direction::Construct => syn::parse_quote!(
                #allow
                pub(crate) unsafe fn #name<'env, 'a>(
                    env: &mut jni::JNIEnv<'env>,
                    v: &#intermediate,
                ) -> ::core::result::Result<#source, __JniErr> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            Direction::Deconstruct => {
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
    InputGated {
        child: syn::Type,
    },
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
    OutputGated {
        child: syn::Type,
        absent: TokenStream,
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
            Self::InputGated { child } | Self::OutputGated { child, .. } => {
                syn::parse_quote!((jni::sys::jboolean, #child))
            }
            Self::InputNiche { wire, .. } | Self::OutputNiche { wire, .. } => wire.clone(),
            Self::InputBoxed { .. } | Self::OutputBoxed { .. } => {
                syn::parse_quote!(jni::objects::JObject)
            }
        }
    }

    fn is_absent(&self) -> TokenStream {
        match self {
            Self::InputGated { .. } => quote!((v).0 == 0u8),
            Self::InputNiche { absent, .. } => quote!(#absent),
            Self::InputBoxed { .. } => quote!((v).is_null()),
            Self::OutputGated { .. } | Self::OutputNiche { .. } | Self::OutputBoxed { .. } => {
                unreachable!("optional bridge operation does not match its planned direction")
            }
        }
    }

    fn present(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::InputGated { .. } => quote!((#value).1),
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
            Self::OutputGated { .. } | Self::OutputNiche { .. } | Self::OutputBoxed { .. } => {
                unreachable!("optional bridge operation does not match its planned direction")
            }
        }
    }

    fn build_absent(&self) -> TokenStream {
        match self {
            Self::OutputNiche { absent, .. } => quote!(#absent),
            Self::OutputGated { absent, .. } => quote!((0u8, #absent)),
            Self::OutputBoxed { .. } => quote!(jni::objects::JObject::null()),
            Self::InputGated { .. } | Self::InputNiche { .. } | Self::InputBoxed { .. } => {
                unreachable!("optional bridge operation does not match its planned direction")
            }
        }
    }

    fn build_present(&self, child: TokenStream) -> TokenStream {
        match self {
            Self::OutputNiche { .. } => child,
            Self::OutputGated { .. } => quote!((1u8, #child)),
            Self::OutputBoxed { inner_wire, helper } => quote!({
                let __raw: #inner_wire = #child;
                ::prebindgen_jni_runtime::#helper(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<String>>::from(
                        format!("Option box: {}", __error)
                    ))?
            }),
            Self::InputGated { .. } | Self::InputNiche { .. } | Self::InputBoxed { .. } => {
                unreachable!("optional bridge operation does not match its planned direction")
            }
        }
    }
}

/// One late-rendered JniGen Optional converter.
#[derive(Clone)]
pub(crate) struct JOptionalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) dependencies: Vec<JFunction>,
    pub(crate) chain: shared::Optional<JSource, JOptionalBridge, JChild>,
    pub(crate) input_by_ref: bool,
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
                let input = if self.input_by_ref {
                    quote!(v: &#intermediate)
                } else {
                    quote!(v: #intermediate)
                };
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        #input,
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
    let wire_id = match intermediate {
        syn::Type::Tuple(tuple) => format!("tuple{}", tuple.elems.len()),
        _ => crate::jni::emit::wire_short(intermediate),
    };
    let suffix = crate::jni::emit::hash_name_pair(source_key, intermediate) & 0xffff_ffff;
    match direction {
        Direction::Construct => format_ident!("{wire_id}_to_{source_id}_{suffix:08x}"),
        Direction::Deconstruct => format_ident!("{source_id}_to_{wire_id}_{suffix:08x}"),
    }
}

#[cfg(test)]
mod tests {
    use prebindgen_registry::flat::ScalarKind;

    use super::*;

    #[test]
    fn planned_tuple_names_are_bounded_by_arity() {
        let tuple = vec!["jni::sys::jlong"; 64].join(",");
        let intermediate: syn::Type =
            syn::parse_str(&format!("({tuple},)")).expect("parse large tuple intermediate");
        let name = planned_name(
            Direction::Construct,
            &TypeRef::scalar(ScalarKind::I64),
            &intermediate,
        )
        .to_string();
        assert!(name.starts_with("tuple64_to_i64_"), "{name}");
        assert!(name.len() < 64, "{name}");
    }

    #[test]
    fn shared_refs_parenthesize_only_compound_values() {
        let bare: syn::ExprReference = syn::parse2(shared_ref(quote!(value))).unwrap();
        assert!(matches!(*bare.expr, syn::Expr::Path(_)));

        let compound: syn::ExprReference = syn::parse2(shared_ref(quote!(value.0))).unwrap();
        assert!(matches!(*compound.expr, syn::Expr::Paren(_)));
    }
}
