package io.prebindgen.covertest

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface ErrorHandler_i64_mapped_3b407141677fe9e5<out R> { public fun run(v0: String): R }
