@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.cleanup

public fun `nativeBulkRows`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_native_bulk_rows_f3ef23b4e405ea00(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long))
            outputbulkItem_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long))

outputbulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(outputbulkItem_g2_0)))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `nativeBulkNested`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_native_bulk_nested_a866f03d6554ae4e(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
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
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var outputbulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputbulkItem_bulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long))
            outputbulkItem_bulkItem_g2_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long))

outputbulkItem_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputbulkItem_bulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(outputbulkItem_bulkItem_g2_0)))
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
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputbulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
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

public fun <R> `nativeBulkPair`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>, kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_ByteArray_ByteArray2_ErrorEmpty_cdd3629414305251.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_native_bulk_pair_a4a8c00804a0cda0(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputg0_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg0_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg0_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg0_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var outputg1_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
 if (outputg0_g0_bulkReader == null) { outputg0_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s0 as ByteArray)
 }
outputg0_g0_owners?.close(failure)
if (outputg0_g0_owners?.complete != true) {
try { if (outputg0_g0_bulkRange == null) outputg0_g0_bulkRange = requireNotNull(outputg0_g0_bulkReader).rows(((requireNotNull(outputg0_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputg0_g0_bulkIndex in 0 until outputg0_g0_bulkRange!!.length) {
val outputg0_g0_bulkBase = outputg0_g0_bulkRange!!.offset + outputg0_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputg0_g0_bulkFailure: Throwable? = failure
try {         var outputg0_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = cleanup else if (outputg0_g0_bulkFailure !== cleanup) outputg0_g0_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = cleanup else if (outputg0_g0_bulkFailure !== cleanup) outputg0_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_bulkCleanupError: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = outputg0_g0_bulkCleanupError else if (outputg0_g0_bulkFailure !== outputg0_g0_bulkCleanupError) outputg0_g0_bulkFailure!!.addSuppressed(outputg0_g0_bulkCleanupError) }

} } catch (outputg0_g0_bulkCleanupError: Throwable) { if (failure !== outputg0_g0_bulkCleanupError) failure.addSuppressed(outputg0_g0_bulkCleanupError) }

}
if (outputg1_g0_bulkReader == null) { outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
 }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try { if (outputg1_g0_bulkRange == null) outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputg1_g0_bulkFailure: Throwable? = failure
try {         var outputg1_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = cleanup else if (outputg1_g0_bulkFailure !== cleanup) outputg1_g0_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = cleanup else if (outputg1_g0_bulkFailure !== cleanup) outputg1_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputg1_g0_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkCleanupError) }

} } catch (outputg1_g0_bulkCleanupError: Throwable) { if (failure !== outputg1_g0_bulkCleanupError) failure.addSuppressed(outputg1_g0_bulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputg0_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg0_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg0_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg0_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var outputg1_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
        try {
outputg0_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s0 as ByteArray)
outputg0_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg0_g0_bulkRange = requireNotNull(outputg0_g0_bulkReader).rows(((requireNotNull(outputg0_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
outputg0_g0_bulkList = kotlin.collections.ArrayList(outputg0_g0_bulkRange!!.length)
for (outputg0_g0_bulkIndex in 0 until outputg0_g0_bulkRange!!.length) {
val outputg0_g0_bulkBase = outputg0_g0_bulkRange!!.offset + outputg0_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var outputg0_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg0_g0_bulkItem_g0_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long))
            outputg0_g0_bulkItem_g2_0 = requireNotNull(outputg0_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long))

outputg0_g0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg0_g0_bulkItem_g0_0), (requireNotNull(outputg0_g0_bulkReader).text((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(outputg0_g0_bulkItem_g2_0)))
}

outputg0_g0_owners!!.complete = true
outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
outputg1_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
outputg1_g0_bulkList = kotlin.collections.ArrayList(outputg1_g0_bulkRange!!.length)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var outputg1_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            outputg1_g0_bulkItem_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long))
            outputg1_g0_bulkItem_g2_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long))

outputg1_g0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(outputg1_g0_bulkItem_g0_0), (requireNotNull(outputg1_g0_bulkReader).text((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(outputg1_g0_bulkItem_g2_0)))
}

outputg1_g0_owners!!.complete = true
            build(requireNotNull(outputg0_g0_bulkList), requireNotNull(outputg1_g0_bulkList))
        } catch (failure: Throwable) {
if (outputg0_g0_bulkReader == null) { outputg0_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s0 as ByteArray)
 }
