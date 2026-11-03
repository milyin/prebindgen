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
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row[0] != null) {
try {
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[0] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[2] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row[0] != null) {
outputitem_list = kotlin.collections.ArrayList((output_row[0] as kotlin.Array<*>).size)
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_item_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row[0] as Long))
            outputitem_item_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row[2] as Long))

outputitem_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_item_g0_0), outputitem__row[1] as String, requireNotNull(outputitem_item_g2_0)))
}
            }

outputlist.add(if (output_row[0] != null) requireNotNull(outputitem_list) else null)
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

            if (output_row[0] != null) {
try {
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[0] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[2] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r840975b732d62e1f.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r840975b732d62e1f.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
        var outputbulkItem_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputbulkItem_bulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long))

outputbulkItem_bulkList!!.add(requireNotNull(outputbulkItem_bulkItem_0))
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r840975b732d62e1f.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

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
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row[0] as Long) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row[2] as Long) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

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
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(output_row[0] as Long))
            outputitem_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(output_row[2] as Long))

outputlist.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_g0_0), output_row[1] as String, requireNotNull(outputitem_g2_0)))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row[0] as Long) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(output_row[2] as Long) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

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
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try {
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[0] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[2] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

outputitem_list = kotlin.collections.ArrayList((output_row[0] as kotlin.Array<*>).size)
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputitem_item_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row[0] as Long))
            outputitem_item_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputitem__row[2] as Long))

outputitem_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputitem_item_g0_0), outputitem__row[1] as String, requireNotNull(outputitem_item_g2_0)))
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
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try {
for (outputitem_index in (output_row[0] as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputitem_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[0] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputitem__row[2] as Long) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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
try { val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as kotlin.Array<*>
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row[0] as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row[2] as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_cleanupError: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = outputg0_g0_cleanupError else if (outputg0_g0_failure !== outputg0_g0_cleanupError) outputg0_g0_failure!!.addSuppressed(outputg0_g0_cleanupError) }

}
} catch (outputg0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_cleanupError) failure.addSuppressed(outputg0_g0_cleanupError) }

}
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as kotlin.Array<*>
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row[0] as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row[2] as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }

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
val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as kotlin.Array<*>
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg0_g0_item_g0_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg0_g0__row[0] as Long))
            outputg0_g0_item_g2_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg0_g0__row[2] as Long))

outputg0_g0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg0_g0_item_g0_0), outputg0_g0__row[1] as String, requireNotNull(outputg0_g0_item_g2_0)))
}

outputg0_g0_owners!!.complete = true
outputg1_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg1_g0_list = kotlin.collections.ArrayList((returnCapture.s1 as kotlin.Array<*>).size)
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as kotlin.Array<*>
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg1_g0_item_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg1_g0__row[0] as Long))
            outputg1_g0_item_g2_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource(outputg1_g0__row[2] as Long))

outputg1_g0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg1_g0_item_g0_0), outputg1_g0__row[1] as String, requireNotNull(outputg1_g0_item_g2_0)))
}

outputg1_g0_owners!!.complete = true
            build(requireNotNull(outputg0_g0_list), requireNotNull(outputg1_g0_list))
        } catch (failure: Throwable) {
outputg0_g0_owners?.close(failure)
if (outputg0_g0_owners?.complete != true) {
try {
for (outputg0_g0_index in (returnCapture.s0 as kotlin.Array<*>).indices) {
var outputg0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_index] as kotlin.Array<*>
        var outputg0_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row[0] as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg0_g0__row[2] as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = cleanup else if (outputg0_g0_failure !== cleanup) outputg0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_cleanupError: Throwable) { if (outputg0_g0_failure == null) outputg0_g0_failure = outputg0_g0_cleanupError else if (outputg0_g0_failure !== outputg0_g0_cleanupError) outputg0_g0_failure!!.addSuppressed(outputg0_g0_cleanupError) }

}
} catch (outputg0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_cleanupError) failure.addSuppressed(outputg0_g0_cleanupError) }

}
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as kotlin.Array<*>
        var outputg1_g0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row[0] as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(outputg1_g0__row[2] as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = cleanup else if (outputg1_g0_failure !== cleanup) outputg1_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg1_g0_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_cleanupError else if (outputg1_g0_failure !== outputg1_g0_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_cleanupError) }

}
} catch (outputg1_g0_cleanupError: Throwable) { if (failure !== outputg1_g0_cleanupError) failure.addSuppressed(outputg1_g0_cleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `jniOutputFold`(p0_w1: Long, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.callbacks.CallbackRow) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_jni_output_fold_fold_a6fbab1712a6f896 { v0, v1, v2 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var foldElementg2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = foldElementg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = foldElementg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_jni_output_fold_fold_30e359f733b27f80(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var foldElementg2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

__pb_pending = { failure ->             try { val owner = foldElementg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = foldElementg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElementg0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(__pb_fold_row[0] as Long)
            foldElementg2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(__pb_fold_row[2] as Long)

current = fold(current, io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(foldElementg0_0), __pb_fold_row[1] as String, requireNotNull(foldElementg2_0)))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElementg0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var foldElementg2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = foldElementg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = foldElementg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}
