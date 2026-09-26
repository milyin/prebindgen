<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "03-requests"} -->

[Stage chapter](../../stages/03-requests.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the language frontend · Previous: [Build and inspect the source model][fn_callback_flat] · Next: [Select conversion relations][fn_callback_select]

# Function taking a callback — Record binding requests

## Input

The element Flat built for this function:

```text
Element::Function(Function {
    name:   stamp_each,
    params: [ Param { name: stamp, ty: Named { id: Stamp } },
              Param { name: each,  ty: Callback { args: [ Scalar(I64) ] } } ],
    ret:    Unit,
})
```

and a build script asking to expose `stamp_each` — and, for C, the callback
signature too: [C][fn_callback_requests_c], [Kotlin/JNI][fn_callback_requests_jni].

## Result

Beside the function's output, the frontend records one for the callback
signature. So that every value of it has a
[conversion](../../stages/04-select.md#select-conversion-relations), it also
records a [conversion rule](../../stages/03-requests.md#conversion-rules)
naming the callback's [representation](../../stages/05-represent.md#represent-and-compose-values):

```text
output   callback:impl Fn(i64)+Send+Sync+'static   type, representation <this target's callback>
rule     Type(impl Fn(i64) + Send + Sync + 'static) -> the same representation
output   fn:stamp_each                              function <the form this target recorded>

positions:
    At(fn:stamp_each, param stamp)         // the owned Stamp, as in the function path
    At(fn:stamp_each, param each)          // the callback — no rule here
    At(fn:stamp_each, param each.arg 0)    // the i64 each call hands out — no rule here
```

The callback's output is `Declaration::Callback`, named by the type's key: no
[source item](../../stages/01-source.md#capture-source-items) declares a
callback, so its signature is the name it goes by. Its form is a type's — a
[representation](../../stages/05-represent.md#represent-and-compose-values)
and the target's metadata — because what it exposes is a foreign type, the
closure struct or `fun interface` a caller fills in; it exports no function of
its own. The argument is one more step below `param each`, and nothing here
names it: it takes the `i64` rule, as any `i64` does.

## Checks

- One output and one rule per signature, not per parameter: every
  `impl Fn(i64) + Send + Sync + 'static` any exported function takes is this
  callback.
- A signature nobody states a rule for leaves the parameter uncovered, and the
  function is skipped with `unsupported.conversion.no_rule` naming the type.
- Nothing here claims the callback or the function can be generated: an
  argument no target can hand out of Rust is found only when it is planned.

## Language variants

- [C][fn_callback_requests_c]
- [Kotlin/JNI][fn_callback_requests_jni]

[fn_callback]: README.md
[fn_callback_flat]: 02-flat.md
[fn_callback_select]: 04-select.md
[fn_callback_requests_c]: 03-requests.c.md
[fn_callback_requests_jni]: 03-requests.jni.md