outputg0_g0_owners?.close(failure)
if (outputg0_g0_owners?.complete != true) {
try { if (outputg0_g0_bulkRange == null) outputg0_g0_bulkRange = requireNotNull(outputg0_g0_bulkReader).rows(((requireNotNull(outputg0_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputg0_g0_bulkIndex in 0 until outputg0_g0_bulkRange!!.length) {
val outputg0_g0_bulkBase = outputg0_g0_bulkRange!!.offset + outputg0_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputg0_g0_bulkFailure: Throwable? = failure
try {         var outputg0_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg0_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = cleanup else if (outputg0_g0_bulkFailure !== cleanup) outputg0_g0_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputg0_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg0_g0_bulkReader).long((outputg0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = cleanup else if (outputg0_g0_bulkFailure !== cleanup) outputg0_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_bulkCleanupError: Throwable) { if (outputg0_g0_bulkFailure == null) outputg0_g0_bulkFailure = outputg0_g0_bulkCleanupError else if (outputg0_g0_bulkFailure !== outputg0_g0_bulkCleanupError) outputg0_g0_bulkFailure!!.addSuppressed(outputg0_g0_bulkCleanupError) }

} } catch (outputg0_g0_bulkCleanupError: Throwable) { if (failure !== outputg0_g0_bulkCleanupError) failure.addSuppressed(outputg0_g0_bulkCleanupError) }

}
if (outputg1_g0_bulkReader == null) { outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
 }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try { if (outputg1_g0_bulkRange == null) outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var outputg1_g0_bulkFailure: Throwable? = failure
try {         var outputg1_g0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var outputg1_g0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = cleanup else if (outputg1_g0_bulkFailure !== cleanup) outputg1_g0_bulkFailure!!.addSuppressed(cleanup) }
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = cleanup else if (outputg1_g0_bulkFailure !== cleanup) outputg1_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputg1_g0_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkCleanupError) }

} } catch (outputg1_g0_bulkCleanupError: Throwable) { if (failure !== outputg1_g0_bulkCleanupError) failure.addSuppressed(outputg1_g0_bulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `nativeBulkCallback`(p0_w1: Long, p1_callback: io.prebindgen.covertest.NativeBulkCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_owners: io.prebindgen.covertest.CollectionOwners? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_owners = io.prebindgen.covertest.CollectionOwners()
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkItem_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkItem_bulkRange!!.length)
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            callbackOutput0_bulkItem_bulkItem_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long))
            callbackOutput0_bulkItem_bulkItem_g2_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long))

callbackOutput0_bulkItem_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_bulkItem_g2_0)))
}

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_bulkList))
}

callbackOutput0_owners!!.complete = true
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try { var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p1_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4870fc95cea3097a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try { var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null

try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_native_bulk_callback_b0aa3d8beb220a07(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun <R> `outputCleanupSurround`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.OwnedResource, kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>, io.prebindgen.covertest.resources.OwnedResource) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_surround_58e43f8fdb96cdf0(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.resources.OwnedResource? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg1_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (outputg1_g0__row.c0 != null) {
try {
for (outputg1_g0_item_index in (outputg1_g0__row.c0 as kotlin.Array<*>).indices) {
var outputg1_g0_item_failure: kotlin.Throwable? = outputg1_g0_failure
try { val outputg1_g0_item__row = (outputg1_g0__row.c0 as kotlin.Array<*>)[outputg1_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputg1_g0_item_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_item_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputg1_g0_item__row.c1 != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg1_g0_item__row.c1) } } catch (cleanup: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = cleanup else if (outputg1_g0_item_failure !== cleanup) outputg1_g0_item_failure!!.addSuppressed(cleanup) }
            }
            if (outputg1_g0_item__row.c2 != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg1_g0_item__row.c2) } } catch (cleanup: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = cleanup else if (outputg1_g0_item_failure !== cleanup) outputg1_g0_item_failure!!.addSuppressed(cleanup) }
            }

} catch (outputg1_g0_item_cleanupError: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = outputg1_g0_item_cleanupError else if (outputg1_g0_item_failure !== outputg1_g0_item_cleanupError) outputg1_g0_item_failure!!.addSuppressed(outputg1_g0_item_cleanupError) }
outputg1_g0_failure = outputg1_g0_item_failure
}
} catch (outputg1_g0_item_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_item_cleanupError else if (outputg1_g0_failure !== outputg1_g0_item_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_item_cleanupError) }
            }

} catch (outputg1_g0_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_cleanupError else if (outputg1_g0_failure !== outputg1_g0_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_cleanupError) }

}
} catch (outputg1_g0_cleanupError: Throwable) { if (failure !== outputg1_g0_cleanupError) failure.addSuppressed(outputg1_g0_cleanupError) }

}
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.resources.OwnedResource? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s0)
outputg1_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg1_g0_list = kotlin.collections.ArrayList((returnCapture.s1 as kotlin.Array<*>).size)
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg1_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (outputg1_g0__row.c0 != null) {
outputg1_g0_item_list = kotlin.collections.ArrayList((outputg1_g0__row.c0 as kotlin.Array<*>).size)
for (outputg1_g0_item_index in (outputg1_g0__row.c0 as kotlin.Array<*>).indices) {
val outputg1_g0_item__row = (outputg1_g0__row.c0 as kotlin.Array<*>)[outputg1_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputg1_g0_item_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_item_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputg1_g0_item__row.c1 != 0L) {
            outputg1_g0_item_item_g1_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.resources.OwnedResource(outputg1_g0_item__row.c1))
            }
            if (outputg1_g0_item__row.c2 != 0L) {
            outputg1_g0_item_item_g2_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.resources.OwnedResource(outputg1_g0_item__row.c2))
            }

