"""Confirm large-history changes through frozen generated JNI/Kotlin bindings."""
import argparse, hashlib, json, os, platform, re, statistics, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--before',type=Path,required=True)
p.add_argument('--after',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--forks',type=int,default=6)
p.add_argument('--aa',action='store_true')
p.add_argument('--compare-thresholds',action='store_true',help='allow only the configured native history constructor to differ')
p.add_argument('--policy',choices=['native','kotlin'],required=True)
p.add_argument('--sizes',default='1,128,512,8192')
p.add_argument('--warmup',type=int,default=10)
p.add_argument('--batches',type=int,default=11)
p.add_argument('--target-ms',type=int,default=30)
a=p.parse_args(); assert a.forks>0 and a.forks%2==0
root=a.output.resolve();root.mkdir(parents=True,exist_ok=True);assert not list(root.iterdir())
(root/'raw').mkdir()
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
info={v:json.loads((d/'build.json').read_text()) for v,d in [('before',a.before),('after',a.after)]}
for key in ['fixture_sha256','kotlin_sources_sha256','rustc','dependency_lock_sha256','cargo_config_sha256','cargo_profile','profile_environment','native_input_checks']:
 assert info['before'][key]==info['after'][key],key
glue={v:{name:sha(directory/name) for name in ['plain.rs','bulk.rs']} for v,directory in [('before',a.before),('after',a.after)]}
if a.compare_thresholds:
 def normalized(path):
  text=re.sub(r'\s+','',path.read_text())
  text=re.sub(r'::prebindgen_runtime::DynamicInputsWithThreshold::<(?:[0-9]+usize|\{(?:[0-9]+usize|::core::primitive::usize::MAX)\}),?>::default\(\)','NATIVE_HISTORY()',text)
  return text.replace('::prebindgen_runtime::DynamicInputs::default()','NATIVE_HISTORY()')
 for name in ['plain.rs','bulk.rs']:
  assert normalized(a.before/name)==normalized(a.after/name), 'generated JNI Rust differs beyond threshold constructor'
else:
 assert glue['before']==glue['after'], 'generated JNI Rust differs'
for d in info.values():
 assert d['native_input_checks']==(a.policy=='native'), 'unexpected alias-validation policy'
 assert not d['native_allocations'] and d['instrumentation_probe']=='NATIVE_ALLOCATIONS,false'
 assert sha(d['jar'])==d['jar_sha256'] and sha(d['library'])==d['native_sha256']
flags=['-Xss16m','-Xms256m','-Xmx256m','-XX:+UseG1GC','-XX:ActiveProcessorCount=2']
groups=[('bulk',a.sizes,'nested_owned_handles_roundtrip,nested_many_owned_handles_roundtrip,callback_nested_owned_handles,callback_many_nested_owned_handles'),
        ('bulk',a.sizes,'owned_handles_roundtrip,owned_handle_records_roundtrip'),('plain','1','control')]
if a.aa: groups=[('bulk',a.sizes,'nested_many_owned_handles_roundtrip,callback_many_nested_owned_handles'),('plain','1','control')]
samples=[];runs=[]
for fork in range(a.forks):
 for group,(mode,sizes,cases) in enumerate(groups):
  for version in (['before','after'] if (fork+group)%2==0 else ['after','before']):
   d=info['before' if a.aa else version]
   # Exact same generated Kotlin/driver bytecode for both libraries.
   cmd=['taskset','-c',str(min(os.sched_getaffinity(0))),'java',*flags,'-Djava.library.path='+str(Path(d['library']).parent),'-cp',info['before']['jar'],'BenchKt',mode,sizes,str(a.warmup),str(a.batches),str(a.target_ms),str(fork),'64',cases]
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
(root/'results.json').write_text(json.dumps(dict(builds=info,rust_glue_sha256=glue,settings=dict(forks=a.forks,aa=a.aa,compare_thresholds=a.compare_thresholds,policy=a.policy,warmup=a.warmup,batches=a.batches,target_ms=a.target_ms),affinity=sorted(os.sched_getaffinity(0)),platform=platform.platform(),java=subprocess.run(['java','-version'],capture_output=True,text=True).stderr,flags=flags,runs=runs,summary=summary,runner_sha256=sha(__file__),samples_sha256=hashlib.sha256(json.dumps(samples,sort_keys=True).encode()).hexdigest()),indent=2)+'\n')
print('Finished',root,flush=True)
