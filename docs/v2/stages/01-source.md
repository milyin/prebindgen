<!-- spec: {"kind": "stage", "stage": "01-source"} -->

# Capture source items

[Project contents](../README.md)

Nothing in this pipeline reads Rust source directly. The first stage turns the
annotated source crate into a stream of captured records, and everything
downstream works from those records and from what the binding crate declares
alongside them.

**Input.** A source crate whose public items are marked with `#[prebindgen]`,
compiled as usual. The proc macro leaves the item in place — the source crate
still compiles and still exports it — and writes a record describing it into the
crate's `OUT_DIR`, one record per line. The crate exports the directory as a
constant, so a binding crate's `build.rs` can find the capture without knowing
where Cargo put it.

**Owner.** The proc macro. It decides what a captured record contains: the item's
syntax as written, the module path it was written in, the feature guards around
it, and the source location for diagnostics. It makes no binding decisions and
knows no target language; marking an item does not say it can be exported to C or
Kotlin, only that it is available to try.

**Second input, from the other side.** A binding crate can also declare the
signature of a **local helper** — a Rust function it provides itself, which the
generated code may call, and which no captured record describes. Those
declarations arrive at the next stage as ordinary source facts, so a helper's
parameters and result are inspected the same way a captured function's are. They
are named here because the source model is built from both inputs, not from the
captures alone.

**Output.** The complete capture for the crate: the marked items, the guards, and
the locations. A construct the capture grammar does not model is retained as an
explicit unsupported record rather than dropped, so the reason survives to the
report at the end of the pipeline. Retaining an item as unsupported is not the
same as failing: only malformed capture data — a record the reader cannot parse,
or a set that contradicts itself — is an error, and it fails the build rather
than silently reducing the generated API.

**What this stage does not decide.** Which items get exposed, under what foreign
names, with which representation, and whether a conversion for them exists. Those
are the [request](03-requests.md) and [planning](04-values.md) stages. Capture is
deliberately indiscriminate: it is cheaper to carry an item that no binding uses
than to lose one that a later target could have exported.

## Elements at this stage

- [Function taking an owned record][fn_source]
- [Record with scalar fields][struct_source]

---

Previous: [Project contents](../README.md) · Next: [Build and inspect the source model](02-flat.md)

[fn_source]: ../examples/fn/01-source.md
[struct_source]: ../examples/struct/01-source.md
