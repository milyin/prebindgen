@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package bench.plain

public fun inputHandleNew(`value`: Long, onError: bench.plain.JniErrorHandler<bench.plain.InputHandle> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.InputHandle = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_input_handle_new_d7aa473c3cabcfa5(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.InputHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.InputHandle? = null
        try {
            output0 = bench.plain.InputHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun inputHandleValue(`value`: bench.plain.InputHandle, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { `value`.raw } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_input_handle_value_9dacbaa67102b1db(nativeArgument0, nativeArgument1) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bufferSizes(text: String, bytes: ByteArray, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { text } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { bytes } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_buffer_sizes_09330eed51da0791(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun selectedHandle(valueVariant: Int, valueVariant0: Long, valueVariant1Value: bench.plain.InputHandle?, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valueVariant == 1) requireNotNull(valueVariant1Value) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { valueVariant0 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valueVariant == 1) requireNotNull(valueVariant1Value) { "null handle" }.raw else 0L } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_selected_handle_0b5f8bb745f1d664(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handleValuesSum(values: kotlin.collections.List<bench.plain.InputHandle>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w6 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w6 }) }, intArrayOf(1)) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_handle_values_sum_cc53b2df8a803a98(nativeArgument0, nativeArgument1) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handleRecordsSum(values: kotlin.collections.List<bench.plain.HandleRecord>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5.handle }, bench.plain.InputValidation.value { p0_w5.tag }) }, intArrayOf(1, 0)) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_handle_records_sum_cabe62d5ee2df7d3(nativeArgument0, nativeArgument1) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun ownedHandlesEcho(values: kotlin.collections.List<bench.plain.InputHandle>, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.InputHandle>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.InputHandle> = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5 }) }, intArrayOf(2)) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_owned_handles_echo_951038e1201850d3(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.plain.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<bench.plain.InputHandle>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<bench.plain.InputHandle>? = null
        try {
outputowners = bench.plain.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_0: bench.plain.InputHandle? = null

            outputitem_0 = requireNotNull(outputowners).own(bench.plain.InputHandle(output_row.c0))

outputlist.add(requireNotNull(outputitem_0))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun ownedHandleRecordsEcho(values: kotlin.collections.List<bench.plain.HandleRecord>, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.HandleRecord>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.HandleRecord> = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5.handle }, bench.plain.InputValidation.value { p0_w5.tag }) }, intArrayOf(2, 0)) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_owned_handle_records_echo_afd2d16618028a6f(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.plain.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<bench.plain.HandleRecord>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_Long_2_c3e8677e51c52ca2
        var outputitem_g0_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<bench.plain.HandleRecord>? = null
        try {
outputowners = bench.plain.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_Long_2_c3e8677e51c52ca2
        var outputitem_g0_0: bench.plain.InputHandle? = null

            outputitem_g0_0 = requireNotNull(outputowners).own(bench.plain.InputHandle(output_row.c0))

outputlist.add(bench.plain.HandleRecord(requireNotNull(outputitem_g0_0), output_row.c1))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_Long_2_c3e8677e51c52ca2
        var outputitem_g0_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun nestedOwnedHandlesEcho(values: kotlin.collections.List<kotlin.collections.List<bench.plain.InputHandle>>, onError: bench.plain.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<bench.plain.InputHandle>>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<bench.plain.InputHandle>> = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
val p0_w5_elements = if (true) p0_w5?.toList()?.let { p0_w5_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w5_snapshot.size) { p0_w5_index -> val p0_w9 = p0_w5_snapshot[p0_w5_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w9 }) }, intArrayOf(2)) } else null
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5_elements }) }, null) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_nested_owned_handles_echo_47d054d07c837967(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.plain.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<bench.plain.InputHandle>>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<bench.plain.InputHandle>? = null

try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_item_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: bench.plain.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<kotlin.collections.List<bench.plain.InputHandle>>? = null
        try {
outputowners = bench.plain.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<bench.plain.InputHandle>? = null

outputitem_list = kotlin.collections.ArrayList((output_row.c0 as kotlin.Array<*>).size)
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_item_0: bench.plain.InputHandle? = null

            outputitem_item_0 = requireNotNull(outputowners).own(bench.plain.InputHandle(outputitem__row.c0))

outputitem_list.add(requireNotNull(outputitem_item_0))
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
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputitem_list: kotlin.collections.MutableList<bench.plain.InputHandle>? = null

try {
for (outputitem_index in (output_row.c0 as kotlin.Array<*>).indices) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row.c0 as kotlin.Array<*>)[outputitem_index] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var outputitem_item_0: bench.plain.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(outputitem__row.c0) } } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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

