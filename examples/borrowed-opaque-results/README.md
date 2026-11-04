# Explicit borrowed opaque outputs

Borrowed opaque values no longer silently become owning copies. C can expose a
shared source reference as a `const T *` loan. JNI owning handle classes require
an explicit copy projection or an expansion into fields.

```rust,ignore
// Ordinary JNI reference returns: authorize an independent owning copy.
jni.expand(expand_return!(&Packet).field_self_ref());
// A local override can expose data instead of copying the object.
fun!(packet_ref).expand_return(
    expand_return!(Packet).field(fun!(packet_len)));
// Explicit callback-argument copies belong to the callback author.
callback!(impl Fn(&Packet)).expand_param(
    0, expand_return!(Packet).field_self_ref());
```

`field_self_ref()` and `Projection::borrowed()` authorize owning copies of opaque
leaves recursively through borrowed records and collections. Owned getter results
still move. Generator-wide defaults and local replacements use the same API.
Exact reference-type defaults preserve existing automatic callback cleanup;
explicit callback-argument and whole-value defaults retain their existing
ownership-transfer rules.

C callback loans last only for the invocation. The source must remain alive and
at a stable address; the callback must not free, consume or mutate the loan.
Retaining a C loan requires a separately established source lifetime. An explicit
copy transfers ownership and must be destroyed by the callback author. JNI copies
are independent handles, never wrappers that own borrowed storage. No runtime
lifetime tracking or extra per-call locks were added.

C callback ABI names now incorporate the resolved wire layout and ownership,
preventing borrowed, copied and expanded signatures from sharing incompatible
typedefs. Regenerate headers and native clients together. Two getters with the
same wire layout can share a callback typedef while keeping separate conversions.

## Measured copy and allocation budgets

These are allocation/count comparisons of two configured paths in this change,
not before/after latency benchmarks. Each packet contains 65,536 bytes. Source
construction and callback registration are outside the C allocation counter.

| Actual generated boundary | Deliveries | Payload clones | Copied payload bytes | Native allocations |
| --- | ---: | ---: | ---: | ---: |
| C shared callback pointer | 1,000 | 0 | 0 | 0 |
| C explicit owning callback copy | 1,000 | 1,000 | 65,536,000 | 2,000 |
| JNI explicit owning callback copy | 32 | 32 | 2,097,152 | 32 large allocations |
| JNI local scalar callback expansion | 32 | 0 | 0 | 0 large allocations |

The C counter records all heap allocations during delivery: each copy allocates
its vector storage and a boxed object descriptor. JNI uses the existing native
buffer allocation probe, which counts allocations of at least 65,536 bytes; it
excludes descriptor boxes and JVM allocations. The two allocation columns must
therefore not be compared across languages. No latency speedup is claimed.

Reproduce the C budget with:

```sh
cargo test -p perftest-c borrowed_callbacks_allocate_only_when_copying_is_requested -- --nocapture
```

The JNI assertions run in `ExplicitBorrowCopies.kt` through
`SANITIZE=1 python3 examples/check.py` with Kotlin and Java installed. They also
cover independent copies after source destruction, optional references, native
workers, local overrides, non-Clone field expansion, and enum/handle rows in a
borrowed bulk collection. The C smoke test checks pointer identity, destruction,
non-Clone loans and distinct getter conversions sharing the same ABI.
