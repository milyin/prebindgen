//! Kotlin record declarations and their occurrence conversions.
use super::*;
use prebindgen_declarations::Condition;
use std::rc::Rc;

/// A Kotlin data class whose fields are converted across the JNI boundary.
///
/// Supports owned public named-field records containing scalars, `String`,
/// `Vec<u8>`, other declared data classes, declared opaque handles, and optional fields.
/// Records containing handles (including nested ones) support output positions,
/// including returns and callback arguments; they remain unsupported as inputs.
/// Output ownership is declared with `expand_return!`: `field_self_ref()` borrows
/// and `field_self_into()` moves, recursively. `project_self(Projection)` allows
/// field overrides. Legacy `field_self()` and unexpanded outputs keep a fixed
/// borrowed policy, including for handles (which then require `Clone`).
/// Moving Rust fields requires that the record itself does not implement `Drop`.
/// Callers close each returned handle explicitly; data-class `copy` shares those
/// handle objects and does not duplicate native ownership. References, empty
/// records, and recursive records are rejected.
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

/// JNI conversions consult the declarations attached to Flat.
#[derive(Default)]
pub(crate) struct DataCodec<'a> {
    // Explicit global argument-output declarations determine callback ownership.
    pub(crate) callback_outputs: Option<&'a BTreeSet<String>>,
}
/// Complete class and enum declarations already attached to their source types.
pub(crate) fn prepare(flat: &mut Bindings) -> Result<()> {
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
            } else {
                return Err(format!("data class `{key}` field `{name}` has unsupported type `{field_key}`; expected a scalar, String, Vec<u8>, declared data class, or declared handle"));
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
impl DataCodec<'_> {
    fn enum_type<'a>(&self, flat: &'a Bindings, ty: &syn::Type) -> Option<&'a Rc<JniEnum>> {
        let ty = if let syn::Type::Reference(r) = ty {
            &r.elem
        } else {
            ty
        };
        flat.type_info(ty).ok()?.metadata.enumeration.as_ref()
    }
    fn source_record<'a>(
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
pub(crate) fn render_record(flat: &Bindings, class: &Class, ty: &syn::Type) -> Result<String> {
    let record = flat.record(ty)?;
    let fields = record
        .fields
        .iter()
        .map(|f| {
            format!(
                "    public val `{}`: {}",
                field_name(f),
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
                let n = field_name(f);
                match &f.metadata {
                    _ if f.metadata.bytes => {
                        format!("this.`{n}`.contentEquals(other.`{n}`)")
                    }
                    _ if f.metadata.floating => {
                        format!("this.`{n}`.compareTo(other.`{n}`) == 0")
                    }
                    _ => format!("this.`{n}` == other.`{n}`"),
                }
            })
            .collect::<Vec<_>>()
            .join(" &&\n            ");
        let hashes = record
            .fields
            .iter()
            .map(|f| {
                let n = field_name(f);
                if f.metadata.bytes {
                    format!("this.`{n}`.contentHashCode()")
                } else {
                    format!("this.`{n}`.hashCode()")
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
                let n = field_name(f);
                let suffix = if f.metadata.bytes {
                    ".contentToString()"
                } else {
                    ""
                };
                format!("{n}=${{this.`{n}`{suffix}}}")
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
    fn mark(
        &self,
        layout: &mut prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        root: String,
        condition: String,
    ) -> Result<()> {
        JniCodec.retire_niches(layout, None)?;
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

impl Codec for DataCodec<'_> {
    fn input_bulk_leaf(
        &self,
        flat: &Bindings,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        if self.enum_type(flat, ty).is_some() {
            self.input_leaf(flat, ty, &(), w, v, active)
        } else {
            crate::bulk::input_leaf(flat, ty, w, v, active)
        }
    }
    fn output_bulk_leaf(
        &self,
        flat: &Bindings,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<JniWire>> {
        if self.enum_type(flat, ty).is_some() {
            self.output_leaf(flat, ty, &(), w, v, borrowed)
        } else {
            crate::bulk::output_leaf(flat, ty, w, v, borrowed)
        }
    }
    fn bulk_input(
        &self,
        _: &Bindings,
        _: &syn::Type,
        _: &(),
        w: syn::Ident,
        child: InputPlan<JniWire>,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        crate::bulk::input(w, child, active)
    }
    fn bulk_output(
        &self,
        _: &Bindings,
        _: &syn::Type,
        _: &(),
        w: syn::Ident,
        child: OutputPlan<JniWire>,
    ) -> Result<OutputPlan<JniWire>> {
        crate::bulk::output(w, child)
    }
    fn sequence_input_bulk(
        &self,
        _: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &kotlin::JniNode,
        active: &Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<JniWire, JniChain>> {
        crate::bulk::sequence_input(w, n, i, element, active)
    }
    fn sequence_output_bulk(
        &self,
        _: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &kotlin::JniNode,
        values: &[(syn::Ident, TokenStream)],
        delivery: prebindgen_declarations::SequenceDelivery,
    ) -> Result<prebindgen_declarations::SequenceOutput<JniWire, JniChain>> {
        if delivery != prebindgen_declarations::SequenceDelivery::Value {
            return Err("bulk collection delivery cannot also use per-element callbacks".into());
        }
        crate::bulk::sequence_output(w, n, i, element, values)
    }

    type Schema = JniSchema;
    fn expand_input(
        &self,
        flat: &Flat<JniSchema>,
        ty: &syn::Type,
        tree: &Input<InputRequest>,
        _: &(),
    ) -> Result<Option<Input<InputRequest>>> {
        if !matches!(tree, Input::Leaf) {
            return Ok(None);
        }
        let Some(record) = Self::source_record(flat, ty)? else {
            return Ok(None);
        };
        if record.has_handles {
            return Err(format!(
                "data class `{}` contains handles and cannot be passed into Rust as a record",
                record.fqn()
            ));
        }
        Ok(Some(Input::Record(vec![
            Input::Leaf;
            flat.record(ty)?.fields.len()
        ])))
    }
    fn expand_output(
        &self,
        flat: &Flat<JniSchema>,
        ty: &syn::Type,
        tree: &Output<OutputRequest>,
        _: &(),
    ) -> Result<Option<Output<OutputRequest>>> {
        let policy = match tree {
            Output::Leaf => None,
            Output::Projected(p) => Some(p.clone()),
            _ => return Ok(None),
        };
        let Some(record) = Self::source_record(flat, ty)? else {
            return Ok(None);
        };
        let fields = &flat.record(ty)?.fields;
        for name in policy.iter().flat_map(|p| p.fields.keys()) {
            if !fields
                .iter()
                .any(|f| f.name.to_string().trim_start_matches("r#") == name)
            {
                return Err(format!(
                    "data class `{}`: unknown projection field `{name}`",
                    record.fqn()
                ));
            }
        }
        let inherited = policy.as_ref().map(Projection::inherited);
        let children = fields.iter().map(|field| {
            let name = field.name.to_string();
            let Some(policy) = &policy else { return Ok(Output::Leaf); };
            let child = policy.fields.get(name.trim_start_matches("r#")).unwrap_or(inherited.as_ref().unwrap());
            if policy.mode == ProjectionMode::Borrowed && child.mode == ProjectionMode::Owned {
                return Err(format!("data class `{}` field `{name}`: owned projection cannot consume a shared-reference occurrence", record.fqn()));
            }
            Ok(Output::Projected(child.clone()))
        }).collect::<Result<Vec<_>>>()?;
        Ok(Some(Output::Record {
            fields: children,
            mode: policy.map(|p| p.mode),
        }))
    }
    fn input_record_metadata(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        wire: &syn::Ident,
        active: &Condition,
        plan: &mut InputPlan<JniWire>,
        _: &(),
    ) -> Result<()> {
        Self::source_record(flat, ty)?
            .ok_or("missing data-class representation")?
            .mark(&mut plan.node, wire.to_string(), kotlin_condition(active))
    }
    fn output_record_metadata(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        wire: &syn::Ident,
        plan: &mut OutputPlan<JniWire>,
        _: &(),
    ) -> Result<()> {
        Self::source_record(flat, ty)?
            .ok_or("missing data-class representation")?
            .mark(&mut plan.outcomes.success, wire.to_string(), "true".into())
    }
    type Niche = crate::optional::JniNiche;
    type Chain = JniChain;
    type Branch = JniBranch;
    fn group_metadata(&self) -> JniBranch {
        JniBranch::Group(JniGroup::Values)
    }

    fn named_output_group(&self, name: &str) -> Result<JniBranch> {
        JniCodec.named_output_group(name)
    }

    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
        controls: &[prebindgen_flat::Wire<JniWire>],
    ) -> Result<JniChain> {
        JniCodec.optional_metadata(presence, discriminator, niche, controls)
    }
    fn alternatives_metadata(
        &self,
        selector: &prebindgen_flat::Wire<JniWire>,
    ) -> Result<JniBranch> {
        JniCodec.alternatives_metadata(selector)
    }
    fn field_group(&self, name: syn::Ident) -> JniBranch {
        JniCodec.field_group(name)
    }
    fn fields_group(&self) -> JniBranch {
        JniCodec.fields_group()
    }
    type Extra = ();
    type Wire = JniWire;
    type Context = JniContext;

    fn default_output_sequence_is_bulk(&self, flat: &Bindings, extra: &()) -> bool {
        JniCodec.default_output_sequence_is_bulk(flat, extra)
    }
    fn sequence_input(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        active: &Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<JniWire, crate::JniChain>> {
        JniCodec.sequence_input(e, w, n, i, element, active)
    }
    fn sequence_output(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        values: &[(syn::Ident, TokenStream)],
        delivery: prebindgen_declarations::SequenceDelivery,
    ) -> Result<prebindgen_declarations::SequenceOutput<JniWire, crate::JniChain>> {
        JniCodec.sequence_output(e, w, n, i, element, values, delivery)
    }
    fn binding_failure_layout(
        &self,
        e: &(),
    ) -> Result<Option<prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>>> {
        JniCodec.binding_failure_layout(e)
    }
    fn source_error_layout(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        mapped: bool,
        _: &(),
    ) -> Result<prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>> {
        flat.type_info(ty)?
            .metadata
            .errors
            .get(&mapped)
            .map(|e| e.output.output_outcomes().success.clone())
            .ok_or_else(|| "missing logical JNI error projection".into())
    }
    fn result_error_value(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        value: TokenStream,
        message: Option<TokenStream>,
        _: &(),
    ) -> Result<TokenStream> {
        let helper = flat
            .type_info(ty)?
            .metadata
            .errors
            .get(&message.is_some())
            .map(|e| &e.helper)
            .ok_or("missing declared JNI error output")?;
        let env = prebindgen_conversions::template_parameter("environment");
        let reported = prebindgen_conversions::template_parameter("domain_output");
        let sink = prebindgen_conversions::template_parameter("return_sink");
        let value = message.unwrap_or(value);
        Ok(quote!({#helper(&mut #env, #value, &mut #reported, #sink)?; String::new()}))
    }
    fn niche(&self, wire: &JniWire) -> Option<crate::optional::JniNiche> {
        wire.available_niche()
    }
    fn set_niche(&self, wire: &mut JniWire, niche: Option<JniNiche>) {
        JniCodec.set_niche(wire, niche);
    }
    fn is_present(
        &self,
        wire: &JniWire,
        niche: &Self::Niche,
        value: TokenStream,
    ) -> Result<TokenStream> {
        Ok(niche.present(wire, value))
    }
    fn remaining_niche(&self, wire: &JniWire, niche: &Self::Niche) -> Option<Self::Niche> {
        optional::remaining(wire, niche)
    }
    fn absent_value(
        &self,
        wire: &JniWire,
        niche: Option<&crate::optional::JniNiche>,
        input: bool,
    ) -> Result<TokenStream> {
        optional::absent(wire, niche, input)
    }
    fn selector(&self, extra: &()) -> JniWire {
        JniCodec.selector(extra)
    }
    fn consumes(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        extra: &(),
    ) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(flat, inner, extra);
        }
        !flat
            .type_info(ty)
            .ok()
            .and_then(|t| t.metadata.class.as_ref())
            .is_some_and(|c| c.data)
            && self.enum_type(flat, ty).is_none()
            && JniCodec.consumes(flat, ty, extra)
    }
    fn finish_input(&self, plan: &mut InputPlan<JniWire>, extra: &()) -> Result<()> {
        JniCodec.finish_input(plan, extra)
    }
    fn finish_output(&self, plan: &mut OutputPlan<JniWire>, extra: &()) -> Result<()> {
        JniCodec.finish_output(plan, extra)?;
        if sequence::callback_delivery(&plan.outcomes.success)? {
            let env = prebindgen_conversions::template_parameter("environment");
            plan.delivery.statements.extend(quote!(#env.set_boolean_array_region(&__pb_output_accepted,0,&[1]).map_err(|e|e.to_string())?;));
        }
        Ok(())
    }
    fn callback_input(
        &self,
        flat: &Bindings,
        configuration: &prebindgen_declarations::CallbackInput,
        _: &(),
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        let outputs = self
            .callback_outputs
            .ok_or("source callback parameters require adapter callback preparation")?;
        callback::input(
            flat,
            configuration,
            outputs,
            &configuration.ty,
            wire,
            var,
            active,
        )
    }
    fn input_leaf(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        if prebindgen_flat::callback_signature(ty)?.is_some() {
            return Err("resolve Callback operations before leaf compilation".into());
        }
        if Self::source_record(flat, ty)?.is_some() {
            return Err("resolve the data-class Input::Record before leaf compilation".into());
        }
        if let Some(e) = self.enum_type(flat, ty) {
            return e.input(ty, wire, var, active);
        }
        JniCodec.input_leaf(flat, ty, extra, wire, var, active)
    }
    fn output_leaf(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<JniWire>> {
        if Self::source_record(flat, ty)?.is_some() {
            return Err("resolve the data-class Output::Record before leaf compilation".into());
        }
        if let Some(e) = self.enum_type(flat, ty) {
            if matches!(ty,syn::Type::Reference(r) if r.mutability.is_some()) {
                return Err("mutable-reference outputs are unsupported".into());
            }
            return Ok(e.output(wire, value));
        }
        JniCodec.output_leaf(flat, ty, extra, wire, value, borrowed)
    }
    fn output_projected(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        value: TokenStream,
        projection: &Projection,
    ) -> Result<OutputPlan<JniWire>> {
        if Self::source_record(flat, ty)?.is_some() {
            return Err("resolve data-class projections before leaf compilation".into());
        }
        if let Some(e) = self.enum_type(flat, ty) {
            if projection.mode == ProjectionMode::Owned && matches!(ty, syn::Type::Reference(_)) {
                return Err("owned projection cannot consume a reference payload".into());
            }
            if !projection.fields.is_empty() {
                return Err("enum values do not have projection fields".into());
            }
            return Ok(e.output(wire, value));
        }
        JniCodec.output_projected(flat, ty, extra, wire, value, projection)
    }

    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<prebindgen_declarations::NicheReservation<Self::Niche>> {
        JniCodec.optional_niche(payload)
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &prebindgen_declarations::NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        JniCodec.optional_presence(payload, reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        JniCodec.absent_output_values(payload, reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        JniCodec.retire_niches(payload, reservation)
    }
    fn validate_input(&self, plan: &prebindgen_declarations::CodecInput<Self>) -> Result<()> {
        JniCodec.validate_input(plan)
    }
    fn validate_output(&self, plan: &prebindgen_declarations::CodecOutput<Self>) -> Result<()> {
        JniCodec.validate_output(plan)
    }
}

type FieldNode<'a> = (
    &'a syn::Ident,
    &'a prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
);
fn fields(nodes: &[kotlin::JniNode]) -> Result<Vec<FieldNode<'_>>> {
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
) -> Result<KotlinInput> {
    let inner = nodes
        .iter()
        .flat_map(kotlin::leaf_wires)
        .collect::<Vec<_>>();
    let mut out = KotlinInput::default();
    let mut c = kotlin::input_components(nodes, context)?;
    out.parameters.push(format!(
        "{}: {}{}",
        d.root,
        d.class_name,
        if d.condition == "true" { "" } else { "?" }
    ));
    let fields = fields(nodes)?;
    if c.parameters.len() != fields.len() {
        return Err("incomplete data-class input occurrence".into());
    }
    for (p, (field, children)) in c.parameters.iter().zip(&fields) {
        let name = p.split_once(": ").unwrap().0;
        let receiver = if d.condition == "true" {
            d.root.clone()
        } else {
            format!("requireNotNull({})", d.root)
        };
        let expr = format!(
            "{receiver}.`{}`",
            field.to_string().trim_start_matches("r#")
        );
        for a in c
            .arguments
            .iter_mut()
            .chain(c.locks.iter_mut())
            .chain(c.consumed.iter_mut())
            .chain(c.lock_groups.iter_mut())
            .chain(c.setup.iter_mut().map(|b| &mut b.expression))
        {
            if matches!(kotlin::leaf_wires(children).as_slice(), [w] if matches!(w.kind, JniWire::Scalar { primitive: Primitive::I128 | Primitive::U128, .. }))
            {
                *a = a.replace(
                    &format!("{name}?.toByteArray()"),
                    &format!("{name}.toByteArray()"),
                );
            }
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
    out.locks.extend(c.locks);
    out.consumed.extend(c.consumed);
    out.setup.extend(c.setup);
    out.lock_groups.extend(c.lock_groups);
    Ok(out)
}
pub(crate) fn output_kotlin(
    nodes: &[kotlin::JniNode],
    d: &DataGroup,
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let mut c = kotlin::output_components(nodes, context)?;
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
        let codec = DataCodec::default();
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
                .output_leaf(
                    &flat,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    false,
                )
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
            variant_names: vec![],
        }));
        prepare(&mut flat).unwrap();
        let codec = DataCodec::default();
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
        assert!(input.arguments[0].contains("point.`type`"));
        let output = kotlin::output_kotlin(&field, &context).unwrap();
        assert_eq!(output.result_types, ["example.Point"]);
        assert!(output.delivered[0].contains("example.Point(result[0]"));
        // An empty record still denotes one Kotlin object with no ABI wires.
        let empty = record("empty");
        let output = kotlin::output_kotlin(&empty, &context).unwrap();
        assert_eq!(output.delivered, ["example.Empty()"]);
    }
}
