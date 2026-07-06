package io.prebindgen.perftest

import io.prebindgen.perftest.storage.payloadHandlerNew
import io.prebindgen.perftest.storage.payloadVecHandlerNew
import io.prebindgen.perftest.storage.storageCallback
import io.prebindgen.perftest.storage.storageCallbackVec
import io.prebindgen.perftest.storage.storageGet
import io.prebindgen.perftest.storage.storageGetVec
import io.prebindgen.perftest.storage.storageNew
import io.prebindgen.perftest.storage.storagePutByTake
import io.prebindgen.perftest.storage.storagePutSlice

/**
 * JVM micro-benchmark over the prebindgen-generated `perftest` JNI bindings.
 *
 * Mirrors `perftest-flat/examples/perftest.rs` (native Rust) and
 * `perftest-c/c/perftest.c` (C ABI) one-for-one. Single-payload ops — `put`/`get`/
 * `callback` — move a **whole** `Payload` across the boundary through an opaque `Storage`
 * handle; `storageGet` returns a nullable `Payload?` (null when the storage is empty).
 *
 * Vector ops — `put_vec`/`get_vec`/`callback_vec` — move a whole **batch** of `VEC_N`
 * payloads per call: `storagePutSlice(List<Payload>)`, `storageGetVec(): List<Payload>?`,
 * and `storageCallbackVec` (one upcall delivering the whole `List<Payload>` to
 * `PayloadListCallback.run`). They run `N / VEC_N` iterations (≈ the single-op `N` of
 * element work) and report **ns per call**, so comparing to `VEC_N ×` the single-op number
 * shows how the JNI per-call overhead amortizes over a batch.
 *
 * Note on the numbers: JNI `get`/`callback` are intrinsically slower than Rust/C because
 * they cross in the native→JVM *upcall* direction (reassembling values on the Kotlin
 * side) — that asymmetry, not the codegen, is the floor.
 */
// Iterations per measured single-op variant. Overridable (so the shared
// `perftest-bench.sh` harness can run all three languages at one N + a fast smoke): the
// `-Dperftest.n` system property (set by the Gradle `run` task from `-PperftestN`), else
// the `PERFTEST_N` env var, else the default.
private val N: Long =
    System.getProperty("perftest.n")?.toLongOrNull()
        ?: System.getenv("PERFTEST_N")?.toLongOrNull()
        ?: 5_000_000L

// Batch size for the vector ops (`-Dperftest.vec.n` ← `-PperftestVecN`, else
// `PERFTEST_VEC_N`, else 16).
private val VEC_N: Int =
    System.getProperty("perftest.vec.n")?.toIntOrNull()
        ?: System.getenv("PERFTEST_VEC_N")?.toIntOrNull()
        ?: 16

// Vector-op call count: N / VEC_N (≈ N elements of work), at least 1.
private val VEC_ITERS: Long = (N / VEC_N).coerceAtLeast(1L)

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
    println("%-12s %-16s %9.2f %9.1f".format(op, variant, nsPerOp, mops))
}

// A vector-op row: runs VEC_ITERS calls (each processing VEC_N payloads) and reports
// ns PER CALL.
private fun benchVec(op: String, variant: String, body: () -> Unit) {
    val start = System.nanoTime()
    for (i in 0 until VEC_ITERS) {
        body()
    }
    val elapsed = (System.nanoTime() - start).toDouble()
    val nsPerOp = elapsed / VEC_ITERS
    val mops = VEC_ITERS.toDouble() / (elapsed / 1.0e9) / 1.0e6
    println("%-12s %-16s %9.2f %9.1f".format(op, variant, nsPerOp, mops))
}

private fun makeBatch(label: String?): List<Payload> =
    (0 until VEC_N).map { Payload(it.toLong(), it, it.toDouble(), it % 2 == 0, label) }

