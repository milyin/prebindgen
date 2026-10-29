@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.getters

public fun <R> `getterOwnerNew`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, Long, String, Long?, String?, io.prebindgen.covertest.getters.GetterDetails?, kotlin.collections.List<io.prebindgen.covertest.getters.GetterDetails>, io.prebindgen.covertest.getters.GetterLeaf) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_new_9b3691104cf4f094(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
var outputg4_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.getters.GetterDetails>? = null
var outputg4_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        var outputg5_0: io.prebindgen.covertest.getters.GetterLeaf? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            if (returnCapture.s4 != null) {
            }
            if (returnCapture.s5 == 1) {
            }
try {
for (outputg4_g0_cleanupAction in outputg4_g0_cleanups.orEmpty()) { try { outputg4_g0_cleanupAction(failure) } catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) } }
for (outputg4_g0_index in (outputg4_g0_cleanups?.size ?: 0) until (returnCapture.s8 as kotlin.Array<*>).size) {
var outputg4_g0_failure: kotlin.Throwable? = failure
try { val outputg4_g0__row = (returnCapture.s8 as kotlin.Array<*>)[outputg4_g0_index] as kotlin.Array<*>


} catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }

}
} catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) }
            try { val owner = outputg5_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s9) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
var outputg4_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.getters.GetterDetails>? = null
var outputg4_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        var outputg5_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(returnCapture.s0)
            if (returnCapture.s4 != null) {
            }
            if (returnCapture.s5 == 1) {
            }
outputg4_g0_cleanups = kotlin.collections.mutableListOf()
outputg4_g0_list = kotlin.collections.mutableListOf()
for (outputg4_g0_rawRow in (returnCapture.s8 as kotlin.Array<*>)) {
val outputg4_g0__row = outputg4_g0_rawRow as kotlin.Array<*>

outputg4_g0_cleanups.add { outputg4_g0_initialFailure -> var outputg4_g0_failure = outputg4_g0_initialFailure; try {  } catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }; outputg4_g0_failure }

