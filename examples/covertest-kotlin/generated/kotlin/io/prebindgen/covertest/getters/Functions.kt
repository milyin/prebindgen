@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.getters

public fun <R> `getterOwnerNew`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, Long, String, Long?, String?, io.prebindgen.covertest.getters.GetterDetails?, kotlin.collections.List<io.prebindgen.covertest.getters.GetterDetails>, io.prebindgen.covertest.getters.GetterLeaf) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_new_9b3691104cf4f094(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
var outputg4_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.getters.GetterDetails>? = null
var outputg4_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        var outputg5_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(result[1] as Long)
            if (result[5] != null) {
            }
            if (result[6] == 1) {
            }
outputg4_g0_cleanups = kotlin.collections.mutableListOf()
outputg4_g0_list = kotlin.collections.mutableListOf()
for (outputg4_g0_rawRow in (result[9] as kotlin.Array<*>)) {
val outputg4_g0__row = outputg4_g0_rawRow as kotlin.Array<*>

outputg4_g0_cleanups.add { outputg4_g0_initialFailure -> var outputg4_g0_failure = outputg4_g0_initialFailure; try {  } catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }; outputg4_g0_failure }

outputg4_g0_list.add(io.prebindgen.covertest.getters.GetterDetails(outputg4_g0__row[0] as Long, outputg4_g0__row[1] as String))
}
            outputg5_0 = io.prebindgen.covertest.getters.GetterLeaf(result[10] as Long)
            build(requireNotNull(outputg0_0), result[2] as Long, result[3] as String, if (result[5] != null) result[4] as Long else null, if (result[5] != null) result[5] as String else null, if (result[6] == 1) io.prebindgen.covertest.getters.GetterDetails(result[7] as Long, result[8] as String) else null, requireNotNull(outputg4_g0_list), requireNotNull(outputg5_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            if (result[5] != null) {
            }
            if (result[6] == 1) {
            }
try {
for (outputg4_g0_cleanupAction in outputg4_g0_cleanups.orEmpty()) { try { outputg4_g0_cleanupAction(failure) } catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) } }
for (outputg4_g0_index in (outputg4_g0_cleanups?.size ?: 0) until (result[9] as kotlin.Array<*>).size) {
var outputg4_g0_failure: kotlin.Throwable? = failure
try { val outputg4_g0__row = (result[9] as kotlin.Array<*>)[outputg4_g0_index] as kotlin.Array<*>


} catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }

}
} catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) }
            try { val owner = outputg5_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(result[10] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `getterOwnerFailure`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, io.prebindgen.covertest.getters.GetterLeaf, String) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_getter_owner_failure_983fd2bf1ecc08ff() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
        var outputg1_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(result[1] as Long)
            outputg1_0 = io.prebindgen.covertest.getters.GetterLeaf(result[2] as Long)
            build(requireNotNull(outputg0_0), requireNotNull(outputg1_0), result[3] as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `getterOwnerId`(p0_w1: io.prebindgen.covertest.getters.GetterOwner?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_id_6e5c79e15f3c8ecb(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `getterLeafId`(p0_w1: io.prebindgen.covertest.getters.GetterLeaf?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_leaf_id_2b89bfb1695b1a8e(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `getterStaticLeaf`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.getters.GetterLeaf> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.getters.GetterLeaf = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_getter_static_leaf_3de084de8ab56cf6() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            output0 = io.prebindgen.covertest.getters.GetterLeaf(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `getterOwnerRead`(p0_w1: io.prebindgen.covertest.getters.GetterOwner?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, String) -> R): R = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_owner_read_405afcf1de6633af(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as Long, result[2] as String) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `getterCounts`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ULong>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ULong> = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_getter_counts_ac432be3e581775f() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<ULong>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add((output_row[0] as Long).toULong())
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `getterTwoClonesNew`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterLeaf, io.prebindgen.covertest.getters.GetterLeaf) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_getter_two_clones_new_c6e10ed44149bf78() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.getters.GetterLeaf? = null
        var outputg0_1: io.prebindgen.covertest.getters.GetterLeaf? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterLeaf(result[1] as Long)
            outputg0_1 = io.prebindgen.covertest.getters.GetterLeaf(result[2] as Long)
            build(requireNotNull(outputg0_0), requireNotNull(outputg0_1))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `getterFieldsNew`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.getters.GetterOwner, Long, String) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_getter_fields_new_6223254642400003() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.getters.GetterOwner? = null
        try {
            outputg0_0 = io.prebindgen.covertest.getters.GetterOwner(result[1] as Long)
            build(requireNotNull(outputg0_0), result[2] as Long, result[3] as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `getterCallback`(p0_callback: io.prebindgen.covertest.getters.GetterObserver?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_getter_callback_261a8655bbbdf16f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

