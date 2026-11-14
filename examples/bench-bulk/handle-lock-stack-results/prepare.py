#!/usr/bin/env python3
"""Recompile Kotlin against a frozen native benchmark image; do not rebuild Rust."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / 'examples/bench-bulk'
p = argparse.ArgumentParser(__doc__)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--kotlinc', default=os.environ.get('KOTLINC', 'kotlinc'))
a = p.parse_args()
assert not subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True), 'commit all sources before preparing a measured build'
out = a.output.resolve()
out.mkdir(parents=True,exist_ok=True)
assert not list(out.iterdir()), 'use an empty output directory'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
before = json.loads((a.before/'build.json').read_text())
assert not before['native_allocations']
assert sha(before['jar']) == before['jar_sha256']
assert sha(before['library']) == before['native_sha256']
for name,digest in before['fixture_sha256'].items():
    assert sha(FIXTURE/name) == digest, name
for name,digest in before['rust_sources_sha256'].items():
    assert sha(FIXTURE/'generated'/name) == digest, name
sources = sorted((FIXTURE/'generated/kotlin').rglob('*.kt'))
hashes = {str(p.relative_to(FIXTURE/'generated/kotlin')): sha(p) for p in sources}
assert hashes.keys() == before['kotlin_sources_sha256'].keys()
assert all(hashes[k]==v or k.endswith('/NativeHandle.kt') for k,v in before['kotlin_sources_sha256'].items())
# Ensure these benchmark snapshots contain the actual production template.
for package in ['bench.plain','bench.bulk']:
    assert (FIXTURE/'generated/kotlin'/package.replace('.','/')/'NativeHandle.kt').read_text() == (ROOT/'prebindgen-jni/src/native_handle.kt').read_text().replace('PACKAGE',package)
jar = out/'bench.jar'
subprocess.run([a.kotlinc,*map(str,sources),str(FIXTURE/'Bench.kt'),str(FIXTURE/'AllocationBench.kt'),'-nowarn','-include-runtime','-d',str(jar)],check=True)
version = subprocess.run([a.kotlinc,'-version'],capture_output=True,text=True,check=True)
assert (version.stdout+version.stderr).strip() == before['kotlin']
probe = subprocess.check_output(['java','-Djava.library.path='+str(Path(before['library']).parent),'-cp',str(jar),'bench.allocations.AllocationBenchKt','--probe-instrumentation'],text=True).strip()
assert probe == before['instrumentation_probe']
info = dict(before)
info.update(checkout=str(ROOT),commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty_paths=[],kotlin_sources_sha256=hashes,jar=str(jar),jar_sha256=sha(jar),native_reused_from=dict(build=str(a.before.resolve()),commit=before['commit'],native_sha256=before['native_sha256']),preparation='Kotlin compiled from committed generated sources; native image and path_dependency_repositories describe the frozen baseline native build')
(out/'build.json').write_text(json.dumps(info,indent=2)+'\n')
print('Prepared Kotlin-only comparison:',out)
