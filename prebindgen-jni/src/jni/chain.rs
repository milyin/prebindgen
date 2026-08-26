//! JniGen representation bridges for registry-composed converter chains.

use prebindgen_registry::{
    chain::{self as shared, Chain as _},
    flat::{TypeKey, TypeRef},
    recipe::Mode,
    write::RustFunction,
    Emit,
};

use super::*;

/// A registry-composed JNI converter plan or a non-emitting compatibility marker.
#[derive(Clone)]
pub(crate) struct JFunction(JBody);

#[derive(Clone)]
enum JBody {
    Marker(syn::Ident),
    ValueCodec(Box<JValueCodecPlan>),
    HandleCodec(Box<JHandleCodecPlan>),
    CustomConversion(Box<JCustomConversionPlan>),
    Result(Box<JResultPlan>),
    Transparent(Box<JTransparentPlan>),
    BorrowedOptionalHandle(Box<JBorrowedOptionalHandlePlan>),
    Product(Box<JProductPlan>),
    Choice(Box<JChoicePlan>),
    Sequence(Box<JSequencePlan>),
    Optional(Box<JOptionalPlan>),
    StructCodec(Box<JStructCodecPlan>),
    SumCodec(Box<JSumCodecPlan>),
    Invoke(Box<crate::jni::emit::JInvokePlan>),
}

impl JFunction {
    pub(crate) fn marker(ident: syn::Ident) -> Self {
        Self(JBody::Marker(ident))
    }

    pub(crate) fn handle_codec(plan: JHandleCodecPlan) -> Self {
        Self(JBody::HandleCodec(Box::new(plan)))
    }

    pub(crate) fn result(plan: JResultPlan) -> Self {
        Self(JBody::Result(Box::new(plan)))
    }

    pub(crate) fn custom_conversion(plan: JCustomConversionPlan) -> Self {
        Self(JBody::CustomConversion(Box::new(plan)))
    }

