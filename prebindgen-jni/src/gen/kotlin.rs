//! Kotlin emission: classes, constants, the runtime prelude, files.

use std::rc::Rc;

use prebindgen_tools::{
    flat::flat::{Function, Type as FlatType, TypeKind},
    names, FnRef,
};
use quote::quote;

use super::{
    codec::{alt_kt_name, kt_prop, Dir, Kind},
    err, Bound, Class, ClassKind, Gen, Placement, Res,
};
use crate::decl::{ConstDecl, ConstSource};

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

/// Kotlin's own runtime for the generated code: the handle base classes,
/// lock ordering, and the binding-error channel.
const PRELUDE: &str = r#"/**
 * Base class for every typed native handle: owns the raw pointer slot and its
 * monitor. Closing sets bit 0 (the closed tag) instead of zeroing, so the
 * address bits are write-once and a sound lock-ordering key.
 */
public abstract class NativeHandle(initialPtr: Long) : AutoCloseable {
    @Volatile internal open var ptr: Long = initialPtr

    /** Mark this handle consumed by value: the native side now owns it. */
    internal open fun markConsumed() {
        ptr = ptr or 1L
    }

    /** The live pointer, or `0` if this handle is closed. */
    public fun peek(): Long {
        val p = ptr
        return if (p == 0L || (p and 1L) != 0L) 0L else p
    }

    public fun isClosed(): Boolean = ptr == 0L || (ptr and 1L) != 0L
}

/**
 * A handle freed by a [java.lang.ref.Cleaner] when unreachable, as well as by
 * `close()`: the pointer lives in a separate cell the cleaning action holds.
 */
public abstract class GcNativeHandle(initialPtr: Long) : NativeHandle(initialPtr) {
    internal val cell: java.util.concurrent.atomic.AtomicLong = java.util.concurrent.atomic.AtomicLong(initialPtr)

    internal final override var ptr: Long
        get() = cell.get()
        set(v) { cell.set(v) }

    internal final override fun markConsumed() {
        releaseCell(cell)
    }
}

/** Win the once-only release of a gc-managed handle: the address, or `0`. */
internal fun releaseCell(cell: java.util.concurrent.atomic.AtomicLong): Long {
    while (true) {
        val v = cell.get()
        if (v == 0L || (v and 1L) != 0L) return 0L
        if (cell.compareAndSet(v, v or 1L)) return v
    }
}

internal object NativeCleaner {
    @JvmField val CLEANER: java.lang.ref.Cleaner = java.lang.ref.Cleaner.create()
}

/** Register a gc-managed handle's release; captures only its cell. */
internal fun registerGcHandle(handle: GcNativeHandle, free: (raw: Long) -> Unit): java.lang.ref.Cleaner.Cleanable? {
    if (handle.isClosed()) return null
    val c = handle.cell
    return NativeCleaner.CLEANER.register(handle) { val p = releaseCell(c); if (p != 0L) free(p) }
}

/** Lock every handle's monitor in address order, then run [body]. */
internal fun <R> withSortedHandleLocks(handles: List<NativeHandle>, body: () -> R): R {
    val sorted = handles.sortedBy { it.ptr and -2L }
    fun rec(i: Int): R = if (i == sorted.size) body() else synchronized(sorted[i]) { rec(i + 1) }
    return rec(0)
}

internal inline fun <R> withSortedHandleLocks(a: NativeHandle, body: () -> R): R = synchronized(a) { body() }

internal inline fun <R> withSortedHandleLocks(a: NativeHandle, b: NativeHandle, body: () -> R): R {
    val first: NativeHandle
    val second: NativeHandle
    if ((a.ptr and -2L) <= (b.ptr and -2L)) { first = a; second = b } else { first = b; second = a }
    return synchronized(first) { synchronized(second) { body() } }
}

internal inline fun <R> withSortedHandleLocks(a: NativeHandle, b: NativeHandle, c: NativeHandle, body: () -> R): R {
    var x = a
    var y = b
    var z = c
    if ((x.ptr and -2L) > (y.ptr and -2L)) { val t = x; x = y; y = t }
    if ((y.ptr and -2L) > (z.ptr and -2L)) { val t = y; y = z; z = t }
    if ((x.ptr and -2L) > (y.ptr and -2L)) { val t = x; x = y; y = t }
    return synchronized(x) { synchronized(y) { synchronized(z) { body() } } }
}

