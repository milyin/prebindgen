package io.prebindgen.perftest

import io.prebindgen.perftest.storage.storageCallback
import io.prebindgen.perftest.storage.storageGet
import io.prebindgen.perftest.storage.storageNew
import io.prebindgen.perftest.storage.storagePutByTake

/**
 * JVM micro-benchmark over the prebindgen-generated `perftest` JNI bindings.
 *
 * Mirrors `perftest-flat/examples/perftest.rs` (native Rust) and
 * `perftest-c/c/perftest.c` (C ABI) one-for-one: `put`/`get`/`callback`, each moving a
 * **whole** `Payload` across the boundary (no special per-field accessors — the same
 * surface every language benchmarks). All operations go through an opaque `Storage`
 * handle (a `NativeHandle` subclass):
 *
 *   * **put** — `storagePutByTake(s, p)` passes the `Payload`'s fields as decoupled
 *     leaves in ONE downcall; Rust reassembles the struct.
 *   * **get** — `storageGet(s)` returns a whole `Payload`; its fields cross as leaves
 *     and are reassembled on the Kotlin side via the generated `Payload.fromParts(...)`
 *     factory (no Java object built on the Rust side).
 *   * **callback** — `storageCallback(s) { p -> … }` delivers the borrowed `Payload` as
 *     a whole object; its fields cross as leaves and a generated adapter reassembles
 *     them before invoking `PayloadCallback.run(Payload)`.
 *
 * Note on the numbers: the JNI `get`/`callback` are intrinsically slower than the Rust
 * and C equivalents because they cross the boundary in the native→JVM *upcall* direction
 * (delivering the leaves to the Kotlin reassembler), which is far costlier than a
 * JVM→native downcall. That asymmetry — not the codegen — is the floor; this benchmark
 * measures it honestly with the same whole-struct surface as Rust and C.
 */
// Iterations per measured variant. Overridable (so the shared `perftest-bench.sh`
// harness can run all three languages at one N + a fast smoke): the `-Dperftest.n`
// system property (set by the Gradle `run` task from `-PperftestN`), else the
// `PERFTEST_N` env var, else the default.
private val N: Long =
    System.getProperty("perftest.n")?.toLongOrNull()
        ?: System.getenv("PERFTEST_N")?.toLongOrNull()
        ?: 5_000_000L

private val onError = JniErrorHandler<Nothing> { je ->
    throw RuntimeException("native error: $je")
}

// One normalized result row: `<op> <variant> <ns_per_op> <mops>`.
private fun bench(op: String, variant: String, n: Long, body: () -> Unit) {
    val start = System.nanoTime()
    for (i in 0 until n) {
        body()
    }
    val elapsed = (System.nanoTime() - start).toDouble()
    val nsPerOp = elapsed / n
    val mops = n.toDouble() / (elapsed / 1.0e9) / 1.0e6
    println("%-10s %-16s %9.2f %9.1f".format(op, variant, nsPerOp, mops))
}

fun main() {
    val s = storageNew(onError)

    var sink = 0L

    // The borrowed `&Payload` is composed natively (via the same `fromParts` factory as
    // `storageGet`) and delivered to the callback as a whole `Payload` object in one
    // `run(Payload)` upcall. Hoisted (like the C `bench_callback` closure) so the
    // measurement isn't per-iteration lambda allocations. (A `.str` payload also encodes
    // the `label` String inside `fromParts`; a `.null` payload does not.)
    val cb = PayloadCallback { p -> sink += p.id }

    // Run put/get/callback for one string category (`str` = a heap `label`, `null` = no
    // `label`). Emits `<op> <variant>.<cat>` rows so the harness can compare like-for-like.
    fun runCategory(label: String?, cat: String) {
        val seed = Payload(42L, 7, 3.5, true, label)
        storagePutByTake(s, seed, onError) // seed so get/callback read this category

        bench("put", "native.$cat", N) {
            storagePutByTake(s, seed, onError)
        }
        bench("get", "native.$cat", N) {
            val g = storageGet(s, onError) // whole Payload, reassembled on the Kotlin side
            sink += g.id
        }
        bench("callback", "native.$cat", N) {
            storageCallback(s, cb, onError) // whole Payload delivered to the callback
        }
    }

    // Warm up the JIT on all paths so steady-state numbers are fair (capped so a small
    // `N` smoke run stays fast).
    val warm = Payload(42L, 7, 3.5, true, "hello, payload")
    storagePutByTake(s, warm, onError)
    repeat(minOf(N, 200_000L).toInt()) {
        storagePutByTake(s, warm, onError)
        storageGet(s, onError)
        storageCallback(s, cb, onError)
    }

    println("BEGIN_PERFTEST lang=kotlin n=$N")
    runCategory("hello, payload", "str")
    runCategory(null, "null")
    println("END_PERFTEST")

    s.close()
    println("(sink = $sink)")
}
