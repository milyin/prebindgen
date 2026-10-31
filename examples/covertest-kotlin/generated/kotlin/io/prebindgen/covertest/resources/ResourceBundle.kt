package io.prebindgen.covertest.resources

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class ResourceBundle(
    public val `label`: kotlin.String,
    public val `pair`: io.prebindgen.covertest.resources.ResourcePair,
    public val `tail`: io.prebindgen.covertest.resources.OwnedResource,
    public val `details`: io.prebindgen.covertest.values.DataFloats
)
