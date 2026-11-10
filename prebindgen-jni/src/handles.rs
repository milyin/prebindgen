//! Prepare handle inputs once for alias validation and globally ordered locking.
use super::*;
use crate::kotlin::{JniNode, KotlinHandleInput};
use prebindgen_flat::WireNode;

/// Stable across declaration order and unrelated additions. Collisions are
/// diagnosed by generate rather than weakening resource separation at runtime.
pub(crate) fn allocation_type_id(ty: &syn::Type) -> i64 {
    prebindgen_reprc::schema_id(&format!("jni-allocation:{}", names::type_identity(ty))) as i64
}
pub(crate) fn access_key(access: Access) -> u8 {
    match access {
        Access::Shared => 0,
        Access::Exclusive => 1,
        Access::Move => 2,
    }
}
pub(crate) fn key(handle: &BoxedWire) -> (String, u8) {
    (type_key(&handle.ty), access_key(handle.access))
}
/// Local groups in one prepared carrier. The same order is used by its writer,
/// patch walk, and the enclosing Kotlin call; no runtime type lookup is needed.
pub(crate) fn kinds(node: &JniNode) -> Vec<BoxedWire> {
    let mut kinds = BTreeMap::new();
    for wire in prebindgen_flat::wire_definitions(std::iter::once(node)) {
        if let JniWire::Handle { handle, .. } = &wire.kind {
            kinds
                .entry(key(handle))
                .or_insert_with(|| (**handle).clone());
        }
    }
    kinds.into_values().collect()
}
pub(crate) fn carrier_inputs(node: &JniNode, state: &str) -> Vec<KotlinHandleInput> {
    let kinds = kinds(node);
    let count = kinds.len();
    kinds
        .into_iter()
        .enumerate()
        .map(|(group, handle)| KotlinHandleInput {
            ty: handle.ty,
            access: handle.access,
            many: true,
            argument: None,
            consumed: None,
            value: if count == 1 {
                format!("{state}?.handles.orEmpty()")
            } else {
                format!("{state}?.handleGroup({group}).orEmpty()")
            },
        })
        .collect()
}
fn controlled_child<'a>(node: &'a JniNode, name: &syn::Ident) -> Option<&'a JniNode> {
    match node {
        WireNode::Chain {
            controls, child, ..
        } => {
            if controls.iter().any(|w| w.name == *name) {
                Some(child)
            } else if matches!(
                node,
                WireNode::Chain {
                    meta: JniChain::Sequence(_) | JniChain::Bulk(_),
                    ..
                }
            ) {
                None // Child rows are a different name scope.
            } else {
                controlled_child(child, name)
            }
        }
        WireNode::Branch { children, .. } => {
            children.iter().find_map(|n| controlled_child(n, name))
        }
        _ => None,
    }
}
pub(crate) fn column_groups(node: &JniNode) -> Result<String> {
    let groups = kinds(node);
    if groups.len() <= 1 {
        return Ok("null".into());
    }
    let columns = crate::group::row_fields(node)
        .map(|wire| {
            let local = if let JniWire::Handle { handle, .. } = &wire.kind {
                vec![(**handle).clone()]
            } else if matches!(wire.kind, JniWire::Rows { .. } | JniWire::BulkBuffer { .. }) {
                let child = controlled_child(node, &wire.name).ok_or_else(|| {
                    format!(
                        "missing prepared child for handle-group column `{}`",
                        wire.name
                    )
                })?;
                kinds(child)
            } else {
                vec![]
            };
            let indices = local
                .iter()
                .map(|kind| {
                    groups
                        .iter()
                        .position(|g| key(g) == key(kind))
                        .map(|index| index.to_string())
                        .ok_or_else(|| {
                            format!("missing parent handle group for `{}`", type_key(&kind.ty))
                        })
                })
                .collect::<Result<Vec<_>>>()?
                .join(", ");
            Ok(format!("intArrayOf({indices})"))
        })
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    Ok(format!("arrayOf({columns})"))
}
/// Every native resource must be represented by prepared Kotlin handle wires.
/// Custom conversions without this information cannot use the generated JNI API.
pub(crate) fn require_preflight<I: KotlinInputConversion>(
    input: &I,
    resources: &[prebindgen_conversions::Resource],
) -> Result<()> {
    fn prepared(node: &JniNode) -> bool {
        match node {
            WireNode::Chain {
                meta: JniChain::Bulk(_),
                ..
            } => true,
            WireNode::Chain {
                meta: JniChain::Sequence(JniSequence::Bulk),
                ..
            } => !crate::kotlin::input_handles(node, false),
            WireNode::Chain { child, .. } => prepared(child),
            WireNode::Branch { children, .. } => children.iter().all(prepared),
            WireNode::Leaf(_) => true,
        }
    }
    if !prepared(input.input_node()) {
        return Err("Kotlin handle preflight requires an enclosing prepared bulk frame".into());
    }
    let mut native = BTreeMap::new();
    let mut kotlin = BTreeMap::new();
    for resource in resources {
        let Some(ty) = &resource.allocation_type else {
            return Err(
                "Kotlin handle preflight requires the allocation type of every native resource"
                    .into(),
            );
        };
        if !resource.registered {
            return Err(
                "Kotlin handle preflight requires eagerly prepared native resources".into(),
            );
        }
        *native
            .entry((type_key(ty), access_key(resource.access)))
            .or_insert(0usize) += 1;
    }
    for wire in prebindgen_flat::wire_definitions(std::iter::once(input.input_node())) {
        if let JniWire::Handle { handle, .. } = &wire.kind {
            *kotlin.entry(key(handle)).or_insert(0usize) += 1;
        }
    }
    if native != kotlin {
        return Err("Kotlin handle preflight requires matching allocation types and access modes for native resources and Kotlin handle wires".into());
    }
    Ok(())
}

