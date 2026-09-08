<!-- spec: {"kind": "stage", "stage": "03-requests"} -->

# Record binding requests

[Project contents](../README.md)

## Input

Users configure a language frontend through its existing Rust API. The frontend holds recorded choices and the Flat views they name.

## Operation and owner

The frontend creates internal BindingRequests. An output request asks for a public function/type/constant; conversion rules select how particular types or value positions cross the language boundary. Type representation and function error policy remain distinct choices. The examples specify the exact recorded choices, without inventing replacement public builder APIs.

## Output and failure contract

The registry receives the complete request set, ignored/unsupported entries and target policies. Invalid names or contradictory choices are errors. A valid but unimplemented setting remains an explicit unsupported request. Parameter names and field positions identify configuration sites; they are not independently exported items.

## Detailed contracts

- [Registry V2: purpose and responsibilities](03-requests/registry-overview.md)
- [Binding requests and conversion identity](03-requests/binding-requests.md)

## Apply this stage

- [Function: stamp_sum](../examples/function/03-requests.md) · [c](../examples/function/03-requests.c.md) · [kotlin](../examples/function/03-requests.kotlin.md)
- [Struct: Stamp](../examples/struct/03-requests.md) · [c](../examples/struct/03-requests.c.md) · [kotlin](../examples/struct/03-requests.kotlin.md)

## Pipeline navigation

[Previous](02-flat.md) [Next](04-values.md)
