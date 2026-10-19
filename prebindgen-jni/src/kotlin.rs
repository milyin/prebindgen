//! JNI slot converters own Kotlin conversion, locking and rollback behavior.
use super::*;
use prebindgen_flat::{InputConversion, OutputConversion, Resource, Wire, WireKind};

impl WireKind for JniKind {
    fn rust_type(&self) -> syn::Type {
        match self {
            JniKind::Scalar(p) => scalar::rust_type(*p),
            JniKind::Handle(_, _) => syn::parse_quote!(i64),
            JniKind::Selector => syn::parse_quote!(i32),
            JniKind::Text => {
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::JString<'local>)
            }
            JniKind::Bytes => {
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>)
            }
        }
    }
}
fn raw_kt(kind: &JniKind) -> &'static str {
    match kind {
        JniKind::Scalar(p) => scalar::raw_kotlin(*p),
        JniKind::Handle(_, _) => "Long",
        JniKind::Selector => "Int",
        JniKind::Text => "String?",
        JniKind::Bytes => "ByteArray?",
    }
}

/// Transport description, without source-type, ownership or selector semantics.
#[derive(Clone, Debug)]
pub struct JniWire {
    /// Rust parameter type at the JNI boundary.
    pub rust_type: syn::Type,
    /// Kotlin type at the native boundary.
    pub kotlin_type: &'static str,
    /// JVM signature slots occupied by this carrier.
    pub parameter_slots: usize,
}
impl WireKind for JniWire {
    fn rust_type(&self) -> syn::Type {
        self.rust_type.clone()
    }
}
impl From<&JniKind> for JniWire {
    fn from(kind: &JniKind) -> Self {
        Self {
            rust_type: kind.rust_type(),
            kotlin_type: raw_kt(kind),
            parameter_slots: match kind {
                JniKind::Handle(_, _) => 2,
                JniKind::Scalar(p) => scalar::parameter_slots(*p),
                _ => 1,
            },
        }
    }
}

