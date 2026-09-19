<!-- spec: {"kind": "report"} -->

[Project contents](README.md)

# The report

Suppose your build succeeds, but a Kotlin function you requested is missing.
Did you forget to expose it, explicitly exclude it, or ask for something V2
cannot generate yet? The report helps distinguish those cases. It records what
happened to every [declaration](stages/03-requests.md#record-binding-requests):
each type, function or other public item the binding configuration asked for.

While V2 is being brought up to V1's coverage, it generates the supported
portion of a binding and skips the requests it cannot handle, rather than
failing the build as the finished engine
[will](stages/07-retain.md#unsupported-requests-and-public-api-dependencies).
A successful generation run therefore does not mean that every requested item
exists — which is the reason this report exists at all. Each declaration's
[outcome](stages/07-retain.md#retain-supported-output) tells you whether it was
emitted, skipped or explicitly ignored, and a skipped entry explains why.

The report is diagnostic output, not configuration. No stage reads a saved
report to decide what to generate, and deleting the report files does not change
the binding. The generated Rust header comment uses counts from the in-memory
report, but the code itself is determined by the retained plans. The sections
below explain how to read the report and request its files from a build script.

## What it says

Start with the run identity. It tells you which engine and target produced the
report, which crate declared the binding, and which source modules and item
count the model was built from. These fields help you recognize the run; they
are not a content hash or proof that the files match the latest source revision.

Next, read the counts of emitted, skipped and ignored declarations. *Ignored*
means the user deliberately excluded an item. *Skipped* means the engine could
not satisfy a request. A skip includes a stable
[capability](stages/07-retain.md#retain-supported-output) code, a readable
explanation and a path to the problem. That path can identify the function
[site](stages/03-requests.md#a-values-position-in-an-exported-function), such
as parameter 0, or the struct field where planning stopped.

Here is the complete JNI report from `examples/v2check`. That test fixture
extends the guide's `Stamp` example with deliberately unsupported cases:

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

Read `type:Reading -> field level` as: “while planning the requested type
`Reading`, the engine could not handle its field `level`.” The code
`unsupported.jni.carrier` identifies the missing JNI support; the explanation
narrows this occurrence to `i32`.

The next group connects two missing outputs to one limitation. `Marker` has no
fields, so it cannot become the requested Kotlin data class. That also prevents
`marker_value` from being exposed: its first parameter needs the missing type.
The path `fn:marker_value -> param 0` explains that dependency. The `Stamp`
type and its three functions remain available because they do not depend on
`Marker`.

The final table accounts for every declaration, including successful ones.
Its [representation](stages/05-represent.md#represent-and-compose-values) column names the requested kind of foreign API, such as
a data class or function. Its *placement* column gives the foreign name, such
as `example.Stamp`. The declaration id, such as `type:Stamp`, identifies the
request by its source origin and does not change when the foreign name changes.

The Markdown rendering groups skips by capability code and uses the first
entry's explanation as the group heading. Different failures can share that
code. For each entry's own explanation and dependency path, inspect the JSON
rendering rather than assuming that the group heading describes every case.

## Where it lands

The built frontend's `write_report(dir)` method writes two renderings of the
same data. Choose the directory explicitly in the build script. For example,
after a configured C builder has produced `c`:

```rust
let out_dir = std::path::PathBuf::from(
    std::env::var("OUT_DIR").expect("Cargo provides OUT_DIR"),
);
c.write_report(&out_dir).expect("write the C report");
```

This excerpt shows the reporting step, not a complete binding configuration.
`jni.write_report(&out_dir)` does the equivalent for a built `JniGen`. The
method creates the directory if needed and returns the paths it wrote. Putting
reports beside generated Rust is a useful convention, not an enforced layout.
With V2 selected, the files are:

- `<target>-report.json` — for tools. Its `schema_version` says which shape it
  has; a consumer checks it before trusting the fields. The CI job that builds
  every example through v2 reads these as a smoke check: the file exists, says
  `v2`, and lists at least one declaration. It does not check that every
  declaration the example made is in it.
- `<target>-report.md` — the rendering above, for a person.

Here `<target>` is `c` or `jni`, so the example writes `c-report.json` and
`c-report.md`. The two targets can share a directory without using the same
filenames. Separate runs for the same target need separate directories if you
want to preserve both reports; writing again replaces the existing files.

`write_report` also prints the summary and grouped skips as Cargo warnings, so
the build log provides a quick overview. These warnings describe skipped
requests, not necessarily a failed build. Under V1, the method writes nothing
and returns an empty list of paths. A build script can therefore call it under
either engine without adding an engine-specific branch.

## How the frontends expose it

Use `Cbindgen::report()` or `JniGen::report()` to inspect the in-memory value
without writing files. These accessors return `Option<&Report>`:

- `Some(report)` means this frontend was built through V2.
- `None` means it was built through V1, which does not produce this report.

There are two separate switches to keep in mind. Compiling the frontend with
its `v2` Cargo feature makes the accessor and its return type available.
Selecting V2 for a particular generation run determines whether there is a
value to return. Without the feature, the accessor does not exist at all.
`write_report`, in contrast, is available with either feature set.

Tests can use `report().unwrap().counts()` or inspect the `declarations` entries
to check expected outcomes without reading files. This generation report is
also distinct from `JniGen::surface_report()`: that V1-only API describes the
resolved Kotlin API in Markdown, rather than accounting for V2's emitted and
skipped requests.

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

The report is not a manifest of what should exist. The binding configuration
expresses that intent; the report records what happened on this run. It is not
a complete inventory of the captured source either: an item never requested
has no entry today. Generated helper functions and native symbol identities
are not separate entries in the current schema.

Finally, *emitted* is not a guarantee that the binding compiles or behaves
correctly. It means generation accepted and produced the declaration. The
binding crate must still compile the generated Rust, compile the foreign code
where applicable, and test the runtime behavior. A report cannot replace those
checks.
