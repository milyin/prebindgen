@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.nativeoutputs

public fun `jniOutputOptional`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>?> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_optional_b3a3cb08bb632879(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>?>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row.c0 != null) {
try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>?>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row.c0 != null) {
outputitem_list = kotlin.collections.ArrayList((output_row.c0 as kotlin.Array<*>).size)
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_item_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row.c0))
            outputitem_item_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row.c2))

outputitem_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_item_g0_0), outputitem__row.c1 as String, requireNotNull(outputitem_item_g2_0)))
}
            }

outputlist.add(if (output_row.c0 != null) requireNotNull(outputitem_list) else null)
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row.c0 != null) {
try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `jniOutputMixed`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_mixed_23877cf213bf0c0c(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_ByteArray_1_92dc0f7ab995287c
var outputitem_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputitem_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputitem_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

if (outputitem_g0_bulkReader == null) { outputitem_g0_bulkReader = io.prebindgen.covertest.BulkReader(output_row.c0 as ByteArray)
 }
try { if (outputitem_g0_bulkRange == null) outputitem_g0_bulkRange = requireNotNull(outputitem_g0_bulkReader).rows(((requireNotNull(outputitem_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (outputitem_g0_bulkIndex in 0 until outputitem_g0_bulkRange!!.length) {
val outputitem_g0_bulkBase = outputitem_g0_bulkRange!!.offset + outputitem_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var outputitem_g0_bulkFailure: Throwable? = outputfailure
try {         var outputitem_g0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputitem_g0_bulkReader).long((outputitem_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputitem_g0_bulkFailure == null) outputitem_g0_bulkFailure = cleanup else if (outputitem_g0_bulkFailure !== cleanup) outputitem_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputitem_g0_bulkCleanupError: Throwable) { if (outputitem_g0_bulkFailure == null) outputitem_g0_bulkFailure = outputitem_g0_bulkCleanupError else if (outputitem_g0_bulkFailure !== outputitem_g0_bulkCleanupError) outputitem_g0_bulkFailure!!.addSuppressed(outputitem_g0_bulkCleanupError) }
outputfailure = outputitem_g0_bulkFailure
} } catch (outputitem_g0_bulkCleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_g0_bulkCleanupError else if (outputfailure !== outputitem_g0_bulkCleanupError) outputfailure!!.addSuppressed(outputitem_g0_bulkCleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_ByteArray_1_92dc0f7ab995287c
var outputitem_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputitem_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputitem_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

outputitem_g0_bulkReader = io.prebindgen.covertest.BulkReader(output_row.c0 as ByteArray)
outputitem_g0_bulkRange = requireNotNull(outputitem_g0_bulkReader).rows(((requireNotNull(outputitem_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
outputitem_g0_bulkList = kotlin.collections.ArrayList(outputitem_g0_bulkRange!!.length)
for (outputitem_g0_bulkIndex in 0 until outputitem_g0_bulkRange!!.length) {
val outputitem_g0_bulkBase = outputitem_g0_bulkRange!!.offset + outputitem_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
        var outputitem_g0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_g0_bulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputitem_g0_bulkReader).long((outputitem_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long))

outputitem_g0_bulkList!!.add(requireNotNull(outputitem_g0_bulkItem_0))
}

outputlist.add(requireNotNull(outputitem_g0_bulkList))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_ByteArray_1_92dc0f7ab995287c
var outputitem_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputitem_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputitem_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

if (outputitem_g0_bulkReader == null) { outputitem_g0_bulkReader = io.prebindgen.covertest.BulkReader(output_row.c0 as ByteArray)
 }
try { if (outputitem_g0_bulkRange == null) outputitem_g0_bulkRange = requireNotNull(outputitem_g0_bulkReader).rows(((requireNotNull(outputitem_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (outputitem_g0_bulkIndex in 0 until outputitem_g0_bulkRange!!.length) {
val outputitem_g0_bulkBase = outputitem_g0_bulkRange!!.offset + outputitem_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var outputitem_g0_bulkFailure: Throwable? = outputfailure
try {         var outputitem_g0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputitem_g0_bulkReader).long((outputitem_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputitem_g0_bulkFailure == null) outputitem_g0_bulkFailure = cleanup else if (outputitem_g0_bulkFailure !== cleanup) outputitem_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputitem_g0_bulkCleanupError: Throwable) { if (outputitem_g0_bulkFailure == null) outputitem_g0_bulkFailure = outputitem_g0_bulkCleanupError else if (outputitem_g0_bulkFailure !== outputitem_g0_bulkCleanupError) outputitem_g0_bulkFailure!!.addSuppressed(outputitem_g0_bulkCleanupError) }
outputfailure = outputitem_g0_bulkFailure
} } catch (outputitem_g0_bulkCleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_g0_bulkCleanupError else if (outputfailure !== outputitem_g0_bulkCleanupError) outputfailure!!.addSuppressed(outputitem_g0_bulkCleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `jniOutputRows`(p0_w1: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>): kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_output_rows_76dcbc8fbe8acf89(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row.c2) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(output_row.c0))
            outputitem_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(output_row.c2))

outputlist.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_g0_0), output_row.c1 as String, requireNotNull(outputitem_g2_0)))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row.c2) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `jniOutputNested`(p0_w1: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>>): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_output_nested_09e14469d381a037(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

outputitem_list = kotlin.collections.ArrayList((output_row.c0 as kotlin.Array<*>).size)
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_item_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row.c0))
            outputitem_item_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row.c2))

outputitem_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_item_g0_0), outputitem__row.c1 as String, requireNotNull(outputitem_item_g2_0)))
}

outputlist.add(requireNotNull(outputitem_list))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `jniOutputPair`(p0_w1: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<R>, build: (kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>, kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Array_Any___Array_Any__2_ErrorLong1_59a6678d4242c812.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_output_pair_654ab15b1845473b(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputg0_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
 outputg0_g0_owners?.close(failure)
if (outputg0_g0_owners?.complete != true) {
try {
for (outputg0_g0_index in (returnCapture.s0 as kotlin.Array<*>).indices) {
var outputg0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row.c2) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_cleanupError: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = outputg0_g0_cleanupError else if (outputg0_g0_failure !== outputg0_g0_cleanupError) outputg0_g0_failure!!.addSuppressed(outputg0_g0_cleanupError) }

}
} catch (outputg0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_cleanupError) failure.addSuppressed(outputg0_g0_cleanupError) }

}
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row.c2) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg1_g0_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_cleanupError else if (outputg1_g0_failure !== outputg1_g0_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_cleanupError) }

}
} catch (outputg1_g0_cleanupError: Throwable) { if (failure !== outputg1_g0_cleanupError) failure.addSuppressed(outputg1_g0_cleanupError) }

}
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputg0_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
        try {
outputg0_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg0_g0_list = kotlin.collections.ArrayList((returnCapture.s0 as kotlin.Array<*>).size)
for (outputg0_g0_index in (returnCapture.s0 as kotlin.Array<*>).indices) {
val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg0_g0_item_g0_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg0_g0__row.c0))
            outputg0_g0_item_g2_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg0_g0__row.c2))

