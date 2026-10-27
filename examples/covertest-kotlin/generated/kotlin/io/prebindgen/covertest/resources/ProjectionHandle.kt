package io.prebindgen.covertest.resources

public class ProjectionHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): ProjectionHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = ProjectionHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop22(raw) }
}
