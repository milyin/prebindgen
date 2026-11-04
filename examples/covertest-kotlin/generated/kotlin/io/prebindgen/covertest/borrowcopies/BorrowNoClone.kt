package io.prebindgen.covertest.borrowcopies

public class BorrowNoClone internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): BorrowNoClone = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = BorrowNoClone(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_BorrowNoClone_583d0749922ca782(raw) }
}
