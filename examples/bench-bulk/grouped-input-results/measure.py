import csv, hashlib, json, os, platform, statistics, subprocess, time
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/data/home/milyin/work/PREBINDGEN_WORKSPACE/prebindgen-shared-c-repr-records')
PREP={'previous':Path('/tmp/prebindgen-eager-input-benchmark'),'updated':Path('/tmp/prebindgen-grouped-jni-benchmark')}
INFO={key:json.loads((path/'build.json').read_text()) for key,path in PREP.items()}
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
assert INFO['previous']['fixture_sha256']==INFO['updated']['fixture_sha256']
assert INFO['previous']['rustc']==INFO['updated']['rustc']
assert INFO['previous']['kotlin']==INFO['updated']['kotlin']
for d in INFO.values():
    assert sha(d['jar'])==d['jar_sha256']
    assert sha(d['library'])==d['native_sha256']
for version in PREP:
    messages=[json.loads(line) for line in Path('/tmp/854-'+('after' if version=='previous' else 'group')+'-native-build.jsonl').read_text().splitlines()]
    out,=[Path(m['out_dir']) for m in messages if m.get('reason')=='build-script-executed' and 'perftest-c' in m['package_id']]
    command=['cc','-O3','-std=c11','-Wall','-Wextra','-Werror','-I'+str(out),str(ROOT/'native.c'),'-L'+str((Path('/tmp/prebindgen-eager-input-benchmark') if version=='previous' else Path('/tmp/prebindgen-grouped-input-benchmark'))/'target/release'),'-lperftest_c','-Wl,-rpath,'+str((Path('/tmp/prebindgen-eager-input-benchmark') if version=='previous' else Path('/tmp/prebindgen-grouped-input-benchmark'))/'target/release'),'-o',str(ROOT/('native-'+version))]
    # Both revisions use size_t ownership receipts.
    subprocess.run(command,check=True)
    INFO[version]['native_c_library_sha256']=sha((Path('/tmp/prebindgen-eager-input-benchmark') if version=='previous' else Path('/tmp/prebindgen-grouped-input-benchmark'))/'target/release/libperftest_c.so')
flags=['-Xms256m','-Xmx256m','-XX:+UseG1GC','-XX:ActiveProcessorCount=2','-XX:+PrintCommandLineFlags']
sizes='0,1,8,64,512'
cases='control,record_input,numbers_input,records_input,callback_record_return'
orders=[['previous-plain','updated-plain','updated-bulk','previous-bulk'],['updated-plain','previous-bulk','previous-plain','updated-bulk'],['previous-bulk','updated-bulk','updated-plain','previous-plain'],['updated-bulk','previous-plain','previous-bulk','updated-plain']]
samples=[]; run_orders=[]
print('All builds finished; starting balanced revision/mode runs.',flush=True)
for trial,order in enumerate(orders):
    for treatment in order:
        version,mode=treatment.split('-'); d=INFO[version]
        command=['java',*flags,'-Djava.library.path='+str(Path(d['library']).parent),'-jar',d['jar'],mode,sizes,'6','9','20',str(trial),'64,4096,65536',cases]
        start=time.time(); result=subprocess.run(command,capture_output=True,text=True)
        (ROOT/f'jvm-{trial}-{treatment}.log').write_text(result.stdout+result.stderr)
        result.check_returncode(); assert 'WARNING in native method' not in result.stdout+result.stderr
        assert 'SINK,' in result.stdout
        count=0
        for line in result.stdout.splitlines():
            if not line.startswith('BULK_BENCH,'):continue
            _,actual,case,size,batch,iterations,nanos,allocated=line.split(',');assert actual==mode
            samples.append(dict(boundary='JNI',version=version,mode=mode,case=case,size=int(size),trial=trial,batch=int(batch),iterations=int(iterations),nanos=int(nanos),jvm_allocated_bytes=int(allocated)))
            count+=1
        assert count==13*9,(treatment,count)
        run_orders.append(dict(trial=trial,treatment=treatment,seconds=time.time()-start,command=command))
        print(f'JNI round {trial+1}/4 {treatment} complete ({time.time()-start:.1f}s)',flush=True)
        (ROOT/'progress.json').write_text(json.dumps(samples))
