# Mutable borrowed C bytes

Rust `&mut [u8]` returns cross C as `buffer_view_mut { uint8_t *data; size_t len; }`.
The pointer addresses the original slice. C writes mutate that same storage;
there is no staging buffer, copy-back or destructor for a view. Optional returns
retain presence flags, distinguishing absent and present empty slices. Result
failures leave success output storage untouched.

The caller keeps the source alive and preserves exclusive access until the last
use of the mutable view. Do not access or move the owner, consume or destroy it,
or use overlapping views during that borrow. End the borrow before another
function uses the owner. The view is never an owning buffer. Loans from inputs
reconstructed in temporary Rust storage remain generation errors. These are
caller contracts and generation checks; no runtime guard was added.

Byte-vector inline owners automatically expose `*_view_mut(owner)`. Existing
`String` accessors remain read-only, preserving UTF-8 validity. The generated
mutable byte getter can also accompany its returned owner:

```rust,ignore
fun!(make_bytes).expand_return(
    expand_return!(Vec<u8>)
        .field_self_into()
        .field(Cbindgen::owned_view_mut("bytes")))
```

This requires an inline owner declared as `bytes`. The mutable getter is the
only getter in the expansion. Its pointer addresses heap bytes, which remain in
place while the inline Vec descriptor is delivered. The C caller ends the loan
before using or dropping the returned owner.

## Allocation measurements

`alias_allocations::mutable_byte_loans_and_vec_accessors_do_not_allocate_or_copy`
counts allocator calls at the actual generated C ABI, on the calling test thread.
Tests use buffer lengths 0, 1 and 65,536. Source construction is outside the
acquisition counters; sources reserve an additional 37 bytes of capacity.

| Operation | Calls per size | Allocations | Reallocations |
| --- | ---: | ---: | ---: |
| Direct mutable slice loan and in-place byte write | 1,000 | 0 | 0 |
| Inline Vec mutable accessor and in-place byte write | 1,000 | 0 | 0 |
| Construct Vec, return owner + mutable view, mutate and drop | 1 | 1 | 0 |

The last row includes its source Vec allocation, even at length zero because the
fixture reserves capacity. It has no separate descriptor box or view allocation.
Address/capacity checks and a non-Clone source owner verify that no replacement
buffer is introduced. Exactly one destruction occurs when the original source
owner is explicitly dropped. These are allocation budgets and identity checks;
no latency or throughput speedup is claimed, since the direct mutable slice
return was previously unsupported.

Reproduce with:

```sh
cargo test -p perftest-c mutable_byte_loans_and_vec_accessors_do_not_allocate_or_copy -- --nocapture
SANITIZE=1 python3 examples/check.py
```

The second command requires C/C++, Java and Kotlin tools. Its C caller exercises
empty and nonempty slices, nested options, successful/failed Results, invalid
output storage, generated Vec accessors and owner-plus-view expansions under
ASan/UBSan. C compilation must reject passing a mutable view to the owning buffer
destructor. The Kotlin/JNI suite runs as a regression check; mutable JNI outputs
and mutable callback arguments are not introduced by this C feature.
