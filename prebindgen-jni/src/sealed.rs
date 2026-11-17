//! Kotlin sealed values and selected-variant JNI conversion.
use super::*;
use prebindgen_declarations::{EnumEncoding, EnumForm};

use prebindgen_conversions::{InputConversion, Wire, WireNode};
use prebindgen_flat::{PayloadEnum, PayloadField, PayloadVariant, TypeRef};

/// Declare a Kotlin sealed interface for a Rust payload enum or concrete Result.
#[derive(Clone)]
pub struct DeclJniSealedClass {
    /// Source enum type.
    pub ty: syn::Type,
    /// Literal Kotlin short name; required for concrete generic types.
    pub name: Option<String>,
    /// Source variant name and foreign class-name overrides.
    pub variant_names: Vec<(String, String)>,
}
impl DeclJniSealedClass {
    /// Declare the ordinary value representation of a source enum.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            variant_names: vec![],
        }
    }
    /// Override the Kotlin short name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Rename a source variant without changing its tag or tag constant.
    pub fn variant_name(mut self, source: impl Into<String>, name: impl Into<String>) -> Self {
        self.variant_names.push((source.into(), name.into()));
        self
    }
}
/// Declare an ordinary sealed Kotlin representation without importing the Rust type.
#[macro_export]
macro_rules! sealed_class {($($ty:tt)+)=>{$crate::DeclJniSealedClass::new($crate::__type(stringify!($($ty)+)))};}

pub(crate) fn class<'a>(
    types: &'a crate::metadata::JniTypes,
    r_type: &TypeRef,
) -> Result<&'a Class> {
    crate::metadata::source_enum(types, r_type)?;
    crate::metadata::source_type(types, r_type)?
        .metadata
        .class
        .as_deref()
        .filter(|class| matches!(class.representation, ClassRepresentation::Sealed { .. }))
        .ok_or_else(|| "ordinary payload enum requires a declared sealed class".into())
}
pub(crate) fn variant_name(class: &Class, variant: &syn::Ident) -> String {
    let source = variant.to_string();
    let source = source.trim_start_matches("r#");
    let ClassRepresentation::Sealed { variant_names, .. } = &class.representation else {
        unreachable!("variant naming requires a sealed class")
    };
    variant_names
        .iter()
        .find(|(name, _)| name.trim_start_matches("r#") == source)
        .map_or_else(|| source.to_owned(), |(_, name)| name.clone())
}
fn validate_variants(class: &Class, source: &PayloadEnum<(), JniField>) -> Result<()> {
    let ClassRepresentation::Sealed { variant_names, .. } = &class.representation else {
        return Err("variant naming requires a sealed class".into());
    };
    let mut overrides = BTreeSet::new();
    for (source_name, _) in variant_names {
        let name = source_name.trim_start_matches("r#");
        if !source
            .variants
            .iter()
            .any(|v| v.name.to_string().trim_start_matches("r#") == name)
        {
            return Err(format!("unknown enum variant `{name}`"));
        }
        if !overrides.insert(name) {
            return Err("duplicate enum variant override".into());
        }
    }
    let mut seen = BTreeSet::new();
    for variant in &source.variants {
        let name = variant_name(class, &variant.name);
        validate_class_name(&name)?;
        if !seen.insert(name) {
            return Err("duplicate Kotlin enum variant name".into());
        }
    }
    Ok(())
}

pub(crate) fn encoding<
    Chain: group::ChainMetadata,
    Branch,
    V: prebindgen_conversions::Conversion<Wire = JniWire, Chain = Chain, Branch = Branch>,
