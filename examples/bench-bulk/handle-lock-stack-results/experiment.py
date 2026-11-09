#!/usr/bin/env python3
"""Compare recursive monitor groups in fresh JVMs; write summaries outside Git."""
import argparse
import json
import os
from pathlib import Path
import random
import statistics
import subprocess

ROOT = Path(__file__).resolve().parent
p = argparse.ArgumentParser(__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--baseline-ref', default='8ef88e0d438e46e9d1982038ddc7a1735c63dd00')
p.add_argument('--kotlinc', default=os.environ.get('KOTLINC', 'kotlinc'))
p.add_argument('--java', default='java')
p.add_argument('--forks', type=int, default=6)
p.add_argument('--variants', default='base,2,4,8,16,32')
p.add_argument('--sizes', default='0,1,3,8,64,512,8192')
a = p.parse_args()
assert a.forks > 0
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=True)
assert not list(out.iterdir()), 'use an empty output directory'
variants = a.variants.split(',')
sizes = list(map(int, a.sizes.split(',')))
assert set(variants) <= {'base', '2', '4', '8', '16', '32'}
baseline = subprocess.check_output(['git', 'show', a.baseline_ref + ':prebindgen-jni/src/native_handle.kt'], cwd=ROOT, text=True)
(out/'NativeHandle.kt').write_text(baseline.replace('PACKAGE', 'bench.lock'))
groups = 'package bench.lock\n'
for size in [2,4,8,16,32]:
    nested = 'acquire(index + %d)' % size
    for index in reversed(range(size)):
        nested = 'synchronized(ordered[index + %d]) { %s }' % (index, nested)
    groups += ("internal fun <R> group%d(ordered: List<NativeHandle>, body: () -> R): R {\n"
               "    val end = ordered.size - ordered.size %% %d\n"
               "    fun tail(index: Int): R = if (index == ordered.size) body() else synchronized(ordered[index]) { tail(index + 1) }\n"
               "    fun acquire(index: Int): R = if (index == end) tail(index) else %s\n"
               "    return acquire(0)\n}\n") % (size, size, nested)
(out/'Groups.kt').write_text(groups)
jar = out/'experiment.jar'
subprocess.run([a.kotlinc, str(out/'NativeHandle.kt'), str(out/'Groups.kt'), str(ROOT/'Experiment.kt'), '-include-runtime', '-d', str(jar)], check=True)
flags = ['-Xms128m', '-Xmx256m', '-Xss1m', '-XX:+UseG1GC', '-XX:ActiveProcessorCount=2', '-XX:CompileCommand=dontinline,bench/lock/ExperimentKt.fixedBody']
bench_flags = [x if x != '-Xss1m' else '-Xss16m' for x in flags]
limits = []
for warm in ['cold', 'warm']:
    for size in [4096,8192,16384,32768]:
        for variant in variants:
            r = subprocess.run([a.java,*flags,'-cp',str(jar),'bench.lock.ExperimentKt','limit',variant,str(size),warm], capture_output=True, text=True, check=True)
            line = next(s for s in r.stdout.splitlines() if s.startswith('LIMIT,'))
            limits.append(dict(variant=variant,size=size,warm=warm,success=line.rsplit(',',1)[1]=='true'))
            print(line, flush=True)
(out/'limits.json').write_text(json.dumps(dict(flags=flags,results=limits),indent=2)+'\n')
samples = []
for fork in range(a.forks):
    cells = [(variant,size) for variant in variants for size in sizes]
    random.Random(20261109+fork).shuffle(cells)
    for variant,size in cells:
        r = subprocess.run([a.java,*bench_flags,'-cp',str(jar),'bench.lock.ExperimentKt','bench',variant,str(size)], capture_output=True, text=True, check=True)
        rows = [line.split(',') for line in r.stdout.splitlines() if line.startswith('LOCK,')]
        assert len(rows) == 11
        samples.append(dict(fork=fork,variant=variant,size=size,ns=statistics.median(int(r[5])/int(r[4]) for r in rows),bytes=statistics.median(int(r[6])/int(r[4]) for r in rows)))
        (out/'progress.json').write_text(json.dumps(samples))
        print('BENCH',fork,variant,size,flush=True)
summary = []
for variant in variants:
    for size in sizes:
        x = [s for s in samples if s['variant']==variant and s['size']==size]
        summary.append(dict(variant=variant,size=size,ns=statistics.median(s['ns'] for s in x),bytes=statistics.median(s['bytes'] for s in x),fork_medians=[s['ns'] for s in x]))
(out/'results.json').write_text(json.dumps(dict(baseline_ref=a.baseline_ref,flags=bench_flags,java=subprocess.run([a.java,'-version'],capture_output=True,text=True).stderr,summary=summary),indent=2)+'\n')
