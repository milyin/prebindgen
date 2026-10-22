//! Explicit Result error outputs. Message buffers and structured errors are caller-owned.
use super::*;
use prebindgen_conversions::OutputFragment;
use prebindgen_flat::Conversion;
use prebindgen_flat::{Function, TypeUse};

pub(crate) struct ErrorBinding {
    pub name: String,
    pub ty: syn::Type,
    pub plan: OutputPlan<CWire>,
    pub fields: Vec<String>,
}
pub(crate) fn helper(name: &str) -> syn::Ident {
    format_ident!("__pb_report_{name}")
}
pub(crate) fn prepare(
    flat: &mut Bindings,
    codec: &enums::EnumCodec,
    declarations: &prebindgen_declarations::Declarations,
    configured: Vec<(String, syn::Type)>,
) -> Result<Vec<ErrorBinding>> {
    let mut out = Vec::new();
    for (name, ty) in configured {
        let ty = flat.resolve_type(&ty)?;
        let recipe = declarations.owned_record_output(flat, &ty)?;
        let id = helper(&name);
        if flat.functions.contains_key(&id.to_string()) {
            return Err("C error helper name collision".into());
        }
        flat.functions.insert(
            id.to_string(),
            Function {
                name: id.clone(),
                path: syn::parse_quote!(#id),
                params: vec![],
                ret: TypeUse {
                    ty: ty.clone(),
                    slot: None,
                },
                location: Default::default(),
            },
        );
        let result =
            prebindgen_declarations::bind_output(flat, codec, &id.to_string(), &recipe, &());
        let function = flat.functions.remove(&id.to_string()).unwrap();
        result?;
        let plan = function.ret.slot.unwrap();
        let fields = field_names(&plan.layout);
        out.push(ErrorBinding {
            name,
            ty,
            plan,
            fields,
        });
    }
    Ok(out)
}

fn field_names(layout: &prebindgen_flat::WireLayout<CWire, CGroup>) -> Vec<String> {
    use prebindgen_flat::WireNode;
    let mut names = Vec::new();
    if let Some(WireNode::Group {
        metadata: CGroup::Fields,
        children,
    }) = layout.nodes().first()
    {
        for node in children.nodes() {
            if let WireNode::Group {
                metadata: CGroup::Field(name),
                children,
            } = node
            {
                for _ in children.iter() {
                    names.push(if children.len() == 1 {
                        name.to_string().trim_start_matches("r#").to_owned()
                    } else {
                        format!("v{}", names.len())
                    });
                }
            }
        }
    }
    let keywords="auto break case char const continue default do double else enum extern float for goto if inline int long register restrict return short signed sizeof static struct switch typedef union unsigned void volatile while _Alignas _Alignof _Atomic _Bool _Complex _Generic _Imaginary _Noreturn _Static_assert _Thread_local bool true false NULL";
    let mut unique = std::collections::BTreeSet::new();
    if names.len() != layout.len()
        || names.iter().any(|name| {
            !name.is_ascii()
                || keywords.split_whitespace().any(|word| word == name)
                || !unique.insert(name.clone())
        })
    {
        return (0..layout.len()).map(|i| format!("v{i}")).collect();
    }
    names
}

pub(crate) fn emit(
    errors: &[ErrorBinding],
    prefix: &str,
    pointers: &[(String, syn::Type)],
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    for error in errors {
        let name = &error.name;
        let c_name = format!("{prefix}_{name}");
        let drop_name = format_ident!("{prefix}_drop_{name}");
        for symbol in [c_name.clone(), drop_name.to_string()] {
            if !symbols.insert(symbol) {
                return Err("C error type/destructor symbol collision".into());
            }
        }
        let rust_name = format_ident!("__PbError_{name}");
        let source_ty = &error.ty;
        let helper = helper(name);
        let input = format_ident!("source_error");
        let context = CContext {
            acknowledgement: format_ident!("unused_acknowledgement"),
            output_pointer: format_ident!("error_output"),
            output_type: syn::parse_quote!(#rust_name),
            error_pointer: format_ident!("unused_error"),
            error_reported: format_ident!("unused_reported"),
            output_committed: TokenStream::new(),
            buffer_type: syn::parse_quote!(__PbBuffer),
            wide_integer_type: syn::parse_quote!(__Pb128),
            read_buffer: syn::parse_quote!(__pb_read),
        };
        let commit = error.plan.code_fragment_slot(
            OutputFragment::Commit {
                context: &context,
                source: &input,
            },
            source_ty,
        )?;
        let commit: syn::ExprBlock = syn::parse2(commit).map_err(|e| e.to_string())?;
        let commit = commit.block.stmts;
        let deliver = error.plan.code_fragment_slot(
            OutputFragment::Deliver {
                context: &context,
                source: &input,
                next: quote!(#(#commit)* *reported=true; Ok(())),
            },
            source_ty,
        )?;
        let prepare = error.plan.code_fragment_slot(
            OutputFragment::Prepare {
                context: &context,
                source: &input,
                next: deliver,
            },
            source_ty,
        )?;
        let mut fields = Vec::new();
        let mut drops = TokenStream::new();
        header.push_str("typedef struct {\n");
        for (i, wire) in error.plan.layout.iter().enumerate() {
            let field = format_ident!("v{i}");
            let ty = wire.rust_type();
            fields.push(quote!(pub #field:#ty));
            let c_field = if error.fields.len() == error.plan.layout.len() {
                error.fields[i].trim_start_matches("r#").to_owned()
            } else {
                format!("v{i}")
            };
            prebindgen_declarations::ident(&c_field)?;
            header.push_str(&format!(
                "    {} {c_field};\n",
                c_ty(wire, prefix, pointers)
            ));
            match &wire.kind {
                CWire::Buffer => drops.extend(quote! { let value=std::mem::replace(&mut (*error).#field,__PbBuffer {data:std::ptr::null(),len:0}); if !value.data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(value.data as *mut u8,value.len)));} }),
                CWire::Pointer {pointer,..} => {let ty=&pointer.ty; drops.extend(quote! {let value=std::mem::replace(&mut (*error).#field,std::ptr::null_mut());if !value.is_null(){drop(Box::from_raw(value as *mut #ty));}});}
                _ => {}
            }
        }
        if fields.is_empty() {
            fields.push(quote!(pub unused:u8));
            header.push_str("    uint8_t unused;\n");
        }
        header.push_str(&format!("}} {c_name};\n/* Frees owned error fields and clears them; repeated calls are harmless. */\nvoid {drop_name}({c_name} *error);\n"));
        let wires = error.plan.layout.iter().collect::<Vec<_>>();
        let mut comments = describe_groups(&error.plan.layout, &wires, true, &mut 0, "error");
        for i in (0..wires.len()).rev() {
            let field = if error.fields.len() == wires.len() {
                error.fields[i].trim_start_matches("r#").to_owned()
            } else {
                format!("v{i}")
            };
            comments = comments.replace(&format!("output->v{i}"), &format!("error->{field}"));
        }
        header.push_str(&comments);
        tokens.extend(quote! {
            #[repr(C)] pub struct #rust_name {#(#fields),*}
            #[allow(clippy::needless_borrow,clippy::let_unit_value,clippy::unnecessary_cast,clippy::useless_conversion)]
            unsafe fn #helper(#input:#source_ty,error_output:*mut #rust_name,reported:&mut bool)->Result<(),String>{
                if error_output.is_null(){return Ok(());}
                #prepare
            }
            #[doc="# Safety\nThe pointer must be null or writable storage containing a live error produced by this library."]
            #[no_mangle]pub unsafe extern "C" fn #drop_name(error:*mut #rust_name) {
                if let Err(message)=::prebindgen_runtime::boundary(|| {if !error.is_null(){#drops}Ok(())}) {panic!("C error destructor failed: {}",message);}
            }
        });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn result_function(
    source_name: &syn::Ident,
    error_ty: &syn::Type,
    outputs: &[&Wire],
    mut params: Vec<TokenStream>,
    mut cparams: Vec<String>,
    body: &TokenStream,
    context: &CContext,
    prefix: &str,
    pointers: &[(String, syn::Type)],
    errors: &[ErrorBinding],
    consumes: bool,
    comments: &str,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    let name = format_ident!("{prefix}_{source_name}");
    let internal = &context.output_type;
    let out = &context.output_pointer;
    let accepted = &context.acknowledgement;
    let error = &context.error_pointer;
    let reported = &context.error_reported;
    let typed = errors
        .iter()
        .find(|e| prebindgen_flat::type_key(&e.ty) == prebindgen_flat::type_key(error_ty));
    let (rust_error, c_error, report_binding) = if let Some(typed) = typed {
        let rust = format_ident!("__PbError_{}", typed.name);
        let report = helper(&typed.name);
        (
            quote!(#rust),
            format!("{prefix}_{}", typed.name),
            quote! {
                let mut delivered=false;
                if ::prebindgen_runtime::boundary(|| unsafe {#report(<#error_ty as From<String>>::from(message),#error,&mut delivered)}).is_err() && !delivered {
                // All CWire carriers have a valid zero representation with no
                // owned allocations. Never expose uninitialized error storage.
                std::ptr::write(#error,std::mem::zeroed());
            }
            },
        )
    } else {
        (
            quote!(*mut std::ffi::c_char),
            "char *".into(),
            quote!(unsafe {__pb_report_message(message,#error)};),
        )
    };
    let niche = outputs
        .iter()
        .enumerate()
        .find_map(|(i, w)| w.kind.available_niche().map(|n| (i, n)));
    let return_position = niche.as_ref().map(|(i, _)| *i);
    let (rust_return, c_return, failure, success) = if let Some((i, niche)) = &niche {
        let wire = outputs[*i];
        let ty = wire.rust_type();
        let field = format_ident!("v{i}");
        let failure = CCodec.absent_value(&wire.kind, Some(niche), false)?;
        (
            quote!(#ty),
            c_ty(wire, prefix, pointers),
            failure,
            quote!(value.#field),
        )
    } else {
        (quote!(bool), "bool".into(), quote!(false), quote!(true))
    };
    let mut checks = TokenStream::new();
    let mut writes = TokenStream::new();
    let mut display = Vec::new();
    for (i, wire) in outputs.iter().enumerate() {
        if return_position == Some(i) {
            display.push("the return value".to_owned());
            continue;
        }
        let id = format_ident!("out{i}");
        let field = format_ident!("v{i}");
        let ty = wire.rust_type();
        params.push(quote!(#id:*mut #ty));
        cparams.push(format!("{} *{id}", c_ty(wire, prefix, pointers)));
        checks.extend(quote!(if #id.is_null(){return Err("null success-output pointer".into());}));
        writes.extend(quote!(std::ptr::write(#id,value.#field);));
        display.push(format!("*{id}"));
    }
    let acknowledgement = if consumes {
        params.push(quote!(ownership:*mut bool));
        cparams.push("bool *ownership".into());
        quote!(if !ownership.is_null(){std::ptr::write(ownership,#accepted);})
    } else {
        quote!()
    };
    let preflight = if consumes {
        quote!(if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        })
    } else {
        quote!()
    };
    params.push(quote!(#error:*mut #rust_error));
    cparams.push(format!("{c_error} *{error}"));
    let mut comments = comments.to_owned();
    for (i, path) in display.iter().enumerate().rev() {
        comments = comments.replace(&format!("output->v{i}"), path);
    }
    header.push_str(&comments);
    if let Some((i, niche)) = &niche {
        let absent = match niche {
            CNiche::Null => "NULL".to_owned(),
            CNiche::Integer(value) => value.to_string(),
        };
        header.push_str(&format!(
            "/* Result: success field v{i} is returned directly; {absent} means failure. */\n"
        ));
    } else {
        header.push_str("/* Result: true = success; false = failure. Success outputs are unchanged on failure. */\n");
    }
    if consumes {
        header.push_str("/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */\n");
    }
    header.push_str("/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */\n");
    header.push_str(&format!("{c_return} {name}({});\n", cparams.join(", ")));
    let mut rollback = TokenStream::new();
    for (i, wire) in outputs.iter().enumerate() {
        let field = format_ident!("v{i}");
        match &wire.kind {
            CWire::Buffer=>rollback.extend(quote!{if !value.#field.data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(value.#field.data as *mut u8,value.#field.len)));}}),
            CWire::Pointer {pointer,..}=>{let ty=&pointer.ty;rollback.extend(quote!{if !value.#field.is_null(){let _=::prebindgen_runtime::boundary(||{drop(Box::from_raw(value.#field as *mut #ty));Ok(())});}});},
            _=>{}
        }
    }
    tokens.extend(quote! {
        #[doc="# Safety\nSuccess, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract."]
        #[allow(clippy::needless_borrow,clippy::let_unit_value,clippy::unnecessary_cast,clippy::useless_conversion)]
        #[no_mangle]pub unsafe extern "C" fn #name(#(#params),*)->#rust_return {
            #[allow(unused_mut)]let mut #accepted=false;
            #[allow(unused_mut)]let mut #reported=false;
            let mut success_committed=false;
            let mut storage=std::mem::MaybeUninit::<#internal>::uninit();
            let #out=storage.as_mut_ptr();
            let outcome=::prebindgen_runtime::boundary(|| {#preflight #checks #body});
            #acknowledgement
            match outcome {
                Ok(())=>{let value=storage.assume_init();#writes #success}
                Err(message)=>{
                    if success_committed {let value=storage.assume_init();#rollback}
                    if !#reported && !#error.is_null(){#report_binding}#failure
                }
            }
        }
    });
    Ok(())
}
