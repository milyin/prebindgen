@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package bench.bulk

public fun inputHandleNew(`value`: Long, onError: bench.bulk.JniErrorHandler<bench.bulk.InputHandle> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.InputHandle = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_input_handle_new_d7aa473c3cabcfa5(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.bulk.InputHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.bulk.InputHandle? = null
        try {
            output0 = bench.bulk.InputHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun inputHandleValue(`value`: bench.bulk.InputHandle, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { `value`.raw } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_input_handle_value_9dacbaa67102b1db(nativeArgument0, nativeArgument1) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bufferSizes(text: String, bytes: ByteArray, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { text } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { bytes } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_buffer_sizes_09330eed51da0791(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun selectedHandle(valueVariant: Int, valueVariant0: Long, valueVariant1Value: bench.bulk.InputHandle?, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valueVariant == 1) requireNotNull(valueVariant1Value) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { valueVariant0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valueVariant == 1) requireNotNull(valueVariant1Value) { "null handle" }.raw else 0L } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_selected_handle_0b5f8bb745f1d664(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handleValuesSum(values: kotlin.collections.List<bench.bulk.InputHandle>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_f738f16d97296060()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rf738f16d97296060.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.rd159346acd2a185e)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rf738f16d97296060.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.rd159346acd2a185e.size) + bench.bulk.BulkLayouts.rd159346acd2a185e.offsets[0], bench.bulk.InputValidation.value { p0_w9 }, false)
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_handle_values_sum_cc53b2df8a803a98(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handleRecordsSum(values: kotlin.collections.List<bench.bulk.HandleRecord>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_cb0e46a0a248e38c()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rcb0e46a0a248e38c.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r06de1c5aa99699f6)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rcb0e46a0a248e38c.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r06de1c5aa99699f6.size) + bench.bulk.BulkLayouts.r06de1c5aa99699f6.offsets[0], bench.bulk.InputValidation.value { p0_w7.handle }, false)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r06de1c5aa99699f6.size) + bench.bulk.BulkLayouts.r06de1c5aa99699f6.offsets[1], bench.bulk.InputValidation.value { p0_w7.tag })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_handle_records_sum_cabe62d5ee2df7d3(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun ownedHandlesEcho(values: kotlin.collections.List<bench.bulk.InputHandle>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.InputHandle>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.InputHandle> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_49da87bb4c377b9a()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.rb2426f0d5f2715c0)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0], bench.bulk.InputValidation.value { p0_w8 }, true)
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_owned_handles_echo_951038e1201850d3(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
        var outputbulkItem_0: bench.bulk.InputHandle? = null

            outputbulkItem_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long))

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun ownedHandleRecordsEcho(values: kotlin.collections.List<bench.bulk.HandleRecord>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.HandleRecord>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.HandleRecord> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_71300b84e922dbba()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.rb3c08eb92fbbc74c)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], bench.bulk.InputValidation.value { p0_w7.handle }, true)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1], bench.bulk.InputValidation.value { p0_w7.tag })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_owned_handle_records_echo_afd2d16618028a6f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.HandleRecord>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.HandleRecord>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
        var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long))

outputbulkList!!.add(bench.bulk.HandleRecord(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1])) as Long))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackOwnedRecords(callback: bench.bulk.OwnedRecordsCallback, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.HandleRecord>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.HandleRecord> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<bench.bulk.HandleRecord>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = bench.bulk.BulkInputs.Input_Range_71300b84e922dbba()
val callback_return_w3_bulkList = bench.bulk.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, bench.bulk.BulkLayouts.rb3c08eb92fbbc74c)
callback_return_w1_writer.range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], bench.bulk.InputValidation.value { callback_return_w7.handle }, true)
callback_return_w1_writer.long((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1], bench.bulk.InputValidation.value { callback_return_w7.tag })
}
}

