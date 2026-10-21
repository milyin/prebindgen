//! Kotlin record declarations and their occurrence conversions.
use super::*;
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
impl ClassDecl {
    pub(crate) fn ty(&self) -> &syn::Type {
        match self {
            Self::Ptr(c) => &c.ty,
            Self::Data(c) => &c.ty,
        }
    }
    pub(crate) fn name(&self) -> &Option<String> {
        match self {
            Self::Ptr(c) => &c.name,
            Self::Data(c) => &c.name,
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
        }
    }
    fn slots(&self) -> usize {
        match self {
            Self::Value(Value::Scalar(p)) => scalar::parameter_slots(*p),
            _ => 1,
        }
    }
}
/// Destination record grouping stored on a conversion plan, never in its wires.
#[derive(Clone, Debug)]
pub struct DataGroup {
    pub(crate) root: String,
    pub(crate) record: Rc<RecordShape>,
    pub(crate) condition: String,
}

pub(crate) struct DataCodec {
    records: BTreeMap<String, Rc<RecordShape>>,
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
        let mut records = BTreeMap::new();
        for (key, class) in classes {
            if class.data {
                resolve(flat, classes, key, &mut Vec::new(), &mut records)?;
            }
        }
        Ok(Self { records })
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
            Shape::Value(_) => false,
        })
    }

    fn input(
        &self,
        w: &syn::Ident,
        v: &syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniKind>> {
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
        Rc::new(self.clone()).mark(
            &plan.wires,
            &mut plan.metadata,
            w.to_string(),
            kotlin_condition(active),
        )?;
        Ok(plan)
    }
    fn output(
        &self,
        w: &syn::Ident,
        value: TokenStream,
        projection: &Projection,
    ) -> Result<OutputPlan<JniKind>> {
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
        Rc::new(self.clone()).mark(
            &plan.wires,
            &mut plan.metadata,
            w.to_string(),
            "true".into(),
        )?;
        Ok(plan)
    }
    fn mark(
        self: &Rc<Self>,
        wires: &[prebindgen_flat::Wire<JniKind>],
        metadata: &mut JniComposition,
        root: String,
        condition: String,
    ) -> Result<()> {
        // Record fields do not donate niches to enclosing optional occurrences.
        metadata.suppress(wires);
        metadata.enclose(
            root.clone(),
            wires.iter().map(|w| w.name.clone()).collect(),
            prebindgen_declarations::GroupKind::Adapter(DataGroup {
                root,
                record: self.clone(),
                condition,
            }),
        )
    }
    pub(crate) fn construct(&self, values: &mut impl Iterator<Item = String>) -> String {
        let fields = self
            .fields
            .iter()
            .map(|_| values.next().expect("one value per field"))
            .collect::<Vec<_>>();
        format!("{}({})", self.fqn, fields.join(", "))
    }
}

impl Codec for DataCodec {
    type Niche = crate::optional::JniNiche;
    type Group = DataGroup;
    type Extra = ();
    type Wire = JniKind;
    type Context = JniContext;
    type Input = JniInput;
    type Output = JniOutput;
    fn niche(&self, wire: &JniKind) -> Option<crate::optional::JniNiche> {
        optional::niche(wire)
    }
    fn is_present(
        &self,
        _: &JniKind,
        niche: &Self::Niche,
        value: TokenStream,
    ) -> Result<TokenStream> {
        Ok(niche.present(value))
    }
    fn remaining_niche(&self, wire: &JniKind, niche: &Self::Niche) -> Option<Self::Niche> {
        optional::remaining(wire, niche)
    }
    fn absent_value(
        &self,
        wire: &JniKind,
        niche: Option<&crate::optional::JniNiche>,
        input: bool,
    ) -> Result<TokenStream> {
        optional::absent(wire, niche, input)
    }
    fn selector(&self, extra: &()) -> JniKind {
        JniCodec.selector(extra)
    }
    fn consumes(&self, ty: &syn::Type, extra: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(inner, extra);
        }
        !self.records.contains_key(&type_key(ty)) && JniCodec.consumes(ty, extra)
    }
    fn finish_input(&self, plan: &mut InputPlan<JniKind>, extra: &()) -> Result<()> {
        JniCodec.finish_input(plan, extra)
    }
    fn finish_output(&self, plan: &mut OutputPlan<JniKind>, extra: &()) -> Result<()> {
        JniCodec.finish_output(plan, extra)
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniKind>> {
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
    ) -> Result<OutputPlan<JniKind>> {
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
    ) -> Result<OutputPlan<JniKind>> {
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
    ) -> Result<InputPlan<JniKind>> {
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
            _ => JniCodec.input_leaf(ty, &(), w.clone(), v.clone(), active),
        }
    }
    fn output(
        &self,
        ty: &syn::Type,
        w: &syn::Ident,
        value: TokenStream,
        policy: &Projection,
    ) -> Result<OutputPlan<JniKind>> {
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
            _ => JniCodec.output_projected(ty, &(), w.clone(), value, policy),
        }
    }
}

