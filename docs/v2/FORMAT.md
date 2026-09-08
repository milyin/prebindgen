<!-- spec: {"kind": "format"} -->

# Structure contract

[Project contents](README.md)

The manifest describes an ordered list of pipeline stages and an ordered list
of example paths. Each cell is one **applicable** `(example, stage)` pair. A cell
can declare one variant page for each language. Missing pairs have no files,
links, or placeholder records. The manifest records applicability, not generator
support.

A stage also owns ordered **detailed contracts**: the shared API, algorithms,
constraints and acceptance requirements that apply across examples. These are
part of the specification. Example cells apply them to concrete inputs. The
manifest's `contracts` list assigns each contract to one owning stage; other
stages can link to its sections when needed.

## Canonical paths

- `stages/<stage>.md`: stage input, operation/owner, output/failure contract and
  links to all declared cells at that stage.
- `stages/<stage>/<contract>.md`: full shared contract and backlink to its stage.
- `examples/<example>/README.md`: TOC of that example's cells, ordered by stage.
- `examples/<example>/<stage>.md`: common contract and links to language variants.
- `examples/<example>/<stage>.<language>.md`: one language-specific contract.
- `README.md`: stage and example TOCs.
- `source.md`: shared fixture; `FORMAT.md`: this format contract.

Every Markdown file begins with exactly one `<!-- spec: {...} -->` JSON metadata
line. The `kind` is `root`, `format`, `fixture`, `stage`, `example`, `cell`, or
`variant`, or `contract`. A contract carries its `contract` ID and owning `stage`.
A stage carries `stage`; an example carries `example`; a cell carries
both; a variant also carries `language`. Identity must match the canonical path.
The validator rejects unknown metadata fields and unlisted Markdown files.

## Required content and links

Each common cell and language variant has exactly one Input, Owner, Result and
Checks section. Those sections must contain text. They state the supplied data,
responsible component, exact resulting state, and observable obligations or
failure behavior. Language variants specialize the common contract.

Every cell links directly to its pipeline chapter and example TOC. Every variant
links directly to its chapter, common cell and example TOC. Each stage chapter
links to every declared common cell and variant for that stage; each example TOC
links to every declared common cell and variant for that example. Both indexes
use manifest order. A common cell lists exactly its declared language variants.

Each stage links to all of its detailed contracts in manifest order before
listing concrete examples. Each detailed contract links back to that stage and
the root TOC. Contract titles and metadata match the manifest. Unlisted or
missing contracts are errors, just like unlisted or missing cells.

Previous/next chapter links follow all stages. Previous/next links within an
example follow that example's declared cells only. The root lists every stage
and example in order. Cross-example links can document dependencies, but do not
replace the two primary indexes or the required backlinks.

A stage's `language_dependent` flag requires both declared languages for each
applicable cell at that stage. Source/Flat inspection and retention currently
share contracts across languages. Requests, values, boundaries and emission
have C and Kotlin variants. A future stage that needs a finer split should be
split or have an explicit schema revision, rather than weakening coverage checks.

The validator checks local links and anchors, balanced code fences, page
identities, required sections, ordered indexes, backlinks, neighbors, duplicate
IDs/cells, complete declared language variants, and unlisted files. It does not
infer semantics from missing pairs or validate Rust/JNI behavior from prose.
Links to supporting material do not replace the required sections in a case
page.

## Adding a path or stage

Add its ID/title to the manifest, then declare only applicable cells. Write each
cell and, at language-dependent stages, both language variants. Update chapter
indexes, example TOCs, root navigation and neighbors together. Remove a kind
from `deferred_examples` when its first defined path is added; keep any remaining
limitations explicit in that path's introduction.

For a vector field, for example, specify its item type, order, length handling,
ownership, allocation/failure behavior and parent-record dependency at the
relevant stages. Reusing the word “vector” without those details is not a
completed specification. Tests of the validator reject broken structure; review
and eventual generator/runtime tests establish semantic correctness.
