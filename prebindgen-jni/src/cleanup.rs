//! Compose cleanup from resolved output shapes and emit each helper once.
use super::*;
use crate::kotlin::JniNode;
use prebindgen_flat::WireNode;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(crate) enum Carrier {
    Objects,
    TypedObjects,
    Native,
    NativeJni,
}
impl Carrier {
    fn role(self) -> &'static str {
        match self {
            Self::Objects => "__pb_cleanup_objects",
            Self::TypedObjects => "__pb_cleanup_typed",
            Self::Native => "__pb_cleanup_native",
            Self::NativeJni => "__pb_cleanup_frame",
        }
    }
    fn has_env(self) -> bool {
        !matches!(self, Self::Native)
    }
}

enum Field {
    Handle(Box<syn::Type>),
    Rows(Box<Row>),
    Frame(Box<Row>),
}
struct Row {
    carrier: Carrier,
    layout: Option<Rc<BulkRecord>>,
    fields: Vec<(usize, Field)>,
    parts: Vec<String>,
    label: String,
    constructor: Option<syn::Ident>,
    selection: crate::selected::Selection,
    wires: Vec<JniWire>,
}

fn carrier_child<'a>(
    node: &'a JniNode,
    name: &syn::Ident,
) -> Option<(&'a JniNode, Option<&'a JniSequence>)> {
    match node {
        WireNode::Chain {
            meta: JniChain::Sequence(sequence),
            controls,
            child,
        } if controls.iter().any(|w| &w.name == name) => Some((child, Some(sequence))),
        WireNode::Chain {
            meta: JniChain::Bulk(_),
            controls,
            child,
        } if controls.iter().any(|w| &w.name == name) => Some((child, None)),
        WireNode::Chain { child, .. } => carrier_child(child, name),
        WireNode::Branch { children, .. } => {
            children.iter().find_map(|child| carrier_child(child, name))
        }
        _ => None,
    }
}

impl Row {
    fn new(node: &JniNode, carrier: Carrier) -> Result<Self> {
        let layout = if matches!(carrier, Carrier::Objects | Carrier::TypedObjects) {
            None
        } else {
            Some(bulk::record(node)?)
        };
        let mut parts = vec![layout.as_ref().map_or_else(
            || {
                if matches!(carrier, Carrier::TypedObjects) {
                    "TypedObject".into()
                } else {
                    "Object[]".into()
                }
            },
            |r| r.key.clone(),
        )];
        // Field IDs belong to one exact class, even if another physical JVM
        // descriptor shape could acquire a different Kotlin spelling later.
        if matches!(carrier, Carrier::TypedObjects) {
            parts.push(typed_rows::cache(node)?.to_string());
        }
        let mut fields = Vec::new();
        let mut labels = Vec::new();
        for (index, wire) in group::row_fields(node).enumerate() {
            // Preserve the complete physical row shape, including fields that
            // need no destruction. No occurrence-local wire names enter the key.
            parts.push(wire.kind.jvm_descriptor().into());
            let field = match &wire.kind {
                JniWire::Handle { handle, .. } => {
                    if handle.access != Access::Move {
                        return Err("output cleanup requires owned handle allocations".into());
                    }
                    parts.push(format!("owned:{}", names::type_identity(&handle.ty)));
                    labels.push(names::type_label(&handle.ty));
                    Field::Handle(Box::new(handle.ty.clone()))
                }
                JniWire::Rows { .. } | JniWire::NativeRows { .. } => {
                    if matches!(
                        (&wire.kind, carrier),
                        (JniWire::Rows { .. }, Carrier::Native | Carrier::NativeJni)
                            | (
                                JniWire::NativeRows { .. },
                                Carrier::Objects | Carrier::TypedObjects
                            )
                    ) {
                        return Err("collection cleanup carrier differs from its row layout".into());
                    }
                    let (child, sequence) = carrier_child(node, &wire.name)
                        .ok_or("missing collection cleanup element")?;
                    let child_carrier =
                        if matches!(carrier, Carrier::Objects | Carrier::TypedObjects) {
                            if matches!(sequence, Some(JniSequence::TypedBuffered)) {
                                Carrier::TypedObjects
                            } else {
                                Carrier::Objects
                            }
                        } else {
                            carrier
                        };
                    let row = Self::new(child, child_carrier)?;
                    if row.fields.is_empty() {
                        continue;
                    }
                    parts.push(row.name()?.to_string());
                    labels.push("Rows".into());
                    Field::Rows(Box::new(row))
                }
                JniWire::BulkBuffer { .. } => {
                    if !matches!(carrier, Carrier::Objects | Carrier::TypedObjects) {
                        return Err("a JVM bulk frame cannot be a native row field".into());
                    }
                    let (child, _) =
                        carrier_child(node, &wire.name).ok_or("missing frame cleanup layout")?;
                    let row = Self::new(child, Carrier::NativeJni)?;
                    if row.fields.is_empty() {
                        continue;
                    }
                    parts.push(row.name()?.to_string());
                    labels.push("Frame".into());
                    Field::Frame(Box::new(row))
                }
                JniWire::Scalar { .. }
                | JniWire::Enum { .. }
                | JniWire::Selector { .. }
                | JniWire::Text { .. }
                | JniWire::Bytes { .. }
                | JniWire::NativeBuffer { .. }
                | JniWire::Native128 { .. } => continue,
                JniWire::Callback { .. } => {
                    return Err("callback objects cannot own output row resources".into())
                }
            };
            let selection = crate::selected::Selection::new(node)?;
            if !selection.tests[index].is_empty() {
                parts.push(format!("selected:{:?}", selection.tests[index]));
            }
            fields.push((index, field));
        }
        Ok(Self {
            carrier,
            layout,
            selection: crate::selected::Selection::new(node)?,
            wires: group::row_fields(node).map(|w| w.kind.clone()).collect(),
            fields,
            parts,
            label: labels.into_iter().take(2).collect::<Vec<_>>().join("_"),
            constructor: if matches!(carrier, Carrier::TypedObjects) {
                Some(typed_rows::cache(node)?)
            } else {
                None
            },
        })
    }

