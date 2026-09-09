<!-- spec: {"kind": "fixture"} -->

[Project contents](README.md)

# The specification's source crate

Every element path in the appendix is specified against one source crate, and
this page is that crate. Sharing it is what lets the paths intersect the way real
bindings do: the function path needs the record path's input conversion, and the
record path's field conversions are what that conversion is built from. A path
that invented its own source could not show that.

It grows with the appendix. Each element kind added there — an enum, a constant,
a callback, a fallible function — adds the declarations it needs here, so the
paths keep describing one coherent crate rather than a collection of unrelated
snippets. What is declared today is what the two specified paths need:

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
implementation. Returning a scalar keeps these paths focused on record input:
obtaining two foreign field values, converting them, constructing a source
record, and calling a function with it.

The binding configuration exposes both roots — `Stamp` as a public data type and
`stamp_sum` as a public function. The argument is an owned record; the fields and
the result are `i64`. C selects a by-value aggregate, which keeps the source name
`Stamp`. Kotlin/JNI — Kotlin's JVM code calling Rust through the Java Native
Interface — selects an `example.Stamp` object whose properties are read through
JNI, rather than separate field arguments. No constructor helper, handle, borrow
or callback is declared here yet; each will arrive with the path that specifies
it.

In generated examples, `crate::source` names this source module. The C and JNI
outputs are distinct target modules built from the same source model. Loading the
native library belongs to the Kotlin test harness. Capture annotations and build
boilerplate are omitted here; the existing example crates supply the complete
captured input.

The pipeline chapters are not written against this crate. They illustrate each
stage with whatever short example makes the point, and today those illustrations
happen to look like the declarations above — a convenience, not a dependency.

The paths specified so far:

- [Function taking an owned record][fn] — `stamp_sum`, its owned record parameter
  and its signed 64-bit result.
- [Record with scalar fields][struct] — `Stamp`, its two `i64` fields and their
  conversions.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
