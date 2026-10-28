package io.prebindgen.covertest

/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */
public fun interface CResourceErrorHandler<out R> { public fun run(v0: io.prebindgen.covertest.resources.OwnedResource, v1: String): R }
