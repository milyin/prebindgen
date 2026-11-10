//! Shared immutable row classes and their native constructor registration.
use super::*;
use kotlin::JniNode;
use prebindgen_flat::WireNode;

pub(crate) fn supported(element: &JniNode) -> bool {
    group::row_fields(element)
        .map(|w| w.kind.parameter_slots())
        .sum::<usize>()
        <= 254
}
pub(crate) fn cache(element: &JniNode) -> Result<syn::Ident> {
    let (name, _) = sequence_callback::capture(element, &mut names::Names::default())?;
    Ok(format_ident!("__pb_row_type_{name}"))
}
fn visit<'a>(node: &'a JniNode, input: bool, rows: &mut Vec<&'a JniNode>) {
    match node {
        WireNode::Chain {
            meta: JniChain::Sequence(JniSequence::TypedBuffered),
            child,
            ..
        } if !input => {
            rows.push(child);
            visit(child, input, rows);
        }
        WireNode::Chain { child, .. } => visit(child, input, rows),
        WireNode::Branch { children, .. } => {
            for child in children {
                visit(child, input, rows)
            }
        }
        WireNode::Leaf(wires) => {
            for wire in wires {
                if let JniWire::Callback { callback } = &wire.kind {
                    for (input, node) in callback.bulk_nodes() {
                        visit(node, input, rows);
                    }
                }
            }
        }
    }
}
pub(crate) fn emit<I: KotlinInputConversion>(
    flat: &Flat<JniSchema<I>>,
    rust: &mut TokenStream,
    native: &mut String,
    names: &mut names::Names,
    emitted: &mut BTreeSet<String>,
) -> Result<()> {
    let mut rows = Vec::new();
    for (input, node) in metadata::nodes(flat) {
        visit(node, input, &mut rows);
    }
    let mut schemas = BTreeMap::new();
    for row in rows {
        let (name, declaration) = sequence_callback::capture(row, names)?;
        if emitted.insert(name.clone()) {
            native.push_str(&declaration);
        }
        schemas.insert(name, row);
    }
    if schemas.is_empty() {
        return Ok(());
    }
    let method = names.name("init", "OutputRows", &["typed output row constructors"])?;
    let symbol = symbol(&flat.metadata.package, &method);
    let mut initialize = TokenStream::new();
    let mut classes = Vec::new();
    for (index, (name, row)) in schemas.into_iter().enumerate() {
        let cache = cache(row)?;
        let descriptors = group::row_fields(row)
            .map(|w| w.kind.jvm_descriptor())
            .collect::<Vec<_>>();
        rust.extend(quote! {
            #[allow(non_upper_case_globals)]
            static #cache: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
        });
        let index = index as i32;
        initialize.extend(quote! {
            let class=env.get_object_array_element(&classes,#index).map_err(|e|e.to_string())?;
            let class=env.auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
            #cache.initialize(&mut env,&class,&[#(#descriptors),*])?;
        });
        classes.push(format!("{name}::class.java"));
    }
    let count = classes.len() as i32;
    rust.extend(quote! {
        #[no_mangle]
        pub extern "system" fn #symbol(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,classes: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'_>) {
            if let Err(error)=::prebindgen_runtime::boundary(|| {
                if env.get_array_length(&classes).map_err(|e|e.to_string())? != #count {return Err("output row type count differs".into());}
                #initialize
                Ok(())
            }) {::prebindgen_jni_runtime::fail(&mut env,&error);}
        }
    });
    // Actual Class objects come from the binding's loader. Registration runs
    // after the configured library loader and before any native wrapper call.
    native.push_str(&format!("    @JvmStatic private external fun {method}(classes: Array<Class<*>>)\n    init {{ {method}(arrayOf({})) }}\n",classes.join(", ")));
    Ok(())
}
