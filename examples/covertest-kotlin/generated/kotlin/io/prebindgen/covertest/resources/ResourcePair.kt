package io.prebindgen.covertest.resources

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class ResourcePair(
    public val `first`: io.prebindgen.covertest.resources.OwnedResource,
    public val `second`: io.prebindgen.covertest.resources.OwnedResource
)