pub(crate) fn input_kotlin(
    wires: &[prebindgen_flat::Wire<JniKind>],
    metadata: &JniComposition,
    roots: &[String],
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let mut out = KotlinInput::default();
    let mut i = 0;
    while i < wires.len() {
        let group = metadata.starting_at(roots, &wires[i].name);
        let Some(prebindgen_declarations::CompositionGroup {
            kind: prebindgen_declarations::GroupKind::Adapter(d),
            ..
        }) = group
        else {
            let end = (i + 1..wires.len())
                .find(|j| {
                    metadata
                        .starting_at(roots, &wires[*j].name)
                        .is_some_and(|g| {
                            matches!(g.kind, prebindgen_declarations::GroupKind::Adapter(_))
                        })
                })
                .unwrap_or(wires.len());
            let scope = roots
                .iter()
                .filter(|id| {
                    metadata.groups[*id]
                        .members
                        .first()
                        .is_some_and(|n| wires[i..end].iter().any(|w| &w.name == n))
                })
                .cloned()
                .collect::<Vec<_>>();
            let c = kotlin::input_kotlin(&wires[i..end], metadata, &scope, context)?;
            out.parameters.extend(c.parameters);
            out.arguments.extend(c.arguments);
            out.locks.extend(c.locks);
            out.consumed.extend(c.consumed);
            i = end;
            continue;
        };
        let group = group.unwrap();
        let end = i + group.members.len();
        let inner = &wires[i..end];
        let mut c = kotlin::input_kotlin(inner, metadata, &group.children, context)?;
        out.parameters.push(format!(
            "{}: {}{}",
            d.root,
            d.record.fqn,
            if d.condition == "true" { "" } else { "?" }
        ));
        for p in &c.parameters {
            let name = p.split_once(": ").unwrap().0;
            let field = d
                .record
                .fields
                .iter()
                .enumerate()
                .find(|(index, _)| {
                    let prefix = format!("{}_f{index}", d.root);
                    name == prefix || name.starts_with(&(prefix + "_"))
                })
                .ok_or("data parameter has no field")?
                .1;
            let receiver = if d.condition == "true" {
                d.root.clone()
            } else {
                format!("requireNotNull({})", d.root)
            };
            let expr = format!("{receiver}.`{}`", field_name(field));
            for a in c
                .arguments
                .iter_mut()
                .chain(c.locks.iter_mut())
                .chain(c.consumed.iter_mut())
            {
                if matches!(
                    field.shape,
                    Shape::Value(Value::Scalar(Primitive::I128 | Primitive::U128))
                ) {
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
        i = end;
    }
    Ok(out)
}
pub(crate) fn output_kotlin(
    wires: &[prebindgen_flat::Wire<JniKind>],
    metadata: &JniComposition,
    roots: &[String],
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let mut out = KotlinOutput {
        delivered: vec![],
        result_types: vec![],
        output_locals: String::new(),
        materialize: String::new(),
        cleanup: String::new(),
    };
    let mut i = 0;
    while i < wires.len() {
        let group = metadata.starting_at(roots, &wires[i].name);
        let (end, record, scope) = if let Some(prebindgen_declarations::CompositionGroup {
            kind: prebindgen_declarations::GroupKind::Adapter(d),
            members,
            children,
            ..
        }) = group
        {
            (i + members.len(), Some(d.record.clone()), children.clone())
        } else {
            let end = (i + 1..wires.len())
                .find(|j| {
                    metadata
                        .starting_at(roots, &wires[*j].name)
                        .is_some_and(|g| {
                            matches!(g.kind, prebindgen_declarations::GroupKind::Adapter(_))
                        })
                })
                .unwrap_or(wires.len());
            let scope = roots
                .iter()
                .filter(|id| {
                    metadata.groups[*id]
                        .members
                        .first()
                        .is_some_and(|n| wires[i..end].iter().any(|w| &w.name == n))
                })
                .cloned()
                .collect();
            (end, None, scope)
        };
        let inner = &wires[i..end];
        let local = format!("{}_record{i}", context.result);
        let prefix = format!("{}r{i}_", context.output_prefix);
        let ctx = KotlinContext {
            result: &local,
            output_prefix: &prefix,
            ..*context
        };
        let mut c = kotlin::output_kotlin(inner, metadata, &scope, &ctx)?;
        for text in
            c.delivered
                .iter_mut()
                .chain([&mut c.output_locals, &mut c.materialize, &mut c.cleanup])
        {
            for j in 0..inner.len() {
                *text = text.replace(
                    &format!("{local}[{j}]"),
                    &format!("{}[{}]", context.result, i + j),
                );
            }
        }
        out.output_locals.push_str(&c.output_locals);
        out.materialize.push_str(&c.materialize);
        out.cleanup.push_str(&c.cleanup);
        if let Some(r) = record {
            if c.delivered.len() != r.fields.len() {
                return Err("incomplete data-class output occurrence".into());
            }
            out.delivered
                .push(r.construct(&mut c.delivered.into_iter()));
            out.result_types.push(r.fqn.clone());
        } else {
            out.delivered.extend(c.delivered);
            out.result_types.extend(c.result_types);
        }
        i = end;
    }
    Ok(out)
}
