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
For example, the record has no native-function-boundary cell because the record
does not itself export a callable function. There is no placeholder page for
that combination. An unsupported feature, in contrast, is explained on an
existing page rather than represented by a missing file.

## Canonical paths

| Page | Path |
| --- | --- |
| Root | `README.md` |
| Format contract | `FORMAT.md` |
| The specification's source crate | `source.md` |
| Stage chapter | `stages/<stage>.md` |
| Implementation plan | `implementation.md` |
| Element path TOC | `examples/<element>/README.md` |
| Cell | `examples/<element>/<stage>.md` |
| Variant | `examples/<element>/<stage>.<language>.md` |

The paths in the table are relative to `docs/v2/`. `<stage>` is the numbered
stage name, such as `04-values`; `<element>` is an example id, such as `fn`;
and `<language>` is `c` or `jni`. TOC means table of contents.

Every Markdown file starts with exactly one `<!-- spec: {...} -->` metadata line.
`kind` is `root`, `format`, `fixture`, `stage`, `implementation`, `example`,
`cell` or `variant`. A stage page carries `stage`; an element TOC carries
`example`; a cell carries both; a variant adds `language`. The identity has to
match the canonical path, and every Markdown file under `docs/v2/` has to be one
the manifest accounts for. This comment is hidden in the rendered page but lets
the validator identify the page without guessing from its title. For example,
a function's C value-planning page declares its kind as `variant`, its example
as `fn`, its stage as `04-values`, and its language as `c`.

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

The stage slugs are `source`, `flat`, `requests`, `values`, `boundary`, `retain`
and `emit`; the language ids are `c` and `jni`. So `fn` is the function path,
`fn_values` is the function at value planning, and `fn_values_c` is its C
variant. A sub-variant extends the element id and keeps the same suffixes:
`fn_callback`, `fn_callback_values_jni`. These ids are reserved for future paths;
the validator's current element-index check must be extended before an example
id containing underscores can be added. The existing examples use `fn` and
`struct`.

An element id names a structural kind — function, record, enum, constant, and
the variants of those — never the names the source crate happens to use. `Stamp`
and `stamp_sum` appear in prose and code, never in an id, so a path stays
recognizable when the declarations it works from change.

Definitions live in one block at the foot of the page:

```markdown
Planned as the record input described in [the record's C value plan][struct_values_c].

[struct_values_c]: ../struct/04-values.c.md
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

A stage marked `language_dependent` requires both languages for each of its
applicable cells: capture, source-model inspection and retention are shared
across targets, while requests, value planning, the boundary and emission are
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
on the enclosing record — at each stage that decides one of them. Reusing the
word "sequence" without those decisions is not a specified path.

The validator checks page identities, required sections, link ids and their
targets, ordered indexes, backlinks, neighbors, resolvable local links and
anchors, balanced code fences, and that no Markdown file is unlisted or missing.
It does not infer meaning from an absent pair and cannot judge whether the
described conversions are correct; review, and eventually the generator's own
tests, do that.