outputg1_g0_item_list.add(io.prebindgen.covertest.cleanup.OutputCleanupRow(outputg1_g0_item__row.c0, if (outputg1_g0_item__row.c1 != 0L) requireNotNull(outputg1_g0_item_item_g1_g0_0) else null, if (outputg1_g0_item__row.c2 != 0L) requireNotNull(outputg1_g0_item_item_g2_g0_0) else null))
}
            }

outputg1_g0_list.add(if (outputg1_g0__row.c0 != null) requireNotNull(outputg1_g0_item_list) else null)
}

outputg1_g0_owners!!.complete = true
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s2)
            build(requireNotNull(outputg0_0), requireNotNull(outputg1_g0_list), requireNotNull(outputg2_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try {
for (outputg1_g0_index in (returnCapture.s1 as kotlin.Array<*>).indices) {
var outputg1_g0_failure: kotlin.Throwable? = failure
try { val outputg1_g0__row = (returnCapture.s1 as kotlin.Array<*>)[outputg1_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg1_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (outputg1_g0__row.c0 != null) {
try {
for (outputg1_g0_item_index in (outputg1_g0__row.c0 as kotlin.Array<*>).indices) {
var outputg1_g0_item_failure: kotlin.Throwable? = outputg1_g0_failure
try { val outputg1_g0_item__row = (outputg1_g0__row.c0 as kotlin.Array<*>)[outputg1_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputg1_g0_item_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_item_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputg1_g0_item__row.c1 != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg1_g0_item__row.c1) } } catch (cleanup: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = cleanup else if (outputg1_g0_item_failure !== cleanup) outputg1_g0_item_failure!!.addSuppressed(cleanup) }
            }
            if (outputg1_g0_item__row.c2 != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg1_g0_item__row.c2) } } catch (cleanup: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = cleanup else if (outputg1_g0_item_failure !== cleanup) outputg1_g0_item_failure!!.addSuppressed(cleanup) }
            }

} catch (outputg1_g0_item_cleanupError: Throwable) { if (outputg1_g0_item_failure == null) outputg1_g0_item_failure = outputg1_g0_item_cleanupError else if (outputg1_g0_item_failure !== outputg1_g0_item_cleanupError) outputg1_g0_item_failure!!.addSuppressed(outputg1_g0_item_cleanupError) }
outputg1_g0_failure = outputg1_g0_item_failure
}
} catch (outputg1_g0_item_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_item_cleanupError else if (outputg1_g0_failure !== outputg1_g0_item_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_item_cleanupError) }
            }

} catch (outputg1_g0_cleanupError: Throwable) { if (outputg1_g0_failure == null) outputg1_g0_failure = outputg1_g0_cleanupError else if (outputg1_g0_failure !== outputg1_g0_cleanupError) outputg1_g0_failure!!.addSuppressed(outputg1_g0_cleanupError) }

}
} catch (outputg1_g0_cleanupError: Throwable) { if (failure !== outputg1_g0_cleanupError) failure.addSuppressed(outputg1_g0_cleanupError) }

}
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `outputCleanupSurroundBulk`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.OwnedResource, kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>, io.prebindgen.covertest.resources.OwnedResource) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_ByteArray_Long3_ErrorEmpty_1d2a2254ab7aac6e.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_surround_bulk_f8fb155e97de0ce3(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.resources.OwnedResource? = null
var outputg1_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
if (outputg1_g0_bulkReader == null) { outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
 }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try { if (outputg1_g0_bulkRange == null) outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputg1_g0_bulkFailure: Throwable? = failure
try { var outputg1_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputg1_g0_bulkReader).int((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (outputg1_g0_bulkItem_bulkRange == null) outputg1_g0_bulkItem_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputg1_g0_bulkItem_bulkIndex in 0 until outputg1_g0_bulkItem_bulkRange!!.length) {
val outputg1_g0_bulkItem_bulkBase = outputg1_g0_bulkItem_bulkRange!!.offset + outputg1_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputg1_g0_bulkItem_bulkFailure: Throwable? = outputg1_g0_bulkFailure
try {         var outputg1_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_bulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = cleanup else if (outputg1_g0_bulkItem_bulkFailure !== cleanup) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = cleanup else if (outputg1_g0_bulkItem_bulkFailure !== cleanup) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputg1_g0_bulkItem_bulkCleanupError: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = outputg1_g0_bulkItem_bulkCleanupError else if (outputg1_g0_bulkItem_bulkFailure !== outputg1_g0_bulkItem_bulkCleanupError) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(outputg1_g0_bulkItem_bulkCleanupError) }
outputg1_g0_bulkFailure = outputg1_g0_bulkItem_bulkFailure
} } catch (outputg1_g0_bulkItem_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkItem_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkItem_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkItem_bulkCleanupError) }
            }
 } catch (outputg1_g0_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkCleanupError) }

} } catch (outputg1_g0_bulkCleanupError: Throwable) { if (failure !== outputg1_g0_bulkCleanupError) failure.addSuppressed(outputg1_g0_bulkCleanupError) }

}
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.resources.OwnedResource? = null
var outputg1_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg1_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg1_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s0)
outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
outputg1_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
outputg1_g0_bulkList = kotlin.collections.ArrayList(outputg1_g0_bulkRange!!.length)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputg1_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputg1_g0_bulkReader).int((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
outputg1_g0_bulkItem_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
outputg1_g0_bulkItem_bulkList = kotlin.collections.ArrayList(outputg1_g0_bulkItem_bulkRange!!.length)
for (outputg1_g0_bulkItem_bulkIndex in 0 until outputg1_g0_bulkItem_bulkRange!!.length) {
val outputg1_g0_bulkItem_bulkBase = outputg1_g0_bulkItem_bulkRange!!.offset + outputg1_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
        var outputg1_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_bulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            outputg1_g0_bulkItem_bulkItem_g1_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long))
            }
            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            outputg1_g0_bulkItem_bulkItem_g2_g0_0 = requireNotNull(outputg1_g0_owners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long))
            }

