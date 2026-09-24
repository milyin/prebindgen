<!-- spec: {"kind": "example", "example": "typedef"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Type alias declaring an opaque handle

```rust
pub type Ledger = crate::ledger::Ledger;
```

This walkthrough follows a type the foreign side can hold but never look into.
A marked type alias declares a name and says nothing about what is behind it:
a type from another crate, a type whose fields are private, a type that owns a
resource. That is the only way such a type enters the flat API. A foreign
caller never holds the value itself; it holds a **handle** — the address of a
Rust-owned value — which one exported function hands out, another takes back,
and a release the binding exports frees.

The handle is the smallest resource-bearing element, and the first whose
obligations outlive a call: a value handed out belongs to the foreign caller
until it is given back or released. What keeps it simple is that it is atomic.
There are no fields to convert, so the
[conversion](../../stages/04-select.md#select-conversion-relations) in each
direction is one operation, and the registry supplies those operations itself,
because moving a value onto the heap and back is Rust rather than C or Kotlin.
The targets differ in the
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
the address is spelled as — `Ledger *` in C, a `Long` on the JVM — in where a
null handle's failure goes, and in what the foreign declaration looks like.

Unlike a struct, a handle exports a function of its own: its **release**. So
this path has a [wrapper-boundary](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
page, for a wrapper that calls
no source function. The two functions that hand a `Ledger` out and take one
back — `ledger_open` and `ledger_close` — are declared in
[the source crate](../../source.md) for this path and reach its conversions.
Their own paths, a function returning a non-scalar value and a function
consuming a handle, are not specified separately; their wrappers appear here
only where the handle's conversions appear in them.

Deliberately not covered: a handle passed by reference (`&Ledger`), which
borrows what the foreign side keeps rather than consuming it; a handle inside
an `Option`; and a struct declared under a handle
[choice](../../stages/03-requests.md#what-a-choice-records), which crosses the same
way with its fields never read.

Concurrency belongs to whatever holds the handle on the foreign side, not to
the boundary, which is sound however the address arrives. It is worth saying
what the Kotlin writer does with that freedom: every use on this path consumes
the handle, so its class hands the address out through one atomic exchange, and
two threads racing to consume or free one cannot both reach it. The sorted
locking the shipping JNI adapter does in Kotlin answers a shape this path
excludes — a borrowed handle, whose address has to stay valid across the call
into Rust.

1. [Capture source items][typedef_source]
2. [Build and inspect the source model][typedef_flat]
3. [Record binding requests][typedef_requests] · [C][typedef_requests_c] · [Kotlin/JNI][typedef_requests_jni]
4. [Select conversion relations][typedef_select] · [C][typedef_select_c] · [Kotlin/JNI][typedef_select_jni]
5. [Represent and compose values][typedef_represent] · [C][typedef_represent_c] · [Kotlin/JNI][typedef_represent_jni]
6. [Assemble the wrapper boundary][typedef_boundary] · [C][typedef_boundary_c] · [Kotlin/JNI][typedef_boundary_jni]
7. [Retain supported output][typedef_retain]
8. [Emit bindings][typedef_emit] · [C][typedef_emit_c] · [Kotlin/JNI][typedef_emit_jni]

[typedef_source]: 01-source.md
[typedef_flat]: 02-flat.md
[typedef_requests]: 03-requests.md
[typedef_requests_c]: 03-requests.c.md
[typedef_requests_jni]: 03-requests.jni.md
[typedef_select]: 04-select.md
[typedef_select_c]: 04-select.c.md
[typedef_select_jni]: 04-select.jni.md
[typedef_represent]: 05-represent.md
[typedef_represent_c]: 05-represent.c.md
[typedef_represent_jni]: 05-represent.jni.md
[typedef_boundary]: 06-boundary.md
[typedef_boundary_c]: 06-boundary.c.md
[typedef_boundary_jni]: 06-boundary.jni.md
[typedef_retain]: 07-retain.md
[typedef_emit]: 08-emit.md
[typedef_emit_c]: 08-emit.c.md
[typedef_emit_jni]: 08-emit.jni.md