callback_return_w1_writer } else null
 bench.bulk.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? bench.bulk.HandleRecord)?.let { returnedValue2 -> (returnedValue2.handle as? bench.bulk.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_callback_owned_records_d83c514a055c27ec(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.HandleRecord>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.HandleRecord>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
        var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long))

outputbulkList!!.add(bench.bulk.HandleRecord(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1])) as Long))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackOwnedRecord(callback: bench.bulk.OwnedRecordCallback, onError: bench.bulk.JniErrorHandler<bench.bulk.HandleRecord> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.HandleRecord = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: bench.bulk.HandleRecord
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = bench.bulk.BulkInputs.Input_i64_i64_b3c08eb92fbbc74c()
callback_return_w1_writer.handle((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], bench.bulk.InputValidation.value { callback_return_w3.handle }, true)
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1], bench.bulk.InputValidation.value { callback_return_w3.tag })

callback_return_w1_writer } else null
 bench.bulk.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); (callback_return_w3 as? bench.bulk.HandleRecord)?.let { returnedValue1 -> (returnedValue1.handle as? bench.bulk.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_callback_owned_record_990f8f2a11136fc7(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
        var outputg0_0: bench.bulk.InputHandle? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
        var outputg0_0: bench.bulk.InputHandle? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
            outputg0_0 = bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long)
            bench.bulk.HandleRecord(requireNotNull(outputg0_0), (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[1])) as Long)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun ownedBufferRecordsEcho(values: kotlin.collections.List<bench.bulk.BufferRecord>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.BufferRecord>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.BufferRecord> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_50871c2a8b72df4b()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r9cc3e9775849e9ab)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[0], bench.bulk.InputValidation.value { p0_w7.handle }, true)
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[1], bench.bulk.InputValidation.value { p0_w7.text })
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[2], bench.bulk.InputValidation.value { p0_w7.bytes })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_owned_buffer_records_echo_d2839a0a4452a203(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.BufferRecord>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.BufferRecord>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size
        var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[0])) as Long))

outputbulkList!!.add(bench.bulk.BufferRecord(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkBase) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[1])) as String, (requireNotNull(outputbulkReader).bytes((outputbulkBase) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[2])) as ByteArray))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun nestedOwnedHandlesEcho(values: kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkInputs.Input_Range_5e24e8a1a6fd1960()
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r49da87bb4c377b9a)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
val p0_w8_elements = if (true) p0_w8?.toList() else null
val p0_w8_bulkList = bench.bulk.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, bench.bulk.BulkLayouts.rb2426f0d5f2715c0)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w13 ->
p0_w1_writer.handle((p0_w8_bulkStart + p0_w8_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0], bench.bulk.InputValidation.value { p0_w13 }, true)
}
}
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_nested_owned_handles_echo_47d054d07c837967(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<bench.bulk.InputHandle>>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<bench.bulk.InputHandle>>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
        var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            outputbulkItem_bulkItem_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long))

outputbulkItem_bulkList!!.add(requireNotNull(outputbulkItem_bulkItem_0))
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
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

public fun callbackNestedOwnedHandles(callback: bench.bulk.OwnedNestedCallback, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<bench.bulk.InputHandle>>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = bench.bulk.BulkInputs.Input_Range_5e24e8a1a6fd1960()
val callback_return_w3_bulkList = bench.bulk.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, bench.bulk.BulkLayouts.r49da87bb4c377b9a)
callback_return_w1_writer.range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val callback_return_w7_elements = if (true) callback_return_w7?.toList() else null
val callback_return_w7_bulkList = bench.bulk.InputValidation.value { callback_return_w7_elements }
if (callback_return_w7_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]) } else {
val callback_return_w7_bulkStart = callback_return_w1_writer.rows(callback_return_w7_bulkList.size, bench.bulk.BulkLayouts.rb2426f0d5f2715c0)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0], callback_return_w7_bulkStart, callback_return_w7_bulkList.size)
callback_return_w7_bulkList.forEachIndexed { callback_return_w7_bulkIndex, callback_return_w12 ->
callback_return_w1_writer.handle((callback_return_w7_bulkStart + callback_return_w7_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0], bench.bulk.InputValidation.value { callback_return_w12 }, true)
}
}
}
}