    pub(crate) fn transparent(plan: JTransparentPlan) -> Self {
        Self(JBody::Transparent(Box::new(plan)))
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

    pub(crate) fn struct_codec(plan: JStructCodecPlan) -> Self {
        Self(JBody::StructCodec(Box::new(plan)))
    }

    pub(crate) fn sum_codec(plan: JSumCodecPlan) -> Self {
        Self(JBody::SumCodec(Box::new(plan)))
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
    pub(crate) fn is_handle_codec(&self) -> bool {
        matches!(self.0, JBody::HandleCodec(_))
    }

    #[cfg(test)]
    pub(crate) fn is_struct_codec(&self) -> bool {
        matches!(self.0, JBody::StructCodec(_))
    }

    #[cfg(test)]
    pub(crate) fn is_sum_codec(&self) -> bool {
        matches!(self.0, JBody::SumCodec(_))
    }

    #[cfg(test)]
    pub(crate) fn is_custom_conversion(&self) -> bool {
        matches!(self.0, JBody::CustomConversion(_))
    }

    #[cfg(test)]
    pub(crate) fn is_result(&self) -> bool {
        matches!(self.0, JBody::Result(_))
    }

    #[cfg(test)]
    pub(crate) fn is_transparent(&self) -> bool {
        matches!(self.0, JBody::Transparent(_))
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
            JBody::Marker(_) | JBody::ValueCodec(_) => {}
            JBody::CustomConversion(_) => {}
            JBody::HandleCodec(plan) => plan.reachable.set(true),
            JBody::Result(plan) => {
                plan.reachable.set(true);
                plan.success.mark_reachable();
            }
            JBody::Transparent(plan) => {
                plan.reachable.set(true);
                plan.inner.mark_reachable();
            }
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
            // The legacy whole-object parents still make every terminal
            // reachable. Whole-object codecs retain no source syntax, but keep
            // that compatibility reachability until those parents are planned
            // too.
            JBody::StructCodec(_) => {}
            JBody::SumCodec(_) => {}
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
            JBody::Marker(ident) => ident,
            JBody::ValueCodec(plan) => &plan.ident,
            JBody::HandleCodec(plan) => &plan.ident,
            JBody::CustomConversion(plan) => &plan.ident,
            JBody::Result(plan) => &plan.ident,
            JBody::Transparent(plan) => &plan.ident,
            JBody::BorrowedOptionalHandle(plan) => &plan.ident,
            JBody::Product(plan) => &plan.ident,
            JBody::Choice(plan) => &plan.ident,
            JBody::Sequence(plan) => &plan.ident,
            JBody::Optional(plan) => &plan.ident,
            JBody::StructCodec(plan) => &plan.ident,
            JBody::SumCodec(plan) => &plan.ident,
            JBody::Invoke(plan) => plan.name(),
        }
    }

    fn should_emit(&self) -> bool {
        match &self.0 {
            JBody::Marker(_) => false,
            // Compatibility parents do not yet propagate reachability to the
            // children they call. Until those parents are planned too, every
            // compiled value codec remains an emitted leaf.
            JBody::ValueCodec(_) => true,
            JBody::HandleCodec(plan) => plan.reachable.get(),
            // Canonical conversion stages historically emit once compiled.
            // Keep that reachability policy until every complete compatibility
            // parent retains stage dependencies as well as its wire codec.
            JBody::CustomConversion(_) => true,
            JBody::Result(plan) => plan.reachable.get(),
            JBody::Transparent(plan) => plan.reachable.get(),
            JBody::BorrowedOptionalHandle(plan) => plan.reachable.get(),
            JBody::Product(plan) => plan.reachable.get(),
            JBody::Optional(plan) => plan.reachable.get(),
            JBody::Sequence(plan) => plan.reachable.get(),
            JBody::Choice(plan) => plan.reachable.get(),
            JBody::StructCodec(_) => true,
            JBody::SumCodec(_) => true,
            // See `mark_reachable`: callback converters are roots, never
            // dormant dependencies waiting for a parent to activate them.
            JBody::Invoke(_) => true,
        }
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.0 {
            JBody::Marker(ident) => planned_marker(ident),
            JBody::ValueCodec(plan) => plan.render(emit),
            JBody::HandleCodec(plan) => plan.render(emit),
            JBody::CustomConversion(plan) => plan.render(emit),
            JBody::Result(plan) => plan.render(emit),
            JBody::Transparent(plan) => plan.render(emit),
            JBody::BorrowedOptionalHandle(plan) => plan.render(emit),
            JBody::Product(plan) => plan.render(emit),
            JBody::Optional(plan) => plan.render(emit),
            JBody::Choice(plan) => plan.render(emit),
            JBody::Sequence(plan) => plan.render(emit),
            JBody::StructCodec(plan) => plan.render(emit),
            JBody::SumCodec(plan) => plan.render(emit),
            JBody::Invoke(plan) => plan.render(emit),
        }
    }
}

/// One whole-object struct terminal retained as Flat shape plus JNI policy.
///
/// The plan deliberately holds no `syn::Type` or token body for the source
/// crossing. The writer supplies [`Emit`] once resolution, glue planning, and
/// Kotlin planning are complete; only then is the source signature spelled and
/// the captured struct delimiter shape assembled.
#[derive(Clone)]
pub(crate) struct JStructCodecPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) direction: Direction,
    pub(crate) body: JStructCodecBody,
}

#[derive(Clone)]
pub(crate) enum JStructCodecBody {
    Input(Box<crate::jni::emit::JObjectStructInputPlan>),
    Output {
        plan: std::rc::Rc<crate::jni::struct_plan::StructPlan>,
        java_class_name: String,
    },
}

impl JStructCodecPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = emit.spell_ty(&self.source);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let body = match &self.body {
            JStructCodecBody::Input(plan) => plan.render(emit),
            JStructCodecBody::Output {
                plan,
                java_class_name,
            } => crate::jni::emit::render_struct_output_body(plan, java_class_name),
        };
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.direction {
            Direction::Construct => {
                let wire = annotate_jobject_with_lifetime(&wire, "v");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#wire,
                    ) -> ::core::result::Result<#source, __JniErr> {
                        Ok(#body)
                    }
                )
            }
            Direction::Deconstruct => {
                let wire = annotate_jobject_with_lifetime(&wire, "a");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'a>(
                        env: &mut jni::JNIEnv<'a>,
                        v: #source,
                    ) -> ::core::result::Result<#wire, __JniErr> {
                        Ok(#body)
                    }
                )
            }
        }
    }
}

/// One whole-object sealed-sum input retained as Flat alternatives plus JNI
/// property policy. Source spelling and alternative construction are deferred
/// to the final writer-owned [`Emit`] pass.
#[derive(Clone)]
pub(crate) struct JSumCodecPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) body: crate::jni::emit::JObjectSumInputPlan,
}

