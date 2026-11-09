# External ownership review

Claude Opus 5.5 reviewed the design through the Claude CLI. A later CLI source
review stopped responding, so source review continued using Copilot's
`claude-opus-5.5` model, as requested. The CLI confirmed the requested model in
its call metadata. Reviews were read-only and covered the relevant source/diff.

Changes made from review:

- Keep pending nested ownership as a stack, retiring child registrations when a
  completed parent row takes over recursive cleanup. Metadata must not grow with
  the number of completed rows.
- Require per-row transfer code to have an infallible unit result. Store all
  fallible fields before transferring their owners.
- Use the existing typed allocation destructor and catch destructor panics per
  field so rollback continues.
- Bind the actual standard Vec, rather than merely type-checking a coerced
  reference to a value that might have different iteration semantics. Use fully
  qualified standard length and iterator operations.
- Check exact destruction counts per allocation, including a completed outer row
  followed by an incomplete nested row. Exercise delivery allocation failure and
  multiple independent output frames through the generated JNI boundary.

The final source review found no defects in the two inline owner slots,
registration truncation, recursive rollback, final transfer or pointer-provenance
handling. Its remaining concern assumed an owned custom iterator whose length
could disagree with iteration. After reviewing `SequenceView` and
`sequence_parts`, Opus withdrew that finding: recognized owned collections are
standard Vecs or literal Rust arrays; borrowed views iterate the same standard
slice used for length. No additional per-row runtime checks were added.

Runtime rollback tests also cover destructor panic during an existing conversion
unwind; they pass under Miri. Miri exposed a preexisting integer-pointer recovery
problem when a Box is moved after exposing its address. The shared boxed-output
commit now exposes the final `Box::into_raw` pointer's provenance. Miri uses its
normal exposed-provenance mode; the passing tests do not establish strict
provenance for this integer-handle ABI.
