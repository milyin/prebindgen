"""Compare frozen JNI callback builds; keep raw observations outside Git."""
import argparse, hashlib, json, os, platform, statistics, subprocess
from pathlib import Path
p=argparse.ArgumentParser(__doc__)
p.add_argument('--before',type=Path,required=True)
p.add_argument('--after',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--java-flag', action='append', default=[], help='additional JVM option; repeat as --java-flag=-Xss16m')
p.add_argument('--forks',type=int,default=6)
p.add_argument('--aa',action='store_true')
p.add_argument('--kotlin-only',action='store_true',help='compare Kotlin runtimes using the same before native image')
p.add_argument('--java',default='java',help='Java executable')
p.add_argument('--group',action='append',help='MODE:SIZES:CASES; replaces the default groups')
p.add_argument('--warmup',type=int,default=10)
p.add_argument('--batches',type=int,default=11)
p.add_argument('--target-ms',type=int,default=30)
a=p.parse_args(); assert a.forks>0 and a.forks%2==0
assert min(a.warmup,a.batches,a.target_ms)>0
root=a.output.resolve();root.mkdir(parents=True,exist_ok=True);assert not list(root.iterdir())
(root/'raw').mkdir()
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
info={v:json.loads((d/'build.json').read_text()) for v,d in [('before',a.before),('after',a.after)]}
for key in ['fixture_sha256','kotlin','rustc','dependency_lock_sha256','cargo_config_sha256','cargo_profile','profile_environment'] + (['rust_sources_sha256'] if a.kotlin_only else ['kotlin_sources_sha256']):
 assert info['before'][key]==info['after'][key],key
if a.kotlin_only:
 before=info['before']['kotlin_sources_sha256']; after=info['after']['kotlin_sources_sha256']
 assert before.keys()==after.keys(), 'generated Kotlin file inventory changed'
 assert all(before[k]==after[k] or k.endswith('/NativeHandle.kt') for k in before), 'unexpected Kotlin source change'
for d in info.values():
 assert not d['native_allocations']
 assert sha(d['jar'])==d['jar_sha256'] and sha(d['library'])==d['native_sha256']
flags=['-Xms256m','-Xmx256m','-XX:+UseG1GC','-XX:ActiveProcessorCount=2'] + a.java_flag
groups=[('bulk','0,1,8,64,512,4096','callback_owned_records'),('bulk','1','callback_owned_record'),('plain','1,512','callback_owned_records'),('plain','1','callback_owned_record'),('bulk','1,512','owned_handle_records_roundtrip,numbers_input'),('bulk','1','control')]
if a.aa: groups=[('bulk','0,1,512,4096','callback_owned_records'),('bulk','1','callback_owned_record'),('bulk','512','owned_handle_records_roundtrip,numbers_input')]
if a.group:
 groups=[tuple(group.split(':')) for group in a.group]
 assert all(len(group)==3 and group[0] in ['plain','bulk'] for group in groups)
samples=[];runs=[]
for fork in range(a.forks):
 for group,(mode,sizes,cases) in enumerate(groups):
  for version in (['before','after'] if (fork+group)%2==0 else ['after','before']):
   d=info['before' if a.aa else version]
   # Isolate either native code or Kotlin bytecode, holding the other image fixed.
   native=info['before'] if a.kotlin_only else d
   kotlin=d if a.kotlin_only else info['before']
   cmd=[a.java,*flags,'-Djava.library.path='+str(Path(native['library']).parent),'-cp',kotlin['jar'],'BenchKt',mode,sizes,str(a.warmup),str(a.batches),str(a.target_ms),str(fork),'64',cases]
   result=subprocess.run(cmd,capture_output=True,text=True)
   ident=f'{fork}-{group}-{version}';log=root/'raw'/f'{ident}.log';log.write_text(result.stdout+result.stderr);result.check_returncode()
   selected=[]
   for line in result.stdout.splitlines():
    if line.startswith('BULK_BENCH,'):
     _,m,case,n,batch,it,nanos,bytes_=line.split(',')
     assert m==mode and case in cases.split(',') and n in sizes.split(',')
     selected.append(dict(fork=fork,version=version,mode=m,case=case,size=int(n),batch=int(batch),iterations=int(it),ns_per_call=int(nanos)/int(it),jvm_bytes_per_call=int(bytes_)/int(it)))
   assert len(selected)==len(cases.split(','))*len(sizes.split(','))*a.batches
   samples.extend(selected);runs.append(dict(id=ident,command=cmd,stdout_sha256=sha(log)))
   (root/'progress.json').write_text(json.dumps(dict(runs=runs,samples=samples)))
   print(ident,flush=True)
summary=[]
for mode,case,size in sorted({(s['mode'],s['case'],s['size']) for s in samples}):
 row=dict(mode=mode,case=case,size=size)
 for metric in ['ns_per_call','jvm_bytes_per_call']:
  fm={v:[statistics.median(s[metric] for s in samples if (s['version'],s['mode'],s['case'],s['size'],s['fork'])==(v,mode,case,size,f)) for f in range(a.forks)] for v in info}
  row[metric]={v:dict(median=statistics.median(vals),min=min(vals),max=max(vals),fork_medians=vals) for v,vals in fm.items()}
  row[metric]['paired_after_before']=statistics.median(y/x if x else 1.0 for x,y in zip(fm['before'],fm['after']))
 summary.append(row)
for d in info.values():assert sha(d['library'])==d['native_sha256'] and sha(d['jar'])==d['jar_sha256']
(root/'results.json').write_text(json.dumps(dict(builds=info,settings=dict(forks=a.forks,aa=a.aa,kotlin_only=a.kotlin_only,warmup=a.warmup,batches=a.batches,target_ms=a.target_ms,groups=groups),affinity=sorted(os.sched_getaffinity(0)),platform=platform.platform(),java=subprocess.run([a.java,'-version'],capture_output=True,text=True).stderr,flags=flags,runs=runs,summary=summary,runner_sha256=sha(__file__),samples_sha256=hashlib.sha256(json.dumps(samples,sort_keys=True).encode()).hexdigest()),indent=2)+'\n')
print('Finished',root,flush=True)