outputg4_g0_list.add(io.prebindgen.covertest.getters.GetterDetails(outputg4_g0__row[0] as Long, outputg4_g0__row[1] as String))
}
            outputg5_0 = io.prebindgen.covertest.getters.GetterLeaf(returnCapture.s9)
            build(requireNotNull(outputg0_0), returnCapture.s1, returnCapture.s2 as String, if (returnCapture.s4 != null) returnCapture.s3 else null, if (returnCapture.s4 != null) returnCapture.s4 as String else null, if (returnCapture.s5 == 1) io.prebindgen.covertest.getters.GetterDetails(returnCapture.s6, returnCapture.s7 as String) else null, requireNotNull(outputg4_g0_list), requireNotNull(outputg5_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            if (returnCapture.s4 != null) {
            }
            if (returnCapture.s5 == 1) {
            }
try {
for (outputg4_g0_cleanupAction in outputg4_g0_cleanups.orEmpty()) { try { outputg4_g0_cleanupAction(failure) } catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) } }
for (outputg4_g0_index in (outputg4_g0_cleanups?.size ?: 0) until (returnCapture.s8 as kotlin.Array<*>).size) {
var outputg4_g0_failure: kotlin.Throwable? = failure
try { val outputg4_g0__row = (returnCapture.s8 as kotlin.Array<*>)[outputg4_g0_index] as kotlin.Array<*>


} catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }

}
} catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) }
            try { val owner = outputg5_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s9) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `getterOwnerFailure`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, io.prebindgen.covertest.getters.GetterLeaf, String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_String3_ErrorEmpty_857937f364385c79.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_failure_983fd2bf1ecc08ff(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
        var outputg1_0: io.prebindgen.covertest.getters.GetterLeaf? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
        var outputg1_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(returnCapture.s0)
            outputg1_0 = io.prebindgen.covertest.getters.GetterLeaf(returnCapture.s1)
            build(requireNotNull(outputg0_0), requireNotNull(outputg1_0), returnCapture.s2 as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `getterOwnerId`(p0_w1: io.prebindgen.covertest.getters.GetterOwner?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_id_6e5c79e15f3c8ecb(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `getterLeafId`(p0_w1: io.prebindgen.covertest.getters.GetterLeaf?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_leaf_id_2b89bfb1695b1a8e(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `getterStaticLeaf`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.getters.GetterLeaf> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.getters.GetterLeaf = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_static_leaf_3de084de8ab56cf6(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.getters.GetterLeaf? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            output0 = io.prebindgen.covertest.getters.GetterLeaf(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `getterOwnerRead`(p0_w1: io.prebindgen.covertest.getters.GetterOwner?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_String2_ErrorEmpty_1b009765d5d6eed0.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_read_405afcf1de6633af(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1 as String)
    } finally { returnCapture.release() }
}

public fun `getterCounts`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ULong>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ULong> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_counts_ac432be3e581775f(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<ULong>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<ULong>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add((output_row[0] as Long).toULong())
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `getterTwoClonesNew`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterLeaf, io.prebindgen.covertest.getters.GetterLeaf) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_two_clones_new_c6e10ed44149bf78(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        var outputg0_1: io.prebindgen.covertest.getters.GetterLeaf? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        var outputg0_1: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterLeaf(returnCapture.s0)
            outputg0_1 = io.prebindgen.covertest.getters.GetterLeaf(returnCapture.s1)
            build(requireNotNull(outputg0_0), requireNotNull(outputg0_1))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `getterFieldsNew`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, Long, String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_String3_ErrorEmpty_857937f364385c79.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_fields_new_6223254642400003(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(returnCapture.s0)
            build(requireNotNull(outputg0_0), returnCapture.s1, returnCapture.s2 as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `getterCallback`(p0_callback: io.prebindgen.covertest.getters.GetterObserver?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: String?, raw2: Long, raw3: String?, raw4: Int, raw5: Long, raw6: String?, raw7: Array<Any?>?, raw8: Long) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_g3_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.getters.GetterDetails>? = null
var callbackOutput0_g3_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        var callbackOutput0_g4_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackArgument0_1: String
        var callbackArgument0_2: Long? = null
        var callbackArgument0_3: String? = null
        var callbackArgument0_4: io.prebindgen.covertest.getters.GetterDetails? = null
        lateinit var callbackArgument0_5: kotlin.collections.List<io.prebindgen.covertest.getters.GetterDetails>
        lateinit var callbackArgument0_6: io.prebindgen.covertest.getters.GetterLeaf
        try {
            if (raw3 != null) {
            }
            if (raw4 == 1) {
            }
callbackOutput0_g3_g0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_g3_g0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_g3_g0_rawRow in (raw7 as kotlin.Array<*>)) {
val callbackOutput0_g3_g0__row = callbackOutput0_g3_g0_rawRow as kotlin.Array<*>

callbackOutput0_g3_g0_cleanups.add { callbackOutput0_g3_g0_initialFailure -> var callbackOutput0_g3_g0_failure = callbackOutput0_g3_g0_initialFailure; try {  } catch (callbackOutput0_g3_g0_cleanupError: Throwable) { if (callbackOutput0_g3_g0_failure == null) callbackOutput0_g3_g0_failure = callbackOutput0_g3_g0_cleanupError else if (callbackOutput0_g3_g0_failure !== callbackOutput0_g3_g0_cleanupError) callbackOutput0_g3_g0_failure!!.addSuppressed(callbackOutput0_g3_g0_cleanupError) }; callbackOutput0_g3_g0_failure }

callbackOutput0_g3_g0_list.add(io.prebindgen.covertest.getters.GetterDetails(callbackOutput0_g3_g0__row[0] as Long, callbackOutput0_g3_g0__row[1] as String))
}
            callbackOutput0_g4_0 = io.prebindgen.covertest.getters.GetterLeaf(raw8 as Long)
            callbackArgument0_0 = raw0 as Long
            callbackArgument0_1 = raw1 as String
            callbackArgument0_2 = if (raw3 != null) raw2 as Long else null
            callbackArgument0_3 = if (raw3 != null) raw3 as String else null
            callbackArgument0_4 = if (raw4 == 1) io.prebindgen.covertest.getters.GetterDetails(raw5 as Long, raw6 as String) else null
            callbackArgument0_5 = requireNotNull(callbackOutput0_g3_g0_list)
            callbackArgument0_6 = requireNotNull(callbackOutput0_g4_0)
        } catch (failure: Throwable) {
            if (raw3 != null) {
            }
            if (raw4 == 1) {
            }
try {
for (callbackOutput0_g3_g0_cleanupAction in callbackOutput0_g3_g0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_g3_g0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_g3_g0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_g3_g0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_g3_g0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_g3_g0_cleanupError) } }
for (callbackOutput0_g3_g0_index in (callbackOutput0_g3_g0_cleanups?.size ?: 0) until (raw7 as kotlin.Array<*>).size) {
var callbackOutput0_g3_g0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0_g3_g0__row = (raw7 as kotlin.Array<*>)[callbackOutput0_g3_g0_index] as kotlin.Array<*>


} catch (callbackOutput0_g3_g0_cleanupError: Throwable) { if (callbackOutput0_g3_g0_failure == null) callbackOutput0_g3_g0_failure = callbackOutput0_g3_g0_cleanupError else if (callbackOutput0_g3_g0_failure !== callbackOutput0_g3_g0_cleanupError) callbackOutput0_g3_g0_failure!!.addSuppressed(callbackOutput0_g3_g0_cleanupError) }
callbackCleanupFailure = callbackOutput0_g3_g0_failure
}
} catch (callbackOutput0_g3_g0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_g3_g0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_g3_g0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_g3_g0_cleanupError) }
            try { val owner = callbackOutput0_g4_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(raw8 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument0_1, callbackArgument0_2, callbackArgument0_3, callbackArgument0_4, callbackArgument0_5, callbackArgument0_6)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_callback_261a8655bbbdf16f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}
