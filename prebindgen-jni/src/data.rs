//! Kotlin record declarations and their occurrence conversions.
use super::*;

use prebindgen_flat::{Field, Record, TypeRef};
use std::rc::Rc;

/// A Kotlin data class whose fields are converted across the JNI boundary.
///
/// Supports owned public named-field records containing scalars, `String`,
/// vectors and fixed-size arrays, other declared data classes,
/// declared opaque handles, and optional fields. Records containing handles,
/// including through collections, require explicit input field ownership, such as
/// `expand_param!(Row).variant_record(param_fields!())`. Select named constructor
/// overrides for fields that must be copied rather than consumed.
/// Output ownership is declared with `expand_return!`: `field_self_cloned()`
/// permits owning copies and `field_self_into()` moves, recursively.
/// `project_self(Projection)` allows
/// field overrides. `field_self()` and unexpanded outputs follow the source:
/// owned records move fields, while borrowed records observe them. Copying opaque
/// handle fields requires an explicit borrowed projection and Rust `Clone`.
/// Moving Rust fields requires that the record itself does not implement `Drop`.
/// Callers close each returned handle explicitly; data-class `copy` shares those
/// handle objects and does not duplicate native ownership. References, empty
/// records, and recursive records are rejected. Byte vectors and fixed-size
/// byte arrays use `ByteArray`.
/// Collection properties use standard Kotlin list equality and hash codes;
/// byte arrays directly nested in lists therefore compare by reference.
///
/// ```
/// use prebindgen_jni::{JniGen, data_class, package, fun};
/// let source = syn::parse_file("pub struct Point { pub x: f64, pub y: f64 } pub fn echo(p: Point) -> Point { p }")?;
/// let generated = JniGen::builder()
///     .items(source.items.into_iter().map(|item| (item, Default::default())))
///     .set_package_prefix("example")
///     .package(package!().class(data_class!(Point)).fun(fun!(echo)))
///     .build()?;
/// assert!(generated.kotlin["example/Point.kt"].contains("data class Point"));
/// assert!(generated.kotlin["example/Functions.kt"].contains(": example.Point"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct DeclJniDataClass {
    /// Source record type, resolved against the selected flat model.
    pub ty: syn::Type,
    /// Literal Kotlin short name, or the source type's last path segment.
    pub name: Option<String>,
}
impl DeclJniDataClass {
    /// Declare a data class for the source record `ty`.
    pub fn new(ty: syn::Type) -> Self {
        Self { ty, name: None }
    }
    /// Set a literal Kotlin short class name, validated during build.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
/// A class declaration placed in a [`DeclJniPackage`].
#[derive(Clone)]
pub enum DeclJniClass {
    /// An owning native handle.
    Ptr(DeclJniPtrClass),
    /// A record mapped to a Kotlin data class.
    Data(DeclJniDataClass),
    /// A fieldless enum mapped to a Kotlin enum class.
    Enum(DeclJniEnumClass),
    /// A source payload enum represented as a sealed interface.
    Sealed(DeclJniSealedClass),
    /// A retained callback interface and its signature defaults.
    Callback(crate::DeclJniCallback),
}
impl From<DeclJniPtrClass> for DeclJniClass {
    fn from(value: DeclJniPtrClass) -> Self {
        Self::Ptr(value)
    }
}
impl From<DeclJniDataClass> for DeclJniClass {
    fn from(value: DeclJniDataClass) -> Self {
        Self::Data(value)
    }
}
impl From<DeclJniSealedClass> for DeclJniClass {
    fn from(value: DeclJniSealedClass) -> Self {
        Self::Sealed(value)
    }
}
impl From<DeclJniEnumClass> for DeclJniClass {
    fn from(value: DeclJniEnumClass) -> Self {
        Self::Enum(value)
    }
}
impl From<crate::DeclJniCallback> for DeclJniClass {
    fn from(value: crate::DeclJniCallback) -> Self {
        Self::Callback(value)
    }
}
impl DeclJniClass {
    pub(crate) fn ty(&self) -> &syn::Type {
        match self {
            Self::Ptr(c) => &c.ty,
            Self::Data(c) => &c.ty,
            Self::Enum(c) => &c.ty,
            Self::Sealed(c) => &c.ty,
            Self::Callback(c) => &c.ty,
        }
    }
    pub(crate) fn name(&self) -> &Option<String> {
        match self {
            Self::Ptr(c) => &c.name,
            Self::Data(c) => &c.name,
            Self::Enum(c) => &c.name,
            Self::Sealed(c) => &c.name,
            Self::Callback(c) => &c.foreign,
        }
    }
}
#[macro_export]
/// Declare a Kotlin data class from Rust type syntax.
///
/// Returns [`DeclJniDataClass`]. The source type need not be in scope. Invalid type
/// syntax panics; unsupported record fields are diagnosed during build.
macro_rules! data_class {($($ty:tt)+)=>{$crate::DeclJniDataClass::new($crate::__type(stringify!($($ty)+)))};}

/// Prepared Kotlin property metadata stored on its source field.
#[derive(Clone, Debug, Default)]
pub struct JniField {
    /// Destination property type, including nullability.
    pub kotlin_type: String,
    /// Whether Kotlin content-based byte-array operators are required.
    pub bytes: bool,
    /// Whether Kotlin floating-point comparison semantics are required.
    pub floating: bool,
    pub(crate) slots: usize,
}
// Resolve the Rust type describing one public value, independently of boundary
// expansions. Function slots still retain the actual domain type and conversion.
enum PropertyType {
    Value(Box<syn::Type>),
    Optional(Box<PropertyType>),
    List(Box<PropertyType>),
}

fn value_type(
    flat: &JniBindings,
    ty: &syn::Type,
    input: bool,
    stack: &mut Vec<String>,
    converted: bool,
) -> Result<(PropertyType, bool)> {
    let key = type_key(ty);
    let conversion = flat
        .type_info(ty)
        .ok()
        .and_then(|i| {
            if input {
                i.metadata.conversion.input.as_ref()
            } else {
                i.metadata.conversion.output.as_ref()
            }
        })
        .or_else(|| {
            if input {
                None
            } else {
                match ty {
                    syn::Type::Reference(r) => flat
                        .type_info(&r.elem)
                        .ok()
                        .and_then(|i| i.metadata.conversion.output.as_ref()),
                    _ => None,
                }
            }
        });
    if let Some(c) = conversion {
        if stack.contains(&key) {
            return Err(format!(
                "recursive canonical property representation `{key}`"
            ));
        }
        stack.push(key);
        let result = value_type(flat, &c.representation, input, stack, true);
        stack.pop();
        return result.map(|(ty, _)| (ty, true));
    }
    if let Some(inner) = prebindgen_flat::option_inner(ty) {
        let (inner, changed) = value_type(flat, inner, input, stack, converted)?;
        return Ok((PropertyType::Optional(Box::new(inner)), changed));
    }
    if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty)
        .filter(|(_, view)| view.is_owned() && Value::of(ty) != Some(Value::Bytes))
    {
        let (inner, changed) = value_type(flat, inner, input, stack, converted)?;
        return Ok((PropertyType::List(Box::new(inner)), changed));
    }
    if let syn::Type::Reference(r) = ty {
        if !converted {
            return Ok((PropertyType::Value(Box::new(ty.clone())), false));
        }
        match Value::of(ty) {
            Some(Value::Text) => {
                return Ok((
                    PropertyType::Value(Box::new(syn::parse_quote!(String))),
                    false,
                ))
            }
            Some(Value::Bytes) => {
                return Ok((
                    PropertyType::Value(Box::new(syn::parse_quote!(Vec<u8>))),
                    false,
                ))
            }
            _ => return value_type(flat, &r.elem, input, stack, converted),
        }
    }
    Ok((PropertyType::Value(Box::new(ty.clone())), false))
}

