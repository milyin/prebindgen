<!-- spec: {"kind": "root"} -->

# V2: the binding pipeline, stage by stage and element by element

This is the specification for the second generation pipeline. It is written to be
read two ways.

**Down the pipeline.** Seven chapters, one per stage, from the captured Rust
source to the emitted C and Kotlin bindings. Each chapter explains what arrives,
who owns the decisions, what leaves, and what happens when a request cannot be
served. Read in order, they are the developer documentation for the design.

**Along one element.** The appendix takes one kind of source element — a
function, a record — and follows it through every stage that applies to it,
showing its exact representation at each step: the captured item, the Flat views,
the recorded request, the conversion plans and target operation descriptions, the
native boundary, the retained output, and the generated code. Read that way, the
document is an example-based specification: the general contract in a chapter and
its concrete application to a fixed input are two views of the same requirement.

Status: proposed architecture for [issue #720](https://github.com/milyin/prebindgen/issues/720).
The engine switch and the initial unsupported-output reporting already exist; the
conversion contracts described here are implementation work still to do. Rust API
sketches illustrate intended contracts, not published APIs, and code shown as
generated output illustrates required behavior rather than bytes produced by the
current scaffold.

## The pipeline

```text
captured Rust source + declared local helper signatures
  -> Flat builds the checked source model
  -> language frontend records binding requests from Flat views and user choices
  -> registry plans value conversions using Flat facts and target descriptions
  -> registry assembles the native boundary of each exported function
  -> registry retains complete supported plans and reports skipped requests
       -> common Rust writer -> native Rust -> cbindgen -> C headers
       -> JNI's Kotlin writer -> Kotlin declarations
```

1. [Capture source items](stages/01-source.md)
2. [Build and inspect the source model](stages/02-flat.md)
3. [Record binding requests](stages/03-requests.md)
4. [Plan value conversions](stages/04-values.md)
5. [Assemble the native boundary](stages/05-boundary.md)
6. [Retain supported output](stages/06-retain.md)
7. [Emit bindings](stages/07-emit.md)

Then: [implementation sequence and acceptance](implementation.md), which is not a
pipeline stage but the plan for building one.

This order is the order of information dependencies, not a requirement to make
seven passes over the project. The registry interleaves selection, child planning
and representation inside one recursive walk; binding choices and local helper
signatures can be recorded before Flat is published, and the request stage turns
those choices into requests once the Flat views exist.

## The appendix: elements and their paths

All example paths share [one source fixture](source.md).

- [Function taking an owned record][fn] — `stamp_sum(Stamp) -> i64`
- [Record with scalar fields][struct] — `Stamp { secs: i64, nanos: i64 }`

Element kinds form a tree: a kind at the root, more specific variants below it,
each becoming its own path when its behavior differs from its parent's. Two paths
are specified today; the rest name the id they will use when they are written.

```text
fn                      specified   function taking an owned record, returning a scalar
  fn_fallible           deferred    function returning Result
  fn_callback           deferred    function taking a callback
  fn_borrowed_param     deferred    function taking a borrowed parameter
  fn_complex_return     deferred    function returning a non-scalar value
struct                  specified   record with scalar fields
  struct_nested         deferred    record with a record field
  struct_option_field   deferred    record with an optional field
  struct_vec_field      deferred    record with a sequence field
enum                    deferred    enum with payload variants
const                   deferred    exported constant
```

Deferring a path is a statement about this appendix, not about the architecture:
the chapters already specify optional values, sequences, variants, callbacks and
error routing in general terms. A deferred path is missing its worked example, and
writing one may well expose a gap in the general contract — that is what the
appendix is for.

The same ids are the document's link vocabulary. `fn` is that element's path;
`fn_values` is that element at the value-planning stage; `fn_values_c` is the C
variant of that cell. A chapter links to the elements that reach it, an element
cell links back to its chapter, and both use these ids.
[The format contract](FORMAT.md) states the grammar and the rules a new path has
to satisfy.

Only applicable combinations exist. The record path has no native-boundary cell,
because a record exports no function of its own; its conversion is reached
through the function that uses it. A missing cell claims nothing about support —
support outcomes belong in the contract of a cell that does exist.

## What V2 has to deliver

Both engines consume the existing examples' complete source inputs and recorded
frontend choices. V2 emits its supported subset and reports the missing capability
for every skipped request; it does not fall back to V1 for individual items.
Emitted bindings preserve logical behavior, ownership, error handling and declared
interfaces. Byte-identical generated text is not required.

The `v2` Cargo feature makes the engine available. `.build()` selects the pipeline
from `PREBINDGEN_PIPELINE=v1|v2`, defaulting to V1 when unset; an explicit
`build_with(Pipeline::...)` overrides that. Selecting V2 without the feature is an
error. [Issue #719](https://github.com/milyin/prebindgen/issues/719) covers that
switch, whose scaffold already exists.

## Keeping the document consistent

[The format contract](FORMAT.md) defines page identities, link ids and the rules
for adding a path or a stage. [manifest.json](manifest.json) lists the stages,
element paths, languages and applicable cells. [The validator](validate.py) checks
the manifest and the documents against each other:

```sh
python3 docs/v2/validate.py
python3 -m unittest discover -s docs/v2 -p 'test_*.py'
```

Both use only Python's standard library. They check structure — identities, ids,
links, anchors, indexes, backlinks, required sections — and cannot check whether
the prose describes a correct compiler.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
