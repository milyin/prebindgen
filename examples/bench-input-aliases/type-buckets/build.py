#!/usr/bin/env python3
"""Build one native fixture and bucketed/flat Kotlin surfaces; no production edits."""
import argparse,hashlib,json,os,re,shutil,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];BASE=ROOT/'examples/bench-handle-inputs'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def call(cmd,cwd,env,log):
 r=subprocess.run(cmd,cwd=cwd,env=env,capture_output=True,text=True);log.write_text(r.stdout+r.stderr);r.check_returncode();return r.stdout

def end_block(s,start):
 depth=0
 for i in range(start,len(s)):
  if s[i]=='{':depth+=1
  elif s[i]=='}':
   depth-=1
   if depth==0:return i+1
 raise AssertionError()
def fn(s,name):
 a=s.index('pub unsafe extern "C" fn '+name+'(');b=s.index('{',a);return a,end_block(s,b)
def c_variant(checked,unchecked,kind):
 if kind=='current':return checked
 for name in ['mut_one','mixed_mut','mixed_shared']:
  symbol='buckets_'+name;a,b=fn(unchecked,symbol);f=unchecked[a:b]
  assert '__prebindgen_input_aliases' not in f
  marker='let __prebindgen_input_consumed';assert f.count(marker)==1
  code=[];shared=name=='mixed_shared';count=1 if name=='mut_one' else 4
  if kind=='typed' and shared:pass # Conditional independence: no type has conflicting accesses.
  else:
   code.append('let mut values=Vec::<(usize,bool)>::new();' if kind=='flat' else 'let mut groups:[Vec<(usize,bool)>;5]=std::array::from_fn(|_|Vec::new());')
   for i in range(count):
    code.append(f'::prebindgen_runtime::validate_slice(p{i}_w1.data as *const __PbRow_{name}_p{i}_w1,p{i}_w1.len)?;')
   if kind=='flat':code.append('values.reserve_exact('+ '+'.join(f'p{i}_w1.len' for i in range(count))+(' + 1' if shared else '')+');')
   for i in range(count):
    root=f'p{i}_w1';row=f'__PbRow_{name}_{root}';push='values' if kind=='flat' else f'groups[{i}]'
    code.append(f'''
    {'' if kind=='flat' else f'{push}.reserve_exact({root}.len);'}
    for index in 0..{root}.len{{let pointer=unsafe{{(*({root}.data as *const {row}).add(index)).v0}};::prebindgen_runtime::validate_pointer(pointer)?;{push}.push((pointer as usize,{str(not shared).lower()}));}}''')
   if shared:
    dest='values' if kind=='flat' else 'groups[4]'
    code.append(f'::prebindgen_runtime::validate_pointer(p4_w1)?;{dest}.push((p4_w1 as usize,true));')
   code.append('experiment::sorted(&mut values)?;' if kind=='flat' else f'experiment::buckets(&mut groups,{str(kind=="cross").lower()})?;')
  offset=f.index(marker);f=f[:offset]+'{'+''.join(code)+'}\n'+f[offset:]
  a,b=fn(checked,symbol);checked=checked[:a]+f+checked[b:]
 return checked

