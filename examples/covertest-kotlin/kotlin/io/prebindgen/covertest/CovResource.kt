package io.prebindgen.covertest

/**
 * Hand-written consumer interface for the `.implements(...)` acceptance test
 * (prebindgen#54): a generated handle class joins an existing SDK hierarchy
 * without hand-editing generated code. Its abstract members are satisfied by
 * the generated surface two ways — `peek()`/`isClosed()` by `NativeHandle`'s
 * inherited public members (no marker needed), `len(...)` by the class-body
 * method generated from `.fun(fun!(storage_len).overrides())` (Kotlin
 * requires the `override` modifier on class-body members, which the marker
 * emits). The default members show interface-injected behavior over both.
 */
interface CovResource {
    fun peek(): Long

    fun isClosed(): Boolean

    /** Satisfied by the generated class-body method (`.overrides()`). */
    fun len(onError: JniErrorHandler<Long>): Long

    /** Interface-provided behavior over the inherited members. */
    fun isLive(): Boolean = !isClosed() && peek() != 0L

    /** Interface-provided behavior over a generated class-specific method. */
    fun isEmpty(): Boolean = len(JniErrorHandler { je -> error(je ?: "len failed") }) == 0L
}
