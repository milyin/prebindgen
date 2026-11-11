# Canonical single-value conversions

A Rust domain type can have one public foreign value without becoming a handle
or a generated foreign class. For example, `Millis` can be Kotlin `Long` and C
`int64_t`, including as a property of `Settings`, an optional value, a collection
element, or a callback argument/result. The existing boundary expansions already
reshape function arguments/results; the missing operation is choosing this value
representation consistently wherever a value occurs.

## API

```rust,ignore
.convert(convert!(Millis)
    .input(fun!(millis_from_long))
    .output(fun!(millis_value)))
.convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
.convert(convert!(Percent).input(try_from!(i32)).output(try_into!(i32)))
```

Each direction is optional and independent. A captured input function takes one
representation value (or a shared reference to it) and returns `T` or
`Result<T, E>`. An output function takes `T` or `&T` and returns its representation
or `Result<Repr, E>`. Captured signatures in Flat remain authoritative; conversions
reference their existing function names instead of copying signatures into a new
registry. Trait declarations explicitly state the other type and use
fully qualified `Into`/`TryInto` calls. Borrowing is never implemented by an
implicit clone. Consuming-only output conversions cannot observe borrowed values.

Both global declarations and local overrides resolve to the same `ValueConversion`
node; they share compilation, error routing, borrowing and ownership reporting.
At a function/callback boundary the precedence is: explicit local tree, type-wide
boundary expansion, canonical value conversion, built-in representation. This
preserves production expansion precedence. Properties and conversion representation
types use canonical values without applying unrelated boundary expansions.

The same declaration can be used as a local override:

```rust,ignore
fun!(echo).expand_param("value", convert!(Millis).input(from!(i32)))
fun!(echo).expand_return(convert!(Millis).output(into!(i32)))
```

Callback argument overrides select the output direction; callback return overrides
select input, as with the existing expansion API. Existing explicit constructor,
getter, projection and bulk trees remain usable as local overrides.

## Preparation and emission

1. Capture/parse source types and helper signatures as today.
2. Resolve and validate each conversion on its source type in Flat. Flat stores
   its representation type and the selected Rust function or trait operation.
   Adapters do not keep a second conversion registry.
3. Resolve declaration requests into the existing recursive Input/Output trees.
   Add a single-value conversion node whose child converts the representation.
   Properties and representation types request the value form, independently of
   function-boundary expansion defaults. Explicit local trees take precedence.
4. Compile those nodes with the existing shared declaration compiler. Input
   conversion stays one scoped operation; output retains prepare/deliver/commit.
   Code emission needs no declaration registry and introduces no runtime dispatch.
5. Derive Kotlin property types from the prepared representation, recursively.
   Where both directions are declared, a property needs compatible public types;
   a function's independent input/output types may differ. Diagnose cycles and
   incompatible properties during generation, rather than emitting broken Kotlin.

An input conversion constructs an owned Rust value. Existing borrowed/native
representations of `&T` remain usable; its owned `T` conversion is not applied
automatically. `&T` is not an instruction to
construct a temporary: callers must explicitly request reconstruction, using the
existing temporary-input operation, if they want that behavior. Output functions
borrowing `&T` can observe owned or borrowed sources without copying. A temporary
representation returned by a conversion must live through its output delivery;
borrowed views into that temporary may not escape its scope.

## Errors and ownership

Conversion failures are binding failures, like malformed enum/input conversion,
and require a displayable error. They use the existing binding-error channel;
Kotlin user handlers run after the native call. Source-function `Result<T, E>`
errors keep their existing independently configured typed error channel. Callback
return conversion failures retain the existing non-unit callback failure policy.
C retains its explicit abort/result policy and trusts raw pointer validity.
Conversion helpers remain source-only unless explicitly exported. Exporting the
same helper separately does not change the conversion use's error role. A failed
unit-callback argument conversion uses the existing unit-callback error reporting
and does not invoke the foreign callback; non-unit callbacks keep their current
panic-on-failure contract. Bulk eligibility and ownership come from the converted
representation tree, with every buffered output prepared before delivery.

Input ownership transfers come from the representation child's wire tree and are
reported by its actual takes. A failed conversion drops constructed Rust values;
no rollback is promised. Output representation resources remain owned until
foreign delivery succeeds. There are no implicit clones, per-value wrappers,
registration maps or runtime conversion dispatch added by this API.

## Scope and verification

This task includes captured helper functions, From/Into and TryFrom/TryInto,
chained representations, local overrides, data properties, options, collections,
callbacks and ordinary/bulk transport. Binding-local helper signatures and declared
scalar domains/niches remain their own backlog tasks. Property representation
support does not imply adding payload enums, reference-valued records or arbitrary
C by-value structs.

Tests must exercise generated Rust plus actual C and Kotlin callers, including
conversion failures, ownership cleanup, borrowed outputs and input reconstruction,
cyclic declarations, incompatible property directions and local overrides. Record
changed generated sources. Compare the new conversions with equivalent explicit
expansions and verify unchanged declarations retain their generated artifacts.
