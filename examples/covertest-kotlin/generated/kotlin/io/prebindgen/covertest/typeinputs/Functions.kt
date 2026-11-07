@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.typeinputs

public fun typeInputANew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.typeinputs.TypeInputA> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.typeinputs.TypeInputA = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_a_new_a666cfeb0749bacc(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.typeinputs.TypeInputA? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputA_ac61bf30bfd3c448(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.typeinputs.TypeInputA? = null
        try {
            output0 = io.prebindgen.covertest.typeinputs.TypeInputA(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputA_ac61bf30bfd3c448(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun typeInputARead(`value`: io.prebindgen.covertest.typeinputs.TypeInputA, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_a_read_752e87ac619cd2ab(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputBNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.typeinputs.TypeInputB> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.typeinputs.TypeInputB = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_b_new_0ac223fcf1a09c24(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.typeinputs.TypeInputB? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputB_ac61c230bfd3c961(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.typeinputs.TypeInputB? = null
        try {
            output0 = io.prebindgen.covertest.typeinputs.TypeInputB(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputB_ac61c230bfd3c961(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun typeInputBRead(`value`: io.prebindgen.covertest.typeinputs.TypeInputB, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_b_read_2a0fffe1bd334d9d(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputControlNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.typeinputs.TypeInputControl> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.typeinputs.TypeInputControl = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_control_new_c01186712acd5471(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.typeinputs.TypeInputControl? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputControl_7c64d7b5440ff06a(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.typeinputs.TypeInputControl? = null
        try {
            output0 = io.prebindgen.covertest.typeinputs.TypeInputControl(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_TypeInputControl_7c64d7b5440ff06a(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun typeInputLive(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_live_03e8c255db4b9ee9(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputConversions(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_conversions_1b6cd416be082059(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputFixed(a: io.prebindgen.covertest.typeinputs.TypeInputA, b: io.prebindgen.covertest.typeinputs.TypeInputB, c: io.prebindgen.covertest.typeinputs.TypeInputA, d: io.prebindgen.covertest.typeinputs.TypeInputB, probe: io.prebindgen.covertest.VoidToboolCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { a } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { b } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: io.prebindgen.covertest.NativeHandle? = try { c } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle3: io.prebindgen.covertest.NativeHandle? = try { d } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles(kotlin.collections.listOfNotNull(__pb_handle1, __pb_handle3), false); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles(kotlin.collections.listOfNotNull(__pb_handle0, __pb_handle2), false); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_handle2?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_handle3?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Boolean = false
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = probe.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_fixed_157e836568a582a4(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputFixedMut(a: io.prebindgen.covertest.typeinputs.TypeInputA, b: io.prebindgen.covertest.typeinputs.TypeInputA, other: io.prebindgen.covertest.typeinputs.TypeInputB, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { a } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { b } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: io.prebindgen.covertest.NativeHandle? = try { other } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1, __pb_handle2) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_handle2?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_fixed_mut_1469d98e8752a26b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputSharedLists(a: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>, b: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputB>, control: io.prebindgen.covertest.typeinputs.TypeInputControl, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) a?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_691042cfe26de2b8()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r691042cfe26de2b8.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r691042cfe26de2b8.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a.size) + io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 }, false, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w3_elements = try { if (true) b?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_writer_state = try { if (true) run { val p1_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_c0e77b118c420293()
val p1_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p1_w3_elements }
if (p1_w3_bulkList == null) { p1_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc0e77b118c420293.offsets[0]) } else {
val p1_w3_bulkStart = p1_w1_writer.rows(p1_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r79d341a04044600f)
p1_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc0e77b118c420293.offsets[0], p1_w3_bulkStart, p1_w3_bulkList.size)
p1_w3_bulkList.forEachIndexed { p1_w3_bulkIndex, p1_w9 ->
p1_w1_writer.handle((p1_w3_bulkStart + p1_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r79d341a04044600f.size) + io.prebindgen.covertest.BulkLayouts.r79d341a04044600f.offsets[0], io.prebindgen.covertest.InputValidation.value { p1_w9 }, false, 0)
}
}

p1_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p1_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: io.prebindgen.covertest.NativeHandle? = try { control } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles(kotlin.collections.listOfNotNull(__pb_handle2), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), false); val __pb_ordered2 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), false); io.prebindgen.covertest.withGcOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withGcOrderedHandleLocks(__pb_ordered1) { io.prebindgen.covertest.withGcOrderedHandleLocks(__pb_ordered2) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_handle2?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_shared_lists_82c951ffa3f2a084(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputMutLists(a: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>, b: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputB>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) a?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_c5f2417c9e799f21()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc5f2417c9e799f21.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r08d07586d1573849)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc5f2417c9e799f21.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r08d07586d1573849.size) + io.prebindgen.covertest.BulkLayouts.r08d07586d1573849.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 }, false, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w3_elements = try { if (true) b?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_writer_state = try { if (true) run { val p1_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_a134f9aa0d8c76d6()
val p1_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p1_w3_elements }
if (p1_w3_bulkList == null) { p1_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra134f9aa0d8c76d6.offsets[0]) } else {
val p1_w3_bulkStart = p1_w1_writer.rows(p1_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r8c73c9ae300a086c)
p1_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra134f9aa0d8c76d6.offsets[0], p1_w3_bulkStart, p1_w3_bulkList.size)
p1_w3_bulkList.forEachIndexed { p1_w3_bulkIndex, p1_w9 ->
p1_w1_writer.handle((p1_w3_bulkStart + p1_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8c73c9ae300a086c.size) + io.prebindgen.covertest.BulkLayouts.r8c73c9ae300a086c.offsets[0], io.prebindgen.covertest.InputValidation.value { p1_w9 }, false, 0)
}
}

p1_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p1_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_mut_lists_30242e7654d4250f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputMixed(reads: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>, writes: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) reads?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_691042cfe26de2b8()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r691042cfe26de2b8.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r691042cfe26de2b8.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a.size) + io.prebindgen.covertest.BulkLayouts.rc2cf5ebe6163507a.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 }, false, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w3_elements = try { if (true) writes?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_writer_state = try { if (true) run { val p1_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_c5f2417c9e799f21()
val p1_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p1_w3_elements }
if (p1_w3_bulkList == null) { p1_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc5f2417c9e799f21.offsets[0]) } else {
val p1_w3_bulkStart = p1_w1_writer.rows(p1_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r08d07586d1573849)
p1_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc5f2417c9e799f21.offsets[0], p1_w3_bulkStart, p1_w3_bulkList.size)
p1_w3_bulkList.forEachIndexed { p1_w3_bulkIndex, p1_w9 ->
p1_w1_writer.handle((p1_w3_bulkStart + p1_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r08d07586d1573849.size) + io.prebindgen.covertest.BulkLayouts.r08d07586d1573849.offsets[0], io.prebindgen.covertest.InputValidation.value { p1_w9 }, false, 0)
}
}

p1_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p1_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks((__pb_handle0), (__pb_handle1)) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_mixed_7b268c9e6948a110(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputRow(`value`: io.prebindgen.covertest.typeinputs.TypeInputRow, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val __pb_p0_w1_field0 = try { `value`.a } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field1 = try { `value`.b } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_p0_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { __pb_p0_w1_field1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_row_d1ca0f73c8d133b6(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputHolder(`value`: io.prebindgen.covertest.typeinputs.TypeInputHolder, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val __pb_p0_w1_field0 = try { `value`.row } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_f0_some_field0 = try { if (__pb_p0_w1_field0 != null) (if (true) requireNotNull(__pb_p0_w1_field0).a else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_f0_some_field1 = try { if (__pb_p0_w1_field0 != null) (if (true) requireNotNull(__pb_p0_w1_field0).b else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (__pb_p0_w1_field0 != null) (if (true) requireNotNull(__pb_p0_w1_f0_some_field0) else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { if (__pb_p0_w1_field0 != null) (if (true) requireNotNull(__pb_p0_w1_f0_some_field1) else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { (if (__pb_p0_w1_field0 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_holder_4673e9a6dda65297(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputRows(values: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputRow>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_62ead3aacdc8be2d()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.a
val __pb_p0_w7_field1 = p0_w7.b
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field1 }, true, 1)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_rows_1943c070e2ef84cb(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputRowsPlain(values: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputRow>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
val __pb_p0_w5_field0 = p0_w5.a
val __pb_p0_w5_field1 = p0_w5.b
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_p0_w5_field0 }, io.prebindgen.covertest.InputValidation.value { __pb_p0_w5_field1 }) }, intArrayOf(2, 2), 2, arrayOf(intArrayOf(0), intArrayOf(1))) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_elements?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_elements?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_rows_plain_8d430d0aa0f4b840(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_elements?.consume(__pb_consume) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputNested(values: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputRow>>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_add1c00744245e75()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.radd1c00744245e75.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.radd1c00744245e75.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_elements = if (true) p0_w7?.toList() else null
val p0_w7_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w7_elements }
if (p0_w7_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.size) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0]) } else {
val p0_w7_bulkStart = p0_w1_writer.rows(p0_w7_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.size) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0], p0_w7_bulkStart, p0_w7_bulkList.size)
p0_w7_bulkList.forEachIndexed { p0_w7_bulkIndex, p0_w11 ->
val __pb_p0_w11_field0 = p0_w11.a
val __pb_p0_w11_field1 = p0_w11.b
p0_w1_writer.handle((p0_w7_bulkStart + p0_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w11_field0 }, true, 0)
p0_w1_writer.handle((p0_w7_bulkStart + p0_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_p0_w11_field1 }, true, 1)
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_nested_d53c1e8c8b8963c9(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputCallback(callback: io.prebindgen.covertest.TypeInputCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputRow>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_62ead3aacdc8be2d()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val __pb_callback_return_w7_field0 = callback_return_w7.a
val __pb_callback_return_w7_field1 = callback_return_w7.b
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field0 }, true, 0)
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field1 }, true, 1)
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.typeinputs.TypeInputRow)?.let { returnedValue2 -> (returnedValue2.a as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
(returnedValue2.b as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6); returnedOwners.add(returnedValue6) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_callback_e567d8ed47f4c01f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputNestedCallback(callback: io.prebindgen.covertest.TypeInputNestedCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputRow>>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_add1c00744245e75()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.radd1c00744245e75.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.radd1c00744245e75.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val callback_return_w7_elements = if (true) callback_return_w7?.toList() else null
val callback_return_w7_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w7_elements }
if (callback_return_w7_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.size) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0]) } else {
val callback_return_w7_bulkStart = callback_return_w1_writer.rows(callback_return_w7_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.size) + io.prebindgen.covertest.BulkLayouts.r62ead3aacdc8be2d.offsets[0], callback_return_w7_bulkStart, callback_return_w7_bulkList.size)
callback_return_w7_bulkList.forEachIndexed { callback_return_w7_bulkIndex, callback_return_w11 ->
val __pb_callback_return_w11_field0 = callback_return_w11.a
val __pb_callback_return_w11_field1 = callback_return_w11.b
callback_return_w1_writer.handle((callback_return_w7_bulkStart + callback_return_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w11_field0 }, true, 0)
callback_return_w1_writer.handle((callback_return_w7_bulkStart + callback_return_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.size) + io.prebindgen.covertest.BulkLayouts.rf9fa355fe087a0c1.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w11_field1 }, true, 1)
}
}
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_ordered0 = io.prebindgen.covertest.prepareInputHandles((__pb_handle1), true); val __pb_ordered1 = io.prebindgen.covertest.prepareInputHandles((__pb_handle0), true); io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered0) { io.prebindgen.covertest.withOrderedHandleLocks(__pb_ordered1) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? io.prebindgen.covertest.typeinputs.TypeInputRow)?.let { returnedValue3 -> (returnedValue3.a as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
(returnedValue3.b as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_nested_callback_91c187ec446a933d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputOptional(values: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w5_elements = try { if (values != null) (if (true) requireNotNull(values).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i32_Range_ab3ae4944fbfb9e9()
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rab3ae4944fbfb9e9.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (values != null) 1 else 0) })
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (values != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rab3ae4944fbfb9e9.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rab3ae4944fbfb9e9.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978.size) + io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_optional_6826ec5cb3fb4351(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun typeInputArray(values: kotlin.collections.List<io.prebindgen.covertest.typeinputs.TypeInputA>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_cedc43bd1c5d31f6()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rcedc43bd1c5d31f6.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rcedc43bd1c5d31f6.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978.size) + io.prebindgen.covertest.BulkLayouts.rf3410c1e0e62c978.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_type_input_array_8fb1d0d4f19b9470(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