// Verify the contract before benchmarking (parity with C running `correctness` first):
// store/get a whole batch, the nullable single get, the empty-storage None case, and the
// whole-batch callback. `Payload` is a data class, so `==` compares fields.
private fun correctness(s: Storage) {
    val batch = listOf(
        Payload(10L, 1, 10.0, false, "a"),
        Payload(20L, 2, 20.0, true, null),
        Payload(30L, 3, 30.0, false, "ccc"),
    )
    storagePutSlice(s, batch, onError)
    check(storageGetVec(s, onError) == batch) { "storageGetVec round-trip mismatch" }
    check(storageGet(s, onError) == batch[0]) { "storageGet should return the first payload" }

    // Whole-batch callback: ONE upcall delivering the entire List<Payload>.
    var cbSum = 0L
    val vh = payloadVecHandlerNew(PayloadListCallback { list -> cbSum += list.sumOf { it.id } }, onError)
    storageCallbackVec(s, vh, onError)
    check(cbSum == batch.sumOf { it.id }) { "callback_vec should observe the whole batch" }
    vh.close()

    // Empty contract: an empty slice clears the storage ⇒ every get reports absence.
    storagePutSlice(s, emptyList(), onError)
    check(storageGetVec(s, onError) == null) { "empty slice should clear to None" }
    check(storageGet(s, onError) == null) { "get on empty should be null" }

    System.err.println("correctness: array round-trip + vec callback OK")
}

fun main() {
    val s = storageNew(onError)

    correctness(s)

    var sink = 0L

    // Single-payload callback handler, prepared ONCE (trampoline built a single time).
    val cb = payloadHandlerNew(PayloadCallback { p -> sink += p.id }, onError)
    // Whole-batch callback handler, prepared once: one upcall delivers the whole List.
    val vcb = payloadVecHandlerNew(PayloadListCallback { list -> sink += list.size.toLong() }, onError)

    // Single-payload ops for one string category (`str` = heap `label`, `null` = none).
    fun runCategory(label: String?, cat: String) {
        val seed = Payload(42L, 7, 3.5, true, label)
        storagePutByTake(s, seed, onError) // seed so get/callback read this category

        bench("put", "native.$cat", N) {
            storagePutByTake(s, seed, onError)
        }
        bench("get", "native.$cat", N) {
            val g = storageGet(s, onError)!! // seeded ⇒ present
            sink += g.id
        }
        bench("callback", "native.$cat", N) {
            storageCallback(s, cb, onError)
        }
    }

    // Vector ops for one string category — a whole VEC_N batch per call.
    fun runVecCategory(label: String?, cat: String) {
        val batch = makeBatch(label)
        benchVec("put_vec", "batch.$cat") {
            storagePutSlice(s, batch, onError)
        }
        storagePutSlice(s, batch, onError) // seed for get/callback
        benchVec("get_vec", "batch.$cat") {
            val out = storageGetVec(s, onError)!! // seeded ⇒ present
            sink += out.size.toLong()
        }
        benchVec("callback_vec", "batch.$cat") {
            storageCallbackVec(s, vcb, onError)
        }
    }

    // Warm up the JIT on all paths so steady-state numbers are fair (capped for a small
    // `N` smoke run).
    val warm = makeBatch("hello, payload")
    storagePutSlice(s, warm, onError)
    val warmN = minOf(VEC_ITERS, 50_000L)
    repeat(warmN.toInt()) {
        storagePutByTake(s, warm[0], onError)
        storageGet(s, onError)
        storageCallback(s, cb, onError)
        storagePutSlice(s, warm, onError)
        storageGetVec(s, onError)
        storageCallbackVec(s, vcb, onError)
    }

    println("BEGIN_PERFTEST lang=kotlin n=$N")
    runCategory("hello, payload", "str")
    runCategory(null, "null")
    runVecCategory("hello, payload", "str")
    runVecCategory(null, "null")
    println("END_PERFTEST")

    cb.close()
    vcb.close()
    s.close()
    println("(sink = $sink)")
}