outputg1_g0_bulkItem_bulkList!!.add(io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(outputg1_g0_bulkItem_bulkItem_g1_g0_0) else null, if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(outputg1_g0_bulkItem_bulkItem_g2_g0_0) else null))
}
            }

outputg1_g0_bulkList!!.add(if ((requireNotNull(outputg1_g0_bulkReader).int((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) requireNotNull(outputg1_g0_bulkItem_bulkList) else null)
}

outputg1_g0_owners!!.complete = true
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s2)
            build(requireNotNull(outputg0_0), requireNotNull(outputg1_g0_bulkList), requireNotNull(outputg2_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
if (outputg1_g0_bulkReader == null) { outputg1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s1 as ByteArray)
 }
outputg1_g0_owners?.close(failure)
if (outputg1_g0_owners?.complete != true) {
try { if (outputg1_g0_bulkRange == null) outputg1_g0_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
for (outputg1_g0_bulkIndex in 0 until outputg1_g0_bulkRange!!.length) {
val outputg1_g0_bulkBase = outputg1_g0_bulkRange!!.offset + outputg1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputg1_g0_bulkFailure: Throwable? = failure
try { var outputg1_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg1_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputg1_g0_bulkReader).int((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (outputg1_g0_bulkItem_bulkRange == null) outputg1_g0_bulkItem_bulkRange = requireNotNull(outputg1_g0_bulkReader).rows(((requireNotNull(outputg1_g0_bulkReader).range((outputg1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputg1_g0_bulkItem_bulkIndex in 0 until outputg1_g0_bulkItem_bulkRange!!.length) {
val outputg1_g0_bulkItem_bulkBase = outputg1_g0_bulkItem_bulkRange!!.offset + outputg1_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputg1_g0_bulkItem_bulkFailure: Throwable? = outputg1_g0_bulkFailure
try {         var outputg1_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_bulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = cleanup else if (outputg1_g0_bulkItem_bulkFailure !== cleanup) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputg1_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputg1_g0_bulkReader).long((outputg1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = cleanup else if (outputg1_g0_bulkItem_bulkFailure !== cleanup) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputg1_g0_bulkItem_bulkCleanupError: Throwable) { if (outputg1_g0_bulkItem_bulkFailure == null) outputg1_g0_bulkItem_bulkFailure = outputg1_g0_bulkItem_bulkCleanupError else if (outputg1_g0_bulkItem_bulkFailure !== outputg1_g0_bulkItem_bulkCleanupError) outputg1_g0_bulkItem_bulkFailure!!.addSuppressed(outputg1_g0_bulkItem_bulkCleanupError) }
outputg1_g0_bulkFailure = outputg1_g0_bulkItem_bulkFailure
} } catch (outputg1_g0_bulkItem_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkItem_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkItem_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkItem_bulkCleanupError) }
            }
 } catch (outputg1_g0_bulkCleanupError: Throwable) { if (outputg1_g0_bulkFailure == null) outputg1_g0_bulkFailure = outputg1_g0_bulkCleanupError else if (outputg1_g0_bulkFailure !== outputg1_g0_bulkCleanupError) outputg1_g0_bulkFailure!!.addSuppressed(outputg1_g0_bulkCleanupError) }

} } catch (outputg1_g0_bulkCleanupError: Throwable) { if (failure !== outputg1_g0_bulkCleanupError) failure.addSuppressed(outputg1_g0_bulkCleanupError) }

}
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `outputCleanupRows`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_rows_d61c4972dafc1c15(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (output_row.c1 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row.c1) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            }
            if (output_row.c2 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row.c2) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (output_row.c1 != 0L) {
            outputitem_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource(output_row.c1))
            }
            if (output_row.c2 != 0L) {
            outputitem_g2_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource(output_row.c2))
            }

outputlist.add(io.prebindgen.covertest.cleanup.OutputCleanupRow(output_row.c0, if (output_row.c1 != 0L) requireNotNull(outputitem_g1_g0_0) else null, if (output_row.c2 != 0L) requireNotNull(outputitem_g2_g0_0) else null))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (output_row.c1 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row.c1) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            }
            if (output_row.c2 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row.c2) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `outputCleanupRowsBulk`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_rows_bulk_b9d0f7fac05677c7(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r636e4a114d74a38b.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r636e4a114d74a38b.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
        var outputbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            outputbulkItem_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long))
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            outputbulkItem_g2_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long))
            }

