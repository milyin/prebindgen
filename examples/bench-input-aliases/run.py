"""Standalone alias-validator experiments; raw samples stay outside the repository."""
import argparse
import hashlib
import json
import os
import platform
import random
import statistics
import subprocess
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--runtime', type=Path)
p.add_argument('--forks', type=int, default=4)
p.add_argument('--batches', type=int, default=5)
p.add_argument('--target-ms', type=int, default=2)
p.add_argument('--sizes', default='0,1,4,16,32,64,128,256,512,2048,8192')
p.add_argument('--skip-counts',action='store_true')
p.add_argument('--quick', action='store_true')
a = p.parse_args()
root = Path(__file__).resolve().parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=True)
assert not any(out.iterdir()),'Use a new output directory'
(out/'raw').mkdir(exist_ok=True)
runtime = (a.runtime or root/'prebindgen-runtime/src/input.rs').resolve()
source = Path(__file__).with_name('bench.rs').resolve()
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
wrapper = out/'main.rs'
wrapper.write_text('#![allow(dead_code)]\n#[path='+json.dumps(str(runtime))+'] mod runtime;\n#[path='+json.dumps(str(root/'prebindgen-runtime/src/input.rs'))+'] mod production;\ninclude!('+json.dumps(str(source))+');\n')
compiler = ['rustc', '--edition=2021', '-C', 'opt-level=3', '-C', 'codegen-units=1', str(wrapper)]
for name, extra in [('time', []), ('counts', ['--cfg', 'counts', '--cfg', 'metrics'])]:
    subprocess.run([*compiler, *extra, '-o', str(out/name)], check=True)
source_hashes={str(path):sha(path) for path in [source,Path(__file__),runtime,root/'prebindgen-runtime/src/input.rs',out/'time',out/'counts']}
subprocess.run([str(out/'time'), 'verify'], check=True)
cases = subprocess.check_output([str(out/'time'), 'list'], text=True).splitlines()
cases = [c for c in cases if c.split('/')[0] in a.sizes.split(',')]
if a.quick:
    cases = [c for c in cases if '/access/' not in c and '/quads/' not in c and '/duplicate/' not in c]
candidates = ['selected', 'sorted', 'linear', 'hash', 'hash_fast', 'btree', 'bloom512', 'bloom8192', 'blocked8192', 'bloom_grow', 'hybrid64', 'hybrid128']
cpu = min(os.sched_getaffinity(0))
samples, counts = [], []
for fork in range(a.forks):
    order = cases.copy()
    random.Random(20261107+fork).shuffle(order)
    for i, case in enumerate(order):
        names=candidates.copy()
        random.Random(case+str(fork//2)).shuffle(names)
        if fork%2: names.reverse()
        for candidate in names:
            command=['taskset','-c',str(cpu),str(out/'time'),'time',candidate,case,str(a.target_ms),str(a.batches),str(fork)]
            run=subprocess.run(command,text=True,capture_output=True,check=True)
            (out/'raw'/f'{fork}-{i}-{candidate}.log').write_text(run.stdout+run.stderr)
            for line in run.stdout.splitlines():
                _,selected,selected_case,batch,iterations,ns=line.split(',')
                assert selected==candidate and selected_case==case
                samples.append(dict(fork=fork,candidate=candidate,case=case,batch=int(batch),iterations=int(iterations),ns_per_call=int(ns)/int(iterations)))
            assert len(run.stdout.splitlines())==a.batches
        (out/'progress.json').write_text(json.dumps(samples))
        print(f'{fork} {i+1}/{len(cases)} {case}',flush=True)
if not a.skip_counts:
    command=['taskset','-c',str(cpu),str(out/'counts'),'countall',';'.join(cases)]
    raw=subprocess.check_output(command,text=True)
    (out/'raw/counts.log').write_text(raw)
    parsed={}
    lines=iter(raw.splitlines())
    for header in lines:
        _,candidate,case=header.split(',')
        fields=next(lines).split(',');assert fields.pop(0)=='COUNT'
        parsed.setdefault((case,candidate),[]).append([int(v) for v in fields])
    for (case,candidate),observations in parsed.items():
        assert len(observations)==3 and all(v==observations[0] for v in observations)
        counts.append(dict(candidate=candidate,case=case,**dict(zip(['allocations','reallocations','requested_bytes','peak_heap_bytes','state_size_bytes','filter_queries','filter_positives','filter_false_positives','exact_entries_examined'],observations[0]))))
    assert len(counts)==len(cases)*len(candidates)
summary = []
for case in cases:
    for candidate in candidates:
        medians = [statistics.median(x['ns_per_call'] for x in samples if x['case']==case and x['candidate']==candidate and x['fork']==f) for f in range(a.forks)]
        summary.append(dict(case=case, candidate=candidate, median_ns=statistics.median(medians), fork_medians=medians))
result = dict(settings=vars(a)|{'output':str(out),'runtime':str(runtime),'cpu':cpu}, rustc=subprocess.check_output(['rustc','-Vv'], text=True), platform=platform.platform(), cpu_model=next((l.split(':',1)[1].strip() for l in Path('/proc/cpuinfo').read_text().splitlines() if l.startswith('model name')),''),libc=platform.libc_ver(),
              hashes=source_hashes, summary=summary, allocations=counts,
              samples_sha256=hashlib.sha256(json.dumps(samples,sort_keys=True).encode()).hexdigest())
result['settings']['runtime'] = str(runtime)
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n')
print('Finished',out,flush=True)