/// Check the Kotlin emitter's retained sources against the wire shape. Carriers
/// combine repeated fields into one list per allocation type and access mode.
fn complete_handle_inputs(node: &JniNode, input: &mut KotlinInput) -> Result<()> {
    fn sources(node: &JniNode, expected: &mut BTreeMap<(String, u8, bool), usize>) {
        match node {
            WireNode::Chain {
                meta: JniChain::Bulk(_) | JniChain::Sequence(_),
                child,
                ..
            } => {
                for kind in kinds(child) {
                    let (ty, access) = key(&kind);
                    *expected.entry((ty, access, true)).or_default() += 1;
                }
            }
            WireNode::Chain { child, .. } => sources(child, expected),
            WireNode::Branch { children, .. } => {
                for child in children {
                    sources(child, expected);
                }
            }
            WireNode::Leaf(wires) => {
                for wire in wires {
                    if let JniWire::Handle { handle, .. } = &wire.kind {
                        let (ty, access) = key(handle);
                        *expected.entry((ty, access, false)).or_default() += 1;
                    }
                }
            }
        }
    }
    let mut expected = BTreeMap::new();
    sources(node, &mut expected);
    if input.handle_inputs.is_none()
        && expected.is_empty()
        && input.locks.is_empty()
        && input.lock_groups.is_empty()
    {
        input.handle_inputs = Some(vec![]);
    }
    let handles = input.handle_inputs.as_ref().ok_or(
        "Kotlin handle preflight requires complete prepared handle inputs from custom conversions",
    )?;
    let mut actual = BTreeMap::new();
    for handle in handles {
        *actual
            .entry((type_key(&handle.ty), access_key(handle.access), handle.many))
            .or_default() += 1usize;
    }
    if actual != expected {
        return Err("Kotlin handle preflight requires retained inputs matching every handle wire's allocation type, access mode and carrier grouping".into());
    }
    if handles.is_empty() && (!input.locks.is_empty() || !input.lock_groups.is_empty()) {
        return Err("prepared handle inputs omit existing Kotlin locks".into());
    }
    Ok(())
}

