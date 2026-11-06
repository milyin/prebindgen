Claude Opus 5.5 reviewed the public source through Copilot CLI.

The first pass requested proof that normal decoding still validates pointer
alignment, that cleanup reads remain bounded for truncated roots, and that the
ownership scan matches actual consumption order. It also recommended excluding
unfamiliar nested transport shapes from the optimization.

- Ordinary boxed-handle validation still calls validate_pointer before unsafe
  acquisition. The raw JNI regression includes a misaligned handle encoding.
- Reader bounds-checks each native field read. Like the prior eager scan, the
  deferred scan reclaims readable owned fields even if the root is incomplete;
  missing sibling fields do not prevent cleanup. A truncated fixed record tests
  this policy with one borrowed and one owned handle.
- Compiler input preparation constructs nested sequence children in field order.
  Flat grouped rows acquire their active handles before source constructors.
  Nested malformed-range and consumed-alias regressions verify the consumption
  prefix and exactly-once destruction, including on native worker threads and
  before retrying a callback.
- Eligibility explicitly excludes non-bulk sequences and additional bulk roots
  inside an already cached root. Mixed/non-bulk owned results keep eager guards.

The follow-up withdrew the consumption-order concern and found no remaining
reachable correctness bugs. Cache/guard drop order, commit-after-success,
consumed-alias deduplication, skipped borrows and JNI-free cleanup were reviewed.
The final generation adjustment preserves ordinary callback decoding and adds
success-only commit wrapping only when a deferred guard needs it.
