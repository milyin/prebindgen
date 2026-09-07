# Concrete primitive examples: C and Kotlin/JNI

[V2 project contents](../README.md) · [Individual target operations](target-operations.md)

These examples continue the `PrimitiveSpec` design with concrete expected output.
The snippets are illustrative output for the proposed V2 engine, not files
produced by its current scaffold. The planning snippets show proposed data
contracts; the complete Rust and Kotlin examples show what those contracts mean
when rendered.

Both bindings expose the same small Rust API. Returning a scalar keeps the
examples focused on record input: accessing two foreign fields, converting their
values, constructing a source record, and invoking its function.

## Source API shared by both bindings

The source crate supplies these definitions (`source.rs`):

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

The language frontend records which function to expose and how to represent
`Stamp`. Flat supplies the function signature and fields. The registry discovers
that the function needs a `Stamp` input and an `i64` output. The selected record
relation supplies the two field types; the registry plans each child conversion.

The two representations below use different field-access operations. In both,
the registry receives already described operations and builds this execution
order:

```text
obtain secs carrier -> convert to source i64
obtain nanos carrier -> convert to source i64
construct source::Stamp { secs, nanos }
call source::stamp_sum once
convert its i64 result -> deliver through the native return
```

Here the signed 64-bit scalar conversions preserve the value directly. The
registry can render an identity conversion without an extra helper call. The
record construction and source call remain registry instructions even when
those scalar conversions emit no additional code.

## C: a struct passed by value

The C frontend selects a C aggregate named `StampC`, with signed 64-bit members,
and an exported function named `stamp_sum_c`. The C adapter describes the
aggregate's Rust ABI type, member identities and ordinary member-read operations.

### The member-read primitive specifications

A **member identity** refers to one member of the described target aggregate.
It is different from the source field identity, because source and target names
can differ. The adapter's representation maps each source child to its target
member. The registry validates that mapping.

This is the stored specification for reading the target `secs` member:

```rust
// Specification sketch. These IDs refer to definitions in the adapter's
// returned description, imported and checked by the registry.
let read_secs = PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![OperandSpec {
            ty: OperationType::Carrier(stamp_c_type), // Rust StampC ABI carrier.
            access: Access::Shared,
        }],
        results: vec![OperationType::Carrier(c_i64_type)],
    },
    failure: PrimitiveFailure::Infallible,
    validity: ValidityContract {
        results: vec![ResultValidity::Independent],
    },
    resources: ResourceContract::none(),
    dependencies: vec![stamp_c_declaration],
    implementation: COperation::Rust(StandardRustOp::ReadMember {
        member: secs_member,
    }),
};
```

`stamp_c_type` and `c_i64_type` are registered carrier type references.
`stamp_c_declaration` identifies the generated `StampC` type artifact.
`secs_member` identifies that declaration's `secs` member. The `nanos`
specification has the same contracts and uses `nanos_member` instead.
`ResourceContract::none()` denotes no extra runtime ownership obligation.

`COperation` is the C adapter's rendering payload type for this example. Its
`Rust` variant selects a common Rust operation supplied by the registry library.
`StandardRustOp::ReadMember` means a typed Rust field read. The common writer
already knows how to render that operation, so C supplies no custom field-read
renderer. A copied integer is independent of the aggregate after the read.

The field expression is **not stored as a string containing `arg0.secs`**.
`PrimitiveSpec.implementation` stores the selected operation and member identity.
The registry's application supplies the particular aggregate value, and the
common writer assigns that value a Rust name. With the allocated operand name
`arg0` and the member spelling `secs`, rendering this one primitive produces:

```rust
arg0.secs
```

The registry puts that expression into a local assignment and then constructs
the source record. Neither the local assignment nor source construction lives
inside the member-read primitive.

### Expected generated Rust

The common Rust writer renders the type from the C adapter's aggregate
description and the wrapper from the registry's function plan (`c.rs`):

```rust
use crate::source;

#[repr(C)]
pub struct StampC {
    pub secs: i64,
    pub nanos: i64,
}

#[no_mangle]
pub extern "C" fn stamp_sum_c(arg0: StampC) -> i64 {
    let v0 = arg0.secs;
    let v1 = arg0.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

The C adapter selects `repr(C)`, the extern calling convention and public names.
The common writer renders those declarations. The expressions `arg0.secs` and
`arg0.nanos` implement the selected target primitives. The registry supplies
their order, local bindings, source-record construction, source call and return
placement. All native local names are allocated by the common writer.

### C header and caller

The C build runs `cbindgen` on generated Rust. The resulting header has this
public shape; formatting and include guards can differ:

```c
#include <stdint.h>