struct Source {
    local: String,
    many: bool,
    access: Access,
    ty: i64,
}
/// Private emission state for a complete ordinary call or callback result.
/// Ownership receipts still walk their original wire order, independently of
/// these groups, which exist only to validate inputs and acquire locks.
pub(crate) struct HandleLocks {
    sources: Vec<Source>,
    pub(crate) setup: String,
    gc: bool,
    prefix: String,
}
impl HandleLocks {
    pub(crate) fn new<'a, I: KotlinInputConversion + 'a>(
        inputs: impl IntoIterator<Item = (&'a I, &'a mut KotlinInput)>,
        context: &KotlinContext<'_>,
    ) -> Result<Self> {
        let mut out = Self {
            sources: vec![],
            setup: String::new(),
            gc: false,
            prefix: context.package.into(),
        };
        for (native, input) in inputs {
            complete_handle_inputs(native.input_node(), input)?;
            for handle in input.handle_inputs.clone().unwrap() {
                let class = context
                    .classes
                    .get(&type_key(&handle.ty))
                    .ok_or("missing handle input class")?;
                let local = format!("__pb_handle{}", out.sources.len());
                let value = &handle.value;
                let ty = if handle.many {
                    format!("kotlin.collections.List<{}.NativeHandle>", context.package)
                } else {
                    format!("{}.NativeHandle?", context.package)
                };
                out.setup.push_str(&format!("val {local}: {ty} = try {{ {value} }} catch (failure: IllegalArgumentException) {{ throw {}.HandleValidationFailure(failure.message ?: \"invalid handle\") }}; ", context.package));
                if handle.many == handle.argument.is_some() {
                    return Err(
                        "retained handles require individual argument indices or carrier lists"
                            .into(),
                    );
                }
                if let Some(index) = handle.argument {
                    let argument = input
                        .arguments
                        .get_mut(index)
                        .ok_or("invalid retained-handle argument index")?;
                    *argument = format!("{local}?.raw ?: 0L");
                }
                if let Some(original) = &handle.consumed {
                    let consume = input
                        .consumed
                        .iter_mut()
                        .find(|code| *code == original)
                        .ok_or("retained handle ownership update does not match its input")?;
                    *consume = format!("{local}?.let {{ __pb_consume(it) }}");
                }
                out.sources.push(Source {
                    local,
                    many: handle.many,
                    access: handle.access,
                    ty: class.handle_type,
                });
                out.gc |= class.gc_managed;
            }
        }
        Ok(out)
    }
    pub(crate) fn wrap(&self, body: &str) -> String {
        let p = &self.prefix;
        let gc = if self.gc { "Gc" } else { "" };
        let mut body = body.to_owned();
        if self.sources.len() <= 3 && self.sources.iter().all(|s| !s.many) {
            let mut checks = String::new();
            for (i, a) in self.sources.iter().enumerate() {
                for b in self.sources.iter().skip(i + 1) {
                    if a.ty == b.ty && (a.access != Access::Shared || b.access != Access::Shared) {
                        let (a, b) = (&a.local, &b.local);
                        checks.push_str(&format!("if ({a} != null && {b} != null && {a}.address == {b}.address) throw {p}.HandleValidationFailure(\"conflicting input aliases\"); "));
                    }
                }
            }
            if !self.sources.is_empty() {
                body = format!(
                    "{checks}{p}.with{gc}HandleLocks({}) {{ {body} }}",
                    self.sources
                        .iter()
                        .map(|s| s.local.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        } else {
            let types = self.sources.iter().map(|s| s.ty).collect::<BTreeSet<_>>();
            let mut prepare = String::new();
            let mut ordered = vec![];
            for (group_index, ty) in types.into_iter().enumerate() {
                let shared = self.list(ty, true);
                let exclusive = self.list(ty, false);
                let shared_present = self
                    .sources
                    .iter()
                    .any(|s| s.ty == ty && s.access == Access::Shared);
                let exclusive_present = self
                    .sources
                    .iter()
                    .any(|s| s.ty == ty && s.access != Access::Shared);
                let name = format!("__pb_ordered{group_index}");
                if ordered.is_empty() && self.sources.iter().all(|s| s.ty == ty) {
                    // Single-type calls retain their short-list fast paths and
                    // need neither a group array nor a second-dimensional walk.
                    let helper = if shared_present && exclusive_present {
                        format!("with{gc}MixedInputLocks({shared}, {exclusive})")
                    } else {
                        format!(
                            "with{gc}InputHandleLocks({}, {})",
                            if shared_present { shared } else { exclusive },
                            exclusive_present
                        )
                    };
                    body = format!("{p}.{helper} {{ {body} }}");
                    ordered.clear();
                    break;
                }
                let expr = if shared_present && exclusive_present {
                    format!("{p}.prepareMixedInputHandles({shared}, {exclusive})")
                } else {
                    format!(
                        "{p}.prepareInputHandles({}, {})",
                        if shared_present { shared } else { exclusive },
                        exclusive_present
                    )
                };
                prepare.push_str(&format!("val {name} = {expr}; "));
                ordered.push(name);
            }
            for name in ordered.iter().rev() {
                body = format!("{p}.with{gc}OrderedHandleLocks({name}) {{ {body} }}");
            }
            body = format!("{prepare}{body}");
        }
        format!("run {{ {}{body} }}", self.setup)
    }
    fn list(&self, ty: i64, shared: bool) -> String {
        let sources = self
            .sources
            .iter()
            .filter(|s| s.ty == ty && (s.access == Access::Shared) == shared)
            .collect::<Vec<_>>();
        crate::kotlin::lock_list(
            &sources
                .iter()
                .filter(|s| !s.many)
                .map(|s| s.local.clone())
                .collect::<Vec<_>>(),
            &sources
                .iter()
                .filter(|s| s.many)
                .map(|s| s.local.clone())
                .collect::<Vec<_>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prebindgen_flat::Wire;

    fn handle(name: &str, ty: syn::Type) -> Wire<JniWire> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::handle(
                BoxedWire {
                    ty,
                    access: Access::Move,
                },
                "true",
            ),
        }
    }
    fn rows() -> Wire<JniWire> {
        Wire {
            name: syn::parse_quote!(rows),
            kind: JniWire::Rows {
                condition: "true".into(),
                null_niche_available: true,
            },
        }
    }
    #[test]
    fn nested_column_groups_cover_every_child_allocation_class() {
        let node = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Values),
            children: vec![
                WireNode::Leaf(vec![handle("first", syn::parse_quote!(A))]),
                WireNode::Chain {
                    controls: vec![rows()],
                    meta: JniChain::Sequence(JniSequence::Buffered),
                    child: Box::new(WireNode::Leaf(vec![
                        handle("a", syn::parse_quote!(A)),
                        handle("b", syn::parse_quote!(B)),
                    ])),
                },
            ],
        };
        assert_eq!(
            column_groups(&node).unwrap(),
            "arrayOf(intArrayOf(0), intArrayOf(0, 1))"
        );
    }
    #[test]
    fn missing_nested_column_metadata_is_a_generation_error() {
        for column in [
            rows(),
            Wire {
                name: syn::parse_quote!(buffer),
                kind: JniWire::BulkBuffer {
                    condition: "true".into(),
                    null_niche_available: true,
                },
            },
        ] {
            let node = WireNode::Leaf(vec![
                handle("a", syn::parse_quote!(A)),
                handle("b", syn::parse_quote!(B)),
                column,
            ]);
            assert!(column_groups(&node)
                .unwrap_err()
                .contains("missing prepared child"));
        }
    }
}

#[cfg(test)]
mod preflight_tests {
    use super::*;
    use prebindgen_flat::Wire;

