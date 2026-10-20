//! Kotlin value-record declarations and their occurrence conversions.
use super::*;
use prebindgen_declarations::Condition;
use std::rc::Rc;

/// A Kotlin data class whose fields are copied across the JNI boundary.
///
/// Supports owned public named-field records containing scalars, `String`,
/// `Vec<u8>`, and other declared data classes. Handles, references, empty records,
/// and recursive records are rejected. No native allocation is owned by this class.
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
    /// A value record copied between Kotlin and Rust.
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
/// Declare a Kotlin value record from Rust type syntax.
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
    Value(Value),
    Record(Rc<RecordShape>),
}
impl Shape {
    fn kotlin_type(&self) -> String {
        match self {
            Self::Value(Value::Scalar(Primitive::I128 | Primitive::U128)) => {
                "java.math.BigInteger".into()
            }
            Self::Value(Value::Scalar(p)) => format!("kotlin.{}", scalar::kotlin(*p)),
            Self::Value(Value::Text) => "kotlin.String".into(),
            Self::Value(Value::Bytes) => "kotlin.ByteArray".into(),
            Self::Record(r) => r.fqn.clone(),
        }
    }
    fn slots(&self) -> usize {
        match self {
            Self::Value(Value::Scalar(p)) => scalar::parameter_slots(*p),
            _ => 1,
        }
    }
}
/// Private preparation metadata connecting a flattened value wire to a data class.
///
/// Constructed by the JNI declaration compiler; wrapper writers see only its
/// underlying transport through [`JniWire`].
#[derive(Clone, Debug)]
pub struct DataWire {
    pub(crate) leaf: JniKind,
    pub(crate) root: String,
    pub(crate) record: Rc<RecordShape>,
    pub(crate) path: Vec<String>,
    pub(crate) condition: String,
}
impl JniKind {
    pub(crate) fn leaf(&self) -> &Self {
        match self {
            Self::Data(d) => d.leaf.leaf(),
            _ => self,
        }
    }
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
                let field_key = type_key(&f.ty);
                let shape = if matches!(f.ty, syn::Type::Reference(_)) {
                    return Err(format!(
                        "data class `{key}` field `{name}` cannot be a reference"
                    ));
                } else if let Some(value) = Value::of(&f.ty) {
                    Shape::Value(value)
                } else if classes.get(&field_key).is_some_and(|c| c.data) {
                    Shape::Record(resolve(flat, classes, &field_key, stack, records)?)
                } else {
                    return Err(format!("data class `{key}` field `{name}` has unsupported type `{field_key}`; expected a scalar, String, Vec<u8>, or declared data class"));
                };
                fields.push(DataField {
                    name: f.name.clone(),
                    ty: f.ty.clone(),
                    shape,
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
        let mut out = format!(
            "package {}\n\npublic data class {}(\n{fields}\n)",
            class.pkg, class.name
        );
        if shape
            .fields
            .iter()
            .any(|f| matches!(f.shape, Shape::Value(Value::Bytes)))
        {
            let equal = shape
                .fields
                .iter()
                .map(|f| {
                    let n = field_name(f);
                    match f.shape {
                        Shape::Value(Value::Bytes) => {
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
                    if matches!(f.shape, Shape::Value(Value::Bytes)) {
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
                    let suffix = if matches!(f.shape, Shape::Value(Value::Bytes)) {
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
    fn input(
        &self,
        w: &syn::Ident,
        v: &syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniKind>> {
        let plan = prebindgen_declarations::record_input(
            &JniCodec,
            &self.ty,
            v.clone(),
            self.fields.iter().enumerate().map(|(i, f)| {
                let fw = format_ident!("{w}_f{i}");
                let fv = format_ident!("{v}_f{i}");
                let child = match &f.shape {
                    Shape::Value(_) => JniCodec.input_leaf(&f.ty, &(), fw, fv, active)?,
                    Shape::Record(r) => r.input(&fw, &fv, active)?,
                };
                Ok((f.name.clone(), child))
            }),
        )?;
        debug_assert!(
            plan.acquire.is_empty() && plan.resources.is_empty() && plan.acknowledgement.is_none(),
            "data fields must remain value-only"
        );
        Ok(plan)
    }
    fn output(&self, w: &syn::Ident, value: TokenStream) -> Result<OutputPlan<JniKind>> {
        let plan = prebindgen_declarations::record_output(
            &JniCodec,
            format_ident!("{w}_record"),
            value,
            self.fields.iter().map(|f| f.name.clone()),
            |i, field| {
                let f = &self.fields[i];
                let fw = format_ident!("{w}_f{i}");
                match &f.shape {
                    Shape::Value(_) => JniCodec.output_leaf(&f.ty, &(), fw, field, true),
                    Shape::Record(r) => r.output(&fw, field),
                }
            },
        )?;
        debug_assert!(
            plan.commit.is_empty(),
            "data fields cannot transfer native ownership"
        );
        Ok(plan)
    }
    fn paths(&self, path: &[String], paths: &mut Vec<Vec<String>>) {
        for f in &self.fields {
            let mut path = path.to_vec();
            path.push(field_name(f));
            match &f.shape {
                Shape::Value(_) => paths.push(path),
                Shape::Record(r) => r.paths(&path, paths),
            }
        }
    }
    fn mark(
        self: &Rc<Self>,
        wires: &mut [prebindgen_flat::Wire<JniKind>],
        root: String,
        condition: String,
    ) {
        let mut paths = Vec::new();
        self.paths(&[], &mut paths);
        assert_eq!(wires.len(), paths.len());
        for (wire, path) in wires.iter_mut().zip(paths) {
            wire.kind = JniKind::Data(Box::new(DataWire {
                leaf: wire.kind.clone(),
                root: root.clone(),
                record: self.clone(),
                path,
                condition: condition.clone(),
            }));
        }
    }
    pub(crate) fn construct(&self, values: &mut impl Iterator<Item = String>) -> String {
        let fields = self
            .fields
            .iter()
            .map(|f| match &f.shape {
                Shape::Value(_) => values.next().expect("one expression per record leaf"),
                Shape::Record(r) => r.construct(values),
            })
            .collect::<Vec<_>>();
        format!("{}({})", self.fqn, fields.join(", "))
    }
    pub(crate) fn leaf_count(&self) -> usize {
        self.fields
            .iter()
            .map(|f| match &f.shape {
                Shape::Value(_) => 1,
                Shape::Record(r) => r.leaf_count(),
            })
            .sum()
    }
}
impl Codec for DataCodec {
    type Extra = ();
    type Wire = JniKind;
    type Context = JniContext;
    type Input = JniInput;
    type Output = JniOutput;
    fn selector(&self, extra: &()) -> JniKind {
        JniCodec.selector(extra)
    }
    fn consumes(&self, ty: &syn::Type, extra: &()) -> bool {
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
        let Some(record) = self.record(ty)? else {
            return JniCodec.input_leaf(ty, extra, wire, var, active);
        };
        let mut plan = record.input(&wire, &var, active)?;
        record.mark(&mut plan.wires, wire.to_string(), kotlin_condition(active));
        Ok(plan)
    }
    fn output_leaf(
        &self,
        ty: &syn::Type,
        extra: &(),
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<JniKind>> {
        let Some(record) = self.record(ty)? else {
            return JniCodec.output_leaf(ty, extra, wire, value, borrowed);
        };
        let mut plan = record.output(&wire, value)?;
        record.mark(&mut plan.wires, wire.to_string(), "true".into());
        Ok(plan)
    }
}
