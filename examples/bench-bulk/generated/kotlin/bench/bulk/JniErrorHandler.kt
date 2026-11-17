package bench.bulk

/** Binding failure callback, invoked after the native call and handle locks finish. */
public fun interface JniErrorHandler<out R> { public fun run(message: String?): R }
