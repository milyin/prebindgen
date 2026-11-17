# Claude Opus 5.5 review

Reviewed through Copilot CLI with source sharing authorized. Three code-review rounds examined grouped replay, eligibility, receipts and cleanup; the final round approved with no remaining blockers in the current API. A fourth focused review approved borrowing stable C rows as a shared Rust slice, conditional on bounds/lifetime checks.

The implementation restricts C replay to built-in Scalar/Pointer leaf mappings and an exhaustive set of declarations whose construction remains deferred. Every resource must be unregistered. Audits cover all 16 scalar types and owning/shared/exclusive opaque pointers. A nested scalar-only constructor beside a handle is emitted once after accept_group. Buffers/enums/inline owners/callbacks/nested collections do not elect outer replay. Both JNI transports explicitly keep reread_rows false.

Existing actual C tests reject duplicate consuming handles with consumed=0, and exercise rows [1,2,3] whose constructor panics on row 2: consumed=2, previous/current owners destroyed exactly once, unread row 3 remains caller-owned. No rollback to original object state is required. Consumption count advances immediately before the infallible Box acquisition, as in the staged path, with no observable failure or user call between them.

The shared-slice review conditions were checked: validate_slice rejects null/misaligned nonempty inputs, checked-multiplies byte size, bounds it by isize::MAX and checks address overflow. Null empty arrays use &[] rather than from_raw_parts. The open/bind scopes cover both passes, and generated numeric wire prefixes cannot collide with the rows-suffix auxiliary variable. Physical row fields are Copy scalars/raw pointers, so no pointee borrow or hidden clone is introduced.

Generated Result and non-Result safety docs and callback-return comments describe stable wire storage. C pointer qualifiers and ABI signatures remain unchanged. The caller precondition is stronger than the previous snapshot behavior and is explicitly documented. No debug/runtime stability registry was added. The reviewer did not execute tests; actual test results are reported in the PR.
