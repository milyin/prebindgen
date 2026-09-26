<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "07-retain"} -->

[Stage chapter](../../stages/07-retain.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the registry · Previous: [Assemble the wrapper boundary][fn_callback_boundary] · Next: [Emit bindings][fn_callback_emit]

# Function taking a callback — Retain supported output

## Input

```text
candidate: output fn:stamp_each, with its FunctionPlan
candidate: output callback:impl Fn(i64)+Send+Sync+'static, with node(each) at its root

requires:  fn:stamp_each -> type:Stamp                                   // param stamp is a Stamp
           fn:stamp_each -> callback:impl Fn(i64)+Send+Sync+'static     // param each is that callback
           callback      -> nothing                                     // an i64 names no type
```

## Result

```text
outcome(type:Stamp)                                  = Emitted
outcome(callback:impl Fn(i64)+Send+Sync+'static)     = Emitted
outcome(fn:stamp_each)                               = Emitted
```

A callback is required the way a type is, by what the parameter crossed as:
the foreign signature of `stamp_each` names the closure struct or the
`fun interface` its parameter is typed as, so the output declaring it has to
survive for the function to. The callback's own requirements are its
arguments', exactly as a struct's are its fields': a callback handing out a
`Ledger` would require the handle's output.

Had the argument been a struct no target hands out of Rust, the refusal
would have been met at the argument, and would take down both:

```text
outcome(callback:impl Fn(Stamp)+Send+Sync+'static) = Skipped { unsupported.struct.out_of_rust,
                                                               path [callback:…, arg 0] }
outcome(fn:stamp_emit)                             = Skipped { unsupported.struct.out_of_rust,
                                                               path [fn:stamp_emit, param emit, arg 0] }
```

## Checks

- A callback is a root: its output is emitted whether or not a function
  takes it, since a C caller may declare a closure struct ahead of the
  function that will take one.
- A function taking a callback nothing declares — a rule covers its type, but
  no `callback:` output exposes that [representation](../../stages/05-represent.md#represent-and-compose-values) — is skipped with
  `unsupported.requirement.unrequested`, as a function taking an undeclared
  struct is.
- Retention is all or nothing: no function over a callback that was not
  retained, and no callback with an argument it cannot hand out.

[fn_callback]: README.md
[fn_callback_boundary]: 06-boundary.md
[fn_callback_emit]: 08-emit.md