/// Resolve public properties, ownership and enum mappings on their source owners.
/// Run before resolving calls, which use these semantic representation facts.
pub(crate) fn resolve_declarations(flat: &mut JniBindings) -> Result<()> {
    fn element(
        flat: &mut JniBindings,
        ty: &PropertyType,
        stack: &mut Vec<String>,
        resolved: &mut BTreeSet<String>,
    ) -> Result<(String, bool)> {
        let ty = match ty {
            PropertyType::Optional(inner)
                if matches!(inner.as_ref(), PropertyType::Optional(_)) =>
            {
                return Err("nested Option data fields require an explicit expansion".into());
            }
            PropertyType::Optional(inner) => {
                let (ty, handles) = element(flat, inner, stack, resolved)?;
                return Ok((format!("{ty}?"), handles));
            }
            PropertyType::List(inner) => {
                let (ty, handles) = element(flat, inner, stack, resolved)?;
                return Ok((format!("kotlin.collections.List<{ty}>"), handles));
            }
            PropertyType::Value(ty) => ty.as_ref(),
        };
        if matches!(ty, syn::Type::Reference(_)) {
            return Err("data class collection elements cannot be references".into());
        }
        if let Some(value) = Value::of(ty) {
            return Ok((
                match value {
                    Value::Scalar(Primitive::I128 | Primitive::U128) => {
                        "java.math.BigInteger".into()
                    }
                    Value::Scalar(p) => format!("kotlin.{}", scalar::kotlin(p)),
                    Value::Text => "kotlin.String".into(),
                    Value::Bytes => "kotlin.ByteArray".into(),
                },
                false,
            ));
        }
        if let Some(class) = flat
            .type_info(ty)
            .ok()
            .and_then(|t| t.metadata.class.clone())
        {
            let handles = if matches!(
                class.representation,
                ClassRepresentation::ConfiguredData
                    | ClassRepresentation::Data { .. }
                    | ClassRepresentation::ConfiguredSealed { .. }
                    | ClassRepresentation::Sealed { .. }
            ) {
                resolve(flat, &type_key(ty), stack, resolved)?
            } else {
                !matches!(
                    class.representation,
                    ClassRepresentation::ConfiguredEnum { .. } | ClassRepresentation::Enum(_)
                )
            };
            return Ok((class.fqn(), handles));
        }
        Err(format!(
            "unsupported data class collection element `{}`",
            type_key(ty)
        ))
    }
    fn resolve(
        flat: &mut JniBindings,
        key: &str,
        stack: &mut Vec<String>,
        resolved: &mut BTreeSet<String>,
    ) -> Result<bool> {
        if stack.iter().any(|k| k == key) {
            return Err(format!("recursive data class `{key}`"));
        }
        if resolved.contains(key) {
            let ty = syn::parse_str::<syn::Type>(key).map_err(|e| e.to_string())?;
            return Ok(flat
                .type_info(&ty)?
                .metadata
                .class
                .as_ref()
                .unwrap()
                .has_handles());
        }
        let class = flat
            .type_info(&syn::parse_str::<syn::Type>(key).map_err(|e| e.to_string())?)?
            .metadata
            .class
            .as_ref()
            .unwrap();
        let sealed = matches!(
            class.representation,
            ClassRepresentation::ConfiguredSealed { .. } | ClassRepresentation::Sealed { .. }
        );
        let mut starts = BTreeSet::new();
        let source_fields = if sealed {
            let source = flat
                .payload_enum_by_key(key)
                .ok_or_else(|| format!("sealed class `{key}` requires a payload enum"))?;
            let mut fields = Vec::new();
            for arm in &source.variants {
                starts.insert(fields.len());
                for f in &arm.fields {
                    fields.push(prebindgen_flat::Field {
                        name: format_ident!("{}", f.property()),
                        ty: f.ty.clone(),
                        metadata: f.metadata.clone(),
                    });
                }
            }
            fields
        } else {
            let record = flat
                .record_by_key(key)
                .ok_or_else(|| format!("data class `{key}` requires a named-field record"))?;
            if record.fields.is_empty() {
                return Err(format!("data class `{key}` must have at least one field"));
            }
            record.fields.clone()
        };
        stack.push(key.into());
        let mut fields = vec![];
        let mut getters = BTreeSet::from(["getClass".to_string()]);
        let mut has_handles = false;
        for (index, f) in source_fields.iter().enumerate() {
            if starts.contains(&index) {
                getters = BTreeSet::from(["getClass".to_string()]);
            }
            let source_name = f.name.to_string();
            let name = source_name.trim_start_matches("r#");
            validate_name(name).map_err(|e| format!("data class `{key}` field `{name}`: {e}"))?;
            let getter = if name.starts_with("is")
                && name.len() > 2
                && !name.as_bytes()[2].is_ascii_lowercase()
            {
                name.to_string()
            } else {
                format!("get{}{}", name[..1].to_ascii_uppercase(), &name[1..])
            };
            if !getters.insert(getter.clone()) {
                return Err(format!(
                    "data class `{key}` field `{name}` has a conflicting JVM getter `{getter}"
                ));
            }
            let (input_type, input_changed) =
                value_type(flat, &f.ty, true, &mut Vec::new(), false)?;
            let (output_type, output_changed) =
                value_type(flat, &f.ty, false, &mut Vec::new(), false)?;
            if input_changed && output_changed {
                let input = element(flat, &input_type, stack, resolved)?;
                let output = element(flat, &output_type, stack, resolved)?;
                if input.0 != output.0 {
                    return Err(format!("data class `{key}` field `{name}` has incompatible canonical input/output representations: {} versus {}",input.0,output.0));
                }
            }
            let property_type = if !input_changed && output_changed {
                &output_type
            } else {
                &input_type
            };
            let (ty, optional) = match property_type {
                PropertyType::Optional(inner) => (inner.as_ref(), true),
                _ => (property_type, false),
            };
            let mut bytes = false;
            let mut floating = false;
            let mut slots = 1;
            let mut kotlin_type = match ty {
                PropertyType::Optional(_) => {
                    return Err("nested Option data fields require an explicit expansion".into());
                }
                PropertyType::List(_) => {
                    let (ty, handles) = element(flat, ty, stack, resolved)
                        .map_err(|error| format!("data class `{key}` field `{name}`: {error}"))?;
                    has_handles |= handles;
                    ty
                }
                PropertyType::Value(ty) => {
                    if matches!(ty.as_ref(), syn::Type::Reference(_)) {
                        return Err(format!(
                            "data class `{key}` field `{name}` cannot be a reference"
                        ));
                    }
                    let field_key = type_key(ty);
                    if field_key == "()" {
                        "kotlin.Unit".into()
                    } else if let Some(value) = Value::of(ty) {
                        match value {
                            Value::Scalar(p @ (Primitive::I128 | Primitive::U128)) => {
                                slots = JniWire::scalar(p).parameter_slots();
                                "java.math.BigInteger".into()
                            }
                            Value::Scalar(p) => {
                                slots = JniWire::scalar(p).parameter_slots();
                                floating = matches!(p, Primitive::F32 | Primitive::F64);
                                format!("kotlin.{}", scalar::kotlin(p))
                            }
                            Value::Text => "kotlin.String".into(),
                            Value::Bytes => {
                                bytes = true;
                                "kotlin.ByteArray".into()
                            }
                        }
                    } else if let Some(class) = flat
                        .type_info(ty)
                        .ok()
                        .and_then(|t| t.metadata.class.clone())
                    {
                        if matches!(
                            class.representation,
                            ClassRepresentation::ConfiguredData
                                | ClassRepresentation::Data { .. }
                                | ClassRepresentation::ConfiguredSealed { .. }
                                | ClassRepresentation::Sealed { .. }
                        ) {
                            has_handles |= resolve(flat, &field_key, stack, resolved)?;
                        } else if !matches!(
                            class.representation,
                            ClassRepresentation::ConfiguredEnum { .. }
                                | ClassRepresentation::Enum(_)
                        ) {
                            has_handles = true;
                        }
                        class.fqn()
                    } else {
                        return Err(format!("data class `{key}` field `{name}` has unsupported type `{field_key}`; expected a scalar, String, owned collection, declared data class, or declared handle"));
                    }
                }
            };
            if optional {
                kotlin_type.push('?');
                slots = 1;
                floating = false;
            }
            fields.push(JniField {
                kotlin_type,
                bytes,
                floating,
                slots,
            });
        }
        let slots = fields.iter().map(|f| f.slots).sum::<usize>() + fields.len().div_ceil(32) + 2;
        if !sealed && slots > 255 {
            return Err(format!("data class `{key}` copy method needs {slots} JVM parameter slots; the limit is 255"));
        }
        stack.pop();
        if sealed {
            let mut start = 0;
            for arm in &flat.payload_enum_by_key(key).unwrap().variants {
                let count = arm.fields.len();
                let slots = fields[start..start + count]
                    .iter()
                    .map(|f| f.slots)
                    .sum::<usize>()
                    + count.div_ceil(32)
                    + 2;
                if slots > 255 {
                    return Err(format!("sealed class `{key}` variant `{}` copy method needs {slots} JVM parameter slots; the limit is 255",arm.name));
                }
                start += count;
            }
            let mut metadata = fields.into_iter();
            for arm in &mut flat.payload_enum_by_key_mut(key).unwrap().variants {
                for field in &mut arm.fields {
                    field.metadata = metadata.next().unwrap();
                }
            }
        } else {
            for (field, metadata) in flat
                .record_by_key_mut(key)
                .unwrap()
                .fields
                .iter_mut()
                .zip(fields)
            {
                field.metadata = metadata;
            }
        }
        let ty = syn::parse_str::<syn::Type>(key).map_err(|e| e.to_string())?;
        let class = flat.type_metadata_mut(&ty)?.class.as_mut().unwrap();
        let representation = &mut Rc::make_mut(class).representation;
        *representation = match representation {
            ClassRepresentation::ConfiguredData | ClassRepresentation::Data { .. } => {
                ClassRepresentation::Data { has_handles }
            }
            ClassRepresentation::ConfiguredSealed { variant_names }
            | ClassRepresentation::Sealed { variant_names, .. } => ClassRepresentation::Sealed {
                variant_names: variant_names.clone(),
                has_handles,
            },
            _ => return Err("record analysis requires a data or sealed representation".into()),
        };
        resolved.insert(key.into());
        Ok(has_handles)
    }
    let mut resolved = BTreeSet::new();
    let mut types = flat
        .types
        .values()
        .filter(|t| t.metadata.class.is_some())
        .map(|t| type_key(&t.ty))
        .collect::<Vec<_>>();
    types.sort();
    for key in types {
        let ty = syn::parse_str::<syn::Type>(&key).map_err(|e| e.to_string())?;
        let class = flat
            .type_info(&ty)?
            .metadata
            .class
            .as_ref()
            .unwrap()
            .clone();
        if matches!(
            class.representation,
            ClassRepresentation::ConfiguredData
                | ClassRepresentation::Data { .. }
                | ClassRepresentation::ConfiguredSealed { .. }
                | ClassRepresentation::Sealed { .. }
        ) {
            resolve(flat, &key, &mut Vec::new(), &mut resolved)?;
        }
        if matches!(
            class.representation,
            ClassRepresentation::ConfiguredEnum { .. }
        ) {
            let enumeration = Rc::new(JniEnum::from_class(flat, &class, &key)?);
            flat.guards.extend(enumeration.mapping.guards());
            let info = flat.type_metadata_mut(&ty)?;
            Rc::make_mut(info.class.as_mut().unwrap()).representation =
                ClassRepresentation::Enum(enumeration);
        }
    }
    Ok(())
}