outputbulkList!!.add(io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(outputbulkItem_g1_g0_0) else null, if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(outputbulkItem_g2_g0_0) else null))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r636e4a114d74a38b.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `outputCleanupNested`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_nested_de8d2b4b9cdb3154(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (output_row.c0 != null) {
try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputitem__row.c1 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row.c1) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            }
            if (outputitem__row.c2 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            }

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
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (output_row.c0 != null) {
outputitem_list = kotlin.collections.ArrayList((output_row.c0 as kotlin.Array<*>).size)
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputitem__row.c1 != 0L) {
            outputitem_item_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource(outputitem__row.c1))
            }
            if (outputitem__row.c2 != 0L) {
            outputitem_item_g2_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource(outputitem__row.c2))
            }

outputitem_list.add(io.prebindgen.covertest.cleanup.OutputCleanupRow(outputitem__row.c0, if (outputitem__row.c1 != 0L) requireNotNull(outputitem_item_g1_g0_0) else null, if (outputitem__row.c2 != 0L) requireNotNull(outputitem_item_g2_g0_0) else null))
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
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (output_row.c0 != null) {
try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var outputitem_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputitem_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (outputitem__row.c1 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row.c1) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            }
            if (outputitem__row.c2 != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row.c2) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
            }

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

public fun `outputCleanupNestedBulk`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_nested_bulk_16ea88d5215cb029(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
        var outputbulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            outputbulkItem_bulkItem_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long))
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            outputbulkItem_bulkItem_g2_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long))
            }

