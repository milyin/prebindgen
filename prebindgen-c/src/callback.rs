//! Typed C callbacks retained by source closures.
use super::*;
use prebindgen_conversions::{InputFragment, OutputFragment, TemplateContext};
use prebindgen_declarations::{CallbackInput, Condition};
use prebindgen_flat::{callback_signature, type_key, InputConversion, WireNode};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

/// Prepared C callback argument and return conversions, retained in its input wire.
#[derive(Clone, Debug)]
pub struct CCallback {
    pub(crate) name: String,
    pub(crate) signature: String,
    arguments: Vec<Argument>,
    returned: Option<(syn::Type, InputPlan<CWire>)>,
    return_owners: BTreeMap<String, syn::Type>,
}
#[derive(Clone, Debug)]
struct Argument {
    ty: syn::Type,
    output: OutputPlan<CWire>,
}

pub(crate) fn input(
    flat: &Bindings,
    configuration: &CallbackInput,
    wire: syn::Ident,
    var: syn::Ident,
    active: &Condition,
) -> Result<InputPlan<CWire>> {
    let signature = callback_signature(&configuration.ty)?.ok_or("expected source callback")?;
    let unit = matches!(&signature.return_type, syn::Type::Tuple(t) if t.elems.is_empty());
    let key = format!(
        "({})->{}",
        signature
            .arguments
            .iter()
            .map(type_key)
            .collect::<Vec<_>>()
            .join(","),
        type_key(&signature.return_type)
    );
    let hash = key.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    let name = format!("{}_callback_{hash:016x}", flat.metadata.prefix);
    let mut arguments = Vec::new();
    for (index, source) in signature.arguments.into_iter().enumerate() {
        if matches!(&source, syn::Type::Reference(r) if r.mutability.is_some()) {
            return Err("mutable callback arguments are unsupported".into());
        }
        let (ty, borrowed) = match &source {
            syn::Type::Reference(r)
                if !matches!(&*r.elem, syn::Type::Slice(_)) && type_key(&r.elem) != "str" =>
            {
                ((*r.elem).clone(), true)
            }
            _ => (source.clone(), false),
        };
        if contains_result(&ty) && prebindgen_flat::result_parts(&ty).is_none() {
            return Err(
                "Result callback arguments require an explicit value representation".into(),
            );
        }
        let output = prebindgen_declarations::compile_callback_argument(
            flat,
            &enums::EnumCodec::callback_argument(index),
            &ty,
            &configuration.arguments[index],
            index,
            borrowed,
            &(),
        )?;
        sequence::validate_buffered(&output.outcomes.success)?;
        arguments.push(Argument { ty: source, output });
    }
    let return_owners = std::cell::RefCell::new(BTreeMap::new());
    let return_codec = enums::EnumCodec {
        return_owners: Some(&return_owners),
        ..Default::default()
    };
    let returned = if unit {
        None
    } else {
        if has_reference(&signature.return_type) {
            return Err(
                "callback return values must be owned; borrowed returns are unsupported".into(),
            );
        }
        return_owners.borrow_mut().clear();
        Some((
            signature.return_type.clone(),
            prebindgen_declarations::compile_callback_return(
                flat,
                &return_codec,
                &signature.return_type,
                &configuration.return_value,
                &(),
            )?,
        ))
    };
    let callback = Rc::new(CCallback {
        name,
        signature: key,
        arguments,
        returned,
        return_owners: return_owners.into_inner(),
    });
    let context = callback.context();
    let owned = format_ident!("{wire}_owner");
    let run = format_ident!("{wire}_run");
    let condition = active.rust();
    let release = format_ident!("{wire}_release_result");
    let release_capture = callback
        .has_release()
        .then(|| quote!(let #release=#wire.release_result;));
    let invocation = callback.invocation(&context, &owned, &run, &release)?;
    let params = callback
        .arguments
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let name = format_ident!("callback_source_{i}");
            let ty = &a.ty;
            quote!(#name:#ty)
        })
        .collect::<Vec<_>>();
    let handled = if unit {
        quote!(if let Err(error) = result {
            eprintln!("prebindgen callback error: {}", error);
        })
    } else {
        quote!(result.unwrap_or_else(|error| panic!("callback failed: {}", error)))
    };
    Ok(InputPlan {
        fallible: true,
        context: Default::default(),
        resources: Vec::new(),
        node: WireNode::Leaf(vec![Wire {
            name: wire.clone(),
            kind: CWire::Callback { callback },
        }]),
        acknowledgement: None,
        validation: quote!(let #run = if #condition {Some(#wire.run.ok_or("null callback function")?)} else {None};),
        acquire: quote!(#release_capture let #owned = if #condition {Some(__PbCallbackOwner::new(#wire.context, #wire.drop))} else {None};),
        statements: quote!(let #run = #run.expect("validated active callback"); let #owned = #owned.expect("validated active callback"); let #var=move |#(#params),*| {let result = ::prebindgen_runtime::boundary(|| #invocation); #handled};),
        argument: quote!(#var),
    })
}
fn contains_result(ty: &syn::Type) -> bool {
    prebindgen_flat::result_parts(ty).is_some()
        || prebindgen_flat::option_inner(ty).is_some_and(contains_result)
        || prebindgen_flat::sequence_parts(ty).is_some_and(|(t, _)| contains_result(t))
        || matches!(ty,syn::Type::Reference(r) if contains_result(&r.elem))
}
fn has_reference(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
        || prebindgen_flat::option_inner(ty).is_some_and(has_reference)
        || prebindgen_flat::sequence_parts(ty).is_some_and(|(ty, _)| has_reference(ty))
}
impl CCallback {
    fn has_release(&self) -> bool {
        self.return_fields()
            .iter()
            .any(|w| matches!(w.kind, CWire::Buffer | CWire::Rows))
    }
    pub(crate) fn absent_value(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let release = self.has_release().then(|| quote!(release_result:None,));
        quote!(#name {run:None,context:std::ptr::null_mut(),drop:None,#release})
    }

    pub(crate) fn argument_fields(&self) -> Vec<&Wire> {
        self.arguments
            .iter()
            .flat_map(|arg| group::callback_parameters(&arg.output.outcomes.success))
            .collect()
    }
    pub(crate) fn return_fields(&self) -> Vec<&Wire> {
        self.returned
            .as_ref()
            .map(|(_, p)| group::argument_parameters(p.input_node()).collect())
            .unwrap_or_default()
    }
    fn return_rust_type(&self) -> syn::Type {
        match self.return_fields().as_slice() {
            [] => syn::parse_quote!(()),
            [field] => field.rust_type(),
            _ => {
                let name = format_ident!("{}_result", self.name);
                syn::parse_quote!(#name)
            }
        }
    }
    pub(crate) fn context(&self) -> CContext {
        let mut schemas = Vec::new();
        for arg in &self.arguments {
            sequence::schemas(&arg.output.outcomes.success, &mut schemas);
        }
        if let Some((_, plan)) = &self.returned {
            sequence::schemas(&plan.node, &mut schemas);
        }
        CContext {
            row_types: schemas
                .into_iter()
                .map(|(w, _, _)| {
                    (
                        w.name.to_string(),
                        format_ident!(
                            "__PbRow_callback_{}_{}",
                            self.name.rsplit_once("_callback_").unwrap().1,
                            w.name
                        ),
                    )
                })
                .collect(),
            acknowledgement: syn::parse_quote!(unused_ack),
            output_pointer: syn::parse_quote!(unused_output),
            output_type: syn::parse_quote!(UnusedOutput),
            error_pointer: syn::parse_quote!(unused_error),
            error_reported: syn::parse_quote!(unused_reported),
            output_committed: quote!(),
            buffer_type: syn::parse_quote!(__PbBuffer),
            wide_integer_type: syn::parse_quote!(__Pb128),
            read_buffer: syn::parse_quote!(__pb_read),
        }
    }
    fn ownership_guard(&self, context: &CContext) -> Result<(TokenStream, TokenStream)> {
        let Some((_, plan)) = &self.returned else {
            return Ok((quote!(), quote!()));
        };
        let fields = self.return_fields();
        if let [field] = fields.as_slice() {
            if let CWire::Pointer { pointer, .. } = &field.kind {
                if self.return_owners.contains_key(&field.name.to_string()) {
                    let name = &field.name;
                    let ty = &pointer.ty;
                    return Ok((
                        quote!(let callback_return_owner=if #name.is_null(){None}else {::prebindgen_runtime::validate_pointer(#name)?;Some(unsafe {Box::<#ty>::from_raw(#name)})};),
                        quote!(let _=callback_return_owner.map(Box::into_raw);),
                    ));
                }
            }
        }
        let scan = receive_return_owners(&plan.node, context, &self.return_owners)?;
        if scan.is_empty() {
            return Ok((quote!(), quote!()));
        }
        Ok((
            quote!(let mut callback_return_owners=::prebindgen_runtime::CallbackReturnAllocations::default();#scan callback_return_owners.check()?;),
            quote!(callback_return_owners.commit();),
        ))
    }
    fn invocation(
        &self,
        context: &CContext,
        owner: &syn::Ident,
        run: &syn::Ident,
        release: &syn::Ident,
    ) -> Result<TokenStream> {
        let mut values = Vec::new();
        let mut commits = TokenStream::new();
        for (index, arg) in self.arguments.iter().enumerate() {
            let source = format_ident!("callback_source_{index}");
            values.extend(
                prebindgen_conversions::match_output_fields(
                    group::callback_parameters(&arg.output.outcomes.success),
                    &arg.output.values,
                )?
                .into_iter()
                .map(|(_, v)| render(context, v, &source))
                .collect::<Result<Vec<_>>>()?,
            );
            commits.extend(arg.output.code_fragment_slot(
                OutputFragment::Commit {
                    context,
                    source: &source,
                },
                &syn::parse_quote!(()),
            )?);
        }
        let fields = self.return_fields();
        let decode = if fields.len() == 1 {
            let field = &fields[0].name;
            quote!(let #field=raw_result;)
        } else {
            let reads = fields.iter().enumerate().map(|(i, w)| {
                let name = &w.name;
                let field = format_ident!("v{i}");
                quote!(let #name=raw_result.#field;)
            });
            quote!(#(#reads)*)
        };
        let (owners, commit_owners) = self.ownership_guard(context)?;
        let result = if let Some((ty, plan)) = &self.returned {
            let target = syn::parse_quote!(callback_return_value);
            let expected = syn::parse_quote!(::prebindgen_runtime::Result<#ty>);
            let mut body = plan.code_fragment_slot(
                InputFragment::Construct {
                    context,
                    target: &target,
                    next: quote!(Ok(#target)),
                },
                &expected,
            )?;
            body = plan.code_fragment_slot(
                InputFragment::Acquire {
                    context,
                    next: body,
                },
                &expected,
            )?;
            body = quote!({#commit_owners #body});
            if !plan.resources.is_empty()
                && !matches!(fields.as_slice(),[field] if matches!(field.kind,CWire::Pointer{..}))
            {
                let seen = syn::parse_quote!(callback_return_resources);
                let check = plan.code_fragment_slot(
                    InputFragment::CheckAliases {
                        context,
                        seen: &seen,
                    },
                    &syn::parse_quote!(()),
                )?;
                body = quote!({let mut #seen:Vec<(usize,bool)>=Vec::new();let _:()=#check;#body});
            }
            plan.code_fragment_slot(
                InputFragment::Validate {
                    context,
                    next: body,
                },
                &expected,
            )?
        } else {
            quote!(Ok(()))
        };
        let names = values
            .iter()
            .enumerate()
            .map(|(i, _)| format_ident!("callback_argument_{i}"))
            .collect::<Vec<_>>();
        let snapshots = names
            .iter()
            .zip(&values)
            .map(|(name, value)| quote!(let #name=#value;));
        let protect_result=self.has_release().then(||quote!(let callback_result=__PbCallbackResult {value:raw_result,context:#owner.context(),release:#release};let raw_result=callback_result.value;));
        let mut body = quote!({ #(#snapshots)* #commits let raw_result=unsafe {#run(#owner.context(), #(#names),*)}; #protect_result #decode #owners #result });
        for (index, arg) in self.arguments.iter().enumerate().rev() {
            let source = format_ident!("callback_source_{index}");
            body = arg.output.code_fragment_slot(
                OutputFragment::Prepare {
                    context,
                    source: &source,
                    next: body,
                },
                &syn::parse_quote!(::prebindgen_runtime::Result<()>),
            )?;
        }
        Ok(body)
    }
}
fn receive_return_owners(
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    context: &CContext,
    owners: &BTreeMap<String, syn::Type>,
) -> Result<TokenStream> {
    Ok(match node {
        WireNode::Leaf(fields) => fields
            .iter()
            .filter_map(|field| {
                if let CWire::Pointer { pointer, .. } = &field.kind {
                    if owners.contains_key(&field.name.to_string()) {
                        let name = &field.name;
                        let ty = &pointer.ty;
                        return Some(
                            quote!(unsafe {callback_return_owners.receive(#name as *mut #ty);}),
                        );
                    }
                }
                None
            })
            .collect(),
        WireNode::Chain {
            meta: CChain::Optional(optional),
            child,
            ..
        } => {
            let scan = receive_return_owners(child, context, owners)?;
            if scan.is_empty() {
                return Ok(scan);
            }
            let name = optional.discriminator();
            let present = optional
                .niche()
                .map(|n| n.present(quote!(#name)))
                .unwrap_or_else(|| quote!(#name==1));
            quote!(if #present {#scan})
        }
        WireNode::Chain {
            meta: CChain::Sequence(_),
            controls,
            child,
        } => {
            let scan = receive_return_owners(child, context, owners)?;
            if scan.is_empty() {
                return Ok(scan);
            }
            let carrier = &controls[0].name;
            let row = &context.row_types[&carrier.to_string()];
            let reads = group::row_fields(child).enumerate().map(|(i, w)| {
                let name = &w.name;
                let field = format_ident!("v{i}");
                quote!(let #name=unsafe {(*((#carrier.data as *const #row).add(index))).#field};)
            });
            quote!(::prebindgen_runtime::validate_slice(#carrier.data as *const #row,#carrier.len)?;for index in 0..#carrier.len {#(#reads)* #scan})
        }
        WireNode::Branch {
            meta: CBranch::Alternatives,
            controls,
            children,
        } => {
            let selector = &controls[0].name;
            let scans = children
                .iter()
                .enumerate()
                .map(|(i, child)| {
                    let scan = receive_return_owners(child, context, owners)?;
                    let i = i as i32;
                    Ok(quote!(if #selector==#i {#scan}))
                })
                .collect::<Result<Vec<_>>>()?;
            quote!(#(#scans)*)
        }
        WireNode::Branch { children, .. } => children
            .iter()
            .map(|child| receive_return_owners(child, context, owners))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect(),
    })
}
fn render(context: &CContext, code: &TokenStream, source: &syn::Ident) -> Result<TokenStream> {
    let code = prebindgen_conversions::render_template(code, &|name| {
        Ok(if name == "source_result" {
            quote!(#source)
        } else {
            prebindgen_conversions::template_parameter(name)
        })
    })?;
    context.render(&code)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit(
    callbacks: &BTreeMap<String, Rc<CCallback>>,
    prefix: &str,
    pointers: &BTreeMap<String, prebindgen_flat::TypeInfo<CType>>,
    enums: &[(&String, &prebindgen_declarations::EnumMapping)],
    symbols: &mut BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    if callbacks.is_empty() {
        return Ok(());
    }
    if callbacks.values().any(|callback| callback.has_release()) {
        tokens.extend(quote!(
            struct __PbCallbackResult<T: Copy> {
                value: T,
                context: *mut ::core::ffi::c_void,
                release: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *const T)>,
            }
            impl<T: Copy> Drop for __PbCallbackResult<T> {
                fn drop(&mut self) {
                    if let Some(release) = self.release {
                        unsafe { release(self.context, &self.value) }
                    }
                }
            }
        ));
    }
    tokens.extend(quote!(
        struct __PbCallbackOwner { context:*mut ::core::ffi::c_void, drop:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)> }
        // The C callback contract requires the context and functions to support
        // concurrent invocation whenever the Rust source requires Send/Sync.
        unsafe impl Send for __PbCallbackOwner {} unsafe impl Sync for __PbCallbackOwner {}
        impl __PbCallbackOwner { fn new(context:*mut ::core::ffi::c_void,drop:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>)->Self{Self{context,drop}} fn context(&self)->*mut ::core::ffi::c_void {self.context} }
        impl Drop for __PbCallbackOwner { fn drop(&mut self){if let Some(drop)=self.drop {unsafe {drop(self.context)}}} }
    ));
    for callback in callbacks.values() {
        if !symbols.insert(callback.name.clone()) {
            return Err("duplicate C callback symbol".into());
        }
        let context = callback.context();
        let mut schemas = Vec::new();
        let mut output_carriers = BTreeSet::new();
        let mut fields = Vec::new();
        for arg in &callback.arguments {
            sequence::schemas(&arg.output.outcomes.success, &mut schemas);
            fields.extend(group::callback_parameters(&arg.output.outcomes.success));
        }
        for (wire, _, _) in &schemas {
            output_carriers.insert(wire.name.to_string());
        }
        if let Some((_, plan)) = &callback.returned {
            sequence::schemas(&plan.node, &mut schemas);
        }
        for field in fields.iter().copied().chain(callback.return_fields()) {
            validate_wire_declaration(field, pointers, enums)?;
        }
        for (_, element, _) in &schemas {
            for field in group::row_fields(element) {
                validate_wire_declaration(field, pointers, enums)?;
            }
        }
        sequence::emit_schemas(
            schemas,
            &output_carriers,
            &context,
            prefix,
            pointers,
            symbols,
            tokens,
            header,
        )?;
        let return_fields = callback.return_fields();
        let result = callback.return_rust_type();
        let c_result = match return_fields.as_slice() {
            [] => "void".into(),
            [field] => c_ty(field, prefix, pointers, &context),
            _ => {
                let name = format!("{}_result", callback.name);
                if !symbols.insert(name.clone()) {
                    return Err("duplicate C callback result symbol".into());
                }
                let names = return_fields.iter().enumerate().map(|(i, w)| {
                    let f = format_ident!("v{i}");
                    let ty = w.rust_type();
                    quote!(pub #f:#ty)
                });
                tokens.extend(
                    quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #result {#(#names),*}),
                );
                header.push_str("typedef struct {\n");
                for (i, w) in return_fields.iter().enumerate() {
                    header.push_str(&format!(
                        "    {} v{i};\n",
                        c_ty(w, prefix, pointers, &context)
                    ));
                }
                header.push_str(&format!("}} {name};\n"));
                name
            }
        };
        let types = fields.iter().map(|w| w.rust_type()).collect::<Vec<_>>();
        let name = format_ident!("{}", callback.name);
        let release_field=callback.has_release().then(||quote!(pub release_result:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void,*const #result)>,));
        tokens.extend(quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #name {pub run:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void,#(#types),*)->#result>,pub context:*mut ::core::ffi::c_void,pub drop:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,#release_field}));
        let release_declaration = if callback.has_release() {
            format!("    void (*release_result)(void *context, const {c_result} *result);\n")
        } else {
            String::new()
        };
        use std::fmt::Write as _;
        let mut args = String::new();
        for (index, field) in fields.iter().enumerate() {
            write!(
                args,
                ", {} arg{index}",
                c_ty(field, prefix, pointers, &context)
            )
            .expect("writing to a String");
        }
        header.push_str(&format!("/* Rust signature: {}. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */\ntypedef struct {{\n    {c_result} (*run)(void *context{args});\n    void *context;\n    void (*drop)(void *context);\n{release_declaration}}} {name};\n",callback.signature));
    }
    Ok(())
}
