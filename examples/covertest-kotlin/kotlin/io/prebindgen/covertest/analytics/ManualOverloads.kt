package io.prebindgen.covertest.analytics

import io.prebindgen.covertest.JniErrorHandler
import io.prebindgen.covertest.Storage

/**
 * Hand-written same-named overload demonstrating issue #52's manual path: the
 * generator's `.split_on_param` overloads for `storageExpectSummary` do not
 * preclude a consumer from adding *their own* same-named function. This
 * `Int`-typed arm
 * lives in the same `analytics` package as the generated top-level overloads
 * (a separate file — generated code is never hand-edited) and is resolved by
 * signature, distinct from the generated `(Storage, Long, Double, …)` /
 * `(Storage, Summary, …)` / selector forms. It simply widens to the generated
 * `Long` overload.
 */
fun storageExpectSummary(
    s: Storage,
    count: Int,
    total: Double,
    onError: JniErrorHandler<Boolean>,
): Boolean = storageExpectSummary(s, count.toLong(), total, onError)
