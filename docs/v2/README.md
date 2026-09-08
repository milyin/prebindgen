<!-- spec: {"kind": "root"} -->

# V2 specification: pipeline and example paths

Flat builds a checked source model from captured Rust definitions and declared
local helper signatures. Users configure a **language frontend**, the public
Rust API for choosing that language's bindings. The frontend uses Flat views to
create internal binding requests. The **registry** reads the same source model
and plans the requested conversions with descriptions supplied by the language
adapter. Writers turn the completed plans into native Rust and foreign APIs.

Status: proposed architecture for [issue #720](https://github.com/milyin/prebindgen/issues/720).
The engine switch and initial unsupported-output reporting already exist; the
conversion contracts described here are implementation work still to do. Rust
API sketches illustrate the design and are not published APIs.

Start with Flat and follow the source-to-output pipeline:

```text
captured Rust source + declared local helper signatures
  -> Flat builds the checked source model
  -> language frontend creates requests from Flat views and recorded choices
  -> registry plans conversions using Flat facts and target descriptions
  -> registry retains complete supported plans and reports skipped requests
       -> common Rust writer -> native Rust -> cbindgen -> C headers
       -> JNI's Kotlin writer -> Kotlin declarations
```

The stage chapters link to detailed API and architecture contracts, followed by
concrete applications to each matching example. The contracts cover the full
V2 design, including source inspection, policies and identities, relationships,
primitive operations, composition protocols, function plans, dependency and
support handling, frozen output, switching, and implementation acceptance.

The initial example paths specify a function taking an owned, two-field record
and returning an integer, with C aggregate and Kotlin/JNI object representations.
Other example paths remain to be specified; that does not remove those types
or capabilities from the general architecture contracts.

Read downward through the pipeline, or follow one source item through all its
applicable stages. A stage page links to the function and struct cases that
apply there. Each case links back to that stage and its example TOC. Where a
stage depends on the language, the case links to separate C and Kotlin pages.

Each case fixes its input, owner, result and checks. These are normative
requirements for this fixture. Rust API sketches are schematic; code presented
as generated output illustrates the required behavior. The validator checks
structure, not whether the prose is a complete or correct compiler design.

## Follow the pipeline

1. [Capture source items](stages/01-source.md)
2. [Build and inspect Flat](stages/02-flat.md)
3. [Record binding requests](stages/03-requests.md)
4. [Plan value conversions](stages/04-values.md)
5. [Assemble the native boundary](stages/05-boundary.md)
6. [Retain supported output](stages/06-retain.md)
7. [Emit bindings](stages/07-emit.md)

This is the order of information dependencies. The registry can interleave
selection, child planning and representation within recursive conversion
planning; the chapters do not require seven separate passes over the project.
Binding choices and local helper signatures can be recorded before Flat is
published. The request stage turns those choices into requests using completed
Flat views.

## Follow an example

- [Function: stamp_sum](examples/function/README.md) traces the callable, its
  owned record parameter, and signed 64-bit result.
- [Struct: Stamp](examples/struct/README.md) traces the public type and its input
  conversion, including both signed 64-bit fields.

The [shared source fixture](source.md) fixes source names and behavior for both
paths. These paths intersect: the function depends on the struct conversion,
and the record's field conversions are reused in that input plan.

Only applicable cells have pages and links. The struct path has no native
boundary page, so it goes from value planning to retained output. An absent
page does not claim that a capability is implemented, unsupported, or invalid.
Such outcomes belong in the contract of an applicable cell.

Enum, const, callback, option, vector, borrowed-parameter and nested-record-field
paths are explicitly deferred in the manifest. Their end-to-end examples remain to be filled in; the general contracts already
describe capabilities such as optional representations, sequences and callbacks. New
field/parameter variants should get their own example paths when their
behavior differs; the first two paths currently fix scalar fields and an owned
record argument.

The correctness requirement is logical behavior, ownership, error handling and
declared interfaces. Generated text need not be byte-identical to V1. Both
pipelines consume the existing examples' full inputs; V2 emits supported
bindings and reports why other requested elements were skipped.

## Integration requirements

Both pipelines consume the existing examples' complete source inputs and
recorded frontend choices. V2 emits supported bindings and reports missing
capabilities for skipped requests. It does not fall back to V1 for individual
items. Emitted bindings preserve logical behavior, ownership, error handling and
declared interfaces; byte-identical generated text is not required.

The `v2` Cargo feature makes V2 available. `.build()` selects the pipeline using
`PREBINDGEN_PIPELINE=v1|v2`, defaulting to V1 when unset; explicit
`build_with(Pipeline::...)` overrides that environment choice. Selecting V2
without the feature is an error. The existing switching scaffold is described
by [issue #719](https://github.com/milyin/prebindgen/issues/719).

## Maintain the structure

[The format contract](FORMAT.md) defines the document identities, links and
extension rules. [manifest.json](manifest.json) lists stages, examples,
languages and the applicable cells. The [validator](validate.py) checks the
manifest and documents together:

```sh
python3 docs/v2/validate.py
python3 -m unittest discover -s docs/v2 -p 'test_*.py'
```

These commands use only Python's standard library.
