# External ownership review

Claude Opus 5.5 reviewed the design and initial implementation through the Claude
CLI. When a source review stalled, review continued through Copilot using its
explicit `claude-opus-5.5` model. The final follow-up reported no remaining blocker,
conditional on the repeated boundary checks, which passed. Reviews used supplied
source/diffs and did not execute tests or edit the repository.

Changes made from review:

- Reclaim completed JNI rows without allocating local-reference frames or looking
  up classes on rollback. Cache the bootstrap `Long.value` field before ownership
  transfer, then read it without invoking Java methods.
- Treat row completion as an unsafe ownership operation and inline successful
  accounting. Infallible element commits precede the completed-prefix update;
  nested guards remain armed until their parent row accepts ownership.
- Clear secondary JVM exceptions between destructors and restore the original
  delivery exception. Mixed bulk frames use that isolation recursively per field.
- Compile and execute optional nested arrays and a prepared object-array/bulk
  combination. Fault probes record accepted outer rows before failure and assert
  exact destruction counts, rather than relying only on allocation ordinals.
- Exercise a destructor panic, a secondary JVM exception, a subsequent sibling
  JNI allocation and preservation of the original IOException in the runtime
  cleanup probe. The generated mixed-frame tests cover ownership rollback but do
  not inject JNI calls from their source-type destructors.

Several conditional findings were checked against the shared implementation:
bulk cleanup starts at the frame's root record and recursively follows its row
descriptors; borrowed-source handle copies still produce owned `Move` wires;
row encoding and reclamation use the same field ordering. Fold cleanup closes
constructed wrappers, whose close/take state prevents a second native free;
tests cover close-then-throw and take-then-throw. The existing policy that reclaims
the failing fold step remains unchanged.

Kotlin cleanup lambdas, temporary row arrays/boxing, callback local frames, the
empty-array presence marker and collector growth remain separate work in #824.
