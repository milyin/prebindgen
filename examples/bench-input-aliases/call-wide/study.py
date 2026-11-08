#!/usr/bin/env python3
"""Verify and measure prototype alias checks. Builds must finish before timing."""
import argparse,csv,hashlib,json,os,random,re,resource,statistics,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
KINDS=['current','hash_grouped','hash','sort','sort_sized']
CHECKERS=['current','hash','sort','sort_sized']

def save_csv(path,rows):
    with path.open('w') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');w.writeheader();w.writerows(rows)

def main():
    p=argparse.ArgumentParser(__doc__);p.add_argument('--build',required=True,type=Path);p.add_argument('--output',required=True,type=Path)
    p.add_argument('--forks',type=int,default=4);p.add_argument('--batches',type=int,default=7);p.add_argument('--target-ms',type=int,default=5)
    p.add_argument('--verify-only',action='store_true');p.add_argument('--cpu',type=int);p.add_argument('--quick',action='store_true');p.add_argument('--aa',action='store_true');a=p.parse_args()
    assert a.forks>0 and a.forks%2==0
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=True);assert not list(out.iterdir());(out/'raw').mkdir()
    build=json.loads((a.build/'builds.json').read_text());builds=build['builds'];core=a.cpu if a.cpu is not None else max(os.sched_getaffinity(0));assert core in os.sched_getaffinity(0)
    for d in builds.values():assert sha(d['library'])==d['library_sha256'] and sha(d['driver'])==d['driver_sha256']
    def command(command,log,library=None,checked=True):
        env=os.environ.copy()
        if library:env['LD_LIBRARY_PATH']=str(Path(library).parent)
        if library and not checked:env['LD_DEBUG']='libs'
        r=subprocess.run(command,capture_output=True,text=True,env=env,preexec_fn=lambda:resource.setrlimit(resource.RLIMIT_CORE,(0,0)))
        log.write_text(r.stdout+r.stderr)
        if checked:r.check_returncode()
        if library:
            loaded=next((line.split(',',1)[1] for line in r.stdout.splitlines() if line.startswith('LIBRARY,')),None)
            if loaded:assert Path(loaded).resolve()==Path(library).resolve()
            elif checked:raise AssertionError('loaded-library identity absent')
            else:
                loaded=re.findall(r'calling init:\s*(\S*libc_collection_bench\.so)',r.stderr)
                assert len(loaded)==1 and Path(loaded[0]).resolve()==Path(library).resolve(), 'rejection used wrong library'
        return r
    for variant in KINDS:
        d=builds[variant+'-time']
        for case in ['flat','nested','groups8','shared','callback','callback_single']:
            r=command([builds['current-time']['driver'],'reject',case],out/'raw'/f'reject-{variant}-{case}.log',d['library'],False)
            assert r.returncode!=0 and 'conflicting input aliases' in r.stderr,(variant,case,r.stdout,r.stderr)
    # These probes exercise real native framing and allow repeated shared
    # aliases. Callback malformed-frame failure must still invoke C's row release.
    probe=out/'checks'
    subprocess.run(['cc','-std=c11','-O2','-Wall','-Wextra','-Werror',str(HERE/'checks.c'),'-I'+str(a.build.resolve()),'-L'+str(Path(builds['current-time']['library']).parent),'-lc_collection_bench','-Wl,-rpath,'+str(Path(builds['current-time']['library']).parent),'-ldl','-o',str(probe)],check=True)
    for v in KINDS:
        library=builds[v+'-time']['library']
        env=os.environ|{'LD_LIBRARY_PATH':str(Path(library).parent)}
        for case in ['shared','within-group','token-first','null','range','overflow','callback-range']:
            r=subprocess.run([str(probe),case],env=env,capture_output=True,text=True,preexec_fn=lambda:resource.setrlimit(resource.RLIMIT_CORE,(0,0)))
            (out/'raw'/f'probe-{v}-{case}.log').write_text(r.stdout+r.stderr)
            loaded=next(line.split(',',1)[1] for line in r.stdout.splitlines() if line.startswith('LIBRARY,'))
            assert Path(loaded).resolve()==Path(library).resolve()
            if case=='shared':assert r.returncode==0,(v,case,r.stderr)
            elif case in ['within-group','token-first']:assert r.returncode!=0 and 'conflicting input aliases' in r.stderr,(v,case,r.stderr)
            else:
                assert r.returncode!=0 and ('pointer' in r.stderr or 'range' in r.stderr),(v,case,r.stderr)
                if case=='callback-range':assert r.stderr.count('RELEASED')==1,(v,r.stderr)
    for v in KINDS:
        d=builds[v+'-time']
        for case in ['flat','nested','groups8','shared','readonly','callback','callback_single']:
            command([builds['current-time']['driver'],'time',case,'8','shuffled','1','1','verify'],out/'raw'/f'valid-{v}-{case}.log',d['library'])
    print('Native alias rejection passed for all five candidates.',flush=True)
    # Compile the standalone checker against the same current runtime.
    for counted in [False,True]:
        cmd=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1',str(HERE/'standalone.rs'),'-o',str(out/('counts' if counted else 'time'))]
        if counted:cmd+=['--cfg','counts']
        command(cmd,out/'raw'/f'compile-{counted}.log')
    command([str(out/'time'),'verify'],out/'raw'/'verify.log');print('Standalone differential verification passed.',flush=True)
    if a.verify_only:
        print('All native and standalone correctness probes passed.',flush=True);return
    cells=[]
    for n in [0,1,8]:
        for case in ['flat','nested','callback','readonly']:cells.append((case,n,'ascending'))
    for n in [128,512,8192]:
        for case in ['flat','nested','groups8','shared','callback']:
            for order in ['ascending','shuffled']:cells.append((case,n,order))
    for case in ['flat','nested','callback']:
        for order in ['ascending','shuffled']:cells.append((case,65536,order))
    cells.extend([('readonly',8192,'ascending'),('groups8',8192,'descending'),('nested',8192,'descending')])
    if a.quick:cells=[c for c in cells if c[1] in [1,512,8192] and c[0] in ['flat','nested','shared','callback','readonly']]
    if a.aa:cells=[('flat',1,'ascending'),('flat',8192,'shuffled'),('nested',8192,'shuffled'),('callback',8192,'shuffled'),('readonly',8192,'ascending')]
    rows=[]
    for fork in range(a.forks):
        order=cells.copy();random.Random(20261109+fork).shuffle(order)
        for i,(case,n,ordering) in enumerate(order):
            variants=KINDS.copy();random.Random(str((case,n,ordering,fork//2))).shuffle(variants)
            if fork%2:variants.reverse()
            for v in variants:
                d=builds[('current' if a.aa else v)+'-time']
                r=command(['taskset','-c',str(core),builds['current-time']['driver'],'time',case,str(n),ordering,str(a.batches),str(a.target_ms),'verify'],out/'raw'/f'{fork}-{i}-{v}.log',d['library'])
                samples=[line.split(',') for line in r.stdout.splitlines() if line.startswith('TIME,')];assert len(samples)==a.batches
                for _,batch,it,ns in samples:rows.append(dict(fork=fork,case=case,size=n,order=ordering,candidate=v,batch=int(batch),ns_per_call=int(ns)/int(it)))
            (out/'progress.json').write_text(json.dumps(rows));print('native',fork,i+1,len(cells),flush=True)
    summaries=[]
    for case,n,order in sorted(set((r['case'],r['size'],r['order']) for r in rows)):
        reference=[statistics.median(r['ns_per_call'] for r in rows if (r['fork'],r['case'],r['size'],r['order'],r['candidate'])==(f,case,n,order,'current')) for f in range(a.forks)]
        for v in KINDS:
            medians=[statistics.median(r['ns_per_call'] for r in rows if (r['fork'],r['case'],r['size'],r['order'],r['candidate'])==(f,case,n,order,v)) for f in range(a.forks)]
            summaries.append(dict(case=case,size=n,order=order,candidate=v,median_ns=statistics.median(medians),paired_ratio=statistics.median(x/y for x,y in zip(medians,reference)),fork_medians=json.dumps(medians)))
    save_csv(out/'native-latency.csv',summaries)
    if a.aa:
        (out/'provenance.json').write_text(json.dumps(dict(build=build,settings={'aa':True,'forks':a.forks,'batches':a.batches,'target_ms':a.target_ms},cpu=core,runner_sha256=sha(HERE/'study.py')),indent=2))
        print('Finished identical-library control',out,flush=True);return
    counts=[]
    for case,n,ordering in cells:
        for v in KINDS:
            d=builds[v+'-count'];r=command([builds['current-count']['driver'],'count',case,str(n),ordering,'3','1','verify'],out/'raw'/f'count-{case}-{n}-{ordering}-{v}.log',d['library'])
            values=[tuple(map(int,line.split(',')[2:])) for line in r.stdout.splitlines() if line.startswith('COUNT,')];assert len(values)==3 and len(set(values))==1
            counts.append(dict(case=case,size=n,order=ordering,candidate=v,allocations=values[0][0],reallocations=values[0][1],requested_bytes=values[0][2]))
    save_csv(out/'native-allocations.csv',counts)
    standalone=[];standalone_counts=[]
    for n in [0,1,8,128,512,8192,65536]:
        for shape in ['flat','singleton','groups8','shared','readonly']:
            for order in ['ascending','shuffled'] if n>8 else ['ascending']:
                for fork in range(a.forks):
                    variants=CHECKERS.copy();random.Random(str((n,shape,order,fork//2))).shuffle(variants)
                    if fork%2:variants.reverse()
                    for v in variants:
                        r=command(['taskset','-c',str(core),str(out/'time'),'time',v,str(n),shape,order,str(a.batches),str(a.target_ms)],out/'raw'/f'standalone-{n}-{shape}-{order}-{v}-{fork}.log')
                        values=[int(s.split(',')[3])/int(s.split(',')[2]) for s in r.stdout.splitlines() if s.startswith('TIME,')];assert len(values)==a.batches
                        standalone.append(dict(size=n,shape=shape,order=order,candidate=v,fork=fork,median_ns=statistics.median(values)))
                for v in CHECKERS:
                    r=command([str(out/'counts'),'count',v,str(n),shape,order],out/'raw'/f'standalone-count-{n}-{shape}-{order}-{v}.log')
                    values=[int(s) for s in r.stdout.strip().split(',')[1:]]
                    standalone_counts.append(dict(size=n,shape=shape,order=order,candidate=v,allocations=values[0],reallocations=values[1],requested_bytes=values[2],peak_heap_bytes=values[3]))
        print('standalone',n,flush=True)
    save_csv(out/'standalone-latency.csv',standalone);save_csv(out/'standalone-allocations.csv',standalone_counts)
    for d in builds.values():assert sha(d['library'])==d['library_sha256'] and sha(d['driver'])==d['driver_sha256']
    (out/'provenance.json').write_text(json.dumps(dict(build=build,settings={'forks':a.forks,'batches':a.batches,'target_ms':a.target_ms,'aa':a.aa},cpu=core,cpu_settings={name:p.read_text().strip() for name in ['scaling_governor','scaling_driver'] if (p:=Path(f'/sys/devices/system/cpu/cpu{core}/cpufreq/{name}')).exists()},rustc=subprocess.check_output(['rustc','-vV'],text=True),sources={str(p.relative_to(ROOT)):sha(p) for p in [HERE/'study.py',HERE/'standalone.rs',HERE/'prototypes.rs',HERE/'checks.c',ROOT/'prebindgen-runtime/src/input.rs']},files={name:sha(out/name) for name in ['native-latency.csv','native-allocations.csv','standalone-latency.csv','standalone-allocations.csv']}),indent=2))
    print('Finished',out,flush=True)
if __name__=='__main__':main()
