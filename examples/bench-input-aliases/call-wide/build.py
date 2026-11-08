#!/usr/bin/env python3
"""Build benchmark-only native alternatives from frozen generated C bindings."""
import argparse,hashlib,importlib.util,json,os,re,shutil,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
FIXTURE=ROOT/'examples/bench-c-collections'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()

def block_end(text,start):
    # Generated benchmark functions have no braces in literals/comments that
    # affect this scan. Assert parsed function identity and replacement counts.
    depth=0
    for i in range(start,len(text)):
        if text[i]=='{':depth+=1
        elif text[i]=='}':
            depth-=1
            if depth==0:return i+1
    raise AssertionError('unterminated block')

def function(text,name):
    match=re.search(r'pub unsafe extern "C" fn '+name+r'\b',text)
    assert match,name
    start=text.index('{',match.end());end=block_end(text,start)
    return match.start(),end,text[match.start():end]

def collect_flat(root,row,access):
    return f'''::prebindgen_runtime::validate_slice({root}.data as *const {row},{root}.len)?;
    for index in 0..{root}.len {{
        let pointer=unsafe {{(*({root}.data as *const {row}).add(index)).v0}};
        ::prebindgen_runtime::validate_pointer(pointer)?;
        addresses.push((pointer as usize,{str(access).lower()}));
    }}'''

def collect_nested(root,outer,inner,access,sized):
    count=f'''let mut capacity=0usize;
    for index in 0..{root}.len {{
        let child=unsafe {{(*({root}.data as *const {outer}).add(index)).v0}};
        ::prebindgen_runtime::validate_slice(child.data as *const {inner},child.len)?;
        capacity=capacity.checked_add(child.len).ok_or("address count overflow")?;
    }}
    addresses.reserve_exact(capacity);''' if sized else ''
    return f'''::prebindgen_runtime::validate_slice({root}.data as *const {outer},{root}.len)?;
    {count}
    for outer_index in 0..{root}.len {{
        let child=unsafe {{(*({root}.data as *const {outer}).add(outer_index)).v0}};
        {collect_flat('child',inner,access)}
    }}'''

def whole_call(checked,unchecked,sized):
    text=checked
    for kind in ['flat','nested','shared','callback']:
        name='bench_history_'+kind
        _,_,fn=function(unchecked,name)
        assert '__prebindgen_input_aliases' not in fn,name
        assert fn.count('let __prebindgen_input_consumed')==1,name
        if kind=='callback':
            root='callback_return_w1'
            # The failure cleanup guard and C row-release owner must already
            # exist when the preflight can reject a transferred result.
            guard=fn.index('let mut callback_return_cleanup')
            marker='let callback_outcome:'
            insertion=fn.index(marker,guard)
            assert fn.count(marker)==1 and fn.count('let mut callback_return_cleanup')==1
            assert fn.index('let callback_result = __PbCallbackResult') < guard < insertion
            outer=re.search(r'(__PbRow_callback_\w+_callback_return_w1)\b',fn).group(1)
            inner=re.search(r'(__PbRow_callback_\w+_callback_return_w5)\b',fn).group(1)
            scan=collect_nested(root,outer,inner,True,sized)
        else:
            insertion=fn.index('let __prebindgen_input_consumed')
            assert not any(token in fn[:insertion] for token in ['from_raw_parts','Box::from_raw','c_collection_flat::history_'])
            root='p0_w1';outer='__PbRow_history_'+kind+'_p0_w1'
            if kind=='flat':
                scan=collect_flat(root,outer,True).replace(')?;\n',f')?;\n    addresses.reserve_exact({root}.len);\n',1)
            else:scan=collect_nested(root,outer,'__PbRow_history_'+kind+'_p0_w5',kind!='shared',sized)
            if kind=='shared' and sized:scan=scan.replace('let mut capacity=0usize;','let mut capacity=1usize;')
        token=''
        if kind=='shared':token='::prebindgen_runtime::validate_pointer(p1_w1)?;'
        tail='addresses.push((p1_w1 as usize,true));' if kind=='shared' else ''
        preflight='{ let mut addresses=Vec::<(usize,bool)>::new();'+token+scan+tail+'experiment::check_sorted(&mut addresses)?;}\n'
        fn=fn[:insertion]+preflight+fn[insertion:]
        start,end,_=function(text,name);text=text[:start]+fn+text[end:]
    return text

