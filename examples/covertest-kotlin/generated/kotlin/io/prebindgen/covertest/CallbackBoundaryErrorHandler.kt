package io.prebindgen.covertest

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface CallbackBoundaryErrorHandler<out R> { public fun run(v0: Long, v1: String): R }
