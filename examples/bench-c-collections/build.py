#!/usr/bin/env python3
"""Freeze one C collection benchmark library and its C caller, without timing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess

FIXTURE = Path(__file__).resolve().parent
ROOT = FIXTURE.parents[1]
FILES = ['flat.rs', 'bindings.rs', 'allocations.rs', 'driver.c', 'build.py', 'Cargo.lock.snapshot']
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()


def prepare(checkout, output):
    project = output / 'fixture'
    (project / 'src').mkdir(parents=True)
    (project / 'flat/src').mkdir(parents=True)
    def deps(names):
        return '\n'.join(f'{name}={{path={json.dumps(str(checkout/name))}}}' for name in names)
    (project/'Cargo.toml').write_text('''[package]
name="c-collection-bench"
version="0.1.0"
edition="2021"
[lib]
crate-type=["cdylib"]
[features]
count-allocations=[]
[dependencies]
c-collection-flat={path="flat"}
konst="0.3.0"
''' + deps(['prebindgen-runtime']) + '''
[build-dependencies]
c-collection-flat={path="flat"}
syn={version="2",features=["full"]}
''' + deps(['prebindgen-c','prebindgen-declarations']) + '\n[workspace]\n')
    (project/'flat/Cargo.toml').write_text('''[package]
name="c-collection-flat"
version="0.1.0"
edition="2021"
[dependencies]
''' + deps(['prebindgen-proc-macro']) + '\n[build-dependencies]\n' + deps(['prebindgen']) + '\n')
    (project/'flat/build.rs').write_text('fn main(){prebindgen::init_prebindgen_out_dir();}\n')
    (project/'src/lib.rs').write_text('''#![allow(non_camel_case_types,unused_variables)]
include!(concat!(env!("OUT_DIR"),"/bindings.rs"));
mod allocations;
''')
    shutil.copyfile(FIXTURE/'flat.rs', project/'flat/src/lib.rs')
    shutil.copyfile(FIXTURE/'bindings.rs', project/'build.rs')
    shutil.copyfile(FIXTURE/'allocations.rs', project/'src/allocations.rs')
    if (FIXTURE/'Cargo.lock.snapshot').exists():
        shutil.copyfile(FIXTURE/'Cargo.lock.snapshot',project/'Cargo.lock')
    return project


def revision(path):
    dirty = subprocess.check_output(['git','status','--porcelain'],cwd=path,text=True)
    assert not dirty, f'Commit the benchmark and implementation before freezing: {path}\n{dirty}'
    return subprocess.check_output(['git','rev-parse','HEAD'],cwd=path,text=True).strip()


def build(args):
    checkout=args.checkout.resolve();output=args.output.resolve()
    output.mkdir(parents=True,exist_ok=True)
    assert not list(output.iterdir()), 'Use an empty output directory'
    revisions={str(p):revision(p) for p in {checkout,ROOT}}
    project=prepare(checkout,output)
    env=dict(os.environ,CARGO_TARGET_DIR=str(output/'target'))
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--offline','--locked','--format-version=1'],cwd=project,env=env))
    package=next(p['id'] for p in metadata['packages'] if p['name']=='c-collection-bench')
    command=['cargo','build','--offline','--locked','--release','--message-format=json']
    if args.count_allocations:command+=['--features','count-allocations']
    with (output/'cargo.jsonl').open('w') as log:subprocess.run(command,cwd=project,env=env,stdout=log,check=True)
    messages=[json.loads(l) for l in (output/'cargo.jsonl').read_text().splitlines()]
    out=next(Path(m['out_dir']) for m in messages if m.get('reason')=='build-script-executed' and m['package_id']==package)
    artifact=next(m for m in messages if m.get('reason')=='compiler-artifact' and m['package_id']==package and 'cdylib' in m['target']['kind'])
    assert artifact['profile']['opt_level']=='3' and not artifact['profile']['debug_assertions']
    library=output/'target/release/libc_collection_bench.so'
    assert library.is_file(), 'This benchmark currently targets Linux'
    driver=output/'driver'
    cc=shlex.split(os.environ.get('CC','cc'))
    compile_command=[*cc,'-std=gnu11','-O3','-fno-plt','-Wall','-Wextra','-Werror',str(FIXTURE/'driver.c'),'-I'+str(out),'-L'+str(library.parent),'-lc_collection_bench','-Wl,-rpath,'+str(library.parent),'-ldl','-o',str(driver)]
    subprocess.run(compile_command,check=True)
    assert revisions=={str(p):revision(p) for p in {checkout,ROOT}}
    configs=[]
    for parent in [project,*project.parents,Path(os.environ.get('CARGO_HOME',Path.home()/'.cargo'))]:
        for name in ['.cargo/config','.cargo/config.toml','config','config.toml']:
            path=parent/name
            if path.is_file():configs.append(path)
    report=dict(revisions=revisions,checkout=str(checkout),count_allocations=args.count_allocations,
                fixture_sha256={name:sha(FIXTURE/name) for name in FILES},
                rustc=subprocess.check_output(['rustc','-vV'],text=True),
                cc=subprocess.check_output([*cc,'--version'],text=True),compile_command=compile_command,
                cargo_command=command,cargo_profile=artifact['profile'],
                lock_sha256=sha(project/'Cargo.lock'),cargo_configs={str(p):sha(p) for p in set(configs)},
                profile_environment={k:v for k,v in os.environ.items() if k.startswith(('CARGO_PROFILE_','RUSTC')) or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTUP_TOOLCHAIN']},
                library=str(library),library_sha256=sha(library),driver=str(driver),driver_sha256=sha(driver),
                header_sha256=sha(out/'bindings.h'),bindings_sha256=sha(out/'bindings.rs'))
    (output/'build.json').write_text(json.dumps(report,indent=2)+'\n')
    destination=FIXTURE/'generated'
    for name in ['bindings.rs','bindings.h']:
        if args.check_generated:assert (destination/name).read_bytes()==(out/name).read_bytes(), f'{name} snapshot differs'
        if args.record_generated:
            destination.mkdir(exist_ok=True);shutil.copyfile(out/name,destination/name)
    print('Frozen:',output,flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(__doc__)
    p.add_argument('--checkout',type=Path,default=ROOT)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--count-allocations',action='store_true')
    p.add_argument('--record-generated',action='store_true')
    p.add_argument('--check-generated',action='store_true')
    a=p.parse_args()
    assert not (a.record_generated and a.check_generated)
    build(a)
