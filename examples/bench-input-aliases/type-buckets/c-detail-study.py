#!/usr/bin/env python3
"""Decompose C alias checking, then confirm with generated native calls."""
import argparse,csv,hashlib,json,os,random,shutil,statistics,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
MICRO=['flat','original','no_local','compact','pairs','token','stable'];NATIVE=['flat','cross','compact','pairs','token','stable']
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(path,rows):
 if not rows:return
 with path.open('w') as f:w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');w.writeheader();w.writerows(rows)
def summary(rows,keys):
 out=[]
 for cell in sorted({tuple(r[k] for k in keys) for r in rows}):
  part=[r for r in rows if tuple(r[k] for k in keys)==cell];variants=sorted({r['variant'] for r in part});forks=sorted({r['fork'] for r in part})
  med={v:[statistics.median(r['ns'] for r in part if r['variant']==v and r['fork']==f) for f in forks] for v in variants}
  for v in variants:
   row=dict(zip(keys,cell))|dict(variant=v,median_ns=statistics.median(med[v]),fork_medians=json.dumps(med[v]))
   if 'stage' not in keys:
    ratios=[x/y for x,y in zip(med[v],med['flat'])];row.update(ratio_to_flat=statistics.median(ratios),min_ratio=min(ratios),max_ratio=max(ratios))
   out.append(row)
 return out

