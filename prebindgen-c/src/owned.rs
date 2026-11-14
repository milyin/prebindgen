//! Inline owning C values: layout, native conversions and generated accessors.
use super::*;
use prebindgen_declarations::Condition;
use std::rc::Rc;

/// The native Rust value and target storage of one inline C owner.
#[derive(Clone, Debug)]
pub struct COwned {
    /// Unprefixed C type and accessor stem.
    pub name: String,
    /// Original, qualified Rust type.
    pub ty: syn::Type,
    /// Storage layout of that Rust type on the binding target.
    pub layout: OpaqueLayout,
    pub(crate) empty: TokenStream,
    pub(crate) explicit: bool,
}
impl COwned {
    pub(crate) fn ident(&self) -> syn::Ident {
        format_ident!("__PbOwned_{}", self.name)
    }

    pub(crate) fn helper(&self, operation: &str) -> syn::Ident {
        helper(&self.name, operation)
    }
}
pub(crate) fn helper(name: &str, operation: &str) -> syn::Ident {
    format_ident!("__pb_owned_{name}_{operation}")
}
pub(crate) fn referent(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Reference(r) = ty {
        &r.elem
    } else {
        ty
    }
}
pub(crate) fn declaration(flat: &CBindings, ty: &syn::Type) -> Option<Rc<COwned>> {
    flat.type_info(referent(ty)).ok()?.metadata.owned.clone()
}
pub(crate) fn pointer_declared(flat: &CBindings, ty: &syn::Type) -> bool {
    flat.type_info(referent(ty))
        .ok()
        .is_some_and(|t| t.metadata.pointer.is_some())
}
pub(crate) fn native_input(flat: &CBindings, ty: &syn::Type) -> bool {
    pointer_declared(flat, ty) || declaration(flat, ty).is_some_and(|o| o.explicit)
}
pub(crate) fn native_output(flat: &CBindings, ty: &syn::Type) -> bool {
    pointer_declared(flat, ty) || declaration(flat, ty).is_some()
}
fn has_borrow(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(_) => true,
        syn::Type::Path(p) => p.path.segments.iter().any(|s| match &s.arguments {
            syn::PathArguments::AngleBracketed(a) => a.args.iter().any(|a| match a {
                syn::GenericArgument::Lifetime(_) => true,
                syn::GenericArgument::Type(t) => has_borrow(t),
                _ => false,
            }),
            _ => false,
        }),
        syn::Type::Tuple(t) => t.elems.iter().any(has_borrow),
        syn::Type::Slice(s) => has_borrow(&s.elem),
        syn::Type::Array(a) => has_borrow(&a.elem),
        _ => false,
    }
}
pub(crate) fn std_owner(ty: &syn::Type) -> bool {
    !has_borrow(ty)
        && (matches!(
            prebindgen_flat::type_key(ty).as_str(),
            "String" | "Vec < u8 >"
        ) || matches!(
            prebindgen_flat::sequence_parts(ty),
            Some((_, prebindgen_flat::SequenceView::Owned))
        ))
}
pub(crate) fn output(
    owner: Rc<COwned>,
    wire: syn::Ident,
    value: TokenStream,
) -> COutputPlan<CWire> {
    let ident = owner.ident();
    COutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: CWire::Owned {
                    owner,
                    access: Access::Move,
                    by_value: true,
                },
            }]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(let #wire = #ident::guard(#value);),
        values: vec![(wire.clone(), quote!(#wire.0))],
        commit: quote!(::core::mem::forget(#wire);),
    }
}
pub(crate) fn borrowed(
    owner: Rc<COwned>,
    wire: syn::Ident,
    value: TokenStream,
    access: Access,
) -> COutputPlan<CWire> {
    let ident = owner.ident();
    let native = &owner.ty;
    let pointer = if access == Access::Shared {
        quote!(*const #native as *const #ident)
    } else {
        quote!(*mut #native as *mut #ident)
    };
    COutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: CWire::Owned {
                    owner,
                    access,
                    by_value: false,
                },
            }]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(let #wire = #value as #pointer;),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    }
}
pub(crate) fn input(
    owner: Rc<COwned>,
    ty: &syn::Type,
    wire: syn::Ident,
    var: syn::Ident,
    active: &Condition,
    by_value: bool,
) -> Result<CInputPlan<CWire>> {
    let access = match ty {
        syn::Type::Reference(r) if r.mutability.is_none() => Access::Shared,
        syn::Type::Reference(_) => Access::Exclusive,
        _ => Access::Move,
    };
    if by_value && access != Access::Move {
        return Err("borrowed inline C inputs require a pointer to caller storage".into());
    }
    let condition = active.rust();
    let native = &owner.ty;
    let pointer = format_ident!("{wire}_pointer");
    let local = format_ident!("{wire}_storage");
    let ptr = if by_value {
        quote!(&mut #local as *mut _ as *mut #native)
    } else {
        quote!(#wire as *mut #native)
    };
    let storage = by_value.then(|| quote!(#[allow(unused_parens)] let #local=#wire;));
    let empty_var = format_ident!("{wire}_empty");
    let empty = &owner.empty;
    let prepare_empty = (access == Access::Move && !by_value)
        .then(|| quote!(let #empty_var = if #condition {Some(#empty)} else {None};));
    let transfer = prebindgen_conversions::record_input_transfer(quote!(1usize));
    let acquire = if access == Access::Move && by_value {
        quote!(let #var=if #condition {let #pointer=::core::ptr::addr_of!(#local).cast::<#native>(); let value=unsafe {::core::ptr::read(#pointer)}; #transfer Some(value)} else {None};)
    } else if access == Access::Move {
        quote!(let #var = if #condition {{let value=unsafe {::core::ptr::replace(#pointer, #empty_var.expect("prepared empty owner"))}; #transfer Some(value)}} else {None};)
    } else {
        quote!()
    };
    // A by-value carrier may move into a prepared row. Derive every pointer
    // where it is used, from that row's captured storage.
    let borrowed_pointer = if by_value {
        ptr.clone()
    } else {
        quote!(#pointer)
    };
    let (statements, argument) = match access {
        Access::Move => (
            quote!(let #var: #ty = #var.expect("active inline owner");),
            quote!(#var),
        ),
        Access::Shared => (
            quote!(let #var: #ty = unsafe {&*(#borrowed_pointer)};),
            quote!(#var),
        ),
        Access::Exclusive => (
            quote!(let #var: #ty = unsafe {&mut *(#borrowed_pointer)};),
            quote!(#var),
        ),
    };
    let prepare = if by_value {
        quote!(#storage)
    } else {
        quote!(#storage let #pointer=#ptr;#prepare_empty)
    };
    let plan = CInputPlan::new(
        prebindgen_flat::WireNode::Leaf(vec![Wire {
            name: wire,
            kind: CWire::Owned {
                owner,
                access,
                by_value,
            },
        }]),
        quote!(#prepare #acquire #statements),
        argument,
    );
    Ok(plan)
}

pub(crate) fn auto_name(ty: &syn::Type) -> String {
    if prebindgen_flat::type_key(ty) == "String" {
        return "owned_string".into();
    }
    let key = prebindgen_flat::type_key(ty);
    let hash = key.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    let mut readable = String::new();
    for c in key.chars() {
        if c.is_ascii_alphanumeric() {
            readable.push(c.to_ascii_lowercase());
        } else if !readable.ends_with('_') {
            readable.push('_');
        }
    }
    format!("owned_{}_{hash:016x}", readable.trim_matches('_'))
}

pub(crate) fn collect(ty: &syn::Type, types: &mut std::collections::BTreeMap<String, syn::Type>) {
    if std_owner(ty) {
        types.insert(prebindgen_flat::type_key(ty), ty.clone());
    }
    if let syn::Type::Reference(r) = ty {
        collect(&r.elem, types);
    }
    if let Some(t) = prebindgen_flat::option_inner(ty) {
        collect(t, types);
    }
    if let Some((t, e)) = prebindgen_flat::result_parts(ty) {
        collect(t, types);
        collect(e, types);
    }
    if let Some((t, _)) = prebindgen_flat::sequence_parts(ty) {
        collect(t, types);
    }
    if let Ok(Some(sig)) = prebindgen_flat::callback_signature(ty) {
        for t in sig.arguments {
            collect(&t, types);
        }
        collect(&sig.return_type, types);
    }
}

pub(crate) fn emit(
    owner: &COwned,
    layout: crate::layouts::StorageLayout,
    prefix: &str,
    tokens: &mut TokenStream,
    header: &mut String,
) {
    let ident = owner.ident();
    let native = &owner.ty;
    let (definition, c) = {
        let size = layout.size;
        let align = layout.alignment;
        let a = syn::LitInt::new(&align.to_string(), proc_macro2::Span::call_site());
        (quote!(#[allow(non_camel_case_types)] #[repr(C, align(#a))] #[derive(Clone,Copy)] pub struct #ident { storage: ::core::mem::MaybeUninit<[u8;#size]> }),format!("typedef struct {{\n#ifdef __cplusplus\n    alignas({align})\n#else\n    _Alignas({align})\n#endif\n    unsigned char _storage[{size}];\n}} {prefix}_{};\n",owner.name))
    };
    let guard = format_ident!("__PbOwnedGuard_{}", owner.name);
    let drop = format_ident!("{prefix}_drop_{}", owner.name);
    let empty = &owner.empty;
    tokens.extend(quote! {
        #definition
        const _: () = { assert!(::core::mem::size_of::<#ident>() == ::core::mem::size_of::<#native>(), "opaque owner size differs from its Rust type"); assert!(::core::mem::align_of::<#ident>() == ::core::mem::align_of::<#native>(), "opaque owner alignment differs from its Rust type"); };
        #[allow(non_camel_case_types)] struct #guard(#ident);
        impl Drop for #guard {fn drop(&mut self){unsafe {::core::ptr::drop_in_place(&mut self.0 as *mut #ident as *mut #native);}}}
        #[allow(dead_code)] impl #ident {
            fn from_rust(value: #native) -> Self {let mut storage=::core::mem::MaybeUninit::<Self>::uninit(); unsafe {::core::ptr::write(storage.as_mut_ptr().cast::<#native>(),value);storage.assume_init()}}
            fn guard(value: #native) -> #guard {#guard(Self::from_rust(value))}
            unsafe fn as_rust(&self) -> &#native { &*(self as *const Self as *const #native) }
            unsafe fn as_rust_mut(&mut self) -> &mut #native { &mut *(self as *mut Self as *mut #native) }
            unsafe fn into_rust(self) -> #native { ::core::ptr::read(&self as *const Self as *const #native) }
            fn empty() -> Self { Self::from_rust(#empty) }
        }
        #[doc="# Safety\nThe pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid."]
        #[no_mangle] pub unsafe extern "C" fn #drop(value:*mut #ident) {
            if let Err(error) = ::prebindgen_runtime::boundary(|| { if !value.is_null() {drop(::core::ptr::replace(value as *mut #native, #empty));} Ok(()) }) { panic!("C destructor failed: {}",error); }
        }
    });
    header.push_str(&c);
    header.push_str(&format!("/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */\nvoid {drop}({prefix}_{} *value);\n",owner.name));
}

/// Register ordinary Rust helper functions so expansions can reference them.
pub(crate) fn accessors(flat: &mut CBindings) -> Result<()> {
    let owners: Vec<_> = flat
        .types
        .values()
        .filter_map(|t| t.metadata.owned.clone())
        .collect();
    for owner in owners {
        if !std_owner(&owner.ty) {
            continue;
        }
        let ty = &owner.ty;
        let len = owner.helper("len");
        let mut functions: Vec<syn::ItemFn> = vec![
            syn::parse_quote!(#[allow(dead_code,clippy::ptr_arg)] fn #len(value: &#ty) -> usize {value.len()}),
        ];
        if matches!(
            prebindgen_flat::type_key(ty).as_str(),
            "String" | "Vec < u8 >"
        ) {
            let view = owner.helper("view");
            let expression = if prebindgen_flat::type_key(ty) == "String" {
                quote!(value.as_bytes())
            } else {
                quote!(value.as_slice())
            };
            functions.push(syn::parse_quote!(#[allow(dead_code,clippy::ptr_arg)] fn #view(value: &#ty) -> &[u8] {#expression}));
        }
        if let Some((element, _)) = prebindgen_flat::sequence_parts(ty) {
            if crate::borrowed::primitive(element, &flat.types).is_some() {
                let view = owner.helper("view");
                functions.push(syn::parse_quote!(#[allow(dead_code,clippy::ptr_arg)] fn #view(value: &#ty) -> &[#element] {value.as_slice()}));
            }
        }
        if prebindgen_flat::type_key(ty) == "Vec < u8 >" {
            let view_mut = owner.helper("view_mut");
            functions.push(syn::parse_quote!(#[allow(dead_code,clippy::ptr_arg)] fn #view_mut(value: &mut #ty) -> &mut [u8] {value.as_mut_slice()}));
        }
        for function in functions {
            let name = function.sig.ident.clone();
            if flat.functions.contains_key(&name.to_string()) {
                return Err(format!(
                    "generated owner accessor `{name}` collides with a source function"
                ));
            }
            let ret = match &function.sig.output {
                syn::ReturnType::Type(_, ty) => (**ty).clone(),
                _ => unreachable!(),
            };
            let receiver = match function.sig.inputs.first().unwrap() {
                syn::FnArg::Typed(parameter) => (*parameter.ty).clone(),
                _ => unreachable!(),
            };
            flat.functions.insert(
                name.to_string(),
                prebindgen_flat::Function {
                    metadata: None,
                    name: name.clone(),
                    path: syn::parse_quote!(#name),
                    params: vec![prebindgen_flat::Param {
                        name: syn::parse_quote!(value),
                        ty: prebindgen_flat::TypeUse {
                            ty: receiver,
                            slot: None,
                        },
                    }],
                    ret: prebindgen_flat::TypeUse {
                        ty: ret,
                        slot: None,
                    },
                    location: Default::default(),
                },
            );
            flat.metadata.helpers.push(function);
        }
    }
    Ok(())
}
pub(crate) fn emit_accessors(
    owner: &COwned,
    prefix: &str,
    types: &std::collections::BTreeMap<String, prebindgen_flat::TypeInfo<CType>>,
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    if !std_owner(&owner.ty) {
        return Ok(());
    }
    let carrier = owner.ident();
    let len = format_ident!("{prefix}_{}_len", owner.name);
    if !symbols.insert(len.to_string()) {
        return Err("generated owner length accessor collides with a C symbol".into());
    }
    tokens.extend(quote!(#[doc="# Safety\nThe owner must be live, aligned, and of this exact type."] #[no_mangle] pub unsafe extern "C" fn #len(value:*const #carrier)->usize {(*value).as_rust().len()}));
    header.push_str(&format!(
        "size_t {len}(const {prefix}_{} *value);\n",
        owner.name
    ));
    let key = prebindgen_flat::type_key(&owner.ty);
    if matches!(key.as_str(), "String" | "Vec < u8 >") {
        let view = format_ident!("{prefix}_{}_view", owner.name);
        let helper = owner.helper("view");
        if !symbols.insert(view.to_string()) {
            return Err("generated owner view accessor collides with a C symbol".into());
        }
        tokens.extend(quote!(#[doc="# Safety\nThe owner must remain live and unmodified while its returned view is used. Obtaining a mutable view invalidates earlier views, even if no bytes are changed."] #[no_mangle] pub unsafe extern "C" fn #view(value:*const #carrier)->__PbBuffer {let bytes=#helper((*value).as_rust());__PbBuffer{data:bytes.as_ptr(),len:bytes.len()}}));
        header.push_str(&format!("/* Borrowed view; never free it. The owner retains the allocation. A later mutable view invalidates earlier views. */\n{prefix}_buffer_view {view}(const {prefix}_{} *value);\n",owner.name));
        if key == "Vec < u8 >" {
            let view_mut = format_ident!("{prefix}_{}_view_mut", owner.name);
            let helper_mut = owner.helper("view_mut");
            if !symbols.insert(view_mut.to_string()) {
                return Err(
                    "generated owner mutable view accessor collides with a C symbol".into(),
                );
            }
            tokens.extend(quote!(#[doc="# Safety\nThe owner must remain live and stationary. Preserve exclusive access to its allocation until the returned view's last use; do not access, consume or drop the owner during that borrow."] #[no_mangle] pub unsafe extern "C" fn #view_mut(value:*mut #carrier)->__PbBufferMut {let bytes=#helper_mut((*value).as_rust_mut());__PbBufferMut{data:bytes.as_mut_ptr(),len:bytes.len()}}));
            header.push_str(&format!("/* Exclusive borrowed view; never free it. Keep the owner alive and preserve exclusive access until the view's last use. */\n{prefix}_buffer_view_mut {view_mut}({prefix}_{} *value);\n",owner.name));
        }
    } else if let Some((element, _)) = prebindgen_flat::sequence_parts(&owner.ty) {
        if let Some(primitive) = crate::borrowed::primitive(element, types) {
            let view = format_ident!("{prefix}_{}_view", owner.name);
            let helper = owner.helper("view");
            if !symbols.insert(view.to_string()) {
                return Err("generated owner view accessor collides with a C symbol".into());
            }
            let rust_view = crate::borrowed::ident(primitive);
            let c_view = crate::borrowed::c_name(primitive, prefix);
            tokens.extend(quote!(#[doc="# Safety\nKeep the owner alive; do not mutate it, obtain overlapping exclusive/mutable access, consume or destroy it until the returned view's last use."] #[no_mangle] pub unsafe extern "C" fn #view(value:*const #carrier)->#rust_view {let values=#helper((*value).as_rust()); #rust_view {data:values.as_ptr(),len:values.len()}}));
            header.push_str(&format!("/* Borrowed view; never free. Keep the owner alive; do not mutate it or obtain overlapping exclusive/mutable access until the view's last use. */\n{c_view} {view}(const {prefix}_{} *value);\n", owner.name));
        }
        let get = format_ident!("{prefix}_{}_get", owner.name);
        if !symbols.insert(get.to_string()) {
            return Err("generated owner element accessor collides with a C symbol".into());
        }
        let declared = types
            .values()
            .find(|t| prebindgen_flat::same_type(&t.ty, element))
            .and_then(|t| {
                t.metadata
                    .pointer
                    .as_ref()
                    .or_else(|| t.metadata.owned.as_ref().map(|o| &o.name))
            });
        let c_element = if let Some(name) = declared {
            format!("{prefix}_{name}")
        } else if let Some(Value::Scalar(primitive)) = Value::of(element)
            .filter(|v| !matches!(v, Value::Scalar(Primitive::I128 | Primitive::U128)))
        {
            c_scalar(primitive, prefix)
        } else {
            "void".into()
        };
        tokens.extend(quote!(#[doc="# Safety\nThe owner must remain live and unmodified while an element pointer is used. The index must be smaller than the owner length."] #[no_mangle] pub unsafe extern "C" fn #get(value:*const #carrier,index:usize)->*const #element {(*value).as_rust().get_unchecked(index) as *const #element}));
        header.push_str(&format!("/* Borrowed element; index must be smaller than the owner length. */\nconst {c_element} *{get}(const {prefix}_{} *value,size_t index);\n",owner.name));
    }
    Ok(())
}

pub(crate) fn validate_type(ty: &syn::Type) -> Result<()> {
    if has_borrow(ty)
        || matches!(ty, syn::Type::Reference(_))
        || prebindgen_flat::option_inner(ty).is_some()
        || prebindgen_flat::result_parts(ty).is_some()
        || prebindgen_flat::callback_signature(ty)?.is_some()
    {
        return Err("opaque storage requires an owned non-optional value type".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expand_return;
    use crate::fun;
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
    fn callback_constructor_borrows_inline_owner_through_a_pointer() {
        let model = Cbindgen::new(flat("pub struct Owner{pub n:i64} pub fn empty()->Owner{} pub fn inspect(value:&Owner)->i64{}"))
            .opaque_owned_struct_with_empty(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<i64>(),
                fun!(empty),
            )
            .function(fun!(inspect)).panic()
            .prepare()
            .unwrap();
        let owner_type = referent(&model.function("inspect").unwrap().params[0].ty.ty).clone();
        let owner = declaration(&model, &owner_type).unwrap();
        for ty in [
            syn::parse_quote!(&#owner_type),
            syn::parse_quote!(&mut #owner_type),
        ] {
            let condition = Default::default();
            let mut request = prebindgen_declarations::InputLeaf::new(
                &model,
                &ty,
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &condition,
            );
            request.role = prebindgen_declarations::InputRole::CallbackResult;
            let plan = CCodec::declared().input_leaf(request).unwrap();
            assert!(
                matches!(&plan.node,prebindgen_flat::WireNode::Leaf(w) if matches!(&w[0].kind,CWire::Owned{by_value:false,..}))
            );
            assert!(input(
                owner.clone(),
                &ty,
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &condition,
                true
            )
            .unwrap_err()
            .contains("pointer to caller storage"));
        }
    }
    #[test]
    fn native_owners_keep_vectors_as_leaves_but_explicit_rows_remain_available() {
        let source = "pub fn f()->Option<Vec<i64>>{}";
        let model = Cbindgen::new(flat(source))
            .function(fun!(f).callback_return())
            .panic()
            .prepare()
            .unwrap();
        let native = model.function("f").unwrap().ret.slot.as_ref().unwrap();
        assert!(group::result_packet_fields(&native.outcomes.success)
            .any(|w| matches!(w.kind, CWire::Owned { by_value: true, .. })));
        let visitor = model.function("f_each").unwrap().ret.slot.as_ref().unwrap();
        assert!(sequence::callback_delivery(&visitor.outcomes.success).unwrap());
        let generated = generate(&model).unwrap();
        syn::parse_file(&generated.rust).unwrap();
    }
    #[test]
    fn inline_getter_loans_are_diagnosed_instead_of_dangling_or_cloning() {
        let source="pub struct Owner{pub child:Child} pub struct Child{pub n:i64} pub fn make()->Owner{} pub fn child(v:&Owner)->&Child{}";
        let error = Cbindgen::new(flat(source))
            .opaque_owned_struct(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<i64>(),
            )
            .opaque_ptr("child", syn::parse_quote!(Child))
            .function(
                fun!(make)
                    .expand_return(expand_return!(Owner).field_self_into().field(fun!(child))),
            )
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("temporary owner"), "{error}");
        let error = Cbindgen::new(flat(
            "pub fn make()->Vec<i64>{} pub fn descriptor(v:&Vec<i64>)->&Vec<i64>{}",
        ))
        .function(
            fun!(make).expand_return(
                expand_return!(Vec<i64>)
                    .field_self_into()
                    .field(fun!(descriptor)),
            ),
        )
        .panic()
        .build()
        .err()
        .unwrap();
        assert!(error.contains("movable descriptor"), "{error}");
    }
    #[test]
    fn temporary_getter_views_are_rejected_for_opaque_and_inline_borrows() {
        for returned in ["&str", "&[u8]", "&String"] {
            let source=format!("pub type Owner=String; pub fn make()->Owner{{}} pub fn view(v:&Owner)->{returned}{{}}");
            let error = Cbindgen::new(flat(&source))
                .opaque_ptr("owner", syn::parse_quote!(Owner))
                .function(fun!(make).expand_return(expand_return!(Owner).field(fun!(view))))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(error.contains("temporary owner"), "{returned}: {error}");
        }
        let source = "pub fn values()->Vec<&'static str>{}";
        assert!(Cbindgen::new(flat(source))
            .function(fun!(values))
            .build()
            .is_err());
        assert!(Cbindgen::new(flat(source))
            .opaque_owned_struct(
                "borrowed",
                syn::parse_quote!(Vec<&'static str>),
                OpaqueLayout::auto::<Vec<u8>>()
            )
            .build()
            .is_err());
    }
    #[test]
    fn borrowed_records_and_sequences_cannot_lend_owned_getter_temporaries() {
        use crate::fields;
        use prebindgen_declarations::{ProjectionMode, SequenceDelivery};
        let source="pub type Owner=String; pub struct Info{pub inner:Inner} pub type Inner=String; pub fn make()->Owner{} pub fn borrow(v:&Owner)->&Owner{} pub fn info(o:&Owner)->Info{} pub fn name(v:&Inner)->&str{} pub fn list(o:&Owner)->Vec<Inner>{}";
        for function in ["make", "borrow"] {
            let fields = fields!(info).field("inner", expand_return!(Inner).field(fun!(name)));
            let (function, expansion) = if function == "make" {
                (
                    fun!(make),
                    expand_return!(Owner).field_self_into().fields(fields),
                )
            } else {
                (fun!(borrow), expand_return!(Owner).fields(fields))
            };
            let error = Cbindgen::new(flat(source))
                .opaque_ptr("owner", syn::parse_quote!(Owner))
                .opaque_ptr("inner", syn::parse_quote!(Inner))
                .function(function.expand_return(expansion))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(error.contains("temporary owner"), "{error}");
        }
        let error = Cbindgen::new(flat(source))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .opaque_ptr("inner", syn::parse_quote!(Inner))
            .function(fun!(borrow).output(COutputConfig::getter(
                "list",
                COutputConfig::Unary {
                    operation: COutputUnary::Sequence {
                        delivery: SequenceDelivery::Value,
                        mode: Some(ProjectionMode::Borrowed),
                    },
                    child: Box::new(COutputConfig::getter("name", COutputConfig::auto())),
                },
            )))
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("temporary owner"), "{error}");
    }
    #[test]
    fn layouts_conflicts_and_empty_factories_are_checked() {
        assert!(Cbindgen::new(flat("pub struct Owner{pub n:i64}"))
            .opaque_owned_struct(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<()>()
            )
            .prepare()
            .is_err());
        let source="pub struct Owner{pub n:i64} pub fn empty()->Owner{} pub fn wrong()->i64{} pub fn make()->Owner{}";
        assert!(Cbindgen::new(flat(source))
            .opaque_owned_struct_with_empty(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<i64>(),
                fun!(wrong)
            )
            .prepare()
            .is_err());
        assert!(Cbindgen::new(flat(source))
            .opaque_ptr("ptr", syn::parse_quote!(Owner))
            .opaque_owned_struct(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<i64>()
            )
            .prepare()
            .is_err());
        let generated = Cbindgen::new(flat(source))
            .opaque_owned_struct_with_empty(
                "owner",
                syn::parse_quote!(Owner),
                OpaqueLayout::auto::<i64>(),
                fun!(empty),
            )
            .function(fun!(make))
            .build()
            .unwrap();
        assert!(generated.rust.contains("MaybeUninit"));
        assert!(!generated.rust.contains("Owner as ::core::default::Default"));
    }
}