impl JSumCodecPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = emit.spell_ty(&self.source);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let wire = annotate_jobject_with_lifetime(&wire, "v");
        let body = self.body.render(emit);
        let allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &#wire,
            ) -> ::core::result::Result<#source, __JniErr> {
                Ok(#body)
            }
        )
    }
}

#[derive(Clone)]
pub(crate) enum JCustomType {
    Model(TypeRef),
    Declared(syn::Type),
}

impl JCustomType {
    fn spell(&self, emit: &Emit) -> syn::Type {
        match self {
            Self::Model(reading) => emit.spell_ty(reading),
            Self::Declared(ty) => ty.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum JCustomCall {
    Function {
        module: syn::Path,
        function: syn::Ident,
        by_ref: bool,
        error: Option<Box<TypeRef>>,
    },
    Trait {
        fallible: bool,
    },
}

/// One adapter-declared semantic conversion stage retained without spelling
/// either source type from the Flat model.
#[derive(Clone)]
pub(crate) struct JCustomConversionPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) representation: JCustomType,
    pub(crate) direction: Direction,
    pub(crate) call: JCustomCall,
    pub(crate) domain: Option<prebindgen_registry::RepresentationDomain>,
}

impl JCustomConversionPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = emit.spell_ty(&self.source);
        let representation = self.representation.spell(emit);
        let (input, output) = match self.direction {
            Direction::Construct => (&representation, &source),
            Direction::Deconstruct => (&source, &representation),
        };
        let (raw, raw_error): (syn::Expr, Option<syn::Type>) = match &self.call {
            JCustomCall::Function {
                module,
                function,
                by_ref,
                error,
            } => {
                let arg = if *by_ref { quote!(&v) } else { quote!(v) };
                (
                    syn::parse_quote!(#module::#function(#arg)),
                    error.as_deref().map(|error| emit.spell_ty(error)),
                )
            }
            JCustomCall::Trait { fallible } => {
                let raw = match (self.direction, *fallible) {
                    (Direction::Construct, false) => syn::parse_quote!(
                        <#representation as ::core::convert::Into<#source>>::into(v)
                    ),
                    (Direction::Construct, true) => syn::parse_quote!(
                        <#representation as ::core::convert::TryInto<#source>>::try_into(v)
                    ),
                    (Direction::Deconstruct, false) => syn::parse_quote!(
                        <#source as ::core::convert::Into<#representation>>::into(v)
                    ),
                    (Direction::Deconstruct, true) => syn::parse_quote!(
                        <#source as ::core::convert::TryInto<#representation>>::try_into(v)
                    ),
                };
                let error = (*fallible).then(|| match self.direction {
                    Direction::Construct => syn::parse_quote!(
                        <#representation as ::core::convert::TryInto<#source>>::Error
                    ),
                    Direction::Deconstruct => syn::parse_quote!(
                        <#source as ::core::convert::TryInto<#representation>>::Error
                    ),
                });
                (raw, error)
            }
        };
        let (body, error) = if let Some(domain) = &self.domain {
            let valid = domain.contains_expr(match self.direction {
                Direction::Construct => quote!(v),
                Direction::Deconstruct => quote!(__repr),
            });
            let converted = if raw_error.is_some() {
                quote!((#raw).map_err(|__e| {
                    <__JniErr as ::core::convert::From<String>>::from(__e.to_string())
                }))
            } else {
                quote!(::core::result::Result::Ok(#raw))
            };
            let diagnostic = self.source.to_string();
            let body: syn::Expr = match self.direction {
                Direction::Construct => syn::parse_quote!({
                    if #valid {
                        #converted
                    } else {
                        ::core::result::Result::Err(
                            <__JniErr as ::core::convert::From<String>>::from(
                                format!(
                                    "{} representation is outside its declared domain",
                                    #diagnostic,
                                )
                            )
                        )
                    }
                }),
                Direction::Deconstruct => syn::parse_quote!({
                    match #converted {
                        ::core::result::Result::Ok(__repr) if #valid => {
                            ::core::result::Result::Ok(__repr)
                        }
                        ::core::result::Result::Ok(_) => {
                            ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<String>>::from(
                                    format!(
                                        "{} representation is outside its declared domain",
                                        #diagnostic,
                                    )
                                )
                            )
                        }
                        ::core::result::Result::Err(__e) => {
                            ::core::result::Result::Err(__e)
                        }
                    }
                }),
            };
            (body, syn::parse_quote!(__JniErr))
        } else {
            (
                crate::jni::body_for_exc(&raw, raw_error.as_ref()),
                raw_error.unwrap_or_else(crate::jni::builder::default_err_type),
            )
        };
        let allow = crate::jni::trait_impl::generated_converter_attr();
        let lifetime = syn::Lifetime::new("\u{27}a", Span::call_site());
        syn::parse_quote!(
            #allow
            pub(crate) unsafe fn #name<#lifetime>(
                env: &mut jni::JNIEnv<#lifetime>,
                v: #input,
            ) -> ::core::result::Result<#output, #error> {
                #body
            }
        )
    }
}

