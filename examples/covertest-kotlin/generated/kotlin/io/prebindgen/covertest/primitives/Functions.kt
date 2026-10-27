@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.primitives

public fun `primitiveI8`(p0_w1: Byte, onError: io.prebindgen.covertest.JniErrorHandler<Byte> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Byte = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_i8_8b845afaa2c2b930(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Byte }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveI16`(p0_w1: Short, onError: io.prebindgen.covertest.JniErrorHandler<Short> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Short = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_i16_f014be4488893e1b(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Short }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveI32`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_i32_47001a8bec684907(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveI64`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_i64_80af91ea21ca8b59(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveI128`(p0_w1: java.math.BigInteger?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val result = try { run { val nativeArgument0 = try { p0_w1?.toByteArray() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_i128_7c7adcb71f399482(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { java.math.BigInteger(result[1] as ByteArray) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveIsize`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_isize_f73f4c9446afe299(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveU8`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_u8_524c6ffed8776520(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveU16`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_u16_f914d3a206cd401b(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveU32`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_u32_c7f6c43082a5b737(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveU64`(p0_w1: ULong, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_u64_180c1433590aabe9(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveU128`(p0_w1: java.math.BigInteger?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val result = try { run { val nativeArgument0 = try { p0_w1?.toByteArray() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_u128_ada5db066814cfea(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { java.math.BigInteger(1, result[1] as ByteArray) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveUsize`(p0_w1: ULong, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_usize_c1dde3a165f98019(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveF32`(p0_w1: Float, onError: io.prebindgen.covertest.JniErrorHandler<Float> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Float = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_f32_0c5b24fab8306069(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Float }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveF64`(p0_w1: Double, onError: io.prebindgen.covertest.JniErrorHandler<Double> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Double = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_f64_3ac16aca0f3ac727(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Double }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveBool`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_bool_044d0aea7d03630a(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Boolean }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveChar`(p0_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_primitive_char_60d88328057d650a(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `primitiveUnit`(onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_primitive_unit_d880ab199b1d4c5a() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

