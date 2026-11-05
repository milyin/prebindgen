#!/usr/bin/env python3
"""Compare frozen C input libraries; source handles are created outside timing/counting."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess

p=argparse.ArgumentParser(__doc__)
p.add_argument('--before',type=Path,required=True)
p.add_argument('--after',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--kind',choices=['count','time'],required=True)
p.add_argument('--forks',type=int,default=4)
p.add_argument('--same-build',choices=['before','after'])
p.add_argument('--smoke',action='store_true')
p.add_argument('--cases',default='handles,records,wide,text,bytes,buffers,text_large,bytes_large,buffers_large,numbers,control')
p.add_argument('--sizes',default='0,1,8,64,512,4096')
p.add_argument('--batches',type=int)
p.add_argument('--target-ms',type=int,default=20)
a=p.parse_args()
assert a.forks>0
root=a.output.resolve();root.mkdir(parents=True,exist_ok=True)
assert not list(root.iterdir()),'Use an empty output directory'
(root/'raw').mkdir()
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
builds={v:json.loads((d/'build.json').read_text()) for v,d in [('before',a.before),('after',a.after)]}
for key in ['fixture_sha256','rustc','cc','cargo_profile','lock_sha256','cargo_configs','profile_environment','header_abi_sha256']:
    assert builds['before'][key]==builds['after'][key],key
for d in builds.values():
    assert d['count_allocations']==(a.kind=='count')
    assert sha(d['library'])==d['library_sha256'] and sha(d['input_driver'])==d['input_driver_sha256']
effective=builds if not a.same_build else {v:builds[a.same_build] for v in builds}
# One executable for both versions. The loader path selects the native library,
# and dladdr in the executable verifies the library that actually answered.
driver=effective['before']['input_driver']
cases=a.cases.split(',');sizes=[int(n) for n in a.sizes.split(',')]
assert set(cases)<=set(['handles','records','wide','text','bytes','buffers','text_large','bytes_large','buffers_large','numbers','control'])
cells=[(case,n) for case in cases if case!='control' for n in sizes]
if 'control' in cases:cells+=[('control',1)]
if a.smoke:cells=[('handles',0),('handles',8),('records',8),('wide',8),('text',8),('bytes',8),('buffers',8),('text_large',8),('bytes_large',8),('buffers_large',8),('numbers',8),('control',1)]
batches=a.batches or (10 if a.kind=='count' else 12)
if a.smoke:batches=2
runs=[];samples=[]
for fork in range(a.forks):
    order=cells.copy();random.Random(20261103+fork).shuffle(order)
    for case,n in order:
        for version in (['before','after'] if fork%2==0 else ['after','before']):
            d=effective[version]
            env=dict(os.environ,LD_LIBRARY_PATH=str(Path(d['library']).parent))
            command=[driver,a.kind,case,str(n),str(batches),str(a.target_ms)]
            result=subprocess.run(command,capture_output=True,text=True,env=env)
            ident=f'{fork}-{case}-{n}-{version}';log=root/'raw'/(ident+'.log')
            log.write_text(result.stdout+result.stderr);result.check_returncode()
            loaded=[l[len('LIBRARY,'):] for l in result.stdout.splitlines() if l.startswith('LIBRARY,')]
            assert len(loaded)==1 and Path(loaded[0]).resolve()==Path(d['library']).resolve(),loaded
            found=[]
            for l in result.stdout.splitlines():
                f=l.split(',')
                if f[0]=='COUNT':
                    _,batch,alloc,realloc,bytes_=f
                    found.append(dict(version=version,case=case,size=n,fork=fork,batch=int(batch),allocations=int(alloc),reallocations=int(realloc),requested_native_bytes=int(bytes_)))
                elif f[0]=='TIME':
                    _,batch,iterations,wall,cpu,checksum=f
                    found.append(dict(version=version,case=case,size=n,fork=fork,batch=int(batch),iterations=int(iterations),wall_ns=int(wall),cpu_ns=int(cpu),checksum=int(checksum)))
            assert len(found)==batches and {s['batch'] for s in found}==set(range(batches))
            samples+=found;runs.append(dict(id=ident,command=command,loaded_library=loaded[0],stdout_sha256=sha(log)))
            (root/'progress.json').write_text(json.dumps(dict(runs=runs,samples=samples)))
            print(f'{len(runs)}/{len(cells)*a.forks*2} {ident}',flush=True)
summary=[]
for case,n in sorted(cells):
    row=dict(case=case,size=n)
    for version in builds:
        selected=[s for s in samples if (s['version'],s['case'],s['size'])==(version,case,n)]
        if a.kind=='count':
            row[version]={metric:dict(median=statistics.median(s[metric] for s in selected),min=min(s[metric] for s in selected),max=max(s[metric] for s in selected)) for metric in ['allocations','reallocations','requested_native_bytes']}
        else:
            scores=[]
            for fork in range(a.forks):
                group=[s for s in selected if s['fork']==fork];count=sum(s['iterations'] for s in group)
                scores.append(dict(fork=fork,wall_ns=sum(s['wall_ns'] for s in group)/count,cpu_ns=sum(s['cpu_ns'] for s in group)/count))
            row[version]=dict(wall_ns=statistics.median(s['wall_ns'] for s in scores),cpu_ns=statistics.median(s['cpu_ns'] for s in scores),min_wall_ns=min(s['wall_ns'] for s in scores),max_wall_ns=max(s['wall_ns'] for s in scores),forks=scores)
    if a.kind=='time':
        row['wall_change_percent']=(row['after']['wall_ns']/row['before']['wall_ns']-1)*100
        row['cpu_change_percent']=(row['after']['cpu_ns']/row['before']['cpu_ns']-1)*100
        row['paired_wall_changes_percent']=[(x['wall_ns']/y['wall_ns']-1)*100 for x,y in zip(row['after']['forks'],row['before']['forks'])]
    summary.append(row)
with (root/'samples.csv').open('w') as out:
    writer=csv.DictWriter(out,fieldnames=list(samples[0]));writer.writeheader();writer.writerows(samples)
for d in builds.values():
    assert sha(d['library'])==d['library_sha256'] and sha(d['input_driver'])==d['input_driver_sha256']
report=dict(builds=builds,effective_builds=effective,driver_sha256=sha(driver),settings={k:str(v) if isinstance(v,Path) else v for k,v in vars(a).items()},cells=cells,runs=runs,summary=summary,
            affinity=sorted(os.sched_getaffinity(0)),platform=platform.platform(),
            cpu=subprocess.run(['lscpu'],capture_output=True,text=True).stdout,
            loader_environment={k:v for k,v in os.environ.items() if k in ['LD_PRELOAD','MALLOC_ARENA_MAX','GLIBC_TUNABLES']},
            samples_sha256=sha(root/'samples.csv'),runner_sha256=sha(__file__))
(root/'results.json').write_text(json.dumps(report,indent=2)+'\n')
print('Finished:',root/'results.json',flush=True)
