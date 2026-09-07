# V2 binding-generation project

This project describes a common registry that plans conversions and assembles
Rust bindings for multiple languages, together with Flat's checked Rust
source-inspection API. Language adapters describe foreign representations and
runtime operations. Flat describes source facts; the registry combines the
required operations into complete bindings.

Status: proposed architecture for [issue #720](https://github.com/milyin/prebindgen/issues/720).
The engine switch and initial unsupported-output reporting already exist; the
conversion contracts described here are implementation work still to do. Rust
API sketches illustrate the design and are not published APIs.

Start with the registry overview, then follow the topics below. The Flat project
can also be read independently. The concrete C and Kotlin/JNI examples show how
adapter-supplied primitive operations become registry-assembled native wrappers.

## Registry and language adapters

1. [Purpose and responsibilities](registry/overview.md)
2. [Binding requests, policies and conversion identity](registry/requests.md)
3. [Source construction and decomposition](registry/source-relations.md)
4. [Individual target operations: `PrimitiveSpec`](registry/target-operations.md)
5. [Concrete primitive examples: C and Kotlin/JNI](registry/primitive-examples.md)
6. [Target representations and composition](registry/representations.md)
7. [Target adapter interface](registry/target-interface.md)
8. [Conversion and exported-function plans](registry/plans.md)
9. [Support, registry state and output](registry/generation.md)
10. [Implementation sequence, switching and acceptance](registry/implementation.md)

## Flat source model

1. [Purpose, scope and the registry boundary](flat/overview.md)
2. [Building snapshots, identity and derived types](flat/model.md)
3. [Lookup, type navigation and independent use](flat/inspection.md)
4. [Implementation sequence and acceptance](flat/implementation.md)

The correctness requirement is logical behavior, ownership, error handling and
declared interfaces. Generated text need not be byte-identical to V1. Both
pipelines consume the existing examples' full inputs; V2 emits supported
bindings and reports why other requested elements were skipped.
