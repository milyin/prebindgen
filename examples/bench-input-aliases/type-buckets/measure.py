#!/usr/bin/env python3
"""Sequential, CPU-pinned paired JNI forks and same-address C candidates."""
import argparse,csv,hashlib,json,os,random,statistics,subprocess,shutil
from pathlib import Path
HERE=Path(__file__).resolve().parent
KINDS=['current','flat','typed','cross'];CASES=['mut_one','mixed_mut','mixed_shared']
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(__doc__);p.add_argument('--build',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--forks',type=int,default=5);p.add_argument('--target-ms',type=int,default=40);p.add_argument('--only',choices=['c','jni','checks']);p.add_argument('--aa',action='store_true');a=p.parse_args()
 build=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True);(out/'raw').mkdir();core=max(os.sched_getaffinity(0));rows=[];checks=[];allocations=[];sources={p.name:sha(p) for p in HERE.iterdir() if p.is_file()}
 frozen={str(p):sha(p) for p in build.rglob('*.so')}|{str(p):sha(p) for p in build.glob('*.jar')}
 def run(cmd,label):
  r=subprocess.run(['taskset','-c',str(core),*map(str,cmd)],capture_output=True,text=True);(out/'raw'/(label+'.log')).write_text(r.stdout+r.stderr);r.check_returncode();return r.stdout
 def java(variant):return ['java','-Xss64m','-Xms384m','-Xmx384m','-Xbatch','-XX:+UseSerialGC','-Djava.library.path='+str(build/'current-time'),'-cp',str(build/(variant+'.jar'))]
 if a.only in [None,'jni','checks']:
  for variant in ['buckets','flat']:checks.append(run(java(variant)+['bench.types.ChecksKt'],variant+'-checks').strip())
 if a.only in [None,'c','checks']:
  subprocess.run(['cc','-O3','-Wall','-Wextra','-Werror','-I',str(build),str(HERE/'driver.c'),'-ldl','-o',str(build/'driver')],check=True)
  libs=[build/(v+'-time')/'libtype_group_bench.so' for v in KINDS]
  checks.append(run([build/'driver',1,0,0,1,10,0,*libs],'c-checks').strip())
  if a.aa:
   libs=[]
   for i in range(4):
    path=out/f'current-{i}.so';shutil.copyfile(build/'current-time/libtype_group_bench.so',path);libs.append(path)
  if a.only!='checks':
   cells=[(case,n,order) for case in CASES for n in [1,8,128,8192,65536] for order in ['ascending','shuffled'] if not(n in [1,8,128] and order=='shuffled')]
   if a.aa:cells=[('mixed_mut',128,'ascending'),('mixed_mut',65536,'shuffled'),('mixed_shared',8192,'shuffled')]
   for fork in range(a.forks):
    random.Random(42+fork).shuffle(cells)
    for case,n,order in cells:
     raw=run([build/'driver',CASES.index(case),n,int(order=='shuffled'),9,a.target_ms,fork,*libs],f'c-{fork}-{case}-{n}-{order}')
     for line in raw.splitlines():
      if line.startswith('TIME,'):
       _,variant,batch,it,ns=line.split(',');rows.append(dict(path='c',fork=fork,case=case,size=n,order=order,variant=KINDS[int(variant)],ns=int(ns)/int(it),bytes=None))
     print('C',fork,case,n,order,flush=True)
   if not a.aa:
    libs=[build/(v+'-count')/'libtype_group_bench.so' for v in KINDS]
    for case,n in [(c,n) for c in CASES for n in [8,8192,65536]]:
     raw=run([build/'driver',CASES.index(case),n,1,1,1,0,*libs],f'alloc-{case}-{n}')
     for line in raw.splitlines():
      if line.startswith('ALLOC,'):
       _,v,alloc,realloc,requested=line.split(',');allocations.append(dict(case=case,size=n,variant=KINDS[int(v)],allocations=float(alloc),reallocations=float(realloc),requested_bytes=float(requested)))
 if a.only in [None,'jni']:
  variants=['buckets','flat'];sizes='1,8,128,8192,65536';selected='control,mut_one,mixed_mut,mixed_shared'
  for fork in range(a.forks):
   for order in ['ascending','shuffled']:
    for v in (variants if fork%2==0 else variants[::-1]):
     raw=run(java('buckets' if a.aa else v)+['BenchKt',sizes,order,fork,4,9,a.target_ms,selected],f'jni-{fork}-{order}-{v}')
     assert 'SINK,' in raw
     for line in raw.splitlines():
      if line.startswith('TYPED_BENCH,'):
       _,case,n,batch,it,ns,allocated=line.split(',');rows.append(dict(path='jni',fork=fork,case=case,size=int(n),order=order,variant=v,ns=int(ns)/int(it),bytes=int(allocated)/int(it)))
     print('JNI',fork,order,v,flush=True)
 if rows:
  with (out/'samples.csv').open('w') as f:w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator="\n");w.writeheader();w.writerows(rows)
  summary=[]
  for path,case,n,order in sorted({(r['path'],r['case'],r['size'],r['order']) for r in rows}):
   variants=KINDS if path=='c' else ['buckets','flat'];base=variants[0];med={}
   for v in variants:med[v]=[statistics.median(r['ns'] for r in rows if (r['path'],r['case'],r['size'],r['order'],r['variant'],r['fork'])==(path,case,n,order,v,f)) for f in range(a.forks)]
   for v in variants:
    sub=[r for r in rows if (r['path'],r['case'],r['size'],r['order'],r['variant'])==(path,case,n,order,v)]
    summary.append(dict(path=path,case=case,size=n,order=order,variant=v,median_ns=statistics.median(med[v]),ratio_to_current=statistics.median(x/y for x,y in zip(med[v],med[base])),bytes_per_call=statistics.median(r['bytes'] for r in sub) if path=='jni' else None,fork_medians_ns=json.dumps(med[v])))
  with (out/'summary.csv').open('w') as f:w=csv.DictWriter(f,fieldnames=list(summary[0]),lineterminator="\n");w.writeheader();w.writerows(summary)
 if allocations:
  with (out/'allocations.csv').open('w') as f:w=csv.DictWriter(f,fieldnames=list(allocations[0]),lineterminator="\n");w.writeheader();w.writerows(allocations)
 assert all(sha(p)==h for p,h in frozen.items())
 (out/'provenance.json').write_text(json.dumps(dict(build=json.loads((build/'build.json').read_text()),files=frozen,cpu=core,forks=a.forks,batches=9,warmups=4,target_ms=a.target_ms,aa=a.aa,checks=checks,java=subprocess.check_output(['java','-version'],stderr=subprocess.STDOUT,text=True),sources=sources),indent=2))
if __name__=='__main__':main()
