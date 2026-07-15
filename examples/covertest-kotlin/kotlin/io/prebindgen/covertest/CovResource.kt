package io.prebindgen.covertest

import io.prebindgen.covertest.model.Priority

/**
 * Hand-written SDK interfaces demonstrating the `.interface()` hatch
 * (prebindgen#54): each EXTENDS the generated `<Name>Api` interface, so its
 * default members call the class's real generated members with full compiler
 * checking — no hand-replicated signatures, no editing of generated code.
 */

/** Extends the generated `StorageApi` (a ptr class). */
interface CovResource : StorageApi {
    /** Default member over the class-specific generated `len(...)`. */
    fun isEmpty(): Boolean = len(JniErrorHandler { je -> error(je ?: "len failed") }) == 0L

    /** Default member over the inherited `NativeHandle` surface. */
    val live: Boolean
        get() = !isClosed() && peek() != 0L
}

/** Extends the generated `PayloadApi` (a data class) — uses its `seq` field. */
interface Timestamped : io.prebindgen.covertest.PayloadApi {
    /** Whether this payload carries a positive sequence number. */
    val fresh: Boolean
        get() = seq > 0
}

/**
 * The enum's generated interface is named `PriorityKind` (via
 * `.interface_name(...)`); this SDK extension adds ranking behavior over its
 * `value` property.
 */
interface Ranked : io.prebindgen.covertest.model.PriorityKind {
    fun outranks(other: Priority): Boolean = value > other.value
}
