package io.prebindgen.covertest.options

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class OptionalResource(
    public val `resource`: io.prebindgen.covertest.resources.OwnedResource?,
    public val `details`: io.prebindgen.covertest.options.OptionalFields?
)
