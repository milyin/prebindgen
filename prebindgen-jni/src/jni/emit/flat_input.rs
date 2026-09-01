//! Flattenable `data_class` inputs: leaf plans, Kotlin destructure
//! expressions, and the Rust-side reconstruct.

// `flat` as a module for `TypeKind`: the bare name in this scope is jnigen's own
// classifier (via `use super::*`), and an explicit import would shadow it.
use prebindgen_registry::{
    flat::{self, TypeRef},
    Conversions,
};

use super::*;
use crate::jni::trait_impl::build_through_erased_wrappers;

/// Takes the **element**, not the `syn::ItemStruct` it was parsed from (#289):
/// `flat::Field::ty` is already a `TypeRef`, so every peel below is the model's
/// answer rather than a last-path-segment test on tokens that had a reading one
/// level up. Same move `build_flat_struct_node` made for the flatten path; this
/// is the whole-object `.jobject_input()` decoder.
#[derive(Clone)]
pub(crate) struct JObjectStructInputPlan {
    shape: flat::Struct,
    fields: Vec<JObjectStructFieldPlan>,
}

impl JObjectStructInputPlan {
    /// Every converter the decoder calls, one per property.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        for field in &self.fields {
            field.kind.calls(out);
        }
    }
}

#[derive(Clone)]
struct JObjectStructFieldPlan {
    name: syn::Ident,
    property: String,
    error: String,
    sum_payload: bool,
    kind: JObjectStructFieldKind,
}

impl JObjectStructFieldKind {
    /// The pipeline this property decodes through.
    fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        match self {
            Self::Handle { pipeline, .. }
            | Self::Unsigned64 { pipeline, .. }
            | Self::Enum { pipeline, .. }
            | Self::Primitive { pipeline, .. }
            | Self::IntoObject { pipeline, .. }
            | Self::Object { pipeline, .. } => pipeline.calls(out),
            Self::Nullable(plan) => plan.chain.child.calls(out),
        }
    }
}

/// The null-reference `Optional` layer a nullable whole-object property
/// crosses as.
///
/// A `ULong?` property arrives as a boxed `kotlin.ULong` and a nullable enum
/// class as its own class: absence is a null reference, and the value is one
/// unboxing call away. The registry composes the layer; this supplies the two
/// answers only JniGen can give — how to test absence, and how to unbox.
#[derive(Clone)]
struct JNullableProperty {
    /// JVM method that unboxes the property, its signature, and the `JValue`
    /// accessor for the result.
    method: &'static str,
    signature: &'static str,
    accessor: syn::Ident,
    /// Per-property error text.
    error: String,
    /// Whether the child already yields the optional, because its own wire
    /// carries a niche encoding of absence. Then a non-null reference is not
    /// yet a value: the child still has to read its sentinel.
    flattens: bool,
}

impl prebindgen_registry::chain::OptionalBridge for JNullableProperty {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn is_absent(&self) -> TokenStream {
        quote!(v.is_null())
    }

