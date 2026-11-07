"""Extract generated input decoders and measure them with controlled pointer ranks.
The checks_once/no_alias variants are valid-input-only diagnostic perturbations;
production validation is unchanged. Raw samples belong outside the repository.
"""
from pathlib import Path
import subprocess,re,argparse,os,shutil,hashlib,json
parser=argparse.ArgumentParser(__doc__)
parser.add_argument("--checkout",type=Path,required=True)
parser.add_argument("--runtime-checkout",type=Path,required=True)
parser.add_argument("--output",type=Path,required=True)
parser.add_argument("--baseline-ref",default="b179b9888")
parser.add_argument("--candidate-ref",default="7bcb47540")
parser.add_argument("--target-dir",type=Path)
args=parser.parse_args()
root=args.checkout.resolve()
runtime=args.runtime_checkout.resolve()
project=args.output.resolve();project.mkdir(parents=True,exist_ok=False);(project/'src').mkdir()
before=subprocess.check_output(['git','show',args.baseline_ref+':examples/bench-bulk/generated/bulk.rs'],cwd=root,text=True)
after=subprocess.check_output(['git','show',args.candidate_ref+':examples/bench-bulk/generated/bulk.rs'],cwd=root,text=True)
def balanced(s,start):
 depth=0
 for i in range(start,len(s)):
  if s[i]=='{':depth+=1
  elif s[i]=='}':
   depth-=1
   if depth==0:return s[start:i+1]
 raise ValueError('braces')
def structs(s):
 return '\n'.join('#[repr(C)]\n'+s[m.start():s.index('{',m.start())]+balanced(s,s.index('{',m.start())) for m in re.finditer(r'struct __PbBulk_\w+\s*\{',s))
def body(s):
 start=s.index('fn Java_bench_bulk_JNINative_call_1nested_1owned_1handles_1echo_')
 a=s.index('let p0_w1_reader',start);b=s.index('let source_value',a)
 return s[a:b]
variants={ 'staged':body(before),'reread':body(after)}
base=variants['reread'];ptr=list(re.finditer(r'::prebindgen_runtime::validate_pointer\([^)]*\)\?;',base));assert len(ptr)==2
m=ptr[1];variants['pointer_once']=base[:m.start()]+base[m.end():]
base=variants['pointer_once'];reads=list(re.finditer(r'p0_w1_reader\s*\.read::<\s*i64,\s*>',base));assert len(reads)==2
m=reads[1];a=base.index('(',m.end());depth=1;i=a+1
while depth:
 if base[i]=='(':depth+=1
 elif base[i]==')':depth-=1
 i+=1
assert base[i]=='?'
arg=base[a+1:i-1].rstrip().rstrip(',')
variants['checks_once']=base[:m.start()]+'unsafe {::std::ptr::read_unaligned(p0_w1_bytes.as_ptr().add('+arg+').cast::<i64>())}'+base[i+1:]
for kind in ['staged','reread']:
 variants[kind+'_no_alias']=variants[kind]
mods=[]
for name,b in variants.items():
 history='SharedInputs' if name.endswith('_no_alias') else 'DynamicInputs'
 mods.append('mod '+name+' {use crate::bench_flat;'+structs(before if name.startswith('staged') else after)+'''
 #[inline(never)] pub fn decode(bytes:&[u8])->Result<Vec<Vec<bench_flat::InputHandle>>,String>{
 let counter=::core::cell::Cell::new(0usize); let __prebindgen_input_consumed=&counter;
 let mut __prebindgen_input_aliases=::prebindgen_runtime::'''+history+'''::default();
 let p0_w1_bytes=bytes;
 '''+b+''' Ok(__prebindgen_argument_0) }}''')
(project/'src/decoders.rs').write_text('\n'.join(mods))
(project/'Cargo.toml').write_text('''[package]
name="nested-decode-probe"
version="0.1.0"
edition="2021"
[dependencies]
prebindgen-runtime={path="'''+str(runtime/'prebindgen-runtime')+'''"}
prebindgen-jni-runtime={path="'''+str(runtime/'prebindgen-jni-runtime')+'''"}
[workspace]
''')

shutil.copyfile(Path(__file__).with_name('native_probe.rs'),project/'src/main.rs')
env=dict(os.environ,CARGO_BUILD_JOBS='1')
if args.target_dir:env['CARGO_TARGET_DIR']=str(args.target_dir.resolve())
subprocess.run(['cargo','build','--release','--offline','--manifest-path',str(project/'Cargo.toml')],env=env,check=True)
metadata=json.loads(subprocess.check_output(['cargo','metadata','--offline','--no-deps','--format-version=1','--manifest-path',str(project/'Cargo.toml')],env=env,text=True))
exe=Path(metadata['target_directory'])/'release/nested-decode-probe'
shutil.copy2(exe,project/'probe')
with (project/'samples.csv').open('w') as log:subprocess.run([str(project/'probe')],stdout=log,check=True)
(project/'provenance.json').write_text(json.dumps(dict(runtime_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=runtime,text=True).strip(),baseline=args.baseline_ref,candidate=args.candidate_ref,probe_sha256=hashlib.sha256((project/'probe').read_bytes()).hexdigest(),samples_sha256=hashlib.sha256((project/'samples.csv').read_bytes()).hexdigest()),indent=2)+'\n')
