@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.bulk

public fun `bulkPairTextCallback`(p0_callback: io.prebindgen.covertest.BulkPairTextCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPairText>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.size)
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w7.`borrowed_handle` }, false)
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[1], io.prebindgen.covertest.InputValidation.value { callback_return_w7.`owned_handle` }, true)
callback_return_w1_writer.text((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[2], io.prebindgen.covertest.InputValidation.value { callback_return_w7.`text` })
}
}

callback_return_w1_writer } else null
 io.prebindgen.covertest.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.bulk.BulkPairText)?.let { returnedValue2 -> (returnedValue2.`borrowed_handle` as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5);  }
(returnedValue2.`owned_handle` as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_pair_text_callback_387f8bdd6e114920(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `bulkHandleTexts`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7.`handle` }, true)
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w7.`text` })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_texts_695fb56c918fa85b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandleText>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandleText>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try {             try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
            outputbulkItem_g0_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long)

outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkHandleText(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1])) as String))
}
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkCollectionLockGroups`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?, p1_w1: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>, p2_callback: io.prebindgen.covertest.CollectionLocksProbe?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val p0_w3_elements = try { if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.let { p0_w3_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w3_snapshot.size) { p0_w3_index -> val p0_w7 = p0_w3_snapshot[p0_w3_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) }, intArrayOf(2)) } else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_elements = try { if (true) p1_w1?.toList()?.let { p1_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p1_w1_snapshot.size) { p1_w1_index -> val p1_w5 = p1_w1_snapshot[p1_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p1_w5 }) }, intArrayOf(2)) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { ((if (p0_w3 != null) (p0_w3_elements?.handles.orEmpty()) else kotlin.collections.emptyList()) + (p1_w1_elements?.handles.orEmpty())) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.encode()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p2_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Boolean = false
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p2_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_collection_lock_groups_e6dfa01d0633d98a(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (p0_w3 != null) { p0_w3_elements?.consume(__pb_consume) }; p1_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (p0_w3 != null) { p0_w3_elements?.consume(__pb_consume) }; p1_w1_elements?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `bulkHandleNested`(p0_w3: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r42eab86179b36709)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
val p0_w8_elements = if (true) p0_w8?.toList() else null
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w12 ->
p0_w1_writer.handle((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w12 }, true)
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_nested_1899ca04cd0105a2(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkItem_bulkActions.orEmpty()) { try { outputbulkFailure = action(outputbulkFailure) } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) } }
for (outputbulkItem_bulkIndex in (outputbulkItem_bulkActions?.size ?: 0) until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try { try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkItem_bulkActions.orEmpty()) { try { outputbulkFailure = action(outputbulkFailure) } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) } }
for (outputbulkItem_bulkIndex in (outputbulkItem_bulkActions?.size ?: 0) until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
outputbulkItem_bulkActions = kotlin.collections.mutableListOf()
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

outputbulkItem_bulkActions!!.add { initial -> var outputbulkItem_bulkFailure = initial; try {             try { val owner = outputbulkItem_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }; outputbulkItem_bulkFailure }
            outputbulkItem_bulkItem_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)

outputbulkItem_bulkList!!.add(requireNotNull(outputbulkItem_bulkItem_0))
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkItem_bulkActions.orEmpty()) { try { outputbulkFailure = action(outputbulkFailure) } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) } }
for (outputbulkItem_bulkIndex in (outputbulkItem_bulkActions?.size ?: 0) until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkHandleTextCallback`(p0_callback: io.prebindgen.covertest.BulkHandleTextCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.size)
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w7.`handle` }, true)
callback_return_w1_writer.text((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1], io.prebindgen.covertest.InputValidation.value { callback_return_w7.`text` })
}
}

