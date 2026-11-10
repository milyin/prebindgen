package io.prebindgen.covertest.outputcontracts

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface i64Handler<out R> { public fun run(v0: Long): R }
