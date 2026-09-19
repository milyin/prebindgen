<!-- spec: {"kind": "fixture"} -->

[Project contents](README.md)

# The specification's source crate

The appendix follows one small Rust library through the generator. This page
defines that library so that every example starts from the same input. It has
two items: a struct named `Stamp` and a function named `stamp_sum` that accepts
the struct. `Stamp` has named fields the generator can inspect; a tuple struct
would not, and the source model declares one as an opaque type instead.

The two items let us follow a dependency as well as an individual function.
Before generated code can call `stamp_sum`, it must obtain both field values
from the foreign caller and construct a Rust `Stamp`. The struct example
explains that [conversion](stages/04-select.md#select-conversion-relations); the function example uses it. Future examples will
extend this same library with the items they need.

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

The capture examples show both items with a `#[prebindgen]` annotation.
The annotation makes their source available to the generator; the original
crate still owns and compiles the implementation. The function consumes its
`Stamp` argument and returns one signed 64-bit integer. `wrapping_add` gives
the example defined behavior even if the addition overflows.

The binding configuration asks for two public outputs: the type `Stamp` and the
function `stamp_sum`. These explicit requests are called roots, because the
generator starts with them and discovers the supporting conversions they need.

For C, the configuration chooses a struct passed by value and explicitly names
it `Stamp`. The generated Rust entry point reads that C-compatible struct's
members and builds the source crate's Rust struct. These are separate types
even though they have the same name and fields.

For Kotlin/JNI, the configuration chooses a data class named `example.Stamp`.
JNI, the Java Native Interface, is how JVM code calls the Rust library. The native
entry point receives a JVM object and reads its `secs` and `nanos` properties
through getter methods. Both targets then call the same Rust function.

The example does not involve an opaque handle (a reference to a Rust-owned
object), a borrow, a callback, or an extra constructor function. Those features
need their own examples before the appendix can specify their behavior.

The emitted examples call the source module as `source::`. In a real binding,
C uses a configured `.source_module(...)` path or the source crate name.
The current JNI V2 route uses the first source module's crate name, falling back
to `crate` when unavailable; it has no corresponding module-override setting.
C and JNI produce separate target modules from the same Rust items.
The Kotlin consumer is responsible for loading the native library.

`examples/v2check` provides a runnable generation fixture based on this source,
with additional cases for rejection and field-order checks. It parses its source
file and feeds `.items(...)` directly to the frontends; it does not exercise
the annotation/capture stage. The capture records in the appendix illustrate
that stage separately. Its JNI checks compile Rust and inspect Kotlin text;
they do not execute a JVM call.

The general chapters sometimes use other functions to explain a feature, such
as returning a struct instead of an integer. Those sketches are separate from
this fixed appendix input. Follow the links below for the complete paths of
the two items defined here.

The paths specified so far:

- [Function taking an owned struct][fn] — `stamp_sum`, its owned struct parameter
  and its signed 64-bit result.
- [Struct with scalar fields][struct] — `Stamp`, its two `i64` fields and their
  conversions.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
