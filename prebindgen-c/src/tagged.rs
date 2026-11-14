//! Converted C tagged values and their ownership-preserving input slots.
use super::*;
use prebindgen_declarations::Condition;
use prebindgen_declarations::{Direction, EnumEncoding, EnumForm, EnumVariants, InputRole};
use prebindgen_flat::type_key;
use prebindgen_flat::{Wire as SourceWire, WireNode};
use std::collections::BTreeMap;
use std::rc::Rc;

/// Ordinary C enum representation shared by its input and output occurrences.
#[derive(Clone, Debug)]
pub struct CTagged {
    /// Unprefixed public C name.
    pub name: String,
    /// Generated target-side carrier identifier.
    pub rust: syn::Ident,
    /// Whether active payload fields may own resources.
    pub owned: bool,
    /// Target-library destructor symbol.
    pub destructor: syn::Ident,
}
/// Source identity of a tag and its ordered alternatives.
#[derive(Clone, Debug)]
pub struct CEnumGroup {
    /// Source type.
    pub ty: syn::Type,
    /// Source variant names in wire-tag order.
    pub variants: Vec<String>,
}
fn is_owner(w: &CWire) -> bool {
    matches!(w, CWire::Buffer)
        || matches!(w,CWire::Pointer{pointer,..} if pointer.access==Access::Move)
        || matches!(
            w,
            CWire::Owned {
                access: Access::Move,
                ..
            }
        )
        || matches!(w,CWire::Tagged{value,..} if value.owned)
}