    fn present(&self, value: TokenStream) -> TokenStream {
        let (method, signature, accessor, error) =
            (self.method, self.signature, &self.accessor, &self.error);
        // The accessor already yields the wire type, so the composer's own
        // binding needs no annotation of its own.
        quote!(
            env.call_method(&#value, #method, #signature, &[])
                .and_then(|val| val.#accessor())
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                    format!(#error, e)
                ))?
        )
    }

    fn source_present(&self, child: TokenStream) -> TokenStream {
        if self.flattens {
            child
        } else {
            quote!(::core::option::Option::Some(#child))
        }
    }

    fn build_absent(&self) -> TokenStream {
        unreachable!("a whole-object property is decoded, never encoded")
    }

    fn build_present(&self, _child: TokenStream) -> TokenStream {
        unreachable!("a whole-object property is decoded, never encoded")
    }
}

/// The sealed interface a whole-object sum crosses as: one JVM object whose
/// class names the live alternative.
///
/// Selection is a run of `instanceof` tests, in declaration order, which this
/// reads once into a tag the composed `Choice` matches on. Absence of a match
/// and a null reference are distinct failures, and both keep the message they
/// had.
#[derive(Clone)]
struct JSumBridge {
    /// One JVM class per alternative, in declaration order.
    classes: Vec<String>,
    /// The sum's Kotlin name, for the two failure messages.
    enum_name: String,
}

impl prebindgen_registry::chain::ChoiceBridge for JSumBridge {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn tag(&self, value: TokenStream) -> TokenStream {
        let enum_name = &self.enum_name;
        let null = format!("{enum_name}: null value where a variant was required");
        let tests = self.classes.iter().enumerate().map(|(index, class)| {
            let index = index as i32;
            let error = format!("{enum_name}: instanceof {class}: {{}}");
            quote! {
                if env.is_instance_of(#value, #class)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                        format!(#error, e)
                    ))?
                {
                    return ::core::result::Result::Ok(#index);
                }
            }
        });
        quote!({
            if #value.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(#null.to_string()),
                );
            }
            let __tag = (|| -> ::core::result::Result<i32, __JniErr> {
                #(#tests)*
                ::core::result::Result::Ok(-1i32)
            })()?;
            __tag
        })
    }

    fn arm(
        &self,
        _emit: &prebindgen_registry::RustWriter,
        value: TokenStream,
        _index: usize,
    ) -> TokenStream {
        // Every alternative's payload is read off the same object; which
        // properties belong to it is the arm's own bridge.
        value
    }

    fn build(
        &self,
        _emit: &prebindgen_registry::RustWriter,
        _active: usize,
        _value: TokenStream,
    ) -> TokenStream {
        unreachable!("a whole-object sum is decoded, never encoded, through this bridge")
    }

    fn invalid_tag(&self, _tag: TokenStream) -> TokenStream {
        let message = format!(
            "{}: value is not one of its declared variants",
            self.enum_name
        );
        quote!(<__JniErr as ::core::convert::From<String>>::from(#message.to_string()))
    }
}

/// The source struct a whole-object decode builds, spelled through the shape
/// the model holds.
///
/// A `data_class` is normally a named-field struct, and the registry's default
/// construction is exactly that. A unit or tuple struct is not, and only the
/// element knows which — so it travels with the policy that constructs it.
#[derive(Clone)]
struct JStructSource {
    ordinary: crate::jni::chain::JSource,
    shape: flat::Struct,
}

impl prebindgen_registry::chain::Source for JStructSource {
    fn spell(&self, source: &TypeRef, emit: &prebindgen_registry::RustWriter) -> TokenStream {
        prebindgen_registry::chain::Source::spell(&self.ordinary, source, emit)
    }

    fn build(&self, canonical: TokenStream) -> TokenStream {
        prebindgen_registry::chain::Source::build(&self.ordinary, canonical)
    }

    fn construct(
        &self,
        head: TokenStream,
        parts: &[(syn::Ident, TokenStream)],
        emit: &prebindgen_registry::RustWriter,
    ) -> TokenStream {
        let bound: Vec<TokenStream> = self
            .shape
            .fields
            .iter()
            .zip(parts)
            .map(|(field, (_, value))| field.bind(value))
            .collect();
        emit.shape_struct(&self.shape, head, &bound)
    }
}

/// The JVM object a whole-object struct crosses as, read one property at a
/// time.
///
/// This is the adapter half of the composed `Product`: the registry walks the
/// struct's fields and builds the source value; this says what a part *is* —
/// a property fetched off the object, which can fail.
#[derive(Clone)]
struct JObjectBridge {
    properties: Vec<JObjectStructFieldPlan>,
}

impl prebindgen_registry::chain::ProductBridge for JObjectBridge {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn part(
        &self,
        value: TokenStream,
        index: usize,
        _name: &syn::Ident,
    ) -> prebindgen_registry::chain::PartRead {
        let (prelude, value) = self.properties[index].read(value);
        prebindgen_registry::chain::PartRead::fallible(prelude, value)
    }

    fn build(&self, _parts: &[(syn::Ident, TokenStream)]) -> TokenStream {
        unreachable!("a whole-object struct is decoded, never encoded, through this bridge")
    }
}

/// What converts one property once its wire value has been read.
///
/// Either the property's own child converter, or — for a nullable property —
/// the `Optional` layer composed over it, which decides absence before the
/// child is reached.
#[derive(Clone)]
pub(crate) struct JPropertyChild {
    call: prebindgen_registry::chain::Call,
    conversion: JPropertyConversion,
}

#[derive(Clone)]
enum JPropertyConversion {
    Converter(crate::jni::chain::JChild),
    Nullable(
        Box<
            prebindgen_registry::chain::Optional<
                crate::jni::chain::JSource,
                JNullableProperty,
                crate::jni::chain::JChild,
            >,
        >,
    ),
}

impl prebindgen_registry::chain::Child for JPropertyChild {
    fn call(&self) -> &prebindgen_registry::chain::Call {
        &self.call
    }

    fn invoke(&self, value: TokenStream, emit: &prebindgen_registry::RustWriter) -> TokenStream {
        match &self.conversion {
            JPropertyConversion::Converter(child) => {
                prebindgen_registry::chain::Child::invoke(child, value, emit)
            }
            // The composed layer names its input `v`, as every chain does, so
            // the block is what keeps that name local to this property.
            JPropertyConversion::Nullable(chain) => {
                let body = prebindgen_registry::chain::Chain::render(chain.as_ref(), emit).body;
                quote!({
                    let v = #value;
                    #body
                })
            }
        }
    }
}

/// One nullable property: the reference fetched off the JVM object, and the
/// registry-composed `Optional` that turns it into the source value.
#[derive(Clone)]
struct JNullablePlan {
    descriptor: String,
    chain: Box<
        prebindgen_registry::chain::Optional<
            crate::jni::chain::JSource,
            JNullableProperty,
            crate::jni::chain::JChild,
        >,
    >,
}

/// Compose one nullable property's `Optional` layer over its child converter.
#[allow(clippy::too_many_arguments)]
fn nullable_property(
    reading: &TypeRef,
    pipeline: crate::jni::chain::JPipeline,
    descriptor: String,
    method: &'static str,
    signature: &'static str,
    accessor: syn::Ident,
    error: String,
    flattens: bool,
) -> JObjectStructFieldKind {
    JObjectStructFieldKind::Nullable(Box::new(JNullablePlan {
        descriptor,
        chain: Box::new(prebindgen_registry::chain::Optional {
            source: reading.clone(),
            direction: prebindgen_registry::recipe::Direction::Construct,
            source_policy: crate::jni::chain::JSource {
                wrappers: Vec::new(),
            },
            bridge: JNullableProperty {
                method,
                signature,
                accessor,
                error,
                flattens,
            },
            child: pipeline.converter_child().clone(),
        }),
    }))
}

#[derive(Clone)]
enum JObjectStructFieldKind {
    Handle {
        descriptor: String,
        pipeline: crate::jni::chain::JPipeline,
    },
    Unsigned64 {
        pipeline: crate::jni::chain::JPipeline,
    },
    Enum {
        descriptor: String,
        pipeline: crate::jni::chain::JPipeline,
    },
    /// A property whose absence is a null reference: either shape above, when
    /// the source reads optional. Its `Optional` layer is composed by the
    /// registry rather than walked here.
    Nullable(Box<JNullablePlan>),
    Primitive {
        wire: syn::Type,
        descriptor: String,
        accessor: syn::Ident,
        pipeline: crate::jni::chain::JPipeline,
    },
    IntoObject {
        wire: syn::Type,
        descriptor: String,
        pipeline: crate::jni::chain::JPipeline,
    },
    Object {
        descriptor: String,
        pipeline: crate::jni::chain::JPipeline,
    },
}

/// Freeze the explicit whole-`JObject` struct decoder without assembling Rust
/// source. Every branch below selects JVM property policy from Flat readings
/// and already-compiled registry fragments. The final writer alone constructs
/// bindings, child calls, and the source struct literal.
pub(crate) fn build_jobject_struct_input_plan(
    ext: &Declarations,
    s: &flat::Struct,
    registry: &impl Conversions,
) -> Option<JObjectStructInputPlan> {
    let struct_name = s.name.to_string();
    let mut fields = Vec::new();
    for field in &s.fields {
        let name = field.name.clone()?;
        let property = kotlin_property_name(&name);
        let error = format!("{struct_name}.{property}: {{}}");
        fields.push(build_jobject_property_plan(
            ext, &field.ty, name, property, error, false,
        )?);
    }
    let _ = registry;
    Some(JObjectStructInputPlan {
        shape: s.clone(),
        fields,
    })
}

fn build_jobject_property_plan(
    ext: &Declarations,
    reading: &TypeRef,
    name: syn::Ident,
    property: String,
    error: String,
    sum_payload: bool,
) -> Option<JObjectStructFieldPlan> {
    let optional = reading.optional_inner().is_some();
    let inner = reading.optional_inner().unwrap_or(reading);
    let entry = ext.in_frag(reading)?;
    let wire = entry.destination().clone();
    let projection = entry.projection();
    let whole = entry.pipeline(
        prebindgen_registry::recipe::Direction::Construct,
        prebindgen_registry::recipe::Mode::Owned,
    );
    let kind = if let Some(projection) = projection
        .as_ref()
        .filter(|projection| matches!(&projection.kind, ProjectionKind::Handle))
    {
        JObjectStructFieldKind::Handle {
            descriptor: format!("L{};", handle_field_fqn(ext, projection).replace('.', "/")),
            pipeline: whole,
        }
    } else if let Some(projection) = projection
        .as_ref()
        .filter(|projection| !sum_payload && matches!(&projection.kind, ProjectionKind::Unsigned64))
    {
        let niche = optional
            && matches!(
                projection.strategy,
                FoldStrategy::Optional(NullableKind::Niche, _)
            );
        if optional {
            // A niche-encoded child reads its own absence out of the unboxed
            // value, so the null reference is only the first of two tests and
            // the child's answer is the whole answer.
            let pipeline = if niche {
                whole
            } else {
                ext.in_frag(inner)?.pipeline(
                    prebindgen_registry::recipe::Direction::Construct,
                    prebindgen_registry::recipe::Mode::Owned,
                )
            };
            nullable_property(
                reading,
                pipeline,
                "Lkotlin/ULong;".to_string(),
                "unbox-impl",
                "()J",
                format_ident!("j"),
                error.clone(),
                niche,
            )
        } else {
            JObjectStructFieldKind::Unsigned64 { pipeline: whole }
        }
    } else if ext.is_kotlin_enum_reading(inner) {
        let fqn = match inner.unwrapped().kind() {
            flat::TypeKind::Named { id, .. } => id.ident(),
            _ => None,
        }
        .and_then(|ident| ext.kotlin_fqn(&TypeKey::from_ident(&ident)))?;
        let descriptor = format!("L{};", fqn.replace('.', "/"));
        if optional {
            let pipeline = ext.in_frag(inner)?.pipeline(
                prebindgen_registry::recipe::Direction::Construct,
                prebindgen_registry::recipe::Mode::Owned,
            );
            nullable_property(
                reading,
                pipeline,
                descriptor,
                "getValue",
                "()I",
                format_ident!("i"),
                error.clone(),
                false,
            )
        } else {
            JObjectStructFieldKind::Enum {
                descriptor,
                pipeline: whole,
            }
        }
    } else {
        match jni_field_access(&wire) {
            Some((descriptor, accessor, false)) => JObjectStructFieldKind::Primitive {
                wire,
                descriptor: descriptor.to_string(),
                accessor,
                pipeline: whole,
            },
            Some((descriptor, _, true)) => JObjectStructFieldKind::IntoObject {
                wire,
                descriptor: descriptor.to_string(),
                pipeline: whole,
            },
            None => {
                let descriptor = ext
                    .in_frag(inner)
                    .and_then(|entry| jni_field_access(entry.destination()))
                    .and_then(|(descriptor, _, is_object)| {
                        if is_object {
                            Some(descriptor.to_string())
                        } else {
                            box_descriptor_for_primitive(descriptor).map(str::to_string)
                        }
                    })
                    .or_else(|| {
                        match inner.unwrapped().kind() {
                            flat::TypeKind::Named { id, .. } => id.ident(),
                            _ => None,
                        }
                        .and_then(|ident| {
                            ext.kotlin_fqn(&TypeKey::from_ident(&ident))
                                .map(|fqn| format!("L{};", fqn.replace('.', "/")))
                        })
                    })
                    .or_else(|| {
                        inner
                            .sequence_elem()
                            .is_some()
                            .then(|| "Ljava/util/List;".to_string())
                    })
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                JObjectStructFieldKind::Object {
                    descriptor,
                    pipeline: whole,
                }
            }
        }
    };
    Some(JObjectStructFieldPlan {
        name,
        property,
        error,
        sum_payload,
        kind,
    })
}

impl JObjectStructInputPlan {
    /// The decode, composed by the registry: it walks the struct's fields,
    /// reads each part through [`JObjectBridge`], calls each property's
    /// conversion, and builds the source value.
    pub(crate) fn render(&self, emit: &prebindgen_registry::RustWriter) -> syn::Expr {
        let product = prebindgen_registry::chain::Product {
            source: self.shape.type_ref().clone(),
            direction: prebindgen_registry::recipe::Direction::Construct,
            source_policy: JStructSource {
                ordinary: crate::jni::chain::JSource {
                    wrappers: Vec::new(),
                },
                shape: self.shape.clone(),
            },
            bridge: JObjectBridge {
                properties: self.fields.clone(),
            },
            parts: self
                .fields
                .iter()
                .map(|field| prebindgen_registry::chain::ProductPart {
                    name: field.name.clone(),
                    child: field.child(),
                    mode: prebindgen_registry::recipe::Mode::Owned,
                    hold_uninit: false,
                })
                .collect(),
        };
        let body = prebindgen_registry::chain::Chain::render(&product, emit).body;
        syn::parse_quote!(#body)
    }
}

impl JObjectStructFieldPlan {
    /// The statements that read this property's wire value off `receiver`,
    /// and the expression naming it.
    ///
    /// Every read can fail — a JVM field access returns a `Result` — which is
    /// why the composed `Product` above takes this as a fallible part.
    fn read(&self, receiver: TokenStream) -> (TokenStream, TokenStream) {
        let name = &self.name;
        let property = &self.property;
        let error = &self.error;
        let raw = if self.sum_payload {
            format_ident!("{}_raw", name)
        } else {
            format_ident!("__{}_raw", name)
        };
        let object = if self.sum_payload {
            format_ident!("{}_obj", name)
        } else {
            format_ident!("__{}_jobj", name)
        };
        let fetch_object = |descriptor: &String| {
            quote! {
                let #object: jni::objects::JObject = env.get_field(#receiver, #property, #descriptor)
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))?;
            }
        };
        match &self.kind {
            JObjectStructFieldKind::Handle { descriptor, .. } => {
                let fetch = fetch_object(descriptor);
                (
                    quote! {
                        #fetch
                        let #raw: jni::sys::jlong = if #object.is_null() {
                            0
                        } else {
                            env.call_method(&#object, "peek", "()J", &[])
                                .and_then(|val| val.j())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))?
                        };
                    },
                    quote!(#raw),
                )
            }
            JObjectStructFieldKind::Unsigned64 { .. } => (
                quote! {
                    let #raw: jni::sys::jlong = env
                        .get_field(#receiver, #property, "J")
                        .and_then(|val| val.j())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))?;
                },
                quote!(#raw),
            ),
            JObjectStructFieldKind::Enum { descriptor, .. } => {
                let fetch = fetch_object(descriptor);
                (
                    quote! {
                        #fetch
                        let #raw: jni::sys::jint = env.call_method(&#object, "getValue", "()I", &[])
                            .and_then(|val| val.i())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))?;
                    },
                    quote!(#raw),
                )
            }
            // The reference itself is the wire value: whether it means absence
            // is the composed `Optional` layer's decision, not the read's.
            JObjectStructFieldKind::Nullable(plan) => {
                (fetch_object(&plan.descriptor), quote!(#object))
            }
            JObjectStructFieldKind::Primitive {
                wire,
                descriptor,
                accessor,
                ..
            } => (
                quote! {
                    let #raw: #wire = env.get_field(#receiver, #property, #descriptor)
                        .and_then(|val| val.#accessor())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))? as _;
                },
                quote!(#raw),
            ),
            JObjectStructFieldKind::IntoObject {
                wire, descriptor, ..
            } => {
                let fetch = fetch_object(descriptor);
                (
                    quote! {
                        #fetch
                        let #raw: #wire = #object.into();
                    },
                    quote!(#raw),
                )
            }
            JObjectStructFieldKind::Object { descriptor, .. } => (
                quote! {
                    let #raw: jni::objects::JObject = env.get_field(#receiver, #property, #descriptor)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#error, e)))?;
                },
                quote!(#raw),
            ),
        }
    }

    /// What converts this property once it is read.
    fn child(&self) -> JPropertyChild {
        let conversion = match &self.kind {
            JObjectStructFieldKind::Nullable(plan) => {
                JPropertyConversion::Nullable(plan.chain.clone())
            }
            JObjectStructFieldKind::Handle { pipeline, .. }
            | JObjectStructFieldKind::Unsigned64 { pipeline }
            | JObjectStructFieldKind::Enum { pipeline, .. }
            | JObjectStructFieldKind::Primitive { pipeline, .. }
            | JObjectStructFieldKind::IntoObject { pipeline, .. }
            | JObjectStructFieldKind::Object { pipeline, .. } => {
                JPropertyConversion::Converter(pipeline.converter_child().clone())
            }
        };
        let call = match &conversion {
            JPropertyConversion::Converter(child) => {
                prebindgen_registry::chain::Child::call(child).clone()
            }
            // The layer propagates its own failures with `?`, so the composer
            // adds none of its own; the identity is the child's, which is what
            // the property depends on.
            JPropertyConversion::Nullable(chain) => prebindgen_registry::chain::Call::operation(
                prebindgen_registry::chain::Child::call(&chain.child)
                    .operation_id()
                    .clone(),
                false,
                false,
            ),
        };
        JPropertyChild { call, conversion }
    }
}

/// Whole-object **input** decode for a `sealed_class` sum: a `JObject` of the
/// sealed interface → the Rust enum, by `instanceof` dispatch over the nested
/// variant classes and a property read per payload.
///
/// This is the counterpart of a nested data class's own input converter, not
/// a second mechanism: a sum reached as a **field** sits inside a parent
/// `JObject` that is already being read field-by-field, so the parent's
/// generic field branch just delegates here — the same delegation
/// `Annotated.payload` uses. The tag-gated *flattened* form is the separate
/// parameter path.
///
/// The output direction deliberately has no such converter: a sum crosses
/// Rust → Kotlin **flattened, always** (`PlanFieldKind::Sum`), which is the
/// design's rejected-alternative note about per-crossing JVM objects. Reading
/// one field out of a `JObject` the caller already handed us costs nothing
/// extra, so the asymmetry is real rather than an oversight.
/// Takes the **element**, not the `syn::ItemEnum` it was parsed from (#289):
/// `Alternative::fields` carries a `TypeRef` per payload, so the property read
/// below asks the model instead of peeling tokens. It also retires the two zips
/// this used to run — a `SumSpec` derived from the item, paired back against the
/// item it came from — because `Alternative` already is that pairing.
#[derive(Clone)]
pub(crate) struct JObjectSumInputPlan {
    shape: flat::Variant,
    enum_name: String,
    alternatives: Vec<JObjectSumAlternativePlan>,
}

#[derive(Clone)]
struct JObjectSumAlternativePlan {
    shape: flat::Alternative,
    jvm_class: String,
    /// One property per payload field, in declaration order — the same plan a
    /// struct's field carries, since a payload is read exactly as a field is.
    fields: Vec<JObjectStructFieldPlan>,
}

impl JObjectSumInputPlan {
    /// Every converter the decoder calls, one per property of every
    /// alternative.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        for field in self
            .alternatives
            .iter()
            .flat_map(|alternative| &alternative.fields)
        {
            field.kind.calls(out);
        }
    }
}

pub(crate) fn build_jobject_sum_input_plan(
    ext: &Declarations,
    v: &flat::Variant,
    registry: &impl Conversions,
) -> Option<JObjectSumInputPlan> {
    let key = TypeKey::from_ident(&v.name);
    let cfg = ext.types.get(&key)?;
    let sum_cfg = cfg.sum()?;
    let iface_fqn = cfg.name_spec.as_ref().map(|s| ext.fqn_of(s))?;
    let iface_path = iface_fqn.replace('.', "/");
    let enum_name = v.name.to_string();
    let mut alternatives = Vec::new();
    for alt in &v.alternatives {
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, &alt.name);
        let jvm_class = format!("{iface_path}${kotlin_name}");
        let mut fields = Vec::new();
        for field in &alt.fields {
            let property = crate::jni::struct_plan::sum_field_prop_name(&field.member());
            let bind = format_ident!("__p_{}", property);
            let error = format!("{enum_name}.{kotlin_name}.{property}: {{}}");
            fields.push(build_jobject_property_plan(
                ext, &field.ty, bind, property, error, true,
            )?);
        }
        alternatives.push(JObjectSumAlternativePlan {
            shape: alt.clone(),
            jvm_class,
            fields,
        });
    }
    let _ = registry;
    Some(JObjectSumInputPlan {
        shape: v.clone(),
        enum_name,
        alternatives,
    })
}

impl JObjectSumInputPlan {
    /// The decode, composed by the registry: it reads the tag, selects the
    /// arm, reads that arm's payload properties through the same bridge the
    /// struct decode uses, and constructs the alternative.
    pub(crate) fn render(&self, emit: &prebindgen_registry::RustWriter) -> syn::Expr {
        let choice = prebindgen_registry::chain::Choice {
            source: self.shape.type_ref().clone(),
            direction: prebindgen_registry::recipe::Direction::Construct,
            source_policy: crate::jni::chain::JSource {
                wrappers: Vec::new(),
            },
            bridge: JSumBridge {
                classes: self
                    .alternatives
                    .iter()
                    .map(|alternative| alternative.jvm_class.clone())
                    .collect(),
                enum_name: self.enum_name.clone(),
            },
            arms: self
                .alternatives
                .iter()
                .enumerate()
                .map(
                    |(index, alternative)| prebindgen_registry::chain::ChoiceArm {
                        alternative: alternative.shape.clone(),
                        tag: {
                            let index = index as i32;
                            syn::parse_quote!(#index)
                        },
                        bridge: JObjectBridge {
                            properties: alternative.fields.to_vec(),
                        },
                        parts: alternative
                            .fields
                            .iter()
                            .map(|field| prebindgen_registry::chain::ChoicePart {
                                child: field.child(),
                                mode: prebindgen_registry::recipe::Mode::Owned,
                                hold_uninit: false,
                            })
                            .collect(),
                    },
                )
                .collect(),
        };
        let body = prebindgen_registry::chain::Chain::render(&choice, emit).body;
        syn::parse_quote!(#body)
    }
}

// ──────────────────────────────────────────────────────────────────────
// Struct input flattening (pass a data_class param as its leaf fields)
// ──────────────────────────────────────────────────────────────────────

/// One flattened leaf of a struct **input** param — the input direction's own
/// slot: instead of reading the field with `env.get_field(...)` out of a single
/// `JObject`, the leaf crosses the JNI boundary as its own wrapper parameter,
/// and the three coordinated sites — the
/// native wrapper signature, the `JNINative` extern declaration and the Kotlin
/// call-site destructure — read it so they cannot drift in order, type, or
/// nullability.
///
/// One JNI parameter a flattened `data_class` occupies, as the site names it.
///
/// A projection of the fragment's [`Wire`](crate::jni::compile::Wire) and
/// nothing more: the wire says what crosses and how Kotlin reaches it, and this
/// pairs that with the one thing a wire cannot know — the parameter name the
/// site hangs it off.
pub(crate) struct FlatLeaf {
    /// Native wrapper parameter ident — also the decode source.
    pub native_ident: syn::Ident,
    /// Kotlin `external fun` parameter name (camelCase).
    pub kt_name: String,
    /// The wire itself.
    pub wire: crate::jni::compile::Wire,
}

impl FlatLeaf {
    /// This wire as one parameter of the site named `param`.
    fn of(param: &syn::Ident, wire: &crate::jni::compile::Wire) -> Self {
        let native = format!("{param}_{}", wire.path.replace('.', "_"));
        Self {
            native_ident: format_ident!("{native}"),
            kt_name: snake_to_camel(&native),
            wire: wire.clone(),
        }
    }

    /// Native wire type, lifetime-annotated for object wires.
    pub fn native_wire_ty(&self) -> TokenStream {
        annotate_jobject_with_lifetime(&self.wire.ty, "a").to_token_stream()
    }

    /// Kotlin `external fun` parameter type (incl. a trailing `?`).
    pub fn kt_wire_ty(&self) -> &str {
        &self.wire.kt_ty
    }

    /// Per-field input converter operation (`None` for a synthetic gate or tag).
    pub fn conv(&self) -> Option<&prebindgen_registry::OperationId> {
        self.wire.conv()
    }

    /// The complete conversion, stages included, or `None` for a gate or tag.
    pub fn entry(&self) -> Option<&prebindgen_registry::ConverterImpl<KotlinMeta>> {
        self.wire.entry.as_ref()
    }

    /// Whether this is a synthetic `…Present: Boolean` gate.
    pub fn is_present_flag(&self) -> bool {
        self.wire.is_present_flag()
    }

    /// Whether the handle access can be null, either because the field itself
    /// is optional or because an optional ancestor gates it.
    pub fn handle_nullable(&self) -> bool {
        self.wire.handle_nullable
    }

    /// Kotlin call-site destructure expression feeding this leaf, rooted at
    /// `base` — the object expression at this call site (the camelCase param
    /// name, `this` for a promoted receiver, `__e` for the vec-build loop
    /// variable).
    pub fn kt_access(&self, base: &str) -> String {
        self.wire.access.render(base)
    }

    /// The Kotlin expression yielding the handle this leaf carries, rooted at
    /// `base` — the thing the lock scaffold locks and `markConsumed()`s.
    /// `None` when the leaf is not a handle.
    pub fn kt_handle_target(&self, base: &str) -> Option<String> {
        self.wire
            .handle_target
            .as_ref()
            .map(|walk| crate::jni::compile::reached(base, walk))
    }

    /// Native call argument for this leaf. Handle pointers are bound under
    /// the unified lock scaffold; every other leaf is read directly from the
    /// Kotlin object graph.
    pub fn kt_call_arg(&self, base: &str) -> String {
        if self.wire.handle_target.is_some() {
            format!("{}_ptr", self.kt_name)
        } else {
            self.kt_access(base)
        }
    }
}

/// The outer layer a flattened call argument restores after its registry chain
/// returns the value represented by the crossing.
///
/// Product/Optional/Choice construction belongs to the registry chain. The
/// renderer only adds the source call's borrow; wrappers outside that borrow
/// still belong afterwards (`Box<&S>` -> `Box::new(&value)`).
pub(crate) struct RebuildTarget {
    /// Wrappers over the borrow, if there is one — the `Box` of `Box<&S>`.
    arg: Vec<&'static str>,
    borrow: RebuildBorrow,
}

#[derive(Clone, Copy)]
enum RebuildBorrow {
    None,
    Outer { mutable: bool },
    OptionalInner { mutable: bool },
}

impl RebuildTarget {
    /// Put back the wrappers over the **borrow** — the `Box` of `Box<&S>`,
    /// which goes on after the call site has added its `&`.
    ///
    /// A no-op when the parameter is not a borrow: wrappers at or below the
    /// crossing are already restored by the registry chain.
    pub fn call_arg(&self, ident: &syn::Ident) -> TokenStream {
        let value = match self.borrow {
            RebuildBorrow::None => quote!(#ident),
            RebuildBorrow::Outer { mutable: false } => quote!(&#ident),
            RebuildBorrow::Outer { mutable: true } => quote!(&mut #ident),
            RebuildBorrow::OptionalInner { mutable: false } => quote!(#ident.as_ref()),
            RebuildBorrow::OptionalInner { mutable: true } => quote!(#ident.as_mut()),
        };
        if !matches!(self.borrow, RebuildBorrow::Outer { .. }) {
            return value;
        }
        crate::jni::trait_impl::build_through_wrappers(&self.arg, value)
            .expect("every layer was checked buildable when the plan was built")
    }

    fn mutable_binding(&self) -> bool {
        matches!(
            self.borrow,
            RebuildBorrow::Outer { mutable: true } | RebuildBorrow::OptionalInner { mutable: true }
        )
    }
}

/// Descend an outer borrow, an `Option`, or the borrow inside an `Option` to the
/// struct a specialized lowering will **rebuild**, keeping each layer's reading
/// so its spelling can be restored.
///
/// One function because it is one rule, and the layers are checked **on the way
/// down**: an erasure sits outside the layer it wraps, so `Box<&S>` classifies
/// as `Ref` and interpreting `kind` before asking would discard the `Box`.
///
/// The only refusal left is a wrapper the adapter cannot **build** — `Cow`, by
/// policy rather than by impossibility (see its `WRAPPER_OPS` recipe). A wrapped
/// spelling that declines here keeps the general converter path, which is
/// correct if less direct.
fn rebuildable_target(arg: &TypeRef) -> Option<(RebuildTarget, &TypeRef)> {
    // A probe per layer: "can this spelling be rebuilt at all", asked before the
    // peel that would hide it. The token is irrelevant — only the `Option` is.
    let buildable = |t: &TypeRef| build_through_erased_wrappers(t, quote!(__probe)).map(|_| ());
    buildable(arg)?;
    let (borrow, t1) = match arg.unwrapped().kind() {
        flat::TypeKind::Ref { mutable, inner, .. } => {
            (RebuildBorrow::Outer { mutable: *mutable }, inner.as_ref())
        }
        _ => (RebuildBorrow::None, arg),
    };
    buildable(t1)?;
    let mut inner = t1.optional_inner().unwrap_or(t1);
    // The chain must also be able to rebuild the crossing's inner spelling.
    buildable(inner)?;
    let borrow = if matches!(borrow, RebuildBorrow::None) {
        match inner.kind() {
            flat::TypeKind::Ref {
                mutable,
                inner: target,
                ..
            } if arg.erased_wrappers().is_empty() => {
                inner = target.as_ref();
                buildable(inner)?;
                RebuildBorrow::OptionalInner { mutable: *mutable }
            }
            _ => borrow,
        }
    } else {
        borrow
    };
    // Only wrappers outside the borrow are kept. Inner wrappers are already
    // part of the registry chain's source policy.
    Some((
        RebuildTarget {
            arg: arg.erased_wrappers(),
            borrow,
        },
        inner,
    ))
}

/// A flattened plan for one struct input parameter. Built once by
/// [`build_flat_input_plan`] and consumed by all three codegen sites.
pub(crate) struct FlatInputPlan {
    pub leaves: Vec<FlatLeaf>,
    /// Registry-composed source converter over those leaves. Planning verifies
    /// its presence and leaf arity; cross-artifact and runtime tests verify that
    /// the ordered leaves have the same meaning on both sides of JNI. There is
    /// no adapter-side reconstruction path.
    pub chain: crate::jni::compile::ComposedChain,
    /// Source identity retained only for the deliberately non-recursive Vec
    /// push helper, which constructs one simple element literal.
    pub struct_module: syn::Path,
    pub struct_ident: syn::Ident,
    /// Whether any Product child is itself composed. The specialized Vec
    /// helper only understands a Product of terminal leaves.
    pub contains_composed_child: bool,
    /// The layer readings the rebuild has to satisfy — carried rather than
    /// re-derived at the emission sites, so the descent is stated once.
    pub target: RebuildTarget,
}

// `impl_into_target` lived here: it extracted `S` from an `impl Into<S> + …`
// spelling for `build_flat_input_plan`'s struct-target peel. It is gone because
// that peel now takes a reading, and the model REFUSES `impl Trait` that is not
// the callback form (`UnsupportedTypeReason::DisallowedImplTrait`) — so a
// parameter spelled `impl Into<S>` never becomes a `TypeRef` and never reached
// the call. `cargo check` confirmed it dead rather than the reasoning alone.
// jnigen's actual `impl Into<…>` support is elsewhere: plugin wrapper exts build
// a converter artifact identity via `Declarations::input_converter_name`,
// which never consults this.
// `flat_probe_inner` lived here: it peeled `&` then `Option` off a SPELLING to
// reach the type an enum probe should ask about. Its last caller now asks
// `is_kotlin_enum_reading`, whose `enum_probe` peels the same two layers off the
// model — so `Box<Priority>` probes as `Priority` where this answered about the
// wrapper (#289).

/// Kotlin literal that fills a leaf slot when its `Option<struct>` parent is
/// absent (the `present` flag tells Rust to ignore it). `None` for nullable
/// leaves, which simply ride a JVM `null`. Mirrors
/// [`primitive_default_for_descriptor`] on the Rust side.
pub(crate) fn kt_leaf_default(sig: &str, nullable: bool) -> Option<String> {
    if nullable {
        return None;
    }
    Some(
        match sig {
            "Z" => "false",
            "B" | "S" | "I" => "0",
            "C" => "'\\u0000'",
            "J" => "0L",
            "F" => "0.0f",
            "D" => "0.0",
            "Ljava/lang/String;" => "\"\"",
            other => {
                // An inert primitive-array slot is an EMPTY array of its own
                // type, never null — the slot is non-nullable in the factory.
                if let Some(n) = crate::jni::wire_access::kotlin_array_of_descriptor(other) {
                    return Some(format!("{n}(0)"));
                }
                "null"
            }
        }
        .to_string(),
    )
}

#[derive(Clone, Debug)]
pub(crate) struct FlatInputError {
    pub root: TypeKey,
    pub path: String,
    pub reason: String,
}

impl FlatInputError {
    pub fn message(&self) -> String {
        format!(
            "data-class input `{}` cannot be flattened at `{}`: {} — fixed-layout data classes must flatten completely; declare `data_class!({}).jobject_input()` to opt this type into an explicit JObject boundary",
            self.root, self.path, self.reason, self.root
        )
    }
}

fn flat_error(root: &TypeKey, path: &str, reason: impl Into<String>) -> FlatInputError {
    FlatInputError {
        root: root.clone(),
        path: path.to_string(),
        reason: reason.into(),
    }
}

pub(crate) fn wire_kotlin_type(entry: &prebindgen_registry::ConverterImpl<KotlinMeta>) -> String {
    if let Some(p) = JniPrim::from_wire(&entry.destination) {
        return p.kotlin_type().to_string();
    }
    if let syn::Type::Path(tp) = &entry.destination {
        if let Some(last) = tp.path.segments.last() {
            return match last.ident.to_string().as_str() {
                "JString" => "String".to_string(),
                "JByteArray" => "ByteArray".to_string(),
                _ => entry
                    .metadata
                    .kotlin_name
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Any".to_string()),
            };
        }
    }
    if matches!(entry.destination, syn::Type::Ptr(_)) {
        "Long".to_string()
    } else {
        entry
            .metadata
            .kotlin_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Any".to_string())
    }
}

pub(crate) fn build_flat_input_plan(
    ext: &Declarations,
    registry: &impl Conversions,
    param_name: &syn::Ident,
    arg: &TypeRef,
) -> Result<Option<FlatInputPlan>, FlatInputError> {
    // 1. Resolve the struct target through `&` and `Option<…>` — off the model,
    //    keeping each layer's reading so the rebuild can restore its spelling.
    let Some((target, inner)) = rebuildable_target(arg) else {
        return Ok(None);
    };
    // `impl Into<S>` is NOT peeled here, and cannot be: the model refuses
    // `impl Trait` that is not the callback form (`DisallowedImplTrait`), so a
    // parameter spelled that way never becomes a reading and never reaches this
    // function.
    // The name off the classification, not off the last path segment: `Box<S>`
    // IS `S` here, and taking the spelling apart would answer about the wrapper.
    let flat::TypeKind::Named { id, .. } = inner.unwrapped().kind() else {
        return Ok(None);
    };
    let Some(name) = id.ident() else {
        return Ok(None);
    };
    // The ELEMENT, not the item it was parsed from: its fields already carry
    // readings, which is the whole of #289.
    let Some(st) = registry.flat().struct_type(&name) else {
        return Ok(None);
    };
    // The DECLARATION is keyed by the type, not by the spelling: a
    // `Box<Payload>` parameter is a `Payload` to Kotlin and must find
    // `Payload`'s data-class declaration.
    let key = inner.stripped_key();
    let Some(cfg) = ext.types.get(&key) else {
        return Ok(None);
    };
    if cfg.special_decl() || cfg.name_spec.is_none() || cfg.jobject_input {
        return Ok(None);
    }
    // Identity / pass-through guard: the resolved param must decode to the
    // struct itself, not an opaque handle / value projection (`projection`
    // present) and not a multi-source / non-identity `impl Into<S>` (which
    // surfaces as `"Any"` Dispatch or a foreign source type). The resolved
    // param's Kotlin type (compared by short name, since metadata carries the
    // FQN) must equal the struct's data-class name.
    let Some(entry) = ext.in_frag(arg) else {
        return Ok(None);
    };
    if entry.metadata.projection.is_some() {
        return Ok(None);
    }
    let dc_short = cfg
        .name_spec
        .as_ref()
        .map(|s| ext.fqn_of(s))
        .map(|fqn| fqn.rsplit('.').next().unwrap_or(&fqn).to_string())
        .unwrap_or_else(|| name.to_string());
    let entry_short = entry
        .metadata
        .kotlin_name
        .as_ref()
        .and_then(|t| t.simple_name());
    if entry_short != Some(dc_short.as_str()) {
        return Ok(None);
    }
    drop(entry);

    // 2. The parameters this crossing occupies, as the recipe states them. What
    //    used to be walked and named here is composed once per crossing, so
    //    the only thing left for the site to say is which parameter each wire
    //    hangs off.
    let Some(wires) = ext.wires_of(arg) else {
        return Ok(None);
    };
    let leaves: Vec<FlatLeaf> = wires.iter().map(|w| FlatLeaf::of(param_name, w)).collect();
    let chain = ext
        .composed_chain(arg, prebindgen_registry::recipe::Direction::Construct)
        .filter(|chain| chain.layout.leaf_count() == leaves.len())
        .ok_or_else(|| {
            flat_error(
                &key,
                &param_name.to_string(),
                "the registry-composed layout arity does not match the declared JNI wires",
            )
        })?;
    chain.activate();
    let contains_composed_child = match &chain.layout {
        crate::jni::compile::JLayout::Product(parts) => {
            parts.iter().any(crate::jni::compile::JLayout::is_composed)
        }
        _ => true,
    };
    Ok(Some(FlatInputPlan {
        leaves,
        chain,
        struct_module: struct_module_path(ext, registry, &st.name),
        struct_ident: st.name.clone(),
        contains_composed_child,
        target,
    }))
}

/// Render native reconstruction for a [`FlatInputPlan`] through its registry
/// chain. Building the Product/Optional/Choice tree is entirely a registry
/// responsibility; this site only supplies the JNI wire leaves.
/// Failures route through `signal_error` and return the function `on_err`
/// sentinel. Returns the prelude and call argument (`arg` or `&arg`).
pub(crate) fn render_flat_input_decode(
    plan: &FlatInputPlan,
    arg_ident: &syn::Ident,
    on_err: &TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, TokenStream) {
    let leaves: Vec<syn::Ident> = plan
        .leaves
        .iter()
        .map(|leaf| leaf.native_ident.clone())
        .collect();
    let intermediate = plan.chain.layout.expression(&leaves);
    let converter = emit.operation_ident("jni", &plan.chain.operation);
    let binding = plan.target.mutable_binding().then(|| quote!(mut));
    let prelude = quote! {
        let #binding #arg_ident = match #converter(&mut env, #intermediate) {
            ::core::result::Result::Ok(__value) => __value,
            ::core::result::Result::Err(__error) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__error.to_string(),
                );
                return #on_err;
            }
        };
    };
    (prelude, plan.target.call_arg(arg_ident))
}

// ──────────────────────────────────────────────────────────────────────
// Slice / Vec input → Rust-side Vec handle (built by pushing leaves)
// ──────────────────────────────────────────────────────────────────────
