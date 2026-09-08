<!-- spec: {"kind": "stage", "stage": "07-emit"} -->

# Emit bindings

[Project contents](../README.md)

## Input

Writers receive the immutable retained Generation. Source data, target metadata and generated artifact dependencies are already resolved.

## Operation and owner

The common Rust writer renders registry instructions and calls target operation renderers with allocated operands. The C build runs cbindgen on generated Rust to obtain headers. JNI's optional foreign writer renders Kotlin from the same retained plans. Output is published only after the required generation steps succeed; an I/O failure is not an unsupported-element report.

## Output and failure contract

The output is native Rust, the selected language artifacts and a matching report/test selection. Logical behavior, ownership, errors and declared interfaces must match the request; spelling and byte identity are not requirements. The example cells identify the source and owner of each emitted fragment.

## Detailed contracts

- [Concrete primitive examples: C and Kotlin/JNI](07-emit/primitive-examples.md)
- [Registry V2 implementation and acceptance](07-emit/implementation.md)

## Apply this stage

- [Function: stamp_sum](../examples/function/07-emit.md) · [c](../examples/function/07-emit.c.md) · [kotlin](../examples/function/07-emit.kotlin.md)
- [Struct: Stamp](../examples/struct/07-emit.md) · [c](../examples/struct/07-emit.c.md) · [kotlin](../examples/struct/07-emit.kotlin.md)

## Pipeline navigation

[Previous](06-retain.md)
