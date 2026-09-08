<!-- spec: {"kind": "format"} -->

# The format contract

[Project contents](README.md)

The document has two axes. [manifest.json](manifest.json) lists the **stages** of
the pipeline and the **element paths** of the appendix; one applicable
`(element, stage)` pair is a **cell**, and a cell at a language-dependent stage
also has one **variant** page per language. Inapplicable pairs have no file, no
link and no placeholder. The manifest records applicability — whether the pair is
meaningful — not whether the generator supports it.

## Canonical paths

| Page | Path |
| --- | --- |
| Root | `README.md` |
| Format contract | `FORMAT.md` |
| Source fixture | `source.md` |
| Stage chapter | `stages/<stage>.md` |
| Implementation plan | `implementation.md` |
| Element path TOC | `examples/<element>/README.md` |
| Cell | `examples/<element>/<stage>.md` |
| Variant | `examples/<element>/<stage>.<language>.md` |

Every Markdown file starts with exactly one `<!-- spec: {...} -->` metadata line.
`kind` is `root`, `format`, `fixture`, `stage`, `implementation`, `example`,
`cell` or `variant`. A stage page carries `stage`; an element TOC carries
`example`; a cell carries both; a variant adds `language`. The identity has to
match the canonical path, and every Markdown file under `docs/v2/` has to be one
the manifest accounts for.

## Link ids

Links into the appendix use reference-style links whose label is the element id:

```text
<element>[_<subvariant>]        the element path's TOC
<element>..._<stage-slug>       that element at that stage
<element>..._<stage-slug>_<lang>  the language variant of that cell
```

The stage slugs are `source`, `flat`, `requests`, `values`, `boundary`, `retain`
and `emit`; the language ids are `c` and `jni`. So `fn` is the function path,
`fn_values` is the function at value planning, and `fn_values_c` is its C
variant. A sub-variant extends the element id and keeps the same suffixes:
`fn_callback`, `fn_callback_values_jni`.

An element id names a structural kind — function, record, enum, constant, and
the variants of those — never the fixture's own names. `Stamp` and `stamp_sum`
appear in prose and code, never in an id, so a path stays recognizable when its
fixture changes.

Definitions live in one block at the foot of the page:

```markdown
Planned as the record input described in [the record's C value plan][struct_values_c].

[struct_values_c]: ../struct/04-values.c.md
```

Every definition has to name a declared id and point at that id's canonical path,
relative to the page. Every used label has to be defined on its page, with no
duplicates and nothing defined but unused. Links into `examples/` are always
reference-style; inline links are for everything else — chapters, the fixture,
external URLs.

## Required content

A cell and a variant each have exactly one **Input**, **Owner**, **Result**,
**Checks** and **Representation** section, all non-empty. Input is what the stage
receives for this element, Owner is the component that decides, Result is the
exact resulting state, Checks are the observable obligations and failure
behavior, and Representation shows the concrete artifact at that stage: the
captured item, the views, the request, the plan, the boundary, the retained entry
or the generated code. A variant specializes its common cell rather than
repeating it.

Every cell links to its stage chapter and its element TOC; every variant links to
its chapter, its common cell and its element TOC. Every stage chapter lists all
of its declared cells and variants, and every element TOC lists all of that
element's, both in manifest order. Chapters carry previous/next links across the
stage order; cells carry them across that element's declared cells. Cross-element
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
