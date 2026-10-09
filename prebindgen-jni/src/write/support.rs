//! The Kotlin interfaces bindings share, and the runtime prelude.

use prebindgen_flat::flat::TypeKind;

use crate::{
    lower::{
        deliver::{raw_name, DParam},
        leaf::Leaf,
        Dir,
    },
    plan::{Plan, Res, Support},
    write::check_slots,
};

/// Kotlin's own runtime for the generated code: the handle base classes,
/// lock ordering, and the binding-error channel.
pub(crate) const PRELUDE: &str = r#"/**
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

/// One support interface: its package and its text.
pub(crate) fn support(plan: &Plan, s: &Support) -> Res<(String, String)> {
    let base = plan.base_pkg.clone();
    Ok(match s {
        Support::Callback(ty) => {
            let fqn = plan.callback_fqn(ty)?;
            let pkg = fqn.rsplit_once('.').map_or(base, |(p, _)| p.to_string());
            (pkg, callback(plan, ty)?)
        }
        Support::Sink { ty, base: b } => {
            let leaves = plan.leaves(ty, Dir::Out)?;
            check_slots(&format!("`__Sink_{b}.run`"), &raw_types(&leaves), true)?;
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            let decode = plan.kt_decode(ty, &raws, false, 0)?;
            (
                base,
                format!(
                    "public fun interface __Sink_{b} {{\n    public fun run({}): Any?\n}}\n\n\
                     internal val __sink_{b}: __Sink_{b} = __Sink_{b} {{ {} ->\n    {decode}\n}}\n",
                    raw_sig(&leaves),
                    raws.join(", ")
                ),
            )
        }
        Support::Builder {
            pkg,
            name,
            params,
            leaves,
        } => {
            check_slots(&format!("`{name}Raw.run`"), &raw_types(leaves), true)?;
            check_slots(&format!("`{name}.run`"), &kt_types(params), true)?;
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            let args = params
                .iter()
                .map(|p| p.decode.clone())
                .collect::<Vec<_>>()
                .join(", ");
            (
                pkg.clone(),
                format!(
                    "public fun interface {name}Raw<out R> {{\n    public fun run({}): R\n}}\n\n\
                     public fun interface {name}<out R> {{\n    public fun run({}): R\n\n\
                     \x20   public fun asRaw(): {name}Raw<R> =\n        {name}Raw<R> {{{}\n            run({args})\n        }}\n}}\n",
                    raw_sig(leaves),
                    sig(params),
                    lambda(&raws)
                ),
            )
        }
        Support::Folder {
            pkg,
            name,
            columns,
            params,
            leaves,
        } => {
            // The folder takes the accumulator before the fields; the
            // columns interface a count before one array per leaf.
            let mut typed = vec!["A".to_string()];
            typed.extend(kt_types(params));
            check_slots(&format!("`{name}.run`"), &typed, true)?;
            let mut cols = vec!["Int".to_string()];
            cols.extend(leaves.iter().map(|l| l.column().kt_raw()));
            check_slots(&format!("`{columns}.run`"), &cols, true)?;
            let cols = leaves
                .iter()
                .map(|l| format!("{}: {}", raw_name(l), l.column().kt_raw()))
                .collect::<Vec<_>>()
                .join(", ");
            let sep = if cols.is_empty() { "" } else { ", " };
            (
                pkg.clone(),
                format!(
                    "public fun interface {name}<A> {{\n    public fun run(acc: A, {}): A\n}}\n\n\
                     public fun interface {columns} {{\n    public fun run(n: Int{sep}{cols}): Any?\n}}\n",
                    sig(params)
                ),
            )
        }
        Support::ErrorHandler {
            pkg,
            name,
            raw,
            capture,
            params,
            leaves,
        } => {
            check_slots(&format!("`{raw}.run`"), &raw_types(leaves), true)?;
            check_slots(&format!("`{name}.run`"), &kt_types(params), true)?;
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            let fields = leaves
                .iter()
                .zip(&raws)
                .map(|(l, n)| {
                    let t = l.kt_raw();
                    let nt = if t.ends_with('?') { t } else { format!("{t}?") };
                    format!("    @JvmField var {n}: {nt} = null")
                })
                .collect::<Vec<_>>()
                .join("\n");
            let assigns = raws
                .iter()
                .map(|n| format!("this.{n} = {n}"))
                .collect::<Vec<_>>()
                .join("; ");
            let resets = raws
                .iter()
                .map(|n| format!("c.{n} = null"))
                .collect::<Vec<_>>()
                .join("; ");
            let raw_sig = raw_sig(leaves);
            (
                pkg.clone(),
                format!(
                    "public fun interface {name}<out R> {{\n    public fun run({}): R\n}}\n\n\
                     public fun interface {raw} {{\n    public fun run({raw_sig})\n}}\n\n\
                     internal class {capture} : {raw} {{\n    @JvmField var failed: Boolean = false\n{fields}\n\
                     \x20   override fun run({raw_sig}) {{ failed = true; {assigns} }}\n\
                     \x20   companion object {{\n\
                     \x20       private val TL: ThreadLocal<{capture}> = ThreadLocal.withInitial {{ {capture}() }}\n\
                     \x20       @JvmStatic fun acquire(): {capture} {{\n\
                     \x20           val c = TL.get()\n\
                     \x20           c.failed = false; {resets}\n\
                     \x20           return c\n\
                     \x20       }}\n\
                     \x20   }}\n}}\n",
                    sig(params)
                ),
            )
        }
    })
}

