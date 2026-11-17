# Claude Opus 5.5 review

Reviewed through Copilot CLI with source sharing approved by the user. The
reviewer received the generator diff, owner-accessor implementation and actual
C caller. This was source review; the reviewer did not execute tests.

The first review reported no confirmed blockers, and requested three additional
rejection checks:

- A shared Vec receiver, including a shared collection element, must not reach a
  mutable getter. Both now have tests and fail during generation.
- A mutable slice cannot produce simultaneous mutable and shared output views.
  The existing multiple-view check rejects this; a test also covers a read-only
  getter alongside the mutable identity.
- Mutable callback arguments must remain unsupported. A recursive check now
  rejects mutable references inside options and collections as well as direct
  references. All three forms have generation-error tests.

All C generator tests and two documentation tests pass. The actual C caller
passed ASan/UBSan checks and the full Kotlin/JNI regression caller passed CheckJNI.
Allocation and pointer-identity checks are documented in README.md.

The focused follow-up review reported **no blockers**. Its non-blocking suggestions
were also addressed: array/tuple/Box/Result callback-borrow rejection cases have
explicit tests, and the read-only accessor documentation now states that obtaining
a mutable view invalidates earlier views even without a byte write.
