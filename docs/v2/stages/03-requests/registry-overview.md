<!-- spec: {"contract": "registry-overview", "kind": "contract", "stage": "03-requests"} -->

[Pipeline chapter](../03-requests.md) · [Project contents](../../README.md)

# Registry V2: purpose and responsibilities

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Flat has now supplied the [checked source model and views](../02-flat/flat-inspection.md).
The next stage combines that source information with the frontend's recorded
binding choices. The registry determines the operations needed to implement
each requested binding.

## Purpose and scope

Build a second generation pipeline around a common **registry**: an engine that takes the Rust source model and the consumer's choices, determines the required conversions, and assembles instructions for generating bindings. Language implementations describe how foreign code represents Rust values and provide the operations specific to their runtime. The registry combines those descriptions into complete conversions and exported functions using algorithms shared by C, JNI/Kotlin, and future languages.

V2 accepts the existing examples' complete Rust inputs and preserves their existing language-frontend APIs. Users configure a C or JNI frontend through Rust builders and macros, typically called from `build.rs`. The choices recorded by those calls—items to expose, names, representations and overrides—are the **binding configuration**. V2 initially generates only supported elements and reports why the others were skipped. The resulting Rust wrappers, C headers and Kotlin sources are the **generated bindings**. For an emitted element, correctness means preserving its declared behavior, ownership, error handling, and public/native interface. Generated source does not have to be byte-identical to v1.

[#719](https://github.com/milyin/prebindgen/issues/719) covers introducing v2 and switching existing examples. The types below are proposed schematic contracts; future capabilities are extension points, not implemented features.

## 1. What the registry does

The registry receives the [Flat V2 source model](../02-flat/flat-model.md) and requests to expose particular types, functions, and constants. For each request, it builds a **plan**: structured data describing the required conversions, calls and their execution order. The common Rust writer renders the native wrappers and supporting Rust types. The C build then uses `cbindgen` to derive C headers from that Rust output; the JNI implementation renders Kotlin declarations from the completed plans. Planning happens in the generator; the generated operations execute later when the bindings are used.

For example, consider this illustrative source API:

```rust
struct Stamp {
    secs: i64,
    nanos: i64,
}

fn normalize(stamp: Stamp) -> Stamp;
```

Generating a binding requires several decisions and operations:

1. Choose how a foreign caller represents a `Stamp`.
2. Obtain and convert its `secs` and `nanos` values.
3. Construct the Rust `Stamp` and call `normalize`.
4. Read and convert the returned fields.
5. Return the result through the chosen foreign interface.

A **target** is a language and its native calling interface, such as C or Kotlin through JNI. Each language implementation provides a **target adapter**, which supplies the representation choices and runtime operations needed by the registry.

A C binding might represent `Stamp` as a C struct. A JNI binding might accept two native integer arguments produced by a Kotlin wrapper, or receive a JVM object whose properties must be read. Those representations need different target operations. The source-side work of discovering two fields, converting them, constructing `Stamp`, invoking the source function, and processing its result is common.

**The registry owns that common work.** When a record gains another nested record field, the shared recursive registry algorithm should process it using the representations supplied by the target. Each language should not need another implementation of record traversal or wrapper assembly.

Each language implementation provides a **frontend**: the public Rust API through which users configure that language's bindings. When the user calls the frontend's build method, the frontend creates **binding requests** for the registry internally. These requests are the registry's input API for frontend implementations; users configure the frontend and do not construct requests themselves. A language implementation may also provide a **foreign writer**, an optional component that renders foreign-language source from the completed plans. The JNI implementation provides a Kotlin writer. The C implementation needs no foreign writer: the external `cbindgen` tool generates C headers from the generated Rust types and functions. The registry library provides the **common Rust writer**, which emits native Rust wrappers and supporting Rust types for both targets.

<table>
<thead>
<tr><th>Component</th><th>Responsibility</th><th>Example C</th><th>Example Kotlin</th></tr>
</thead>
<tbody>
<tr>
<td>Language frontend</td>
<td>Provide the public language-specific Rust API and pass the recorded choices to the registry as binding requests.</td>
<td>Expose <code>Stamp</code> as a C data struct and <code>normalize</code> as a C function.</td>
<td>Expose <code>Stamp</code> as a Kotlin data class in a specified package and place <code>normalize</code> in the requested API.</td>
</tr>
<tr>
<td>Registry</td>
<td>Plan Rust field access, value construction and helper calls; combine child conversions and track dependencies.</td>
<td colspan="2">Convert both fields, construct the Rust <code>Stamp</code>, call <code>normalize</code> once, and process its result using the target's representation operations.</td>
</tr>
<tr>
<td>Target adapter</td>
<td>Describe representations, runtime operations and boundary conventions.</td>
<td>Represent <code>Stamp</code> as a C struct with member-access operations; select the declared C return/out-parameter and error conventions.</td>
<td>Represent <code>Stamp</code> as separate JNI arguments or a JVM object with property-read operations; select the declared JNI result and error-handler conventions.</td>
</tr>
<tr>
<td>Common Rust writer</td>
<td>Render the registry's completed native conversion and function plans.</td>
<td colspan="2">Emit Rust locals, field accesses, source calls, branches and the extern wrapper from the common plans, using the chosen native calling convention and target operations.</td>
</tr>
<tr>
<td>Foreign writer (optional)</td>
<td>Render foreign-language source when the language implementation needs a custom writer.</td>
<td>Not needed. The build invokes the external <code>cbindgen</code> tool on generated Rust to produce C headers.</td>
<td>Emit the Kotlin <code>Stamp</code> class and typed wrappers that call the generated JNI boundary.</td>
</tr>
</tbody>
</table>

The implementation divides the registry's data between two structures:

- **`Registry`** uses an existing `Flat` model and adds the binding-generation operation, `generate(adapter, requests)`. `adapter` is the language implementation's object implementing the proposed `Target` interface; `requests` contains the choices recorded by the frontend. The registry determines how to construct or read Rust values—through their fields, constructors, accessors or conversion helpers—then combines the required conversions, checks dependencies and assembles binding plans. Source types, fields and signatures remain described by `Flat`.
- **`GenerationRun`** holds the temporary planning state inside `Registry::generate`: binding requests, conversion plans being built, dependencies and skip reasons. The registry creates this state internally and processes the complete request set in it. On success, the registry returns a **`Generation`** containing the completed plans and report; on failure, the registry returns an error.

A binding crate normally builds one configured frontend. The frontend calls `Registry::generate` once for all requests. `Flat` supplies source facts; the registry plans conversions using `GenerationRun` working state.