    fn claim(&self, names: &mut names::Names) -> Result<syn::Ident> {
        let parts = self.parts.iter().map(String::as_str).collect::<Vec<_>>();
        Ok(format_ident!(
            "{}",
            names.name(self.carrier.role(), &self.label, &parts)?
        ))
    }
    fn name(&self) -> Result<syn::Ident> {
        self.claim(&mut names::Names::default())
    }
}

/// A stable reference for preparation; definitions are interned during emission.
pub(crate) fn helper(node: &JniNode, carrier: Carrier) -> Result<Option<syn::Ident>> {
    let row = Row::new(node, carrier)?;
    if row.fields.is_empty() {
        Ok(None)
    } else {
        row.name().map(Some)
    }
}

#[derive(Default)]
struct Helpers {
    names: names::Names,
    definitions: BTreeMap<String, TokenStream>,
}
impl Helpers {
    fn intern(&mut self, row: &Row) -> Result<syn::Ident> {
        // Intern children first, including for existing parents: a child hash
        // collision must never be hidden behind an identical parent name.
        for (_, field) in &row.fields {
            if let Field::Rows(child) | Field::Frame(child) = field {
                self.intern(child)?;
            }
        }
        let name = row.claim(&mut self.names)?;
        if self.definitions.contains_key(&name.to_string()) {
            return Ok(name);
        }
        let mut body = TokenStream::new();
        for (index, field) in &row.fields {
            let environment =
                if matches!((row.carrier, field), (Carrier::NativeJni, Field::Handle(_))) {
                    quote!(_env)
                } else {
                    quote!(env)
                };
            let at = row.layout.as_ref().map(|layout| {
                let offset = &layout.offsets[*index];
                quote!(base+(#offset))
            });
            let typed_object = row.constructor.as_ref().map(|cache| quote! {
                env.get_field_unchecked(row,#cache.get()?.field(#index),::prebindgen_jni_runtime::jni::signature::ReturnType::Object).and_then(|v|v.l()).map_err(|e|e.to_string())?
            });
            let release = match field {
                Field::Handle(ty) => {
                    let read = if let Some(at) = at {
                        quote!(reader.read::<i64>(#at)?)
                    } else if let Some(cache) = &row.constructor {
                        quote!(env.get_field_unchecked(row,#cache.get()?.field(#index),::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(::prebindgen_jni_runtime::jni::signature::Primitive::Long)).and_then(|v|v.j()).map_err(|e|e.to_string())?)
                    } else {
                        quote!(unsafe {::prebindgen_jni_runtime::output_handle(env,row,#index)}?)
                    };
                    quote! {
                        let raw = #read;
                        if raw != 0 { let _=::prebindgen_runtime::boundary(|| unsafe {::prebindgen_jni_runtime::drop_pointer::<#ty>(raw)}); }
                    }
                }
                Field::Rows(child) => {
                    let stride = child.layout.as_ref().map(|r| {
                        let record = &r.name;
                        quote!(::core::mem::size_of::<#record>())
                    });
                    let has_env = child.carrier.has_env();
                    let reclaim = if matches!(child.carrier, Carrier::TypedObjects) {
                        quote!(::prebindgen_jni_runtime::reclaim_objects)
                    } else {
                        quote!(::prebindgen_jni_runtime::reclaim_rows)
                    };
                    let child = child.name()?;
                    if let Some(at) = at {
                        let env = if has_env { quote!(env,) } else { quote!() };
                        quote! {
                            if let Some(range) = reader.range(#at)? {
                                let (start,length) = reader.rows(Some(range),#stride)?;
                                for index in 0..length { unsafe {#child(#env reader,start+index*#stride);} }
                            }
                        }
                    } else {
                        let read=typed_object.clone().unwrap_or_else(||quote!(env.get_object_array_element(row,#index as i32).map_err(|e|e.to_string())?));
                        quote! {
                            let rows=#read;
                            let rows=env.auto_local(::prebindgen_jni_runtime::jni::objects::JObjectArray::from(rows));
                            if !rows.is_null() {
                                let length=env.get_array_length(&*rows).map_err(|e|e.to_string())? as usize;
                                unsafe {#reclaim(env,rows.as_raw(),length,#child);}
                            }
                        }
                    }
                }
                Field::Frame(child) => {
                    let child = child.name()?;
                    let read=typed_object.unwrap_or_else(||quote!(env.get_object_array_element(row,#index as i32).map_err(|e|e.to_string())?));
                    quote! {
                        let bytes=#read;
                        let bytes=env.auto_local(::prebindgen_jni_runtime::jni::objects::JByteArray::from(bytes));
                        if !bytes.is_null() {
                            let bytes=env.convert_byte_array(&*bytes).map_err(|e|e.to_string())?;
                            let reader=::prebindgen_jni_runtime::bulk::Reader::new(&bytes);
                            unsafe {#child(env,&reader,0usize);}
                        }
                    }
                }
            };
            let mut reads = BTreeMap::new();
            for test in &row.selection.tests[*index] {
                let i = test.index();
                let kind = &row.wires[i];
                let read = if let Some(layout) = &row.layout {
                    let wire = prebindgen_flat::Wire {
                        name: syn::parse_quote!(control),
                        kind: kind.clone(),
                    };
                    bulk::native_read(&wire, layout, i, quote!(base), quote!(reader))?
                } else {
                    let read = if let Some(cache) = &row.constructor {
                        let result = kind.callback_return_type();
                        let member = kind.value_member();
                        quote!(env.get_field_unchecked(row,#cache.get()?.field(#i),#result).and_then(|v|v.#member()).map_err(|e|e.to_string())?)
                    } else {
                        let read = kind.read_array_element(&quote!(row), i);
                        let read = prebindgen_conversions::render_template(
                            &read,
                            &|name| {
                                if name == "environment" {
                                    Ok(quote!(*env))
                                } else {
                                    Err(format!("unexpected wire-reader parameter `{name}`"))
                                }
                            },
                            &|_| Err("wire reads cannot transfer owners".into()),
                        )?;
                        quote!(#read?)
                    };
                    if kind.is_reference() {
                        quote!({let discriminator=env.auto_local(#read);discriminator.as_raw()})
                    } else {
                        read
                    }
                };
                reads.insert(i, read);
            }
            let predicate = row.selection.rust(*index, |i| reads[&i].clone());
            let release = if row.selection.tests[*index].is_empty() {
                release
            } else {
                quote!(if #predicate {#release})
            };
            body.extend(match row.carrier {
                Carrier::Objects | Carrier::TypedObjects => quote!(::prebindgen_jni_runtime::reclaim_output(env, |env| {#release Ok(())});),
                Carrier::Native => quote!(let _=::prebindgen_runtime::boundary(|| {#release Ok(())});),
                Carrier::NativeJni => quote!(::prebindgen_jni_runtime::reclaim_output(env, |#environment| {let _=::prebindgen_runtime::boundary(|| {#release Ok(())}); Ok(())});),
            });
        }
        let definition = match row.carrier {
            Carrier::Objects => quote! {
                #[allow(dead_code,non_snake_case)]
                fn #name<'local>(env:&mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,row:&::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>) {#body}
            },
            Carrier::TypedObjects => quote! {
                #[allow(dead_code,non_snake_case)]
                fn #name<'local>(env:&mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,row:&::prebindgen_jni_runtime::jni::objects::JObject<'local>) {#body}
            },
            Carrier::Native => quote! {
                #[allow(dead_code,non_snake_case)]
                unsafe fn #name(reader:&::prebindgen_jni_runtime::bulk::Reader<'_>,base:usize) {#body}
            },
            Carrier::NativeJni => quote! {
                #[allow(dead_code,non_snake_case)]
                unsafe fn #name(env:&mut ::prebindgen_jni_runtime::jni::JNIEnv<'_>,reader:&::prebindgen_jni_runtime::bulk::Reader<'_>,base:usize) {#body}
            },
        };
        self.definitions.insert(name.to_string(), definition);
        Ok(name)
    }

    fn visit(&mut self, node: &JniNode, input: bool) -> Result<()> {
        match node {
            WireNode::Chain {
                meta: JniChain::Sequence(sequence),
                child,
                ..
            } => {
                if !input {
                    let carrier = match sequence {
                        JniSequence::Buffered => Some(Carrier::Objects),
                        JniSequence::TypedBuffered => Some(Carrier::TypedObjects),
                        JniSequence::Bulk => Some(Carrier::Native),
                        JniSequence::Callback => None,
                    };
                    if let Some(carrier) = carrier {
                        let row = Row::new(child, carrier)?;
                        if !row.fields.is_empty() {
                            self.intern(&row)?;
                        }
                    }
                }
                self.visit(child, input)?;
            }
            WireNode::Chain { child, .. } => self.visit(child, input)?,
            WireNode::Branch { children, .. } => {
                for child in children {
                    self.visit(child, input)?;
                }
            }
            WireNode::Leaf(wires) => {
                for wire in wires {
                    if let JniWire::Callback { callback } = &wire.kind {
                        for (input, node) in callback.bulk_nodes() {
                            self.visit(node, input)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn emit<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
) -> Result<TokenStream> {
    let mut helpers = Helpers::default();
    for (input, node) in metadata::nodes(flat) {
        helpers.visit(node, input)?;
    }
    Ok(helpers.definitions.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner(name: &str) -> JniNode {
        WireNode::Leaf(vec![prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::handle(
                BoxedWire {
                    ty: syn::parse_quote!(String),
                    access: Access::Move,
                },
                "true",
            ),
        }])
    }

    #[test]
    fn helper_identity_ignores_occurrence_names_and_separates_access_modes() {
        let mut helpers = Helpers::default();
        let mut names = BTreeSet::new();
        for carrier in [Carrier::Objects, Carrier::Native, Carrier::NativeJni] {
            let first = Row::new(&owner("first"), carrier).unwrap();
            let second = Row::new(&owner("second"), carrier).unwrap();
            let one = helpers.intern(&first).unwrap();
            let two = helpers.intern(&second).unwrap();
            assert_eq!(one, two);
            names.insert(one.to_string());
        }
        assert_eq!(names.len(), 3);
        assert_eq!(helpers.definitions.len(), 3);
    }

    #[test]
    fn helper_names_reject_collisions_instead_of_reusing_wrong_cleanup() {
        let row = Row::new(&owner("value"), Carrier::Objects).unwrap();
        let name = row.name().unwrap();
        let mut helpers = Helpers::default();
        helpers
            .names
            .reserve(&name.to_string(), "unrelated", &["other shape"])
            .unwrap();
        assert!(helpers.intern(&row).unwrap_err().contains("hash collision"));
    }
}