pub(crate) fn encoding(request: EnumVariants<'_, CCodec>) -> Result<EnumEncoding<CCodec>> {
    let source = request.flat.payload_enum(request.ty)?;
    if request.form == EnumForm::Value
        && request
            .flat
            .type_info(request.ty)
            .ok()
            .and_then(|t| t.metadata.tagged.as_ref())
            .is_none()
    {
        return Err(format!(
            "ordinary payload enum `{}` requires tagged_union",
            type_key(request.ty)
        ));
    }
    for arm in request.arms.iter_mut() {
        for wire in group::values_mut(arm) {
            wire.kind.clear_niche();
        }
    }
    let inactive = if request.direction == Direction::Output {
        request
            .arms
            .iter()
            .map(|arm| {
                group::values(arm)
                    .map(|wire| {
                        let absent = match &wire.kind {
                            CWire::Owned { owner, by_value: true, .. } => {
                                // This is uninitialized *carrier storage*, not
                                // an uninitialized Rust owner. Inactive slots
                                // must never create, read or drop a native T.
                                let name = owner.ident();
                                quote!(unsafe { ::core::mem::MaybeUninit::<#name>::uninit().assume_init() })
                            }
                            _ => crate::wire::absent_output_value(&wire.kind, None)?,
                        };
                        Ok((wire.name.clone(), absent))
                    })
                    .collect()
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };
    Ok(EnumEncoding {
        metadata: CBranch::Enum(Rc::new(CEnumGroup {
            ty: request.ty.clone(),
            variants: source.variants.iter().map(|v| v.name.to_string()).collect(),
        })),
        inactive,
    })
}
fn storage_type(w: &CWire) -> syn::Type {
    match w {
        CWire::Owned {
            owner,
            access: Access::Move,
            ..
        } => {
            let n = owner.ident();
            syn::parse_quote!(#n)
        }
        CWire::Tagged { value, .. } => {
            let n = &value.rust;
            syn::parse_quote!(#n)
        }
        _ => prebindgen_flat::Wire {
            name: syn::parse_quote!(unused),
            kind: w.clone(),
        }
        .rust_type(),
    }
}
fn c_member(name: &str) -> Result<syn::Ident> {
    const KEYWORDS:&str="auto break case char const continue default do double else enum extern float for goto if inline int long register restrict return short signed sizeof static struct switch typedef union unsigned void volatile while _Alignas _Alignof _Atomic _Bool _Complex _Generic _Imaginary _Noreturn _Static_assert _Thread_local typeof typeof_unqual _BitInt _Decimal32 _Decimal64 _Decimal128 bool true false NULL alignas alignof constexpr nullptr static_assert thread_local alignas alignof and and_eq asm bitand bitor catch char8_t char16_t char32_t class compl concept const_cast consteval constinit co_await co_return co_yield decltype delete dynamic_cast explicit export false friend mutable namespace new noexcept not not_eq operator or or_eq private protected public reinterpret_cast requires static_cast template this throw true try typeid typename using virtual wchar_t xor xor_eq";
    if !name.is_ascii() || KEYWORDS.split_whitespace().any(|w| w == name) {
        return Err(format!(
            "enum member `{name}` is not a supported C identifier"
        ));
    }
    if let Ok(ident) = syn::parse_str(name) {
        Ok(ident)
    } else {
        Ok(syn::Ident::new_raw(name, proc_macro2::Span::call_site()))
    }
}
fn field_name(property: &syn::Ident, index: usize, count: usize) -> Result<syn::Ident> {
    let property = property.to_string();
    let property = property.trim_start_matches("r#");
    c_member(&if count == 1 {
        property.to_owned()
    } else {
        format!("{property}_v{index}")
    })
}
#[allow(clippy::type_complexity)]
fn arm_fields(node: &WireNode<CWire, CChain, CBranch>) -> Result<Vec<(&syn::Ident, Vec<&Wire>)>> {
    let WireNode::Branch { children, .. } = node else {
        return Err("enum payload requires field groups".into());
    };
    children
        .iter()
        .map(|node| {
            let WireNode::Branch {
                meta: CBranch::Group(CGroup::Field(name)),
                ..
            } = node
            else {
                return Err("enum payload requires named source fields".into());
            };
            Ok((name, group::values(node).collect()))
        })
        .collect()
}
pub(crate) fn shape<S: prebindgen_flat::Schema<Module = CModule, Type = CType>>(
    flat: &Flat<S>,
    ty: &syn::Type,
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Result<(Rc<CTagged>, prebindgen_reprc::TaggedUnion)> {
    let name = flat
        .type_info(ty)?
        .metadata
        .tagged
        .clone()
        .ok_or("missing C tagged declaration")?;
    let rust = format_ident!("__PbEnum_{}_{}", flat.metadata.prefix, name);
    let WireNode::Branch { children, .. } = node else {
        return Err("expected source enum group".into());
    };
    let mut arms = vec![];
    let mut owned = false;
    for (a, arm) in children.iter().enumerate() {
        let mut fields = vec![];
        for (field, wires) in arm_fields(arm)? {
            for (i, w) in wires.iter().enumerate() {
                owned |= is_owner(&w.kind);
                fields.push((field_name(field, i, wires.len())?, storage_type(&w.kind)));
            }
        }
        arms.push(if fields.is_empty() {
            None
        } else {
            Some(prebindgen_reprc::Record::new(
                format_ident!("{rust}_a{a}"),
                fields,
            )?)
        });
    }
    Ok((
        Rc::new(CTagged {
            destructor: format_ident!("{}_drop_{}", flat.metadata.prefix, name),
            name,
            rust: rust.clone(),
            owned,
        }),
        prebindgen_reprc::TaggedUnion::new(rust, arms)?,
    ))
}

fn stored_wires(node: &WireNode<CWire, CChain, CBranch>) -> Vec<Wire> {
    group::values(node)
        .map(|w| {
            let mut w = w.clone();
            match &mut w.kind {
                CWire::Owned {
                    access: Access::Move,
                    by_value,
                    ..
                }
                | CWire::Tagged { by_value, .. } => *by_value = true,
                _ => {}
            }
            w
        })
        .collect()
}
fn slots(
    node: &WireNode<CWire, CChain, CBranch>,
    layout: &prebindgen_reprc::TaggedUnion,
    base: TokenStream,
) -> Result<BTreeMap<String, TokenStream>> {
    let WireNode::Branch { children, .. } = node else {
        return Err("expected enum payload".into());
    };
    let mut slots = BTreeMap::new();
    for (a, arm) in children.iter().enumerate() {
        for (field, wires) in arm_fields(arm)? {
            for (i, w) in wires.iter().enumerate() {
                let field = field_name(field, i, wires.len())?;
                let offset = layout.field_offset(a, &field)?;
                let storage = storage_type(&w.kind);
                slots.insert(
                    w.name.to_string(),
                    quote!((#base).cast::<u8>().wrapping_add(#offset).cast::<#storage>()),
                );
            }
        }
    }
    Ok(slots)
}

pub(crate) fn input(
    flat: &CBindings,
    ty: &syn::Type,
    wire: syn::Ident,
    mut plan: CInputPlan<CWire>,
    active: &Condition,
    role: InputRole,
) -> Result<CInputPlan<CWire>> {
    let (value, layout) = shape(flat, ty, &plan.node)?;
    let carrier = format_ident!("{wire}_carrier");
    let by_value = role == InputRole::CallbackResult || !value.owned;
    let pointer = if by_value {
        quote!(::core::ptr::addr_of_mut!(#wire))
    } else {
        quote!(#wire)
    };
    let WireNode::Branch { controls, .. } = &plan.node else {
        return Err("expected enum input".into());
    };
    let tag = &controls[0].name;
    let condition = active.rust();
    let mut bind = quote!(#[allow(unused_parens)] let #carrier=#pointer;let #tag=if #condition {unsafe{(*#carrier).tag}}else{0};);
    let slots = slots(&plan.node, &layout, quote!(#carrier))?;
    for (w, tests) in crate::selected::values(&plan.node)? {
        let name = &w.name;
        if name == tag {
            continue;
        }
        let slot = &slots[&name.to_string()];
        let selected = crate::selected::condition(&tests, |n| quote!(#n));
        let selected = active.and(selected);
        let expression = match &w.kind {
            CWire::Owned {
                access: Access::Move,
                by_value: false,
                ..
            }
            | CWire::Tagged {
                by_value: false, ..
            } => slot.clone(),
            _ => quote!(unsafe{::core::ptr::read(#slot)}),
        };
        if matches!(&w.kind,CWire::Pointer{pointer,..} if pointer.access==Access::Move) {
            let saved = format_ident!("{name}_enum_slot");
            bind.extend(quote!(let #saved=if #selected {#slot}else{::core::ptr::null_mut()};));
        }
        let absent = crate::wire::absent_output_value(&w.kind, None)?;
        bind.extend(quote!(let #name=if #selected {#expression}else{#absent};));
    }
    if value.owned {
        // Claim the whole physical carrier once. The cleanup guard releases
        // untouched active members on failure; acquired Rust values drop normally.
        plan.code = prebindgen_conversions::render_template(
            &plan.code,
            &|name| Ok(prebindgen_conversions::template_parameter(name)),
            &|_| Ok(TokenStream::new()),
        )?;
        let transfer = prebindgen_conversions::record_input_transfer(quote!(1usize));
        let guard = format_ident!("{wire}_remaining");
        let owned = stored_wires(&plan.node);
        let fields = owned
            .iter()
            .filter_map(|w| {
                slots
                    .get(&w.name.to_string())
                    .map(|slot| (w, quote!(::core::ptr::read(#slot))))
            })
            .collect::<Vec<_>>();
        let original = group::values(&plan.node)
            .map(|w| {
                let n = &w.name;
                (w, quote!(#n))
            })
            .collect::<Vec<_>>();
        let cleanup =
            crate::sequence::release_fields(&plan.node, &fields, &original, &|_| quote!(()), true);
        bind.extend(quote!(let #guard=if #condition {unsafe{(*#carrier).tag=-1;} #transfer Some(::prebindgen_runtime::CallbackReturnCleanup::new(move || {unsafe {#cleanup} Ok(())}))}else{None};));
    }
    let mutability = by_value.then(|| quote!(#[allow(unused_mut)] let mut #wire=#wire;));
    let code = plan.code;
    plan.code = quote!({#mutability #bind #code});
    plan.node = WireNode::Chain {
        controls: vec![SourceWire {
            name: wire,
            kind: CWire::Tagged {
                value: value.clone(),
                by_value,
            },
        }],
        child: Box::new(plan.node),
        meta: CChain::Tagged(value),
    };
    Ok(plan)
}
pub(crate) fn output(
    flat: &CBindings,
    ty: &syn::Type,
    wire: syn::Ident,
    mut plan: COutputPlan<CWire>,
) -> Result<COutputPlan<CWire>> {
    let (value, layout) = shape(flat, ty, &plan.outcomes.success)?;
    let native = &value.rust;
    let WireNode::Branch {
        controls, children, ..
    } = &plan.outcomes.success
    else {
        return Err("expected enum output".into());
    };
    let expressions = plan
        .values
        .iter()
        .map(|(n, v)| (n.to_string(), v))
        .collect::<BTreeMap<_, _>>();
    let tag = expressions
        .get(&controls[0].name.to_string())
        .ok_or("missing enum output tag")?;
    let mut cases = vec![];
    for (a, arm) in children.iter().enumerate() {
        let index = a as i32;
        let alternative = format_ident!("a{a}");
        let expression = if let Some(record) = &layout.arms()[a] {
            let record = record.definition().ident;
            let payload = layout.payload_name().unwrap();
            let mut fields = vec![];
            for (field, wires) in arm_fields(arm)? {
                for (i, w) in wires.iter().enumerate() {
                    let name = field_name(field, i, wires.len())?;
                    let expression = expressions
                        .get(&w.name.to_string())
                        .ok_or("missing enum payload expression")?;
                    fields.push(quote!(#name:#expression));
                }
            }
            quote!(#native{tag:#index,payload:#payload{#alternative: ::core::mem::ManuallyDrop::new(#record{#(#fields,)*})}})
        } else if let Some(payload) = layout.payload_name() {
            quote!(#native{tag:#index,payload:#payload{__empty:[]}})
        } else {
            quote!(#native{tag:#index})
        };
        cases.push(quote!(#index=>#expression));
    }
    // No source enum layout is copied: construct its converted carrier.
    let packed = format_ident!("{wire}_carrier_value");
    plan.statements.extend(quote!(let #packed=match #tag {#(#cases,)*_=>unsafe{::core::hint::unreachable_unchecked()}};));
    plan.values = vec![(wire.clone(), quote!(#packed))];
    plan.outcomes.success = WireNode::Chain {
        controls: vec![SourceWire {
            name: wire,
            kind: CWire::Tagged {
                value: value.clone(),
                by_value: true,
            },
        }],
        child: Box::new(plan.outcomes.success),
        meta: CChain::Tagged(value),
    };
    Ok(plan)
}

/// Emit each converted carrier once, after its nested dependencies.
pub(crate) fn emit<S: prebindgen_flat::Schema<Module = CModule, Type = CType>>(
    flat: &Flat<S>,
    node: &WireNode<CWire, CChain, CBranch>,
    seen: &mut BTreeMap<String, String>,
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    match node {
        WireNode::Leaf(wires) => {
            for wire in wires {
                if let CWire::Callback { callback } = &wire.kind {
                    for arg in &callback.arguments {
                        emit(
                            flat,
                            &arg.output.outcomes.success,
                            seen,
                            symbols,
                            tokens,
                            header,
                        )?;
                    }
                    if let Some((_, plan)) = &callback.returned {
                        emit(flat, &plan.node, seen, symbols, tokens, header)?;
                    }
                }
            }
        }
        WireNode::Branch { children, meta, .. } => {
            if let CBranch::Enum(source) = meta {
                let key = format!("tags:{}", type_key(&source.ty));
                if let std::collections::btree_map::Entry::Vacant(entry) = seen.entry(key) {
                    entry.insert(String::new());
                    let stem = tag_stem(&source.ty);
                    for (i, variant) in source.variants.iter().enumerate() {
                        let name = format!(
                            "{}_{}_{}_TAG",
                            flat.metadata.prefix,
                            stem,
                            tag_stem(
                                &syn::parse_str(variant).map_err(|e| format!("variant: {e}"))?
                            )
                        );
                        if !symbols.insert(name.clone()) {
                            return Err("C enum tag constant collision".into());
                        }
                        header.push_str(&format!("#define {name} ((int32_t){i})\n"));
                    }
                }
            }
            for child in children {
                emit(flat, child, seen, symbols, tokens, header)?;
            }
        }
        WireNode::Chain { child, meta, .. } => {
            emit(flat, child, seen, symbols, tokens, header)?;
            let CChain::Tagged(value) = meta else {
                return Ok(());
            };
            let WireNode::Branch {
                meta: CBranch::Enum(source),
                children,
                ..
            } = child.as_ref()
            else {
                return Err("tagged carrier needs source variants".into());
            };
            let (_, layout) = shape(flat, &source.ty, child)?;
            let signature = format!(
                "{}|{}",
                layout.definition(),
                group::values(child)
                    .map(|w| match &w.kind {
                        CWire::Owned { owner, access, .. } =>
                            format!("owned:{}:{access:?}", type_key(&owner.ty)),
                        CWire::Tagged { value, .. } =>
                            format!("tagged:{}:{}", value.name, value.owned),
                        kind => format!("{kind:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join(";")
            );
            if let Some(previous) = seen.get(&value.name) {
                return if previous == &signature {
                    Ok(())
                } else {
                    Err("conflicting input/output layouts for C tagged enum".into())
                };
            }
            seen.insert(value.name.clone(), signature);
            let prefix = &flat.metadata.prefix;
            let name = format!("{prefix}_{}", value.name);
            let native = &value.rust;
            let destructor = &value.destructor;
            let take = format_ident!("{prefix}_take_{}", value.name);
            for symbol in [&name, &destructor.to_string(), &take.to_string()] {
                if !symbols.insert(symbol.clone()) {
                    return Err(format!("duplicate C symbol `{symbol}`"));
                }
            }
            let mut definitions =
                syn::parse2::<syn::File>(layout.definition()).map_err(|e| e.to_string())?;
            for item in &mut definitions.items {
                match item {
                    syn::Item::Struct(s) => s.attrs.push(syn::parse_quote!(#[derive(Clone,Copy)])),
                    syn::Item::Union(u) => u.attrs.push(syn::parse_quote!(#[derive(Clone,Copy)])),
                    _ => {}
                }
            }
            let definition = quote!(#definitions);
            let empty = if let Some(payload) = layout.payload_name() {
                quote!(Self{tag:-1,payload:#payload{__empty:[]}})
            } else {
                quote!(Self { tag: -1 })
            };
            let context = CContext {
                input_transfers: prebindgen_conversions::InputTransfers::Forbidden,
                row_types: Default::default(),
                acknowledgement: syn::parse_quote!(unused),
                output_pointer: syn::parse_quote!(unused),
                output_type: syn::parse_quote!(Unused),
                error_pointer: syn::parse_quote!(unused),
                error_reported: syn::parse_quote!(unused),
                output_committed: TokenStream::new(),
                buffer_type: syn::parse_quote!(__PbBuffer),
                wide_integer_type: syn::parse_quote!(__Pb128),
                read_buffer: syn::parse_quote!(__pb_read),
            };
            let mut union = String::new();
            let slots = slots(child, &layout, quote!(value))?;
            let mut drops = vec![];
            for (a, arm) in children.iter().enumerate() {
                c_member(source.variants[a].trim_start_matches("r#"))?;
                let mut members = String::new();

                for (field, wires) in arm_fields(arm)? {
                    for (i, w) in wires.iter().enumerate() {
                        let member = field_name(field, i, wires.len())?;
                        let mut stored = (*w).clone();
                        match &mut stored.kind {
                            CWire::Owned {
                                access: Access::Move,
                                by_value,
                                ..
                            }
                            | CWire::Tagged { by_value, .. } => *by_value = true,
                            _ => {}
                        }
                        super::validate_wire_declaration(
                            &stored,
                            &flat.types,
                            &flat
                                .types
                                .values()
                                .filter_map(|t| t.metadata.enumeration.as_ref())
                                .map(|e| (&e.name, e.mapping.as_ref()))
                                .collect::<Vec<_>>(),
                        )?;
                        members.push_str(&format!(
                            "        {} {};\n",
                            super::c_ty(&stored, prefix, &flat.types, &context),
                            member.to_string().trim_start_matches("r#")
                        ));
                        if matches!(w.kind, CWire::Rows | CWire::Callback { .. }) {
                            return Err(
                                "ordinary tagged carrier requires self-contained canonical fields"
                                    .into(),
                            );
                        }
                    }
                }
                if !members.is_empty() {
                    union.push_str(&format!(
                        "    struct {{\n{members}    }} {};\n",
                        source.variants[a].trim_start_matches("r#")
                    ));
                }
                let wires = stored_wires(arm);
                let fields = wires
                    .iter()
                    .map(|w| {
                        let slot = &slots[&w.name.to_string()];
                        (w, quote!(::core::ptr::read(#slot)))
                    })
                    .collect::<Vec<_>>();
                let cleanup =
                    crate::sequence::release_fields(arm, &fields, &fields, &|_| quote!(()), false);
                let index = a as i32;
                drops.push(quote!(#index=>{#cleanup}));
                let tag = format!("{name}_{}", source.variants[a].trim_start_matches("r#"));
                if !symbols.insert(tag.clone()) {
                    return Err("duplicate C tagged variant symbol".into());
                }
                header.push_str(&format!("#define {tag} ((int32_t){a})\n"));
            }
            header.push_str(&format!(
                "typedef struct {{\n    int32_t tag;\n{} }} {name};\n",
                if union.is_empty() {
                    String::new()
                } else {
                    format!("    union {{\n{union}    }} payload;\n")
                }
            ));
            header.push_str(&format!("/* Owning tagged value: never copy ownership. Tag -1 is empty. Drop releases only the selected payload. Take transfers the carrier and empties its old slot. */\nvoid {destructor}({name} *value);\n{name} {take}({name} *value);\n"));
            tokens.extend(quote!(#definition
                #[allow(dead_code)] impl #native {fn empty()->Self {#empty}}
                #[doc="# Safety\nValue must point to a live carrier of this exact type, or be null."]
                #[no_mangle] pub unsafe extern "C" fn #destructor(value:*mut #native) {
                    if value.is_null(){return;} let tag=(*value).tag;(*value).tag=-1;
                    match tag {-1=>{},#(#drops,)*_=>::core::hint::unreachable_unchecked()}
                }
                #[doc="# Safety\nValue must point to a live carrier of this exact type."]
                #[no_mangle] pub unsafe extern "C" fn #take(value:*mut #native)->#native {::core::ptr::replace(value,#native::empty())}
            ));
        }
    }
    Ok(())
}

fn tag_stem(ty: &syn::Type) -> String {
    fn label(ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(p) => {
                let segment = p.path.segments.last().unwrap();
                let mut out = segment.ident.to_string();
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            out.push('_');
                            out.push_str(&label(ty));
                        }
                    }
                }
                out
            }
            syn::Type::Tuple(t) if t.elems.is_empty() => "Unit".into(),
            _ => "Value".into(),
        }
    }
    let mut stem = String::new();
    let mut previous = false;
    for c in label(ty).trim_start_matches("r#").chars() {
        if c.is_uppercase() && previous {
            stem.push('_');
        }
        stem.push(c.to_ascii_uppercase());
        previous = c.is_lowercase() || c.is_ascii_digit();
    }
    if matches!(ty,syn::Type::Path(p) if p.path.segments.iter().any(|s|!matches!(s.arguments,syn::PathArguments::None)))
    {
        stem.push_str(&format!(
            "_{:016X}",
            prebindgen_reprc::schema_id(&type_key(ty))
        ));
    }
    stem
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fun;
    use prebindgen_declarations::{convert, from, into, ProjectionMode};
    fn flat(source: &str) -> CBindings {
        CBindings::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn canonical_trait_representations_receive_native_owner_storage() {
        let generated=Cbindgen::new(flat("pub type Label=hidden::Label;pub enum Event{Label(Label),Empty}pub fn echo(value:Event)->Event{}"))
            .convert(convert!(Label).input(from!(String)).output(into!(String)))
            .tagged_union("event",syn::parse_quote!(Event)).function(fun!(echo)).panic().build().unwrap();
        assert!(
            generated.header.contains("pb_owned_string v0;"),
            "{}",
            generated.header
        );
    }
    #[test]
    fn observing_an_owned_enum_does_not_lend_a_temporary_field() {
        let generated=Cbindgen::new(flat("pub struct Child{pub n:i64}pub struct Holder{pub child:Child}pub enum Event{One(Holder),Empty}pub fn child(value:&Holder)->&Child{}pub fn make()->Event{}"))
            .opaque_ptr("child",syn::parse_quote!(Child))
            .tagged_union("event",syn::parse_quote!(Event))
            .function(fun!(make).expand_return(COutputConfig::enum_variants(vec![vec![COutputConfig::Unary { operation: COutputUnary::Getter { function: "child".into() }, child: Box::new(COutputConfig::Leaf(COutputLeafOp::Value)),  }],vec![]], true, Some(ProjectionMode::Borrowed)))).panic().build();
        let error = generated.err().unwrap();
        assert!(error.contains("temporary"), "{error}");
    }
    #[test]
    fn c23_keywords_are_rejected_as_payload_members() {
        let generated = Cbindgen::new(flat(
            "pub enum Event{A{r#typeof:u32},Empty}pub fn make()->Event{}",
        ))
        .tagged_union("event", syn::parse_quote!(Event))
        .function(fun!(make))
        .panic()
        .build();
        assert!(generated.err().unwrap().contains("C identifier"));
    }
}