>(
    request: prebindgen_declarations::EnumVariants<'_, JniCodec, V>,
    direction: prebindgen_declarations::Direction,
    wrap: impl FnOnce(TypeRef, EnumForm) -> Branch,
) -> Result<EnumEncoding<Branch>> {
    let source = request.flat.payload_enum(request.ty)?;
    let class = request
        .flat
        .type_info(request.ty)
        .ok()
        .and_then(|t| t.metadata.class.as_ref())
        .filter(|c| matches!(c.representation, ClassRepresentation::Sealed { .. }));
    let grouped = request.form == EnumForm::Value;
    if grouped && class.is_none() {
        return Err(format!(
            "ordinary payload enum `{}` requires sealed_class!",
            type_key(request.ty)
        ));
    }
    if let Some(class) = class {
        validate_variants(class, source)?;
    }
    for arm in request.arms.iter_mut() {
        prebindgen_declarations::retire_optional_niches(
            crate::group::values_mut(arm),
            None,
            |_, _| None,
            |w, niche| w.set_niche(niche),
        )?;
    }
    request
        .selector
        .kind
        .set_niche(Some(JniNiche::Integer(-2)))?;
    let inactive = if direction == prebindgen_declarations::Direction::Output {
        request
            .arms
            .iter()
            .map(|arm| {
                crate::group::values(arm)
                    .map(|wire| {
                        Ok((
                            wire.name.clone(),
                            crate::optional::absent(&wire.kind, None, false)?,
                        ))
                    })
                    .collect()
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };
    Ok(EnumEncoding {
        metadata: wrap(request.flat.type_ref(request.ty)?, request.form),
        inactive,
    })
}

// Source field identities and order come from the canonical variant. Occurrence
// nodes only describe conversions of those fields.
#[allow(clippy::type_complexity)] // Borrow each canonical field with its directional conversion subtree.
pub(crate) fn variant_fields<
    'a,
    Chain: group::ChainMetadata,
    Branch,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &'a JniWireNode<Chain, Branch, Annotation, Container>,
    source: &'a PayloadVariant<(), JniField>,
) -> Result<
    impl Iterator<
        Item = Result<(
            &'a PayloadField<JniField>,
            &'a JniWireNode<Chain, Branch, Annotation, Container>,
        )>,
    >,
> {
    let WireNode::Branch { children, .. } = node else {
        return Err("enum arm must group fields".into());
    };
    if children.len() != source.fields.len() {
        return Err(format!(
            "enum occurrence differs from source variant `{}` fields",
            source.name
        ));
    }
    Ok(source
        .fields
        .iter()
        .zip(children)
        .map(|(source_field, node)| {
            if data::field(node)?.to_string().trim_start_matches("r#") != source_field.property() {
                return Err(format!(
                    "enum occurrence differs from source variant `{}` fields",
                    source.name
                ));
            }
            Ok((source_field, node))
        }))
}

pub(crate) fn input(
    nodes: &[JniInputWireNode],
    controls: &[Wire<JniWire>],
    r_type: &TypeRef,
    k_name: Option<&str>,
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let source = crate::metadata::source_enum(context.source_types, r_type)?;
    let declaration = k_name
        .map(|_| class(context.source_types, r_type))
        .transpose()?;
    if source.variants.len() != nodes.len() {
        return Err("enum occurrence differs from its source variants".into());
    }
    let class_name = declaration.map(Class::fqn);
    let [tag] = controls else {
        return Err("enum needs one tag".into());
    };
    if class_name.is_none() {
        let tree = WireNode::Branch {
            controls: controls.to_vec(),
            children: nodes.to_vec(),
            meta: JniInputWireBranch::RChoices,
        };
        let mut out = kotlin::input_kotlin_when(&tree, context, active)?;
        for path in &mut out.parameter_paths {
            for step in &mut path.0 {
                if let prebindgen_declarations::ComponentSegment::Alternative(index) = step {
                    *step = prebindgen_declarations::ComponentSegment::Named(
                        source.variants[*index]
                            .name
                            .to_string()
                            .trim_start_matches("r#")
                            .into(),
                    );
                }
            }
        }
        return Ok(out);
    }
    let class = class_name.as_ref().unwrap();
    let root = k_name.expect("KSealed input binding");
    let tag_name = tag.name.to_string();
    let cases = source
        .variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let name = variant_name(declaration.unwrap(), &variant.name);
            format!("is {class}.{name} -> {i}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let select = format!("when (val value = requireNotNull({root})) {{ {cases} }}");
    let select = if active == "true" {
        select
    } else {
        format!("if ({active}) {select} else 0")
    };
    let mut out = KotlinInput::default();
    out.parameters.push(format!(
        "{root}: {class}{}",
        if active == "true" { "" } else { "?" }
    ));
    out.setup.push(KotlinBinding {
        name: tag_name.clone(),
        expression: select,
        absent: "0".into(),
    });
    out.arguments.push(tag_name.clone());
    out.prepared_arguments.push(tag_name.clone());
    out.handle_inputs = Some(vec![]);
    for (index, (node, variant)) in nodes.iter().zip(&source.variants).enumerate() {
        let variant_class = variant_name(declaration.unwrap(), &variant.name);
        let selected = if active == "true" {
            format!("{tag_name} == {index}")
        } else {
            format!("({active}) && {tag_name} == {index}")
        };
        for pair in variant_fields(node, variant)? {
            let (field, node) = pair?;
            let mut child = kotlin::input_kotlin_when(node, context, &selected)?;
            child.prepare_arguments(kotlin::has_handles(node, false))?;
            data::check_property_input(&field.metadata.kotlin_type, &child.parameters).map_err(
                |error| format!("{class}.{}.{}: {error}", variant_class, field.property()),
            )?;
            if let Some(parameter) = child.parameters.first() {
                let (parameter_name, ty) = parameter
                    .split_once(": ")
                    .ok_or("invalid enum field parameter")?;
                let name = parameter_name.to_owned();
                let absent = match ty.trim_start_matches("kotlin.") {
                    "ULong" => "0uL",
                    "UInt" => "0u",
                    "UShort" => "0.toUShort()",
                    "UByte" => "0.toUByte()",
                    ty => crate::wire::jvm::primitive_local_default(ty).unwrap_or("null"),
                };
                let property = kotlin_identifier(&field.property());
                let expression=format!("(if ({selected}) (requireNotNull({root}) as {class}.{}).{property} else {absent})",variant_class);
                replace_input(&mut child, &name, &expression);
            }
            child.parameters.clear();
            child.parameter_paths.clear();
            kotlin::merge_input(&mut out, child)?;
        }
    }
    Ok(out)
}
fn replace_input(input: &mut KotlinInput, old: &str, new: &str) {
    for text in input
        .arguments
        .iter_mut()
        .chain(&mut input.prepared_arguments)
        .chain(&mut input.locks)
        .chain(&mut input.lock_groups)
        .chain(&mut input.consumed)
        .chain(input.setup.iter_mut().map(|b| &mut b.expression))
        .chain(
            input
                .handle_inputs
                .iter_mut()
                .flatten()
                .flat_map(|h| std::iter::once(&mut h.value).chain(h.consumed.iter_mut())),
        )
    {
        *text = optional::replace(text, old, new);
    }
}

pub(crate) fn output(
    nodes: &[JniOutputWireNode],
    controls: &[Wire<JniWire>],
    r_type: &TypeRef,
    declaration: Option<&Class>,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let source = crate::metadata::source_enum(context.source_types, r_type)?;
    if source.variants.len() != nodes.len() {
        return Err("enum occurrence differs from its source variants".into());
    }
    let class_name = declaration.map(Class::fqn);
    let tag = kotlin::output_leaves(&controls.iter().collect::<Vec<_>>(), context, owners)?;
    let tag_expr = tag.delivered.first().ok_or("missing enum tag")?;
    let mut out = KotlinOutput::default();
    let local = format!("{}enumValue", context.output_prefix);
    let mut body = String::new();
    let mut cleanup = String::new();
    let mut offset = controls.len();
    let mut expanded = Vec::new();
    if class_name.is_none() {
        out.delivered.extend(tag.delivered.clone());
        out.result_types.extend(tag.result_types);
        out.result_names.push(Some("tag".into()));
    }
    for (index, (node, variant)) in nodes.iter().zip(&source.variants).enumerate() {
        let prefix = format!("{}a{index}_", context.output_prefix);
        let ctx = KotlinContext {
            output_prefix: &prefix,
            ..*context
        };
        let mut child = if declaration.is_some() {
            let mut fields = variant_fields(node, variant)?;
            let WireNode::Branch { children, .. } = node else {
                unreachable!()
            };
            kotlin::output_components_checked(children, &ctx, owners, |child| {
                let (field, _) = fields.next().ok_or("missing enum field")??;
                data::check_property_output(&field.metadata.kotlin_type, &child.result_types)?;
                match child.delivered.len() {
                    0 => { child.delivered.push("kotlin.Unit".into()); child.result_types.push("kotlin.Unit".into()); }
                    1 => {}
                    _ => return Err("ordinary enum fields require one Kotlin value; use enum_fields for expanded delivery".into()),
                }
                Ok(())
            })?
        } else {
            kotlin::output_kotlin_scoped(node, &ctx, owners)?
        };
        let count = crate::group::kotlin_result_fields(node).len();
        kotlin::remap_output(&mut child, context.result, context.result, offset, count);
        offset += count;
        out.output_locals.push_str(&child.output_locals);
        if !child.cleanup.is_empty() {
            cleanup.push_str(&format!(
                "if ({tag_expr} == {index}) {{ {} }}\n",
                child.cleanup
            ));
        }
        if let Some(class) = &class_name {
            let variant_class = variant_name(declaration.unwrap(), &variant.name);
            let args = child.delivered.join(", ");
            let constructor = if variant.fields.is_empty() {
                format!("{class}.{variant_class}")
            } else {
                format!("{class}.{variant_class}({args})")
            };
            body.push_str(&format!(
                "{index} -> {{ {} {constructor} }}\n",
                child.materialize
            ));
        } else {
            let source_names = expanded_names(node, &ctx, owners)?;
            let count = child.result_types.len();
            for (i, (ty, expression)) in child
                .result_types
                .into_iter()
                .zip(child.delivered)
                .enumerate()
            {
                let name = format!("{}a{index}_value{i}", context.output_prefix);
                let primitive = matches!(
                    ty.trim_start_matches("kotlin."),
                    "UByte"
                        | "UShort"
                        | "UInt"
                        | "Boolean"
                        | "Byte"
                        | "Short"
                        | "Int"
                        | "Long"
                        | "ULong"
                        | "Float"
                        | "Double"
                        | "Char"
                );
                let absent = match ty.trim_start_matches("kotlin.") {
                    "Boolean" => "false",
                    "Byte" => "0.toByte()",
                    "Short" => "0.toShort()",
                    "Int" => "0",
                    "Long" => "0L",
                    "ULong" => "0uL",
                    "UInt" => "0u",
                    "UShort" => "0.toUShort()",
                    "UByte" => "0.toUByte()",

                    "Float" => "0.0f",
                    "Double" => "0.0",
                    "Char" => "'\\u0000'",
                    _ => "null",
                };
                let result_type = if primitive || ty.ends_with('?') {
                    ty
                } else {
                    format!("{ty}?")
                };
                out.output_locals
                    .push_str(&format!("var {name}: {result_type} = {absent}\n"));
                out.result_types.push(result_type);
                let variant = lower_camel(variant.name.to_string().trim_start_matches("r#"));
                let field_name = child
                    .result_names
                    .get(i)
                    .cloned()
                    .flatten()
                    .or_else(|| source_names.get(i).cloned().flatten());
                out.result_names.push(Some(if let Some(name) = field_name {
                    format!("{variant}{}", upper_first(&name))
                } else if count == 1 && variant != "tag" {
                    variant
                } else {
                    format!("{variant}Value{i}")
                }));
                out.delivered.push(name.clone());
                expanded.push((index, format!("{name} = {expression}\n")));
            }
            body.push_str(&format!(
                "if ({tag_expr} == {index}) {{ {} {} }}\n",
                child.materialize,
                expanded
                    .iter()
                    .filter(|(arm, _)| *arm == index)
                    .map(|(_, s)| s.as_str())
                    .collect::<String>()
            ));
        }
    }
    if let Some(class) = &class_name {
        out.output_locals
            .push_str(&format!("var {local}: {class}? = null\n"));
        out.materialize = format!(
            "{local} = when ({tag_expr}) {{ {body} else -> error(\"invalid native enum tag\") }}\n"
        );
        out.delivered = vec![format!("requireNotNull({local})")];
        out.result_types = vec![class.clone()];
        out.result_names = vec![None];
    } else {
        out.materialize = body;
    }
    out.cleanup = cleanup;
    Ok(out)
}

fn lower_camel(name: &str) -> String {
    let value = camel(name.trim_start_matches("r#"));
    let mut chars = value.chars();
    chars
        .next()
        .map(|c| c.to_lowercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn upper_first(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
// Read source field paths from the same resolved tree. Publicly grouped values
// remain one property; expanded child fields retain their meaningful names.
fn expanded_names(
    node: &JniOutputWireNode,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<Vec<Option<String>>> {
    if let WireNode::Branch {
        meta:
            JniOutputWireBranch::KDataClass { .. }
            | JniOutputWireBranch::KSealed { .. }
            | JniOutputWireBranch::KEnumParams { .. },
        ..
    } = node
    {
        return Ok(kotlin::output_kotlin_scoped(node, context, owners)?.result_names);
    }
    if let WireNode::Annotation {
        meta: JniOutputWireAnnotation::RField { name: field },
        child,
        ..
    } = node
    {
        let mut names = expanded_names(child, context, owners)?;
        let field = field.to_string();
        let tuple = field
            .strip_prefix('v')
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
        if !tuple {
            let field = lower_camel(&field);
            for name in &mut names {
                *name = Some(
                    name.as_ref()
                        .map_or_else(|| field.clone(), |n| format!("{field}{}", upper_first(n))),
                );
            }
        }
        return Ok(names);
    }
    if let WireNode::Branch { children, .. } = node {
        return Ok(children
            .iter()
            .map(|n| expanded_names(n, context, owners))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect());
    }
    Ok(kotlin::output_kotlin_scoped(node, context, owners)?.result_names)
}

pub(crate) fn render<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
    class: &Class,
    ty: &syn::Type,
) -> Result<String> {
    let source = flat.payload_enum(ty)?;
    validate_variants(class, source)?;
    let mut out = format!(
        "package {}\n\npublic sealed interface {}{} {{\n",
        class.pkg,
        class.name,
        if class.has_handles() {
            " : AutoCloseable"
        } else {
            ""
        }
    );
    for arm in &source.variants {
        let name = variant_name(class, &arm.name);
        if arm.fields.is_empty() {
            out.push_str(&format!(
                "public data object {name} : {}{}\n",
                class.fqn(),
                if class.has_handles() {
                    " { override fun close() {} }"
                } else {
                    ""
                }
            ));
            continue;
        }
        let nested = Class {
            name: name.clone(),
            pkg: class.pkg.clone(),
            representation: ClassRepresentation::Data { has_handles: false },
        };
        let rendered = data::render_record_fields(
            &nested,
            arm.fields.iter().map(|f| (f.property(), &f.metadata)),
        )?;
        let text = rendered
            .split_once("public data class")
            .ok_or("invalid variant class")?
            .1;
        let mut text = format!("public data class{text}");
        let at = text.find("\n)").ok_or("invalid variant constructor")? + 2;
        text.insert_str(at, &format!(" : {}", class.fqn()));
        if class.has_handles() {
            let mut close = String::new();
            for f in &arm.fields {
                close.push_str(&close_member(
                    flat,
                    &f.ty,
                    &format!("this.{}", kotlin_identifier(&f.property())),
                    0,
                )?);
            }
            if text.trim_end().ends_with('}') {
                let at = text.rfind('}').unwrap();
                text.insert_str(at, &format!("\noverride fun close() {{ {close} }}\n"));
            } else {
                text.push_str(&format!(" {{ override fun close() {{ {close} }} }}\n"));
            }
        }
        out.push_str(&text);
    }
    out.push_str("}\n");
    Ok(out)
}
fn close_member<S: prebindgen_flat::Schema<Field = data::JniField, Type = JniType>>(
    flat: &Flat<S>,
    ty: &syn::Type,
    value: &str,
    depth: usize,
) -> Result<String> {
    if let Some(inner) = prebindgen_flat::option_inner(ty) {
        let name = format!("member{depth}");
        let body = close_member(flat, inner, &name, depth + 1)?;
        return Ok(if body.is_empty() {
            String::new()
        } else {
            format!("{value}?.let {{ {name} -> {body} }}\n")
        });
    }
    if let Some(inner) = prebindgen_flat::vec_inner(ty) {
        let name = format!("member{depth}");
        let body = close_member(flat, inner, &name, depth + 1)?;
        return Ok(if body.is_empty() {
            String::new()
        } else {
            format!("{value}.forEach {{ {name} -> {body} }}\n")
        });
    }
    let Some(info) = flat.type_info(ty).ok() else {
        return Ok(String::new());
    };
    if let Some(c) = info.metadata.conversion.input.as_ref().or(info
        .metadata
        .conversion
        .output
        .as_ref())
    {
        return close_member(flat, &c.representation, value, depth + 1);
    }
    let Some(class) = &info.metadata.class else {
        return Ok(String::new());
    };
    Ok(match &class.representation {
        ClassRepresentation::Handle { .. } => format!("{value}.close()\n"),
        ClassRepresentation::Sealed {
            has_handles: true, ..
        } => format!("{value}.close()\n"),
        ClassRepresentation::Data { .. } => flat
            .record(ty)?
            .fields
            .iter()
            .map(|f| {
                close_member(
                    flat,
                    &f.ty,
                    &format!(
                        "{value}.{}",
                        kotlin_identifier(f.name.to_string().trim_start_matches("r#"))
                    ),
                    depth + 1,
                )
            })
            .collect::<Result<Vec<_>>>()?
            .join(""),
        ClassRepresentation::ConfiguredData
        | ClassRepresentation::ConfiguredEnum { .. }
        | ClassRepresentation::ConfiguredSealed { .. } => {
            return Err(format!("JNI class `{}` is unresolved", class.fqn()))
        }
        ClassRepresentation::Enum(_) | ClassRepresentation::Sealed { .. } => String::new(),
    })
}

fn tag_word(label: &str) -> String {
    let mut stem = String::new();
    let mut previous_lower = false;
    for c in label.trim_start_matches("r#").chars() {
        if c.is_uppercase() && previous_lower {
            stem.push('_');
        }
        stem.push(c.to_ascii_uppercase());
        previous_lower = c.is_lowercase() || c.is_ascii_digit();
    }
    stem
}
fn tag_stem(ty: &syn::Type) -> String {
    let mut stem = tag_word(&names::type_label(ty));
    if matches!(ty,syn::Type::Path(p) if p.path.segments.iter().any(|s|!matches!(s.arguments,syn::PathArguments::None)))
    {
        stem.push_str(&format!(
            "_{:016X}",
            prebindgen_reprc::schema_id(&names::type_identity(ty))
        ));
    }
    stem
}
/// Publish source tags for every occurrence package and verify the complete
/// source-tag catalogue against the loaded native library before any native call.
pub(crate) fn emit_tags<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
    rust: &mut TokenStream,
    native: &mut String,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    fn visit<
        Chain: group::ChainMetadata,
        Branch: group::BranchMetadata,
        Annotation: group::AnnotationMetadata,
        Container: group::ContainerMetadata,
    >(
        node: &JniWireNode<Chain, Branch, Annotation, Container>,
        package: &str,
        by_package: &mut BTreeMap<String, BTreeSet<TypeRef>>,
    ) -> Result<()> {
        match node {
            WireNode::Leaf(wires) => {
                for wire in wires {
                    if let JniWire::KCallback { callback } = &wire.kind {
                        for root in callback.bulk_nodes() {
                            match root {
                                metadata::BoundaryNode::Input(node) => {
                                    visit(node, &callback.package, by_package)?;
                                }
                                metadata::BoundaryNode::Output(node) => {
                                    visit(node, &callback.package, by_package)?;
                                }
                            }
                        }
                    }
                }
            }
            WireNode::Annotation { child, .. }
            | WireNode::Chain { child, .. }
            | WireNode::Container { child, .. } => visit(child, package, by_package)?,
            WireNode::Branch { meta, children, .. } => {
                if let Some(e) = meta.r_enum() {
                    by_package
                        .entry(package.into())
                        .or_default()
                        .insert(e.clone());
                }
                for child in children {
                    visit(child, package, by_package)?;
                }
            }
        }
        Ok(())
    }
    let mut packages = BTreeMap::new();
    for f in crate::bindings::views(flat)? {
        {
            let metadata = f.metadata;
            for input in f.input_conversions()? {
                visit(input.input_node(), &metadata.package, &mut packages)?;
            }
            let outcomes = f.output_conversion()?.output_outcomes();
            for node in std::iter::once(&outcomes.success)
                .chain(outcomes.source_error.iter())
                .chain(outcomes.binding_failure.iter())
            {
                visit(node, &metadata.package, &mut packages)?;
            }
            if let Some(error) = &metadata.error {
                visit(
                    &error.output()?.output_outcomes().success,
                    &metadata.package,
                    &mut packages,
                )?;
                if let Some((package, _)) = error.handler.rsplit_once('.') {
                    visit(
                        &error.output()?.output_outcomes().success,
                        package,
                        &mut packages,
                    )?;
                }
            }
        }
    }
    for callback in crate::callback::declarations(flat)? {
        visit(callback.input_node(), &flat.metadata.package, &mut packages)?;
    }
    if packages.is_empty() {
        return Ok(());
    }
    let mut catalogue = BTreeMap::new();
    for (package, enums) in packages {
        let mut constants = BTreeMap::new();
        for reference in enums {
            let source = crate::metadata::source_enum(&flat.types, &reference)?;
            let identity = names::type_identity(&source.ty);
            let stem = tag_stem(&source.ty);
            for (index, variant) in source.variants.iter().enumerate() {
                let variant = tag_word(&variant.name.to_string());
                let name = format!("{stem}_{variant}_TAG");
                validate_name(&name)?;
                if constants
                    .insert(
                        name.clone(),
                        format!("public const val {name}: Int = {index}\n"),
                    )
                    .is_some()
                {
                    return Err("payload enum tag constant collision".into());
                }
            }
            catalogue.insert(identity, source);
        }
        files.insert(
            format!("{}/PayloadTags.kt", package.replace('.', "/")),
            format!(
                "package {package}\n\n{}",
                constants.values().cloned().collect::<String>()
            ),
        );
    }
    let schema = catalogue
        .iter()
        .map(|(ty, source)| {
            format!(
                "{ty}:{}",
                source
                    .variants
                    .iter()
                    .map(|v| v.name.to_string().trim_start_matches("r#").to_owned())
                    .collect::<Vec<_>>()
                    .join(";")
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let hash = prebindgen_reprc::schema_id(&schema) as i64;
    let symbol = super::symbol(&flat.metadata.package, "payloadEnumSchema");
    rust.extend(quote!(#[no_mangle] pub extern "system" fn #symbol(_env: ::prebindgen_jni_runtime::jni::JNIEnv,_class: ::prebindgen_jni_runtime::jni::objects::JClass)->i64 {#hash}));
    native.push_str(&format!("    @JvmStatic external fun payloadEnumSchema(): Long\n    init {{ require(payloadEnumSchema() == {hash}L) {{ \"incompatible payload enum tag mapping\" }} }}\n"));
    Ok(())
}

use prebindgen_conversions::FunctionConversions as _;
