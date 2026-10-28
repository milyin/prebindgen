package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference
import java.util.jar.JarFile

private fun rowHandles(rows: List<CallbackRow>) = rows.flatMap { listOf(it.first, it.second) }

// Find the compiled bridge by its enclosing source function and raw Array
// signature, without depending on generated native ordinals or class numbers.
private fun rowBridge(function: String, callback: Any, arity: Int = 1): Any {
    val prefix = "io.prebindgen.covertest.callbacks.FunctionsKt\$$function\$"
    val jar = File(CallbackRow::class.java.protectionDomain.codeSource.location.toURI())
    JarFile(jar).use { archive ->
        for (entry in archive.entries()) {
            val name = entry.name.removeSuffix(".class").replace('/', '.')
            if (!entry.name.endsWith(".class") || !name.startsWith(prefix)) continue
            val type = Class.forName(name, false, CallbackRow::class.java.classLoader)
            val run = type.declaredMethods.singleOrNull {
                it.name == "run" && it.parameterCount == arity && it.parameterTypes.all { p -> p.isArray }
            } ?: continue
            val constructor = type.declaredConstructors.singleOrNull {
                it.parameterCount == 1 && it.parameterTypes.single().isInstance(callback)
            } ?: continue
            check(run.returnType == Void.TYPE)
            constructor.isAccessible = true
            return constructor.newInstance(callback)
        }
    }
    error("missing callback bridge for $function")
}

