#!/usr/bin/env python3
"""Confirm Kotlin grouping with shared objects, one JVM and alternating batches."""
import argparse,csv,hashlib,json,os,statistics,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(__doc__);p.add_argument('--build',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--forks',type=int,default=5);p.add_argument('--target-ms',type=int,default=35);p.add_argument('--aa',action='store_true');a=p.parse_args()
 build=a.build.resolve();out=a.output.resolve();out.mkdir();(out/'raw').mkdir();core=max(os.sched_getaffinity(0));rows=[];files=[build/'paired.jar',build/'current-time/libtype_group_bench.so'];frozen={str(p):sha(p) for p in files}
 for fork in range(a.forks):
  for order in ['ascending','shuffled']:
   cmd=['taskset','-c',str(core),'java','-Xss64m','-Xms384m','-Xmx384m','-Xbatch','-XX:+UseSerialGC','-Djava.library.path='+str(build/'current-time'),'-cp',str(build/'paired.jar'),'PairedKt','1,8,128,8192,65536',order,str(fork),'11',str(a.target_ms),'aa' if a.aa else 'ab']
   r=subprocess.run(cmd,capture_output=True,text=True);(out/'raw'/f'{fork}-{order}.log').write_text(r.stdout+r.stderr);r.check_returncode();assert 'SINK,' in r.stdout
   for line in r.stdout.splitlines():
    if line.startswith('PAIR,'):
     _,case,n,v,batch,it,ns,allocated=line.split(',');rows.append(dict(fork=fork,order=order,case=case,size=int(n),variant=['buckets','flat'][int(v)],batch=int(batch),ns_per_call=int(ns)/int(it),bytes_per_call=int(allocated)/int(it)))
   print(fork,order,flush=True)
 summary=[]
 for case,n,order in sorted({(r['case'],r['size'],r['order']) for r in rows}):
  med={v:[statistics.median(r['ns_per_call'] for r in rows if (r['case'],r['size'],r['order'],r['variant'],r['fork'])==(case,n,order,v,f)) for f in range(a.forks)] for v in ['buckets','flat']}
  for v in med:
   ratios=[x/y for x,y in zip(med[v],med['buckets'])];sub=[r for r in rows if(r['case'],r['size'],r['order'],r['variant'])==(case,n,order,v)]
   summary.append(dict(case=case,size=n,order=order,variant=v,median_ns=statistics.median(med[v]),ratio_to_buckets=statistics.median(ratios),min_fork_ratio=min(ratios),max_fork_ratio=max(ratios),bytes_per_call=statistics.median(r['bytes_per_call'] for r in sub),fork_medians_ns=json.dumps(med[v])))
 for name,data in [('samples',rows),('summary',summary)]:
  with (out/(name+'.csv')).open('w') as f:w=csv.DictWriter(f,fieldnames=list(data[0]),lineterminator="\n");w.writeheader();w.writerows(data)
 assert all(sha(p)==h for p,h in frozen.items())
 (out/'provenance.json').write_text(json.dumps(dict(files=frozen,cpu=core,forks=a.forks,batches=11,warmups=4,target_ms=a.target_ms,aa=a.aa,java=subprocess.check_output(['java','-version'],stderr=subprocess.STDOUT,text=True),sources={p.name:sha(p) for p in HERE.iterdir() if p.is_file()}),indent=2))
if __name__=='__main__':main()
