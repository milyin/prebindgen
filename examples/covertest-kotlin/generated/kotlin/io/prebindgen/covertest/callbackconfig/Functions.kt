@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.callbackconfig

public fun `callbackConfigDefault`(p0_callback: io.prebindgen.covertest.callbackconfig.ConfiguredHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_default_7ba54fc6e91d1046(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigTypeDefault`(p0_callback: io.prebindgen.covertest.callbackconfig.TypeDefaultHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_type_default_2e21d1170d142a9b(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigArgument`(p0_callback: io.prebindgen.covertest.callbackconfig.FlagArgumentHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean, raw1: Long): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Boolean
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_argument_f2ca22fac606e26f(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigReturn`(p0_callback: io.prebindgen.covertest.callbackconfig.NumberReturnHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_return_1a2a025b2d23bf61(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigPair`(p0_callback: io.prebindgen.covertest.callbackconfig.ConfiguredHandler?, p1_callback: io.prebindgen.covertest.callbackconfig.FlagArgumentHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean, raw1: Long): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Boolean
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p1_callback.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_pair_075058877ec579cf(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigNested`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.callbackconfig.NestedHandler?>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(if (p0_w6 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean, raw1: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Boolean
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_w6.run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
}) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_nested_c023613beb216159(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigOptional`(p0_w4: io.prebindgen.covertest.callbackconfig.ConfiguredHandler?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w4 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w4 != null) if (requireNotNull(p0_w4) == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument1_0: Long = 0L
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = requireNotNull(p0_w4).run(callbackArgument0_0, callbackArgument1_0)
return kotlin.run { val callback_return_w2 = callbackReturned
  val callbackReturnWire = callback_return_w2

callbackReturnWire }
    }
} else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_optional_c1ccbb78d74cca03(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `callbackConfigOrdinary`(p0_w2: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_config_ordinary_7a1de11aa6329a97(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

