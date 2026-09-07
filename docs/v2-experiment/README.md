<!-- spec: {"kind": "root"} -->

# V2 specification experiment: pipeline and example paths

This is an alternative organization of the [V2 design](../v2/README.md).
The existing documents remain unchanged. This experiment specifies one concrete
slice: a function taking an owned, two-field record and returning an integer,
with C aggregate and Kotlin/JNI object representations. It is proposed behavior,
not a claim that the current V2 scaffold implements these contracts.

Read downward through the pipeline, or follow one source item through all its
applicable stages. A stage page links to the function and struct cases that
apply there. Each case links back to that stage and its example TOC. Where a
stage depends on the language, the case links to separate C and Kotlin pages.
There is no rendered matrix to scan.

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
paths are explicitly deferred in the manifest. Their absence is an unfilled
specification scope, not an assertion that they can skip the pipeline. New
field/parameter variants should get their own example paths when their
behavior differs; the first two paths currently fix scalar fields and an owned
record argument.

## Maintain the structure

[The format contract](FORMAT.md) defines the document identities, links and
extension rules. [manifest.json](manifest.json) lists stages, examples,
languages and the applicable cells. The [validator](validate.py) checks the
manifest and documents together:

```sh
python3 docs/v2-experiment/validate.py
python3 -m unittest discover -s docs/v2-experiment -p 'test_*.py'
```

These commands use only Python's standard library. No changes to generation,
existing examples, or the current proposal are required to try this structure.
