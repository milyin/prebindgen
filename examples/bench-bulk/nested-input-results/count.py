"""Measure calling-thread Rust allocations separately from uninstrumented timing."""
import argparse
import csv
import hashlib
import json
import os
import platform
import random
import statistics
import subprocess
from pathlib import Path

p = argparse.ArgumentParser(__doc__)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--after', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--forks', type=int, default=2)
p.add_argument('--smoke', action='store_true')
a = p.parse_args()
assert a.forks > 0
root = a.output.resolve()
root.mkdir(parents=True, exist_ok=True)
assert not (root / 'results.json').exists(), 'Results already complete'
(root / 'raw').mkdir(exist_ok=True)
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
info = {v: json.loads((d / 'build.json').read_text()) for v, d in [('before', a.before), ('after', a.after)]}
for key in ['kotlin_sources_sha256', 'fixture_sha256', 'rustc', 'dependency_lock_sha256', 'cargo_config_sha256', 'cargo_profile', 'profile_environment']:
    assert info['before'][key] == info['after'][key], key
# Namespace-local JVM PIDs can collide in the shared hsperfdata directory and
# prepend a startup warning. Preserve raw metadata, compare the compiler line.
versions = []
for d in info.values():
    lines = [line for line in d['kotlin'].splitlines() if line.startswith('info: kotlinc-jvm ')]
    assert len(lines) == 1
    versions.append(lines[0])
assert versions[0] == versions[1]
for d in info.values():
    assert d['native_allocations'] is True and d['instrumentation_probe']=='NATIVE_ALLOCATIONS,true'
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
cells = [('bulk', case, n) for case in ['nested','nested_many','callback_nested','callback_nested_many'] for n in [1,64,512]]
cells += [('plain', case, n) for case in ['nested','nested_many','callback_nested','callback_nested_many'] for n in [1,512]]
cells += [('bulk','flat',512),('bulk','records',512),('plain','control',1)]
if a.smoke:
    cells = [(mode,case,8) for mode in ['plain','bulk'] for case in ['nested_many','callback_nested_many']]
flags = ['-Xms256m', '-Xmx256m', '-XX:+UseG1GC', '-XX:ActiveProcessorCount=2']
samples, runs = [], []
for fork in range(a.forks):
    order = cells.copy()
    random.Random(20261103 + fork).shuffle(order)
    for mode, case, size in order:
        for version in (['before', 'after'] if fork % 2 == 0 else ['after', 'before']):
            d = info[version]
            command = ['java', *flags, *(['-Xcheck:jni'] if a.smoke else []),
                       '-Djava.library.path=' + str(Path(d['library']).parent), '-cp', d['jar'],
                       'bench.allocations.AllocationBenchKt', mode, case, str(size)]
            ident = f'{fork}-{mode}-{case}-{size}-{version}'
            log = root / 'raw' / f'{ident}.log'
            if log.exists():
                result = subprocess.CompletedProcess(command, 0, stdout=log.read_text(), stderr='')
            else:
                result = subprocess.run(command, capture_output=True, text=True)
                log.write_text(result.stdout + result.stderr)
            result.check_returncode()
            assert 'WARNING in native method' not in result.stdout + result.stderr
            records = []
            for line in result.stdout.splitlines():
                if line.startswith('NATIVE_ALLOC,'):
                    _, m, c, n, batch, allocs, reallocs, bytes_ = line.split(',')
                    assert (m, c, int(n)) == (mode, case, size)
                    records.append(dict(version=version, mode=mode, case=case, size=size, fork=fork,
                                        batch=int(batch), allocations=int(allocs), reallocations=int(reallocs),
                                        requested_native_bytes=int(bytes_)))
            assert len(records) == 10 and {r['batch'] for r in records} == set(range(10))
            samples.extend(records)
            runs.append(dict(id=ident, command=command, stdout_sha256=sha(log)))
            (root / 'progress.json').write_text(json.dumps(dict(runs=runs, samples=samples)))
            print(f'{len(runs)}/{len(cells)*a.forks*2} {ident}', flush=True)
summary = []
for mode, case, size in sorted(cells):
    row = dict(mode=mode, case=case, size=size)
    for version in info:
        selected = [s for s in samples if (s['version'],s['mode'],s['case'],s['size']) == (version,mode,case,size)]
        row[version] = {metric: dict(median=statistics.median(s[metric] for s in selected),
                                    min=min(s[metric] for s in selected), max=max(s[metric] for s in selected))
                        for metric in ['allocations','reallocations','requested_native_bytes']}
    summary.append(row)
with (root / 'samples.csv').open('w') as out:
    w = csv.DictWriter(out, fieldnames=list(samples[0]));w.writeheader();w.writerows(samples)
for d in info.values():
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
report = dict(builds=info, settings=vars(a) | {'before':str(a.before),'after':str(a.after),'output':str(a.output)},
              affinity=sorted(os.sched_getaffinity(0)), platform=platform.platform(),
              java=subprocess.run(['java','-version'],capture_output=True,text=True).stderr,
              flags=flags, runs=runs, summary=summary, runner_sha256=sha(__file__),
              samples_sha256=sha(root/'samples.csv'))
(root / 'results.json').write_text(json.dumps(report,indent=2)+'\n')
print('Finished:', root / 'results.json', flush=True)
