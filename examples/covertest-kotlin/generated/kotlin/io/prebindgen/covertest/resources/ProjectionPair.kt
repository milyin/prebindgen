package io.prebindgen.covertest.resources

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class ProjectionPair(
    public val `first`: io.prebindgen.covertest.resources.ProjectionHandle,
    public val `second`: io.prebindgen.covertest.resources.ProjectionHandle
)
