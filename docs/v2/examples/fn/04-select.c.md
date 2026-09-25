<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_select] · [Element path][fn]
Owner: the registry, from the rules the C frontend recorded

# Function taking an owned struct — Select conversion relations — C

## Input

The two [crossings](../../stages/03-requests.md#finding-an-existing-conversion-plan), and the rules the C frontend recorded for their types:

```text
Crossing { source: Stamp, direction: IntoRust  }   Type(Stamp), into Rust -> Parts through Fields, wire type `Stamp`
Crossing { source: i64,   direction: OutOfRust }   Type(i64), out of Rust -> Whole over `i64`, identity
```

## Result

```text
param stamp   Stamp, IntoRust   -> Stamp.fields, carried in `Stamp`
  field secs  i64,   IntoRust   -> atomic,       carried in `i64`
  field nanos i64,   IntoRust   -> atomic,       carried in `i64`
return        i64,   OutOfRust  -> atomic,       carried in `i64`
```

`Stamp` is read through its fields because the build script declared it with
`data_type!`, which the C frontend records as `Parts` through `Fields`: a C
struct passed by value is made of its members, so the
[conversion](../../stages/04-select.md#select-conversion-relations) has to be
made of the field conversions. Had it declared `Stamp` with `ptr_type!`, the
rule would name a `Whole` over a pointer, and the registry would plan the
whole value behind a pointer, its fields never read. No C code runs to decide
either: the [relation](../../stages/04-select.md#what-a-relation-is) is the
rule's.

The three `i64` positions take the rules the C frontend records for every
`i64`, one per direction: a `Whole` over the `i64`
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
each way, which crosses unchanged. One [representation](../../stages/05-represent.md#represent-and-compose-values) per direction, so the registry plans one `i64`
conversion per direction rather than one per position.

## Checks

- The relation is resolved by the registry from the rule; `Stamp.fields` is
  the struct relation it worked out from Flat's field list.
- A C aggregate can have only `I64` parts, and both fields resolve to one, so
  the aggregate accepts them. A field of a type no rule covers, or one
  carried as anything else, is refused where the member is, and the function
  is [skipped with that cause][fn_retain] rather than planned around.
- The selection for `Stamp` is the same one [the struct's C page][struct_select_c]
  shows; this function does not choose differently for its own parameter.

[fn]: README.md
[fn_select]: 04-select.md
[fn_retain]: 07-retain.md
[struct_select_c]: ../struct/04-select.c.md