/// One fallible-output peel retained as model readings until final emission.
///
/// The error never crosses as a value: returning it from this stage makes the
/// already-frozen site pipeline route it through the function's domain-error
/// channel. The success value then continues through `success`'s converter.
#[derive(Clone)]
pub(crate) struct JResultPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) success: JFunction,
    pub(crate) source: TypeRef,
    pub(crate) ok: TypeRef,
    pub(crate) err: TypeRef,
}

impl JResultPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = emit.spell_ty(&self.source);
        let ok = emit.spell_ty(&self.ok);
        let err = emit.spell_ty(&self.err);
        let ok = annotate_jobject_with_lifetime(&ok, "a");
        let allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #allow
            pub(crate) unsafe fn #name<'a>(
                env: &mut jni::JNIEnv<'a>,
                v: #source,
            ) -> ::core::result::Result<#ok, #err> {
                v
            }
        )
    }
}

/// One transparent-wrapper bridge retained as Flat wrapper policy plus its
/// already-compiled inner conversion.
///
/// Planning knows that the wrapper is erased and how to traverse it, but never
/// spells the wrapped Rust type. The final renderer performs that spelling and
/// assembles the standard converter signature around the frozen child call.
#[derive(Clone)]
pub(crate) struct JTransparentPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) inner: JFunction,
    pub(crate) source: TypeRef,
    pub(crate) wire: syn::Type,
    pub(crate) direction: Direction,
    pub(crate) child: JChild,
}

impl JTransparentPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = emit.spell_ty(&self.source);
        let wire = &self.wire;
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.direction {
            Direction::Construct => {
                let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "v");
                let input = if matches!(wire, syn::Type::Ptr(_)) {
                    quote!(v: #wire)
                } else {
                    quote!(v: &#wire_with_lifetime)
                };
                let child = self.child.invoke_with_env(quote!(env), quote!(v));
                let built =
                    super::trait_impl::build_through_erased_wrappers(&self.source, quote!(__inner))
                        .expect("transparent-wrapper planning accepted this input spelling");
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        #input,
                    ) -> ::core::result::Result<#source, __JniErr> {
                        ::core::result::Result::Ok({
                            let __inner = #child?;
                            #built
                        })
                    }
                )
            }
            Direction::Deconstruct => {
                let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
                let read = super::trait_impl::read_through_erased_wrappers(&self.source, quote!(v))
                    .expect("transparent-wrapper planning accepted this output spelling");
                let child = self.child.invoke_with_env(quote!(env), quote!(__inner));
                syn::parse_quote!(
                    #allow
                    pub(crate) unsafe fn #name<'a>(
                        env: &mut jni::JNIEnv<'a>,
                        v: #source,
                    ) -> ::core::result::Result<#wire_with_lifetime, __JniErr> {
                        ::core::result::Result::Ok({
                            let __inner = #read;
                            #child?
                        })
                    }
                )
            }
        }
    }
}

#[derive(Clone)]
enum JValueBody {
    Ready(syn::Expr),
    /// A fieldless enum decoder. Variant identity and discriminants are Flat
    /// facts; the enum's Rust spelling is deliberately supplied only while
    /// rendering the final file.
    EnumInput {
        diagnostic_name: String,
        source_module: syn::Path,
        enum_name: syn::Ident,
        variants: Vec<(syn::Ident, i64)>,
    },
    /// Array classification and JNI method selection are adapter policy. The
    /// decoder's element and full-array type ascriptions are source spelling,
    /// so its body is built only during final rendering.
    PrimitiveArray(Box<crate::jni::prim_array::PrimArray>),
}