callback_return_w1_writer } else null
 io.prebindgen.covertest.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.bulk.BulkHandleText)?.let { returnedValue2 -> (returnedValue2.`handle` as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_text_callback_97d45feee32cd50a(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `bulkOptionalBorrowPair`(p0_w3: Boolean, p0_w6: io.prebindgen.covertest.bulk.BulkHandle?, p0_w8: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.size)
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.offsets[0], io.prebindgen.covertest.InputValidation.value { if (p0_w3) requireNotNull(if (true) p0_w6 else null) { "invalid option payload" } else null }, false)
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (p0_w3) p0_w8 else 0L })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_borrow_pair_5145594a0d96696c(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `bulkSelect`(p0_w3: Int, p0_w6: Long, p0_w8: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3 })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w6 })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[2], io.prebindgen.covertest.InputValidation.value { p0_w8 })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_select_b3dae360ea0eecfb(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun `bulkKinds`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size) + io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.wireValue })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_kinds_9b033197b22af4fe(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkKind>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkKind.fromWire((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.offsets[0])) as Int))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkUnits`(p0_w3: kotlin.collections.List<kotlin.Unit>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.Unit>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.Unit> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, bulkUnit ->
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_units_1560a205c63d4c8a(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.Unit>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size


outputbulkList!!.add(kotlin.Unit)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkFlags`(p0_w3: kotlin.collections.List<Boolean>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Boolean>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Boolean> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.byte((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size) + io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.offsets[0], (if (io.prebindgen.covertest.InputValidation.value { p0_w8 }) 1 else 0).toByte())
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_flags_fe0ac76234b14846(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Boolean>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size


outputbulkList!!.add(((requireNotNull(outputbulkReader).byte((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.offsets[0]).toInt() != 0)) as Boolean)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkEnvelope`(p0_w3: io.prebindgen.covertest.bulk.BulkEnvelope?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkEnvelope?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkEnvelope? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Double4_ErrorEmpty_4244d173b136f580.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) if (true) if (true) requireNotNull(requireNotNull(requireNotNull(p0_w3)).`point`).`x` else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w3 != null) if (true) if (true) requireNotNull(requireNotNull(requireNotNull(p0_w3)).`point`).`y` else 0.0 else 0.0 else 0.0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)).`tag` else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_envelope_c78cdfdfd18c33f1(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 == 1) {
            }
            if (returnCapture.s0 == 1) io.prebindgen.covertest.bulk.BulkEnvelope(io.prebindgen.covertest.bulk.BulkPoint(returnCapture.s1, returnCapture.s2), returnCapture.s3) else null
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkPrimitives`(p0_w3: io.prebindgen.covertest.bulk.BulkPrimitives, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPrimitives> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPrimitives = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[0], (if (io.prebindgen.covertest.InputValidation.value { p0_w3.`flag` }) 1 else 0).toByte())
p0_w1_writer.byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w3.`byte` })
p0_w1_writer.short((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[2], io.prebindgen.covertest.InputValidation.value { p0_w3.`short` })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[3], io.prebindgen.covertest.InputValidation.value { p0_w3.`octet` })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[4], io.prebindgen.covertest.InputValidation.value { p0_w3.`word` })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[5], io.prebindgen.covertest.InputValidation.value { p0_w3.`number` })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[6], io.prebindgen.covertest.InputValidation.value { p0_w3.`scalar` })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[7], io.prebindgen.covertest.InputValidation.value { p0_w3.`address`.toLong() })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_primitives_be9bffc80840ca87(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

io.prebindgen.covertest.bulk.BulkPrimitives(((requireNotNull(outputbulkReader).byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[0]).toInt() != 0)) as Boolean, (requireNotNull(outputbulkReader).byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[1])) as Byte, (requireNotNull(outputbulkReader).short((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[2])) as Short, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[3])) as Int, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[4])) as Int, (requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[5])) as Long, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[6])) as Int, ((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[7])) as Long).toULong())
    } finally { returnCapture.release() }
}

