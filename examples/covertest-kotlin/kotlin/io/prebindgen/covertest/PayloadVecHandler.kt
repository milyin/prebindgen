package io.prebindgen.covertest

/**
 * Hand-written typed handle for the native `PayloadVecHandler`.
 *
 * This is the one type whose binding declares `.suppress_kotlin_code()`, so
 * JniGen emits **neither** this Kotlin class **nor** the Rust `freePtr`
 * destructor for it — the author owns both sides. It mirrors the generated
 * [PayloadHandler] shape exactly; the matching
 * `Java_io_prebindgen_covertest_PayloadVecHandler_freePtr` extern is hand-written
 * in `src/lib.rs`.
 *
 * Being a [NativeHandle] subclass in the base package lets the generated
 * `payloadVecHandlerNew` / `storageCallbackVec` functions construct it and take
 * its `ptr` under `withSortedHandleLocks`, exactly like every generated handle.
 */
public class PayloadVecHandler(initialPtr: Long) : NativeHandle(initialPtr) {
    @Synchronized
    override fun close() {
        val p = ptr
        if (p != 0L) {
            ptr = 0L
            freePtr(p)
        }
    }

    @Synchronized
    public fun take(): PayloadVecHandler {
        val p = ptr
        ptr = 0L
        return PayloadVecHandler(p)
    }

    public companion object {
        @JvmStatic
        external fun freePtr(ptr: Long)
    }
}
