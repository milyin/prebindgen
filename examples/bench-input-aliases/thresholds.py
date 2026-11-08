#!/usr/bin/env python3
"""Measure compile-time alias-history limits through frozen generated C calls."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import random
import re
import statistics
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--limits',default='64,128,256,512,1024,max')
p.add_argument('--sizes',default='64,128,256,512,1024,8192')
p.add_argument('--cases',default='nested,groups8,shared,callback,flat,readonly')
p.add_argument('--orders',default='ascending,descending,shuffled')
p.add_argument('--forks',type=int,default=4)
p.add_argument('--batches',type=int,default=7)
p.add_argument('--target-ms',type=int,default=8)
p.add_argument('--target-dir',type=Path,default=Path('/tmp/threshold-c-target'))
p.add_argument('--build-only',action='store_true')
p.add_argument('--prepared',action='store_true')
p.add_argument('--reference-build',type=Path,help='optional older checked timing build with frozen bindings.rs')
a=p.parse_args();assert a.forks>0 and a.forks%2==0
out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
limits=a.limits.split(',');builds={}
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()

def normalized_text(text):
    text=re.sub(r'\s+','',text)
    text=re.sub(r'::prebindgen_runtime::DynamicInputsWithThreshold::<(?:[0-9]+usize|\{(?:[0-9]+usize|::core::primitive::usize::MAX)\}),?>::default\(\)','NATIVE_HISTORY()',text)
    return text.replace('::prebindgen_runtime::DynamicInputs::default()','NATIVE_HISTORY()')

def normalized(path):
    return normalized_text(Path(path).read_text())

for argument in ['128usize', '{128usize},', '{::core::primitive::usize::MAX}']:
    assert normalized_text(f'::prebindgen_runtime::DynamicInputsWithThreshold::<{argument}>::default()')=='NATIVE_HISTORY()'

for limit in limits:
    for kind in ['time','count']:
        directory=out/f'{limit}-{kind}'
        if not a.prepared:
            command=['python3',str(ROOT/'examples/bench-c-collections/build.py'),'--checkout',str(ROOT),'--output',str(directory),'--target-dir',str(a.target_dir),'--alias-index-threshold',limit]
            if kind=='count':command+=['--count-allocations']
            with (out/f'build-{limit}-{kind}.log').open('w') as log:
                subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True,env=os.environ|{'CARGO_BUILD_JOBS':'2'})
        d=json.loads((directory/'build.json').read_text())
        assert d['alias_checks'] and d['count_allocations']==(kind=='count') and d['alias_index_threshold']==limit
        assert sha(d['library'])==d['library_sha256'] and sha(d['history_driver'])==d['history_driver_sha256']
        d['frozen_bindings']=str(directory/'bindings.rs');assert sha(d['frozen_bindings'])==d['bindings_sha256']
        builds[limit,kind]=d
        print('build ready',limit,kind,flush=True)
if a.reference_build:
    directory=a.reference_build.resolve();d=json.loads((directory/'build.json').read_text())
    assert d['alias_checks'] and not d['count_allocations']
    d['frozen_bindings']=str(directory/'bindings.rs');assert sha(d['frozen_bindings'])==d['bindings_sha256']
    builds['old128','time']=d
    limits=['old128',*limits]
reference=builds[limits[0],'time']
for d in builds.values():
    assert d['header_abi_sha256']==reference['header_abi_sha256']
    assert normalized(d['frozen_bindings'])==normalized(reference['frozen_bindings']), 'glue differs beyond the configured history constructor'
if a.build_only:
    print('all variants frozen; timing not started',flush=True);raise SystemExit(0)
raw=out/'raw';raw.mkdir(exist_ok=True)
assert not list(raw.iterdir()), 'use new measurements or remove only your own previous raw directory'
core=min(os.sched_getaffinity(0));rows=[]
# One C executable selects each library via the loader path; the executable
# reports the loaded path and verifies every result. Each call uses unique owners.
driver=reference['history_driver']
cells=[(case,int(size),order) for case in a.cases.split(',') for size in a.sizes.split(',') for order in a.orders.split(',')]
for fork in range(a.forks):
    order=cells.copy();random.Random(20261108+fork).shuffle(order)
    for cell,(case,size,pointer_order) in enumerate(order):
        variants=limits.copy();random.Random(str((case,size,pointer_order,fork//2))).shuffle(variants)
        if fork%2:variants.reverse()
        for limit in variants:
            d=builds[limit,'time']
            command=['taskset','-c',str(core),driver,'time',case,str(size),pointer_order,str(a.batches),str(a.target_ms),'verify']
            result=subprocess.run(command,text=True,capture_output=True,env=os.environ|{'LD_LIBRARY_PATH':str(Path(d['library']).parent)},check=True)
            (raw/f'{fork}-{case}-{size}-{pointer_order}-{limit}.log').write_text(result.stdout+result.stderr)
            loaded=next(line.removeprefix('LIBRARY,') for line in result.stdout.splitlines() if line.startswith('LIBRARY,'))
            assert Path(loaded).resolve()==Path(d['library']).resolve()
            samples=[line.split(',') for line in result.stdout.splitlines() if line.startswith('TIME,')];assert len(samples)==a.batches
            for _,batch,it,ns in samples:rows.append(dict(fork=fork,case=case,size=size,order=pointer_order,limit=limit,batch=int(batch),ns_per_call=int(ns)/int(it)))
        (out/'progress.json').write_text(json.dumps(rows))
        print('time',fork,cell+1,len(cells),flush=True)
summary=[]
for case,size,order in sorted(set((x['case'],x['size'],x['order']) for x in rows)):
    for limit in limits:
        medians=[statistics.median(x['ns_per_call'] for x in rows if (x['case'],x['size'],x['order'],x['limit'],x['fork'])==(case,size,order,limit,f)) for f in range(a.forks)]
        summary.append(dict(case=case,size=size,order=order,limit=limit,median_ns=statistics.median(medians),fork_medians=medians))
counts=[]
for limit in limits:
    if (limit,'count') not in builds:continue
    d=builds[limit,'count']
    for case,size,order in cells:
        command=[d['history_driver'],'count',case,str(size),order,'5','1','verify']
        result=subprocess.run(command,text=True,capture_output=True,check=True,env=os.environ|{'LD_LIBRARY_PATH':str(Path(d['library']).parent)})
        loaded=next(line.removeprefix('LIBRARY,') for line in result.stdout.splitlines() if line.startswith('LIBRARY,'))
        assert Path(loaded).resolve()==Path(d['library']).resolve()
        (raw/f'count-{case}-{size}-{order}-{limit}.log').write_text(result.stdout+result.stderr)
        values=[tuple(map(int,line.split(',')[2:])) for line in result.stdout.splitlines() if line.startswith('COUNT,')]
        assert len(values)==5 and len(set(values))==1
        counts.append(dict(case=case,size=size,order=order,limit=limit,allocations=values[0][0],reallocations=values[0][1],requested_bytes=values[0][2]))
for d in builds.values():assert sha(d['library'])==d['library_sha256']
for name,data in [('samples',rows),('summary',summary),('counts',counts)]:
    if name=='summary':data=[{**x,'fork_medians':json.dumps(x['fork_medians'])} for x in data]
    with (out/f'{name}.csv').open('w') as f:
        w=csv.DictWriter(f,fieldnames=list(data[0]));w.writeheader();w.writerows(data)
(out/'results.json').write_text(json.dumps(dict(settings={k:str(v) if isinstance(v,Path) else v for k,v in vars(a).items()},builds={str(k):v for k,v in builds.items()},core=core,summary=summary,counts=counts,runner_sha256=sha(__file__)),indent=2)+'\n')
print('finished',out,flush=True)