def online_hash(checked):
    text=checked
    for kind in ['flat','nested','shared','callback']:
        name='bench_history_'+kind
        start,end,fn=function(text,name)
        fn,n=re.subn(r'::prebindgen_runtime::DynamicInputs(?:WithThreshold\s*::\s*<\s*128usize\s*,?\s*>)?\s*::default\(\)', 'experiment::OnlineHash::default()',fn)
        assert n==1,(name,n)
        groups=re.findall(r'let mut (\w+) = __prebindgen_input_aliases\s*\.begin_group\(([^;]+)\);',fn)
        assert groups,(name,'missing batches')
        for group,hint in groups:
            fn,n=re.subn(r'let mut '+group+r' = __prebindgen_input_aliases\s*\.begin_group\([^;]+\);','__prebindgen_input_aliases.reserve_group('+hint+');',fn)
            assert n==1
            fn,n=re.subn(group+r'\.push\(\(([^,]+),\s*(true|false)\)\);',r'__prebindgen_input_aliases.access(\1,\2)?;',fn)
            assert n>=1,(name,group)
            fn,n=re.subn(r'__prebindgen_input_aliases\s*\.accept_group\('+group+r'\)\?;','',fn)
            assert n==1,(name,'accept')
        assert 'begin_group' not in fn and 'accept_group' not in fn
        text=text[:start]+fn+text[end:]
    return text

def direct_hash(checked,unchecked):
    text=checked
    for kind in ['flat','nested','shared','callback']:
        name='bench_history_'+kind
        _,_,fn=function(unchecked,name)
        assert '__prebindgen_input_aliases' not in fn,name
        marker='let __prebindgen_input_consumed'
        assert fn.count(marker)==1,name
        at=fn.index(marker)
        fn=fn[:at]+'let mut aliases=experiment::OnlineHash::default();\n'+fn[at:]
        def pointer(match):
            value=match.group(1)
            exclusive=kind!='shared' or not value.startswith('p0_')
            return match.group(0)+f' aliases.access({value} as usize,{str(exclusive).lower()})?;'
        fn,n=re.subn(r'::prebindgen_runtime::validate_pointer\(\s*(\w+)\s*,?\s*\)\?;',pointer,fn)
        assert n==(2 if kind=='shared' else 1),(name,'pointer validation count changed')
        # This prototype forwards vector capacity hints, including outer hints.
        # The inner hint overwrites the outer hint before acquisition in these fixtures.
        fn,n=re.subn(r'(\w+\.reserve_exact\((\w+)\);)',r'aliases.reserve_group(\2); \1',fn)
        assert n==(1 if kind=='flat' else 2),(name,'reserve count changed')
        start,end,_=function(text,name);text=text[:start]+fn+text[end:]
    return text

