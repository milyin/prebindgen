package io.prebindgen.covertest.cleanup

import io.prebindgen.covertest.OutputCleanupProbe
import io.prebindgen.covertest.resources.OwnedResource

// Same fields as the generated class. The init hook injects failures inside
// actual generated JNI collection decoding, including a partially built row.
public data class OutputCleanupRow(val id: Long, val first: OwnedResource?, val second: OwnedResource?) {
    init { OutputCleanupProbe.row(this) }
}
