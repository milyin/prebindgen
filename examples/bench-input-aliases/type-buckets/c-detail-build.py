#!/usr/bin/env python3
"""Build additional C-only variants from the earlier frozen generated fixture."""
import argparse,hashlib,importlib.util,json,os,shutil,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(__doc__);p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--target-dir',type=Path,default=Path('/tmp/alias-role-jni-target'));a=p.parse_args()
 source=a.build.resolve();baseline=json.loads((source/'build.json').read_text())
 toolchain=subprocess.check_output(['rustc','-vV'],text=True);assert toolchain==baseline['rustc'],'toolchain differs from frozen baseline'
 root=HERE.parents[2]
 subprocess.run(['git','diff','--quiet',baseline['production_base'],'HEAD','--','prebindgen-*'],cwd=root,check=True)
 subprocess.run(['git','diff','--quiet','HEAD','--','prebindgen-*'],cwd=root,check=True)
 out=a.output.resolve();out.mkdir();project=out/'fixture';shutil.copytree(source/'fixture',project)
 spec=importlib.util.spec_from_file_location('bucket_build',HERE/'build.py');builder=importlib.util.module_from_spec(spec);spec.loader.exec_module(builder)
 checked=(source/'c_checked.rs').read_text();unchecked=(source/'c_unchecked.rs').read_text()
 shutil.copyfile(HERE/'c-algorithms.rs',project/'src/c-algorithms.rs')
 (project/'src/prototypes.rs').write_text((HERE/'c-prototypes.rs').read_text()+'\npub use crate::algorithms::{compact,pairs,stable};\n')
 with (project/'src/lib.rs').open('a') as f:f.write('\n#[path="c-algorithms.rs"]mod algorithms;\n')
 builds={v:json.loads((source/'build.json').read_text())['builds'][v+'-time']['library'] for v in ['flat','cross']}
 env=os.environ|{'CARGO_TARGET_DIR':str(a.target_dir.resolve()),'CARGO_BUILD_JOBS':'1','RUSTFLAGS':'-C llvm-args=-align-all-functions=6'}
 for v in ['compact','pairs','token','stable']:
  rust=builder.c_variant(checked,unchecked,'flat' if v in ['token','stable'] else 'cross')
  if v in ['compact','pairs']:
   assert rust.count('experiment::buckets(&mut groups,true)?;')==3
   rust=rust.replace('experiment::buckets(&mut groups,true)?;',f'experiment::{v}(&mut groups)?;')
  elif v=='stable':
   assert rust.count('experiment::sorted(&mut values)?;')==3
   rust=rust.replace('experiment::sorted(&mut values)?;','experiment::stable(&mut values)?;')
  else:
   symbol='buckets_mixed_shared';u,w=builder.fn(unchecked,symbol);f=unchecked[u:w]
   code=['::prebindgen_runtime::validate_pointer(p4_w1)?;let token=p4_w1 as usize;']
   for i in range(4):
    code.append(f'::prebindgen_runtime::validate_slice(p{i}_w1.data as *const __PbRow_mixed_shared_p{i}_w1,p{i}_w1.len)?;')
    code.append(f'for index in 0..p{i}_w1.len{{let pointer=unsafe{{(*(p{i}_w1.data as *const __PbRow_mixed_shared_p{i}_w1).add(index)).v0}};::prebindgen_runtime::validate_pointer(pointer)?;if pointer as usize==token{{return Err("conflicting input aliases".into());}}}}')
   marker='let __prebindgen_input_consumed';assert f.count(marker)==1;f=f.replace(marker,'{'+''.join(code)+'}'+marker)
   u,w=builder.fn(rust,symbol);rust=rust[:u]+f+rust[w:]
  (project/'variant.rs').write_text(rust);(out/(v+'.rs')).write_text(rust)
  r=subprocess.run(['cargo','build','--offline','--release','--message-format=json'],cwd=project,env=env,capture_output=True,text=True);(out/(v+'-build.log')).write_text(r.stdout+r.stderr);r.check_returncode()
  messages=[json.loads(l) for l in r.stdout.splitlines() if l.startswith('{')];artifact=next(m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='type_group_bench')
  dest=out/(v+'.so');shutil.copyfile(next(f for f in artifact['filenames'] if f.endswith('.so')),dest);builds[v]=str(dest);print('built',v,flush=True)
 shutil.copyfile(source/'bindings.h',out/'bindings.h')
 (out/'build.json').write_text(json.dumps(dict(libraries=builds,hashes={v:sha(p) for v,p in builds.items()},rustc=toolchain,baseline_commit=baseline['production_base'],production_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),production_diff_sha256=hashlib.sha256(subprocess.check_output(['git','diff',baseline['production_base'],'--','prebindgen-*'],cwd=root)).hexdigest(),sources={p.name:sha(p) for p in HERE.iterdir() if p.is_file()}),indent=2))
if __name__=='__main__':main()
