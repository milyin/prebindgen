#!/usr/bin/env python3
"""Check the pipeline/example/language document structure using only stdlib."""
import argparse
import json
import re
from pathlib import Path


class InvalidSpec(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise InvalidSpec(message)


def fields(value, expected, where):
    require(isinstance(value, dict) and set(value) == set(expected),
            f'{where}: expected fields {sorted(expected)}')


def text_id(value, where):
    require(isinstance(value, str) and re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', value),
            f'{where}: invalid ID {value!r}')


def nonempty(value, where):
    require(isinstance(value, str) and value.strip(), f'{where}: empty text')


def prose(text, where):
    """Remove fenced code so example strings are not interpreted as document links."""
    result, fence = [], None
    for line in text.splitlines():
        marker = re.match(r'^\s*(`{3,}|~{3,})', line)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence = run
            elif run[0] == fence[0] and len(run) >= len(fence):
                fence = None
            continue
        if fence is None:
            result.append(line)
    require(fence is None, f'{where}: unclosed code fence')
    return '\n'.join(result)


def anchors(text):
    result, counts = set(), {}
    for label in re.findall(r'^#{1,6}\s+(.+)$', text, re.M):
        base = re.sub(r'[^\w\- ]', '', label.strip().lower()).replace(' ', '-')
        count = counts.get(base, 0)
        result.add(base if count == 0 else f'{base}-{count}')
        counts[base] = count + 1
    return result


def section(text, heading, where):
    matches = list(re.finditer(r'^## ' + re.escape(heading) + r'\s*$', text, re.M))
    require(len(matches) == 1, f'{where}: expected one {heading!r} section')
    tail = text[matches[0].end():]
    end = re.search(r'^## ', tail, re.M)
    body = tail[:end.start()] if end else tail
    require(body.strip(), f'{where}: empty {heading!r} section')
    return body


def local_links(text, page):
    links = []
    for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', text):
        if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
            continue
        file, _, anchor = target.partition('#')
        dest = (page.parent / file).resolve() if file else page.resolve()
        links.append((dest, anchor))
    return links


def validate(root):
    root = root.resolve()
    try:
        manifest = json.loads((root / 'manifest.json').read_text())
    except (OSError, ValueError) as error:
        raise InvalidSpec(f'manifest.json: {error}') from error
    fields(manifest, ['version', 'languages', 'stages', 'examples', 'deferred_examples', 'cells'], 'manifest')
    require(type(manifest['version']) is int and manifest['version'] == 1, 'unsupported manifest version')
    for key in ['languages', 'stages', 'examples', 'deferred_examples', 'cells']:
        require(isinstance(manifest[key], list), f'{key}: expected list')
    langs = manifest['languages']
    require(langs and len(langs) == len(set(langs)), 'languages: empty or duplicate')
    for lang in langs:
        text_id(lang, 'language')
    ordered = {}
    for kind in ['stages', 'examples']:
        ids = []
        for item in manifest[kind]:
            expected = ['id', 'title', 'language_dependent'] if kind == 'stages' else ['id', 'title']
            fields(item, expected, kind)
            text_id(item['id'], kind)
            nonempty(item['title'], kind)
            if kind == 'stages':
                require(type(item['language_dependent']) is bool, 'language_dependent must be boolean')
            ids.append(item['id'])
        require(ids and len(ids) == len(set(ids)), f'{kind}: empty or duplicate IDs')
        ordered[kind] = ids
    stages, examples = ordered['stages'], ordered['examples']
    dependent = {s['id']: s['language_dependent'] for s in manifest['stages']}
    deferred = manifest['deferred_examples']
    for item in deferred:
        text_id(item, 'deferred example')
    require(len(set(deferred)) == len(deferred) and not set(deferred) & set(examples),
            'deferred examples duplicate active or deferred IDs')
    cells = {}
    for cell in manifest['cells']:
        fields(cell, ['example', 'stage', 'languages'], 'cell')
        e, s = cell['example'], cell['stage']
        require(e in examples and s in stages, f'cell: unknown coordinates {e}/{s}')
        require((e, s) not in cells, f'duplicate cell {e}/{s}')
        require(cell['languages'] == (langs if dependent[s] else []),
                f'{e}/{s}: incomplete or unexpected language coverage')
        cells[e, s] = cell
    require(all(any(e == key[0] for key in cells) for e in examples), 'example without cells')
    require(all(any(s == key[1] for key in cells) for s in stages), 'stage without cells')
    expected = {'README.md': {'kind': 'root'}, 'FORMAT.md': {'kind': 'format'},
                'source.md': {'kind': 'fixture'}}
    for s in stages:
        expected[f'stages/{s}.md'] = {'kind': 'stage', 'stage': s}
    for e in examples:
        expected[f'examples/{e}/README.md'] = {'kind': 'example', 'example': e}
    for (e, s), cell in cells.items():
        expected[f'examples/{e}/{s}.md'] = {'kind': 'cell', 'example': e, 'stage': s}
        for lang in cell['languages']:
            expected[f'examples/{e}/{s}.{lang}.md'] = {
                'kind': 'variant', 'example': e, 'stage': s, 'language': lang}
    actual = {p.relative_to(root).as_posix() for p in root.rglob('*.md')}
    require(actual == set(expected), f'page coverage: missing {sorted(set(expected)-actual)}, unlisted {sorted(actual-set(expected))}')
    docs, links = {}, {}
    for name, identity in expected.items():
        p = root / name
        raw = p.read_text()
        match = re.match(r'<!-- spec: (.+) -->\n', raw)
        require(match is not None, f'{name}: missing first-line metadata')
        try:
            metadata = json.loads(match.group(1))
        except ValueError as error:
            raise InvalidSpec(f'{name}: malformed metadata') from error
        require(metadata == identity, f'{name}: metadata does not match path/manifest')
        body = prose(raw, name)
        require(len(re.findall(r'^<!-- spec:', body, re.M)) == 1, f'{name}: repeated metadata')
        docs[name] = body
        links[name] = local_links(body, p)
        for dest, anchor in links[name]:
            require(dest.is_file(), f'{name}: missing link target {dest}')
            if anchor:
                require(dest.suffix == '.md' and anchor in anchors(prose(dest.read_text(), str(dest))),
                        f'{name}: missing anchor {dest}#{anchor}')
        for title in (['Input', 'Owner', 'Result', 'Checks'] if identity['kind'] in ['cell', 'variant'] else []):
            section(body, title, name)
    def destinations(name, body=None):
        return [p for p, _ in (links[name] if body is None else local_links(body, root / name))]
    def contains(name, targets):
        for t in targets:
            require((root / t).resolve() in destinations(name), f'{name}: missing required link to {t}')
    def exact(name, heading, targets):
        body = section(docs[name], heading, name)
        require(destinations(name, body) == [(root / t).resolve() for t in targets],
                f'{name}: {heading} links do not match ordered manifest')
    def expanded(e, s):
        c = cells[e, s]
        return [f'examples/{e}/{s}.md'] + [f'examples/{e}/{s}.{l}.md' for l in c['languages']]
    exact('README.md', 'Follow the pipeline', [f'stages/{s}.md' for s in stages])
    # The example intro section may also refer to the shared fixture.
    example_links = [p for p in destinations('README.md', section(docs['README.md'], 'Follow an example', 'README.md'))
                     if p != (root / 'source.md').resolve()]
    require(example_links == [(root / f'examples/{e}/README.md').resolve() for e in examples],
            'root example TOC does not match manifest')
    for i, s in enumerate(stages):
        name = f'stages/{s}.md'
        for h in ['Input', 'Operation and owner', 'Output and failure contract']:
            section(docs[name], h, name)
        exact(name, 'Apply this stage', [p for e in examples if (e,s) in cells for p in expanded(e,s)])
        neighbors = [f'stages/{stages[j]}.md' for j in [i-1,i+1] if 0 <= j < len(stages)]
        exact(name, 'Pipeline navigation', neighbors)
        contains(name, ['README.md'])
    for e in examples:
        name = f'examples/{e}/README.md'
        expected_links = [(root/p).resolve() for s in stages if (e,s) in cells for p in expanded(e,s)]
        actual_links = [p for p in destinations(name) if p.parent == (root / f'examples/{e}').resolve() and p.name != 'README.md']
        require(actual_links == expected_links, f'{name}: example TOC does not match ordered cells')
        contains(name, ['README.md', 'source.md'])
        active = [s for s in stages if (e,s) in cells]
        for i,s in enumerate(active):
            name = f'examples/{e}/{s}.md'
            contains(name, [f'stages/{s}.md', f'examples/{e}/README.md'])
            exact(name, 'Along this example', [f'examples/{e}/{active[j]}.md' for j in [i-1,i+1] if 0 <= j < len(active)])
            if cells[e,s]['languages']:
                exact(name, 'Language variants', expanded(e,s)[1:])
            else:
                require('## Language variants' not in docs[name], f'{name}: unexpected language variants')
            for lang in cells[e,s]['languages']:
                contains(f'examples/{e}/{s}.{lang}.md', [name, f'stages/{s}.md', f'examples/{e}/README.md'])
    return len(expected), len(stages), len(examples), len(cells)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args()
    try:
        pages, stages, examples, cells = validate(args.root)
    except (InvalidSpec, OSError, TypeError) as error:
        parser.exit(1, f'INVALID: {error}\n')
    print(f'Valid: {pages} pages, {stages} stages, {examples} example paths, {cells} applicable cells.')


if __name__ == '__main__':
    main()
