<!-- spec: {"kind": "format"} -->

[Project contents](README.md)

# The format contract

This page is for contributors editing or extending the guide. Its rules keep the
stage chapters and worked examples connected, and let a script detect missing
pages or broken navigation. You do not need to learn these rules to use the
generator.

The guide organizes information in two directions. A **stage** is one step of
generation, such as planning conversions. An **element path** follows one kind
of Rust item through those steps, such as the function example. A **cell** is
the page where the two meet: the function at conversion planning, for example.
When C and JNI behave differently at that step, the cell links to a **variant**
page for each language.

[manifest.json](manifest.json) lists those stages, examples and combinations.
It records which combinations make sense, not which features are implemented.
For example, the struct has no wrapper-boundary cell because the struct
does not itself export a callable function. There is no placeholder page for
that combination. An unsupported feature, in contrast, is explained on an
existing page rather than represented by a missing file.

## Canonical paths

| Page | Path |
| --- | --- |
| Root | `README.md` |
| Format contract | `FORMAT.md` |
| Vocabulary introduction | `concepts.md` |
| The specification's source crate | `source.md` |
| Stage chapter | `stages/<stage>.md` |
| Implementation plan | `implementation.md` |
| Extension contracts | `extensions.md` |
| Element path TOC | `examples/<element>/README.md` |
| Cell | `examples/<element>/<stage>.md` |
| Variant | `examples/<element>/<stage>.<language>.md` |

The paths in the table are relative to `docs/v2/`. `<stage>` is the numbered
stage name, such as `04-select`; `<element>` is an example id, such as `fn`;
and `<language>` is `c` or `jni`. TOC means table of contents.

A stage chapter describes what its cells demonstrate. A contract the engine
does not implement, and no cell shows, goes on the extension contracts page
under a section that names the stage it extends; the chapter links there at
the point where the gap is, instead of carrying the design itself.

Every Markdown file starts with exactly one `<!-- spec: {...} -->` metadata line.
`kind` is `root`, `format`, `concepts`, `fixture`, `stage`, `implementation`,
`extensions`, `example`, `cell` or `variant`. A stage page carries `stage`; an element TOC carries
`example`; a cell carries both; a variant adds `language`. The identity has to
match the canonical path, and every Markdown file under `docs/v2/` has to be one
the manifest accounts for. This comment is hidden in the rendered page but lets
the validator identify the page without guessing from its title. For example,
a function's C relation-selection page declares its kind as `variant`, its example
as `fn`, its stage as `04-select`, and its language as `c`.

## Link ids

Links into the appendix use Markdown reference links: visible words followed
by a label, with a definition at the bottom mapping that label to a relative
file path. The complete example below shows the syntax. These ids follow a
shared naming scheme:

```text
<element>[_<subvariant>]        the element path's TOC
<element>..._<stage-slug>       that element at that stage
<element>..._<stage-slug>_<lang>  the language variant of that cell
```

The stage slugs are `source`, `flat`, `requests`, `select`, `represent`, `boundary`, `retain`
and `emit`; the language ids are `c` and `jni`. So `fn` is the function path,
`fn_select` is the function at relation selection, and `fn_select_c` is its C
variant. A sub-variant extends the element id and keeps the same suffixes:
`fn_callback`, `fn_callback_select_jni`. These ids are reserved for future paths;
the validator's current element-index check must be extended before an example
id containing underscores can be added. The existing examples use `fn` and
`struct`.

An element id names a structural kind — function, struct, enum, constant, and
the variants of those — never the names the source crate happens to use. `Stamp`
and `stamp_sum` appear in prose and code, never in an id, so a path stays
recognizable when the declarations it works from change.

Definitions live in one block at the foot of the page:

```markdown
Planned as the struct input described in [the struct's C representation][struct_represent_c].

[struct_represent_c]: ../struct/05-represent.c.md
```

Every definition has to name a declared id and point at that id's canonical path,
relative to the page. Every used label has to be defined on its page, with no
duplicates and nothing defined but unused. Links into `examples/` are always
reference-style; inline links are for everything else — chapters, the source
crate, external URLs.

## Required content

A cell or variant must show the data or code being discussed. Here, an artifact
means a concrete result of a step: captured text, a request, a plan, or generated
code. Showing that result lets readers compare successive steps instead of
having to reconstruct an invisible example from prose.