/// Configured names for a generated handle class and its native destructor.
#[derive(Clone, Debug)]
pub struct KotlinClass {
    /// Fully qualified Kotlin class name.
    pub name: String,
    /// Fully qualified native function accepting a raw handle address.
    pub drop_function: String,
}
/// Names and class settings supplied by the Kotlin wrapper writer.
pub struct KotlinContext<'a> {
    /// Configured classes indexed by [`prebindgen_flat::type_key`].
    pub classes: &'a BTreeMap<String, KotlinClass>,
    /// Variable holding the native output array.
    pub result: &'a str,
    /// Caught delivery exception to which rollback failures are attached.
    pub failure: &'a str,
    /// Prefix for variables retaining materialized outputs during rollback.
    pub output_prefix: &'a str,
}
impl KotlinContext<'_> {
    /// Resolve the configured Kotlin class name for a source type.
    pub fn class_name(&self, ty: &syn::Type) -> Result<String> {
        Ok(self
            .classes
            .get(&type_key(ty))
            .ok_or_else(|| format!("no handle class for `{}`", type_key(ty)))?
            .name
            .clone())
    }
    /// Emit the configured native destructor invocation for an unwrapped handle.
    pub fn drop_handle(&self, ty: &syn::Type, value: &str) -> Result<String> {
        let class = self
            .classes
            .get(&type_key(ty))
            .ok_or_else(|| format!("no handle class for `{}`", type_key(ty)))?;
        Ok(format!("{}({value})", class.drop_function))
    }
}
/// Prepared Kotlin input code; the writer orders these operations across slots.
#[derive(Default)]
pub struct KotlinInput {
    /// Public parameter declarations.
    pub parameters: Vec<String>,
    /// Expressions passed to native wires, in wire order.
    pub arguments: Vec<String>,
    /// Handle expressions to lock for the duration of the native call.
    pub locks: Vec<String>,
    /// Statements run after native ownership acknowledgement, including on failure.
    pub consumed: Vec<String>,
}
/// Kotlin result conversion and rollback code owned by the result slot.
pub struct KotlinOutput {
    /// Expressions delivered to the caller, in result order.
    pub delivered: Vec<String>,
    /// Public Kotlin result types.
    pub result_types: Vec<String>,
    /// Declarations retaining partially materialized outputs.
    pub output_locals: String,
    /// Statements materializing outputs before delivery.
    pub materialize: String,
    /// Rollback statements executed if materialization or delivery fails.
    pub cleanup: String,
}
/// Input conversion with opaque Kotlin-side emission.
pub trait KotlinInputConversion: InputConversion<Wire = JniWire, Context = JniContext> {
    /// Emit destination-side input conversion for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput>;
}
/// Output conversion with opaque Kotlin-side emission.
pub trait KotlinOutputConversion: OutputConversion<Wire = JniWire, Context = JniContext> {
    /// Emit destination-side output conversion and rollback for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput>;
}
/// Self-contained input slot. Its preparation metadata is private.
#[derive(Clone, Debug)]
pub struct JniInput {
    rust: InputPlan<JniWire>,
    kotlin: Vec<Wire<JniKind>>,
}
/// Self-contained output slot. Its preparation metadata is private.
#[derive(Clone, Debug)]
pub struct JniOutput {
    rust: OutputPlan<JniWire>,
    kotlin: Vec<Wire<JniKind>>,
}
impl From<InputPlan<JniKind>> for JniInput {
    fn from(plan: InputPlan<JniKind>) -> Self {
        let wires = plan
            .wires
            .iter()
            .map(|wire| Wire {
                name: wire.name.clone(),
                kind: JniWire::from(&wire.kind),
            })
            .collect();
        Self {
            kotlin: plan.wires,
            rust: InputPlan {
                wires,
                context: Default::default(),
                acknowledgement: plan.acknowledgement,
                validation: plan.validation,
                acquire: plan.acquire,
                resources: plan.resources,
                statements: plan.statements,
                argument: plan.argument,
            },
        }
    }
}
impl From<OutputPlan<JniKind>> for JniOutput {
    fn from(plan: OutputPlan<JniKind>) -> Self {
        let wires = plan
            .wires
            .iter()
            .map(|wire| Wire {
                name: wire.name.clone(),
                kind: JniWire::from(&wire.kind),
            })
            .collect();
        Self {
            kotlin: plan.wires,
            rust: OutputPlan {
                wires,
                context: Default::default(),
                delivery: plan.delivery,
                commit: plan.commit,
                statements: plan.statements,
                values: plan.values,
            },
        }
    }
}
impl InputConversion for JniInput {
    type Wire = JniWire;
    type Context = JniContext;
    fn wires(&self) -> &[Wire<JniWire>] {
        self.rust.wires()
    }
    fn resources(&self, context: &JniContext) -> Result<Vec<Resource>> {
        self.rust.resources(context)
    }
    fn acknowledgement(&self, context: &JniContext) -> Result<Option<TokenStream>> {
        self.rust.acknowledgement(context)
    }
    fn validation(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.validation(context)
    }
    fn acquisition(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.acquisition(context)
    }
    fn construction(&self, context: &JniContext, target: &syn::Ident) -> Result<TokenStream> {
        self.rust.construction(context, target)
    }
}
impl OutputConversion for JniOutput {
    type Wire = JniWire;
    type Context = JniContext;
    fn wires(&self) -> &[Wire<JniWire>] {
        self.rust.wires()
    }
    fn values(&self, context: &JniContext) -> Result<Vec<TokenStream>> {
        self.rust.values(context)
    }
    fn preparation(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.preparation(context)
    }
    fn delivery(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.delivery(context)
    }
    fn commit(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.commit(context)
    }
    fn result(&self, context: &JniContext) -> Result<TokenStream> {
        self.rust.result(context)
    }
}
impl KotlinInputConversion for JniInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        let mut kt_params = Vec::new();
        let mut kt_args = Vec::new();
        let mut locks = Vec::new();
        let mut consumed = Vec::new();
        for w in &self.kotlin {
            let n = &w.name;
            let (kt, arg) = match &w.kind {
                JniKind::Scalar(p) => scalar::input(*p, n),
                JniKind::Handle(handle, condition) => {
                    locks.push(format!(
                        "if ({condition}) requireNotNull({n}) {{ \"null handle\" }} else null"
                    ));
                    if handle.access == Access::Move {
                        consumed.push(format!("if ({condition}) {n}!!.markConsumed()"));
                    }
                    (
                        format!("{}?", context.class_name(&handle.ty)?),
                        format!("{n}?.raw ?: 0L"),
                    )
                }
                _ => (raw_kt(&w.kind).into(), n.to_string()),
            };
            kt_params.push(format!("{n}: {kt}"));
            kt_args.push(arg);
        }
        Ok(KotlinInput {
            parameters: kt_params,
            arguments: kt_args,
            locks,
            consumed,
        })
    }
}
impl KotlinOutputConversion for JniOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        let result = context.result;
        let failure = context.failure;
        let output = context.output_prefix;
        let mut delivered = Vec::new();
        let mut result_types = Vec::new();
        for (i, w) in self.kotlin.iter().enumerate() {
            let (kt, decode) = match &w.kind {
                JniKind::Scalar(p) => scalar::output_kotlin(*p, i, result),
                JniKind::Handle(handle, _) => {
                    let c = context.class_name(&handle.ty)?;
                    (c.clone(), format!("{c}({result}[{i}] as Long)"))
                }
                JniKind::Text => ("String".into(), format!("{result}[{i}] as String")),
                JniKind::Bytes => ("ByteArray".into(), format!("{result}[{i}] as ByteArray")),
                JniKind::Selector => return Err("selector output is unsupported".into()),
            };
            delivered.push(decode);
            result_types.push(kt);
        }
        let mut output_locals = String::new();
        let mut materialize = String::new();
        let mut cleanup = String::new();
        for (i, w) in self.kotlin.iter().enumerate() {
            if let JniKind::Handle(handle, _) = &w.kind {
                let c = context.class_name(&handle.ty)?;
                output_locals.push_str(&format!("        var {output}{i}: {c}? = null\n"));
                materialize.push_str(&format!("            {output}{i} = {}\n", delivered[i]));
                delivered[i] = format!("{output}{i}");
                let drop = context.drop_handle(&handle.ty, &format!("{result}[{i}] as Long"))?;
                cleanup.push_str(&format!("            try {{ if ({output}{i} != null) {output}{i}.close() else {drop} }} catch (cleanup: Throwable) {{ {failure}.addSuppressed(cleanup) }}\n"));
            }
        }
        Ok(KotlinOutput {
            delivered,
            result_types,
            output_locals,
            materialize,
            cleanup,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_emit_private_conditions_and_use_supplied_output_bindings() {
        let classes = BTreeMap::from([(
            "Counter".into(),
            KotlinClass {
                name: "example.Counter".into(),
                drop_function: "example.Native.release".into(),
            },
        )]);
        let context = KotlinContext {
            classes: &classes,
            result: "native_values",
            failure: "delivery_error",
            output_prefix: "wrapped_",
        };
        // This condition is arbitrary Kotlin code, not a declaration selector path.
        let mut plan = JniCodec
            .input_leaf(
                &syn::parse_quote!(Counter),
                &(),
                syn::parse_quote!(incoming),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        let JniKind::Handle(_, condition) = &mut plan.wires[0].kind else {
            panic!()
        };
        *condition = "isEnabled() && count > 3".into();
        JniCodec.finish_input(&mut plan, &()).unwrap();
        let input = JniInput::from(plan);
        // A handle and an ordinary i64 expose identical transport information.
        let scalar = JniWire::from(&JniKind::Scalar(Primitive::I64));
        assert_eq!(
            prebindgen_flat::type_key(&input.wires()[0].kind.rust_type()),
            prebindgen_flat::type_key(&scalar.rust_type())
        );
        assert_eq!(input.wires()[0].kind.kotlin_type, scalar.kotlin_type);
        let emitted = input.kotlin(&context).unwrap();
        assert_eq!(emitted.locks, ["if (isEnabled() && count > 3) requireNotNull(incoming) { \"null handle\" } else null"]);
        assert_eq!(
            emitted.consumed,
            ["if (isEnabled() && count > 3) incoming!!.markConsumed()"]
        );

        let plan = JniCodec
            .output_leaf(
                &syn::parse_quote!(Counter),
                &(),
                syn::parse_quote!(wire),
                quote!(returned),
                false,
            )
            .unwrap();
        let output = JniOutput::from(plan).kotlin(&context).unwrap();
        assert_eq!(output.delivered, ["wrapped_0"]);
        assert!(output
            .materialize
            .contains("example.Counter(native_values[0] as Long)"));
        assert!(output
            .cleanup
            .contains("example.Native.release(native_values[0] as Long)"));
        assert!(output
            .cleanup
            .contains("delivery_error.addSuppressed(cleanup)"));
    }
}
