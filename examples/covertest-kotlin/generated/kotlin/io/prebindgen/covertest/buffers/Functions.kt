@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.buffers

public fun `jniBufferResource`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.buffers.JniBufferResourceRow> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.buffers.JniBufferResourceRow = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_resource_f33d027f936f6663() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        try {
            outputg0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(result[1] as Long)
            io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputg0_0), result[2] as String, result[3] as ByteArray)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferDomainFailure`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.buffers.JniBufferResourceRowHandler<Unit>): Unit = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_domain_failure_f36020e4f4562fe0() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> {         var outputg0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(result[1] as Long)

onError.run(io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputg0_0), result[2] as String, result[3] as ByteArray))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(result[1] as Long) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferResourceRows`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferResourceRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferResourceRow> = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_resource_rows_54fd0c38ecd1f227() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.buffers.JniBufferResourceRow>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_g0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(output_row[0] as Long)

outputlist.add(io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputitem_g0_0), output_row[1] as String, output_row[2] as ByteArray))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

            try { val owner = outputitem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferText`(onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_text_d33d1c8fd91af20d() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as String }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferString`(onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_string_649b053bccecf6a7() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as String }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferBytes`(onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_bytes_801aa24b9714561c() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as ByteArray }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferVec`(onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_vec_18f26d75535912ae() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as ByteArray }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `jniBufferOptionalText`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, String?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_optional_text_d47e68fa29b4bf83(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            build(result[1] == 1, if (result[1] == 1) if (result[2] != null) result[2] as String else null else null)
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `jniBufferOptionalBytes`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, ByteArray?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_optional_bytes_2e41382fb70d4d62(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            build(result[1] == 1, if (result[1] == 1) if (result[2] != null) result[2] as ByteArray else null else null)
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `jniBufferOwnedOptional`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.buffers.JniBufferRow?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_optional_3ea9b140c39283fe(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            build(result[1] != -1, if (result[1] != -1) if (result[1] == 1) io.prebindgen.covertest.buffers.JniBufferRow(result[2] as String, result[3] as ByteArray) else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `jniBufferOwned`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, ByteArray) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_1b20562b3c5bfef8() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as String, result[2] as ByteArray) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferRows`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferRow> = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_rows_0e85d0bcc7902cb5() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.buffers.JniBufferRow>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(io.prebindgen.covertest.buffers.JniBufferRow(output_row[0] as String, output_row[1] as ByteArray))
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

public fun <R> `jniBufferWide`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_jni_buffer_wide_6983ec538bbdc661() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as String, result[2] as String, result[3] as String, result[4] as String, result[5] as String, result[6] as String, result[7] as String, result[8] as String, result[9] as String, result[10] as String, result[11] as String, result[12] as String, result[13] as String, result[14] as String, result[15] as String, result[16] as String, result[17] as String, result[18] as String, result[19] as String, result[20] as String, result[21] as String, result[22] as String, result[23] as String, result[24] as String, result[25] as String, result[26] as String, result[27] as String, result[28] as String, result[29] as String, result[30] as String, result[31] as String, result[32] as String, result[33] as String, result[34] as String, result[35] as String, result[36] as String, result[37] as String, result[38] as String, result[39] as String, result[40] as String, result[41] as String, result[42] as String, result[43] as String, result[44] as String, result[45] as String, result[46] as String, result[47] as String, result[48] as String) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferCallback`(p0_callback: io.prebindgen.covertest.stru8ListOptionalstrOptionalu8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?, raw1: ByteArray?, raw2: String?, raw3: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: String
        lateinit var callbackArgument1_0: ByteArray
        var callbackArgument2_0: String? = null
        var callbackArgument3_0: ByteArray? = null
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackArgument0_0 = raw0 as String
            callbackArgument1_0 = raw1 as ByteArray
            if (raw2 != null) {
            }
            callbackArgument2_0 = if (raw2 != null) raw2 as String else null
            if (raw3 != null) {
            }
            callbackArgument3_0 = if (raw3 != null) raw3 as ByteArray else null
        } catch (failure: Throwable) {
            if (raw2 != null) {
            }
            if (raw3 != null) {
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0, callbackArgument1_0, callbackArgument2_0, callbackArgument3_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            if (raw2 != null) {
            }
            if (raw3 != null) {
            }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_callback_a41319fcdf187075(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferFailure`(p0_callback: io.prebindgen.covertest.callbacks.CallbackOwnedstru8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: String?, raw2: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackOwned
        lateinit var callbackArgument1_0: String
        lateinit var callbackArgument2_0: ByteArray
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
            callbackArgument1_0 = raw1 as String
            callbackArgument2_0 = raw2 as ByteArray
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0, callbackArgument1_0, callbackArgument2_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_failure_6fdac6591b1a6726(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferWorker`(p0_callback: io.prebindgen.covertest.stru8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?, raw1: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: String
        lateinit var callbackArgument1_0: ByteArray
        try {
            callbackArgument0_0 = raw0 as String
            callbackArgument1_0 = raw1 as ByteArray
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_worker_8d2b0e9314030afd(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `jniBufferOwnedCallback`(p0_callback: io.prebindgen.covertest.buffers.OwnedBufferViews?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?, raw1: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: String? = null
        var callbackArgument0_1: ByteArray? = null
        try {
            if (raw0 != null) {
            }
            callbackArgument0_0 = if (raw0 != null) raw0 as String else null
            callbackArgument0_1 = if (raw0 != null) raw1 as ByteArray else null
        } catch (failure: Throwable) {
            if (raw0 != null) {
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument0_1)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_callback_9fa8a662ae731b98(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

