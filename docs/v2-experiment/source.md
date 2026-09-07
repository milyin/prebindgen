<!-- spec: {"kind": "fixture"} -->

# Shared source fixture

[Experiment contents](README.md)

This fixture is the source shared by the [function](examples/function/README.md)
and [struct](examples/struct/README.md) paths. Capture marks both declarations;
the source module retains the actual implementation. The body below is source
code, not a primitive payload or a generated wrapper.

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

Configure both public roots: expose Stamp as a data type and stamp_sum as a
function. The argument is an owned record; fields and return are i64. C selects
a by-value StampC aggregate. Kotlin selects an example.Stamp JVM object, not
separate field arguments. No constructor helper, handle, borrow or callback is
part of this fixture.

In generated examples, `crate::source` names this source module. The C and JNI
modules are distinct target outputs. Native-library loading belongs to the
Kotlin test harness. Source capture annotations and build boilerplate are
omitted here; existing examples continue to supply the complete captured input.
