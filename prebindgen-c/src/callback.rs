//! Typed C callbacks retained by source closures.
use super::*;
use prebindgen_conversions::{InputFragment, OutputFragment, TemplateContext};
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
    pub(crate) arguments: Vec<Argument>,
    pub(crate) returned: Option<(syn::Type, CInputPlan<CWire>)>,
}
#[derive(Clone, Debug)]
pub(crate) struct Argument {
    ty: syn::Type,
    pub(crate) output: COutputPlan<CWire>,
}

// Stable wire semantics, without spans, generated local names or recipe Debug.
fn schema(node: &WireNode<CWire, CChain, CBranch>, out: &mut String) {
    fn access(value: Access) -> &'static str {
        match value {
            Access::Move => "owned",
            Access::Shared => "shared",
            Access::Exclusive => "exclusive",
        }
    }
    fn fields(wires: &[Wire], out: &mut String) {
        for wire in wires {
            let value = match &wire.kind {
                CWire::Pointer { pointer, .. } => format!(
                    "pointer:{}:{}",
                    access(pointer.access),
                    type_key(&pointer.ty)
                ),
                CWire::Owned {
                    owner,
                    access: mode,
                    by_value,
                } => format!(
                    "inline:{}:{}:{}:{}",
                    owner.name,
                    type_key(&owner.ty),
                    access(*mode),
                    by_value
                ),
                CWire::Tagged { value, by_value } => format!("tagged:{}:{by_value}", value.name),
                CWire::Enum { name, .. } => format!("enum:{name}"),
                CWire::Scalar { .. } => type_key(&wire.rust_type()),
                CWire::Rows => "rows".into(),
                CWire::View => "view".into(),
                CWire::SliceView { element } => {
                    format!("slice_view:{}", type_key(&element.rust_type()))
                }
                CWire::MutableView => "view_mut".into(),
                CWire::Buffer => "buffer".into(),
                CWire::Callback { callback } => format!("callback:{}", callback.name),
            };
            out.push_str(&format!("{}:{value};", value.len()));
        }
    }
    match node {
        WireNode::Leaf(wires) => {
            out.push('L');
            fields(wires, out);
        }
        WireNode::Chain {
            controls,
            child,
            meta,
        } => {
            out.push('(');
            match meta {
                CChain::Sequence(_) => out.push('S'),
                CChain::Tagged(value) => {
                    out.push('E');
                    out.push_str(&value.name);
                }
                CChain::Optional(optional) => {
                    out.push('O');
                    match optional.niche() {
                        Some(CNiche::Null) => out.push('N'),
                        Some(CNiche::Integer(n)) => out.push_str(&format!("I{n};")),
                        None => out.push('F'),
                    }
                }
            }
            fields(controls, out);
            schema(child, out);
            out.push(')');
        }
        WireNode::Branch {
            controls,
            children,
            meta,
        } => {
            out.push(if matches!(meta, CBranch::Alternatives) {
                'A'
            } else {
                'G'
            });
            out.push('[');
            fields(controls, out);
            for child in children {
                schema(child, out);
            }
            out.push(']');
        }
    }
}