/// Class-only models have no function binding whose readiness could guard them.
pub(crate) fn validate_resolved<I, O>(flat: &Flat<JniSchema<I, O>>) -> Result<()> {
    for source in flat.types.values() {
        if let Some(class) = &source.metadata.class {
            if matches!(
                class.representation,
                ClassRepresentation::ConfiguredData
                    | ClassRepresentation::ConfiguredEnum { .. }
                    | ClassRepresentation::ConfiguredSealed { .. }
            ) {
                return Err(format!(
                    "JNI class `{}` for `{}` is unresolved; call resolve before compilation or generation",
                    class.fqn(),
                    type_key(&source.ty)
                ));
            }
        }
    }
    Ok(())
}
impl JniCodec {
    pub(super) fn enum_type<'a>(
        &self,
        flat: &'a JniBindings,
        ty: &syn::Type,
    ) -> Option<&'a Rc<JniEnum>> {
        if !self.declared {
            return None;
        }
        let ty = if let syn::Type::Reference(r) = ty {
            &r.elem
        } else {
            ty
        };
        flat.type_info(ty).ok()?.metadata.enumeration()
    }
    pub(super) fn source_record<'a>(
        flat: &'a Flat<JniSchema>,
        ty: &syn::Type,
    ) -> Result<Option<&'a Rc<Class>>> {
        if let syn::Type::Reference(r) = ty {
            if flat
                .type_info(&r.elem)
                .ok()
                .and_then(|t| t.metadata.class.as_ref())
                .is_some_and(|c| matches!(c.representation, ClassRepresentation::Data { .. }))
            {
                return Err(format!(
                    "data class `{}` supports owned occurrences only",
                    type_key(&r.elem)
                ));
            }
        }
        Ok(flat
            .type_info(ty)
            .ok()
            .and_then(|t| t.metadata.class.as_ref())
            .filter(|c| matches!(c.representation, ClassRepresentation::Data { .. })))
    }
}
pub(crate) fn render_record<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
    class: &Class,
    ty: &syn::Type,
) -> Result<String> {
    let record = flat.record(ty)?;
    render_record_fields(
        class,
        record.fields.iter().map(|f| (field_name(f), &f.metadata)),
    )
}
pub(crate) fn render_record_fields<'a>(
    class: &Class,
    fields: impl Iterator<Item = (String, &'a JniField)> + Clone,
) -> Result<String> {
    let parameters = fields
        .clone()
        .map(|(name, metadata)| {
            format!(
                "    public val {}: {}",
                kotlin_identifier(&name),
                metadata.kotlin_type
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let lifecycle = if class.has_handles() {
        "/** Close each handle member explicitly. Copies share the same handle objects. */\n"
    } else {
        ""
    };
    let mut out = format!(
        "package {}\n\n{lifecycle}public data class {}(\n{parameters}\n)",
        class.pkg, class.name
    );
    if fields.clone().any(|(_, metadata)| metadata.bytes) {
        let equal = fields
            .clone()
            .map(|(name, metadata)| {
                let n = kotlin_identifier(&name);
                match metadata {
                    _ if metadata.bytes => {
                        format!("this.{n}.contentEquals(other.{n})")
                    }
                    _ if metadata.floating => {
                        format!("this.{n}.compareTo(other.{n}) == 0")
                    }
                    _ => format!("this.{n} == other.{n}"),
                }
            })
            .collect::<Vec<_>>()
            .join(" &&\n            ");
        let hashes = fields
            .clone()
            .map(|(name, metadata)| {
                let n = kotlin_identifier(&name);
                if metadata.bytes {
                    format!("this.{n}.contentHashCode()")
                } else {
                    format!("this.{n}.hashCode()")
                }
            })
            .collect::<Vec<_>>();
        let mut hash = hashes[0].clone();
        for h in &hashes[1..] {
            hash = format!("31 * ({hash}) + {h}");
        }
        let display = fields
            .clone()
            .map(|(name, metadata)| {
                let label = name;
                let n = kotlin_identifier(&label);
                let suffix = if metadata.bytes {
                    ".contentToString()"
                } else {
                    ""
                };
                format!("{label}=${{this.{n}{suffix}}}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(" {{\n    override fun equals(other: kotlin.Any?): kotlin.Boolean =\n        this === other || (other is {} &&\n            {equal})\n    override fun hashCode(): kotlin.Int = {hash}\n    override fun toString(): kotlin.String = \"{}({display})\"\n}}", class.name, class.name));
    }
    out.push('\n');
    Ok(out)
}
fn field_name(field: &prebindgen_flat::Field<JniField>) -> String {
    field.name.to_string().trim_start_matches("r#").into()
}

pub(super) fn mark<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    layout: &mut JniWireNode<Chain, Branch, Annotation, Container>,
    record: Branch,
) -> Result<()> {
    group::retire_niches(layout, None)?;
    let prebindgen_conversions::WireNode::Branch { meta, .. } = layout else {
        return Err("record compiler must produce one field collection".into());
    };
    if !(meta.is_fields()) {
        return Err("record annotation requires a field collection".into());
    }
    *meta = record;
    Ok(())
}

pub(crate) fn field<
    Chain: group::ChainMetadata,
    Branch,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> Result<&syn::Ident> {
    if let prebindgen_conversions::WireNode::Annotation { meta, .. } = node {
        if let Some(name) = meta.field() {
            return Ok(name);
        }
    }
    Err("Kotlin record requires named field children".into())
}
// A grouped Kotlin value has a fixed property contract. An occurrence override
// may change native conversion/ownership but must still accept that property.
fn property_assignable(value: &str, destination: &str) -> bool {
    let value = value.trim();
    let destination = destination.trim();
    if value.ends_with('?') && !destination.ends_with('?') {
        return false;
    }
    let value = value.trim_end_matches('?');
    let destination = destination.trim_end_matches('?');
    // Short built-in names and fully qualified built-ins denote the same type.
    // Preserve namespace segments inside generated class names.
    fn builtin(name: &str) -> &str {
        name.strip_prefix("kotlin.")
            .filter(|name| !name.contains(['.', '<']))
            .unwrap_or(name)
    }
    if builtin(value) == builtin(destination) {
        return true;
    }
    fn list_element(name: &str) -> Option<&str> {
        name.strip_prefix("kotlin.collections.List<")
            .or_else(|| name.strip_prefix("List<"))?
            .strip_suffix('>')
    }
    match (list_element(value), list_element(destination)) {
        (Some(value), Some(destination)) => property_assignable(value, destination),
        _ => false,
    }
}
pub(crate) fn check_property_input(expected: &str, parameters: &[String]) -> Result<()> {
    let actual = match parameters {
        [] if expected == "kotlin.Unit" || expected == "Unit" => return Ok(()),
        [parameter] => {
            parameter
                .split_once(": ")
                .ok_or("invalid Kotlin property parameter")?
                .1
        }
        _ => return Err("a grouped Kotlin property requires exactly one input value".into()),
    };
    if property_assignable(expected, actual) {
        Ok(())
    } else {
        Err(format!(
            "Kotlin property `{expected}` cannot use input conversion accepting `{actual}`"
        ))
    }
}
pub(crate) fn check_property_output(expected: &str, results: &[String]) -> Result<()> {
    let actual = match results {
        [] if expected == "kotlin.Unit" || expected == "Unit" => return Ok(()),
        [actual] => actual,
        _ => return Err("a grouped Kotlin property requires exactly one output value".into()),
    };
    if property_assignable(actual, expected) {
        Ok(())
    } else {
        Err(format!(
            "output conversion returning `{actual}` cannot initialize Kotlin property `{expected}`"
        ))
    }
}

#[allow(clippy::type_complexity)] // Borrow each canonical field with its directional conversion subtree.
pub(crate) fn record_fields<
    'a,
    Chain: group::ChainMetadata,
    Branch,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    nodes: &'a [JniWireNode<Chain, Branch, Annotation, Container>],
    source: &'a Record<JniField>,
) -> Result<
    impl Iterator<
        Item = Result<(
            &'a Field<JniField>,
            &'a JniWireNode<Chain, Branch, Annotation, Container>,
        )>,
    >,
> {
    if nodes.len() != source.fields.len() {
        return Err("record occurrence differs from its source fields".into());
    }
    Ok(source.fields.iter().zip(nodes).map(|(source, node)| {
        if field(node)? != &source.name {
            return Err("record occurrence differs from its source fields".into());
        }
        Ok((source, node))
    }))
}

pub(crate) fn input_kotlin(
    nodes: &[JniInputWireNode],
    r_type: &TypeRef,
    k_name: &str,
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let (record, class) = crate::metadata::source_data_class(context.source_types, r_type)?;
    let class_name = class.fqn();
    let inner = nodes
        .iter()
        .flat_map(kotlin::leaf_wires)
        .collect::<Vec<_>>();
    let mut out = KotlinInput::default();
    let mut c = KotlinInput {
        handle_inputs: Some(Vec::new()),
        ..Default::default()
    };
    for pair in record_fields(nodes, record)? {
        let (field, node) = pair?;
        // Check the property's contract when this record is present. An
        // inactive arm may widen temporary parameters to nullable, but that
        // must not let a nullable property feed a required conversion.
        let child = kotlin::input_kotlin(node, context)?;
        check_property_input(&field.metadata.kotlin_type, &child.parameters)
            .map_err(|error| format!("{class_name}.{}: {error}", field.name))?;
        let child = if active == "true" {
            child
        } else {
            kotlin::input_kotlin_when(node, context, active)?
        };
        kotlin::merge_input(&mut c, child)?;
    }
    c.prepare_arguments(nodes.iter().any(|n| kotlin::has_handles(n, false)))?;
    out.parameters.push(format!(
        "{}: {}{}",
        k_name,
        class_name,
        if active == "true" { "" } else { "?" }
    ));
    if c.parameters.len() != record.fields.len() {
        return Err("incomplete data-class input occurrence".into());
    }
    let snapshot_fields = nodes.iter().any(|node| kotlin::has_handles(node, false));
    for (index, (p, field)) in c.parameters.iter().zip(&record.fields).enumerate() {
        let name = p.split_once(": ").unwrap().0;
        let receiver = if active == "true" {
            k_name.to_owned()
        } else {
            format!("requireNotNull({})", k_name)
        };
        let expr = format!(
            "{receiver}.{}",
            kotlin_identifier(field.name.to_string().trim_start_matches("r#"))
        );
        // Retain each field of a handle-bearing record once. Controls and
        // handle values then describe the same input, even for handwritten
        // property getters. Primitive locals stay unboxed in inactive arms.
        let expr = if snapshot_fields {
            let local = format!("__pb_{}_field{index}", k_name);
            let ty = p.split_once(": ").unwrap().1;
            let primitive = match ty.trim_start_matches("kotlin.") {
                "Boolean" => Some("false"),
                "Byte" => Some("0.toByte()"),
                "Short" => Some("0.toShort()"),
                "Int" => Some("0"),
                "Long" => Some("0L"),
                "Float" => Some("0.0f"),
                "Double" => Some("0.0"),
                "UInt" => Some("0u"),
                "ULong" => Some("0uL"),
                _ => None,
            };
            let absent = primitive.unwrap_or("null").to_owned();
            let expression = if active == "true" {
                expr
            } else {
                format!("if ({active}) {expr} else {absent}")
            };
            out.setup.push(crate::KotlinBinding {
                name: local.clone(),
                expression,
                absent,
            });
            if active != "true" && primitive.is_none() && !ty.ends_with('?') {
                format!("requireNotNull({local})")
            } else {
                local
            }
        } else {
            expr
        };
        for a in c
            .arguments
            .iter_mut()
            .chain(c.prepared_arguments.iter_mut())
            .chain(c.locks.iter_mut())
            .chain(c.consumed.iter_mut())
            .chain(c.lock_groups.iter_mut())
            .chain(
                c.handle_inputs
                    .iter_mut()
                    .flatten()
                    .flat_map(|h| std::iter::once(&mut h.value).chain(h.consumed.iter_mut())),
            )
            .chain(c.setup.iter_mut().map(|b| &mut b.expression))
        {
            *a = optional::replace(a, name, &expr);
        }
    }
    for (j, a) in c.arguments.into_iter().enumerate() {
        out.arguments.push(if active == "true" {
            a
        } else {
            format!(
                "if ({active}) {a} else {}",
                optional::default_kt(&inner[j].kind, None)
            )
        });
    }
    for (j, a) in c.prepared_arguments.into_iter().enumerate() {
        out.prepared_arguments.push(if active == "true" {
            a
        } else {
            let absent = if matches!(inner[j].kind, JniWire::RHandle { .. }) {
                "null".into()
            } else {
                optional::default_kt(&inner[j].kind, None)
            };
            format!("if ({active}) {a} else {absent}")
        });
    }
    out.locks.extend(c.locks);
    out.consumed.extend(c.consumed);
    out.setup.extend(c.setup);
    out.lock_groups.extend(c.lock_groups);
    out.handle_inputs = c.handle_inputs;
    Ok(out)
}
pub(crate) fn output_kotlin(
    nodes: &[JniOutputWireNode],
    r_type: &TypeRef,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let (record, class) = crate::metadata::source_data_class(context.source_types, r_type)?;
    let class_name = class.fqn();
    let mut fields = record_fields(nodes, record)?;
    let mut c = kotlin::output_components_checked(nodes, context, owners, |child| {
        let (field, _) = fields.next().ok_or("missing record field")??;
        check_property_output(&field.metadata.kotlin_type, &child.result_types)
            .map_err(|error| format!("{class_name}.{}: {error}", field.name))
    })?;
    if c.delivered.len() != record.fields.len() {
        return Err("incomplete data-class output occurrence".into());
    }
    c.delivered = vec![format!("{}({})", class_name, c.delivered.join(", "))];
    c.result_types = vec![class_name.clone()];
    c.result_names = vec![None];
    Ok(c)
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use prebindgen_conversions::Wire;
    use prebindgen_conversions::WireNode;

    #[test]
    fn enum_preparation_replaces_its_configuration_and_can_be_repeated() {
        let items = syn::parse_file("pub enum E { A, B }").unwrap().items;
        let mut flat =
            JniBindings::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let ty = flat.resolve_type(&syn::parse_quote!(E)).unwrap();
        flat.type_metadata_mut(&ty).unwrap().class = Some(Rc::new(Class {
            pkg: "example".into(),
            name: "E".into(),
            representation: ClassRepresentation::ConfiguredEnum {
                variant_names: vec![("A".into(), "First".into())],
            },
        }));
        assert!(matches!(
            flat.type_info(&ty)
                .unwrap()
                .metadata
                .class
                .as_ref()
                .unwrap()
                .representation,
            ClassRepresentation::ConfiguredEnum { .. }
        ));
        resolve_declarations(&mut flat).unwrap();
        let mapping = flat
            .type_info(&ty)
            .unwrap()
            .metadata
            .enumeration()
            .unwrap()
            .clone();
        assert_eq!(mapping.variant_name(&syn::parse_quote!(A)), "First");
        assert!(matches!(
            flat.type_info(&ty)
                .unwrap()
                .metadata
                .class
                .as_ref()
                .unwrap()
                .representation,
            ClassRepresentation::Enum(_)
        ));
        let guards = flat.guards.len();
        resolve_declarations(&mut flat).unwrap();
        assert!(Rc::ptr_eq(
            &mapping,
            flat.type_info(&ty).unwrap().metadata.enumeration().unwrap()
        ));
        assert_eq!(flat.guards.len(), guards);
    }

    #[test]
    fn codec_reads_current_enum_metadata_without_a_preparation_cache() {
        let mut flat = JniBindings::parse(
            syn::parse_file("pub enum E{A,B}")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let ty = flat.resolve_type(&syn::parse_quote!(E)).unwrap();
        let codec = JniCodec::declared();
        let mapping = Rc::new(
            prebindgen_declarations::EnumMapping::new(flat.enums().next().unwrap()).unwrap(),
        );
        for name in ["example.First", "example.Updated"] {
            let (pkg, short) = name.rsplit_once('.').unwrap();
            flat.type_metadata_mut(&ty).unwrap().class = Some(Rc::new(Class {
                pkg: pkg.into(),
                name: short.into(),
                representation: ClassRepresentation::Enum(Rc::new(JniEnum {
                    mapping: (*mapping).clone(),
                    class_name: name.into(),
                    variant_names: BTreeMap::new(),
                })),
            }));
            let output = codec
                .output_leaf(prebindgen_declarations::OutputLeaf::new(
                    &flat,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    prebindgen_declarations::ValueAccess::Owned,
                ))
                .unwrap();
            let wire =
                prebindgen_conversions::wire_definitions(std::iter::once(&output.outcomes.success))
                    .next()
                    .unwrap();
            assert!(
                matches!(&wire.kind,JniWire::REnumTag{enum_type,..} if enum_type.class_name==name)
            );
        }
    }
    #[test]
    fn resolved_record_projection_survives_preparation_round_trip() {
        let mut flat = JniBindings::parse(
            syn::parse_file("pub struct Record{pub n:i64}pub fn items()->Vec<Record>{}")
                .unwrap()
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let ty = flat.resolve_type(&syn::parse_quote!(Record)).unwrap();
        flat.type_metadata_mut(&ty).unwrap().class = Some(Rc::new(Class {
            pkg: "example".into(),
            name: "Record".into(),
            representation: ClassRepresentation::Data { has_handles: false },
        }));
        resolve_declarations(&mut flat).unwrap();
        let codec = JniCodec::declared();
        let defaults = crate::JniSourceResolver::new(&mut flat, vec![], vec![], vec![]).unwrap();
        let request = JniOutputConfig::Unary {
            operation: JniOutputUnary::Sequence {
                delivery: prebindgen_declarations::SequenceDelivery::Callback,
                mode: Some(ProjectionMode::Owned),
            },
            child: Box::new(JniOutputConfig::Leaf(JniOutputLeafOp::Projected(
                Projection::owned(),
            ))),
        };
        let resolved = defaults
            .resolve_output(
                &flat,
                &codec,
                &flat.function("items").unwrap().ret.ty,
                &request,
                &(),
            )
            .unwrap();
        assert!(
            matches!(&resolved,crate::operations::JniOutput::Unary { operation: crate::operations::JniOutputUnary::Sequence { .. }, child: element, .. } if matches!(element.as_ref(),crate::operations::JniOutput::Branch { operation: crate::operations::JniOutputBranch::Record { mode: Some(ProjectionMode::Owned) }, children: fields,  } if matches!(fields.as_slice(),[crate::operations::JniOutput::Leaf(crate::operations::JniOutputLeafOp::Projected(_))])))
        );
        let plan = prebindgen_declarations::compile_output(&flat, &codec, "items", &resolved, &())
            .unwrap();
        bindings::set_output(&mut flat, "items", plan).unwrap();
    }

    fn record(root: &str, source: TypeRef) -> JniInputWireNode {
        WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::KDataClass {
                k_name: root.into(),
                r_type: source,
            },
            children: Vec::new(),
        }
    }
    #[test]
    fn kotlin_record_and_presence_bindings_share_a_namespace() {
        let flat: JniBindings = JniBindings::parse([(
            syn::parse_quote!(
                pub struct Empty {}
            ),
            Default::default(),
        )])
        .unwrap();
        let reference = flat.type_ref(&syn::parse_quote!(Empty)).unwrap();
        let record = |root| record(root, reference.clone());

        let optional = WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(flag),
                kind: JniWire::selector(),
            }],
            meta: JniInputWireChain::ROption {
                presence: JniPresence::Flag(syn::parse_quote!(flag)),
            },
            child: Box::default(),
        };
        let mut combined = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::KValues,
            children: vec![optional, record("value")],
        };
        crate::group::validate_layout(&combined).unwrap();
        let WireNode::Branch {
            meta: JniInputWireBranch::KValues,
            children: elements,
            ..
        } = &mut combined
        else {
            panic!()
        };
        elements.push(record("flag"));
        assert!(crate::group::validate_layout(&combined)
            .unwrap_err()
            .contains("duplicate Kotlin group binding `flag`"));
        let duplicates = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::KValues,
            children: vec![record("value"), record("value")],
        };
        assert!(crate::group::validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate Kotlin group binding `value`"));
    }
    #[test]
    fn output_records_do_not_introduce_kotlin_input_bindings() {
        let flat: JniBindings = JniBindings::parse([(
            syn::parse_quote!(
                pub struct Empty {}
            ),
            Default::default(),
        )])
        .unwrap();
        let record = WireNode::Branch {
            controls: vec![],
            meta: JniOutputWireBranch::KDataClass {
                r_type: flat.type_ref(&syn::parse_quote!(Empty)).unwrap(),
            },
            children: vec![],
        };
        let optional = WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(flag),
                kind: JniWire::selector(),
            }],
            meta: JniOutputWireChain::ROption {
                presence: JniPresence::Flag(syn::parse_quote!(flag)),
            },
            child: Box::default(),
        };
        // Repeated source records have no Kotlin input binding to collide with
        // one another or with the output presence binding.
        let output: JniOutputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniOutputWireBranch::KValues,
            children: vec![record.clone(), optional, record],
        };
        crate::group::validate_layout(&output).unwrap();
    }
    #[test]
    fn record_values_preserve_named_field_validation() {
        let flat: JniBindings = JniBindings::parse([(
            syn::parse_quote!(
                pub struct Row {
                    pub value: i64,
                }
            ),
            Default::default(),
        )])
        .unwrap();
        let mut node = record("row", flat.type_ref(&syn::parse_quote!(Row)).unwrap());
        let WireNode::Branch { children, .. } = &mut node else {
            panic!()
        };
        let leaf = WireNode::Leaf(vec![Wire {
            name: syn::parse_quote!(value),
            kind: JniWire::scalar(Primitive::I64),
        }]);
        children.push(leaf.clone());
        assert!(crate::group::validate_layout(&node)
            .unwrap_err()
            .contains("named field children"));
        let field = WireNode::Annotation {
            meta: JniInputWireAnnotation::RField {
                name: syn::parse_quote!(value),
            },
            child: Box::new(leaf),
        };
        let WireNode::Branch { children, .. } = &mut node else {
            panic!()
        };
        *children = vec![field.clone()];
        crate::group::validate_layout(&node).unwrap();
        let WireNode::Branch { children, .. } = &mut node else {
            panic!()
        };
        let mut duplicate = field;
        let WireNode::Annotation { child, .. } = &mut duplicate else {
            panic!()
        };
        let WireNode::Leaf(wires) = child.as_mut() else {
            panic!()
        };
        // Wire bindings are distinct; only the source field name is duplicated.
        wires[0].name = syn::parse_quote!(second_value);
        children.push(duplicate);
        assert!(crate::group::validate_layout(&node)
            .unwrap_err()
            .contains("duplicate field `value`"));
    }
    #[test]
    fn kotlin_record_emission_reads_names_from_shared_fields() {
        let items = syn::parse_file("pub struct Point { pub r#type:i32 }pub struct Empty {}")
            .unwrap()
            .items;
        let flat = JniGen::builder()
            .flat(JniBindings::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap())
            .set_package_prefix("example")
            .package(package!().class(data_class!(Point)))
            .prepare()
            .unwrap();
        fn field<Chain, Branch, Annotation, Container>(
            named: impl FnOnce(syn::Ident) -> Annotation,
            record: Branch,
        ) -> JniWireNode<Chain, Branch, Annotation, Container> {
            let mut field = prebindgen_conversions::WireNode::Leaf(vec![Wire {
                name: syn::parse_quote!(incoming),
                kind: JniWire::scalar(Primitive::I32),
            }]);
            field = prebindgen_conversions::WireNode::Annotation {
                meta: named(syn::parse_quote!(r#type)),
                child: Box::new(std::mem::take(&mut field)),
            };
            field = prebindgen_conversions::WireNode::Branch {
                controls: vec![],
                meta: record,
                children: vec![std::mem::take(&mut field)],
            };
            field
        }
        crate::group::validate_layout(&field::<
            JniInputWireChain,
            JniInputWireBranch,
            JniInputWireAnnotation,
            JniInputWireContainer,
        >(
            |name| JniInputWireAnnotation::RField { name },
            JniInputWireBranch::KDataClass {
                k_name: "point".into(),
                r_type: flat.type_ref(&syn::parse_quote!(Point)).unwrap(),
            },
        ))
        .unwrap();
        let empty: JniOutputWireNode = WireNode::Branch {
            controls: vec![],
            children: vec![],
            meta: JniOutputWireBranch::KDataClass {
                r_type: flat.type_ref(&syn::parse_quote!(Empty)).unwrap(),
            },
        };
        let point = flat.type_ref(&syn::parse_quote!(Point)).unwrap();
        let mut types = flat.types;
        // Zero-field records are only exercised by this direct wire-contract test.
        types.get_mut("Empty").unwrap().metadata.class = Some(Rc::new(Class {
            pkg: "example".into(),
            name: "Empty".into(),
            representation: ClassRepresentation::Data { has_handles: false },
        }));
        let context = KotlinContext {
            package: "example",
            source_types: &types,
            bulk: false,
            result: "result",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "output",
        };
        let input_record: JniInputWireNode = field(
            |name| JniInputWireAnnotation::RField { name },
            JniInputWireBranch::KDataClass {
                k_name: "point".into(),
                r_type: point.clone(),
            },
        );
        let input = kotlin::input_kotlin(&input_record, &context).unwrap();
        assert_eq!(input.parameters, ["point: example.Point"]);
        assert!(input.arguments[0].contains("point.type"));
        // The same prepared record obtains its guard from each enclosing
        // traversal rather than keeping the condition from its compilation.
        for active in ["first == 0", "second == 1"] {
            let input = kotlin::input_kotlin_when(&input_record, &context, active).unwrap();
            assert_eq!(input.parameters, ["point: example.Point?"]);
            assert!(input.arguments[0].starts_with(&format!("if ({active})")));
        }
        let output_record: JniOutputWireNode = field(
            |name| JniOutputWireAnnotation::RField { name },
            JniOutputWireBranch::KDataClass { r_type: point },
        );
        let output = kotlin::output_kotlin(&output_record, &context).unwrap();
        assert_eq!(output.result_types, ["example.Point"]);
        assert_eq!(output.result_names, [None]);
        assert!(output.delivered[0].contains("example.Point(result[0]"));
        let named = WireNode::Annotation {
            meta: JniOutputWireAnnotation::KParamName {
                name: "location".into(),
            },
            child: Box::new(output_record),
        };
        let output = kotlin::output_kotlin(&named, &context).unwrap();
        assert_eq!(output.result_types, ["example.Point"]);
        assert_eq!(output.result_names, [Some("location".into())]);
        // An empty record still denotes one Kotlin object with no ABI wires.
        let output = kotlin::output_kotlin(&empty, &context).unwrap();
        assert_eq!(output.delivered, ["example.Empty()"]);
    }
}
