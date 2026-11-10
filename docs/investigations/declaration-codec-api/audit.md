# Audit of the current declaration codec

Baseline: #898 at `a2a9c8849c5e997654dc18a51d8d3dcfabd49e02`.
This appendix records source observations, not implemented changes. Start with
[the proposal](README.md) for the concepts and [API details](api.md) for the
replacement contracts.

## Scope and method counts

The `Codec` trait contains 55 methods: 11 required and 44 with defaults. Its
seven associated types are `Schema`, `Wire`, `Context`, `Extra`, `Chain`,
`Branch`, and `Niche`. The proposal keeps the first six and moves niche details
inside complete optional encodings.

The 11 required methods are `group_metadata`, `optional_metadata`,
`alternatives_metadata`, `field_group`, `fields_group`, `selector`, `consumes`,
`input_leaf`, `optional_niche`, `retire_niches`, and `output_leaf`.

| Production codec implementation | Methods implemented |
| --- | ---: |
| [`CCodec`](../../../prebindgen-c/src/lib.rs) | 37 |
| [`EnumCodec`](../../../prebindgen-c/src/enums.rs) | 41 |
| [`JniCodec`](../../../prebindgen-jni/src/lib.rs) | 37 |
| [`DataCodec<'_>`](../../../prebindgen-jni/src/data.rs) | 44 |

These counts include substantive overrides and forwarding methods. The adapters
use one shared declaration compiler; the forwarding wrappers are not independent
runtime emitters. A smaller required-method count helps a third codec, while
removing the wrappers also reduces the burden of maintaining C and JNI.

A strict syntactic count finds 31 C-wrapper methods and 19 JNI-wrapper methods
whose bodies are just a call to the base codec (with an optional trailing
semicolon). This excludes nontrivial wrappers and calls to other helpers. The
migration must remove those 50 forwarding bodies, not merely rename them.

The method totals above can be reproduced from the repository root:

```sh
python3 - <<'PYTHON'
from pathlib import Path
import re
s = Path("prebindgen-declarations/src/lib.rs").read_text()
s = s.split("pub trait Codec {", 1)[1].split("\n}\n/// A flat model", 1)[0]
methods = list(re.finditer(r"^    fn (\w+)\b", s, re.M))
required = sum(re.search(r"[;{]", s[m.end():]).group() == ";" for m in methods)
print("Codec:", len(methods), "methods,", required, "required")
for file, name in [
    ("prebindgen-c/src/lib.rs", "CCodec"),
    ("prebindgen-c/src/enums.rs", "EnumCodec"),
    ("prebindgen-jni/src/lib.rs", "JniCodec"),
    ("prebindgen-jni/src/data.rs", "DataCodec<'_>"),
]:
    code = Path(file).read_text().split("impl Codec for " + name + " {", 1)[1]
    code = code.split("\n}\n", 1)[0]
    print(name, len(re.findall(r"^    fn \w+", code, re.M)))
PYTHON
```

The audit table below independently lists every method with its source line and
proposed operation; the CLI output is not a substitute for the semantic review.

## Source traces behind the recommendation

