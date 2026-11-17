package io.prebindgen.covertest

import io.prebindgen.covertest.returns.*
import io.prebindgen.covertest.primitives.*
import io.prebindgen.covertest.resources.ownedResourceDrops
import java.util.concurrent.atomic.AtomicReference

fun checkReturnTransport() {
    val start = ownedResourceDrops()
    // Reenter a shared result signature after the outer values were delivered,
    // while the outer Rust source destructor still runs.
    returnOwnership(111, false, {
        returnOwnership(222, false, {}) { value, resource ->
            check(value == 222L); resource.close()
        }
    }) { value, resource -> check(value == 111L); resource.close() }
    check(ownedResourceDrops() == start + 2uL)

    // Ownership reached the capture, but Rust source cleanup then failed.
    // The wrapper must reclaim the captured native handle before recovery.
    check(returnOwnership(333, true, {}, onError = JniErrorHandler {
        check(it == "Rust operation panicked"); 17L
    }) { _, resource -> resource.close(); error("unexpected success") } == 17L)
    check(ownedResourceDrops() == start + 3uL)

    val fallback = checkNotNull(io.prebindgen.covertest.options.optionResourceNew(true))
    val beforeSingle = ownedResourceDrops()
    check(returnSingleOwnership(0, true, {}, onError = JniErrorHandler {
        check(ownedResourceDrops() == beforeSingle + 1uL)
        fallback
    }) === fallback)
    check(!fallback.isClosed)
    fallback.close()

    // The error capture also survives source-error destructor reentry and panic.
    for (panic in listOf(false, true)) {
        val recovered = returnDomain(41, panic, {
            check(returnDomain(99, false, {}, onError = ReturnFailureHandler { code ->
                check(code == 99L); code + 1
            }) == 100L)
        }, onError = ReturnFailureHandler { code -> check(code == 41L); code + 1 })
        check(recovered == 42L)
    }
    val original = IllegalStateException("return handler identity")
    try {
        returnDomain(1, false, {}, onError = ReturnFailureHandler { throw original })
        error("handler did not throw")
    } catch (failure: IllegalStateException) { check(failure === original) }
    check(primitiveI64(Long.MIN_VALUE) == Long.MIN_VALUE)
    check(primitiveU64(1uL shl 63) == 1uL shl 63)
    for (bits in listOf(0x7ff8000000000001L, 0x7ff8fffffffffffeL, -2251799813685247L)) {
        val value = Double.fromBits(bits)
        check(primitiveF64(value).toRawBits() == bits)
    }
    for (bits in listOf(0x7fc00001, 0x7ffffffe, -4194303)) {
        val value = Float.fromBits(bits)
        check(primitiveF32(value).toRawBits() == bits)
    }

    val failure = AtomicReference<Throwable?>()
    val workers = (1..4).map { thread -> Thread {
        try { repeat(100) {
            val value = (thread * 1000 + it).toLong()
            check(returnDomain(value, false, {}, onError = ReturnFailureHandler { it }) == value)
            returnPair(value) { a, b -> check(a == value && b == value + 1) }
        } } catch (e: Throwable) { failure.set(e) }
    }.also { it.start() } }
    workers.forEach { it.join(10000); check(!it.isAlive) }
    failure.get()?.let { throw it }
    checkWideReturn()
    returnWideError(onError = ReturnWideErrorHandler { error ->
        check(error.a.v0 == 0L && error.a.v63 == 63L && error.b.v0 == 0L && error.b.v65 == 65L)
    })
    check(returnLimited(ReturnArgOuter(
        ReturnArgA(1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L),
        ReturnArgB(2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L, 2L)
    ), true) == 7L)
    println("return transport PASS: native primitives, typed success/error captures, reentry, late-drop cleanup and concurrency")
}

@Suppress("UNCHECKED_CAST")
private fun checkWideReturn() {
    // Kotlin's typed lambda bridge itself cannot have 130 Long parameters.
    // FunctionN uses Object[] only for this user-selected wide builder boundary.
    val build = object : kotlin.jvm.functions.FunctionN<Unit> {
        override val arity: Int get() = 130
        override fun invoke(vararg args: Any?) {
            check(args.size == 130 && args.all { it == 123L })
        }
    }
    returnWide(123, build = build as (Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long, Long) -> Unit)
}
