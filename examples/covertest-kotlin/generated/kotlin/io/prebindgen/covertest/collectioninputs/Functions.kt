@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.collectioninputs

public fun `collectionInputPlain`(p0_w1: kotlin.collections.List<Long>, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) p0_w1?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_plain_49edb34e988dff39(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputBulk`(p0_w3: kotlin.collections.List<Long>, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_bulk_7564a92b14e3a524(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputCustom`(p0_w1: kotlin.collections.List<Long>, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) p0_w1?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w6 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w6 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_custom_3f5932b5fe2bdeb0(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputCustomBulk`(p0_w3: kotlin.collections.List<Long>, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r560cf018ca83c8a9.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r560cf018ca83c8a9.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r97145e0ef6a1193d)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r560cf018ca83c8a9.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97145e0ef6a1193d.size) + io.prebindgen.covertest.BulkLayouts.r97145e0ef6a1193d.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_custom_bulk_80b5719b93fcace3(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputNarrow`(p0_w1: kotlin.collections.List<Int>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) p0_w1?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w6 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w6 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_narrow_a2e0942031835410(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputNarrowBulk`(p0_w3: kotlin.collections.List<Int>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) p0_w3?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rca7ae8840b1d5037.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rca7ae8840b1d5037.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r0724546ec24d95a1)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rca7ae8840b1d5037.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r0724546ec24d95a1.size) + io.prebindgen.covertest.BulkLayouts.r0724546ec24d95a1.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_narrow_bulk_01859c8a0312491b(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputConstructions`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_constructions_07412dbe4e08ef68(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `collectionInputSlice`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) p0_w1?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_slice_009ff8f8b9e77cd5(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputArray`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) p0_w1?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_array_517465a977f1d8a5(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputOptionalArray`(p0_w3: kotlin.collections.List<Long>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.let { p0_w3_snapshot -> kotlin.Array<kotlin.Any?>(p0_w3_snapshot.size) { p0_w3_index -> val p0_w7 = p0_w3_snapshot[p0_w3_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) } } else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_optional_array_c11bc48fb5ea6c63(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputOptionalArrayBulk`(p0_w5: kotlin.collections.List<Long>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w5_elements = try { if (p0_w5 != null) (if (true) requireNotNull(p0_w5).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w5 != null) 1 else 0) })
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w5 != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w9 ->
p0_w1_writer.long((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_optional_array_bulk_c810a58a5f8b13ca(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionInputOptional`(p0_w5: kotlin.collections.List<Long>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w5_elements = try { if (p0_w5 != null) (if (true) requireNotNull(p0_w5).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer = try { io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.size) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w5 != null) 1 else 0) })
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w5 != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w9 ->
p0_w1_writer.long((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 })
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_input_optional_9f143cf5440a4074(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
