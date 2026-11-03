#!/usr/bin/env python3
"""Compare native ownership queues with identical Kotlin driver/fixtures."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import random
import statistics
import subprocess

p = argparse.ArgumentParser(__doc__)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--after', type=Path, required=True)
p.add_argument('--driver', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--forks', type=int, default=4)
p.add_argument('--counts', action='store_true')
p.add_argument('--check-jni', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
info = {key: json.loads((path / 'build.json').read_text()) for key, path in [('before', a.before), ('after', a.after)]}
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
for key in ['fixture_sha256', 'rustc', 'dependency_lock_sha256', 'cargo_config_sha256', 'cargo_profile', 'profile_environment']:
    assert info['before'][key] == info['after'][key], key
for build in info.values():
    assert build['native_allocations'] == a.counts
    assert sha(build['jar']) == build['jar_sha256'] and sha(build['library']) == build['native_sha256']
cells = [('control',1), ('primitive_array',512), ('primitive_fold',512), ('flat',1), ('flat',512), ('fold_flat',1), ('fold_flat',512), ('nested',64), ('fold_nested',64)]
if a.counts:
    cells = [(kind, size) for kind in ['flat','fold_flat','nested','fold_nested'] for size in [0,1,64,512]] + [('control',1),('primitive_array',64),('primitive_fold',64)]
if a.check_jni:
    cells = [(kind,8) for kind in ['flat','fold_flat','nested','fold_nested','primitive_array','primitive_fold']]
flags = ['-Xms256m','-Xmx256m','-XX:+UseG1GC','-XX:ActiveProcessorCount=2','-XX:-UsePerfData']
samples, runs = [], []
for fork in range(a.forks):
    order = cells.copy()
    random.Random(20261103 + fork).shuffle(order)
    for case, size in order:
        for version in (['before','after'] if fork % 2 == 0 else ['after','before']):
            build = info[version]
            command = ['java', *flags, *(['-Xcheck:jni'] if a.check_jni else []), '-Djava.library.path='+str(Path(build['library']).parent), '-cp', str(a.driver)+os.pathsep+build['jar'], 'bench.outputowners.DriverKt', case, str(size), 'counts' if a.counts else 'time']
            result = subprocess.run(command, capture_output=True, text=True)
            ident = f'{fork}-{case}-{size}-{version}'
            (a.output / (ident+'.log')).write_text(result.stdout + result.stderr)
            result.check_returncode()
            assert 'WARNING in native method' not in result.stdout + result.stderr
            batch = []
            for line in result.stdout.splitlines():
                if line.startswith('COUNT,'):
                    _, c, n, b, allocs, reallocs, requested = line.split(',')
                    batch.append(dict(version=version,case=c,size=int(n),fork=fork,batch=int(b),allocations=int(allocs),reallocations=int(reallocs),requested_bytes=int(requested)))
                elif line.startswith('TIME,'):
                    _, c, n, b, iterations, ns, cpu, allocated, gc = line.split(',')
                    batch.append(dict(version=version,case=c,size=int(n),fork=fork,batch=int(b),iterations=int(iterations),nanos=int(ns),cpu_nanos=int(cpu),jvm_bytes=int(allocated),gc_collections=int(gc)))
            assert len(batch) == (10 if a.counts else 12)
            if not a.counts:
                assert 'WARMUP,true,' in result.stdout, 'JIT warmup did not settle'
            samples.extend(batch)
            runs.append(dict(id=ident,command=command,output_sha256=sha(a.output/(ident+'.log'))))
            print(f'{len(runs)}/{len(cells)*a.forks*2} {ident}', flush=True)
summary = []
for case, size in sorted(cells):
    row = dict(case=case,size=size)
    for version in info:
        scores=[]
        for fork in range(a.forks):
            rows=[s for s in samples if (s['version'],s['case'],s['size'],s['fork']) == (version,case,size,fork)]
            if a.counts:
                scores.append({m:statistics.median(s[m] for s in rows) for m in ['allocations','reallocations','requested_bytes']})
            else:
                operations=sum(s['iterations'] for s in rows)
                scores.append({m:sum(s[m] for s in rows)/operations for m in ['nanos','cpu_nanos','jvm_bytes']})
        row[version]={m:dict(median=statistics.median(s[m] for s in scores),min=min(s[m] for s in scores),max=max(s[m] for s in scores)) for m in scores[0]}
        row[version]['forks']=scores
    summary.append(row)
with (a.output/'samples.csv').open('w') as output:
    w=csv.DictWriter(output,fieldnames=list(samples[0])); w.writeheader(); w.writerows(samples)
report=dict(builds=info,driver_sha256=sha(a.driver),runner_sha256=sha(__file__),samples_sha256=sha(a.output/'samples.csv'),settings={k:str(v) if isinstance(v,Path) else v for k,v in vars(a).items()},runs=runs,summary=summary)
(a.output/'results.json').write_text(json.dumps(report,indent=2)+'\n')
for build in info.values():
    assert sha(build['jar']) == build['jar_sha256'] and sha(build['library']) == build['native_sha256']