    #[test]
    fn retained_handle_metadata_must_cover_the_wires() {
        let handle = BoxedWire {
            ty: syn::parse_quote!(Handle),
            access: Access::Move,
        };
        let node = WireNode::Leaf(vec![Wire {
            name: format_ident!("wire"),
            kind: JniWire::handle(handle, "true"),
        }]);
        for wrong in ["missing", "access", "type", "group"] {
            let mut input = KotlinInput {
                handle_inputs: Some(vec![]),
                ..Default::default()
            };
            if wrong != "missing" {
                input
                    .handle_inputs
                    .as_mut()
                    .unwrap()
                    .push(KotlinHandleInput {
                        ty: if wrong == "type" {
                            syn::parse_quote!(Other)
                        } else {
                            syn::parse_quote!(Handle)
                        },
                        access: if wrong == "access" {
                            Access::Shared
                        } else {
                            Access::Move
                        },
                        many: wrong == "group",
                        value: "handle".into(),
                        argument: Some(0),
                        consumed: None,
                    });
            }
            assert!(complete_handle_inputs(&node, &mut input)
                .unwrap_err()
                .contains("matching every handle wire"));
        }
    }

    #[test]
    fn resource_free_custom_inputs_need_no_handle_metadata() {
        let mut input = KotlinInput::default();
        complete_handle_inputs(&WireNode::Leaf(vec![]), &mut input).unwrap();
        assert!(input.handle_inputs.unwrap().is_empty());
    }
}
