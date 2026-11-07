# Claude Opus 5.5 reviews

Three reviews were requested through Copilot CLI, with repository source sharing
approved by the user. The reviewer received generator source/diffs and boundary
fixtures; these were code reviews, not independent test executions.

The first review identified a real C callback identity problem: source signatures
alone could reuse a typedef across borrowed, copied and expanded argument layouts.
Callback identities now include canonical physical wire and ownership schemas,
with no Debug formatting, spans or generated temporary names. Tests verify stable
names under declaration reordering and shared names for equivalent layouts.

Other questions were checked against compiler context: source references are
unwrapped before leaf projections; explicit borrowed projections intentionally
authorize recursive owning copies; owned getter results still move; native handle
numbers and enum tags are scalar bulk fields. Unsized string/slice views retain
their existing encoder shapes. Default JNI copy configuration preserves existing
callback cleanup scope.

The second review reported no confirmed blockers and requested two concrete
checks: callbacks sharing a wire layout but using different getters, and enums
inside explicitly copied borrowed bulk rows. Actual C and Kotlin fixtures cover
both. The final focused review reported **no blockers**, conditional on the final
Kotlin boundary run passing. That run is recorded with the PR validation results.

Coverage also includes retained optional copies, source destruction, native worker
callbacks, non-Clone objects, pointer identity and exact allocation/clone budgets.
No latency improvement is claimed from these count-based measurements.