fn sig(params: &[DParam]) -> String {
    params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.kt))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The Kotlin types of raw leaf parameters.
fn raw_types(leaves: &[Leaf]) -> Vec<String> {
    leaves.iter().map(Leaf::kt_raw).collect()
}

/// The Kotlin types of delivered parameters.
fn kt_types<'p>(params: impl IntoIterator<Item = &'p DParam>) -> Vec<String> {
    params.into_iter().map(|p| p.kt.clone()).collect()
}

fn raw_sig(leaves: &[Leaf]) -> String {
    leaves
        .iter()
        .map(|l| format!("{}: {}", raw_name(l), l.kt_raw()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn lambda(raws: &[String]) -> String {
    if raws.is_empty() {
        String::new()
    } else {
        format!(" {} ->", raws.join(", "))
    }
}

/// A callback type's interfaces: the typed one the user implements, and
/// the raw one Rust calls, into which `asRaw()` adapts it.
fn callback(plan: &Plan, ty: &prebindgen_flat::flat::TypeRef) -> Res<String> {
    let TypeKind::Callback { args } = ty.kind() else {
        unreachable!("a callback type");
    };
    let fqn = plan.callback_fqn(ty)?;
    let name = fqn.rsplit('.').next().unwrap_or(&fqn);
    let raw = format!("{name}Raw");
    let deliveries = plan.callback_args(args)?;
    let params: Vec<&DParam> = deliveries.iter().flat_map(|d| &d.params).collect();
    let leaves: Vec<Leaf> = deliveries.iter().flat_map(|d| d.leaves.clone()).collect();
    check_slots(&format!("`{raw}.run`"), &raw_types(&leaves), true)?;
    check_slots(
        &format!("`{name}.run`"),
        &kt_types(params.iter().copied()),
        true,
    )?;
    let raws: Vec<String> = leaves.iter().map(raw_name).collect();
    let locals: Vec<String> = params
        .iter()
        .enumerate()
        .map(|(i, p)| format!("            val __p{i} = {}", p.decode))
        .collect();
    let call_args = (0..params.len())
        .map(|i| format!("__p{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let closes: Vec<String> = params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            p.close
                .as_ref()
                .map(|c| c.replacen(&p.name, &format!("__p{i}"), 1))
        })
        .collect();
    let body = if closes.is_empty() {
        format!("{}\n            run({call_args})", locals.join("\n"))
    } else {
        format!(
            "{}\n            try {{\n                run({call_args})\n            }} finally {{\n                {}\n            }}",
            locals.join("\n"),
            closes.join("\n                ")
        )
    };
    let sig = params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.kt))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "public fun interface {raw} {{\n    public fun run({})\n}}\n\n\
         public fun interface {name} {{\n    public fun run({sig})\n\n\
         \x20   public fun asRaw(): {raw} =\n        {raw} {{{}\n    {body}\n        }}\n}}\n",
        raw_sig(&leaves),
        lambda(&raws)
    ))
}