- [Leaf dispatch](../../../prebindgen-declarations/src/lib.rs#L1945) selects
  ordinary/grouped-row/bulk/grouped-bulk-row input entry points. The `bulk`,
  `bulk_input_row` and `grouped_input` state is private compiler context.
- [C row preparation](../../../prebindgen-c/src/sequence.rs#L28) changes inline
  owner acquisition and records completed moves per leaf. This is necessary
  failure cleanup/reporting, not redundant wrapper code to delete.
- [JNI bulk row preparation](../../../prebindgen-jni/src/bulk.rs#L324) keeps
  decoded native buffers and enums alive through grouped row construction.
- [Option composition](../../../prebindgen-declarations/src/optional.rs)
  already shares Rust control-flow/commit assembly. Its ten codec hooks are
  representation operations and niche bookkeeping, not ten independent
  language features.
- [C optional traversal](../../../prebindgen-c/src/group.rs#L108) and
  [JNI optional traversal](../../../prebindgen-jni/src/group.rs#L108) stop at
  physical carrier boundaries. Their child tree still exists for row/bulk
  decoding. A replacement must preserve that distinction.
- [Alternative input construction](../../../prebindgen-declarations/src/lib.rs#L2070)
  retires niches in every inactive arm, independently of optional composition.
  The proposed alternatives operation must absorb this use as well.
- [Source Result preparation](../../../prebindgen-declarations/src/lib.rs#L2131)
  asks separately for propagation and layout. The declaration-aware C/JNI
  codecs consult the same Flat error metadata for both halves.
- [Final input preparation](../../../prebindgen-declarations/src/lib.rs#L1715)
  may put acquisition and construction inside the fragment named `validation`.
  Changing codec authoring must preserve these phases, not silently reinterpret
  them as pure validation.
- [Record composition](../../../prebindgen-declarations/src/record.rs) is
  already shared and preserves field groups and source paths. Foreign-record
  metadata remains a separate optional feature from expanding a Rust record.

## Disposition of all 55 methods

“Required” describes the current trait. “Default” does not always mean a no-op:
several defaults reject a feature or delegate to another codec operation.
Methods without a substantive replacement retain their existing signatures.
Line links point to the audited source; no generator source changes in this PR.

| Current method | Current obligation | Proposed operation | Reason |
| --- | --- | --- | --- |
| [`validates_inputs`](../../../prebindgen-declarations/src/lib.rs#L663) | Default | `validates_inputs` | Retain the distinct operation and current default/behavior. |
| [`input_bulk_leaf`](../../../prebindgen-declarations/src/lib.rs#L674) | Default | `input_leaf` | Explicit representation and construction context. |
| [`input_bulk_row_leaf`](../../../prebindgen-declarations/src/lib.rs#L688) | Default | `input_leaf` | Explicit representation and construction context. |
| [`output_bulk_leaf`](../../../prebindgen-declarations/src/lib.rs#L701) | Default | `output_leaf` | Explicit representation and source access. |
| [`output_bulk_projected`](../../../prebindgen-declarations/src/lib.rs#L714) | Default | `output_projected` | Explicit projection remains independent; representation is request context. |
| [`bulk_input`](../../../prebindgen-declarations/src/lib.rs#L736) | Default | `bulk_input` | Retain the distinct operation and current default/behavior. |
| [`bulk_output`](../../../prebindgen-declarations/src/lib.rs#L748) | Default | `bulk_output` | Retain the distinct operation and current default/behavior. |
| [`sequence_input_bulk`](../../../prebindgen-declarations/src/lib.rs#L760) | Default | `sequence_input` | One transport request with explicit representation. |
| [`sequence_output_bulk`](../../../prebindgen-declarations/src/lib.rs#L773) | Default | `sequence_output` | One request with explicit representation and delivery. |
| [`callback_input`](../../../prebindgen-declarations/src/lib.rs#L786) | Default | `callback_input` | Retain the distinct operation and current default/behavior. |
| [`expand_input`](../../../prebindgen-declarations/src/lib.rs#L799) | Default | `expand_input` | Retain the distinct operation and current default/behavior. |
| [`expand_output`](../../../prebindgen-declarations/src/lib.rs#L809) | Default | `expand_output` | Retain the distinct operation and current default/behavior. |
| [`default_output_sequence_is_bulk`](../../../prebindgen-declarations/src/lib.rs#L820) | Default | `default_output_sequence_is_bulk` | Retain the distinct operation and current default/behavior. |
| [`input_collection_is_leaf`](../../../prebindgen-declarations/src/lib.rs#L830) | Default | `input_collection_is_leaf` | Retain the distinct operation and current default/behavior. |
| [`output_collection_is_leaf`](../../../prebindgen-declarations/src/lib.rs#L840) | Default | `output_collection_is_leaf` | Retain the distinct operation and current default/behavior. |
| [`input_record_metadata`](../../../prebindgen-declarations/src/lib.rs#L849) | Default | `input_record_metadata` | Retain the distinct operation and current default/behavior. |
| [`output_record_metadata`](../../../prebindgen-declarations/src/lib.rs#L861) | Default | `output_record_metadata` | Retain the distinct operation and current default/behavior. |
| [`group_metadata`](../../../prebindgen-declarations/src/lib.rs#L882) | Required | `group` | One kind selects values, fields, a field, or a named view. |
| [`named_output_group`](../../../prebindgen-declarations/src/lib.rs#L884) | Default | `group` | One kind selects values, fields, a field, or a named view. |
| [`optional_metadata`](../../../prebindgen-declarations/src/lib.rs#L895) | Required | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`alternatives_metadata`](../../../prebindgen-declarations/src/lib.rs#L905) | Required | `alternatives` | Receives the prefixed selector and retires inactive arm offers. |
| [`field_group`](../../../prebindgen-declarations/src/lib.rs#L907) | Required | `group` | One kind selects values, fields, a field, or a named view. |
| [`fields_group`](../../../prebindgen-declarations/src/lib.rs#L910) | Required | `group` | One kind selects values, fields, a field, or a named view. |
| [`selector`](../../../prebindgen-declarations/src/lib.rs#L927) | Required | `selector` | Retain alternative-tag construction before prefixing; optional flag construction moves into optional. |
| [`consumes`](../../../prebindgen-declarations/src/lib.rs#L938) | Required | `output_consumes` | Keep early output move analysis; clarify the name. |
| [`input_leaf`](../../../prebindgen-declarations/src/lib.rs#L962) | Required | `input_leaf` | Explicit representation and construction context. |
| [`input_borrowed_sequence`](../../../prebindgen-declarations/src/lib.rs#L976) | Default | `input_borrowed_sequence` | Retain the distinct operation and current default/behavior. |
| [`input_row_leaf`](../../../prebindgen-declarations/src/lib.rs#L991) | Default | `input_leaf` | Explicit representation and construction context. |
| [`output_projected`](../../../prebindgen-declarations/src/lib.rs#L1006) | Default | `output_projected` | Explicit projection remains independent; representation is request context. |
| [`sequence_input`](../../../prebindgen-declarations/src/lib.rs#L1043) | Default | `sequence_input` | One transport request with explicit representation. |
| [`sequence_output`](../../../prebindgen-declarations/src/lib.rs#L1061) | Default | `sequence_output` | One request with explicit representation and delivery. |
| [`result_error_value`](../../../prebindgen-declarations/src/lib.rs#L1078) | Default | `source_error` | Matching typed/mapped error layout and propagation together. |
| [`source_error_layout`](../../../prebindgen-declarations/src/lib.rs#L1100) | Default | `source_error` | Matching typed/mapped error layout and propagation together. |
| [`binding_failure_layout`](../../../prebindgen-declarations/src/lib.rs#L1125) | Default | `binding_failure_layout` | Retain the distinct operation and current default/behavior. |
| [`result_error`](../../../prebindgen-declarations/src/lib.rs#L1134) | Default | `mapped_error` | Retain the mapped-message convenience route used by the source_error default. |
| [`optional_niche`](../../../prebindgen-declarations/src/lib.rs#L1145) | Required | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`optional_presence`](../../../prebindgen-declarations/src/lib.rs#L1150) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`absent_output_values`](../../../prebindgen-declarations/src/lib.rs#L1158) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`retire_niches`](../../../prebindgen-declarations/src/lib.rs#L1167) | Required | `optional` / `alternatives` | Each encoding retires inactive offers in its own native scope. |
| [`validate_input`](../../../prebindgen-declarations/src/lib.rs#L1175) | Default | `validate_input` | Retain the distinct operation and current default/behavior. |
| [`validate_output`](../../../prebindgen-declarations/src/lib.rs#L1179) | Default | `validate_output` | Retain the distinct operation and current default/behavior. |
| [`prefix_layout`](../../../prebindgen-declarations/src/lib.rs#L1185) | Default | `prefix_layout` | Retain the distinct operation and current default/behavior. |
| [`niche`](../../../prebindgen-declarations/src/lib.rs#L1195) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`set_niche`](../../../prebindgen-declarations/src/lib.rs#L1202) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`is_present`](../../../prebindgen-declarations/src/lib.rs#L1207) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`remaining_niche`](../../../prebindgen-declarations/src/lib.rs#L1218) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`absent_value`](../../../prebindgen-declarations/src/lib.rs#L1222) | Default | `optional` | Complete encoding; wire primitives become optional utilities/inherent operations. |
| [`finish_input`](../../../prebindgen-declarations/src/lib.rs#L1235) | Default | `finish_input` | Retain the distinct operation and current default/behavior. |
| [`finish_output`](../../../prebindgen-declarations/src/lib.rs#L1242) | Default | `finish_output` | Retain the distinct operation and current default/behavior. |
| [`output_leaf`](../../../prebindgen-declarations/src/lib.rs#L1263) | Required | `output_leaf` | Explicit representation and source access. |
| [`output_return_reference`](../../../prebindgen-declarations/src/lib.rs#L1285) | Default | `output_return_reference` | Retain the distinct operation and current default/behavior. |
| [`output_borrowed_sequence`](../../../prebindgen-declarations/src/lib.rs#L1306) | Default | `output_borrowed_sequence` | Retain the distinct operation and current default/behavior. |
| [`output_owner`](../../../prebindgen-declarations/src/lib.rs#L1326) | Default | `output_owner` | Retain the distinct operation and current default/behavior. |
| [`validate_owner_getter`](../../../prebindgen-declarations/src/lib.rs#L1337) | Default | `validate_owner_getter` | Retain the distinct operation and current default/behavior. |
| [`validate_getter_output`](../../../prebindgen-declarations/src/lib.rs#L1348) | Default | `validate_getter_output` | Retain the distinct operation and current default/behavior. |

The mapping has 35 distinct target methods. The proposal additionally introduces
`supports_bulk` so the compiler can enforce default bulk rejection, for 36
methods in total. The intended required set is
`input_leaf`, `output_leaf`, `output_consumes`, and `group`. Optional feature
operations reject unsupported work by default; mapped source errors retain their
existing convenience fallback; specializations retain their
existing `None`/no-op behavior where appropriate. See the full defaults table
in [API details](api.md#complete-proposed-method-inventory).

## Additional callers and wrapper policy

The audit also follows hooks outside the shared recipe compiler:

- [`Class::mark`](../../../prebindgen-jni/src/data.rs#L466) retires field niche
  offers before annotating a foreign data-class record. That caller must migrate
  to reusable retirement/wire helpers; optional/alternative hooks alone do not
  cover it.
- [C error initialization and failure returns](../../../prebindgen-c/src/errors.rs#L300)
  use `CCodec.absent_value(..., false)` directly. They must use the same C wire
  encoding helper after the trait hook is removed.
- [JNI bulk input](../../../prebindgen-jni/src/bulk.rs#L147) uses
  `optional::absent(..., true)` directly for inactive native fields. It does not
  call the trait's hook. Removing input absent values from the *optional result*
  must not remove this behavior.

C's callback-result flag is read by `EnumCodec::input_leaf`, including when its
row-specific conversion calls that leaf before row postprocessing. Its other
structural/sequence/optional operations do not read that flag. JNI's
`DataCodec` normalizes data-class recipes, annotates foreign record groups,
selects declared enums/errors, and carries a borrowed callback-policy set. Both
wrappers fall back to raw conversion helpers; neither implements a separate
record-recursion algorithm. The new language codec must preserve explicit raw
and declared preparation policies, including missing-context diagnostics.

## Why several apparent simplifications are rejected

- **Only add defaults.** This removes compulsory stubs but leaves duplicate
  entry points, fragmented error/option contracts and production forwarding.
  Useful as an implementation step, insufficient as the final architecture.
- **A trait for every feature.** This introduces provider types, trait bounds and
  forwarding across combinations of optional features. It makes unsupported
  features compile-time bounds rather than the current useful occurrence-level
  diagnostics. No current requirement warrants that extra public machinery.
- **One universal `convert` enum with 55 cases.** This changes dispatch syntax
  without reducing the adapter's decisions and weakens typed input/output
  results. Operation-specific requests are useful only where they describe the
  same conversion under a different representation or lifetime scope.
- **Move all structural metadata into the shared core.** Flat deliberately
  retains adapter-owned `Chain`/`Branch` metadata; Kotlin class reconstruction
  and C representations differ. The proposal preserves that ownership.
- **Ignore row context.** This can remove prefix receipts, extend JNI references
  past their frame, or force copies of borrowed storage. Consolidation preserves
  context and generated fragments.
- **Derive consumption from the finished output.** The compiler needs ownership
  policy before it orders getter observations and moves. Keeping this operation
  is necessary even though it increases the minimal codec's method count.

## What this PR verifies

The audit enumerates the trait methods and checks that each has exactly one
mapping row (optional/alternative encodings share retirement responsibilities).
The 36-method total is the 35 distinct destinations plus `supports_bulk`, not an
estimate.
Source links, relative document links and the proposal's inventory are checked
before opening the PR. The C/JNI implementations, optional/record/sequence
compiler paths and production forwarding codecs were inspected.

The proposed signatures are design sketches. This PR does not compile a new
codec API, demonstrate that a replacement implementation passes runtime tests,
or claim a runtime performance improvement. Those are implementation acceptance
criteria in the main proposal. The architecture checkbox stays open.