outputbulkItem_bulkList!!.add(io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(outputbulkItem_bulkItem_g1_g0_0) else null, if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(outputbulkItem_bulkItem_g2_g0_0) else null))
}
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) requireNotNull(outputbulkItem_bulkList) else null)
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputbulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `outputCleanupCallback`(p0_callback: io.prebindgen.covertest.OutputCleanupCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_owners: io.prebindgen.covertest.CollectionOwners? = null
var callbackOutput0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>
        try {
callbackOutput0_owners = io.prebindgen.covertest.CollectionOwners()
callbackOutput0_list = kotlin.collections.ArrayList((raw0 as kotlin.Array<*>).size)
for (callbackOutput0_index in (raw0 as kotlin.Array<*>).indices) {
val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var callbackOutput0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (callbackOutput0__row.c0 != null) {
callbackOutput0_item_list = kotlin.collections.ArrayList((callbackOutput0__row.c0 as kotlin.Array<*>).size)
for (callbackOutput0_item_index in (callbackOutput0__row.c0 as kotlin.Array<*>).indices) {
val callbackOutput0_item__row = (callbackOutput0__row.c0 as kotlin.Array<*>)[callbackOutput0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var callbackOutput0_item_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var callbackOutput0_item_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (callbackOutput0_item__row.c1 != 0L) {
            callbackOutput0_item_item_g1_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.resources.OwnedResource(callbackOutput0_item__row.c1))
            }
            if (callbackOutput0_item__row.c2 != 0L) {
            callbackOutput0_item_item_g2_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.resources.OwnedResource(callbackOutput0_item__row.c2))
            }

callbackOutput0_item_list.add(io.prebindgen.covertest.cleanup.OutputCleanupRow(callbackOutput0_item__row.c0, if (callbackOutput0_item__row.c1 != 0L) requireNotNull(callbackOutput0_item_item_g1_g0_0) else null, if (callbackOutput0_item__row.c2 != 0L) requireNotNull(callbackOutput0_item_item_g2_g0_0) else null))
}
            }

callbackOutput0_list.add(if (callbackOutput0__row.c0 != null) requireNotNull(callbackOutput0_item_list) else null)
}

callbackOutput0_owners!!.complete = true
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try {
for (callbackOutput0_index in (raw0 as kotlin.Array<*>).indices) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var callbackOutput0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (callbackOutput0__row.c0 != null) {
try {
for (callbackOutput0_item_index in (callbackOutput0__row.c0 as kotlin.Array<*>).indices) {
var callbackOutput0_item_failure: kotlin.Throwable? = callbackOutput0_failure
try { val callbackOutput0_item__row = (callbackOutput0__row.c0 as kotlin.Array<*>)[callbackOutput0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var callbackOutput0_item_item_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var callbackOutput0_item_item_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (callbackOutput0_item__row.c1 != 0L) {
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(callbackOutput0_item__row.c1) } } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            }
            if (callbackOutput0_item__row.c2 != 0L) {
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(callbackOutput0_item__row.c2) } } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            }

} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_item_failure !== callbackOutput0_item_cleanupError) callbackOutput0_item_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }
callbackOutput0_failure = callbackOutput0_item_failure
}
} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_failure !== callbackOutput0_item_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }
            }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }

}
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_callback_3661b832911fa506(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `outputCleanupCallbackBulk`(p0_callback: io.prebindgen.covertest.OutputCleanupBulkCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_owners: io.prebindgen.covertest.CollectionOwners? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_owners = io.prebindgen.covertest.CollectionOwners()
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
callbackOutput0_bulkItem_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkItem_bulkRange!!.length)
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
        var callbackOutput0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var callbackOutput0_bulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            callbackOutput0_bulkItem_bulkItem_g1_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long))
            }
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            callbackOutput0_bulkItem_bulkItem_g2_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long))
            }

callbackOutput0_bulkItem_bulkList!!.add(io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(callbackOutput0_bulkItem_bulkItem_g1_g0_0) else null, if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(callbackOutput0_bulkItem_bulkItem_g2_g0_0) else null))
}
            }

callbackOutput0_bulkList!!.add(if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) requireNotNull(callbackOutput0_bulkItem_bulkList) else null)
}