public fun rowMake(seed: Long, onError: bench.plain.JniErrorHandler<bench.plain.Row> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.Row = run {
    val returnCapture = bench.plain.Return_Long_Long_Long8_ErrorEmpty_f77363a35e75c509.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_make_bcd43e7b136151dd(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    bench.plain.Row(returnCapture.s0, returnCapture.s1, returnCapture.s2, returnCapture.s3, returnCapture.s4, returnCapture.s5, returnCapture.s6, returnCapture.s7)
    } finally { returnCapture.release() }
}

public fun wideRowMake(seed: Long, onError: bench.plain.JniErrorHandler<bench.plain.WideRow> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.WideRow = run {
    val returnCapture = bench.plain.Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_make_1dbd8ca2fd4616ac(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    bench.plain.WideRow(returnCapture.s0, returnCapture.s1, returnCapture.s2, returnCapture.s3, returnCapture.s4, returnCapture.s5, returnCapture.s6, returnCapture.s7, returnCapture.s8, returnCapture.s9, returnCapture.s10, returnCapture.s11, returnCapture.s12, returnCapture.s13, returnCapture.s14, returnCapture.s15, returnCapture.s16, returnCapture.s17, returnCapture.s18, returnCapture.s19, returnCapture.s20, returnCapture.s21, returnCapture.s22, returnCapture.s23, returnCapture.s24, returnCapture.s25, returnCapture.s26, returnCapture.s27, returnCapture.s28, returnCapture.s29, returnCapture.s30, returnCapture.s31)
    } finally { returnCapture.release() }
}

public fun wideRowSum(`value`: bench.plain.WideRow, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value`.f0 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.f1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { `value`.f2 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { `value`.f3 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { `value`.f4 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { `value`.f5 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { `value`.f6 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { `value`.f7 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { `value`.f8 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument9 = try { `value`.f9 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument10 = try { `value`.f10 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument11 = try { `value`.f11 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument12 = try { `value`.f12 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument13 = try { `value`.f13 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument14 = try { `value`.f14 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument15 = try { `value`.f15 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument16 = try { `value`.f16 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument17 = try { `value`.f17 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument18 = try { `value`.f18 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument19 = try { `value`.f19 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument20 = try { `value`.f20 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument21 = try { `value`.f21 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument22 = try { `value`.f22 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument23 = try { `value`.f23 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument24 = try { `value`.f24 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument25 = try { `value`.f25 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument26 = try { `value`.f26 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument27 = try { `value`.f27 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument28 = try { `value`.f28 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument29 = try { `value`.f29 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument30 = try { `value`.f30 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument31 = try { `value`.f31 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument32 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_sum_861bdfef124a25ad(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8, nativeArgument9, nativeArgument10, nativeArgument11, nativeArgument12, nativeArgument13, nativeArgument14, nativeArgument15, nativeArgument16, nativeArgument17, nativeArgument18, nativeArgument19, nativeArgument20, nativeArgument21, nativeArgument22, nativeArgument23, nativeArgument24, nativeArgument25, nativeArgument26, nativeArgument27, nativeArgument28, nativeArgument29, nativeArgument30, nativeArgument31, nativeArgument32) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowReturnHandlerNew(callback: bench.plain.RowReturnCallback, onError: bench.plain.JniErrorHandler<bench.plain.RowReturnHandler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.RowReturnHandler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.plain.Row
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.a, callback_return_w1.b, callback_return_w1.c, callback_return_w1.d, callback_return_w1.e, callback_return_w1.f, callback_return_w1.g, callback_return_w1.h)
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_return_handler_new_9a8fcadcab7b9640(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.RowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.RowReturnHandler? = null
        try {
            output0 = bench.plain.RowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wideRowReturnHandlerNew(callback: bench.plain.WideRowReturnCallback, onError: bench.plain.JniErrorHandler<bench.plain.WideRowReturnHandler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.WideRowReturnHandler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.plain.WideRow
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.f0, callback_return_w1.f1, callback_return_w1.f2, callback_return_w1.f3, callback_return_w1.f4, callback_return_w1.f5, callback_return_w1.f6, callback_return_w1.f7, callback_return_w1.f8, callback_return_w1.f9, callback_return_w1.f10, callback_return_w1.f11, callback_return_w1.f12, callback_return_w1.f13, callback_return_w1.f14, callback_return_w1.f15, callback_return_w1.f16, callback_return_w1.f17, callback_return_w1.f18, callback_return_w1.f19, callback_return_w1.f20, callback_return_w1.f21, callback_return_w1.f22, callback_return_w1.f23, callback_return_w1.f24, callback_return_w1.f25, callback_return_w1.f26, callback_return_w1.f27, callback_return_w1.f28, callback_return_w1.f29, callback_return_w1.f30, callback_return_w1.f31)
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_return_handler_new_67f38d7168020371(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.WideRowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.WideRowReturnHandler? = null
        try {
            output0 = bench.plain.WideRowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun rowReturnFire(handler: bench.plain.RowReturnHandler, seed: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_return_fire_7c8a1183dcdd277c(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun wideRowReturnFire(handler: bench.plain.WideRowReturnHandler, seed: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_return_fire_88ad5bbf9c8c73e2(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun control(`value`: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_control_b0073ee32096caea(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun scalar(`value`: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_scalar_a4a5da3f79ebbcaf(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowSum(`value`: bench.plain.Row, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value`.a } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.b } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { `value`.c } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { `value`.d } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { `value`.e } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { `value`.f } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { `value`.g } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { `value`.h } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_sum_db65517187cece4c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun numbersSum(values: kotlin.collections.List<Long>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5 }) } } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_numbers_sum_47543406a5cdf588(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowsSum(values: kotlin.collections.List<bench.plain.Row>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5.a }, bench.plain.InputValidation.value { p0_w5.b }, bench.plain.InputValidation.value { p0_w5.c }, bench.plain.InputValidation.value { p0_w5.d }, bench.plain.InputValidation.value { p0_w5.e }, bench.plain.InputValidation.value { p0_w5.f }, bench.plain.InputValidation.value { p0_w5.g }, bench.plain.InputValidation.value { p0_w5.h }) } } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_sum_d4a5ecfa43bf26b9(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowsMake(count: Long, seed: Long, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.Row>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.Row> = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { count } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_make_6f7eb2a3ee93e0ae(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a


outputlist.add(bench.plain.Row(output_row.c0, output_row.c1, output_row.c2, output_row.c3, output_row.c4, output_row.c5, output_row.c6, output_row.c7))
}

requireNotNull(outputlist)
    } finally { returnCapture.release() }
}

