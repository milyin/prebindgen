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
    Marker(syn::Ident),
    ValueCodec(Box<JValueCodecPlan>),
    OwnedHandle(Box<JOwnedHandlePlan>),
    BorrowedOptionalHandle(Box<JBorrowedOptionalHandlePlan>),
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

    pub(crate) fn retained(function: syn::ItemFn) -> Self {
        if is_planned_marker(&function) {
            Self(JBody::Marker(function.sig.ident))
        } else {
            Self::complete(function)
        }
    }

    pub(crate) fn owned_handle(plan: JOwnedHandlePlan) -> Self {
        Self(JBody::OwnedHandle(Box::new(plan)))
    }

    pub(crate) fn value_codec(plan: JValueCodecPlan) -> Self {
        Self(JBody::ValueCodec(Box::new(plan)))
    }

    pub(crate) fn borrowed_optional_handle(plan: JBorrowedOptionalHandlePlan) -> Self {
        Self(JBody::BorrowedOptionalHandle(Box::new(plan)))
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

    #[cfg(test)]
    pub(crate) fn is_value_codec(&self) -> bool {
        matches!(self.0, JBody::ValueCodec(_))
    }

    #[cfg(test)]
    pub(crate) fn is_borrowed_optional_handle(&self) -> bool {
        matches!(self.0, JBody::BorrowedOptionalHandle(_))
    }

    pub(crate) fn is_borrowed_optional_value(&self) -> bool {
        matches!(
            &self.0,
            JBody::Optional(plan) if plan.chain.source_policy.borrowed_input.is_some()
        )
    }

    #[cfg(test)]
    pub(crate) fn is_optional(&self) -> bool {
        matches!(self.0, JBody::Optional(_))
    }

    pub(crate) fn mark_reachable(&self) {
        match &self.0 {
            JBody::Complete(_) => {}
            JBody::Marker(_) | JBody::ValueCodec(_) => {}
            JBody::OwnedHandle(plan) => plan.reachable.set(true),
            JBody::BorrowedOptionalHandle(plan) => plan.reachable.set(true),
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
    fn ident(&self) -> &syn::Ident {
        match &self.0 {
            JBody::Complete(function) => &function.sig.ident,
            JBody::Marker(ident) => ident,
            JBody::ValueCodec(plan) => &plan.ident,
            JBody::OwnedHandle(plan) => &plan.ident,
            JBody::BorrowedOptionalHandle(plan) => &plan.ident,
            JBody::Product(plan) => &plan.ident,
            JBody::Choice(plan) => &plan.ident,
            JBody::Sequence(plan) => &plan.ident,
            JBody::Optional(plan) => &plan.ident,
            JBody::Invoke(plan) => plan.name(),
        }
    }

    fn should_emit(&self) -> bool {
        match &self.0 {
            JBody::Complete(_) => true,
            JBody::Marker(_) => false,
            // Complete compatibility parents do not yet propagate
            // reachability to the children they call. Until those parents are
            // planned too, every compiled value codec remains an emitted leaf.
            JBody::ValueCodec(_) => true,
            JBody::OwnedHandle(plan) => plan.reachable.get(),
            JBody::BorrowedOptionalHandle(plan) => plan.reachable.get(),
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
            JBody::Marker(ident) => planned_marker(ident),
            JBody::ValueCodec(plan) => plan.render(emit),
            JBody::OwnedHandle(plan) => plan.render(emit),
            JBody::BorrowedOptionalHandle(plan) => plan.render(emit),
            JBody::Product(plan) => plan.render(emit),
            JBody::Optional(plan) => plan.render(emit),
            JBody::Choice(plan) => plan.render(emit),
            JBody::Sequence(plan) => plan.render(emit),
            JBody::Invoke(plan) => plan.render(emit),
        }
    }
}

#[derive(Clone)]
enum JValueBody {
    Ready(syn::Expr),
    /// Array classification and JNI method selection are adapter policy. The
    /// decoder's element and full-array type ascriptions are source spelling,
    /// so its body is built only during final rendering.
    PrimitiveArray(Box<crate::jni::prim_array::PrimArray>),
}

/// One terminal value crossing, retained without source Rust syntax.
///
/// The JNI representation and source-independent conversion policy are safe
/// to freeze during recipe compilation. Source Rust spelling is not: the plan
/// keeps the Flat reading opaque until the writer supplies [`Emit`]. Most
/// codecs retain a ready expression; fixed-size arrays retain their JNI policy
/// and build the source-typed expression only while rendering. Bare `str`
/// records its adapter-authored `String`/`&str` signature explicitly without
/// deriving either spelling from the crossing.
#[derive(Clone)]
pub(crate) struct JValueCodecPlan {
    ident: syn::Ident,
    direction: Direction,
    source: TypeRef,
    adapter_source: Option<syn::Type>,
    wire: syn::Type,
    body: JValueBody,
}

impl JValueCodecPlan {
    pub(crate) fn new(
        direction: Direction,
        source: TypeRef,
        wire: syn::Type,
        body: syn::Expr,
    ) -> Self {
        let ident = planned_name(direction, &source, &wire);
        Self {
            ident,
            direction,
            source,
            adapter_source: None,
            wire,
            body: JValueBody::Ready(body),
        }
    }

    /// A codec whose Rust signature deliberately differs from its crossing.
    ///
    /// The supplied type is adapter-authored syntax, not syntax extracted from
    /// `source`: bare `str` is unsized, so JNI decodes it into `String` and
    /// encodes it through `&str`. Keeping that exception explicit lets the
    /// crossing remain opaque while preserving the actual converter contract.
    pub(crate) fn with_adapter_source(
        direction: Direction,
        source: TypeRef,
        adapter_source: syn::Type,
        wire: syn::Type,
        body: syn::Expr,
    ) -> Self {
        use quote::ToTokens as _;

        let source_tokens = adapter_source.to_token_stream();
        let ident = match direction {
            Direction::Construct => crate::jni::emit::input_name(&source_tokens, &wire),
            Direction::Deconstruct => crate::jni::emit::output_name(&source_tokens, &wire),
        };
        Self {
            ident,
            direction,
            source,
            adapter_source: Some(adapter_source),
            wire,
            body: JValueBody::Ready(body),
        }
    }

    pub(crate) fn primitive_array(
        direction: Direction,
        source: TypeRef,
        spec: crate::jni::prim_array::PrimArray,
    ) -> Self {
        let ident = planned_name(direction, &source, &spec.wire);
        Self {
            ident,
            direction,
            source,
            adapter_source: None,
            wire: spec.wire.clone(),
            body: JValueBody::PrimitiveArray(Box::new(spec)),
        }
    }

    pub(crate) fn name(&self) -> &syn::Ident {
        &self.ident
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = self
            .adapter_source
            .as_ref()
            .map(|source| quote::quote!(#source))
            .unwrap_or_else(|| emit.spell(&self.source));
        let wire = &self.wire;
        let body = match &self.body {
            JValueBody::Ready(body) => body.clone(),
            JValueBody::PrimitiveArray(spec) => match self.direction {
                Direction::Construct => {
                    crate::jni::prim_array::input_body(&self.source, spec, emit)
                }
                Direction::Deconstruct => crate::jni::prim_array::output_body(spec),
            },
        };
        let body = super::builder::body_for_exc(&body, None);
        let allow = super::trait_impl::generated_converter_attr();
        match self.direction {
            Direction::Construct => {
                let wire = annotate_jobject_with_lifetime(wire, "v");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#wire,
                    ) -> ::core::result::Result<#source, __JniErr> {
                        #body
                    }
                )
            }
            Direction::Deconstruct => {
                let wire = annotate_jobject_with_lifetime(wire, "a");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'a>(
                        env: &mut jni::JNIEnv<'a>,
                        v: #source,
                    ) -> ::core::result::Result<#wire, __JniErr> {
                        #body
                    }
                )
            }
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

/// Optional source policy with an owned carrier for `Option<&T>` input.
///
/// The registry still owns construction of the Optional shape. A Rust
/// reference cannot escape the converter that creates its referent, so a
/// borrowed data-class parameter is represented transiently as `Option<T>`;
/// the exported wrapper borrows that local immediately before the source call.
/// The target remains a Flat identity until this policy is rendered.
#[derive(Clone)]
pub(crate) struct JOptionalSource {
    pub(crate) ordinary: JSource,
    pub(crate) borrowed_input: Option<TypeRef>,
}

impl shared::Source for JOptionalSource {
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
        if let Some(target) = &self.borrowed_input {
            let target = emit.spell_ty(target);
            syn::parse_quote!(::core::option::Option<#target>)
        } else {
            shared::Source::spell(&self.ordinary, source, emit)
        }
    }

    fn build(&self, canonical: TokenStream) -> TokenStream {
        shared::Source::build(&self.ordinary, canonical)
    }

    fn read(&self, source: TokenStream) -> TokenStream {
        shared::Source::read(&self.ordinary, source)
    }

    fn field(&self, source: TokenStream, name: &syn::Ident) -> TokenStream {
        shared::Source::field(&self.ordinary, source, name)
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
    body: JPipelineBody,
    borrowed_optional_value: bool,
}

#[derive(Clone)]
enum JPipelineBody {
    Converter(JChild),
    VecHandle(Box<JVecHandleInput>),
}

/// A parameter-site ABI that borrows or consumes the transient `Vec<T>` built
/// by Kotlin's push-helper trio. The site retains model readings and wrapper
/// policy only; Rust types are spelled when the final wrapper is rendered.
#[derive(Clone)]
struct JVecHandleInput {
    source: TypeRef,
    elem: TypeRef,
    by_ref: bool,
    elem_wrappers: Vec<&'static str>,
}

impl JPipeline {
    pub(crate) fn new(wire: syn::Type, child: JChild, borrowed_optional_value: bool) -> Self {
        Self {
            wire,
            body: JPipelineBody::Converter(child),
            borrowed_optional_value,
        }
    }

    pub(crate) fn vec_handle(
        source: TypeRef,
        elem: TypeRef,
        by_ref: bool,
        elem_wrappers: Vec<&'static str>,
    ) -> Self {
        Self {
            wire: syn::parse_quote!(jni::sys::jlong),
            body: JPipelineBody::VecHandle(Box::new(JVecHandleInput {
                source,
                elem,
                by_ref,
                elem_wrappers,
            })),
            borrowed_optional_value: false,
        }
    }

    pub(crate) fn wire(&self) -> &syn::Type {
        &self.wire
    }

    /// Whether an `Option<&T>` input converter yields an owned `Option<T>`
    /// carrier that the wrapper must borrow with `as_ref` / `as_mut`.
    pub(crate) fn borrowed_optional_value(&self) -> bool {
        self.borrowed_optional_value
    }

    /// Render a site operation that cannot fail. Terminal converters keep
    /// their `Result` contract; the transient Vec-handle ABI is a direct
    /// borrow or move and should not acquire an unreachable error branch just
    /// because it now shares the ordinary decode scaffold.
    pub(crate) fn invoke_infallible(&self, value: TokenStream, emit: &Emit) -> Option<TokenStream> {
        match &self.body {
            JPipelineBody::VecHandle(plan) => Some(plan.invoke(value, emit)),
            JPipelineBody::Converter(_) => None,
        }
    }

    /// Render the already-planned call graph around `value`.
    pub(crate) fn invoke(&self, value: TokenStream, emit: &Emit) -> TokenStream {
        match &self.body {
            JPipelineBody::Converter(child) => {
                let call = child.invoke_with_env(quote!(&mut env), value);
                if child.stages.is_empty() {
                    call
                } else {
                    quote!((|| -> ::core::result::Result<_, __JniErr> { #call })())
                }
            }
            JPipelineBody::VecHandle(plan) => {
                let value = plan.invoke(value, emit);
                quote!(::core::result::Result::<_, __JniErr>::Ok(#value))
            }
        }
    }
}

impl JVecHandleInput {
    fn invoke(&self, handle: TokenStream, emit: &Emit) -> TokenStream {
        let elem = emit.spell_ty(&self.elem);
        if self.by_ref {
            return quote!(unsafe {
                OwnedObject::from_raw(#handle as *const Vec<#elem>)
            });
        }

        let taken = quote!(unsafe {
            ::core::mem::take(&mut *(#handle as *mut Vec<#elem>))
        });
        let taken = if self.elem_wrappers.is_empty() {
            taken
        } else {
            let wrapped =
                super::trait_impl::build_through_wrappers(&self.elem_wrappers, quote!(__e))
                    .expect("Vec-handle planning accepted this element spelling");
            quote!(
                #taken
                    .into_iter()
                    .map(|__e| #wrapped)
                    .collect::<Vec<_>>()
            )
        };
        super::trait_impl::build_through_erased_wrappers(&self.source, taken)
            .expect("Vec-handle planning accepted this run spelling")
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

/// Optional borrowed opaque-handle input, kept syntax-free until final emission.
///
/// The converter deliberately returns a non-owning `OwnedObject<T>` carrier,
/// not the crossing's `Option<&T>` spelling. The wrapper borrows through that
/// carrier with `.as_deref()` / `.as_deref_mut()` while Kotlin's handle lock
/// keeps the allocation alive for the native call.
#[derive(Clone)]
pub(crate) struct JBorrowedOptionalHandlePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) target: TypeRef,
    pub(crate) module: syn::Path,
}

impl JBorrowedOptionalHandlePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let target = shared::Source::spell(
            &JSource {
                wrappers: Vec::new(),
                module: Some(self.module.clone()),
            },
            &self.target,
            emit,
        );
        let allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::sys::jlong,
            ) -> ::core::result::Result<Option<OwnedObject<#target>>, __JniErr> {
                if *v == 0 {
                    Ok(None)
                } else if (*v & 1) == 1 {
                    Err(<__JniErr as ::core::convert::From<String>>::from(
                        "Operation on a closed native handle.".to_string(),
                    ))
                } else {
                    Ok(Some(unsafe { OwnedObject::from_raw(*v as *const #target) }))
                }
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
    pub(crate) chain: shared::Optional<JOptionalSource, JOptionalBridge, JChild>,
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

fn is_planned_marker(function: &syn::ItemFn) -> bool {
    function.attrs.is_empty()
        && function.vis == syn::Visibility::Inherited
        && function.sig.constness.is_none()
        && function.sig.asyncness.is_none()
        && function.sig.unsafety.is_none()
        && function.sig.abi.is_none()
        && function.sig.generics.params.is_empty()
        && function.sig.inputs.is_empty()
        && matches!(function.sig.output, syn::ReturnType::Default)
        && function.block.stmts.is_empty()
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
        syn::Type::Tuple(tuple) if !tuple.elems.is_empty() => {
            format!("tuple{}", tuple.elems.len())
        }
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
