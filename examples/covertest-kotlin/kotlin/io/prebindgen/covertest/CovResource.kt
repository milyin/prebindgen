package io.prebindgen.covertest

/**
 * Hand-written consumer interface for the `.implements(...)` acceptance test
 * (prebindgen#54): a generated handle class joins an existing SDK hierarchy
 * without hand-editing generated code. Its abstract members are satisfied by
 * the generated surface (`NativeHandle`'s public `peek()`/`isClosed()`), and
 * the default member shows interface-injected behavior on top of it.
 */
interface CovResource {
    fun peek(): Long

    fun isClosed(): Boolean

    /** Interface-provided behavior over the generated members. */
    fun isLive(): Boolean = !isClosed() && peek() != 0L
}