public fun `bulkOptionalWide`(p0_w5: java.math.BigInteger?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger? = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w5 != null) 1 else 0) })
p0_w1_writer.wide((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[1], io.prebindgen.covertest.InputValidation.value { if (p0_w5 != null) if (true) requireNotNull(requireNotNull(p0_w5)) { "null 128-bit integer" }.toByteArray() else null else null }, true)

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_wide_0dbde2915189c95e(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) {
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) java.math.BigInteger((requireNotNull(outputbulkReader).wide((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[1])) as ByteArray) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkOptionalCallback`(p0_callback: io.prebindgen.covertest.BulkOptionalCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w6 = callbackReturned
 try { val callback_return_w6_elements = if (callback_return_w6 != null) (if (true) requireNotNull(callback_return_w6).toList() else null) else (null)
val callback_return_w4_writer_state = if (callback_return_w6 != null) (if (true) run { val callback_return_w4_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
val callback_return_w6_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w6_elements }
if (callback_return_w6_bulkList == null) { callback_return_w4_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val callback_return_w6_bulkStart = callback_return_w4_writer.rows(callback_return_w6_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
callback_return_w4_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], callback_return_w6_bulkStart, callback_return_w6_bulkList.size)
callback_return_w6_bulkList.forEachIndexed { callback_return_w6_bulkIndex, callback_return_w10 ->
callback_return_w4_writer.handle((callback_return_w6_bulkStart + callback_return_w6_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w10 }, true)
}
}

callback_return_w4_writer } else null) else (null)
 io.prebindgen.covertest.withHandleLocks((if (callback_return_w6 != null) (callback_return_w4_writer_state?.handles.orEmpty()) else kotlin.collections.emptyList())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>((if (callback_return_w6 != null) 1 else 0), if (callback_return_w6 != null) callback_return_w4_writer_state?.encode() else null, if (callback_return_w6 != null) callback_return_w4_writer_state?.length ?: 0 else 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
if (callback_return_w6 != null) { callback_return_w4_writer_state?.consume(__pb_consume) }
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w6 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_callback_0b45be4b854f6c11(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `bulkNative`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.`prefix` })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.`number` })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[2], io.prebindgen.covertest.InputValidation.value { p0_w8.`unsigned`.toLong() })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_native_939b6cd52b96a019(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkNative>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkNative((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[0])) as Int, (requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[1])) as Long, ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[2])) as Long).toULong()))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> `bulkExpanded`(p0_w4: Long, p0_w6: String?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, String) -> R): R = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w4 })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w6 })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_expanded_65b04628441e244b(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

build((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[1])) as String)
    } finally { returnCapture.release() }
}

public fun `bulkExpandedRows`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.`number` })
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.`text` })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_expanded_rows_8611c0e3cec0b75a(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkExpanded>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkExpanded((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[1])) as String))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> `bulkResult`(p0_w4: Long, p0_w6: String?, p1_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.BulkFailureHandler<R>, build: (Long, String) -> R): R = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorByteArray1_056b654d5078f47e.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w4 })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w6 })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_result_22b6c65613a56b35(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run { var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(returnCapture.d0 as ByteArray)

onError.run((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r97145e0ef6a1193d.offsets[0])) as Long) }
    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

build((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[1])) as String)
    } finally { returnCapture.release() }
}

public fun `bulkHandleNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_new_e5a9aa77fa861745(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
            output0 = io.prebindgen.covertest.bulk.BulkHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkHandleValue`(p0_w1: io.prebindgen.covertest.bulk.BulkHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_value_020d7d074b3973cd(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `bulkDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_drops_0c93c5a0b1492a12(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `bulkLate`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_late_051f1ea6dcd2506f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            output0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `bulkHandleViews`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.bulk.BulkHandle, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_views_c1f9da5de20e3fc3(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var outputg0_0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var outputg0_0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            outputg0_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long)
            build(requireNotNull(outputg0_0), (requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[1])) as Long)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkHandles`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 }, true)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handles_0a30179490c49dbc(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try {             try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
            outputbulkItem_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkBorrow`(p0_w3: io.prebindgen.covertest.bulk.BulkHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.rd515d9783303b819.size)
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.rd515d9783303b819.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3 }, false)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_borrow_588a078d0466d481(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `bulkChecked`(p0_w3: io.prebindgen.covertest.bulk.BulkHandle?, p1_w3: Int, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3 }, true)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r062153e85348d725.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { run {
p1_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r062153e85348d725.offsets[0], io.prebindgen.covertest.InputValidation.value { p1_w3 })

p1_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_checked_01f9de13d8c4a8b8(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            output0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkWide`(p0_w3: kotlin.collections.List<java.math.BigInteger?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.wide((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size) + io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.offsets[0], io.prebindgen.covertest.InputValidation.value { if (true) requireNotNull(p0_w8) { "null 128-bit integer" }.toByteArray() else null }, true)
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_wide_6b955d7b4ee7dc0d(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<java.math.BigInteger>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size


outputbulkList!!.add(java.math.BigInteger((requireNotNull(outputbulkReader).wide((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.offsets[0])) as ByteArray))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkUnsignedWide`(p0_w3: kotlin.collections.List<java.math.BigInteger?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r98c254983b527011)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.wide((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size) + io.prebindgen.covertest.BulkLayouts.r98c254983b527011.offsets[0], io.prebindgen.covertest.InputValidation.value { if (true) requireNotNull(p0_w8) { "null 128-bit integer" }.toByteArray() else null }, false)
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_unsigned_wide_407b0f069bb7ed7a(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<java.math.BigInteger>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size


outputbulkList!!.add(java.math.BigInteger(1, (requireNotNull(outputbulkReader).wide((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r98c254983b527011.offsets[0])) as ByteArray))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkCallbackWorker`(p0_callback: io.prebindgen.covertest.BulkWorkerCallback?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<Long>
        lateinit var callbackReturned: kotlin.collections.List<Long>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput0_bulkList!!.add((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
 val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(run {
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.long((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 })
}
}

callback_return_w2_writer.array }, callback_return_w2_writer.length)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_callback_worker_4498defd15d5c1bf(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkCallback`(p0_callback: io.prebindgen.covertest.BulkNumbersCallback?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<Long>
        lateinit var callbackReturned: kotlin.collections.List<Long>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput0_bulkList!!.add((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
 val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(run {
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.long((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 })
}
}

callback_return_w2_writer.array }, callback_return_w2_writer.length)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_callback_63ae4882756ac2cf(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkHandleCallback`(p0_callback: io.prebindgen.covertest.BulkHandlesCallback?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var callbackOutput0_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_0))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = callbackOutput0_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 try { val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer_state = if (true) run { val callback_return_w2_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.handle((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 }, true)
}
}

callback_return_w2_writer } else null
 io.prebindgen.covertest.withHandleLocks((callback_return_w2_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w2_writer_state?.encode(), callback_return_w2_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w2_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w4 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_callback_34c4be17902be9ab(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try {             try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
            outputbulkItem_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkPoint`(p0_w3: io.prebindgen.covertest.bulk.BulkPoint, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPoint> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPoint = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3.`x` })
p0_w1_writer.double((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w3.`y` })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_e770ca9da9c12852(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double)
    } finally { returnCapture.release() }
}

public fun `bulkPoints`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.`x` })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.`y` })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_points_6101af74290565c5(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkPointArray`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.`x` })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.`y` })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_array_aa3dcf2590ca3a28(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkSlice`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.`x` })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.`y` })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_slice_c30fbd330fdb06e6(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkPointPlain`(p0_w1: io.prebindgen.covertest.bulk.BulkPoint, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPoint> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPoint = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1.`x` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1.`y` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_plain_53e7bd65e4464218(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.bulk.BulkPoint(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun `bulkOptionalHandles`(p0_w5: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>? = run {
    val p0_w5_elements = try { if (p0_w5 != null) (if (true) requireNotNull(p0_w5).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkHandleWriter(io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w5 != null) 1 else 0) })
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w5 != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w12 ->
p0_w1_writer.handle((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (p0_w12 != null) requireNotNull(if (true) requireNotNull(p0_w12) else null) { "invalid option payload" } else null }, true)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_handles_f105b0e272c7fda2(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle?>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle?>? = null
var outputbulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
outputbulkActions = kotlin.collections.mutableListOf()
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

outputbulkActions!!.add { initial -> var outputbulkFailure = initial; try {             if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }; outputbulkFailure }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            outputbulkItem_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long)
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) requireNotNull(outputbulkItem_0) else null)
}
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) requireNotNull(outputbulkList) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
for (action in outputbulkActions.orEmpty()) { try { action(failure) } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) } }
for (outputbulkIndex in (outputbulkActions?.size ?: 0) until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            try { val owner = outputbulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long) } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkArray`(p0_w3: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_array_b64f34d955cfbc66(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkArrayBorrow`(p0_w3: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_array_borrow_08a33cfde93f0897(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `bulkScalar`(p0_w3: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3 })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_scalar_859e15b07fe6305f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun `bulkNumbers`(p0_w3: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_numbers_a3f5c3da005fa8ec(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkNested`(p0_w3: kotlin.collections.List<kotlin.collections.List<Long>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<Long>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<Long>> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
val p0_w8_elements = if (true) p0_w8?.toList() else null
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w13 ->
p0_w1_writer.long((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w13 })
}
}
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_nested_0e44e904f1fd8209(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<Long>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkItem_bulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkText`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3 })

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_text_af334975f2297859(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0])) as String
    } finally { returnCapture.release() }
}

public fun `bulkStrings`(p0_w3: kotlin.collections.List<String?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_strings_106737903dcecf60(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size


outputbulkList!!.add((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0])) as String)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `bulkOptional`(p0_w5: kotlin.collections.List<String?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String?>? = run {
    val p0_w5_elements = try { if (p0_w5 != null) (if (true) requireNotNull(p0_w5).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w5 != null) 1 else 0) })
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w5 != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w12 ->
p0_w1_writer.int((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w12 != null) 1 else 0) })
p0_w1_writer.text((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[1], io.prebindgen.covertest.InputValidation.value { if (p0_w12 != null) requireNotNull(p0_w12) else null })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_4abfbe9a71e24339(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) {
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) {
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[1])) as String else null)
}
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) requireNotNull(outputbulkList) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `bulkBytes`(p0_w3: kotlin.collections.List<ByteArray?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ByteArray>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ByteArray> = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.re048ef6766326a01)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size) + io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_bytes_711128b36c976142(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<ByteArray>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size


outputbulkList!!.add((requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.offsets[0])) as ByteArray)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}