def main():
 p=argparse.ArgumentParser(__doc__);p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--forks',type=int,default=3);p.add_argument('--target-ms',type=int,default=8);p.add_argument('--spatial-only',action='store_true');p.add_argument('--aa',action='store_true');p.add_argument('--verify-only',action='store_true');a=p.parse_args()
 build=a.build.resolve();out=a.output.resolve();out.mkdir();(out/'raw').mkdir();core=max(os.sched_getaffinity(0));b=json.loads((build/'build.json').read_text());libs=[b['libraries'][v] for v in NATIVE];frozen={p:sha(p) for p in libs};sources={p.name:sha(p) for p in HERE.iterdir() if p.is_file()}
 rust=['rustc','--edition=2021','-O','-C','llvm-args=-align-all-functions=6',str(HERE/'c-detail.rs')]
 subprocess.run(rust+['-o',str(out/'checker')],check=True)
 subprocess.run(rust+['--cfg','feature="count-allocations"','-o',str(out/'count')],check=True)
 subprocess.run(['cc','-O3','-Wall','-Wextra','-Werror','-I',str(build),str(HERE/'c-detail-driver.c'),'-ldl','-o',str(out/'native')],check=True)
 def run(cmd,name):
  r=subprocess.run(['taskset','-c',str(core),*map(str,cmd)],capture_output=True,text=True);(out/'raw'/(name+'.log')).write_text(r.stdout+r.stderr);r.check_returncode();return r.stdout
 checks=[run([out/'checker','--check'],'checker-checks'),run([out/'native',1,0,'sorted','interleaved',1,1,0,'ab',*libs],'native-checks')]
 if a.verify_only:
  checks.append(run([out/'checker',128,4,'shuffled','mutable',0,1,1,'ab'],'checker-smoke'))
  for layout in ['random_sorted','random_shuffled']:checks.append(run([out/'checker',128,4,layout,'mutable',0,1,1,'ab'],'checker-'+layout))
  checks.append(run([out/'native',2,128,'creation','fragmented',1,1,0,'ab',*libs],'native-smoke'))
  (out/'verification.json').write_text(json.dumps(dict(checks=checks,files=frozen,sources=sources),indent=2));return
 if a.aa:
  libs=[]
  for i in range(6):path=out/f'flat-{i}.so';shutil.copyfile(b['libraries']['flat'],path);libs.append(str(path))
 micro=[];stages=[];counts=[];native=[];shapes=[];alloc=[]
 mcells=[(n,4,layout,access) for n in [128,8192,65536,262144,1048576] for layout in ['striped','blocked','shuffled','near','reverse'] for access in ['mutable','shared']]
 mcells += [(65536,k,layout,'mutable') for k in [1,2] for layout in ['striped','blocked','shuffled']]
 ncells=[(access,n,layout,'interleaved') for access in [1,2] for n in [8192,65536] for layout in ['creation','sorted','shuffled','near']]
 ncells += [(1,65536,layout,allocator) for allocator in ['by_type','fragmented'] for layout in ['creation','shuffled']]
 ncells += [(1,262144,layout,'interleaved') for layout in ['sorted','shuffled']]
 if a.spatial_only:
  mcells=[(n,4,layout,'mutable') for n in [65536,1048576] for layout in ['striped','shuffled','random_sorted','random_shuffled']];ncells=[]
 if a.aa:mcells=[(65536,4,'striped','mutable'),(65536,4,'shuffled','mutable'),(65536,4,'shuffled','shared')];ncells=[(1,65536,'sorted','interleaved'),(1,65536,'shuffled','interleaved'),(2,65536,'shuffled','interleaved')]
 for fork in range(a.forks):
  cells=mcells.copy();random.Random(71+fork).shuffle(cells)
  for n,k,layout,access in cells:
   cell=dict(size=n,groups=k,layout=layout,access=access,fork=fork);raw=run([out/'checker',n,k,layout,access,fork,7,a.target_ms,'aa' if a.aa else 'ab'],f'micro-{fork}-{n}-{k}-{layout}-{access}')
   for line in raw.splitlines():
    v=line.split(',')
    if v[0]=='TIME':micro.append(cell|dict(variant=MICRO[int(v[1])],ns=int(v[4])/int(v[3])))
    elif v[0]=='STAGES':
     for i,stage in enumerate(['allocation_copy','sort','local_scan','cross_check','free']):stages.append(cell|dict(variant=MICRO[int(v[1])],stage=stage,ns=int(v[3+i])/int(v[2])))
    elif v[0]=='COUNTS':counts.append(cell|dict(flat_comparisons=int(v[1]),bucket_comparisons=int(v[2]),active_groups=int(v[3]),stable_comparisons=int(v[4])))
   print('micro',fork,n,k,layout,access,flush=True)
  cells=ncells.copy();random.Random(83+fork).shuffle(cells)
  for access,n,layout,allocator in cells:
   cell=dict(access=['','mutable','shared'][access],size=n,layout=layout,allocator=allocator,fork=fork);raw=run([out/'native',access,n,layout,allocator,7,a.target_ms,fork,'aa' if a.aa else 'ab',*libs],f'native-{fork}-{access}-{n}-{layout}-{allocator}')
   assert len([l for l in raw.splitlines() if l.startswith('LIBRARY,')])==6
   for line in raw.splitlines():
    v=line.split(',')
    if v[0]=='TIME':native.append(cell|dict(variant=NATIVE[int(v[1])],ns=int(v[4])/int(v[3])))
    elif v[0]=='INPUT':shapes.append(cell|dict(group=int(v[1]),length=int(v[2]),descending_edges=int(v[3]),global_descending_edges='',overlapping_type_ranges=''))
    elif v[0]=='SHAPE':shapes.append(cell|dict(group=-1,length=n,descending_edges='',global_descending_edges=int(v[1]),overlapping_type_ranges=int(v[2])))
   print('native',fork,access,n,layout,allocator,flush=True)
 if not a.aa:
  for n in [65536,1048576]:
   for access in ['mutable','shared']:
    raw=run([out/'count',n,4,'shuffled',access,0,1,1,'ab'],f'count-{n}-{access}')
    for line in raw.splitlines():
     if line.startswith('ALLOC,'):
      _,v,allocations,reallocations,requested=line.split(',');alloc.append(dict(size=n,access=access,variant=MICRO[int(v)],allocations=int(allocations),reallocations=int(reallocations),requested_bytes=int(requested)))
 save(out/'micro.csv',summary(micro,['size','groups','layout','access']));save(out/'native.csv',summary(native,['size','layout','allocator','access']));save(out/'stages.csv',[{k:v for k,v in row.items() if k not in ['ratio_to_flat','min_ratio','max_ratio']} for row in summary(stages,['size','groups','layout','access','stage'])]);save(out/'counts.csv',counts);save(out/'input-shapes.csv',shapes);save(out/'allocations.csv',alloc)
 assert all(sha(p)==v for p,v in frozen.items())
 (out/'provenance.json').write_text(json.dumps(dict(build=b,files=frozen,sources=sources,forks=a.forks,batches=7,target_ms=a.target_ms,cpu=core,aa=a.aa,rustc=subprocess.check_output(['rustc','-vV'],text=True),checks=[x[:2000] for x in checks],note='Stage timers are separate diagnostics; numeric microbenchmark addresses are never dereferenced.'),indent=2))
if __name__=='__main__':main()
