@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.gc

public fun gcResourceNew(id: ULong, `value`: ULong, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.gc.GcResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.gc.GcResource = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_new_7ae6fd267af3001d(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.gc.GcResource? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.gc.GcResource? = null
        try {
            output0 = io.prebindgen.covertest.gc.GcResource(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun gcResourceValue(`value`: io.prebindgen.covertest.gc.GcResource?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`value`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0) {
        run { val nativeArgument0 = try { `value`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_value_20a9f8cccf5d47f5(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceConsume(`value`: io.prebindgen.covertest.gc.GcResource?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`value`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { `value`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_consume_8efd1138a357f171(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (true) __pb_consume(`value`!!) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (true) __pb_consume(`value`!!); handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceFail(`value`: io.prebindgen.covertest.gc.GcResource?, onBindingError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.i64Handler<ULong>): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`value`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { `value`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_fail_bbca11f56e815020(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (true) __pb_consume(`value`!!) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (true) __pb_consume(`value`!!); handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourcePair(first: io.prebindgen.covertest.gc.GcResource?, second: io.prebindgen.covertest.gc.GcResource?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(first) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (true) requireNotNull(second) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0, handle1) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { first?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { second?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_pair_5b8cb5677c43d3b0(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (true) __pb_consume(first!!); if (true) __pb_consume(second!!) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (true) __pb_consume(first!!); if (true) __pb_consume(second!!); handle0?.let { if (it.address in taken) it.markConsumed() }; handle1?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceDrops(id: ULong, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_drops_53900a369e1b88bb(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceThree(a: io.prebindgen.covertest.gc.GcResource?, b: io.prebindgen.covertest.gc.GcResource?, c: io.prebindgen.covertest.gc.GcResource?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(a) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (true) requireNotNull(b) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle2 = try { if (true) requireNotNull(c) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0, handle1, handle2) {
        run { val nativeArgument0 = try { a?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { b?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { c?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_three_43d8964dd265c043(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceConsumeRows(values: kotlin.collections.List<io.prebindgen.covertest.gc.GcResource?>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5 }) }, intArrayOf(2)) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_consume_rows_4fb308d2bb5df313(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceReadRows(values: kotlin.collections.List<io.prebindgen.covertest.gc.GcResource?>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w6 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w6 }) }, intArrayOf(1)) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_read_rows_855e8e95b1044388(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceEmit(id: ULong, `value`: ULong, callback: io.prebindgen.covertest.gc.GcResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.gc.GcResource? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.gc.GcResource
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.gc.GcResource(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_emit_c3aa9044644f4b9c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun gcResourceHold(`value`: io.prebindgen.covertest.gc.GcResource?, callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`value`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withGcHandleLocks(handle0) {
        run { val nativeArgument0 = try { `value`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_hold_c14426251e7e19b0(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceReturn(callback: io.prebindgen.covertest.VoidToGcResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: io.prebindgen.covertest.gc.GcResource? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  io.prebindgen.covertest.withHandleLocks(if (true) requireNotNull(callback_return_w1) { "null handle" } else null) { val callbackReturnWire = callback_return_w1?.raw ?: 0L
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
if (true) __pb_consume(callback_return_w1!!)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_return_6e25e73ee67bf26e(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun gcResourceEmitOwned(id: ULong, `value`: ULong, callback: io.prebindgen.covertest.gc.GcOwnedCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.gc.GcResource? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.gc.GcResource
        try {
            callbackOutput0_0 = io.prebindgen.covertest.gc.GcResource(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_emit_owned_5c69b3edc59eb5a1(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun gcResourceRows(id: ULong, count: ULong, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.gc.GcResource>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.gc.GcResource> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { count.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_rows_cdbbaa46996fd3a0(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.gc.GcResource>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff2ee5d56b0351a9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.gc.GcResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.gc.GcResource>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff2ee5d56b0351a9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size
        var outputbulkItem_0: io.prebindgen.covertest.gc.GcResource? = null

            outputbulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.gc.GcResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.offsets[0])) as Long))

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff2ee5d56b0351a9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.gc.GcResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r53ea75e1654e9359.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun plainResourceNew(id: ULong, `value`: ULong, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.gc.PlainResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.gc.PlainResource = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_plain_resource_new_d97fe9314c02793e(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.gc.PlainResource? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_PlainResource_b08b40798e6273bd(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.gc.PlainResource? = null
        try {
            output0 = io.prebindgen.covertest.gc.PlainResource(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_PlainResource_b08b40798e6273bd(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun plainResourceValue(`value`: io.prebindgen.covertest.gc.PlainResource?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`value`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { `value`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_plain_resource_value_efe7741f532a6c5b(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun <A> gcResourceRowsFold(id: ULong, count: ULong, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.gc.GcResource) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_gc_resource_rows_fold_a0cec018fb39eb7c()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElement0: io.prebindgen.covertest.gc.GcResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { id.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { count.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_gc_resource_rows_fold_5c247ce3042d4525(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
        try {
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElement0: io.prebindgen.covertest.gc.GcResource? = null

try {
            foldElement0 = io.prebindgen.covertest.gc.GcResource(__pb_fold_row.c0)

current = fold(current, requireNotNull(foldElement0))
} catch (failure: Throwable) {
__pb_processed++
try {
            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
throw failure
}
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
if (nativeResult == 0) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}