typedef struct StampC {
    int64_t secs;
    int64_t nanos;
} StampC;

int64_t stamp_sum_c(struct StampC arg0);
```

A caller can use it as follows:

```c
#include "bindings.h"

int main(void) {
    StampC stamp = { .secs = 12, .nanos = 34 };
    return stamp_sum_c(stamp) == 46 ? 0 : 1;
}
```

The header is `cbindgen` output. There is no generated C implementation file or
custom C foreign writer in this design; the function body is the Rust wrapper.

## Kotlin/JNI: an object with property getters

The JNI frontend selects a Kotlin data class and object input at the native
boundary. JNI means the Java Native Interface used by Kotlin's JVM code to call
Rust. This example deliberately receives one object; separate-argument input
would use slots and would not need property-read primitives.

### Expected generated Kotlin

The JNI implementation's optional foreign writer renders the retained public
plans as Kotlin (`Bindings.kt`):

```kotlin
package example

data class Stamp(val secs: Long, val nanos: Long)

object Bindings {
    @JvmStatic
    external fun sum(stamp: Stamp): Long
}
```

After the application loads the native library, Kotlin can call
`Bindings.sum(Stamp(12, 34))` and receive `46L`. Native-library loading belongs to
the application/test harness in this example. The Kotlin compiler supplies the
data class's JVM getters `getSecs(): long` and `getNanos(): long`.

The adapter's retained class metadata determines the getter names and JVM
descriptors. The Kotlin writer and primitive renderer use the same selected
metadata, so a naming override must affect both consistently.

### The getter primitive specifications

The adapter describes each getter as a local operation. It does not build the
record converter. For `getSecs`, the stored specification is:

```rust
let read_secs = PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![
            OperandSpec {
                ty: OperationType::Carrier(jni_environment),
                access: Access::Exclusive,
            },
            OperandSpec {
                ty: OperationType::Carrier(stamp_object),
                access: Access::Shared,
            },
        ],
        results: vec![OperationType::Carrier(jni_long)],
    },
    failure: PrimitiveFailure::Fallible {
        error: OperationType::Carrier(jni_error),
        category: FailureCategory::Runtime,
    },
    validity: ValidityContract {
        results: vec![ResultValidity::Independent],
    },
    resources: ResourceContract::none(),
    dependencies: vec![],
    implementation: JniOperation::CallLongGetter {
        name: "getSecs".into(),
        descriptor: "()J".into(),
    },
};
```

`jni_environment` describes the Rust `JNIEnv` runtime carrier, used through a
mutable borrow for this operation. `stamp_object` describes the input object
reference. `jni_long` is the JNI signed 64-bit integer carrier; `jni_error`
is the Rust `jni::errors::Error` type. `()J` is the JVM method descriptor for a
method taking no arguments and returning a signed 64-bit integer. These are
runtime/representation facts supplied by the JNI adapter.

The second specification changes the getter name to `getNanos`. Both successful
integer results are independent of the object after access. No reference or
resource ownership escapes the read. The getter operation calls the existing
`jni` crate directly, so its generated-helper dependency list is empty. The
binding's build requirements still include that crate.

`JniOperation::CallLongGetter` is the rendering payload. The JNI implementation
provides a renderer that takes that payload and the operand expressions assigned
by the common writer. With environment operand `env`, object operand `arg0`,
and the metadata above, **this primitive alone** renders:

```rust
env.call_method(&arg0, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

The expression returns `Result<jlong, jni::errors::Error>`. The `.and_then`
extracts a typed integer from this one getter result. It performs no recursive
source conversion and makes no return from the enclosing native function.
`PrimitiveSpec` stores the getter description, not a finished `match`, converter
body or native wrapper. The common writer calls the JNI operation renderer only
while rendering an application in a completed registry plan.

### A separate error-reporting primitive

For this illustration the selected boundary policy preserves a pending JVM
exception. If no exception is pending, the policy requests a
`RuntimeException`. Returning zero is only the required native return value
while an exception is pending; Kotlin observes the exception, not a successful
zero result. This is one explicit policy example, not a change to existing
consumer error conventions.

The JNI adapter can provide this reusable runtime helper (`jni_support.rs`):

```rust
use jni::{errors::Error, JNIEnv};

pub fn report_jni_error(env: &mut JNIEnv<'_>, error: Error)
    -> jni::errors::Result<()>
{
    if env.exception_check()? {
        Ok(())
    } else {
        env.throw_new("java/lang/RuntimeException", error.to_string())
    }
}
```

This helper implements one error-reporting operation. Its primitive specification
has an exclusive environment operand, an owned error operand, no success value,
a possible runtime error and a dependency on the helper artifact. The JNI
payload selects a call to this helper. The helper's local `?` propagates its own
failure to its caller; it never returns from the generated native wrapper.

The boundary policy also specifies what happens if reporting itself fails:
abort the process in this example. The registry therefore emits the failure
check and terminal action. The registry never recursively attempts to report
that new error with the same failed operation. A different configured policy
needs its corresponding explicit plan.

### Expected generated Rust/JNI

The common writer combines the getter expressions supplied by the JNI renderer
with the registry's control flow (`kotlin.rs`):

```rust
use crate::{jni_support::report_jni_error, source};
use jni::{objects::{JClass, JObject}, sys::jlong, JNIEnv};

#[no_mangle]
pub extern "system" fn Java_example_Bindings_sum(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    arg0: JObject<'_>,
) -> jlong {
    let v0 = match env.call_method(&arg0, "getSecs", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v1 = match env.call_method(&arg0, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

The JNI adapter supplies the symbol/calling convention, environment and class
parameter conventions, carrier types, getter operations and error policy. The
registry supplies each `match`, the placement of the reporting operation, its
failure branch, the terminal return, the source construction and the source
call. If `getSecs` fails, `getNanos` and `stamp_sum` do not run. If `getNanos`
fails, the source `Stamp` is never constructed.

The source `i64` and JNI `jlong` use the same Rust value representation here.
A target scalar rule declares that relationship, and the registry resolves the
identity conversions. More complicated child types would insert their own
registry-planned conversions between each getter and `Stamp` construction.

## From description to generated code

The operation payload is only one field of `PrimitiveSpec`. The other fields
let the registry validate where and how that operation can be used. These
concrete contributions stay separate throughout planning and writing:

| Contribution | C aggregate | Kotlin/JNI object | Component responsible |
| --- | --- | --- | --- |
| Source facts | Two `i64` fields and `stamp_sum(Stamp) -> i64` | Same source facts | Flat |
| Requested public API | `StampC`, `stamp_sum_c` | `example.Stamp`, `Bindings.sum` | Language frontend records user choices. |
| Target representation | `repr(C)` struct with members | JVM object, getters and JNI integer carriers | Target adapter describes it from policy and direct child descriptors. |
| `PrimitiveSpec.implementation` | Common `ReadMember` plus member identity | `CallLongGetter` plus getter metadata | Adapter selects payload; registry retains it. |
| One primitive's rendered operation | `arg0.secs` | `env.call_method(...).and_then(...)` | Common Rust operation renderer for C; JNI operation renderer for the getter. |
| Primitive application and result use | `let v0 = ...` | `let v0 = match ...` with error path | Registry plans instructions; common writer renders them. |
| Source construction and call | `source::Stamp { ... }`, then `stamp_sum` | Same source instructions | Registry plans; common Rust writer renders. |
| Error-reporting operation | Not needed by these field reads | Runtime helper using `exception_check` and `throw_new` | JNI supplies operation; registry places it and handles its failure. |
| Public foreign source | Header derived from Rust | Kotlin classes and native declaration | `cbindgen` for C; JNI's Kotlin writer for Kotlin. |

For the input record, the registry asks the selected relation for its fields,
resolves the child conversions, and asks the target for a representation using
those child descriptions. The target returns the member/getter mappings and
primitive specifications. The registry registers their definitions, creates
applications with concrete operand identities, composes the wrapper and freezes
the result. Writers then render that result without discovering new conversions.

Adding a third supported field makes the registry visit another source child
and apply the same composition algorithm. The target describes one more member
or getter through its existing local representation interface. The target does
not need another handwritten record converter or wrapper-assembly algorithm.
