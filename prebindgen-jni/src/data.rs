//! Kotlin record declarations and their occurrence conversions.
use super::*;
use std::rc::Rc;

/// A Kotlin data class whose fields are converted across the JNI boundary.
///
/// Supports owned public named-field records containing scalars, `String`,
/// `Vec<T>` collections (`Vec<u8>` is a byte array), other declared data classes,
/// declared opaque handles, and optional fields. Records containing handles,
/// including through collections, support outputs; inputs require an explicit
/// field or constructor expansion describing their ownership.
/// Output ownership is declared with `expand_return!`: `field_self_ref()` borrows
/// and `field_self_into()` moves, recursively. `project_self(Projection)` allows
/// field overrides. `field_self()` and unexpanded outputs follow the source:
/// owned records move fields, while borrowed records observe them. Copying opaque
/// handle fields requires an explicit borrowed projection and Rust `Clone`.
/// Moving Rust fields requires that the record itself does not implement `Drop`.
/// Callers close each returned handle explicitly; data-class `copy` shares those
/// handle objects and does not duplicate native ownership. References, empty
/// records, and recursive records are rejected.
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
pub struct DataClassDecl {
    /// Source record type, resolved against the selected flat model.
    pub ty: syn::Type,
    /// Literal Kotlin short name, or the source type's last path segment.
    pub name: Option<String>,
}
impl DataClassDecl {
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
/// A class declaration placed in a [`PackageDecl`].
#[derive(Clone)]
pub enum ClassDecl {
    /// An owning native handle.
    Ptr(PtrClassDecl),
    /// A record mapped to a Kotlin data class.
    Data(DataClassDecl),
    /// A fieldless enum mapped to a Kotlin enum class.
    Enum(EnumClassDecl),
    /// A retained callback interface and its signature defaults.
    Callback(prebindgen_declarations::CallbackDecl),
}
impl From<PtrClassDecl> for ClassDecl {
    fn from(value: PtrClassDecl) -> Self {
        Self::Ptr(value)
    }
}
impl From<DataClassDecl> for ClassDecl {
    fn from(value: DataClassDecl) -> Self {
        Self::Data(value)
    }
}
impl From<EnumClassDecl> for ClassDecl {
    fn from(value: EnumClassDecl) -> Self {
        Self::Enum(value)
    }
}
impl From<prebindgen_declarations::CallbackDecl> for ClassDecl {
    fn from(value: prebindgen_declarations::CallbackDecl) -> Self {
        Self::Callback(value)
    }
}
impl ClassDecl {
    pub(crate) fn ty(&self) -> &syn::Type {
        match self {
            Self::Ptr(c) => &c.ty,
            Self::Data(c) => &c.ty,
            Self::Enum(c) => &c.ty,
            Self::Callback(c) => &c.ty,
        }
    }
    pub(crate) fn name(&self) -> &Option<String> {
        match self {
            Self::Ptr(c) => &c.name,
            Self::Data(c) => &c.name,
            Self::Enum(c) => &c.name,
            Self::Callback(c) => &c.foreign,
        }
    }
}
#[macro_export]
/// Declare a Kotlin data class from Rust type syntax.
///
/// Returns [`DataClassDecl`]. The source type need not be in scope. Invalid type
/// syntax panics; unsupported record fields are diagnosed during build.
macro_rules! data_class {($($ty:tt)+)=>{$crate::DataClassDecl::new($crate::__type(stringify!($($ty)+)))};}

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
/// Kotlin record information attached to a parent node in the wire tree.
#[derive(Clone, Debug)]
pub struct DataGroup {
    pub(crate) root: String,
    pub(crate) class_name: String,
    pub(crate) condition: String,
}

/// Complete class and enum declarations already attached to their source types.
pub(crate) fn prepare(flat: &mut Bindings) -> Result<()> {
    fn element(
        flat: &mut Bindings,
        ty: &syn::Type,
        stack: &mut Vec<String>,
        resolved: &mut BTreeSet<String>,
    ) -> Result<(String, bool)> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            if prebindgen_declarations::option_inner(inner).is_some() {
                return Err("nested Option data fields require an explicit expansion".into());
            }
            let (ty, handles) = element(flat, inner, stack, resolved)?;
            return Ok((format!("{ty}?"), handles));
        }
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
            let handles = if class.data {
                resolve(flat, &type_key(ty), stack, resolved)?
            } else {
                !class.enumeration
            };
            return Ok((class.fqn(), handles));
        }
        if let Some(inner) = prebindgen_flat::vec_inner(ty) {
            let (ty, handles) = element(flat, inner, stack, resolved)?;
            return Ok((format!("kotlin.collections.List<{ty}>"), handles));
        }
        Err(format!(
            "unsupported data class collection element `{}`",
            type_key(ty)
        ))
    }
    fn resolve(
        flat: &mut Bindings,
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
                .has_handles);
        }
        let record = flat
            .records
            .get(key)
            .ok_or_else(|| format!("data class `{key}` requires a named-field record"))?;
        let source_fields = record.fields.clone();
        if record.fields.is_empty() {
            return Err(format!("data class `{key}` must have at least one field"));
        }
        stack.push(key.into());
        let mut fields = vec![];
        let mut getters = BTreeSet::from(["getClass".to_string()]);
        let mut has_handles = false;
        for f in &source_fields {
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
            let optional = prebindgen_declarations::option_inner(&f.ty);
            let ty = optional.unwrap_or(&f.ty);
            if optional
                .and_then(prebindgen_declarations::option_inner)
                .is_some()
            {
                return Err("nested Option data fields require an explicit expansion".into());
            }
            if matches!(ty, syn::Type::Reference(_)) {
                return Err(format!(
                    "data class `{key}` field `{name}` cannot be a reference"
                ));
            }
            let field_key = type_key(ty);
            let mut bytes = false;
            let mut floating = false;
            let mut slots = 1;
            let mut kotlin_type = if let Some(value) = Value::of(ty) {
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
            } else if let Some(child) = flat
                .type_info(ty)
                .ok()
                .and_then(|t| t.metadata.class.clone())
            {
                if child.data {
                    has_handles |= resolve(flat, &field_key, stack, resolved)?;
                } else if !child.enumeration {
                    has_handles = true;
                }
                child.fqn()
            } else if let Some(inner) = prebindgen_flat::vec_inner(ty) {
                let (ty, handles) = element(flat, inner, stack, resolved)
                    .map_err(|error| format!("data class `{key}` field `{name}`: {error}"))?;
                has_handles |= handles;
                format!("kotlin.collections.List<{ty}>")
            } else {
                return Err(format!("data class `{key}` field `{name}` has unsupported type `{field_key}`; expected a scalar, String, Vec<T>, declared data class, or declared handle"));
            };
            if optional.is_some() {
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
        if slots > 255 {
            return Err(format!("data class `{key}` copy method needs {slots} JVM parameter slots; the limit is 255"));
        }
        stack.pop();
        let record = flat.records.get_mut(key).unwrap();
        for (field, metadata) in record.fields.iter_mut().zip(fields) {
            field.metadata = metadata;
        }
        let ty = syn::parse_str::<syn::Type>(key).map_err(|e| e.to_string())?;
        let class = flat.type_metadata_mut(&ty)?.class.as_mut().unwrap();
        Rc::make_mut(class).has_handles = has_handles;
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
        if class.data {
            resolve(flat, &key, &mut Vec::new(), &mut resolved)?;
        }
        if class.enumeration && flat.type_info(&ty)?.metadata.enumeration.is_none() {
            let enumeration = Rc::new(JniEnum::from_class(flat, &class, &key)?);
            flat.guards.extend(enumeration.mapping.guards());
            let info = flat.type_metadata_mut(&ty)?;
            info.enumeration = Some(enumeration);
            Rc::make_mut(info.class.as_mut().unwrap())
                .variant_names
                .clear();
        }
    }
    Ok(())
}
impl JniCodec<'_> {
    pub(super) fn enum_type<'a>(
        &self,
        flat: &'a Bindings,
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
        flat.type_info(ty).ok()?.metadata.enumeration.as_ref()
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
                .is_some_and(|c| c.data)
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
            .filter(|c| c.data))
    }
}
pub(crate) fn render_record<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
    class: &Class,
    ty: &syn::Type,
) -> Result<String> {
    let record = flat.record(ty)?;
    let fields = record
        .fields
        .iter()
        .map(|f| {
            format!(
                "    public val {}: {}",
                kotlin_identifier(&field_name(f)),
                f.metadata.kotlin_type
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let lifecycle = if class.has_handles {
        "/** Close each handle member explicitly. Copies share the same handle objects. */\n"
    } else {
        ""
    };
    let mut out = format!(
        "package {}\n\n{lifecycle}public data class {}(\n{fields}\n)",
        class.pkg, class.name
    );
    if record.fields.iter().any(|f| f.metadata.bytes) {
        let equal = record
            .fields
            .iter()
            .map(|f| {
                let n = kotlin_identifier(&field_name(f));
                match &f.metadata {
                    _ if f.metadata.bytes => {
                        format!("this.{n}.contentEquals(other.{n})")
                    }
                    _ if f.metadata.floating => {
                        format!("this.{n}.compareTo(other.{n}) == 0")
                    }
                    _ => format!("this.{n} == other.{n}"),
                }
            })
            .collect::<Vec<_>>()
            .join(" &&\n            ");
        let hashes = record
            .fields
            .iter()
            .map(|f| {
                let n = kotlin_identifier(&field_name(f));
                if f.metadata.bytes {
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
        let display = record
            .fields
            .iter()
            .map(|f| {
                let label = field_name(f);
                let n = kotlin_identifier(&label);
                let suffix = if f.metadata.bytes {
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

impl Class {
    pub(super) fn mark(
        &self,
        layout: &mut prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        root: String,
        condition: String,
    ) -> Result<()> {
        group::retire_niches(layout, None)?;
        let mut nodes = std::mem::take(layout);
        let prebindgen_flat::WireNode::Branch {
            meta: JniBranch::Group(metadata),
            ..
        } = &mut nodes
        else {
            return Err("record compiler must produce one field collection".into());
        };
        if !matches!(metadata, JniGroup::Fields(None)) {
            return Err("record annotation requires a field collection".into());
        }
        *metadata = JniGroup::Fields(Some(DataGroup {
            root,
            class_name: self.fqn(),
            condition,
        }));
        *layout = nodes;
        Ok(())
    }
}

type FieldNode<'a> = (
    &'a syn::Ident,
    &'a prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
);
pub(crate) fn fields(nodes: &[kotlin::JniNode]) -> Result<Vec<FieldNode<'_>>> {
    nodes
        .iter()
        .map(|node| match node {
            kotlin::JniNode::Branch {
                meta: JniBranch::Group(JniGroup::Field(name)),
                ..
            } => Ok((name, node)),
            _ => Err("Kotlin record requires named field children".into()),
        })
        .collect()
}
pub(crate) fn input_kotlin(
    nodes: &[kotlin::JniNode],
    d: &DataGroup,
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let inner = nodes
        .iter()
        .flat_map(kotlin::leaf_wires)
        .collect::<Vec<_>>();
    let mut out = KotlinInput::default();
    let mut c = kotlin::input_components(nodes, context)?;
    c.prepare_arguments(nodes.iter().any(|n| kotlin::input_handles(n, false)))?;
    out.parameters.push(format!(
        "{}: {}{}",
        d.root,
        d.class_name,
        if active == "true" { "" } else { "?" }
    ));
    let fields = fields(nodes)?;
    if c.parameters.len() != fields.len() {
        return Err("incomplete data-class input occurrence".into());
    }
    let snapshot_fields = nodes.iter().any(|node| kotlin::input_handles(node, false));
    for (index, (p, (field, _))) in c.parameters.iter().zip(&fields).enumerate() {
        let name = p.split_once(": ").unwrap().0;
        let receiver = if active == "true" {
            d.root.clone()
        } else {
            format!("requireNotNull({})", d.root)
        };
        let expr = format!(
            "{receiver}.{}",
            kotlin_identifier(field.to_string().trim_start_matches("r#"))
        );
        // Retain each field of a handle-bearing record once. Controls and
        // handle values then describe the same input, even for handwritten
        // property getters. Primitive locals stay unboxed in inactive arms.
        let expr = if snapshot_fields {
            let local = format!("__pb_{}_field{index}", d.root);
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
            let expression = if d.condition == "true" {
                expr
            } else {
                format!("if ({}) {expr} else {absent}", d.condition)
            };
            out.setup.push(crate::KotlinBinding {
                name: local.clone(),
                expression,
                absent,
            });
            if d.condition != "true" && primitive.is_none() && !ty.ends_with('?') {
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
        out.arguments.push(if d.condition == "true" {
            a
        } else {
            format!(
                "if ({}) {a} else {}",
                d.condition,
                optional::default_kt(&inner[j].kind, None)
            )
        });
    }
    for (j, a) in c.prepared_arguments.into_iter().enumerate() {
        out.prepared_arguments.push(if d.condition == "true" {
            a
        } else {
            let absent = if matches!(inner[j].kind, JniWire::Handle { .. }) {
                "null".into()
            } else {
                optional::default_kt(&inner[j].kind, None)
            };
            format!("if ({}) {a} else {absent}", d.condition)
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
    nodes: &[kotlin::JniNode],
    d: &DataGroup,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let mut c = kotlin::output_components(nodes, context, owners)?;
    if c.delivered.len() != fields(nodes)?.len() {
        return Err("incomplete data-class output occurrence".into());
    }
    c.delivered = vec![format!("{}({})", d.class_name, c.delivered.join(", "))];
    c.result_types = vec![d.class_name.clone()];
    c.result_names = vec![None];
    Ok(c)
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use prebindgen_flat::Wire;
    use prebindgen_flat::WireNode;

    #[test]
    fn codec_reads_current_enum_metadata_without_a_preparation_cache() {
        let mut flat = Bindings::parse(
            syn::parse_file("pub enum E{A,B}")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let ty = flat.resolve_type(&syn::parse_quote!(E)).unwrap();
        let codec = JniCodec::declared(None);
        let mapping = Rc::new(
            prebindgen_declarations::EnumMapping::new(flat.enums.values().next().unwrap()).unwrap(),
        );
        for name in ["example.First", "example.Updated"] {
            flat.type_metadata_mut(&ty).unwrap().enumeration = Some(Rc::new(JniEnum {
                mapping: (*mapping).clone(),
                class_name: name.into(),
                variant_names: BTreeMap::new(),
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
            let wire = prebindgen_flat::wire_definitions(std::iter::once(&output.outcomes.success))
                .next()
                .unwrap();
            assert!(matches!(&wire.kind,JniWire::Enum{enum_type,..} if enum_type.class_name==name));
        }
    }
    #[test]
    fn resolved_record_projection_survives_preparation_round_trip() {
        let mut flat = Bindings::parse(
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
            data: true,
            enumeration: false,
            has_handles: false,
            gc_managed: false,
            variant_names: vec![],
        }));
        prepare(&mut flat).unwrap();
        let codec = JniCodec::declared(None);
        let defaults =
            prebindgen_declarations::Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
        let request = Output::Sequence {
            delivery: prebindgen_declarations::SequenceDelivery::Callback,
            mode: Some(ProjectionMode::Owned),
            element: Box::new(Output::Projected(Projection::owned())),
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
            matches!(&resolved,Output::Sequence{element,..} if matches!(element.as_ref(),Output::Record {mode:Some(ProjectionMode::Owned),fields} if matches!(fields.as_slice(),[Output::Projected(_)])))
        );
        let round_trip = defaults
            .resolve_output(
                &flat,
                &codec,
                &flat.function("items").unwrap().ret.ty,
                &resolved.clone().into_preparation(),
                &(),
            )
            .unwrap();
        assert_eq!(format!("{resolved:?}"), format!("{round_trip:?}"));
        prebindgen_declarations::bind_output(
            &mut flat,
            &codec,
            "items",
            &round_trip.into_preparation(),
            &(),
        )
        .unwrap();
    }

    fn record(root: &str) -> WireNode<JniWire, crate::JniChain, crate::JniBranch> {
        WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(Some(DataGroup {
                root: root.into(),
                class_name: "example.Empty".into(),
                condition: "true".into(),
            }))),
            children: Vec::new(),
        }
    }
    #[test]
    fn kotlin_record_and_presence_bindings_share_a_namespace() {
        let optional = WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(flag),
                kind: JniWire::selector(),
            }],
            meta: JniChain::Optional(JniOptional::Flag(syn::parse_quote!(flag))),
            child: Box::default(),
        };
        let mut combined = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Values),
            children: vec![optional, record("value")],
        };
        crate::group::validate_layout(&combined).unwrap();
        let WireNode::Branch {
            meta: JniBranch::Group(_),
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
            meta: JniBranch::Group(JniGroup::Values),
            children: vec![record("value"), record("value")],
        };
        assert!(crate::group::validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate Kotlin group binding `value`"));
    }
    #[test]
    fn kotlin_record_emission_reads_names_from_shared_fields() {
        let mut field = prebindgen_flat::WireNode::Leaf(vec![Wire {
            name: syn::parse_quote!(incoming),
            kind: JniWire::scalar(Primitive::I32),
        }]);
        field = prebindgen_flat::WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Field(syn::parse_quote!(r#type))),
            children: vec![std::mem::take(&mut field)],
        };
        field = prebindgen_flat::WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(Some(DataGroup {
                root: "point".into(),
                class_name: "example.Point".into(),
                condition: "true".into(),
            }))),
            children: vec![std::mem::take(&mut field)],
        };
        crate::group::validate_layout(&field).unwrap();
        let classes = BTreeMap::new();
        let context = KotlinContext {
            package: "example",
            classes: &classes,
            result: "result",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "output",
        };
        let input = kotlin::input_kotlin(&field, &context).unwrap();
        assert_eq!(input.parameters, ["point: example.Point"]);
        assert!(input.arguments[0].contains("point.type"));
        let output = kotlin::output_kotlin(&field, &context).unwrap();
        assert_eq!(output.result_types, ["example.Point"]);
        assert!(output.delivered[0].contains("example.Point(result[0]"));
        // An empty record still denotes one Kotlin object with no ABI wires.
        let empty = record("empty");
        let output = kotlin::output_kotlin(&empty, &context).unwrap();
        assert_eq!(output.delivered, ["example.Empty()"]);
    }
}
