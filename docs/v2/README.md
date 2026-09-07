# V2 binding-generation project

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

The contents and each page's previous/next links follow this order. Implementation
milestones come after the complete pipeline has been explained.

## 1. Source model: Flat

1. [Purpose and source-inspection guarantees](flat/overview.md)
2. [Building snapshots, identity and derived types](flat/model.md)
3. [Lookup, type navigation and independent use](flat/inspection.md)

## 2. Binding requests and conversion planning

1. [The registry's role after Flat](registry/overview.md)
2. [Binding requests, policies and conversion identity](registry/requests.md)
3. [Source construction and decomposition](registry/source-relations.md)
4. [Individual target operations: `PrimitiveSpec`](registry/target-operations.md)
5. [Target representations and composition](registry/representations.md)
6. [Target adapter interface](registry/target-interface.md)
7. [Conversion and exported-function plans](registry/plans.md)

## 3. Complete output

1. [Support, retained plans, reports and output](registry/generation.md)
2. [Concrete generated bindings: C and Kotlin/JNI](registry/primitive-examples.md)

## 4. Implementation and validation

1. [Flat implementation sequence and acceptance](flat/implementation.md)
2. [Registry implementation, switching and acceptance](registry/implementation.md)

The correctness requirement is logical behavior, ownership, error handling and
declared interfaces. Generated text need not be byte-identical to V1. Both
pipelines consume the existing examples' full inputs; V2 emits supported
bindings and reports why other requested elements were skipped.