callbackOutput0_owners!!.complete = true
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ref20bcad2f01452a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try { var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var callbackOutput0_bulkItem_bulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
            }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_callback_bulk_4a1f8c925d7c71c7(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun <A> `outputCleanupRowsFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.cleanup.OutputCleanupRow) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_output_cleanup_rows_fold_a3d8debfd37d4def()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElementg1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementg2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (__pb_fold_row.c1 != 0L) {
            try { val owner = foldElementg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row.c1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (__pb_fold_row.c2 != 0L) {
            try { val owner = foldElementg2_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row.c2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_rows_fold_d4a060fc1b46cffe(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
        try {
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElementg1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementg2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

try {
            if (__pb_fold_row.c1 != 0L) {
            foldElementg1_g0_0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row.c1)
            }
            if (__pb_fold_row.c2 != 0L) {
            foldElementg2_g0_0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row.c2)
            }

current = fold(current, io.prebindgen.covertest.cleanup.OutputCleanupRow(__pb_fold_row.c0, if (__pb_fold_row.c1 != 0L) requireNotNull(foldElementg1_g0_0) else null, if (__pb_fold_row.c2 != 0L) requireNotNull(foldElementg2_g0_0) else null))
} catch (failure: Throwable) {
__pb_processed++
try {
            if (__pb_fold_row.c1 != 0L) {
            try { val owner = foldElementg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row.c1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (__pb_fold_row.c2 != 0L) {
            try { val owner = foldElementg2_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row.c2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }

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

public fun <A> `outputCleanupRowsBulkFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.cleanup.OutputCleanupRow) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_output_cleanup_rows_bulk_fold_aeddb997c707853d()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
        var foldElementg1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementg2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { val owner = foldElementg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { val owner = foldElementg2_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_rows_bulk_fold_8a047108d9782a30(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
        try {
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
        var foldElementg1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementg2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

try {
foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            foldElementg1_g0_0 = io.prebindgen.covertest.resources.OwnedResource((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long)
            }
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            foldElementg2_g0_0 = io.prebindgen.covertest.resources.OwnedResource((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long)
            }

current = fold(current, io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(foldElementg1_g0_0) else null, if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(foldElementg2_g0_0) else null))
} catch (failure: Throwable) {
__pb_processed++
try {
if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { val owner = foldElementg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if ((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { val owner = foldElementg2_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }

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

public fun <A> `outputCleanupNestedFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_output_cleanup_nested_fold_f1c07d387aa43c20()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementowners: io.prebindgen.covertest.CollectionOwners? = null
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

            if (__pb_fold_row.c0 != null) {
foldElementowners?.close(failure)
if (foldElementowners?.complete != true) {
try {
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var foldElementitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (foldElement_row.c1 != 0L) {
            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row.c1) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
            }
            if (foldElement_row.c2 != 0L) {
            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row.c2) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
            }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}
            }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_nested_fold_4f16ec74e528db69(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_foldOwners: io.prebindgen.covertest.CollectionOwners? = null
        try {
if (nativeResult == 0) {
if (__pb_rows.size != 0) __pb_foldOwners = io.prebindgen.covertest.CollectionOwners()
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

try {
            if (__pb_fold_row.c0 != null) {
foldElementlist = kotlin.collections.ArrayList((__pb_fold_row.c0 as kotlin.Array<*>).size)
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var foldElementitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (foldElement_row.c1 != 0L) {
            foldElementitem_g1_g0_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.resources.OwnedResource(foldElement_row.c1))
            }
            if (foldElement_row.c2 != 0L) {
            foldElementitem_g2_g0_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.resources.OwnedResource(foldElement_row.c2))
            }

foldElementlist.add(io.prebindgen.covertest.cleanup.OutputCleanupRow(foldElement_row.c0, if (foldElement_row.c1 != 0L) requireNotNull(foldElementitem_g1_g0_0) else null, if (foldElement_row.c2 != 0L) requireNotNull(foldElementitem_g2_g0_0) else null))
}
            }

__pb_foldOwners!!.complete = true
current = fold(current, if (__pb_fold_row.c0 != null) requireNotNull(foldElementlist) else null)
} catch (failure: Throwable) {
__pb_processed++
try {
__pb_foldOwners?.close(failure)
if (__pb_foldOwners?.complete != true) {
            if (__pb_fold_row.c0 != null) {
try {
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Long_3_ad466713697fa9e4
        var foldElementitem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementitem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if (foldElement_row.c1 != 0L) {
            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row.c1) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
            }
            if (foldElement_row.c2 != 0L) {
            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row.c2) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
            }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }
            }

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
if (nativeResult == 0) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `outputCleanupNestedBulkFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.cleanup.OutputCleanupRow>?) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_output_cleanup_nested_bulk_fold_f9eaa6c74f7feefb()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
var foldElementowners: io.prebindgen.covertest.CollectionOwners? = null
var foldElementbulkRange: io.prebindgen.covertest.BulkRange? = null
var foldElementbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
            if ((requireNotNull(foldElementbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
foldElementowners?.close(failure)
if (foldElementowners?.complete != true) {
try { if (foldElementbulkRange == null) foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var foldElementbulkFailure: Throwable? = failure
try {         var foldElementbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (foldElementowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (foldElementbulkCleanupError: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = foldElementbulkCleanupError else if (foldElementbulkFailure !== foldElementbulkCleanupError) foldElementbulkFailure!!.addSuppressed(foldElementbulkCleanupError) }

} } catch (foldElementbulkCleanupError: Throwable) { if (failure !== foldElementbulkCleanupError) failure.addSuppressed(foldElementbulkCleanupError) }

}
            }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_output_cleanup_nested_bulk_fold_2c8fd0b69d9609fe(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_foldOwners: io.prebindgen.covertest.CollectionOwners? = null
        try {
if (nativeResult == 0) {
if (__pb_rows.size != 0) __pb_foldOwners = io.prebindgen.covertest.CollectionOwners()
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementbulkReader: io.prebindgen.covertest.BulkReader? = null
var foldElementbulkRange: io.prebindgen.covertest.BulkRange? = null
var foldElementbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.cleanup.OutputCleanupRow>? = null

try {
foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
            if ((requireNotNull(foldElementbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
foldElementbulkList = kotlin.collections.ArrayList(foldElementbulkRange!!.length)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
        var foldElementbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            foldElementbulkItem_g1_g0_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long))
            }
            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            foldElementbulkItem_g2_g0_0 = requireNotNull(__pb_foldOwners).own(io.prebindgen.covertest.resources.OwnedResource((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long))
            }

foldElementbulkList!!.add(io.prebindgen.covertest.cleanup.OutputCleanupRow((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[0])) as Long, if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) requireNotNull(foldElementbulkItem_g1_g0_0) else null, if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) requireNotNull(foldElementbulkItem_g2_g0_0) else null))
}
            }

__pb_foldOwners!!.complete = true
current = fold(current, if ((requireNotNull(foldElementbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) requireNotNull(foldElementbulkList) else null)
} catch (failure: Throwable) {
__pb_processed++
try {
__pb_foldOwners?.close(failure)
if (__pb_foldOwners?.complete != true) {
if (foldElementbulkReader == null) { foldElementbulkReader = io.prebindgen.covertest.BulkReader(__pb_fold_row.c0 as ByteArray)
 }
            if ((requireNotNull(foldElementbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[0])) == 1) {
try { if (foldElementbulkRange == null) foldElementbulkRange = requireNotNull(foldElementbulkReader).rows(((requireNotNull(foldElementbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rcd29b859e7eb9df2.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size)
for (foldElementbulkIndex in 0 until foldElementbulkRange!!.length) {
val foldElementbulkBase = foldElementbulkRange!!.offset + foldElementbulkIndex * io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.size
var foldElementbulkFailure: Throwable? = failure
try {         var foldElementbulkItem_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var foldElementbulkItem_g2_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) != 0L) {
            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) != 0L) {
            try { if (__pb_foldOwners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88((requireNotNull(foldElementbulkReader).long((foldElementbulkBase) + io.prebindgen.covertest.BulkLayouts.r11b10e2543ffa977.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = cleanup else if (foldElementbulkFailure !== cleanup) foldElementbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (foldElementbulkCleanupError: Throwable) { if (foldElementbulkFailure == null) foldElementbulkFailure = foldElementbulkCleanupError else if (foldElementbulkFailure !== foldElementbulkCleanupError) foldElementbulkFailure!!.addSuppressed(foldElementbulkCleanupError) }

} } catch (foldElementbulkCleanupError: Throwable) { if (failure !== foldElementbulkCleanupError) failure.addSuppressed(foldElementbulkCleanupError) }
            }

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
if (nativeResult == 0) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}