callback_return_w1_writer } else null
 bench.bulk.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<bench.bulk.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? bench.bulk.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_callback_nested_owned_handles_af3eabfc6c44ae0b(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<bench.bulk.InputHandle>>? = null
 if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputowners: bench.bulk.CollectionOwners? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<bench.bulk.InputHandle>>? = null
        try {
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputowners = bench.bulk.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
        var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            outputbulkItem_bulkItem_0 = requireNotNull(outputowners).own(bench.bulk.InputHandle((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long))

outputbulkItem_bulkList!!.add(requireNotNull(outputbulkItem_bulkItem_0))
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r49da87bb4c377b9a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: bench.bulk.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<bench.bulk.InputHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: bench.bulk.InputHandle? = null

            try { if (outputowners?.unread() != false) { bench.bulk.JNINative.drop_InputHandle_7cc67535bbd4a8ee((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
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

public fun rowMake(seed: Long, onError: bench.bulk.JniErrorHandler<bench.bulk.Row> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.Row = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_row_make_bcd43e7b136151dd(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)

bench.bulk.Row((requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long)
    } finally { returnCapture.release() }
}

public fun wideRowMake(seed: Long, onError: bench.bulk.JniErrorHandler<bench.bulk.WideRow> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.WideRow = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_wide_row_make_1dbd8ca2fd4616ac(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)

bench.bulk.WideRow((requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[7])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[8])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[9])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[10])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[11])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[12])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[13])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[14])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[15])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[16])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[17])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[18])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[19])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[20])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[21])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[22])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[23])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[24])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[25])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[26])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[27])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[28])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[29])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[30])) as Long, (requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[31])) as Long)
    } finally { returnCapture.release() }
}

