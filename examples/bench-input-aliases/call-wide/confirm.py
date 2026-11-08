#!/usr/bin/env python3
"""Confirm borrowed C calls using shared addresses and one process per cell."""
import argparse,csv,hashlib,json,os,random,statistics,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
KINDS=['current','hash_grouped','hash','sort','sort_sized']
p=argparse.ArgumentParser(__doc__);p.add_argument('--build',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--aa',action='store_true');p.add_argument('--forks',type=int,default=4);p.add_argument('--target-ms',type=int,default=20);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True);(out/'raw').mkdir();build=json.loads((a.build/'builds.json').read_text());builds=build['builds'];driver=out/'driver';core=max(os.sched_getaffinity(0))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
subprocess.run(['cc','-std=gnu11','-O3','-fno-plt','-Wall','-Wextra','-Werror',str(HERE/'same-process.c'),'-I'+str(a.build.resolve()),'-L'+str(Path(builds['current-time']['library']).parent),'-lc_collection_bench','-Wl,-rpath,'+str(Path(builds['current-time']['library']).parent),'-ldl','-o',str(driver)],check=True)
cells=[('flat',1,'ascending'),('flat',8,'ascending'),('flat',8192,'ascending'),('flat',8192,'shuffled'),('flat',65536,'ascending'),('flat',65536,'shuffled'),('nested',8192,'shuffled'),('groups8',8192,'shuffled'),('shared',8192,'shuffled'),('readonly',8192,'ascending')]
if a.aa:cells=[('flat',8192,'shuffled'),('nested',8192,'shuffled'),('readonly',8192,'ascending')]
rows=[];cases=['flat','nested','groups8','shared','readonly'];libraries=[builds[('current' if a.aa else v)+'-time']['library'] for v in KINDS]
for fork in range(a.forks):
 order=cells.copy();random.Random(20261109+fork).shuffle(order)
 for case,n,ordering in order:
  cmd=['taskset','-c',str(core),str(driver),str(cases.index(case)),str(n),ordering,'11',str(a.target_ms),str(fork),*libraries]
  r=subprocess.run(cmd,capture_output=True,text=True);(out/'raw'/f'{fork}-{case}-{n}-{ordering}.log').write_text(r.stdout+r.stderr);r.check_returncode()
  paths=[line.split(',')[2] for line in r.stdout.splitlines() if line.startswith('LIBRARY,')];assert paths==libraries
  data=[line.split(',') for line in r.stdout.splitlines() if line.startswith('TIME,')];assert len(data)==55
  if a.aa:assert len({line.split(',')[3] for line in r.stdout.splitlines() if line.startswith('LIBRARY,')})==1
  for _,index,batch,it,ns in data:rows.append(dict(fork=fork,case=case,size=n,order=ordering,candidate=KINDS[int(index)],batch=int(batch),ns_per_call=int(ns)/int(it)))
  print(fork,case,n,ordering,flush=True)
summary=[]
for case,n,ordering in sorted(set((r['case'],r['size'],r['order']) for r in rows)):
 med={v:[statistics.median(r['ns_per_call'] for r in rows if (r['case'],r['size'],r['order'],r['candidate'],r['fork'])==(case,n,ordering,v,f)) for f in range(a.forks)] for v in KINDS}
 for v in KINDS:summary.append(dict(case=case,size=n,order=ordering,candidate=v,median_ns=statistics.median(med[v]),paired_ratio=statistics.median(x/y for x,y in zip(med[v],med['current'])),fork_medians=json.dumps(med[v])))
with (out/'summary.csv').open('w') as f:
 w=csv.DictWriter(f,fieldnames=list(summary[0]),lineterminator='\n');w.writeheader();w.writerows(summary)
for d in builds.values():assert sha(d['library'])==d['library_sha256']
(out/'provenance.json').write_text(json.dumps(dict(build=build,aa=a.aa,cpu=core,forks=a.forks,batches=11,target_ms=a.target_ms,driver_sha256=sha(driver),source_sha256={str(p):sha(p) for p in [HERE/'confirm.py',HERE/'same-process.c']},summary_sha256=sha(out/'summary.csv')),indent=2))