def main():
    p=argparse.ArgumentParser(__doc__);p.add_argument('--output',required=True,type=Path)
    p.add_argument('--target-dir',type=Path,default=Path('/tmp/alias-role-target'))
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=True);assert not list(out.iterdir())
    # All variants use exactly the same compiler, fixtures, ABI and alignment.
    env=os.environ|{'CARGO_BUILD_JOBS':'1','PYTHONDONTWRITEBYTECODE':'1','RUSTFLAGS':'-C llvm-args=-align-all-functions=6'}
    # Loading the existing fixture helper must not create untracked cache files.
    sys.dont_write_bytecode=True
    spec=importlib.util.spec_from_file_location('base',FIXTURE/'build.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
    project=base.prepare(ROOT,out/'seed')
    original_build=(project/'build.rs').read_text()
    project_env=env|{'CARGO_TARGET_DIR':str(a.target_dir.resolve())}
    package='c-collection-bench'
    seeds={}
    for checked in [True,False]:
        project_env['BENCH_ALIAS_CHECKS']='1' if checked else '0'
        command=['cargo','build','--offline','--locked','--release','--message-format=json']
        process=subprocess.run(command,cwd=project,env=project_env,capture_output=True,text=True)
        (out/f'seed-{checked}.log').write_text(process.stderr);process.check_returncode()
        messages=[json.loads(line) for line in process.stdout.splitlines()]
        generated=next(Path(m['out_dir']) for m in messages if m.get('reason')=='build-script-executed' and m['package_id'].split('#')[0].endswith('/fixture'))
        seeds[checked]=(generated/'bindings.rs').read_text()
        if checked:shutil.copyfile(generated/'bindings.h',out/'bindings.h')
    assert seeds[True]!=seeds[False]
    assert not any('include!' in source for source in seeds.values())
    for kind in ['flat','nested','shared','callback']:
        _,_,body=function(seeds[True],'bench_history_'+kind)
        flags=set(re.findall(r'\.push\(\([^,]+,\s*(true|false)\)\)',body))
        assert flags==({'false'} if kind=='shared' else {'true'}),(kind,flags)
    variants={'current':seeds[True],'hash_grouped':online_hash(seeds[True]),'hash':direct_hash(seeds[True],seeds[False]),'sort':whole_call(seeds[True],seeds[False],False),'sort_sized':whole_call(seeds[True],seeds[False],True)}
    helper=(HERE/'prototypes.rs').read_text();(project/'src/prototypes.rs').write_text(helper)
    (project/'src/lib.rs').write_text('#![allow(non_camel_case_types,unused_variables,dead_code)]\ninclude!(concat!(env!("OUT_DIR"),"/bindings.rs"));\n#[path="prototypes.rs"]mod experiment;\nmod allocations;\n')
    # Generated source is frozen per candidate; no production generator changes.
    (project/'build.rs').write_text('fn main(){println!("cargo:rerun-if-changed=variant.rs");std::fs::copy("variant.rs",std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();}\n')
    builds={}
    for variant,source in variants.items():
        (project/'variant.rs').write_text(source)
        (out/f'{variant}.rs').write_text(source)
        for counted in [False,True]:
            directory=out/(variant+('-count' if counted else '-time'));directory.mkdir()
            command=['cargo','build','--offline','--locked','--release','--message-format=json']+(['--features','count-allocations'] if counted else [])
            process=subprocess.run(command,cwd=project,env=project_env,capture_output=True,text=True)
            (directory/'build.log').write_text(process.stderr + (process.stdout if process.returncode else ''));process.check_returncode()
            messages=[json.loads(line) for line in process.stdout.splitlines()]
            artifact=next(m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='c_collection_bench' and 'cdylib' in m['target']['kind'])
            library=directory/'libc_collection_bench.so';shutil.copyfile(next(n for n in artifact['filenames'] if n.endswith('.so')),library)
            driver=directory/'driver'
            subprocess.run(['cc','-std=gnu11','-O3','-fno-plt','-Wall','-Wextra','-Werror',str(FIXTURE/'history-driver.c'),'-I'+str(out),'-L'+str(directory),'-lc_collection_bench','-Wl,-rpath,'+str(directory),'-ldl','-o',str(driver)],check=True)
            generated=next(Path(m['out_dir']) for m in messages if m.get('reason')=='build-script-executed' and m['package_id'].split('#')[0].endswith('/fixture'))
            assert (generated/'bindings.rs').read_text()==source,variant
            symbols=subprocess.check_output(['nm','-D',str(library)],text=True)
            aligned=set()
            for line in symbols.splitlines():
                if re.search(r'\bbench_history_(flat|nested|shared|readonly|callback)$',line):
                    assert int(line.split()[0],16)%64==0,line
                    aligned.add(line.split()[-1])
            assert len(aligned)==5,aligned
            builds[variant+('-count' if counted else '-time')]={'library':str(library),'library_sha256':sha(library),'driver':str(driver),'driver_sha256':sha(driver),'rust_sha256':sha(out/f'{variant}.rs'),'profile':artifact['profile']}
            print('built',variant,'count' if counted else 'time',flush=True)
    for mode in ['time','count']:
        assert len({builds[v+'-'+mode]['library_sha256'] for v in variants})==len(variants),'candidate libraries must differ'
    (out/'builds.json').write_text(json.dumps({'builds':builds,'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'rustc':subprocess.check_output(['rustc','-vV'],text=True),'lock_sha256':sha(project/'Cargo.lock'),'runtime_sha256':sha(ROOT/'prebindgen-runtime/src/input.rs'),'dirty_paths':subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).splitlines(),'env':{k:env[k] for k in ['RUSTFLAGS','CARGO_BUILD_JOBS']},'hashes':{str(p.relative_to(ROOT)):sha(p) for p in [HERE/'build.py',HERE/'prototypes.rs',FIXTURE/'flat.rs',FIXTURE/'bindings.rs',FIXTURE/'history-driver.c',FIXTURE/'allocations.rs',FIXTURE/'Cargo.lock.snapshot']},'header_sha256':sha(out/'bindings.h')},indent=2))

if __name__=='__main__':main()
