<!-- spec: {"kind": "variant", "example": "typedef", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][typedef_represent] · [Element path][typedef]
Owner: the registry; the JNI adapter states the [carrier](../../stages/05-represent.md#describing-target-values-and-operations)

# Type alias declaring an opaque handle — Represent and compose values — Kotlin/JNI

## Input

```text
Crossing { source: Ledger, direction: IntoRust }    relation: atomic   policy: ptr_class example.Ledger
Crossing { source: Ledger, direction: OutOfRust }   relation: atomic   policy: ptr_class example.Ledger
```

## Result

The carrier is `jni::sys::jlong`, and the descriptions are the same three the
C adapter gives, over it:

```text
node(Ledger, IntoRust)   ReprSpec { layout: Scalar(jlong),
                                    protocol: Terminal(from_raw(jlong -> Ledger)),
                                    release: Some(release(jlong)) }
node(Ledger, OutOfRust)  ReprSpec { layout: Scalar(jlong),
                                    protocol: Terminal(into_raw(Ledger -> jlong)),
                                    release: None }
```

Applied to the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s value
`v3` and its parameters `ledger` and `ptr`, they render:

```rust
Box::into_raw(Box::new(v3)) as jni::sys::jlong
```

```rust
::core::ptr::NonNull::new(ledger as *mut source::Ledger)
    .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
    .ok_or_else(|| String::from("null `Ledger` handle"))
```

```rust
drop(::core::ptr::NonNull::new(ptr as *mut source::Ledger)
    .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }))
```

The into-Rust expression is the same text as C's: an integer casts to a
pointer as a pointer does, so one standard description serves both carriers
and the adapters differ in one line. No JNI call is made in any of the three —
the environment is not an operand — so a handle
[conversion](../../stages/04-select.md#select-conversion-relations) is never a
runtime failure. The one failure is the binding one, and
[the boundary][typedef_boundary_jni] routes it to an exception.

## Checks

- A `jlong` is not a `JObject`: a handle is a declared class carried as a
  scalar, and [the Kotlin declaration][typedef_emit_jni] is read off the
  [policy](../../stages/03-requests.md#what-policy-means) — `ptr_class` rather than `data_class` — not off the source type.
- A zero `Long` from Kotlin is the null address, and the message it produces
  names the type, so the exception says which handle was closed or never
  opened.
- Both results are independent: the address handed out owes nothing to the
  JVM, and the value taken back owes nothing to the `Long` it came from.

[typedef]: README.md
[typedef_represent]: 05-represent.md
[typedef_boundary_jni]: 06-boundary.jni.md
[typedef_emit_jni]: 08-emit.jni.md