outputg0_g0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg0_g0_item_g0_0), outputg0_g0__row.c1 as String, requireNotNull(outputg0_g0_item_g2_0)))
}

outputg0_g0_owners!!.complete = true
outputg1_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg1_g0_list = kotlin.collections.ArrayList((returnCapture.s1 as kotlin.Array<*>).size)
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg1_g0_item_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg1_g0__row.c0))
            outputg1_g0_item_g2_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg1_g0__row.c2))

outputg1_g0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg1_g0_item_g0_0), outputg1_g0__row.c1 as String, requireNotNull(outputg1_g0_item_g2_0)))
}

outputg1_g0_owners!!.complete = true
            build(requireNotNull(outputg0_g0_list), requireNotNull(outputg1_g0_list))
        } catch (failure: Throwable) {
outputg0_g0_owners?.close(failure)
if (outputg0_g0_owners?.complete != true) {
try {
for (outputg0_g0_index in (returnCapture.s0 as kotlin.Array<*>).indices) {
var outputg0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row.c2) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_cleanupError: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = outputg0_g0_cleanupError else if (outputg0_g0_failure !== outputg0_g0_cleanupError) outputg0_g0_failure!!.addSuppressed(outputg0_g0_cleanupError) }

}
} catch (outputg0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_cleanupError) failure.addSuppressed(outputg0_g0_cleanupError) }

}
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_String_Long_3_b4558cb7b740d7c6
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row.c2) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg1_g0_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_cleanupError else if (outputg1_g0_failure !== outputg1_g0_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_cleanupError) }

}
} catch (outputg1_g0_cleanupError: Throwable) { if (failure !== outputg1_g0_cleanupError) failure.addSuppressed(outputg1_g0_cleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `jniOutputMixedFramesFold`(p0_w1: Long, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_jni_output_mixed_frames_fold_a5668456db986db2()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
var foldElementowners: io.prebindgen.covertest.CollectionOwners? = null
var foldElementbulkRange: io.prebindgen.covertest.BulkRange? = null
var foldElementbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
foldElementowners?.close(failure)
if (foldElementowners?.complete != true) {
try { if (foldElementbulkRange == null) foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var foldElementbulkFailure: Throwable? = failure
try {         var foldElementbulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
 } catch (foldElementbulkCleanupError: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = foldElementbulkCleanupError else if (foldElementbulkFailure !== foldElementbulkCleanupError) foldElementbulkFailure!!.addSuppressed(foldElementbulkCleanupError) }

} } catch (foldElementbulkCleanupError: Throwable) { if (failure !== foldElementbulkCleanupError) failure.addSuppressed(foldElementbulkCleanupError) }

}

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_mixed_frames_fold_e9474c6b8b0056cf(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_foldOwners: io.prebindgen.covertest.CollectionOwners? = null
        try {
if (nativeResult != null) {
if (__pb_rows.size != 0) __pb_foldOwners = io.prebindgen.covertest.CollectionOwners()
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
var foldElementbulkRange: io.prebindgen.covertest.BulkRange? = null
var foldElementbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

try {
foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
foldElementbulkList = kotlin.collections.ArrayList(foldElementbulkRange!!.length)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
        var foldElementbulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            foldElementbulkItem_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long))

foldElementbulkList!!.add(requireNotNull(foldElementbulkItem_0))
}

__pb_foldOwners!!.complete = true
current = fold(current, requireNotNull(foldElementbulkList))
} catch (failure: Throwable) {
__pb_processed++
try {
__pb_foldOwners?.close(failure)
if (__pb_foldOwners?.complete != true) {
if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
try { if (foldElementbulkRange == null) foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var foldElementbulkFailure: Throwable? = failure
try {         var foldElementbulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
 } catch (foldElementbulkCleanupError: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = foldElementbulkCleanupError else if (foldElementbulkFailure !== foldElementbulkCleanupError) foldElementbulkFailure!!.addSuppressed(foldElementbulkCleanupError) }

} } catch (foldElementbulkCleanupError: Throwable) { if (failure !== foldElementbulkCleanupError) failure.addSuppressed(foldElementbulkCleanupError) }

}
} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
throw failure
}
__pb_processed++
__pb_foldOwners!!.release()
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
if (nativeResult != null) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `jniOutputFoldDropFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_jni_output_fold_drop_fold_a39a891c792a6c19()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementowners: io.prebindgen.covertest.CollectionOwners? = null
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

foldElementowners?.close(failure)
if (foldElementowners?.complete != true) {
try {
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(foldElement_row.c0) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_fold_drop_fold_8e0eaffd2786301c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_foldOwners: io.prebindgen.covertest.CollectionOwners? = null
        try {
if (nativeResult != null) {
if (__pb_rows.size != 0) __pb_foldOwners = io.prebindgen.covertest.CollectionOwners()
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

try {
foldElementlist = kotlin.collections.ArrayList((__pb_fold_row.c0 as kotlin.Array<*>).size)
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            foldElementitem_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(foldElement_row.c0))

foldElementlist.add(requireNotNull(foldElementitem_0))
}

__pb_foldOwners!!.complete = true
current = fold(current, requireNotNull(foldElementlist))
} catch (failure: Throwable) {
__pb_processed++
try {
__pb_foldOwners?.close(failure)
if (__pb_foldOwners?.complete != true) {
try {
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(foldElement_row.c0) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}
} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
throw failure
}
__pb_processed++
__pb_foldOwners!!.release()
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
if (nativeResult != null) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `jniOutputFold`(p0_w1: Long, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.callbacks.CallbackRow) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_jni_output_fold_fold_da0d511a519a94f5()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElementg0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var foldElementg2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = foldElementg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = foldElementg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row.c2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_fold_fold_30e359f733b27f80(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
        try {
if (nativeResult != null) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElementg0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var foldElementg2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

try {
            foldElementg0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(__pb_fold_row.c0)
            foldElementg2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(__pb_fold_row.c2)

current = fold(current, io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(foldElementg0_0), __pb_fold_row.c1 as String, requireNotNull(foldElementg2_0)))
} catch (failure: Throwable) {
__pb_processed++
try {
            try { val owner = foldElementg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = foldElementg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row.c2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
throw failure
}
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
if (nativeResult != null) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}
