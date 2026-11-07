package io.prebindgen.covertest.returns

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface ReturnWideErrorHandler<out R> { public fun run(v0: io.prebindgen.covertest.returns.ReturnWideError): R }