# Native boundaries: constructors and consuming ownership. Box creation occurs before timing;
# unboxing, Box deallocation, final values, alias checks and source drops are timed.
for trial in range(4):
    combinations=[(kind,n) for kind in range(3) for n in [0,1,8,64,512,4096]]
    if trial%2:combinations.reverse()
    for kind,n in combinations:
        for version in (['staged','previous','updated'] if trial%2==0 else ['updated','previous','staged']):
            command=[str(ROOT/('native-'+version)),str(kind),str(n),str(trial)]
            result=subprocess.run(command,capture_output=True,text=True)
            (ROOT/f'c-{trial}-{version}-{kind}-{n}.log').write_text(result.stdout+result.stderr)
            result.check_returncode()
            lines=result.stdout.splitlines();assert len(lines)==9
            for line in lines:
                _,actual_kind,size,actual_trial,batch,iterations,nanos=line.split(',')
                assert int(actual_kind)==kind and int(actual_trial)==trial
                samples.append(dict(boundary='C',version=version,mode='native',case=['primitive','constructor','consuming_handles'][kind],size=int(size),trial=trial,batch=int(batch),iterations=int(iterations),nanos=int(nanos),jvm_allocated_bytes=None))
    print(f'Native C round {trial+1}/4 complete',flush=True)
keys=sorted(set((s['boundary'],s['mode'],s['case'],s['size']) for s in samples))
summary=[]
for boundary,mode,case,size in keys:
    row=dict(boundary=boundary,mode=mode,case=case,size=size)
    for version in PREP:
        per_jvm=[];allocated=[]
        for trial in range(4):
            group=[s for s in samples if (s['boundary'],s['mode'],s['case'],s['size'],s['version'],s['trial'])==(boundary,mode,case,size,version,trial)]
            assert len(group)==9
            per_jvm.append(statistics.median(s['nanos']/s['iterations'] for s in group))
            if boundary=='JNI':allocated.append(statistics.median(s['jvm_allocated_bytes']/s['iterations'] for s in group))
        row[version]=dict(ns=statistics.median(per_jvm),min_ns=min(per_jvm),max_ns=max(per_jvm),trial_medians=per_jvm,jvm_bytes=statistics.median(allocated) if allocated else None)
    if boundary=='C':
        medians=[]
        for trial in range(4):
            group=[s for s in samples if (s['boundary'],s['case'],s['size'],s['version'],s['trial'])==('C',case,size,'staged',trial)]
            assert len(group)==9
            medians.append(statistics.median(s['nanos']/s['iterations'] for s in group))
        row['staged']=dict(ns=statistics.median(medians),trial_medians=medians)
    row['speedup']=row['previous']['ns']/row['updated']['ns']
    row['latency_change_percent']=(row['updated']['ns']/row['previous']['ns']-1)*100
    row['paired_speedups']=[a/b for a,b in zip(row['previous']['trial_medians'],row['updated']['trial_medians'])]
    summary.append(row)
with (ROOT/'samples.csv').open('w') as f:
    writer=csv.DictWriter(f,fieldnames=list(samples[0]));writer.writeheader();writer.writerows(samples)
report=dict(previous_revision='9fe03d086',updated_revision=INFO['updated']['commit'],builds=INFO,affinity=sorted(os.sched_getaffinity(0)),platform=platform.platform(),java=subprocess.run(['java','-version'],capture_output=True,text=True).stderr,jvm_flags=flags,jni_orders=run_orders,warmup=6,batches=9,batch_target_ms=20,summary=summary)
(ROOT/'results.json').write_text(json.dumps(report,indent=2)+'\n')
text=['# PR 854: grouped validation versus per-value hashing','', 'Compared with 9fe03d086 (previous PR revision). Identical fixtures and release compiler settings. Median of four process medians, each with nine measured batches. JNI uses six warmup batches, balanced revision/mode order, and no -Xcheck:jni. C handle creation is outside the timer; consuming decode, deallocation and Rust source drops remain timed.','', '| Boundary | Mode | Case | Elements/bytes | Previous ns/call | Updated ns/call | Previous / updated | Updated latency |','|---|---|---|---:|---:|---:|---:|---:|']
for row in summary:
    text.append(f"| {row['boundary']} | {row['mode']} | {row['case']} | {row['size']} | {row['previous']['ns']:.1f} | {row['updated']['ns']:.1f} | {row['speedup']:.3f}x | {row['latency_change_percent']:+.1f}% |")
text+=['', 'Native C also remeasures ed9faf756, before the per-value hash-history change:', '', '| Case | Rows | Original staged ns | Per-value hash ns | Grouped ns | Staged / grouped |', '|---|---:|---:|---:|---:|---:|']
for row in summary:
    if row['boundary']=='C':text.append(f"| {row['case']} | {row['size']} | {row['staged']['ns']:.1f} | {row['previous']['ns']:.1f} | {row['updated']['ns']:.1f} | {row['staged']['ns']/row['updated']['ns']:.3f}x |")
(ROOT/'REPORT.md').write_text('\n'.join(text)+'\n')
for d in INFO.values():
    assert sha(d['jar'])==d['jar_sha256'] and sha(d['library'])==d['native_sha256']
print('Done. '+str(ROOT/'REPORT.md'),flush=True)