/// How a terminal codec obtains the Rust type in its generated signature.
///
/// A crossing is spelled by [`Emit`] at the final write. Text terminals can
/// instead select a semantic owned or borrowed carrier. The concrete Rust
/// spellings for those carriers are deliberately confined to `render` too.
#[derive(Clone, Copy)]
enum JValueSource {
    Crossing,
    Text(JTextCarrier),
}

/// Adapter-semantic text ownership, before its Rust carrier is spelled.
#[derive(Clone, Copy)]
pub(crate) enum JTextCarrier {
    Owned,
    Borrowed,
}

impl JTextCarrier {
    fn identity(self) -> &'static str {
        match self {
            Self::Owned => "owned_text",
            Self::Borrowed => "borrowed_text",
        }
    }
}

/// One terminal value crossing, retained without source Rust syntax.
///
/// The JNI representation and source-independent conversion policy are safe
/// to freeze during recipe compilation. Source Rust spelling is not: the plan
/// keeps the Flat reading opaque until the writer supplies [`Emit`]. Most
/// codecs retain a ready expression; fixed-size arrays retain their JNI policy
/// and build the source-typed expression only while rendering. Text codecs
/// retain only their ownership semantics; the concrete owned/borrowed Rust
/// carrier is chosen inside final rendering.
#[derive(Clone)]
pub(crate) struct JValueCodecPlan {
    ident: syn::Ident,
    direction: Direction,
    source: TypeRef,
    source_kind: JValueSource,
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
            source_kind: JValueSource::Crossing,
            wire,
            body: JValueBody::Ready(body),
        }
    }

    /// A text codec whose concrete owned/borrowed carrier is selected only
    /// during final rendering.
    pub(crate) fn text(
        direction: Direction,
        source: TypeRef,
        carrier: JTextCarrier,
        wire: syn::Type,
        body: syn::Expr,
    ) -> Self {
        let ident = planned_name_for_key(direction, carrier.identity(), &wire);
        Self {
            ident,
            direction,
            source,
            source_kind: JValueSource::Text(carrier),
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
            source_kind: JValueSource::Crossing,
            wire: spec.wire.clone(),
            body: JValueBody::PrimitiveArray(Box::new(spec)),
        }
    }

    pub(crate) fn enum_input(
        source: TypeRef,
        source_module: syn::Path,
        enum_name: syn::Ident,
        variants: Vec<(syn::Ident, i64)>,
    ) -> Self {
        let wire = syn::parse_quote!(jni::sys::jint);
        let ident = planned_name(Direction::Construct, &source, &wire);
        let diagnostic_name = enum_name.to_string();
        Self {
            ident,
            direction: Direction::Construct,
            source,
            source_kind: JValueSource::Crossing,
            wire,
            body: JValueBody::EnumInput {
                diagnostic_name,
                source_module,
                enum_name,
                variants,
            },
        }
    }

    pub(crate) fn name(&self) -> &syn::Ident {
        &self.ident
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source: syn::Type = match self.source_kind {
            JValueSource::Crossing => emit.spell_ty(&self.source),
            JValueSource::Text(JTextCarrier::Owned) => syn::parse_quote!(String),
            JValueSource::Text(JTextCarrier::Borrowed) => syn::parse_quote!(&str),
        };
        let wire = &self.wire;
        let body = match &self.body {
            JValueBody::Ready(body) => body.clone(),
            JValueBody::EnumInput {
                diagnostic_name,
                source_module,
                enum_name,
                variants,
            } => {
                let arms = variants.iter().map(|(variant, value)| {
                    let value = proc_macro2::Literal::i64_unsuffixed(*value);
                    quote::quote!(#value => #source_module::#enum_name::#variant,)
                });
                syn::parse_quote!({
                    match *v as i64 {
                        #(#arms)*
                        other => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<String>>::from(
                                    format!(
                                        "invalid {} discriminant: {}",
                                        #diagnostic_name,
                                        other
                                    )
                                )
                            );
                        }
                    }
                })
            }
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

    /// Render an already-planned Rust-to-JNI call graph.
    ///
    /// Output pipelines can only contain converter children. The transient
    /// Vec-handle operation is an input-site ABI and is the sole pipeline body
    /// that needs [`Emit`] to spell its element type. Keeping that case out of
    /// this API lets return, error, and callback delivery assemble their JNI
    /// protocol without gaining access to source Rust spelling.
    pub(crate) fn invoke_output(&self, value: TokenStream) -> TokenStream {
        let JPipelineBody::Converter(child) = &self.body else {
            unreachable!("a Rust-to-JNI pipeline cannot contain an input Vec handle")
        };
        let call = child.invoke_with_env(quote!(&mut env), value);
        if child.stages.is_empty() {
            call
        } else {
            quote!((|| -> ::core::result::Result<_, __JniErr> { #call })())
        }
    }

    /// Render a converter pipeline inside another generated converter, where
    /// `env` is already the borrowed `JNIEnv` rather than an owned wrapper
    /// parameter. Whole-object property plans use this to retain child chains
    /// without retaining their generated Rust expressions.
    pub(crate) fn invoke_converter(
        &self,
        env: TokenStream,
        value: TokenStream,
        stage_base: &syn::Ident,
    ) -> TokenStream {
        let JPipelineBody::Converter(child) = &self.body else {
            unreachable!("a whole-object field cannot use the transient Vec-handle site ABI")
        };
        child.invoke_input_value_with_env(env, value, stage_base)
    }

    /// Render the already-planned call graph around `value`.
    pub(crate) fn invoke(&self, value: TokenStream, emit: &Emit) -> TokenStream {
        match &self.body {
            JPipelineBody::Converter(_) => self.invoke_output(value),
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
    /// Render a constructing child as the value it yields inside an enclosing
    /// converter. Unlike the ordinary chain API this consumes `Result` here,
    /// because the enclosing function already owns the error channel. Stage
    /// bindings are named by the property that owns them, preventing sibling
    /// fields from colliding.
    fn invoke_input_value_with_env(
        &self,
        env: TokenStream,
        value: TokenStream,
        stage_base: &syn::Ident,
    ) -> TokenStream {
        assert_eq!(self.direction, Direction::Construct);
        let converter = self.call.ident();
        let value = match self.value_use {
            JValueUse::Direct => value,
            JValueUse::SharedRef => shared_ref(value),
            JValueUse::Cloned => quote!((*#value).clone()),
        };
        let first = quote!(#converter(#env, #value));
        if self.stages.is_empty() {
            return quote!(#first?);
        }
        let first_name = format_ident!("{}_s0", stage_base);
        let mut body = quote!(let #first_name = #first?;);
        let mut previous = first_name;
        for (index, stage) in self.stages.iter().enumerate() {
            let next = format_ident!("{}_s{}", stage_base, index + 1);
            body.extend(quote!(
                let #next = #stage(#env, #previous)
                    .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                        __e.to_string()
                    ))?;
            ));
            previous = next;
        }
        quote!({ #body #previous })
    }

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

/// Which ownership operation an opaque-handle terminal performs.
#[derive(Clone, Copy)]
pub(crate) enum JHandleOperation {
    ConsumeInput,
    BorrowInput,
    OwnOutput,
    CloneOutput,
}

/// One opaque `Box`-handle terminal, kept source-syntax-free until final
/// emission.
///
/// All operations use `jni::sys::jlong`, not `*mut T`, because JNI's wire is
/// 64 bits even on a 32-bit target. Owned output leaks `Box<T>` to the JVM;
/// owned input reclaims it with `Box::from_raw`. Borrowed input instead keeps
/// the allocation JVM-owned through `OwnedObject<T>`, and borrowed output
/// clones the referent into a fresh owned handle.
///
/// Both input operations reject zero and odd values. Zero is the optional
/// niche (`Box::into_raw` cannot produce it). Bit zero is reserved by Kotlin's
/// `NativeHandle` as the closed tag: an odd pointer means close won a race
/// after the wrapper's pre-lock check and must never be dereferenced. This
/// convention depends on the separately emitted `align_of::<T>() >= 2` guard
/// for every opaque handle type.
#[derive(Clone)]
pub(crate) struct JHandleCodecPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) reachable: std::rc::Rc<std::cell::Cell<bool>>,
    pub(crate) source: TypeRef,
    pub(crate) module: syn::Path,
    pub(crate) target: syn::Ident,
    pub(crate) operation: JHandleOperation,
}

impl JHandleCodecPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let mut source = emit.spell_ty(&self.source);
        let mut qualifier = QualifySource {
            module: &self.module,
            target: self.target.clone(),
        };
        syn::visit_mut::VisitMut::visit_type_mut(&mut qualifier, &mut source);
        let allow = crate::jni::trait_impl::generated_converter_attr();
        match self.operation {
            JHandleOperation::ConsumeInput => syn::parse_quote!(
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
            ),
            JHandleOperation::BorrowInput => syn::parse_quote!(
                #allow
                pub(crate) unsafe fn #name<'env, 'v>(
                    env: &mut jni::JNIEnv<'env>,
                    v: &jni::sys::jlong,
                ) -> ::core::result::Result<OwnedObject<#source>, __JniErr> {
                    if *v == 0 || (*v & 1) == 1 {
                        return ::core::result::Result::Err(
                            <__JniErr as ::core::convert::From<String>>::from(
                                "Operation on a closed native handle.".to_string(),
                            ),
                        );
                    }
                    Ok(unsafe { OwnedObject::from_raw(*v as *const #source) })
                }
            ),
            JHandleOperation::OwnOutput => syn::parse_quote!(
                #allow
                pub(crate) unsafe fn #name<'a>(
                    env: &mut jni::JNIEnv<'a>,
                    v: #source,
                ) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
                    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
                }
            ),
            JHandleOperation::CloneOutput => syn::parse_quote!(
                #allow
                pub(crate) unsafe fn #name<'a>(
                    env: &mut jni::JNIEnv<'a>,
                    v: #source,
                ) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
                    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v.clone())) as i64)
                }
            ),
        }
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

/// Stable private name for a converter whose source spelling is deliberately
/// unavailable until final rendering.
pub(crate) fn planned_name(
    direction: Direction,
    source: &TypeRef,
    intermediate: &syn::Type,
) -> syn::Ident {
    let key = source.key();
    planned_name_for_key(direction, key.as_str(), intermediate)
}

/// Stable private name for an adapter operation identified by a model key.
///
/// Hashing treats `TypeKey` as opaque table identity. In particular, this does
/// not obtain its normalized-source label, parse it, or branch on its text;
/// #558 tracks removing that textual capability from `TypeKey` itself.
pub(crate) fn model_operation_name(operation: &str, key: &TypeKey) -> syn::Ident {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let mut hash = DefaultHasher::new();
    operation.hash(&mut hash);
    key.hash(&mut hash);
    format_ident!("{operation}_{:08x}", hash.finish() & 0xffff_ffff)
}

/// Stable private name that keeps a nominal model identity readable while the
/// table key itself remains opaque.
///
/// TypeId is a Flat-model fact, not recovered Rust syntax. The key still
/// supplies uniqueness for wrappers and generic instantiations that share the
/// same nominal identity.
pub(crate) fn named_model_operation_name(
    operation: &str,
    id: &prebindgen_registry::flat::TypeId,
    key: &TypeKey,
) -> syn::Ident {
    let nominal = crate::jni::emit::sanitize_for_ident(&id.name);
    model_operation_name(&format!("{operation}_{nominal}"), key)
}

/// Stable private name for a plan identified by an adapter semantic rather
/// than by a source crossing. This accepts a semantic label, not Rust syntax.
fn planned_name_for_key(
    direction: Direction,
    source_key: &str,
    intermediate: &syn::Type,
) -> syn::Ident {
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

    /// The writer-facing carrier may retain semantic plans and marker
    /// identities, but never a pre-rendered Rust function. Otherwise a new
    /// compatibility caller could silently restore the old eager path.
    #[test]
    fn function_carrier_cannot_store_complete_rust_syntax() {
        let source = include_str!("chain.rs");
        let body = source
            .split_once("enum JBody {")
            .expect("JBody declaration")
            .1
            .split_once("impl JFunction")
            .expect("end of JBody declaration")
            .0;
        assert!(!body.contains("syn::ItemFn"), "{body}");
        assert!(!body.contains("Complete"), "{body}");
    }

    #[test]
    fn compatibility_markers_never_emit() {
        let marker = JFunction::marker(format_ident!("__jni_parts"));
        assert!(!marker.should_emit());
    }

    #[test]
    fn shared_refs_parenthesize_only_compound_values() {
        let bare: syn::ExprReference = syn::parse2(shared_ref(quote!(value))).unwrap();
        assert!(matches!(*bare.expr, syn::Expr::Path(_)));

        let compound: syn::ExprReference = syn::parse2(shared_ref(quote!(value.0))).unwrap();
        assert!(matches!(*compound.expr, syn::Expr::Paren(_)));
    }
}
