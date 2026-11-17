# Claude Opus 5.5 review

Claude Opus 5.5 reviewed the ownership design through the Claude CLI (`claude-opus-5-5`) and the implementation through Copilot (`claude-opus-5.5`). The source review found no blockers. Both reviews were supplied repository source and requested read-only analysis.

The design review required an independent, unscoped plan for unread-row cleanup. The live row uses the shared journal only for nested handle collections; unread rows always start with fresh locals and no constructed owners. Current-row cleanup advances the processed count before releasing resources, so outer failure cleanup cannot destroy the row twice. Successful delivery advances that count and releases journal references outside the row catch.

Source-review follow-ups were verified:

- `jni_output_mixed_frames` exists in the flat fixture and its generated fold compiles and runs through JNI. It tests bulk frames inside the per-element transport; ordinary nested object arrays are covered separately.
- A new native destructor-panic case verifies that a failing current-row destructor does not prevent sibling/unread cleanup, replace the original fold exception, or close previously delivered rows. Exact live/drop counts and suppression are asserted.
- Code inspection confirms that a shared journal is passed through record components in schema order. Existing tests also cover handles around a nested collection; callback-fold coverage includes flat records, optional/nested records and mixed frames.
- Clippy caught unnecessary mutable references in the test setup; those were removed.

Complete C sanitizer and Kotlin/JNI callers passed, with JNI diagnostics enabled. The Kotlin tests exercise constructor failures at every source row, user-fold failures, close/take, reentry, optional/empty rows, mixed bulk frames and destruction counts. Corrupted generated raw rows and catastrophic failure while adding a suppressed exception retain the previous limitations.
