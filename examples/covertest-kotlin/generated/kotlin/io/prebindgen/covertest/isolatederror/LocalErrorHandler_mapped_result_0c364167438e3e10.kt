package io.prebindgen.covertest.isolatederror

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface LocalErrorHandler_mapped_result_0c364167438e3e10<out R> { public fun run(v0: String): R }