fun checkCallbackHandleCollections() {
    check(callbackRowResourceLive() == 0L)
    val failure = AtomicReference<Throwable?>()
    fun inside(body: () -> Unit) {
        try { body() } catch (t: Throwable) { failure.compareAndSet(null, t) }
    }
    var delivered: List<CallbackRow> = emptyList()
    var retained: CallbackRowResource? = null
    var calls = 0
    callbackRowsOwned({ rows -> inside {
        calls++
        check(rows.map { it.label } == listOf("row0", "row1", "row2"))
        check(rowHandles(rows).map(::callbackRowResourceId) == listOf(1L, 2L, 11L, 12L, 21L, 22L))
        retained = rows[1].second.take()
        delivered = rows
    } })
    check(calls == 1 && rowHandles(delivered).all { it.isClosed })
    check(callbackRowResourceLive() == 1L && callbackRowResourceId(retained) == 12L)
    retained!!.close()
    check(callbackRowResourceLive() == 0L)
    callbackRowsBorrowed({ rows -> inside {
        calls++
        check(rowHandles(rows).map(::callbackRowResourceId) == listOf(1L, 2L, 11L, 12L, 21L, 22L))
        delivered = rows
    } })
    check(calls == 2 && rowHandles(delivered).all { it.isClosed } && callbackRowResourceLive() == 0L)
    var throwingCalls = 0
    callbackRowsOwned({ rows ->
        inside { throwingCalls++; delivered = rows }
        throw IllegalStateException("expected row callback failure")
    })
    check(throwingCalls == 1 && rowHandles(delivered).all { it.isClosed } && callbackRowResourceLive() == 0L)
    for (mode in 0..2) {
        var optionalCalls = 0
        callbackRowsOptional({ rows -> inside {
            optionalCalls++
            when (mode) {
                0 -> check(rows == null)
                1 -> check(rows?.isEmpty() == true)
                else -> {
                    check(rows?.size == 3 && rows[1] == null)
                    check(callbackRowResourceId(rows!![2]!!.second) == 12L)
                }
            }
        } }, mode)
        check(optionalCalls == 1 && callbackRowResourceLive() == 0L)
    }
    var nestedHandles: List<CallbackRowResource> = emptyList()
    callbackRowsNested({ groups -> inside {
        calls++
        check(groups.map { it.size } == listOf(2, 0, 2) && groups[0][1] == null)
        val rows = groups.flatten().filterNotNull()
        check(rows[1].detail == null && rows[2].detail?.resource == null)
        nestedHandles = rows.flatMap { rowHandles(listOf(it.row)) + listOfNotNull(it.detail?.resource) }
        check(nestedHandles.size == 7 && callbackRowResourceId(nestedHandles[2]) == 90L)
    } })
    check(calls == 3 && nestedHandles.all { it.isClosed } && callbackRowResourceLive() == 0L)
    callbackRowsPair({ first, second -> inside {
        calls++
        check(first.size == 1 && second.size == 2)
        check(callbackRowResourceId(first[0].first) == 11L && callbackRowResourceId(second[1].second) == 32L)
        delivered = first + second
    } })
    check(calls == 4 && rowHandles(delivered).all { it.isClosed } && callbackRowResourceLive() == 0L)
    var direct: List<CallbackRowResource> = emptyList()
    callbackRowHandles({ handles -> inside {
        calls++
        check(handles.map(::callbackRowResourceId) == listOf(71L, 72L))
        direct = handles
    } })
    check(calls == 5 && direct.all { it.isClosed } && callbackRowResourceLive() == 0L)
    var conversionCalls = 0
    callbackRowsConversionFailure({ rows -> inside {
        conversionCalls++
        check(rows.single().label == "row4")
    } })
    check(conversionCalls == 1 && callbackRowResourceLive() == 0L)

    // Owned rows move non-Clone resources by default. The enclosing collection
    // stays scoped unless that collection itself has an output expansion.
    var uniqueRows: List<CallbackUniqueRow> = emptyList()
    var uniqueOwner: CallbackUniqueResource? = null
    var uniqueCalls = 0
    callbackRowsUnique({ rows -> inside {
        uniqueCalls++
        check(rows.map { callbackUniqueResourceId(it.resource) } == listOf(101L, 102L))
        uniqueRows = rows
        uniqueOwner = rows[0].resource.take()
    } })
    check(uniqueCalls == 1 && uniqueRows.all { it.resource.isClosed })
    check(callbackRowResourceLive() == 1L && callbackUniqueResourceId(uniqueOwner) == 101L)
    uniqueOwner!!.close()
    check(callbackRowResourceLive() == 0L)

    val originalFailure = IllegalStateException("expected row delivery and cleanup failures")
    var cleanupCalls = 0
    callbackRowsUniqueError({ rows ->
        inside { cleanupCalls++; uniqueRows = rows }
        throw originalFailure
    })
    check(cleanupCalls == 1 && uniqueRows.all { it.resource.isClosed })
    check(callbackRowResourceLive() == 0L)
    check(originalFailure.suppressed.single() is IllegalArgumentException)

    // A borrowing output expansion still boxes owned clones. The source row
    // is dropped before the retained Kotlin handle is read or closed.
    var observed: List<CallbackObservedRow> = emptyList()
    var observedCalls = 0
    callbackRowsRef({ rows -> inside { observedCalls++; observed = rows } })
    check(observedCalls == 1 && callbackRowResourceLive() == 1L)
    check(callbackRowResourceId(observed.single().resource) == 111L)
    observed.single().resource.close()
    check(callbackRowResourceLive() == 0L)

    var transferred: List<CallbackTransferredRow> = emptyList()
    for (throws in listOf(false, true)) {
        var transferredCalls = 0
        callbackRowsTransferred({ rows ->
            inside { transferredCalls++; transferred = rows }
            if (throws) throw IllegalStateException("expected transferred row callback failure")
        })
        check(transferredCalls == 1 && callbackRowResourceLive() == 6L)
        check(transferred.flatMap { listOf(it.first, it.second) }.none { it.isClosed })
        transferred.forEach { it.first.close(); it.second.close() }
        check(callbackRowResourceLive() == 0L)
    }
    var mixedCalls = 0
    callbackRowsTransferredPair({ owned, scoped ->
        inside { mixedCalls++; transferred = owned; delivered = scoped }
        throw IllegalStateException("expected mixed row callback failure")
    })
    check(mixedCalls == 1 && callbackRowResourceLive() == 2L && rowHandles(delivered).all { it.isClosed })
    transferred.forEach { it.first.close(); it.second.close() }
    check(callbackRowResourceLive() == 0L)

    // A native probe enters the real generated bridge with a malformed middle
    // row: completed rows, the failed row and unread rows must all be reclaimed.
    var malformedCalls = 0
    val scopedBridge = rowBridge("callbackRowsOwned", CallbackRowVecCallback { malformedCalls++ })
    val transferredBridge = rowBridge("callbackRowsTransferred", CallbackTransferredRowVecCallback { malformedCalls++ })
    val pairBridge = rowBridge("callbackRowsPair", CallbackRowVecCallbackRowVecCallback { _, _ -> malformedCalls++ }, 2)
    val mixedBridge = rowBridge("callbackRowsTransferredPair", CallbackTransferredRowVecCallbackRowVecCallback { _, _ -> malformedCalls++ }, 2)
    for ((bridge, mode, drops) in listOf(
        Triple(scopedBridge, 0, 6L), Triple(transferredBridge, 0, 6L),
        Triple(pairBridge, 1, 12L), Triple(mixedBridge, 1, 12L),
        Triple(scopedBridge, 2, 4L), Triple(transferredBridge, 2, 4L),
        Triple(pairBridge, 3, 6L), Triple(mixedBridge, 3, 6L),
    )) {
        val before = callbackRowResourceDrops()
        check(CallbackRowsRuntimeProbe.malformed(bridge, mode) == 0L)
        check(callbackRowResourceDrops() - before == drops)
    }
    check(malformedCalls == 0)
    // Observe the actual JNI invocation result so the runtime's exception
    // containment cannot hide a spurious successful-delivery cleanup error.
    var validCalls = 0
    val validBridge = rowBridge("callbackRowsOwned", CallbackRowVecCallback { validCalls++ })
    check(CallbackRowsRuntimeProbe.malformed(validBridge, 4) == 0L && validCalls == 1)
    var absentCalls = 0
    val absentBridge = rowBridge("callbackRowsOptional", OptionalOptionalCallbackRowVecCallback { rows ->
        check(rows == null)
        absentCalls++
    })
    check(CallbackRowsRuntimeProbe.malformed(absentBridge, 5) == 0L && absentCalls == 1)
    // Reflection propagates any exception from the generated nested bridge,
    // including cleanup of absent optional rows and empty inner collections.
    var emptyCalls = 0
    val emptyBridge = rowBridge("callbackRowsNested", OptionalCallbackNestedRowVecVecCallback { groups ->
        check(groups == listOf(listOf(null), emptyList<CallbackNestedRow?>()))
        emptyCalls++
    })
    val runEmpty = emptyBridge.javaClass.getDeclaredMethod("run", Array<Any?>::class.java)
    runEmpty.isAccessible = true
    runEmpty.invoke(emptyBridge, arrayOf<Any?>(
        arrayOf<Any?>(arrayOf<Any?>(arrayOf<Any?>(0))),
        arrayOf<Any?>(emptyArray<Any?>()),
    ) as Any)
    check(emptyCalls == 1)
    val workers = AtomicInteger()
    callbackJobJoin(callbackRowsSpawn({ rows -> inside {
        check(rows.size == 2 && callbackRowResourceId(rows[1].second) == 12L)
        workers.incrementAndGet()
    } }, 500))
    check(workers.get() == 1000 && callbackRowResourceLive() == 0L)

    // Cleanup waits for an escaped wrapper in use on another Java thread.
    val borrowed = CountDownLatch(1)
    val released = CountDownLatch(1)
    val returned = CountDownLatch(1)
    val readers = java.util.Collections.synchronizedList(mutableListOf<Thread>())
    val started = AtomicInteger()
    val job = callbackRowsSpawn({ rows -> inside {
        if (started.getAndIncrement() == 0) {
            val handle = rows[0].first
            val reader = Thread { inside { callbackRowResourceHold(handle, { inside {
                borrowed.countDown()
                check(released.await(5, TimeUnit.SECONDS))
            } }) } }.also { it.start() }
            readers.add(reader)
            check(borrowed.await(5, TimeUnit.SECONDS))
            returned.countDown()
        }
    } }, 1)
    try {
        check(returned.await(5, TimeUnit.SECONDS))
        Thread.sleep(150)
        check(!callbackRowJobFinished(job))
    } finally { released.countDown() }
    callbackJobJoin(job)
    readers.forEach { it.join(5000); check(!it.isAlive) }
    check(started.get() == 2 && callbackRowResourceLive() == 0L)
    failure.get()?.let { throw it }
    println("callback handle collections PASS: records, slices, ownership, partial failures and native workers")
}

private object CallbackRowsRuntimeProbe {
    @JvmStatic external fun malformed(bridge: Any, mode: Int): Long
}
