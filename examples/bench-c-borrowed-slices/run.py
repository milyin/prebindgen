#!/usr/bin/env python3
"""Compare generated borrowed C slice outputs with their previous owning rows."""
import argparse, csv, hashlib, json, os, platform, random, re, shlex, statistics, subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--before',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--target-dir',type=Path,default=Path('/tmp/alias-role-target'))
p.add_argument('--sizes',default='0,1,16,128,512,8192,65536')
p.add_argument('--forks',type=int,default=4)
p.add_argument('--batches',type=int,default=9)
p.add_argument('--target-ms',type=int,default=10)
p.add_argument('--build-only',action='store_true')
p.add_argument('--prepared',action='store_true')
p.add_argument('--aa',action='store_true')
a=p.parse_args();assert a.forks>0 and a.forks%2==0
out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
sha=lambda x:hashlib.sha256(Path(x).read_bytes()).hexdigest()
builds={}
for version,checkout in [('before',a.before.resolve()),('after',ROOT)]:
 for kind in ['time','count']:
  directory=out/f'{version}-{kind}'
  if not a.prepared:
   command=['python3',str(ROOT/'examples/bench-c-collections/build.py'),'--checkout',str(checkout),'--output',str(directory),'--target-dir',str(a.target_dir)]
   if kind=='count':command+=['--count-allocations']
   with (out/f'build-{version}-{kind}.log').open('w') as log:subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True,env=os.environ|{'CARGO_BUILD_JOBS':'2'})
  d=json.loads((directory/'build.json').read_text());assert d['count_allocations']==(kind=='count')
  assert sha(d['library'])==d['library_sha256']
  header=(directory/'bindings.h').read_text()
  direct='bench_i64_slice_view bench_borrowed_values(' in header
  assert direct==(version=='after'), 'wrong output conversion in frozen bindings'
  returned=re.search(r'(\w+) bench_borrowed_values\(',header).group(1)
  callback=re.search(r'void bench_borrowed_emit\([^;]*\b(bench_callback_[a-f0-9]+) p1_',header).group(1)
  definition=re.search(r'typedef struct \{([^{}]*)\} '+re.escape(callback)+';',header,re.S).group(1)
  argument=re.search(r'void \(\*run\)\(void \*context, (\w+) arg0\)',definition).group(1)
  driver=directory/'borrowed-driver'
  if not a.prepared:
   cc=shlex.split(os.environ.get('CC','cc'))
   command=[*cc,'-O3','-std=c11','-Wall','-Wextra','-Werror','-I'+str(directory),'-DDIRECT_VIEW='+str(int(direct)),'-DRETURN_CARRIER='+returned,'-DCALLBACK_CARRIER='+argument,'-DCALLBACK_TYPE='+callback,'-DCALLBACK_FREE='+argument+'_free',str(HERE/'driver.c'),'-L'+str(Path(d['library']).parent),'-lc_collection_bench','-Wl,-rpath,'+str(Path(d['library']).parent),'-ldl','-o',str(driver)]
   subprocess.run(command,check=True)
   (directory/'borrowed-driver.json').write_text(json.dumps(dict(command=command,sha256=sha(driver),source_sha256=sha(HERE/'driver.c')),indent=2)+'\n')
  driver_info=json.loads((directory/'borrowed-driver.json').read_text());assert sha(driver)==driver_info['sha256']
  d['borrowed_driver']=str(driver);d['borrowed_driver_info']=driver_info;builds[version,kind]=d
  print('build ready',version,kind,flush=True)
assert all(d['fixture_sha256']==builds['before','time']['fixture_sha256'] for d in builds.values())
if a.build_only:raise SystemExit(0)
raw=out/('raw-aa' if a.aa else 'raw');raw.mkdir(exist_ok=True);assert not list(raw.iterdir())
core=min(os.sched_getaffinity(0));rows=[];cells=[(case,int(n)) for case in ['view','read','callback','control'] for n in a.sizes.split(',')]
for fork in range(a.forks):
 cells_order=cells.copy();random.Random(20261108+fork).shuffle(cells_order)
 for case,n in cells_order:
  for version in (['before','after'] if fork%2==0 else ['after','before']):
   d=builds['before' if a.aa else version,'time'];command=['taskset','-c',str(core),d['borrowed_driver'],'time',case,str(n),str(a.batches),str(a.target_ms),'verify']
   run=subprocess.run(command,text=True,capture_output=True,check=True,env=os.environ|{'LD_LIBRARY_PATH':str(Path(d['library']).parent)})
   (raw/f'{fork}-{case}-{n}-{version}.log').write_text(run.stdout+run.stderr)
   loaded=next(s[8:] for s in run.stdout.splitlines() if s.startswith('LIBRARY,'));assert Path(loaded).resolve()==Path(d['library']).resolve()
   samples=[s.split(',') for s in run.stdout.splitlines() if s.startswith('TIME,')];assert len(samples)==a.batches
   rows.extend(dict(fork=fork,case=case,size=n,version=version,batch=int(b),ns_per_call=int(ns)/int(it)) for _,b,it,ns in samples)
 print('timed fork',fork,flush=True)
summary=[]
for case,n in cells:
 vals={v:[statistics.median(r['ns_per_call'] for r in rows if (r['fork'],r['case'],r['size'],r['version'])==(f,case,n,v)) for f in range(a.forks)] for v in ['before','after']}
 summary.append(dict(case=case,size=n,before_ns=statistics.median(vals['before']),after_ns=statistics.median(vals['after']),paired_after_before=statistics.median(y/x for x,y in zip(vals['before'],vals['after'])),before_forks=vals['before'],after_forks=vals['after']))
counts=[]
if not a.aa:
 for (version,kind),d in builds.items():
  if kind!='count':continue
  for case,n in cells:
   run=subprocess.run([d['borrowed_driver'],'count',case,str(n),'5','1','verify'],text=True,capture_output=True,check=True,env=os.environ|{'LD_LIBRARY_PATH':str(Path(d['library']).parent)})
   loaded=next(s[8:] for s in run.stdout.splitlines() if s.startswith('LIBRARY,'));assert Path(loaded).resolve()==Path(d['library']).resolve()
   (raw/f'count-{version}-{case}-{n}.log').write_text(run.stdout+run.stderr)
   values=[tuple(map(int,s.split(',')[2:])) for s in run.stdout.splitlines() if s.startswith('COUNT,')];assert len(values)==5 and len(set(values))==1
   counts.append(dict(version=version,case=case,size=n,allocations=values[0][0],reallocations=values[0][1],requested_bytes=values[0][2]))
for d in builds.values():assert sha(d['library'])==d['library_sha256']
report=dict(settings={k:str(v) if isinstance(v,Path) else v for k,v in vars(a).items()},core=core,platform=platform.platform(),libc=platform.libc_ver(),cpuinfo=Path('/proc/cpuinfo').read_text().split('processor\t: 1')[0],builds={str(k):v for k,v in builds.items()},summary=summary,counts=counts,runner_sha256=sha(__file__))
(out/('aa-results.json' if a.aa else 'results.json')).write_text(json.dumps(report,indent=2)+'\n')
for name,data in [('latency',summary),('allocations',counts)]:
 if not data:continue
 with (out/(('aa-' if a.aa else '')+name+'.csv')).open('w') as f:
  w=csv.DictWriter(f,fieldnames=list(data[0]),lineterminator='\n');w.writeheader();w.writerows(data)
print('finished',out,flush=True)
