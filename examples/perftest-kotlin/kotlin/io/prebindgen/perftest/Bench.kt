package io.prebindgen.perftest

import io.prebindgen.perftest.storage.storageCallback
import io.prebindgen.perftest.storage.storageGet
import io.prebindgen.perftest.storage.storageGetFlag
import io.prebindgen.perftest.storage.storageGetId
import io.prebindgen.perftest.storage.storageGetLabel
import io.prebindgen.perftest.storage.storageGetSeq
import io.prebindgen.perftest.storage.storageGetValue
import io.prebindgen.perftest.storage.storageNew
import io.prebindgen.perftest.storage.storagePutByTake

/**
 * JVM micro-benchmark over the prebindgen-generated `perftest` JNI bindings.
 *
 * Mirrors `perftest-flat/examples/perftest.rs` (native Rust) and
 * `perftest-c/c/perftest.c` (C ABI). All operations go through an opaque `Storage`
 * handle (a `NativeHandle` subclass), and exercise both the data-structure approach
 * and the foreign-side composition technique:
 *
 *   * **put** — `storagePutByTake(s, p)` passes ALL fields of the `Payload` data class in
 *     ONE downcall (the data-structure approach for input).
 *   * **get (composition)** — `storageGet(s)` returns a `Payload` composed on the
 *     Kotlin side via the generated `Payload.fromParts(...)` factory in a single
 *     native→JVM upcall.
 *   * **get (naive)** — fetch each field with a separate downcall (5 crossings),
 *     then build `Payload` in Kotlin.
 *   * **callback** — `storageCallback(s) { p -> … }` composes the borrowed `Payload`
 *     natively (the same `fromParts` factory as `get (composition)`) and delivers it
 *     as a whole `Payload` to a generated `PayloadCallback.run(Payload)`.
 *
 * Note on the result: for this FLAT, all-scalar struct the naive path is often
 * the faster one. A `storageGet` composition is a single native→JVM *upcall*
 * (`call_static_method` on `fromParts`, which resolves the class + method by name
 * each call), while each naive accessor is a cheap JVM→native *downcall*. The
 * composition technique pays off when the naive alternative would be many
 * EXPENSIVE crossings — e.g. nested fields delivered as opaque handles, each
 * needing its own call — not when the fields are a handful of cheap scalars. This
 * benchmark makes that trade-off measurable.
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
    val seed = Payload(42L, 7, 3.5, true, "hello, payload")
    val s = storageNew(onError)
    storagePutByTake(s, seed, onError) // seed the storage

    var sink = 0L

    // The borrowed `&Payload` is composed natively (via the same `fromParts` factory as
    // `storageGet`) and delivered to the callback as a whole `Payload` object in one
    // `run(Payload)` upcall. Hoisted (like the C `bench_callback` closure) so the
    // measurement isn't per-iteration lambda allocations.
    val cb = PayloadCallback { p -> sink += p.id }

    // Warm up the JIT on all paths so steady-state numbers are fair (capped so a small
    // `N` smoke run stays fast).
    repeat(minOf(N, 200_000L).toInt()) {
        storagePutByTake(s, seed, onError)
        storageGet(s, onError)
        storageGetId(s, onError)
        storageGetSeq(s, onError)
        storageGetValue(s, onError)
        storageGetFlag(s, onError)
        storageGetLabel(s, onError)
        storageCallback(s, cb, onError)
    }

    println("BEGIN_PERFTEST lang=kotlin n=$N")

    bench("put", "native", N) {
        storagePutByTake(s, seed, onError)
    }

    bench("get", "composition", N) {
        val g = storageGet(s, onError) // 1 JNI crossing, composed via fromParts
        sink += g.id
    }

    bench("get", "naive", N) {
        val g = Payload( // 5 JNI crossings, composed in Kotlin
            storageGetId(s, onError),
            storageGetSeq(s, onError),
            storageGetValue(s, onError),
            storageGetFlag(s, onError),
            storageGetLabel(s, onError),
        )
        sink += g.id
    }

    bench("callback", "whole", N) {
        storageCallback(s, cb, onError)
    }

    println("END_PERFTEST")

    s.close()
    println("(sink = $sink)")
}
