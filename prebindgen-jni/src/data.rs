//! Kotlin record declarations and their occurrence conversions.
use super::*;
use prebindgen_conversions::NativeLayoutExt;
use prebindgen_declarations::Condition;
use std::rc::Rc;

/// A Kotlin data class whose fields are converted across the JNI boundary.
///
/// Supports owned public named-field records containing scalars, `String`,
/// `Vec<u8>`, other declared data classes, declared opaque handles, and optional fields.
/// Records containing handles (including nested ones) support returns only.
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
impl ClassDecl {
    pub(crate) fn ty(&self) -> &syn::Type {
        match self {
            Self::Ptr(c) => &c.ty,
            Self::Data(c) => &c.ty,
            Self::Enum(c) => &c.ty,
        }
    }
    pub(crate) fn name(&self) -> &Option<String> {
        match self {
            Self::Ptr(c) => &c.name,
            Self::Data(c) => &c.name,
            Self::Enum(c) => &c.name,
        }
    }
}
#[macro_export]
/// Declare a Kotlin data class from Rust type syntax.
///
/// Returns [`DataClassDecl`]. The source type need not be in scope. Invalid type
/// syntax panics; unsupported record fields are diagnosed during build.
macro_rules! data_class {($($ty:tt)+)=>{$crate::DataClassDecl::new($crate::__type(stringify!($($ty)+)))};}

#[derive(Clone, Debug)]
pub(crate) struct RecordShape {
    ty: syn::Type,
    pub(crate) fqn: String,
    fields: Vec<DataField>,
}
#[derive(Clone, Debug)]
struct DataField {
    name: syn::Ident,
    ty: syn::Type,
    shape: Shape,
}
#[derive(Clone, Debug)]
enum Shape {
    Optional(Box<Shape>),
    Value(Value),
    Record(Rc<RecordShape>),
    Handle(String),
    Enum(Box<JniEnum>),
}
impl Shape {
    fn is_bytes(&self) -> bool {
        match self {
            Self::Value(Value::Bytes) => true,
            Self::Optional(s) => s.is_bytes(),
            _ => false,
        }
    }
    fn kotlin_type(&self) -> String {
        match self {
            Self::Optional(inner) => format!("{}?", inner.kotlin_type()),
            Self::Value(Value::Scalar(Primitive::I128 | Primitive::U128)) => {
                "java.math.BigInteger".into()
            }
            Self::Value(Value::Scalar(p)) => format!("kotlin.{}", scalar::kotlin(*p)),
            Self::Value(Value::Text) => "kotlin.String".into(),
            Self::Value(Value::Bytes) => "kotlin.ByteArray".into(),
            Self::Record(r) => r.fqn.clone(),
            Self::Handle(fqn) => fqn.clone(),
            Self::Enum(e) => e.class_name.clone(),
        }
    }
    fn slots(&self) -> usize {
        match self {
            Self::Value(Value::Scalar(p)) => JniWire::scalar(*p).parameter_slots(),
            _ => 1,
        }
    }
}
/// Kotlin record information attached to a parent node in the wire tree.
#[derive(Clone, Debug)]
pub struct DataGroup {
    pub(crate) root: String,
    pub(crate) class_name: String,
    pub(crate) condition: String,
}

