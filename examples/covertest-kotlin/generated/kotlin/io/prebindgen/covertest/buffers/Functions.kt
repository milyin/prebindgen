@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.buffers

public fun `jniBufferResource`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.buffers.JniBufferResourceRow> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.buffers.JniBufferResourceRow = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_String_ByteArray3_ErrorEmpty_b6abe218690243a7.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_resource_f33d027f936f6663(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        try {
            outputg0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(returnCapture.s0)
            io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputg0_0), returnCapture.s1 as String, returnCapture.s2 as ByteArray)
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `jniBufferDomainFailure`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.buffers.JniBufferResourceRowHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong_String_ByteArray3_2803e103835e4ca7.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_domain_failure_f36020e4f4562fe0(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {         var outputg0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(returnCapture.d0)

onError.run(io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputg0_0), returnCapture.d1 as String, returnCapture.d2 as ByteArray))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(returnCapture.d0) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
    Unit
    } finally { returnCapture.release() }
}

public fun `jniBufferResourceRows`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferResourceRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferResourceRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_resource_rows_54fd0c38ecd1f227(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.buffers.JniBufferResourceRow>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r605ce764399b6a69.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

            try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.buffers.JniBufferResourceRow>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r605ce764399b6a69.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try {             try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
            outputbulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackOwned((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[0])) as Long)

outputbulkList!!.add(io.prebindgen.covertest.buffers.JniBufferResourceRow(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[1])) as String, (requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[2])) as ByteArray))
}
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r605ce764399b6a69.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null

            try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8318083250c8683b.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `jniBufferText`(onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_text_d33d1c8fd91af20d(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as String
    } finally { returnCapture.release() }
}

public fun `jniBufferString`(onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_string_649b053bccecf6a7(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as String
    } finally { returnCapture.release() }
}

public fun `jniBufferBytes`(onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_bytes_801aa24b9714561c(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as ByteArray
    } finally { returnCapture.release() }
}

public fun `jniBufferVec`(onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_vec_18f26d75535912ae(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as ByteArray
    } finally { returnCapture.release() }
}

public fun <R> `jniBufferOptionalText`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, String?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_String2_ErrorEmpty_7e68b805da700de5.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_optional_text_d47e68fa29b4bf83(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            build(returnCapture.s0 == 1, if (returnCapture.s0 == 1) if (returnCapture.s1 != null) returnCapture.s1 as String else null else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `jniBufferOptionalBytes`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, ByteArray?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_optional_bytes_2e41382fb70d4d62(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            build(returnCapture.s0 == 1, if (returnCapture.s0 == 1) if (returnCapture.s1 != null) returnCapture.s1 as ByteArray else null else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `jniBufferOwnedOptional`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.buffers.JniBufferRow?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_String_ByteArray3_ErrorEmpty_e58bbdba0f222ca2.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_optional_3ea9b140c39283fe(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }
            build(returnCapture.s0 != -1, if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) io.prebindgen.covertest.buffers.JniBufferRow(returnCapture.s1 as String, returnCapture.s2 as ByteArray) else null else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `jniBufferOwned`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, ByteArray) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_1b20562b3c5bfef8(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0 as String, returnCapture.s1 as ByteArray)
    } finally { returnCapture.release() }
}

public fun `jniBufferRows`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.buffers.JniBufferRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_rows_0e85d0bcc7902cb5(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.buffers.JniBufferRow>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rbca58b4ea7ea820d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5dd4dff188dd3619.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5dd4dff188dd3619.size


outputbulkList!!.add(io.prebindgen.covertest.buffers.JniBufferRow((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5dd4dff188dd3619.offsets[0])) as String, (requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5dd4dff188dd3619.offsets[1])) as ByteArray))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> `jniBufferWide`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String, String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_String_String48_ErrorEmpty_181c8abbd254d130.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_wide_6983ec538bbdc661(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0 as String, returnCapture.s1 as String, returnCapture.s2 as String, returnCapture.s3 as String, returnCapture.s4 as String, returnCapture.s5 as String, returnCapture.s6 as String, returnCapture.s7 as String, returnCapture.s8 as String, returnCapture.s9 as String, returnCapture.s10 as String, returnCapture.s11 as String, returnCapture.s12 as String, returnCapture.s13 as String, returnCapture.s14 as String, returnCapture.s15 as String, returnCapture.s16 as String, returnCapture.s17 as String, returnCapture.s18 as String, returnCapture.s19 as String, returnCapture.s20 as String, returnCapture.s21 as String, returnCapture.s22 as String, returnCapture.s23 as String, returnCapture.s24 as String, returnCapture.s25 as String, returnCapture.s26 as String, returnCapture.s27 as String, returnCapture.s28 as String, returnCapture.s29 as String, returnCapture.s30 as String, returnCapture.s31 as String, returnCapture.s32 as String, returnCapture.s33 as String, returnCapture.s34 as String, returnCapture.s35 as String, returnCapture.s36 as String, returnCapture.s37 as String, returnCapture.s38 as String, returnCapture.s39 as String, returnCapture.s40 as String, returnCapture.s41 as String, returnCapture.s42 as String, returnCapture.s43 as String, returnCapture.s44 as String, returnCapture.s45 as String, returnCapture.s46 as String, returnCapture.s47 as String)
    } finally { returnCapture.release() }
}

public fun `jniBufferCallback`(p0_callback: io.prebindgen.covertest.stru8ListOptionalstrOptionalu8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_callback_a41319fcdf187075(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `jniBufferFailure`(p0_callback: io.prebindgen.covertest.callbacks.CallbackOwnedstru8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_failure_6fdac6591b1a6726(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `jniBufferWorker`(p0_callback: io.prebindgen.covertest.stru8ListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_worker_8d2b0e9314030afd(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `jniBufferOwnedCallback`(p0_callback: io.prebindgen.covertest.buffers.OwnedBufferViews?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_buffer_owned_callback_9fa8a662ae731b98(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}
