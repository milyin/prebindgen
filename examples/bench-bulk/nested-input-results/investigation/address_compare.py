"""Compare frozen JNI libraries with fixed JVM warm-up and controlled pointer ranks."""
import argparse,hashlib,json,statistics,subprocess
from pathlib import Path
p=argparse.ArgumentParser(__doc__)
p.add_argument('--before',type=Path,required=True);p.add_argument('--after',type=Path,required=True)
p.add_argument('--driver-jar',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
p.add_argument('--forks',type=int,default=4)
a=p.parse_args();assert a.forks>0 and a.forks%2==0
root=a.output.resolve();root.mkdir(parents=True,exist_ok=False)
info={v:json.loads((d/'build.json').read_text()) for v,d in [('before',a.before),('after',a.after)]}
for key in ['fixture_sha256','kotlin_sources_sha256','rustc','dependency_lock_sha256','cargo_profile']:
 assert info['before'][key]==info['after'][key],key
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for d in info.values():
 assert not d['native_allocations'] and d['instrumentation_probe']=='NATIVE_ALLOCATIONS,false'
 assert sha(d['library'])==d['native_sha256'] and sha(d['jar'])==d['jar_sha256']
flags=['-Xms256m','-Xmx256m','-XX:+UseG1GC','-XX:ActiveProcessorCount=2']
samples=[];runs=[]
for fork in range(a.forks):
 for version in (['before','after'] if fork%2==0 else ['after','before']):
  d=info[version]
  cmd=['java',*flags,'-Djava.library.path='+str(Path(d['library']).parent),'-cp',str(a.driver_jar.resolve())+':'+info['before']['jar'],'AddressBenchKt']
  result=subprocess.run(cmd,capture_output=True,text=True);log=root/f'{fork}-{version}.log';log.write_text(result.stdout+result.stderr);result.check_returncode()
  assert 'FIXED_WARMUP_DONE' in result.stdout
  selected=[]
  for line in result.stdout.splitlines():
   if line.startswith('ADDRESS,'):
    _,shape,order,returned,batch,it,ns=line.split(',');selected.append(dict(fork=fork,version=version,shape=shape,order=order,callback=returned=='true',batch=int(batch),ns=int(ns)/int(it)))
  assert len(selected)==12*9
  samples.extend(selected);runs.append(dict(command=cmd,stdout_sha256=sha(log)))
  (root/'progress.json').write_text(json.dumps(samples))
  print(fork,version,flush=True)
summary=[]
for shape in ['one','many']:
 for order in ['ascending','descending','shuffled']:
  for returned in [False,True]:
   fm={v:[statistics.median(x['ns'] for x in samples if (x['fork'],x['version'],x['shape'],x['order'],x['callback'])==(f,v,shape,order,returned)) for f in range(a.forks)] for v in info}
   summary.append(dict(shape=shape,order=order,callback=returned,fork_medians=fm,paired_ratio=statistics.median(y/x for x,y in zip(fm['before'],fm['after']))))
for d in info.values():assert sha(d['library'])==d['native_sha256']
(root/'results.json').write_text(json.dumps(dict(builds=info,summary=summary,runs=runs,forks=a.forks,flags=flags,fixed_warmup_calls=80000,case_warmup_batches=7,batches=9,target_ms=30,driver_sha256=sha(a.driver_jar),samples_sha256=hashlib.sha256(json.dumps(samples,sort_keys=True).encode()).hexdigest()),indent=2)+'\n')
