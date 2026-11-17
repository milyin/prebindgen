package io.prebindgen.covertest.outputcontracts

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface LocalErrorHandler_duration_result_c64b5b1058c5c0d0<out R> { public fun run(v0: java.time.Duration): R }