public fun wideRowSum(`value`: bench.bulk.WideRow, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r633c6cf82d3acfc1.size)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[0], bench.bulk.InputValidation.value { `value`.f0 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[1], bench.bulk.InputValidation.value { `value`.f1 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[2], bench.bulk.InputValidation.value { `value`.f2 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[3], bench.bulk.InputValidation.value { `value`.f3 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[4], bench.bulk.InputValidation.value { `value`.f4 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[5], bench.bulk.InputValidation.value { `value`.f5 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[6], bench.bulk.InputValidation.value { `value`.f6 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[7], bench.bulk.InputValidation.value { `value`.f7 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[8], bench.bulk.InputValidation.value { `value`.f8 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[9], bench.bulk.InputValidation.value { `value`.f9 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[10], bench.bulk.InputValidation.value { `value`.f10 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[11], bench.bulk.InputValidation.value { `value`.f11 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[12], bench.bulk.InputValidation.value { `value`.f12 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[13], bench.bulk.InputValidation.value { `value`.f13 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[14], bench.bulk.InputValidation.value { `value`.f14 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[15], bench.bulk.InputValidation.value { `value`.f15 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[16], bench.bulk.InputValidation.value { `value`.f16 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[17], bench.bulk.InputValidation.value { `value`.f17 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[18], bench.bulk.InputValidation.value { `value`.f18 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[19], bench.bulk.InputValidation.value { `value`.f19 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[20], bench.bulk.InputValidation.value { `value`.f20 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[21], bench.bulk.InputValidation.value { `value`.f21 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[22], bench.bulk.InputValidation.value { `value`.f22 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[23], bench.bulk.InputValidation.value { `value`.f23 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[24], bench.bulk.InputValidation.value { `value`.f24 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[25], bench.bulk.InputValidation.value { `value`.f25 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[26], bench.bulk.InputValidation.value { `value`.f26 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[27], bench.bulk.InputValidation.value { `value`.f27 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[28], bench.bulk.InputValidation.value { `value`.f28 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[29], bench.bulk.InputValidation.value { `value`.f29 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[30], bench.bulk.InputValidation.value { `value`.f30 })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[31], bench.bulk.InputValidation.value { `value`.f31 })

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_wide_row_sum_861bdfef124a25ad(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowReturnHandlerNew(callback: bench.bulk.RowReturnCallback, onError: bench.bulk.JniErrorHandler<bench.bulk.RowReturnHandler> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.RowReturnHandler = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.bulk.Row
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w3 = callbackReturned
 val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r789cd54eb117685f.size)
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], bench.bulk.InputValidation.value { callback_return_w3.a })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], bench.bulk.InputValidation.value { callback_return_w3.b })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], bench.bulk.InputValidation.value { callback_return_w3.c })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], bench.bulk.InputValidation.value { callback_return_w3.d })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], bench.bulk.InputValidation.value { callback_return_w3.e })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], bench.bulk.InputValidation.value { callback_return_w3.f })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], bench.bulk.InputValidation.value { callback_return_w3.g })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], bench.bulk.InputValidation.value { callback_return_w3.h })

callback_return_w1_writer } else null
 val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.array, callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_row_return_handler_new_9a8fcadcab7b9640(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.bulk.RowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.bulk.RowReturnHandler? = null
        try {
            output0 = bench.bulk.RowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wideRowReturnHandlerNew(callback: bench.bulk.WideRowReturnCallback, onError: bench.bulk.JniErrorHandler<bench.bulk.WideRowReturnHandler> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.WideRowReturnHandler = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.bulk.WideRow
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w3 = callbackReturned
 val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r633c6cf82d3acfc1.size)
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[0], bench.bulk.InputValidation.value { callback_return_w3.f0 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[1], bench.bulk.InputValidation.value { callback_return_w3.f1 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[2], bench.bulk.InputValidation.value { callback_return_w3.f2 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[3], bench.bulk.InputValidation.value { callback_return_w3.f3 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[4], bench.bulk.InputValidation.value { callback_return_w3.f4 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[5], bench.bulk.InputValidation.value { callback_return_w3.f5 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[6], bench.bulk.InputValidation.value { callback_return_w3.f6 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[7], bench.bulk.InputValidation.value { callback_return_w3.f7 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[8], bench.bulk.InputValidation.value { callback_return_w3.f8 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[9], bench.bulk.InputValidation.value { callback_return_w3.f9 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[10], bench.bulk.InputValidation.value { callback_return_w3.f10 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[11], bench.bulk.InputValidation.value { callback_return_w3.f11 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[12], bench.bulk.InputValidation.value { callback_return_w3.f12 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[13], bench.bulk.InputValidation.value { callback_return_w3.f13 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[14], bench.bulk.InputValidation.value { callback_return_w3.f14 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[15], bench.bulk.InputValidation.value { callback_return_w3.f15 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[16], bench.bulk.InputValidation.value { callback_return_w3.f16 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[17], bench.bulk.InputValidation.value { callback_return_w3.f17 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[18], bench.bulk.InputValidation.value { callback_return_w3.f18 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[19], bench.bulk.InputValidation.value { callback_return_w3.f19 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[20], bench.bulk.InputValidation.value { callback_return_w3.f20 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[21], bench.bulk.InputValidation.value { callback_return_w3.f21 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[22], bench.bulk.InputValidation.value { callback_return_w3.f22 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[23], bench.bulk.InputValidation.value { callback_return_w3.f23 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[24], bench.bulk.InputValidation.value { callback_return_w3.f24 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[25], bench.bulk.InputValidation.value { callback_return_w3.f25 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[26], bench.bulk.InputValidation.value { callback_return_w3.f26 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[27], bench.bulk.InputValidation.value { callback_return_w3.f27 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[28], bench.bulk.InputValidation.value { callback_return_w3.f28 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[29], bench.bulk.InputValidation.value { callback_return_w3.f29 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[30], bench.bulk.InputValidation.value { callback_return_w3.f30 })
callback_return_w1_writer.long((0) + bench.bulk.BulkLayouts.r633c6cf82d3acfc1.offsets[31], bench.bulk.InputValidation.value { callback_return_w3.f31 })

callback_return_w1_writer } else null
 val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.array, callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_wide_row_return_handler_new_67f38d7168020371(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.bulk.WideRowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.bulk.WideRowReturnHandler? = null
        try {
            output0 = bench.bulk.WideRowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun rowReturnFire(handler: bench.bulk.RowReturnHandler, seed: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_row_return_fire_7c8a1183dcdd277c(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun wideRowReturnFire(handler: bench.bulk.WideRowReturnHandler, seed: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_wide_row_return_fire_88ad5bbf9c8c73e2(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun control(`value`: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_control_b0073ee32096caea(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun scalar(`value`: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.ra60d3960993b9a77.size)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0], bench.bulk.InputValidation.value { `value` })

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_scalar_a4a5da3f79ebbcaf(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun rowSum(`value`: bench.bulk.Row, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r789cd54eb117685f.size)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], bench.bulk.InputValidation.value { `value`.a })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], bench.bulk.InputValidation.value { `value`.b })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], bench.bulk.InputValidation.value { `value`.c })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], bench.bulk.InputValidation.value { `value`.d })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], bench.bulk.InputValidation.value { `value`.e })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], bench.bulk.InputValidation.value { `value`.f })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], bench.bulk.InputValidation.value { `value`.g })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], bench.bulk.InputValidation.value { `value`.h })

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_row_sum_db65517187cece4c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun numbersSum(values: kotlin.collections.List<Long>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r1800d3dd3eceec6d.size)
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.ra60d3960993b9a77.size) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0], bench.bulk.InputValidation.value { p0_w7 })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_numbers_sum_47543406a5cdf588(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowsSum(values: kotlin.collections.List<bench.bulk.Row>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.rf4d5524a08b9867b.size)
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r789cd54eb117685f)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], bench.bulk.InputValidation.value { p0_w7.a })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], bench.bulk.InputValidation.value { p0_w7.b })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], bench.bulk.InputValidation.value { p0_w7.c })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], bench.bulk.InputValidation.value { p0_w7.d })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], bench.bulk.InputValidation.value { p0_w7.e })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], bench.bulk.InputValidation.value { p0_w7.f })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], bench.bulk.InputValidation.value { p0_w7.g })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], bench.bulk.InputValidation.value { p0_w7.h })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_sum_d4a5ecfa43bf26b9(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun rowsMake(count: Long, seed: Long, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { count } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_make_6f7eb2a3ee93e0ae(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun rowsForReduce(count: Long, seed: Long, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { count } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_for_reduce_3c2e82abbc965f1c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun rowsEcho(values: kotlin.collections.List<bench.bulk.Row>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.rf4d5524a08b9867b.size)
val p0_w3_bulkList = bench.bulk.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r789cd54eb117685f)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], bench.bulk.InputValidation.value { p0_w7.a })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], bench.bulk.InputValidation.value { p0_w7.b })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], bench.bulk.InputValidation.value { p0_w7.c })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], bench.bulk.InputValidation.value { p0_w7.d })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], bench.bulk.InputValidation.value { p0_w7.e })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], bench.bulk.InputValidation.value { p0_w7.f })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], bench.bulk.InputValidation.value { p0_w7.g })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], bench.bulk.InputValidation.value { p0_w7.h })
}
}

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_echo_5c26c8cf4751c454(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun packetSum(packet: bench.bulk.Packet, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.re5733c882bf9c317.size)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[0], bench.bulk.InputValidation.value { packet.header.a })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[1], bench.bulk.InputValidation.value { packet.header.b })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[2], bench.bulk.InputValidation.value { packet.header.c })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[3], bench.bulk.InputValidation.value { packet.header.d })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[4], bench.bulk.InputValidation.value { packet.header.e })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[5], bench.bulk.InputValidation.value { packet.header.f })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[6], bench.bulk.InputValidation.value { packet.header.g })
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[7], bench.bulk.InputValidation.value { packet.header.h })
p0_w1_writer.bytes((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[8], bench.bulk.InputValidation.value { packet.bytes })

p0_w1_writer } else null } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_packet_sum_c07ede9eb5de050f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun handlerNew(callback: bench.bulk.RowsCallback, onError: bench.bulk.JniErrorHandler<bench.bulk.Handler> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.Handler = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Long {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: bench.bulk.BulkReader? = null
var callbackOutput0_bulkRange: bench.bulk.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<bench.bulk.Row>
        var callbackReturned: Long = 0L
        try {
callbackOutput0_bulkReader = bench.bulk.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


callbackOutput0_bulkList!!.add(bench.bulk.Row((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1
val __pb_consume: (bench.bulk.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_handler_new_916a49ec8343b596(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.bulk.Handler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.bulk.Handler? = null
        try {
            output0 = bench.bulk.Handler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun fire(handler: bench.bulk.Handler, count: Long, seed: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { handler } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { handler.raw } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { count } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { seed } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_fire_6fdab6ebdaeffb11(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
