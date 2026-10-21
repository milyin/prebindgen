//! JNI slot converters own Kotlin conversion, locking and rollback behavior.
use super::*;
use crate::group::validate_layout;
use prebindgen_flat::{Conversion, Wire};

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
#[derive(Default)]
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
pub trait KotlinInputConversion:
    for<'a> Conversion<
    Wire = JniWire,
    SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, JniContext>,
>
{
    /// Emit destination-side input conversion for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput>;
}
/// Output conversion with opaque Kotlin-side emission.
pub trait KotlinOutputConversion:
    for<'a> Conversion<
    Wire = JniWire,
    SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, JniContext>,
    WireFragment<'a> = prebindgen_conversions::OutputWireFragment<'a, JniContext>,
>
{
    /// Emit destination-side output conversion and rollback for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput>;
}
/// Prepared JNI input, owning one tree used for Rust and Kotlin emission.
#[derive(Clone, Debug)]
pub struct JniInput {
    rust: InputPlan<JniWire>,
}
/// Prepared JNI output, owning one tree used for Rust and Kotlin emission.
#[derive(Clone, Debug)]
pub struct JniOutput {
    rust: OutputPlan<JniWire>,
}
impl From<InputPlan<JniWire>> for JniInput {
    fn from(rust: InputPlan<JniWire>) -> Self {
        Self { rust }
    }
}
impl From<OutputPlan<JniWire>> for JniOutput {
    fn from(rust: OutputPlan<JniWire>) -> Self {
        Self { rust }
    }
}
impl Conversion for JniInput {
    type Wire = JniWire;
    type Group = JniGroup;
    type SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, JniContext>;
    type WireFragment<'a> = prebindgen_conversions::InputWireFragment;
    fn layout(&self) -> &prebindgen_flat::WireLayout<JniWire, JniGroup> {
        &self.rust.layout
    }
    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.rust.code_fragment_slot(mode, expected)
    }
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: Self::WireFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.rust.code_fragment_wire(pos, mode, expected)
    }
}
impl Conversion for JniOutput {
    type Wire = JniWire;
    type Group = JniGroup;
    type SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, JniContext>;
    type WireFragment<'a> = prebindgen_conversions::OutputWireFragment<'a, JniContext>;
    fn layout(&self) -> &prebindgen_flat::WireLayout<JniWire, JniGroup> {
        &self.rust.layout
    }
    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.rust.code_fragment_slot(mode, expected)
    }
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: Self::WireFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.rust.code_fragment_wire(pos, mode, expected)
    }
}
impl KotlinInputConversion for JniInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        validate_layout(self.layout())?;
        input_kotlin(self.layout().nodes(), context)
    }
}
fn input_leaves(wires: &[&Wire<JniWire>], context: &KotlinContext<'_>) -> Result<KotlinInput> {
    let mut kt_params = Vec::new();
    let mut kt_args = Vec::new();
    let mut locks = Vec::new();
    let mut consumed = Vec::new();
    for w in wires {
        let n = &w.name;
        let (kt, arg) = match &w.kind {
            JniWire::Scalar { primitive: p, .. } => scalar::input(*p, &n.to_string()),
            JniWire::Handle {
                handle, condition, ..
            } => {
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
            _ => (w.kind.kotlin_type().into(), n.to_string()),
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
impl KotlinOutputConversion for JniOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        validate_layout(self.layout())?;
        output_kotlin(self.layout().nodes(), context)
    }
}
fn output_leaves(wires: &[&Wire<JniWire>], context: &KotlinContext<'_>) -> Result<KotlinOutput> {
    let result = context.result;
    let failure = context.failure;
    let output = context.output_prefix;
    let mut delivered = Vec::new();
    let mut result_types = Vec::new();
    for (i, w) in wires.iter().enumerate() {
        let (kt, decode) = match &w.kind {
            JniWire::Scalar { primitive: p, .. } => scalar::output_kotlin(*p, i, result),
            JniWire::Handle { handle, .. } => {
                let c = context.class_name(&handle.ty)?;
                (c.clone(), format!("{c}({result}[{i}] as Long)"))
            }
            JniWire::Text { .. } => ("String".into(), format!("{result}[{i}] as String")),
            JniWire::Bytes { .. } => ("ByteArray".into(), format!("{result}[{i}] as ByteArray")),
            JniWire::Selector { .. } => ("Int".into(), format!("{result}[{i}] as Int")),
        };
        delivered.push(decode);
        result_types.push(kt);
    }
    let mut output_locals = String::new();
    let mut materialize = String::new();
    let mut cleanup = String::new();
    for (i, w) in wires.iter().enumerate() {
        if let JniWire::Handle { handle, .. } = &w.kind {
            let c = context.class_name(&handle.ty)?;
            output_locals.push_str(&format!("        var {output}{i}: {c}? = null\n"));
            materialize.push_str(&format!("            {output}{i} = {}\n", delivered[i]));
            delivered[i] = format!("requireNotNull({output}{i})");
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

pub(crate) type JniNode = prebindgen_flat::WireNode<JniWire, JniGroup>;
pub(crate) fn leaf_wires(nodes: &[JniNode]) -> Vec<&Wire<JniWire>> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            JniNode::Wire(w) => out.push(w),
            JniNode::Group { children, .. } => out.extend(children.iter()),
        }
    }
    out
}
pub(crate) fn has_optional(nodes: &[JniNode]) -> bool {
    nodes.iter().any(|node| match node {
        JniNode::Wire(_) => false,
        JniNode::Group { metadata, children } => match metadata {
            JniGroup::Optional(_) => true,
            JniGroup::Fields(Some(_)) => false,
            _ => has_optional(children.nodes()),
        },
    })
}
pub(crate) fn input_kotlin(nodes: &[JniNode], context: &KotlinContext<'_>) -> Result<KotlinInput> {
    let mut out = KotlinInput::default();
    for node in nodes {
        let c = match node {
            JniNode::Wire(w) => input_leaves(&[w], context)?,
            JniNode::Group { metadata, children } => match metadata {
                JniGroup::Optional(o) => optional::input_kotlin(children.nodes(), o, context)?,
                JniGroup::Fields(Some(d)) => data::input_kotlin(children.nodes(), d, context)?,
                JniGroup::Fields(None) | JniGroup::Field(_) => {
                    input_kotlin(children.nodes(), context)?
                }
            },
        };
        out.parameters.extend(c.parameters);
        out.arguments.extend(c.arguments);
        out.locks.extend(c.locks);
        out.consumed.extend(c.consumed);
    }
    Ok(out)
}
pub(crate) fn remap_output(
    c: &mut KotlinOutput,
    local: &str,
    result: &str,
    offset: usize,
    count: usize,
) {
    for text in
        c.delivered
            .iter_mut()
            .chain([&mut c.output_locals, &mut c.materialize, &mut c.cleanup])
    {
        for i in 0..count {
            *text = text.replace(
                &format!("{local}[{i}]"),
                &format!("{result}[{}]", offset + i),
            );
        }
    }
}
pub(crate) fn output_kotlin(
    nodes: &[JniNode],
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    if nodes.iter().all(|n| matches!(n, JniNode::Wire(_))) {
        return output_leaves(&leaf_wires(nodes), context);
    }
    let mut out = KotlinOutput::default();
    let mut offset = 0;
    for (index, node) in nodes.iter().enumerate() {
        let local = format!("{}_group{index}", context.result);
        let prefix = format!("{}g{index}_", context.output_prefix);
        let ctx = KotlinContext {
            result: &local,
            output_prefix: &prefix,
            ..*context
        };
        let (mut c, count) = match node {
            JniNode::Wire(w) => (output_leaves(&[w], &ctx)?, 1),
            JniNode::Group { metadata, children } => {
                let c = match metadata {
                    JniGroup::Optional(o) => optional::output_kotlin(children.nodes(), o, &ctx)?,
                    JniGroup::Fields(Some(d)) => data::output_kotlin(children.nodes(), d, &ctx)?,
                    JniGroup::Fields(None) | JniGroup::Field(_) => {
                        output_kotlin(children.nodes(), &ctx)?
                    }
                };
                (c, children.len())
            }
        };
        remap_output(&mut c, &local, context.result, offset, count);
        out.delivered.extend(c.delivered);
        out.result_types.extend(c.result_types);
        out.output_locals.push_str(&c.output_locals);
        out.materialize.push_str(&c.materialize);
        out.cleanup.push_str(&c.cleanup);
        offset += count;
    }
    Ok(out)
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
        let JniWire::Handle { condition, .. } = &mut plan.layout[0].kind else {
            panic!()
        };
        *condition = "isEnabled() && count > 3".into();
        JniCodec.finish_input(&mut plan, &()).unwrap();
        let input = JniInput::from(plan);
        // A handle and an ordinary i64 expose identical transport information.
        let scalar = JniWire::scalar(Primitive::I64);
        assert_eq!(
            prebindgen_flat::type_key(&input.wire(0).unwrap().kind.rust_type()),
            prebindgen_flat::type_key(&scalar.rust_type())
        );
        assert_eq!(
            input.wire(0).unwrap().kind.kotlin_type(),
            scalar.kotlin_type()
        );
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
        assert_eq!(output.delivered, ["requireNotNull(wrapped_0)"]);
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
