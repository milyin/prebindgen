<!-- spec: {"kind": "report"} -->

[Project contents](README.md)

# The report

A diagnostic the engine writes beside the generated code, saying what became of
each [declaration](stages/03-requests.md#record-binding-requests). It is an
output and nothing else: no generation decision depends on it, no stage reads a
report back, and a binding built with the report files deleted is the same
binding. (The one thing generation takes from the in-memory value is the count
line in the generated file's header comment.) The pipeline chapters mention it
only where an [outcome](stages/06-retain.md#retain-supported-output) is decided
and where the generated files are written; what it is and how to read it is this
page.

## What it says

The run's identity — which engine, which target, the crate that declared the
binding, the source modules the model was built from and how many items they
held — followed by every declaration with its outcome. An emitted declaration is listed as such.
A skipped one carries the [capability](stages/06-retain.md#retain-supported-output)
it waits on, a sentence saying why, and the path from the declaration to the
[site](stages/03-requests.md#a-values-position-in-an-exported-function) where
planning stopped, so the report says what to look at rather than only which
declaration vanished. An ignored one is listed apart, because an ignore is a
decision and not a gap.

```text
# prebindgen v2 report — jni

Declared by `v2check` over 8 captured item(s) from: source

| emitted | skipped | ignored |
| ---: | ---: | ---: |
| 4 | 3 | 0 |

## Skipped, by cause

### `unsupported.jni.carrier` — `i32` has no JNI carrier yet

- `type:Reading` (type:Reading -> field level)

### `unsupported.jni.empty_class` — `Marker` has no fields, and a Kotlin data class needs at least one property

- `type:Marker` (type:Marker)
- `fn:marker_value` (fn:marker_value -> param 0)

## Every declaration

| declaration | representation | placement | outcome |
| --- | --- | --- | --- |
| `type:Marker` | data_class | `example.Marker` | skipped: `unsupported.jni.empty_class` |
| `type:Reading` | data_class | `example.Reading` | skipped: `unsupported.jni.carrier` |
| `type:Stamp` | data_class | `example.Stamp` | emitted |
| `fn:marker_value` | fun | `example.markerValue` | skipped: `unsupported.jni.empty_class` |
| `fn:stamp_delta` | fun | `example.stampDelta` | emitted |
| `fn:stamp_show` | fun | `example.stampShow` | emitted |
| `fn:stamp_sum` | fun | `example.stampSum` | emitted |
```

That is `examples/v2check`'s JNI report, in full.

Skips are grouped by cause first, because that is how the next piece of work is
chosen: one missing capability is stated once with every declaration it took
down.

## Where it lands

Two renderings of the same data, written by the frontend's `write_report(dir)`
into whatever directory the build script names — the examples put them beside
the generated Rust, under the engine's own output root, which is a convention
and not something the call enforces:

- `<target>-report.json` — for tools. Its `schema_version` says which shape it
  has; a consumer checks it before trusting the fields. The CI job that builds
  every example through v2 reads these as a smoke check: the file exists, says
  `v2`, and lists at least one declaration. It does not check that every
  declaration the example made is in it.
- `<target>-report.md` — the rendering above, for a person.

`write_report` also prints the summary line and the grouped skips as cargo
warnings, so a build's terminal shows what a binding did without opening a
file. Under the v1 engine it writes nothing and returns no paths: v1's answer
is "everything declared, or the build failed", so it has no partial surface to
report. A build script can therefore call it under either engine.

## How the frontends expose it

`Cbindgen::report()` and `JniGen::report()` exist when the frontend is compiled
with its `v2` feature — the type they return comes from the v2 crate — and
return `Option<&Report>`: the value for a v2 build, `None` when the feature is
compiled in but the selected engine is v1. `write_report` is available under
either feature set and, as said above, writes nothing for v1. Tests read
`report()` to assert counts, outcomes and capabilities without touching the
file system. (`JniGen::surface_report()` is a different, v1-only thing: a
Markdown explanation of the resolved Kotlin surface.)

## A planned use: selecting tests

The example crates' test suites are written against the full API, so while v2
emits a subset, a test referring to what it skipped would not compile — a
runtime guard cannot hide a missing class from `kotlinc` or a missing symbol
from a C compiler. The plan is to divide each suite into sections, each naming
the declarations it needs, and let the report select the sections whose
declarations were all emitted; a milestone would still have to require that
meaningful sections run, so that skipping everything cannot pass. This is not
built ([implementation](implementation.md#acceptance-and-feasibility-evidence)
lists it), and when it is, it changes what the *tests* do with the report, not
what generation does.

## What it is not

Not an input: nothing selects declarations from a report, and nothing is
configured through one. Not a manifest, which names what should exist; the
report says what did. Not a guarantee of correctness: a declaration listed as
emitted has been planned, assembled and rendered, and rustc has yet to compile
it — that is what the binding crate's build does next.
