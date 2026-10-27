@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.callbackreturns

public fun `callbackReturnI64`(p0_callback: io.prebindgen.covertest.i64Toi64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call15(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnI8`(p0_callback: io.prebindgen.covertest.i8Toi8Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Byte> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Byte = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Byte): Byte {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Byte = 0.toByte()
        var callbackReturned: Byte = 0.toByte()
        try {
            callbackArgument0_0 = raw0 as Byte
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call16(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Byte }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnI16`(p0_callback: io.prebindgen.covertest.i16Toi16Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Short> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Short = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Short): Short {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Short = 0.toShort()
        var callbackReturned: Short = 0.toShort()
        try {
            callbackArgument0_0 = raw0 as Short
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call17(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Short }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnF32`(p0_callback: io.prebindgen.covertest.f32Tof32Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Float> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Float = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Float): Float {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Float = 0.0f
        var callbackReturned: Float = 0.0f
        try {
            callbackArgument0_0 = raw0 as Float
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call18(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Float }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnU128`(p0_callback: io.prebindgen.covertest.u128Tou128Callback?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: java.math.BigInteger
        var callbackReturned: java.math.BigInteger? = null
        try {
            callbackArgument0_0 = java.math.BigInteger(1, raw0 as ByteArray)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1?.toByteArray()

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call19(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { java.math.BigInteger(1, result[1] as ByteArray) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnZero`(p0_callback: io.prebindgen.covertest.VoidToi64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Long = 0L
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call20(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnBool`(p0_callback: io.prebindgen.covertest.boolToboolCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Boolean
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call21(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Boolean }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnChar`(p0_callback: io.prebindgen.covertest.charTocharCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int): Int {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Int = 0
        var callbackReturned: Int = 0
        try {
            callbackArgument0_0 = raw0 as Int
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call22(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnFloat`(p0_callback: io.prebindgen.covertest.f64Tof64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Double> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Double = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Double): Double {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Double = 0.0
        var callbackReturned: Double = 0.0
        try {
            callbackArgument0_0 = raw0 as Double
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call23(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Double }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnUnsigned`(p0_callback: io.prebindgen.covertest.u64Tou64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: ULong = 0UL
        var callbackReturned: ULong = 0UL
        try {
            callbackArgument0_0 = (raw0 as Long).toULong()
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1.toLong()

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call24(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnWide`(p0_callback: io.prebindgen.covertest.i128Toi128Callback?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: java.math.BigInteger
        var callbackReturned: java.math.BigInteger? = null
        try {
            callbackArgument0_0 = java.math.BigInteger(raw0 as ByteArray)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1?.toByteArray()

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call25(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { java.math.BigInteger(result[1] as ByteArray) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnText`(p0_callback: io.prebindgen.covertest.StringToStringCallback?, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?): String? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: String
        var callbackReturned: String? = null
        try {
            callbackArgument0_0 = raw0 as String
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call26(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as String }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnBytes`(p0_callback: io.prebindgen.covertest.u8VecTou8VecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: ByteArray
        var callbackReturned: ByteArray? = null
        try {
            callbackArgument0_0 = raw0 as ByteArray
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call27(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as ByteArray }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnOptional`(p0_callback: io.prebindgen.covertest.Optionali64ToOptionali64Callback?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long? = null
        var callbackReturned: Long? = null
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            if (raw0 == 1) {
            }
            callbackArgument0_0 = if (raw0 == 1) raw1 as Long else null
        } catch (failure: Throwable) {
            if (raw0 == 1) {
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            if (raw0 == 1) {
            }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }
return kotlin.run { val callback_return_w3 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>((if (callback_return_w3 != null) 1 else 0), if (callback_return_w3 != null) requireNotNull(callback_return_w3) else 0L)

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call28(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) result[2] as Long else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnVector`(p0_callback: io.prebindgen.covertest.i64VecToi64VecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<Long>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<Long>
        lateinit var callbackReturned: kotlin.collections.List<Long>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {  } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }

callbackOutput0_list.add(callbackOutput0__row[0] as Long)
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>


} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>


} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.map { callback_return_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(callback_return_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
 val callbackReturnWire = callback_return_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call29(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnRecord`(p0_callback: io.prebindgen.covertest.callbackreturns.CallbackReturnRecordToCallbackReturnRecordCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: String?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnRecord
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnRecord
        try {
            callbackArgument0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnRecord(raw0 as Long, raw1 as String)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.`number`, callback_return_w1.`label`)

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call30(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnEnum`(p0_callback: io.prebindgen.covertest.callbackreturns.CallbackReturnChoiceToCallbackReturnChoiceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int): Int {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnChoice
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnChoice
        try {
            callbackArgument0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnChoice.fromWire(raw0 as Int)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1.wireValue

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call31(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Boolean }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnResourceNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbackreturns.CallbackReturnResource = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call32(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop12(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnResourceLive`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call33() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnOwned`(p0_callback: io.prebindgen.covertest.VoidToCallbackReturnResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  io.prebindgen.covertest.withHandleLocks(kotlin.collections.listOfNotNull(if (true) requireNotNull(callback_return_w1) { "null handle" } else null)) { val callbackReturnWire = callback_return_w1?.raw ?: 0L
if (true) callback_return_w1!!.markConsumed()
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w1); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call34(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnTake`(p0_callback: io.prebindgen.covertest.callbackreturns.CallbackReturnResourceToCallbackReturnResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
        var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackReturnReady = false
        try { var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop12(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = p0_callback.run(callbackArgument0_0); callbackReturnReady = true
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop12(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        } } catch (failure: Throwable) { if (callbackReturnReady) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callbackReturned) }; throw failure }
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  io.prebindgen.covertest.withHandleLocks(kotlin.collections.listOfNotNull(if (true) requireNotNull(callback_return_w1) { "null handle" } else null)) { val callbackReturnWire = callback_return_w1?.raw ?: 0L
if (true) callback_return_w1!!.markConsumed()
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w1); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call35(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnOwnedOptional`(p0_callback: io.prebindgen.covertest.VoidToOptionalCallbackReturnResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try {  io.prebindgen.covertest.withHandleLocks(kotlin.collections.listOfNotNull(if (callback_return_w3 != null) if (true) requireNotNull(requireNotNull(callback_return_w3)) { "null handle" } else null else null)) { val callbackReturnWire = if (callback_return_w3 != null) requireNotNull(callback_return_w3).raw else 0L
if (callback_return_w3 != null) { if (true) requireNotNull(callback_return_w3).markConsumed() }
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w3); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call36(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) result[2] as Long else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnOwnedVector`(p0_callback: io.prebindgen.covertest.VoidToCallbackReturnResourceVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackReturnResource?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.map { callback_return_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(callback_return_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(callback_return_w5) { "null handle" } else null) }, { if (true) callback_return_w5!!.markConsumed(); kotlin.Unit })
} else null
 io.prebindgen.covertest.withHandleLocks((kotlin.collections.emptyList<Nothing>() + (callback_return_w1_elements.orEmpty().flatMap { it.second() }))) { val callbackReturnWire = callback_return_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()
callback_return_w1_elements.orEmpty().forEach { it.third() }
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w1); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call37(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnSpawn`(p0_callback: io.prebindgen.covertest.i64Toi64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnJob> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbackreturns.CallbackReturnJob = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call38(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnJob? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnJob(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop11(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnJoin`(p0_w1: io.prebindgen.covertest.callbackreturns.CallbackReturnJob?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call39(nativeArgument0, nativeArgument1) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnHandlerNew`(p0_callback: io.prebindgen.covertest.i64Toi64Callback?, p1_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnHandler> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnHandler>): io.prebindgen.covertest.callbackreturns.CallbackReturnHandler = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call40(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnHandler? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnHandler(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop10(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        true -> { 
onError.run(result[1] as String) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnHandlerCall`(p0_w1: io.prebindgen.covertest.callbackreturns.CallbackReturnHandler?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call41(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnCatch`(p0_callback: io.prebindgen.covertest.i64Toi64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call42(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnHandlerMany`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.i64Toi64Callback?>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(if (p0_w6 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_w6.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
}) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call43(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnCharCatch`(p0_callback: io.prebindgen.covertest.VoidTocharCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Int {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Int = 0
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call44(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnOwnedVectorCatch`(p0_callback: io.prebindgen.covertest.VoidToCallbackReturnResourceVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackReturnResource?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.map { callback_return_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(callback_return_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(callback_return_w5) { "null handle" } else null) }, { if (true) callback_return_w5!!.markConsumed(); kotlin.Unit })
} else null
 io.prebindgen.covertest.withHandleLocks((kotlin.collections.emptyList<Nothing>() + (callback_return_w1_elements.orEmpty().flatMap { it.second() }))) { val callbackReturnWire = callback_return_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()
callback_return_w1_elements.orEmpty().forEach { it.third() }
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w1); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call45(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnUnit`(p0_callback: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call46(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackReturnHandlerValue`(p0_w2: io.prebindgen.covertest.i64Toi64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_w2.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call47(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

