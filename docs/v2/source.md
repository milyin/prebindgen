<!-- spec: {"kind": "fixture"} -->

# The source fixture

[Project contents](README.md)

Every example path in this specification is written against one small Rust source
crate. Keeping a single fixture lets the paths intersect the way real bindings do:
the function path needs the record path's input conversion, and the record path's
field conversions are what that input conversion is built from.

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

Both declarations are marked for capture; the source module keeps the actual
implementation. Returning a scalar keeps the examples focused on record input:
accessing two foreign fields, converting their values, constructing a source
record, and invoking its function.

The binding configuration exposes both roots — `Stamp` as a public data type and
`stamp_sum` as a public function. The argument is an owned record; the fields and
the result are `i64`. C selects a by-value `StampC` aggregate. Kotlin/JNI — Kotlin's
JVM code calling Rust through the Java Native Interface — selects an
`example.Stamp` object whose properties are read through JNI, rather than
separate field arguments. No constructor helper, handle, borrow or callback is
part of this fixture.

In generated code, `crate::source` names this source module. The C and JNI
outputs are distinct target modules built from the same source model. Loading the
native library belongs to the Kotlin test harness. Capture annotations and build
boilerplate are omitted here; the existing examples supply the complete captured
input.

Two element kinds walk out of this fixture, and each has its own path through the
pipeline:

- [Function taking an owned record][fn] — `stamp_sum`, its owned record parameter
  and its signed 64-bit result.
- [Record with scalar fields][struct] — `Stamp`, its two `i64` fields and their
  conversions.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