public fun rowsEcho(values: kotlin.collections.List<bench.plain.Row>, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.Row>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.Row> = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5.a }, bench.plain.InputValidation.value { p0_w5.b }, bench.plain.InputValidation.value { p0_w5.c }, bench.plain.InputValidation.value { p0_w5.d }, bench.plain.InputValidation.value { p0_w5.e }, bench.plain.InputValidation.value { p0_w5.f }, bench.plain.InputValidation.value { p0_w5.g }, bench.plain.InputValidation.value { p0_w5.h }) } } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_echo_5c26c8cf4751c454(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as bench.plain.JNINative.SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a


outputlist.add(bench.plain.Row(output_row.c0, output_row.c1, output_row.c2, output_row.c3, output_row.c4, output_row.c5, output_row.c6, output_row.c7))
}

requireNotNull(outputlist)
    } finally { returnCapture.release() }
}

public fun packetSum(packet: bench.plain.Packet, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { packet.header.a } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { packet.header.b } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { packet.header.c } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { packet.header.d } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { packet.header.e } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { packet.header.f } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { packet.header.g } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { packet.header.h } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { packet.bytes } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument9 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_packet_sum_c07ede9eb5de050f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8, nativeArgument9) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handlerNew(callback: bench.plain.RowsCallback, onError: bench.plain.JniErrorHandler<bench.plain.Handler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.Handler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?): Long {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<bench.plain.Row>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<bench.plain.Row>
        var callbackReturned: Long = 0L
        try {
callbackOutput0_list = kotlin.collections.ArrayList((raw0 as kotlin.Array<*>).size)
for (callbackOutput0_index in (raw0 as kotlin.Array<*>).indices) {
val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as bench.plain.JNINative.SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a


callbackOutput0_list.add(bench.plain.Row(callbackOutput0__row.c0, callbackOutput0__row.c1, callbackOutput0__row.c2, callbackOutput0__row.c3, callbackOutput0__row.c4, callbackOutput0__row.c5, callbackOutput0__row.c6, callbackOutput0__row.c7))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_handler_new_916a49ec8343b596(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.Handler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.Handler? = null
        try {
            output0 = bench.plain.Handler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun fire(handler: bench.plain.Handler, count: Long, seed: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { count } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_fire_6fdab6ebdaeffb11(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <A> ownedHandlesEchoFold(values: kotlin.collections.List<bench.plain.InputHandle>, initial: A, onError: bench.plain.JniErrorHandler<A> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, bench.plain.InputHandle) -> A): A = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5 }) }, intArrayOf(2)) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_sink = bench.plain.JNINative.SequenceCollector_owned_handles_echo_fold_8c0e0fda84512cef()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElement0: bench.plain.InputHandle? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_owned_handles_echo_fold_769ead1c15ceb95c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.plain.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
        try {
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
        var foldElement0: bench.plain.InputHandle? = null

try {
            foldElement0 = bench.plain.InputHandle(__pb_fold_row.c0)

current = fold(current, requireNotNull(foldElement0))
} catch (failure: Throwable) {
__pb_processed++
try {
            try { val owner = foldElement0; if (owner != null) owner.close() else bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(__pb_fold_row.c0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

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

public fun <A> nestedOwnedHandlesEchoFold(values: kotlin.collections.List<kotlin.collections.List<bench.plain.InputHandle>>, initial: A, onError: bench.plain.JniErrorHandler<A> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<bench.plain.InputHandle>) -> A): A = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
val p0_w5_elements = if (true) p0_w5?.toList()?.let { p0_w5_snapshot -> bench.plain.InputRows(kotlin.Array<kotlin.Any?>(p0_w5_snapshot.size) { p0_w5_index -> val p0_w9 = p0_w5_snapshot[p0_w5_index]
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w9 }) }, intArrayOf(2)) } else null
kotlin.arrayOf<kotlin.Any?>(bench.plain.InputValidation.value { p0_w5_elements }) }, null) } else null } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_sink = bench.plain.JNINative.SequenceCollector_nested_owned_handles_echo_fold_9847d4ca29b981db()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable, start: Int = 0) {
for (__pb_rowIndex in start until __pb_rows.size) {
try {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementowners: bench.plain.CollectionOwners? = null
var foldElementlist: kotlin.collections.MutableList<bench.plain.InputHandle>? = null

foldElementowners?.close(failure)
if (foldElementowners?.complete != true) {
try {
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: bench.plain.InputHandle? = null

            try { if (foldElementowners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(foldElement_row.c0) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}

} catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
}
}
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (p0_w1_elements?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_nested_owned_handles_echo_fold_55ebe6c8ba226f50(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.plain.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_foldOwners: bench.plain.CollectionOwners? = null
        try {
if (nativeResult == 0) {
if (__pb_rows.size != 0) __pb_foldOwners = bench.plain.CollectionOwners()
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]
var foldElementlist: kotlin.collections.MutableList<bench.plain.InputHandle>? = null

try {
foldElementlist = kotlin.collections.ArrayList((__pb_fold_row.c0 as kotlin.Array<*>).size)
for (foldElementindex in (__pb_fold_row.c0 as kotlin.Array<*>).indices) {
val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: bench.plain.InputHandle? = null

            foldElementitem_0 = requireNotNull(__pb_foldOwners).own(bench.plain.InputHandle(foldElement_row.c0))

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
try { val foldElement_row = (__pb_fold_row.c0 as kotlin.Array<*>)[foldElementindex] as bench.plain.JNINative.SequenceRow_Long_1_127a3c24b600dbe0
        var foldElementitem_0: bench.plain.InputHandle? = null

            try { if (__pb_foldOwners?.unread() != false) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(foldElement_row.c0) } } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

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
if (nativeResult == 0) __pb_discard(failure, __pb_processed)
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> rowsForReduceFold(count: Long, seed: Long, initial: A, onError: bench.plain.JniErrorHandler<A> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, bench.plain.Row) -> A): A = run {
    val __pb_sink = bench.plain.JNINative.SequenceCollector_rows_for_reduce_fold_bac33805078e0166()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { count } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_for_reduce_fold_587a8aef1d9a0485(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]


current = fold(current, bench.plain.Row(__pb_fold_row.c0, __pb_fold_row.c1, __pb_fold_row.c2, __pb_fold_row.c3, __pb_fold_row.c4, __pb_fold_row.c5, __pb_fold_row.c6, __pb_fold_row.c7))
}
__pb_accumulator = current
}

__pb_accumulator as A
    } finally { returnCapture.release() }
}
