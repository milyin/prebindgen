package io.prebindgen.covertest.resources

public class ProjectionHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 130266342272206208L
    public fun take(): ProjectionHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = ProjectionHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(raw) }
}