pub(crate) struct DataCodec {
    pub(crate) callback_wire: Option<syn::Ident>,
    pub(crate) error_classes: BTreeMap<String, String>,
    pub(crate) error_layouts:
        BTreeMap<(String, bool), prebindgen_flat::WireLayout<JniWire, JniGroup>>,
    pub(crate) error_helpers: BTreeMap<(String, bool), syn::Ident>,
    records: BTreeMap<String, Rc<RecordShape>>,
    pub(crate) enums: BTreeMap<String, JniEnum>,
}
impl DataCodec {
    pub(crate) fn new<I, O>(flat: &Flat<I, O>, classes: &BTreeMap<String, Class>) -> Result<Self> {
        fn resolve<I, O>(
            flat: &Flat<I, O>,
            classes: &BTreeMap<String, Class>,
            key: &str,
            stack: &mut Vec<String>,
            records: &mut BTreeMap<String, Rc<RecordShape>>,
        ) -> Result<Rc<RecordShape>> {
            if stack.iter().any(|k| k == key) {
                return Err(format!("recursive data class `{key}`"));
            }
            if let Some(r) = records.get(key) {
                return Ok(r.clone());
            }
            let class = &classes[key];
            let record = flat
                .record(&class.ty)
                .map_err(|_| format!("data class `{key}` requires a named-field record"))?;
            if record.fields.is_empty() {
                return Err(format!("data class `{key}` must have at least one field"));
            }
            stack.push(key.into());
            let mut fields = Vec::new();
            let mut getters = BTreeSet::from(["getClass".to_string()]);
            for f in &record.fields {
                let name = f.name.to_string();
                let name = name.trim_start_matches("r#");
                validate_name(name)
                    .map_err(|e| format!("data class `{key}` field `{name}`: {e}"))?;
                // Distinct Rust names such as `x` and `X` can generate the
                // same JVM accessor even though Kotlin properties are distinct.
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
                        "data class `{key}` field `{name}` has a conflicting JVM getter `{getter}`"
                    ));
                }
                let optional = prebindgen_declarations::option_inner(&f.ty);
                let field_ty = optional.unwrap_or(&f.ty);
                if optional
                    .and_then(prebindgen_declarations::option_inner)
                    .is_some()
                {
                    return Err("nested Option data fields require an explicit expansion".into());
                }
                let field_key = type_key(field_ty);
                let shape = if matches!(field_ty, syn::Type::Reference(_)) {
                    return Err(format!(
                        "data class `{key}` field `{name}` cannot be a reference"
                    ));
                } else if let Some(value) = Value::of(field_ty) {
                    Shape::Value(value)
                } else if classes.get(&field_key).is_some_and(|c| c.data) {
                    Shape::Record(resolve(flat, classes, &field_key, stack, records)?)
                } else if let Some(class) = classes.get(&field_key).filter(|c| c.enumeration) {
                    Shape::Enum(Box::new(JniEnum::from_class(flat, class)?))
                } else if let Some(handle) = classes.get(&field_key) {
                    Shape::Handle(handle.fqn())
                } else {
                    return Err(format!("data class `{key}` field `{name}` has unsupported type `{field_key}`; expected a scalar, String, Vec<u8>, declared data class, or declared handle"));
                };
                fields.push(DataField {
                    name: f.name.clone(),
                    ty: f.ty.clone(),
                    shape: if optional.is_some() {
                        Shape::Optional(Box::new(shape))
                    } else {
                        shape
                    },
                });
            }
            // Kotlin's generated copy$default also needs a receiver, default masks,
            // and a marker. Check that method, not just the primary constructor.
            let slots = fields.iter().map(|f| f.shape.slots()).sum::<usize>()
                + fields.len().div_ceil(32)
                + 2;
            if slots > 255 {
                return Err(format!("data class `{key}` copy method needs {slots} JVM parameter slots; the limit is 255"));
            }
            stack.pop();
            let shape = Rc::new(RecordShape {
                ty: class.ty.clone(),
                fqn: class.fqn(),
                fields,
            });
            records.insert(key.into(), shape.clone());
            Ok(shape)
        }
        for (key, _) in classes.iter().filter(|(_, c)| c.enumeration) {
            if !flat.enums.contains_key(key) {
                return Err("enum_class requires a fieldless source enum".into());
            }
        }
        let mut records = BTreeMap::new();
        for (key, class) in classes {
            if class.data {
                resolve(flat, classes, key, &mut Vec::new(), &mut records)?;
            }
        }
        let mut enums = BTreeMap::new();
        for (key, class) in classes.iter().filter(|(_, c)| c.enumeration) {
            enums.insert(key.clone(), JniEnum::from_class(flat, class)?);
        }
        Ok(Self {
            records,
            enums,
            callback_wire: None,
            error_classes: BTreeMap::new(),
            error_helpers: BTreeMap::new(),
            error_layouts: BTreeMap::new(),
        })
    }
    fn enum_type(&self, ty: &syn::Type) -> Option<&JniEnum> {
        let ty = if let syn::Type::Reference(r) = ty {
            &r.elem
        } else {
            ty
        };
        self.enums.get(&type_key(ty))
    }
    fn record(&self, ty: &syn::Type) -> Result<Option<&Rc<RecordShape>>> {
        if let syn::Type::Reference(r) = ty {
            if self.records.contains_key(&type_key(&r.elem)) {
                return Err(format!(
                    "data class `{}` supports owned occurrences only",
                    type_key(&r.elem)
                ));
            }
        }
        Ok(self.records.get(&type_key(ty)))
    }
    pub(crate) fn render(&self, class: &Class) -> Result<String> {
        let shape = &self.records[&type_key(&class.ty)];
        let fields = shape
            .fields
            .iter()
            .map(|f| {
                format!(
                    "    public val `{}`: {}",
                    field_name(f),
                    f.shape.kotlin_type()
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let lifecycle = if shape.has_handles() {
            "/** Close each handle member explicitly. Copies share the same handle objects. */\n"
        } else {
            ""
        };
        let mut out = format!(
            "package {}\n\n{lifecycle}public data class {}(\n{fields}\n)",
            class.pkg, class.name
        );
        if shape.fields.iter().any(|f| f.shape.is_bytes()) {
            let equal = shape
                .fields
                .iter()
                .map(|f| {
                    let n = field_name(f);
                    match f.shape {
                        _ if f.shape.is_bytes() => {
                            format!("this.`{n}`.contentEquals(other.`{n}`)")
                        }
                        Shape::Value(Value::Scalar(Primitive::F32 | Primitive::F64)) => {
                            format!("this.`{n}`.compareTo(other.`{n}`) == 0")
                        }
                        _ => format!("this.`{n}` == other.`{n}`"),
                    }
                })
                .collect::<Vec<_>>()
                .join(" &&\n            ");
            let hashes = shape
                .fields
                .iter()
                .map(|f| {
                    let n = field_name(f);
                    if f.shape.is_bytes() {
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
            let display = shape
                .fields
                .iter()
                .map(|f| {
                    let n = field_name(f);
                    let suffix = if f.shape.is_bytes() {
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
}
fn field_name(field: &DataField) -> String {
    field.name.to_string().trim_start_matches("r#").into()
}

impl RecordShape {
    fn has_handles(&self) -> bool {
        self.fields.iter().any(|f| match &f.shape {
            Shape::Optional(s) => s.has_handles(),
            Shape::Handle(_) => true,
            Shape::Record(r) => r.has_handles(),
            Shape::Value(_) | Shape::Enum(_) => false,
        })
    }

    fn input(
        &self,
        w: &syn::Ident,
        v: &syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        if self.has_handles() {
            return Err(format!(
                "data class `{}` contains handles and supports returns only",
                self.fqn
            ));
        }
        let mut plan = prebindgen_declarations::record_input(
            &JniCodec,
            &self.ty,
            v.clone(),
            self.fields.iter().enumerate().map(|(i, f)| {
                let fw = format_ident!("{w}_f{i}");
                let fv = format_ident!("{v}_f{i}");
                let child = f.shape.input(&f.ty, &fw, &fv, active)?;
                Ok((f.name.clone(), child))
            }),
        )?;
        debug_assert!(
            plan.acquire.is_empty() && plan.resources.is_empty() && plan.acknowledgement.is_none(),
            "data fields must remain value-only"
        );
        self.mark(&mut plan.layout, w.to_string(), kotlin_condition(active))?;
        Ok(plan)
    }
    fn output(
        &self,
        w: &syn::Ident,
        value: TokenStream,
        projection: &Projection,
    ) -> Result<OutputPlan<JniWire>> {
        for name in projection.fields.keys() {
            if !self.fields.iter().any(|f| field_name(f) == *name) {
                return Err(format!(
                    "data class `{}`: unknown projection field `{name}`",
                    self.fqn
                ));
            }
        }
        let owned = projection.mode == ProjectionMode::Owned;
        let inherited = projection.inherited();
        let convert = |i: usize, field: TokenStream| {
            let f = &self.fields[i];
            let policy = projection.fields.get(&field_name(f)).unwrap_or(&inherited);
            if !owned && policy.mode == ProjectionMode::Owned {
                return Err(format!("data class `{}` field `{}`: owned projection cannot consume a shared-reference occurrence", self.fqn, field_name(f)));
            }
            let value = if owned && policy.mode == ProjectionMode::Borrowed {
                quote!(&(#field))
            } else {
                field
            };
            let fw = format_ident!("{w}_f{i}");
            f.shape.output(&f.ty, &fw, value, policy)
        };
        let mut plan = if owned {
            prebindgen_declarations::record_output_owned(
                &JniCodec,
                format_ident!("{w}_record"),
                value,
                self.fields.iter().map(|f| f.name.clone()),
                convert,
            )
        } else {
            prebindgen_declarations::record_output(
                &JniCodec,
                format_ident!("{w}_record"),
                value,
                self.fields.iter().map(|f| f.name.clone()),
                convert,
            )
        }?;
        self.mark(&mut plan.layout, w.to_string(), "true".into())?;
        Ok(plan)
    }
    fn mark(
        &self,
        layout: &mut prebindgen_flat::WireLayout<JniWire, JniGroup>,
        root: String,
        condition: String,
    ) -> Result<()> {
        for wire in layout.native_wires_mut() {
            wire.kind.clear_niche();
        }
        let mut nodes = std::mem::take(layout).into_nodes();
        let [prebindgen_flat::WireNode::Group { metadata, .. }] = nodes.as_mut_slice() else {
            return Err("record compiler must produce one field collection".into());
        };
        if !matches!(metadata, JniGroup::Fields(None)) {
            return Err("record annotation requires a field collection".into());
        }
        *metadata = JniGroup::Fields(Some(DataGroup {
            root,
            class_name: self.fqn.clone(),
            condition,
        }));
        *layout = prebindgen_flat::WireLayout::from_nodes(nodes);
        Ok(())
    }
}

impl Codec for DataCodec {
    type Niche = crate::optional::JniNiche;
    type Group = JniGroup;
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
        controls: Vec<prebindgen_flat::Wire<JniWire>>,
    ) -> JniGroup {
        JniCodec.optional_metadata(presence, discriminator, niche, controls)
    }
    fn sequence_metadata(&self, carrier: prebindgen_flat::Wire<JniWire>) -> JniGroup {
        JniCodec.sequence_metadata(carrier)
    }
    fn alternatives_metadata(&self, selector: prebindgen_flat::Wire<JniWire>) -> JniGroup {
        JniCodec.alternatives_metadata(selector)
    }
    fn field_group(&self, name: syn::Ident) -> JniGroup {
        JniCodec.field_group(name)
    }
    fn fields_group(&self) -> JniGroup {
        JniCodec.fields_group()
    }
    type Extra = ();
    type Wire = JniWire;
    type Context = JniContext;
    type Input = JniInput;
    type Output = JniOutput;
    fn sequence_input(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireLayout<JniWire, JniGroup>,
        active: &Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<JniWire>> {
        JniCodec.sequence_input(e, w, n, i, element, active)
    }
    fn sequence_output(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireLayout<JniWire, JniGroup>,
        values: &[TokenStream],
    ) -> Result<prebindgen_declarations::SequenceOutput<JniWire>> {
        if self.callback_wire.as_ref() == Some(&w) {
            crate::sequence_callback::transport(w, element, values)
        } else {
            JniCodec.sequence_output(e, w, n, i, element, values)
        }
    }
    fn binding_failure_layout(
        &self,
        e: &(),
    ) -> Result<Option<prebindgen_flat::WireLayout<JniWire, JniGroup>>> {
        JniCodec.binding_failure_layout(e)
    }
    fn source_error_layout(
        &self,
        ty: &syn::Type,
        mapped: bool,
        _: &(),
    ) -> Result<prebindgen_flat::WireLayout<JniWire, JniGroup>> {
        self.error_layouts
            .get(&(type_key(ty), mapped))
            .cloned()
            .ok_or_else(|| "missing logical JNI error projection".into())
    }
    fn result_error_value(
        &self,
        ty: &syn::Type,
        value: TokenStream,
        message: Option<TokenStream>,
        _: &(),
    ) -> Result<TokenStream> {
        let helper = self
            .error_helpers
            .get(&(type_key(ty), message.is_some()))
            .ok_or("missing declared JNI error output")?;
        let env = prebindgen_conversions::template_parameter("environment");
        let reported = prebindgen_conversions::template_parameter("domain_output");
        let value = message.unwrap_or(value);
        Ok(quote!({#helper(&mut #env, #value, &mut #reported)?; String::new()}))
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
    fn consumes(&self, ty: &syn::Type, extra: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(inner, extra);
        }
        !self.records.contains_key(&type_key(ty))
            && self.enum_type(ty).is_none()
            && JniCodec.consumes(ty, extra)
    }
    fn finish_input(&self, plan: &mut InputPlan<JniWire>, extra: &()) -> Result<()> {
        JniCodec.finish_input(plan, extra)
    }
    fn finish_output(&self, plan: &mut OutputPlan<JniWire>, extra: &()) -> Result<()> {
        JniCodec.finish_output(plan, extra)?;
        if self.callback_wire.is_some() {
            let env = prebindgen_conversions::template_parameter("environment");
            plan.delivery.statements.extend(quote!(#env.set_boolean_array_region(&__pb_output_accepted,0,&[1]).map_err(|e|e.to_string())?;));
        }
        Ok(())
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{wire}_present");
            let child = self.input_leaf(
                inner,
                extra,
                format_ident!("{wire}_some"),
                format_ident!("{var}_some"),
                &active.arm(tag.clone(), 1),
            )?;
            return prebindgen_declarations::optional_input(self, extra, tag, var, active, child);
        }
        if let Some(e) = self.enum_type(ty) {
            return e.input(ty, wire, var, active);
        }
        let Some(record) = self.record(ty)? else {
            return JniCodec.input_leaf(ty, extra, wire, var, active);
        };
        record.input(&wire, &var, active)
    }
    fn output_leaf(
        &self,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<JniWire>> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{wire}_present");
            let payload = format_ident!("{wire}_payload");
            let child = self.output_leaf(
                inner,
                extra,
                format_ident!("{wire}_some"),
                quote!(#payload),
                borrowed,
            )?;
            return prebindgen_declarations::optional_output(
                self, extra, tag, payload, value, borrowed, child,
            );
        }
        if let Some(e) = self.enum_type(ty) {
            if matches!(ty,syn::Type::Reference(r) if r.mutability.is_some()) {
                return Err("mutable-reference outputs are unsupported".into());
            }
            return Ok(e.output(wire, value));
        }
        let Some(record) = self.record(ty)? else {
            return JniCodec.output_leaf(ty, extra, wire, value, borrowed);
        };
        // The legacy data-class leaf remains a borrowed projection, regardless
        // of its fields. Ownership transfer requires an explicit projection.
        record.output(&wire, value, &Projection::borrowed())
    }
    fn output_projected(
        &self,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        value: TokenStream,
        projection: &Projection,
    ) -> Result<OutputPlan<JniWire>> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{wire}_present");
            let payload = format_ident!("{wire}_payload");
            let child = self.output_projected(
                inner,
                extra,
                format_ident!("{wire}_some"),
                quote!(#payload),
                projection,
            )?;
            return prebindgen_declarations::optional_output(
                self,
                extra,
                tag,
                payload,
                value,
                projection.mode == ProjectionMode::Borrowed,
                child,
            );
        }
        if let Some(e) = self.enum_type(ty) {
            if projection.mode == ProjectionMode::Owned && matches!(ty, syn::Type::Reference(_)) {
                return Err("owned projection cannot consume a reference payload".into());
            }
            if !projection.fields.is_empty() {
                return Err("enum values do not have projection fields".into());
            }
            return Ok(e.output(wire, value));
        }
        let Some(record) = self.record(ty)? else {
            return JniCodec.output_projected(ty, extra, wire, value, projection);
        };
        record.output(&wire, value, projection)
    }
}

impl Shape {
    fn has_handles(&self) -> bool {
        match self {
            Self::Handle(_) => true,
            Self::Record(r) => r.has_handles(),
            Self::Optional(s) => s.has_handles(),
            _ => false,
        }
    }
    fn input(
        &self,
        ty: &syn::Type,
        w: &syn::Ident,
        v: &syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        match self {
            Self::Optional(s) => {
                let tag = format_ident!("{w}_present");
                let child = s.input(
                    prebindgen_declarations::option_inner(ty).unwrap(),
                    &format_ident!("{w}_some"),
                    &format_ident!("{v}_some"),
                    &active.arm(tag.clone(), 1),
                )?;
                prebindgen_declarations::optional_input(
                    &JniCodec,
                    &(),
                    tag,
                    v.clone(),
                    active,
                    child,
                )
            }
            Self::Record(r) => r.input(w, v, active),
            Self::Enum(e) => e.input(ty, w.clone(), v.clone(), active),
            _ => JniCodec.input_leaf(ty, &(), w.clone(), v.clone(), active),
        }
    }
    fn output(
        &self,
        ty: &syn::Type,
        w: &syn::Ident,
        value: TokenStream,
        policy: &Projection,
    ) -> Result<OutputPlan<JniWire>> {
        match self {
            Self::Optional(s) => {
                let tag = format_ident!("{w}_present");
                let payload = format_ident!("{w}_payload");
                let child = s.output(
                    prebindgen_declarations::option_inner(ty).unwrap(),
                    &format_ident!("{w}_some"),
                    quote!(#payload),
                    policy,
                )?;
                prebindgen_declarations::optional_output(
                    &JniCodec,
                    &(),
                    tag,
                    payload,
                    value,
                    policy.mode == ProjectionMode::Borrowed,
                    child,
                )
            }
            Self::Record(r) => r.output(w, value, policy),
            Self::Enum(e) => {
                if !policy.fields.is_empty() {
                    return Err("enum values do not have projection fields".into());
                }
                Ok(e.output(w.clone(), value))
            }
            _ => JniCodec.output_projected(ty, &(), w.clone(), value, policy),
        }
    }
}

type FieldNode<'a> = (
    &'a syn::Ident,
    &'a prebindgen_flat::WireLayout<JniWire, JniGroup>,
);
fn fields(nodes: &[kotlin::JniNode]) -> Result<Vec<FieldNode<'_>>> {
    nodes
        .iter()
        .map(|node| match node {
            kotlin::JniNode::Group {
                metadata: JniGroup::Field(name),
                children,
            } => Ok((name, children)),
            _ => Err("Kotlin record requires named field children".into()),
        })
        .collect()
}
pub(crate) fn input_kotlin(
    nodes: &[kotlin::JniNode],
    d: &DataGroup,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let inner = kotlin::leaf_wires(nodes);
    let mut out = KotlinInput::default();
    let mut c = kotlin::input_kotlin(nodes, context)?;
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
            if matches!(children.nodes(), [kotlin::JniNode::Wire(w)] if matches!(w.kind, JniWire::Scalar { primitive: Primitive::I128 | Primitive::U128, .. }))
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
    let mut c = kotlin::output_kotlin(nodes, context)?;
    if c.delivered.len() != fields(nodes)?.len() {
        return Err("incomplete data-class output occurrence".into());
    }
    c.delivered = vec![format!("{}({})", d.class_name, c.delivered.join(", "))];
    c.result_types = vec![d.class_name.clone()];
    Ok(c)
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use prebindgen_flat::Wire;
    use prebindgen_flat::{WireLayout, WireNode};

    fn record(root: &str) -> WireNode<JniWire, JniGroup> {
        WireNode::Group {
            metadata: JniGroup::Fields(Some(DataGroup {
                root: root.into(),
                class_name: "example.Empty".into(),
                condition: "true".into(),
            })),
            children: WireLayout::default(),
        }
    }
    #[test]
    fn kotlin_record_and_presence_bindings_share_a_namespace() {
        let mut optional: WireLayout<JniWire, JniGroup> = WireLayout::default();
        optional.enclose_optional(Some(JniGroup::Optional(JniOptional {
            controls: vec![Wire {
                name: syn::parse_quote!(flag),
                kind: JniWire::selector(),
            }],
            presence: syn::parse_quote!(flag),
            discriminator: syn::parse_quote!(flag),
            niche: None,
        })));
        optional.extend(WireLayout::from_nodes(vec![record("value")]));
        crate::group::validate_layout(&optional).unwrap();
        optional.extend(WireLayout::from_nodes(vec![record("flag")]));
        assert!(crate::group::validate_layout(&optional)
            .unwrap_err()
            .contains("duplicate Kotlin group binding `flag`"));
        let duplicates = WireLayout::from_nodes(vec![record("value"), record("value")]);
        assert!(crate::group::validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate Kotlin group binding `value`"));
        // These adapter-owned names impose no constraint on the shared compiler.
        duplicates.validate().unwrap();
    }
    #[test]
    fn kotlin_record_emission_reads_names_from_shared_fields() {
        let mut field = WireLayout::from(vec![Wire {
            name: syn::parse_quote!(incoming),
            kind: JniWire::scalar(Primitive::I32),
        }]);
        field.enclose(JniGroup::Field(syn::parse_quote!(r#type)));
        field.enclose(JniGroup::Fields(Some(DataGroup {
            root: "point".into(),
            class_name: "example.Point".into(),
            condition: "true".into(),
        })));
        crate::group::validate_layout(&field).unwrap();
        let classes = BTreeMap::new();
        let context = KotlinContext {
            classes: &classes,
            result: "result",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "output",
        };
        let input = kotlin::input_kotlin(field.nodes(), &context).unwrap();
        assert_eq!(input.parameters, ["point: example.Point"]);
        assert!(input.arguments[0].contains("point.`type`"));
        let output = kotlin::output_kotlin(field.nodes(), &context).unwrap();
        assert_eq!(output.result_types, ["example.Point"]);
        assert!(output.delivered[0].contains("example.Point(result[0]"));
        // An empty record still denotes one Kotlin object with no ABI wires.
        let empty = WireLayout::from_nodes(vec![record("empty")]);
        let output = kotlin::output_kotlin(empty.nodes(), &context).unwrap();
        assert_eq!(output.delivered, ["example.Empty()"]);
    }
}