def main():
 p=argparse.ArgumentParser(__doc__);p.add_argument('--output',required=True,type=Path);p.add_argument('--resume',action='store_true');p.add_argument('--no-count',action='store_true');p.add_argument('--variants',nargs='+',choices=['current','flat','typed','cross'],default=['current','flat','typed','cross']);p.add_argument('--target-dir',type=Path,default=Path('/tmp/alias-role-jni-target'));a=p.parse_args()
 out=a.output.resolve();out.mkdir(parents=True,exist_ok=a.resume);project=out/'fixture';
 if not a.resume:
  (project/'flat/src').mkdir(parents=True);(project/'src').mkdir()
  deps=lambda names:'\n'.join(f'{n}={{path={json.dumps(str(ROOT/n))}}}' for n in names)
  (project/'Cargo.toml').write_text('''[package]
 name="type-group-bench"
 version="0.1.0"
 edition="2021"
 [lib]
 name="type_group_bench"
 crate-type=["cdylib"]
 [features]
 count-allocations=[]
 [dependencies]
 type-group-flat={path="flat"}
 konst="0.3.0"
 '''+deps(['prebindgen-runtime','prebindgen-jni-runtime'])+'''
 [build-dependencies]
 type-group-flat={path="flat"}
 syn={version="2",features=["full"]}
 '''+deps(['prebindgen-jni','prebindgen-declarations','prebindgen-c'])+'\n[workspace]\n')
  (project/'flat/Cargo.toml').write_text('[package]\nname="type-group-flat"\nversion="0.1.0"\nedition="2021"\n[dependencies]\n'+deps(['prebindgen-proc-macro'])+'\n[build-dependencies]\n'+deps(['prebindgen'])+'\n')
  shutil.copyfile(HERE/'flat.rs',project/'flat/src/lib.rs');(project/'flat/build.rs').write_text('fn main(){prebindgen::init_prebindgen_out_dir();}\n')
  script=(BASE/'build.rs').read_text().replace('out.join("bindings.rs")','out.join("jni.rs")');script=script.replace('    generated.write_kotlin(&out.join("kotlin")).unwrap();','    generated.write_kotlin(&out.join("kotlin")).unwrap();\n    c_bindings(&out);')+'\n'+(HERE/'c-build.rs').read_text()
  (project/'build.rs').write_text(script);(project/'src/lib.rs').write_text('#![allow(non_camel_case_types,unused_variables,dead_code)]\nmod jni{include!(concat!(env!("OUT_DIR"),"/jni.rs"));}\nmod c{use crate::experiment;include!(concat!(env!("OUT_DIR"),"/c_variant.rs"));}\n#[path="prototypes.rs"]mod experiment;\nmod allocations;\n')
  shutil.copyfile(HERE/'c-prototypes.rs',project/'src/prototypes.rs');shutil.copyfile(ROOT/'examples/bench-c-collections/allocations.rs',project/'src/allocations.rs');shutil.copyfile(ROOT/'examples/bench-bulk/Cargo.lock.snapshot',project/'Cargo.lock')
  # First build only prepares generation; Cargo must see a Rust module to include.
  (project/'src/lib.rs').write_text('#![allow(non_camel_case_types,unused_variables)]\nmod jni{include!(concat!(env!("OUT_DIR"),"/jni.rs"));}\n')
  env=os.environ|{'CARGO_TARGET_DIR':str(a.target_dir.resolve()),'CARGO_BUILD_JOBS':'1','RUSTFLAGS':'-C llvm-args=-align-all-functions=6'}
  raw=call(['cargo','build','--offline','--release','--message-format=json'],project,env,out/'seed.log');messages=[json.loads(s) for s in raw.splitlines() if s.startswith('{')];gen=next(Path(m['out_dir']) for m in messages if m.get('reason')=='build-script-executed' and m['package_id'].split('#')[0].endswith('/fixture'))
  for n in ['c_checked.rs','c_unchecked.rs','jni.rs','bindings.h']:shutil.copyfile(gen/n,out/n)
  shutil.copytree(gen/'kotlin',out/'kotlin-buckets')
 else:
  env=os.environ|{'CARGO_TARGET_DIR':str(a.target_dir.resolve()),'CARGO_BUILD_JOBS':'1','RUSTFLAGS':'-C llvm-args=-align-all-functions=6'}
  shutil.copyfile(HERE/'c-prototypes.rs',project/'src/prototypes.rs')
 checked=(out/'c_checked.rs').read_text();unchecked=(out/'c_unchecked.rs').read_text();assert checked!=unchecked
 # Freeze generated files; all C variants share the identical JNI module.
 (project/'jni.rs').write_text((out/'jni.rs').read_text());(project/'build.rs').write_text('fn main(){println!("cargo:rerun-if-changed=variant.rs");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());std::fs::copy("variant.rs",out.join("c_variant.rs")).unwrap();std::fs::copy("jni.rs",out.join("jni.rs")).unwrap();}\n')
 (project/'src/lib.rs').write_text('#![allow(non_camel_case_types,unused_variables,dead_code)]\nmod jni{include!(concat!(env!("OUT_DIR"),"/jni.rs"));}\nmod c{use crate::experiment;include!(concat!(env!("OUT_DIR"),"/c_variant.rs"));}\n#[path="prototypes.rs"]mod experiment;\nmod allocations;\n')
 builds=json.loads((out/'build.json').read_text())['builds'] if a.resume and (out/'build.json').exists() else {}
 for kind in a.variants:
  source=c_variant(checked,unchecked,kind);(project/'variant.rs').write_text(source);(out/(kind+'.rs')).write_text(source)
  for counted in ([False] if a.no_count else [False,True]):
   dest=out/(kind+('-count' if counted else '-time'));dest.mkdir(exist_ok=a.resume);cmd=['cargo','build','--offline','--release','--message-format=json']+(['--features','count-allocations'] if counted else [])
   raw=call(cmd,project,env,dest/'build.log');messages=[json.loads(s) for s in raw.splitlines() if s.startswith('{')];artifact=next(m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='type_group_bench')
   lib=dest/'libtype_group_bench.so';shutil.copyfile(next(f for f in artifact['filenames'] if f.endswith('.so')),lib)
   builds[kind+('-count' if counted else '-time')]={'library':str(lib),'sha256':sha(lib)};print('built',kind,counted,flush=True)
 for suffix in ['-time','-count']:
  hashes=[builds[k+suffix]['sha256'] for k in ['current','flat','typed','cross'] if k+suffix in builds]
  assert len(set(hashes))==len(hashes),'stale or identical native variants'
 subprocess.run(['git','diff','--quiet','HEAD','--','prebindgen-*'],cwd=ROOT,check=True)
 # Kotlin compilation is performed sequentially by kotlin.py.
 (out/'build.json').write_text(json.dumps({'builds':builds,'production_base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'rustc':subprocess.check_output(['rustc','-vV'],text=True),'sources':{p.name:sha(p) for p in HERE.iterdir() if p.is_file()},'lock_sha256':sha(project/'Cargo.lock'),'external_sources':{str(p.relative_to(ROOT)):sha(p) for p in [BASE/'build.rs',ROOT/'examples/bench-c-collections/allocations.rs']},'production_diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--','prebindgen-*'],cwd=ROOT)).hexdigest()},indent=2))
if __name__=='__main__':main()
