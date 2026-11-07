Claude Opus 5.5 reviewed the public source through Copilot CLI in three passes.

- The initial review found that a separate bulk field inside an ordinary JVM
  object row could defer a borrowed buffer temporary beyond its row-local frame.
  The compiler now tracks bulk sequence row scope separately from grouped input
  scope, resetting that scope when entering a separate bulk root. A compiled
  Kotlin/JNI regression covers borrowed String/Vec constructor arguments in
  ordinary object rows, including partial failure. The follow-up confirmed this
  fix and the reader/temporary lifetimes.
- The follow-up raised a possible direct nullable-leaf conversion problem. Review
  of the declaration resolver established that Option fields become Optional
  recipes before leaf compilation, and the leaf codec rejects unresolved Option
  types. Opus withdrew this finding after checking optional niche rewriting.
- Final source review reported no remaining substantive findings. Constructor
  counts, repeatable native reads, callback unread-owner cleanup, consumed-input
  reporting and default Codec compatibility were reviewed.

The suggestion to repeat UTF-8 checks to preserve all-or-nothing consumption was
not adopted. Conversion errors may consume inputs; generated code must drop
acquired values and correctly report consumed handles. The boundary suite checks
malformed later buffers, inactive optional descriptors, conflicting handles,
constructor panics, ordinary unread inputs and callback-returned unread owners.
