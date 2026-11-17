# Opus 5.5 review

Claude Opus 5.5 reviewed the design through the Claude CLI. Its later source
review stopped responding after 90 seconds, so the source review continued
through Copilot `claude-opus-5.5`, as requested. Copilot confirmed that model in
its call metadata and completed the read-only review with no concrete
correctness or memory-safety defects found.

Reviewed invariants and supporting checks:

- Write the wire row, perform its infallible ownership transfer, then advance the
  completed prefix. A partially prepared row stays with ordinary Rust locals.
- Allocate final storage once and retain its raw pointer, avoiding Box retagging
  when the guard moves after exposing its address. Recover the Box before deriving
  references for failure cleanup. Runtime pointer-movement tests pass under Miri.
- Only standard Vec, literal arrays and standard borrowed slices drive the
  generated loop. Their length matches iteration by construction. Unsafe buffer
  methods document this requirement; no release-mode per-row bounds checks were
  added. `commit` only relinquishes storage.
- C Result wrappers first write private stack storage and copy to caller outputs
  only on success. Failure leaves caller outputs untouched; late failure after
  ownership commitment uses existing typed rollback.
- Empty exported buffers remain null/zero. Aligned rows and uninitialized tails
  are covered by Miri. `MaybeUninit<T>` and T use the same allocation layout, so
  the existing generated `Box<[T]>` row-storage free remains compatible.
- One schema-driven cleanup implementation serves pending row guards and
  post-commit rollback, including nested buffers, owned pointers and inline
  opaque String owners. Conversion rollback contains destructor panics per field
  and per row. Tests include a destructor panic during conversion unwinding.

The reviewer noted that borrowed inline-owner views (`by_value: false`) are
ignored by cleanup; these are non-owning output views and must remain alive.
The existing non-catching error-destructor path is unchanged. Neither note
requires a change to buffered conversion ownership.