pub(crate) fn input(
    request: prebindgen_declarations::InputArrow<'_, CCodec>,
) -> Result<CInputPlan<CWire>> {
    let prebindgen_declarations::InputArrow {
        flat,
        operation: configuration,
        domain,
        codomain,
        wire,
        target: var,
        active,
        ..
    } = request;
    let argument_recipes = domain.argument_nodes()?;
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
    let codec = CCodec::declared();
    let mut arguments = Vec::new();
    for (index, source) in signature.arguments.into_iter().enumerate() {
        if has_mutable_reference(&source) {
            return Err("mutable callback arguments are unsupported".into());
        }
        let ty = source.clone();
        let output = prebindgen_declarations::compile_callback_argument(
            flat,
            &codec,
            &ty,
            argument_recipes[index].callback_argument_parts()?.1,
            index,
            false,
            &(),
        )?;
        sequence::validate_buffered(&output.outcomes.success)?;
        if output.outcomes.source_error.is_some() {
            return Err(
                "Result callback arguments require an explicit value representation".into(),
            );
        }
        arguments.push(Argument { ty: source, output });
    }
    let return_codec = CCodec::declared();
    let returned = if unit {
        None
    } else {
        if has_reference(&signature.return_type) {
            return Err(
                "callback return values must be owned; borrowed returns are unsupported".into(),
            );
        }
        Some((
            signature.return_type.clone(),
            prebindgen_declarations::compile_callback_return(
                flat,
                &return_codec,
                &signature.return_type,
                codomain,
                &(),
            )?,
        ))
    };
    let mut layout_key = format!("{key}:abi:");
    for argument in &arguments {
        schema(&argument.output.outcomes.success, &mut layout_key);
    }
    layout_key.push_str("->");
    if let Some((_, plan)) = &returned {
        schema(&plan.node, &mut layout_key);
    }
    let hash = layout_key
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    let name = format!("{}_callback_{hash:016x}", flat.metadata.prefix);
    let callback = Rc::new(CCallback {
        name,
        signature: key,
        arguments,
        returned,
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
    let callback_name = callback.name.clone();
    let callback_signature = callback.signature.clone();
    let handled = if unit {
        quote!(if let Err(error) = result {
            ::prebindgen_runtime::report_callback_error(#callback_name, #callback_signature, &error);
        })
    } else {
        quote!(result.unwrap_or_else(|error| {
            ::prebindgen_runtime::report_callback_error(#callback_name, #callback_signature, &error);
            panic!("callback failed: {}", error)
        }))
    };
    let transfer = prebindgen_conversions::record_input_transfer(quote!(1usize));
    Ok(CInputPlan::new(
        WireNode::Leaf(vec![Wire {
            name: wire.clone(),
            kind: CWire::Callback { callback },
        }]),
        quote!(
            let #run=if #condition {Some(unsafe {#wire.run.unwrap_unchecked()})}else{None};
            #release_capture
            let #owned=if #condition {let owner=__PbCallbackOwner::new(#wire.context,#wire.drop); #transfer Some(owner)}else{None};
            let #run=#run.expect("active callback");let #owned=#owned.expect("active callback");
            let #var=move |#(#params),*| {let result=::prebindgen_runtime::boundary(|| #invocation);#handled};
        ),
        quote!(#var),
    ))
}

fn has_mutable_reference(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(reference) => {
            reference.mutability.is_some() || has_mutable_reference(&reference.elem)
        }
        _ => {
            prebindgen_flat::option_inner(ty).is_some_and(has_mutable_reference)
                || prebindgen_flat::sequence_parts(ty)
                    .is_some_and(|(ty, _)| has_mutable_reference(ty))
        }
    }
}
fn has_reference(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
        || prebindgen_flat::option_inner(ty).is_some_and(has_reference)
        || prebindgen_flat::sequence_parts(ty).is_some_and(|(ty, _)| has_reference(ty))
}
impl CCallback {
    pub(crate) fn slice_views(&self, out: &mut BTreeMap<String, Primitive>) {
        for argument in &self.arguments {
            crate::borrowed::collect(&argument.output.outcomes.success, out);
        }
        if let Some((_, input)) = &self.returned {
            crate::borrowed::collect(&input.node, out);
        }
    }
    pub(crate) fn layout_names(&self, names: &mut BTreeSet<String>) {
        for argument in &self.arguments {
            crate::layouts::output_names(&argument.output, names);
        }
        if let Some((_, plan)) = &self.returned {
            crate::layouts::input_names(plan, names);
        }
    }
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
            input_transfers: prebindgen_conversions::InputTransfers::Forbidden,
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
        for field in group::row_fields(&plan.node) {
            if matches!(
                field.kind,
                CWire::Callback { .. }
                    | CWire::Owned {
                        access: Access::Move,
                        by_value: false,
                        ..
                    }
            ) {
                return Err(
                    "callback return ownership requires a tracked pointer or inline value".into(),
                );
            }
        }
        let mut locations = BTreeMap::new();
        let mut snapshots = TokenStream::new();
        for field in group::row_fields(&plan.node) {
            if matches!(field.kind, CWire::Tagged { .. }) {
                let name = &field.name;
                let pointer = format_ident!("{name}_return_location");
                snapshots.extend(quote!(let #pointer=::core::ptr::addr_of!(#name);));
                locations.insert(name.to_string(), quote!(#pointer));
            }
        }
        let scan = receive_return_owners(&plan.node, context, &locations)?;
        if scan.is_empty() {
            return Ok((quote!(), quote!()));
        }
        Ok((
            quote!(#snapshots let mut callback_return_cleanup=::prebindgen_runtime::CallbackReturnCleanup::new(|| {
            let mut __pb_return_ordinal=0usize;
            #scan Ok(())
        });),
            quote!(callback_return_cleanup.commit();),
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
            let mut local = context.clone();
            local.input_transfers = if crate::takes_input_owners(&plan.node) {
                prebindgen_conversions::InputTransfers::Report(format_ident!(
                    "__pb_callback_consumed"
                ))
            } else {
                prebindgen_conversions::InputTransfers::Forbidden
            };
            plan.code_fragment_slot(
                InputFragment {
                    context: &local,
                    target: &target,
                    next: quote!(Ok(#target)),
                },
                &expected,
            )?
        } else {
            quote!(Ok(()))
        };
        let result = if commit_owners.is_empty() {
            result
        } else {
            quote!({let callback_outcome: ::prebindgen_runtime::Result<_> = #result; let callback_decoded=callback_outcome?; #commit_owners Ok(callback_decoded)})
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
        let counter = self
            .returned
            .as_ref()
            .is_some_and(|(_, plan)| crate::takes_input_owners(&plan.node))
            .then(|| {
                quote!(
                    let __pb_callback_consumed=::core::cell::Cell::new(0usize);
                    let __prebindgen_input_consumed=&__pb_callback_consumed;
                )
            });
        let mut body = quote!({
            #counter
            #(#snapshots)* #commits let raw_result=unsafe {#run(#owner.context(), #(#names),*)}; #protect_result #decode #owners #result });
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
    locations: &BTreeMap<String, TokenStream>,
) -> Result<TokenStream> {
    Ok(match node {
        WireNode::Leaf(fields) => fields
            .iter()
            .filter_map(|field| {
                if let CWire::Pointer { pointer, .. } = &field.kind {
                    if pointer.access == Access::Move {
                        let name = &field.name;
                        let ty = &pointer.ty;
                        return Some(
                            quote!({let ordinal=__pb_return_ordinal;__pb_return_ordinal+=1;
                            if ordinal>=__prebindgen_input_consumed.get(){if let Err(error)=::prebindgen_runtime::boundary(||{unsafe {drop(Box::from_raw(#name as *mut #ty));}Ok(())}){::prebindgen_runtime::report_callback_cleanup_error(&error);}}}),
                        );
                    }
                }
                if let CWire::Owned {owner,access:Access::Move,by_value:true}=&field.kind {
                    let name=&field.name;let ty=&owner.ty;
                    let carrier=owner.ident();
                    let pointer=locations.get(&name.to_string()).cloned().unwrap_or_else(||quote!(&#name));
                    return Some(quote!({let ordinal=__pb_return_ordinal;__pb_return_ordinal+=1;
                        if ordinal>=__prebindgen_input_consumed.get(){if let Err(error)=::prebindgen_runtime::boundary(||{unsafe {drop(::core::ptr::read((#pointer as *const #carrier).cast::<#ty>()));}Ok(())}){::prebindgen_runtime::report_callback_cleanup_error(&error);}}}));
                }
                None
            })
            .collect(),
        WireNode::Chain {meta:CChain::Tagged(value),controls,..}=>{
            if !value.owned {TokenStream::new()}else{
                let name=&controls[0].name;let native=&value.rust;let drop=&value.destructor;
                let pointer=locations.get(&name.to_string()).cloned().unwrap_or_else(||quote!(::core::ptr::addr_of!(#name)));
                quote!({let ordinal=__pb_return_ordinal;__pb_return_ordinal+=1;
                    if ordinal>=__prebindgen_input_consumed.get(){unsafe{#drop((#pointer as *const #native).cast_mut());}}})
            }
        }
        WireNode::Chain {
            meta: CChain::Optional(optional),
            child,
            ..
        } => {
            let scan = receive_return_owners(child, context, locations)?;
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
            let carrier = &controls[0].name;
            let row = &context.row_types[&carrier.to_string()];
            let mut row_locations=locations.clone();
            for (i,w) in group::row_fields(child).enumerate(){let field=format_ident!("v{i}");row_locations.insert(w.name.to_string(),quote!(::core::ptr::addr_of!((*((#carrier.data as *const #row).add(index))).#field)));}
            let scan = receive_return_owners(child, context, &row_locations)?;
            if scan.is_empty() {return Ok(scan);}
            let reads = group::row_fields(child).enumerate().map(|(i, w)| {
                let name = &w.name;
                let field = format_ident!("v{i}");
                quote!(let #name=unsafe {(*((#carrier.data as *const #row).add(index))).#field};)
            });
            // The ordinal counts owned fields across rows and nested collections.
            quote!(#[allow(clippy::explicit_counter_loop)] for index in 0..#carrier.len {#(#reads)* #scan})
        }
        WireNode::Branch {
            meta: CBranch::Alternatives | CBranch::Enum(_),
            controls,
            children,
        } => {
            let selector = &controls[0].name;
            let scans = children
                .iter()
                .enumerate()
                .map(|(i, child)| {
                    let scan = receive_return_owners(child, context, locations)?;
                    let i = i as i32;
                    Ok(quote!(if #selector==#i {#scan}))
                })
                .collect::<Result<Vec<_>>>()?;
            quote!(#(#scans)*)
        }
        WireNode::Branch { children, .. } => children
            .iter()
            .map(|child| receive_return_owners(child, context, locations))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect(),
    })
}
fn render(context: &CContext, code: &TokenStream, source: &syn::Ident) -> Result<TokenStream> {
    let code = prebindgen_conversions::render_template(
        code,
        &|name| {
            Ok(if name == "source_result" {
                quote!(#source)
            } else {
                prebindgen_conversions::template_parameter(name)
            })
        },
        &|count| Ok(prebindgen_conversions::record_input_transfer(count.clone())),
    )?;
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
        header.push_str(&format!("/* Rust signature: {}. The callback context transfers and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */\ntypedef struct {{\n    {c_result} (*run)(void *context{args});\n    void *context;\n    void (*drop)(void *context);\n{release_declaration}}} {name};\n",callback.signature));
    }
    Ok(())
}