Each has exactly one **Input**, **Result** and **Checks** section, all non-empty,
and an `Owner: …` line above the title naming the component responsible for the step. Input shows
what that component receives; Result shows what it produces. Start those
sections with code when the data has a useful written form, then explain how to
read it. Define unfamiliar names, explain why the transformation is needed, and
connect the result to the next step. Code alone is not an explanation.

Checks describe observable requirements and failure behavior: for example, that
both fields are read in the intended order, or that a failed getter prevents the
source function call. A language variant concentrates on what that language adds
to the common cell, but should explain enough context to be read independently.

Navigation lives above the title, between the metadata line and the heading: a
chapter's line carries the link to the contents and its previous/next chapters, a
cell's carries its stage chapter, its element TOC, the source crate and its
previous/next cells along that element, and a variant's carries its chapter, its
common cell and its element TOC. Nothing navigational sits at the foot of a page
except the link definitions.

Every stage chapter lists all of its declared cells and variants under the exact
heading `## Elements at this stage`. A language-dependent cell lists its variants
under `## Language variants`. Every element TOC lists all of that element's
pages. These lists follow manifest order. Cross-element
links document dependencies; they never replace an index or a backlink.

## Vocabulary

The nouns that name what flows between stages — source item, declaration,
conversion rule, root, site, part, conversion, crossing, relation,
representation, carrier, primitive, node, artifact, wrapper, outcome,
capability — are listed
in the manifest's `vocabulary` with the page and heading that define each. A
term is **defined**, set in bold, under that heading and nowhere else;
[concepts.md](concepts.md) introduces every term in a `### Term` entry that
itself links to the definition; and on every other page the first mention of
the term in prose links to the definition. A mention inside a code span, a code
block, a heading, a link target, or a chapter, element or language title quoted
in a link of either syntax is not a mention; Markdown wrapped across lines is
read as one text. Any other reference-style link is prose: a first mention
inside one is unlinked to its definition, so write the term as a plain linked
word and give the cross-reference a link of its own with its own text. A term used in another sense — "wrapper" for `Option<T>`,
"site" for a use site — is reworded rather than linked: the rule exists so a
reader meets each word once with its meaning, and a link to the wrong meaning
defeats it.

Adding a term: add its entry to the manifest with a `match` pattern (the word and
its plural), define it in bold on one page under a heading the entry names, add
its `### Term` entry to `concepts.md`, then run the validator and link what it
reports. The validator reads Markdown as this document writes it — fenced code, `**`
or `__` for strong emphasis, inline and reference-style links, backslash
escapes, code spans of any delimiter length, and the one HTML construct in
use, a table whose `<code>` cells are code and whose other cells are prose —
and not as a CommonMark implementation: an indented code block, any other
HTML, or a link reference defined by title are not recognized, and the
document does not use them. A new construct the document needs is added to
the validator with a test, not worked around.

A term the rule fits badly — one the chapters must also use in its
ordinary English sense — is entered with `"link_first_mention": false`, which
keeps the single-definition rule and drops the linking one: `declaration`,
`root`, `part` and `capability` today, each also an ordinary English word the
chapters need.

A stage marked `language_dependent` requires both languages for each of its
applicable cells: capture, source-model inspection and retention are shared
across targets, while requests, selection, representation, the boundary and emission are
not. A future stage that needs a finer split should be split, rather than having
its coverage check weakened.

## Adding a path or a stage

1. Add the id and title to the manifest, and declare only the applicable cells.
2. Write each cell, plus both language variants at language-dependent stages.
3. Update the stage indexes, the element TOC, the root's appendix tree and the
   previous/next chains together.
4. Move the id out of `deferred_examples` and state in the new path's
   introduction whatever remains unspecified about it.

A path is finished when its cells say what actually happens, not when they name
the feature. For a sequence field, that means the item type, element order,
length handling, ownership, allocation and failure behavior, and the dependency
on the enclosing struct — at each stage that decides one of them. Reusing the
word "sequence" without those decisions is not a specified path.

The validator checks page identities, required sections, link ids and their
targets, ordered indexes, backlinks, neighbors, resolvable local links and
anchors, balanced code fences, and that no Markdown file is unlisted or missing.
It does not infer meaning from an absent pair and cannot judge whether the
described conversions are correct; review, and eventually the generator's own
tests, do that.
