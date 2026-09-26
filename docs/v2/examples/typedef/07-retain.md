<!-- spec: {"kind": "cell", "example": "typedef", "stage": "07-retain"} -->

[Stage chapter](../../stages/07-retain.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry · Previous: [Assemble the wrapper boundary][typedef_boundary] · Next: [Emit bindings][typedef_emit]

# Type alias declaring an opaque handle — Retain supported output

## Input

The candidates this alias produced, and what they require:

```text
candidate: output type:Ledger, with its release FunctionPlan  // under the type's own identity

planned:   node(Ledger, IntoRust)
           node(Ledger, OutOfRust)
           the release wrapper
requires:  nothing
```

## Result

```text
Retained {
    output:       type:Ledger,
    declaration:  public Ledger in this target,
    output_value: node(Ledger, IntoRust),
}

outcome(public Ledger) = Emitted { artifacts: [ carrier declaration, release wrapper ] }

outcome(exported ledger_open)  = Emitted    // requires public Ledger, which is
outcome(exported ledger_close) = Emitted    // emitted — so both stand
```

Had the binding given the type output no release form:

```text
outcome(public Ledger)         = Skipped { cause#1: unsupported.type.no_release }
outcome(exported ledger_open)  = Skipped { cause#1, path [fn:ledger_open,  type:Ledger] }
outcome(exported ledger_close) = Skipped { cause#1, path [fn:ledger_close, type:Ledger] }
```

The type's [outcome](../../stages/07-retain.md#retain-supported-output)
requires both directions and the release. One refused direction, or a release
with no form, skips the type; the report names the refused direction
(`out_of_rust`) or the release as the place the walk stopped. A function
returning a handle requires the handle's public declaration, exactly as
[one taking a struct requires the struct's][fn_retain]: a C caller cannot hold
a `Ledger *` to a type the header never names, and a Kotlin caller cannot free
a `Long` with a method that was not emitted. Requirements travel from return
types as well as from parameters.

## Checks

- A handle is a root and is retained on its own, release included, whether or
  not a function returns or takes one — a caller holding a handle from an
  earlier build still needs to free it.
- The release [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
  is never listed as a declaration of its own. It is retained with the type and
  counted in its outcome, so a report of "one type emitted" means the
  destructor exists.
- Retention is all or nothing: no type without its release, no release
  without its type, no function over a type that was not retained.

[typedef]: README.md
[typedef_boundary]: 06-boundary.md
[typedef_emit]: 08-emit.md
[fn_retain]: ../fn/07-retain.md