/** The binding-error channel: a native call that cannot complete reports here. */
public fun interface JniErrorHandler<out R> {
    public fun run(je: String?): R
}

internal class JniErrorHandlerCapture : JniErrorHandler<Unit> {
    @JvmField var failed: Boolean = false
    @JvmField var ze0: String? = null
    override fun run(je: String?) { failed = true; this.ze0 = je }
    companion object {
        private val TL: ThreadLocal<JniErrorHandlerCapture> = ThreadLocal.withInitial { JniErrorHandlerCapture() }
        @JvmStatic fun acquire(): JniErrorHandlerCapture {
            val c = TL.get()
            c.failed = false; c.ze0 = null
            return c
        }
    }
}
"#;

impl Gen<'_> {
    pub(crate) fn kotlin_prelude(&mut self) {
        let pkg = self.base_pkg.clone();
        self.kt_push(&pkg, PRELUDE.to_string());
    }

    /// Bind a member function of a class.
    fn member(&mut self, c: &Rc<Class>, f: &crate::decl::FunctionDecl, ctor: bool) -> Res<String> {
        let (func, callee) = self.resolve_fn(&f.fun)?;
        let camel = names::camel(&names::bare(&func.name));
        let kt_name = match (&f.name, &self.b.method_hook) {
            (Some(n), _) => n.clone(),
            (None, Some(h)) => h(&c.pkg, &c.name, &camel),
            (None, None) => camel,
        };
        let placement = if ctor {
            Placement::Constructor(c.clone())
        } else {
            Placement::Method(c.clone())
        };
        let bound = self.bind(f.clone(), func, callee, placement, kt_name)?;
        let line = format!(
            "- `{}.{}` ← `{}`",
            c.name,
            bound.kt_name,
            names::bare(&bound.func.name)
        );
        self.report.push((c.pkg.clone(), line));
        self.function(&bound)
    }

    /// Emit one class.
    pub(crate) fn class(&mut self, c: &Rc<Class>) -> Res<()> {
        let mut methods = Vec::new();
        for m in &c.methods.clone() {
            methods.push(self.member(c, m, false)?);
        }
        let mut ctors = Vec::new();
        for m in &c.constructors.clone() {
            ctors.push(self.member(c, m, true)?);
        }
        let line = format!("- class `{}` ({})", c.name, kind_label(&c.kind));
        self.report.push((c.pkg.clone(), line));
        let text = match &c.kind {
            ClassKind::Ptr { gc } => {
                self.rust_free_ptr(c);
                self.ptr_class(c, *gc, &methods, &ctors)
            }
            ClassKind::Data { .. } => self.data_class(c, &methods, &ctors)?,
            ClassKind::Enum => self.enum_class(c)?,
            ClassKind::Sealed { .. } => self.sealed_class(c)?,
        };
        let pkg = c.pkg.clone();
        self.kt_push(&pkg, text);
        Ok(())
    }

    fn supertypes(&self, c: &Class, base: &[String]) -> String {
        let mut all: Vec<String> = base.to_vec();
        if let Some(i) = &c.iface {
            all.push(i.clone());
        }
        all.extend(c.implements.iter().cloned());
        if all.is_empty() {
            String::new()
        } else {
            format!(" : {}", all.join(", "))
        }
    }

    fn iface_decl(&self, c: &Class, extends: &str, members: Vec<String>) -> String {
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

    fn ptr_class(
        &mut self,
        c: &Rc<Class>,
        gc: bool,
        methods: &[String],
        ctors: &[String],
    ) -> String {
        let name = &c.name;
        let base = self.base_pkg.clone();
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
        members.extend(self.iface_members.get(&c.rust).cloned().unwrap_or_default());
        let iface = self.iface_decl(c, " : AutoCloseable", members);
        let methods = methods
            .iter()
            .map(|m| indent(m, 4))
            .collect::<Vec<_>>()
            .join("\n\n");
        let ctors = ctors
            .iter()
            .map(|m| indent(m, 8))
            .collect::<Vec<_>>()
            .join("\n\n");
        format!(
            "{iface}public class {name}(initialPtr: Long){} {{\n{lifecycle}\n{methods}\n\n    public companion object {{\n        @JvmStatic\n        external fun freePtr(ptr: Long)\n\n{ctors}\n    }}\n}}\n",
            self.supertypes(c, &[parent])
        )
    }

    fn data_class(&mut self, c: &Rc<Class>, methods: &[String], ctors: &[String]) -> Res<String> {
        let Some(FlatType::Struct(s)) = self.flat.declared_type(&c.rust) else {
            return err(format!("`{}` is not a struct", c.rust));
        };
        let over = if c.iface.is_some() { "override " } else { "" };
        let mut props = Vec::new();
        let mut iface_members = Vec::new();
        let mut args = Vec::new();
        let mut closes = Vec::new();
        let mut arrays = Vec::new();
        for f in &s.fields {
            let p = kt_prop(f);
            let t = self.kt_type(&f.ty)?;
            props.push(format!("{over}val {p}: {t}"));
            iface_members.push(format!("val {p}: {t}"));
            args.push(format!("{p}: {t}"));
            if let Some(cl) = self.kt_close(&f.ty, &p)? {
                closes.push(cl);
            }
            arrays.push((p, self.array_eq(&f.ty)?));
        }
        let name = &c.name;
        let owns = !closes.is_empty();
        let mut supers = Vec::new();
        if owns {
            supers.push("AutoCloseable".to_string());
        }
        iface_members.extend(self.iface_members.get(&c.rust).cloned().unwrap_or_default());
        let iface = self.iface_decl(c, "", iface_members);
        let mut body = Vec::new();
        if owns {
            body.push(format!(
                "    override fun close() {{\n        {}\n    }}",
                closes.join("\n        ")
            ));
        }
        if arrays.iter().any(|(_, a)| a.is_some()) {
            body.push(self.array_members(name, &arrays));
        }
        body.extend(methods.iter().map(|m| indent(m, 4)));
        let names_list = s.fields.iter().map(kt_prop).collect::<Vec<_>>().join(", ");
        let from_parts = format!(
            "        @JvmStatic\n        public fun fromParts({}): {name} = {name}({names_list})",
            args.join(", ")
        );
        let ctors = ctors.iter().map(|m| indent(m, 8)).collect::<Vec<_>>();
        let companion = std::iter::once(from_parts)
            .chain(ctors)
            .collect::<Vec<_>>()
            .join("\n\n");
        Ok(format!(
            "{iface}public data class {name}({}){} {{\n{}\n\n    public companion object {{\n{companion}\n    }}\n}}\n",
            props.join(", "),
            self.supertypes(c, &supers),
            body.join("\n\n")
        ))
    }

    /// How a field compares when it is array-backed: `Some((eq, hash, show))`
    /// templates over `$a` / `$b`, or `None` for ordinary equality.
    fn array_eq(&self, ty: &prebindgen_tools::flat::flat::TypeRef) -> Res<Option<ArrayEq>> {
        Ok(match self.kind(ty)? {
            Kind::Bytes(_) | Kind::Array { .. } => Some((
                "$a.contentEquals($b)".to_string(),
                "$a.contentHashCode()".to_string(),
                "${$a.contentToString()}".to_string(),
            )),
            Kind::Seq { elem, .. } => self.array_eq(elem)?.map(|(eq, hash, show)| {
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
            Kind::Option(inner) => self.array_eq(inner)?.map(|(eq, hash, show)| {
                (
                    format!("(if ($a == null || $b == null) $a === $b else {})", eq.replace("$a", "$a!!").replace("$b", "$b!!")),
                    format!("($a?.let {{ {} }} ?: 0)", hash.replace("$a", "it")),
                    show,
                )
            }),
            _ => None,
        })
    }

    fn array_members(&self, name: &str, fields: &[(String, Option<ArrayEq>)]) -> String {
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

    fn enum_class(&mut self, c: &Rc<Class>) -> Res<String> {
        let values = self.enum_arms(c)?;
        let over = if c.iface.is_some() { "override " } else { "" };
        let entries = values
            .iter()
            .map(|(n, d)| format!("    {}({d})", upper_snake(&names::bare(n))))
            .collect::<Vec<_>>()
            .join(",\n");
        let iface = self.iface_decl(c, "", vec!["val value: Int".to_string()]);
        let name = &c.name;
        Ok(format!(
            "{iface}public enum class {name}(public {over}val value: Int){} {{\n{entries};\n\n    public companion object {{\n        @JvmStatic\n        public fun fromInt(value: Int): {name} = entries.first {{ it.value == value }}\n    }}\n}}\n",
            self.supertypes(c, &[])
        ))
    }

    fn sealed_class(&mut self, c: &Rc<Class>) -> Res<String> {
        let Some(FlatType::Variant(v)) = self.flat.declared_type(&c.rust) else {
            return err(format!("`{}` is not a data-carrying enum", c.rust));
        };
        let name = &c.name;
        let fqn = c.fqn();
        let mut owns = false;
        for a in &v.alternatives {
            for f in &a.fields {
                owns |= self.owns_handle(&f.ty)?;
            }
        }
        let mut variants = Vec::new();
        let mut from_params = vec!["tag: Int".to_string()];
        let mut arms = Vec::new();
        let mut companion_clash = false;
        for (i, alt) in v.alternatives.iter().enumerate() {
            let vname = alt_kt_name(c, alt);
            companion_clash |= vname == "Companion";
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
            if alt.fields.is_empty() {
                if owns {
                    variants.push(format!(
                        "    public data object {vname} : {name} {{\n{}\n    }}",
                        close_body(Vec::new())
                    ));
                } else {
                    variants.push(format!("    public data object {vname} : {name}"));
                }
                arms.push(format!("{i} -> {vname}"));
                continue;
            }
            let mut props = Vec::new();
            let mut closes = Vec::new();
            let mut args = Vec::new();
            for f in &alt.fields {
                let p = kt_prop(f);
                let t = self.kt_type(&f.ty)?;
                props.push(format!("public val {p}: {t}"));
                if let Some(cl) = self.kt_close(&f.ty, &p)? {
                    closes.push(cl);
                }
                let fp = format!(
                    "{}_{}",
                    names::snake(&names::bare(&alt.name)),
                    super::codec::field_seg(f)
                );
                from_params.push(format!("{fp}: {t}"));
                args.push(fp);
            }
            let body = if owns {
                format!(" {{\n{}\n    }}", close_body(closes))
            } else {
                String::new()
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
        let companion = if companion_clash {
            "companion object Companion_"
        } else {
            "companion object"
        };
        let mut supers = Vec::new();
        if owns {
            supers.push("AutoCloseable".to_string());
        }
        let _ = fqn;
        Ok(format!(
            "public sealed interface {name}{} {{\n{}\n\n    public {companion} {{\n        @JvmStatic\n        public fun fromParts({}): {name} =\n            when (tag) {{\n                {}\n            }}\n    }}\n}}\n",
            self.supertypes(c, &supers),
            variants.join("\n\n"),
            from_params.join(", "),
            arms.join("\n                ")
        ))
    }

    // ── constants ───────────────────────────────────────────────────────

    /// A constant: a nullary getter extern and a lazily initialized `val`.
    pub(crate) fn const_getter(&mut self, pkg: &str, c: &ConstDecl) -> Res<String> {
        let vname = names::bare(&c.name);
        let getter = format!("constGet{}", names::pascal(&vname.to_lowercase()));
        let (func, callee, kt_fn): (Function, proc_macro2::TokenStream, String) = match &c.source {
            ConstSource::Const => {
                let k = self.flat.constant(&c.name).ok_or_else(|| {
                    crate::Error(format!("`{vname}` is not a #[prebindgen] const"))
                })?;
                let f = Function::synthetic_getter(names::ident(&getter), k.ty.clone());
                let path = self.q.path(&c.name);
                (f, quote!(#path), getter.clone())
            }
            ConstSource::Fun(fd) => {
                let (f, callee) = self.resolve_fn(&fd.fun)?;
                let n = names::camel(&names::bare(&f.name));
                (f, quote!(#callee()), n)
            }
            ConstSource::With(ty, path) => {
                let t = self
                    .flat
                    .classify(ty)
                    .map_err(|e| crate::Error(e.to_string()))?;
                let f = Function::synthetic_getter(names::ident(&getter), t);
                (f, quote!(#path()), getter.clone())
            }
            ConstSource::Expr(ty, expr) => {
                let t = self
                    .flat
                    .classify(ty)
                    .map_err(|e| crate::Error(e.to_string()))?;
                let f = Function::synthetic_getter(names::ident(&getter), t);
                let mut modules: Vec<String> = self
                    .flat
                    .elements()
                    .filter_map(|e| e.location().crate_name.clone())
                    .map(|c| c.replace('-', "_"))
                    .collect();
                modules.sort();
                modules.dedup();
                let uses = modules.iter().map(|m| {
                    let m: syn::Path = syn::parse_str(m).expect("a crate name");
                    quote!(#[allow(unused_imports)] use #m::*;)
                });
                (f, quote!({ #(#uses)* #expr }), getter.clone())
            }
        };
        let mut decl = crate::decl::FunctionDecl::new(syn::parse_quote!(__const));
        decl.name = Some(kt_fn.clone());
        let call = callee.clone();
        let bound = self.bind(
            decl,
            func.clone(),
            quote!(__unused),
            Placement::Package,
            kt_fn.clone(),
        )?;
        let bound = Bound {
            callee: quote!(#call),
            ..bound
        };
        // The getter's value is the whole call expression, not a call of it.
        let text = self.function_valued(&bound)?;
        let ty = self.kt_type(&func.ret)?;
        self.report
            .push((pkg.to_string(), format!("- `val {vname}: {ty}`")));
        let private = text.replacen("public fun", "private fun", 1);
        Ok(format!(
            "{private}\npublic val {vname}: {ty} by lazy {{ {kt_fn}({}.JniErrorHandler {{ je -> error(je ?: \"const {vname}: JNI getter failed\") }}) }}\n",
            self.base_pkg
        ))
    }

    /// Like [`Gen::function`], for a getter whose callee is an expression
    /// producing the value rather than a function to call.
    fn function_valued(&mut self, b: &Bound) -> Res<String> {
        self.valued = true;
        let r = self.function(b);
        self.valued = false;
        r
    }

    // ── files ───────────────────────────────────────────────────────────

    pub(crate) fn kt_files(&mut self) -> Vec<(String, String)> {
        // The native-method holder.
        let init = self
            .b
            .native_init
            .as_ref()
            .map(|s| format!("    init {{\n        {s}\n    }}\n\n"))
            .unwrap_or_default();
        let holder = format!(
            "internal object {} {{\n{init}{}\n}}\n",
            self.harness,
            self.externs.join("\n\n")
        );
        let pkg = self.base_pkg.clone();
        self.kt_push(&pkg, holder);
        let mut files = Vec::new();
        for (pkg, decls) in &self.kt {
            let text = format!(
                "// Auto-generated by prebindgen-jni — do not edit by hand.\n@file:Suppress(\"RedundantVisibilityModifier\", \"unused\", \"UNCHECKED_CAST\", \"FunctionName\", \"ClassName\", \"PropertyName\", \"LocalVariableName\", \"RemoveRedundantQualifierName\")\n\npackage {pkg}\n\n{}",
                decls.join("\n")
            );
            files.push((pkg.clone(), text));
        }
        files
    }
}

fn kind_label(k: &ClassKind) -> &'static str {
    match k {
        ClassKind::Ptr { gc: false } => "handle",
        ClassKind::Ptr { gc: true } => "gc-managed handle",
        ClassKind::Data { .. } => "data class",
        ClassKind::Enum => "enum class",
        ClassKind::Sealed { .. } => "sealed interface",
    }
}

fn upper_snake(s: &str) -> String {
    names::snake(s).to_uppercase()
}

#[allow(dead_code)]
fn unused(_: FnRef, _: TypeKind, _: Dir) {}
