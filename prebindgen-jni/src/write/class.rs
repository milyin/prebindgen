//! Kotlin classes and constants.

use prebindgen_flat::flat::TypeRef;
use prebindgen_tools::{names, Shape};
use quote::format_ident;

use super::{
    function::{function, Wrapper},
    jni_symbol, Out,
};
use crate::{
    lower::{alt_kt_name, field_seg, is_u8, kt_prop},
    plan::{Binding, Class, ClassKind, Plan, Res},
};

/// Content equality for an array-backed field: `(equals, hashCode,
/// toString)` templates over `$a` / `$b`.
type ArrayEq = (String, String, String);

fn indent(text: &str, by: usize) -> String {
    let pad = " ".repeat(by);
    text.lines()
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("{pad}{l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One class with its members: the Kotlin text into its package, the
/// members' externs (and a handle's `freePtr`) into the Rust file.
pub(crate) fn class(
    plan: &Plan,
    out: &mut Out,
    c: &Class,
    methods: &[Binding],
    ctors: &[Binding],
) -> Res<()> {
    let methods: Vec<Wrapper> = methods
        .iter()
        .map(|m| function(plan, out, m))
        .collect::<Res<_>>()?;
    let ctors: Vec<Wrapper> = ctors
        .iter()
        .map(|m| function(plan, out, m))
        .collect::<Res<_>>()?;
    let text = match &c.kind {
        ClassKind::Ptr { gc } => {
            out.rust.push(free_ptr(plan, c));
            ptr_class(plan, c, *gc, &methods, &ctors)
        }
        ClassKind::Data => data_class(plan, c, &methods, &ctors)?,
        ClassKind::Enum => enum_class(plan, c)?,
        ClassKind::Sealed { .. } => sealed_class(plan, c)?,
    };
    out.kt(&c.pkg, text);
    Ok(())
}

/// A handle class's `freePtr`, and the alignment its tag bit relies on.
fn free_ptr(plan: &Plan, c: &Class) -> proc_macro2::TokenStream {
    let sym = format_ident!("{}", jni_symbol(&c.fqn(), "freePtr"));
    let t = plan.q.path(&names::ident(&c.rust));
    let msg = format!(
        "`{}`: a handle type must have alignment >= 2 (bit 0 is the closed tag)",
        c.rust
    );
    quote::quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn #sym(
            _env: ::prebindgen_jni_runtime::jni::JNIEnv,
            _class: ::prebindgen_jni_runtime::jni::objects::JClass,
            ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
        ) {
            ::prebindgen_jni_runtime::free_handle::<#t>(ptr)
        }
        const _: () = assert!(::core::mem::align_of::<#t>() >= 2, #msg);
    }
}

fn supertypes(c: &Class, base: &[String]) -> String {
    let mut all: Vec<String> = base.to_vec();
    all.extend(c.iface.clone());
    all.extend(c.implements.iter().cloned());
    if all.is_empty() {
        String::new()
    } else {
        format!(" : {}", all.join(", "))
    }
}

fn iface_decl(c: &Class, extends: &str, members: Vec<String>) -> String {
    let Some(name) = &c.iface else {
        return String::new();
    };
    let body = members
        .iter()
        .map(|m| format!("    {m}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("public interface {name}{extends} {{\n{body}\n}}\n\n")
}

fn ptr_class(plan: &Plan, c: &Class, gc: bool, methods: &[Wrapper], ctors: &[Wrapper]) -> String {
    let name = &c.name;
    let base = &plan.base_pkg;
    let parent = if gc {
        format!("{base}.GcNativeHandle(initialPtr)")
    } else {
        format!("{base}.NativeHandle(initialPtr)")
    };
    let over = if c.iface.is_some() { "override " } else { "" };
    let lifecycle = if gc {
        format!(
            "    private val __cleanable = {base}.registerGcHandle(this) {{ freePtr(it) }}\n\n\
             \x20   @Synchronized\n    override fun close() {{\n        val p = {base}.releaseCell(cell)\n        if (p != 0L) freePtr(p)\n        __cleanable?.clean()\n    }}\n\n\
             \x20   @Synchronized\n    public {over}fun take(): {name} {{\n        val p = {base}.releaseCell(cell)\n        __cleanable?.clean()\n        return {name}(if (p != 0L) p else cell.get())\n    }}\n"
        )
    } else {
        format!(
            "    @Synchronized\n    override fun close() {{\n        val p = ptr\n        if (p != 0L && (p and 1L) == 0L) {{\n            ptr = p or 1L\n            freePtr(p)\n        }}\n    }}\n\n\
             \x20   @Synchronized\n    public {over}fun take(): {name} {{\n        val p = ptr\n        ptr = p or 1L\n        return {name}(p)\n    }}\n"
        )
    };
    let mut members = vec![
        "fun peek(): Long".to_string(),
        "fun isClosed(): Boolean".to_string(),
        format!("fun take(): {name}"),
    ];
    members.extend(methods.iter().map(|m| m.header.clone()));
    let iface = iface_decl(c, " : AutoCloseable", members);
    let methods = methods
        .iter()
        .map(|m| indent(&m.text, 4))
        .collect::<Vec<_>>()
        .join("\n\n");
    let ctors = ctors
        .iter()
        .map(|m| indent(&m.text, 8))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "{iface}public class {name}(initialPtr: Long){} {{\n{lifecycle}\n{methods}\n\n    public companion object {{\n        @JvmStatic\n        external fun freePtr(ptr: Long)\n\n{ctors}\n    }}\n}}\n",
        supertypes(c, &[parent])
    )
}

/// A Kotlin `data class` with fields of `types` fits the JVM's slot limit.
/// Its widest method is the synthetic `copy$default` Kotlin generates: a
/// static method taking the instance, every field, one `Int` mask per 32
/// fields, and a marker object. Its constructor and `fromParts` take less.
fn check_data_class(name: &str, types: &[String]) -> Res<()> {
    let mut copy = vec![name.to_string()];
    copy.extend(types.iter().cloned());
    copy.extend(std::iter::repeat_n(
        "Int".to_string(),
        types.len().div_ceil(32),
    ));
    copy.push("Any".to_string());
    super::check_slots(
        &format!("`{name}.copy$default`, the copy Kotlin generates,"),
        &copy,
        false,
    )
}

fn data_class(plan: &Plan, c: &Class, methods: &[Wrapper], ctors: &[Wrapper]) -> Res<String> {
    let s = plan.struct_of(c)?;
    let over = if c.iface.is_some() { "override " } else { "" };
    let mut props = Vec::new();
    let mut iface_members = Vec::new();
    let mut args = Vec::new();
    let mut closes = Vec::new();
    let mut arrays = Vec::new();
    let mut types = Vec::new();
    for f in &s.fields {
        let p = kt_prop(f);
        let t = plan.kt_type(&f.ty)?;
        types.push(t.clone());
        props.push(format!("{over}val {p}: {t}"));
        iface_members.push(format!("val {p}: {t}"));
        args.push(format!("{p}: {t}"));
        closes.extend(plan.kt_close(&f.ty, &p)?);
        arrays.push((p, array_eq(plan, &f.ty)?));
    }
    let name = &c.name;
    check_data_class(name, &types)?;
    let owns = !closes.is_empty();
    let supers = if owns {
        vec!["AutoCloseable".to_string()]
    } else {
        Vec::new()
    };
    iface_members.extend(methods.iter().map(|m| m.header.clone()));
    let iface = iface_decl(c, "", iface_members);
    let mut body = Vec::new();
    if owns {
        body.push(format!(
            "    override fun close() {{\n        {}\n    }}",
            closes.join("\n        ")
        ));
    }
    if arrays.iter().any(|(_, a)| a.is_some()) {
        body.push(array_members(name, &arrays));
    }
    body.extend(methods.iter().map(|m| indent(&m.text, 4)));
    let names_list = s.fields.iter().map(kt_prop).collect::<Vec<_>>().join(", ");
    let from_parts = format!(
        "        @JvmStatic\n        public fun fromParts({}): {name} = {name}({names_list})",
        args.join(", ")
    );
    let companion = std::iter::once(from_parts)
        .chain(ctors.iter().map(|m| indent(&m.text, 8)))
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(format!(
        "{iface}public data class {name}({}){} {{\n{}\n\n    public companion object {{\n{companion}\n    }}\n}}\n",
        props.join(", "),
        supertypes(c, &supers),
        body.join("\n\n")
    ))
}

/// How a field compares when it is array-backed: `Some((eq, hash, show))`
/// templates over `$a` / `$b`, or `None` for ordinary equality.
fn array_eq(plan: &Plan, ty: &TypeRef) -> Res<Option<ArrayEq>> {
    Ok(match plan.shape(ty)? {
        Shape::Array { .. } => Some(content_eq()),
        Shape::Seq { elem, .. } if is_u8(elem) => Some(content_eq()),
        Shape::Seq { elem, .. } => array_eq(plan, elem)?.map(|(eq, hash, show)| {
            (
                format!(
                    "($a.size == $b.size && $a.indices.all {{ __i -> val __x = $a[__i]; val __y = $b[__i]; {} }})",
                    eq.replace("$a", "__x").replace("$b", "__y")
                ),
                format!("($a.fold(1) {{ __acc, __e -> 31 * __acc + {} }})", hash.replace("$a", "__e")),
                format!(
                    "${{$a.joinToString(\", \", \"[\", \"]\") {{ __e -> \"{}\" }}}}",
                    show.replace("$a", "__e")
                ),
            )
        }),
        Shape::Cow(inner) => match inner.kind() {
            prebindgen_flat::flat::TypeKind::Str | prebindgen_flat::flat::TypeKind::Slice(_) =>
                array_eq(plan, &crate::lower::cow_view(inner))?,
            _ => None,
        },
        Shape::Option(inner) => array_eq(plan, inner)?.map(|(eq, hash, show)| {
            (
                format!(
                    "(if ($a == null || $b == null) $a === $b else {})",
                    eq.replace("$a", "$a!!").replace("$b", "$b!!")
                ),
                format!("($a?.let {{ {} }} ?: 0)", hash.replace("$a", "it")),
                show,
            )
        }),
        _ => None,
    })
}

fn content_eq() -> ArrayEq {
    (
        "$a.contentEquals($b)".to_string(),
        "$a.contentHashCode()".to_string(),
        "${$a.contentToString()}".to_string(),
    )
}

fn array_members(name: &str, fields: &[(String, Option<ArrayEq>)]) -> String {
    let eqs = fields
        .iter()
        .map(|(p, a)| match a {
            Some((eq, _, _)) => eq.replace("$a", p).replace("$b", &format!("other.{p}")),
            None => format!("{p} == other.{p}"),
        })
        .collect::<Vec<_>>()
        .join(" && ");
    let hashes: Vec<String> = fields
        .iter()
        .map(|(p, a)| match a {
            Some((_, h, _)) => h.replace("$a", p),
            None => format!("{p}.hashCode()"),
        })
        .collect();
    let mut hash = format!(
        "        var result = {}\n",
        hashes.first().cloned().unwrap_or("0".into())
    );
    for h in hashes.iter().skip(1) {
        hash.push_str(&format!("        result = 31 * result + {h}\n"));
    }
    let shows = fields
        .iter()
        .map(|(p, a)| match a {
            Some((_, _, s)) => format!("{p}={}", s.replace("$a", p)),
            None => format!("{p}=${{{p}}}"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "    override fun equals(other: Any?): Boolean {{\n        if (this === other) return true\n        if (other !is {name}) return false\n        return {eqs}\n    }}\n\n\
         \x20   override fun hashCode(): Int {{\n{hash}        return result\n    }}\n\n\
         \x20   override fun toString(): String = \"{name}({shows})\""
    )
}

fn enum_class(plan: &Plan, c: &Class) -> Res<String> {
    let values = plan.enum_arms(c)?;
    let over = if c.iface.is_some() { "override " } else { "" };
    let entries = values
        .iter()
        .map(|(n, d)| format!("    {}({d})", names::snake(&names::bare(n)).to_uppercase()))
        .collect::<Vec<_>>()
        .join(",\n");
    let iface = iface_decl(c, "", vec!["val value: Int".to_string()]);
    let name = &c.name;
    Ok(format!(
        "{iface}public enum class {name}(public {over}val value: Int){} {{\n{entries};\n\n    public companion object {{\n        @JvmStatic\n        public fun fromInt(value: Int): {name} = entries.first {{ it.value == value }}\n    }}\n}}\n",
        supertypes(c, &[])
    ))
}

fn sealed_class(plan: &Plan, c: &Class) -> Res<String> {
    let v = plan.variant_of(c)?;
    let name = &c.name;
    let mut owns = false;
    for a in &v.alternatives {
        for f in &a.fields {
            owns |= plan.owns_handle(&f.ty)?;
        }
    }
    let close_body = |closes: Vec<String>| {
        if closes.is_empty() {
            "        override fun close() {\n        }".to_string()
        } else {
            format!(
                "        override fun close() {{\n            {}\n        }}",
                closes.join("\n            ")
            )
        }
    };
    let mut variants = Vec::new();
    let mut from_params = vec!["tag: Int".to_string()];
    let mut from_types = vec!["Int".to_string()];
    let mut arms = Vec::new();
    let mut companion_clash = false;
    for (i, alt) in v.alternatives.iter().enumerate() {
        let vname = alt_kt_name(c, alt);
        companion_clash |= vname == "Companion";
        if alt.fields.is_empty() {
            variants.push(if owns {
                format!(
                    "    public data object {vname} : {name} {{\n{}\n    }}",
                    close_body(Vec::new())
                )
            } else {
                format!("    public data object {vname} : {name}")
            });
            arms.push(format!("{i} -> {vname}"));
            continue;
        }
        let mut props = Vec::new();
        let mut closes = Vec::new();
        let mut args = Vec::new();
        let mut arrays = Vec::new();
        for f in &alt.fields {
            let p = kt_prop(f);
            let t = plan.kt_type(&f.ty)?;
            props.push(format!("public val {p}: {t}"));
            closes.extend(plan.kt_close(&f.ty, &p)?);
            arrays.push((p.clone(), array_eq(plan, &f.ty)?));
            let fp = format!("{}_{}", names::snake(&names::bare(&alt.name)), field_seg(f));
            from_params.push(format!("{fp}: {t}"));
            from_types.push(t);
            args.push(fp);
        }
        check_data_class(
            &format!("{name}.{vname}"),
            &from_types[from_types.len() - alt.fields.len()..],
        )?;
        let mut members = Vec::new();
        if owns {
            members.push(close_body(closes));
        }
        if arrays.iter().any(|(_, a)| a.is_some()) {
            members.push(indent(&array_members(&vname, &arrays), 4));
        }
        let body = if members.is_empty() {
            String::new()
        } else {
            format!(" {{\n{}\n    }}", members.join("\n\n"))
        };
        variants.push(format!(
            "    public data class {vname}({}) : {name}{body}",
            props.join(", ")
        ));
        arms.push(format!("{i} -> {vname}({})", args.join(", ")));
    }
    arms.push(format!(
        "else -> throw IllegalArgumentException(\"{name}: invalid tag $tag\")"
    ));
    // Each alternative's constructor takes a subset of these.
    super::check_slots(&format!("`{name}.fromParts`"), &from_types, true)?;
    let companion = if companion_clash {
        "companion object Companion_"
    } else {
        "companion object"
    };
    let supers = if owns {
        vec!["AutoCloseable".to_string()]
    } else {
        Vec::new()
    };
    Ok(format!(
        "public sealed interface {name}{} {{\n{}\n\n    public {companion} {{\n        @JvmStatic\n        public fun fromParts({}): {name} =\n            when (tag) {{\n                {}\n            }}\n    }}\n}}\n",
        supertypes(c, &supers),
        variants.join("\n\n"),
        from_params.join(", "),
        arms.join("\n                ")
    ))
}

/// A constant: a private getter and a lazily initialized `val`.
pub(crate) fn constant(plan: &Plan, out: &mut Out, pkg: &str, name: &str, b: &Binding) -> Res<()> {
    let w = function(plan, out, b)?;
    let ty = plan.kt_type(&b.func.ret)?;
    let private = w.text.replacen("public fun", "private fun", 1);
    out.kt(
        pkg,
        format!(
            "{private}\npublic val {name}: {ty} by lazy {{ {}({}.JniErrorHandler {{ je -> error(je ?: \"const {name}: JNI getter failed\") }}) }}\n",
            b.kt_name, plan.base_pkg
        ),
    );
    Ok(())
}
