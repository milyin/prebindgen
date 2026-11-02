"""Run one benchmark case and size per fresh JVM, with compilation/GC diagnostics."""
import argparse
import csv
import hashlib
import json
import os
import platform
import random
import re
import statistics
import subprocess
import time
import xml.etree.ElementTree as ET
from pathlib import Path

p = argparse.ArgumentParser(__doc__)
p.add_argument('--driver', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--forks', type=int, default=4)
p.add_argument('--warmup', type=int, default=256)
p.add_argument('--batches', type=int, default=24)
p.add_argument('--target-ms', type=int, default=50)
p.add_argument('--min-warmup-ms', type=int, default=3000)
p.add_argument('--compile-quiet-ms', type=int, default=1000)
p.add_argument('--max-warmup-ms', type=int, default=20000)
p.add_argument('--smoke', action='store_true')
p.add_argument('--pilot', action='store_true')
a = p.parse_args()
assert a.forks > 0 and min(a.warmup, a.batches, a.target_ms) > 0
assert 0 <= a.min_warmup_ms <= a.max_warmup_ms and a.compile_quiet_ms >= 0
root = a.output.resolve()
root.mkdir(parents=True, exist_ok=True)
assert not list(root.iterdir()), 'Use an empty output directory'
raw = root / 'raw'
raw.mkdir()
driver = json.loads((a.driver / 'driver.json').read_text())
info = driver['builds']
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
assert sha(driver['driver_jar']) == driver['driver_jar_sha256']
for d in info.values():
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
flags = ['-Xms256m', '-Xmx256m', '-XX:+UseG1GC', '-XX:ActiveProcessorCount=2',
         '-XX:+UnlockDiagnosticVMOptions', '-XX:+LogCompilation', '-XX:+PrintCommandLineFlags',
         '-XX:CompileCommand=exclude,BenchKt::main',
         '-XX:CompileCommand=dontinline,BenchKt::main$sample',
         '-XX:CompileCommand=option,BenchKt::main$sample,double,CompileThresholdScaling,0.01',
         '-XX:CompileCommand=exclude,BenchKt::main$gcCount',
         '-XX:CompileCommand=exclude,BenchKt::main$gcTime']
cells = [('bulk', case, size) for case in [
    'handles_input', 'owned_handles_roundtrip', 'owned_handle_records_roundtrip',
    'nested_owned_handles_roundtrip'] for size in [1, 64, 512]]
cells += [('bulk', 'control', 1), ('plain', 'control', 1)]
cells += [('plain', case, 512) for case in [
    'owned_handles_roundtrip', 'owned_handle_records_roundtrip',
    'nested_owned_handles_roundtrip', 'records_output']]
if a.smoke:
    cells = [('bulk', 'control', 1), ('bulk', 'nested_owned_handles_roundtrip', 64),
             ('plain', 'owned_handle_records_roundtrip', 1)]
if a.pilot:
    cells = [('bulk', 'control', 1), ('bulk', 'handles_input', 1),
             ('bulk', 'owned_handle_records_roundtrip', 512),
             ('bulk', 'nested_owned_handles_roundtrip', 512),
             ('plain', 'owned_handles_roundtrip', 512)]
env = dict(os.environ, PB_MIN_WARMUP_MS=str(a.min_warmup_ms),
           PB_COMPILE_QUIET_MS=str(a.compile_quiet_ms), PB_MAX_WARMUP_MS=str(a.max_warmup_ms))
samples, runs = [], []
for fork in range(a.forks):
    order = cells.copy()
    random.Random(20261102 + fork).shuffle(order)
    for mode, case, size in order:
        versions = ['before', 'after'] if fork % 2 == 0 else ['after', 'before']
        for version in versions:
            ident = f'{fork}-{mode}-{case}-{size}-{version}'
            d = info[version]
            jit, gc = raw / f'{ident}-jit.xml', raw / f'{ident}-gc.log'
            command = ['java', *flags, f'-XX:LogFile={jit}',
                       f'-Xlog:gc*,safepoint:file={gc}:uptimemillis,level,tags',
                       *(['-Xcheck:jni'] if a.smoke else []),
                       '-Djava.library.path=' + str(Path(d['library']).parent),
                       '-cp', driver['driver_jar'] + os.pathsep + d['jar'], 'BenchKt',
                       mode, str(size), str(a.warmup), str(a.batches), str(a.target_ms),
                       str(fork), '64', case]
            begin = time.time()
            r = subprocess.run(command, env=env, capture_output=True, text=True)
            (raw / f'{ident}.log').write_text(r.stdout + r.stderr)
            r.check_returncode()
            assert 'SINK,' in r.stdout and 'WARNING in native method' not in r.stdout + r.stderr
            diag, values, phases, warm = {}, {}, {}, None
            for line in r.stdout.splitlines():
                parts = line.split(',')
                if parts[0] == 'BENCH_WARMUP':
                    _, actual_case, actual_size, n, end, quiet, stable = parts
                    assert (actual_case, int(actual_size)) == (case, size)
                    warm = dict(batches=int(n), end_ms=int(end), quiet_ms=int(quiet), stable=stable == 'true')
                elif parts[0] == 'BENCH_PHASE':
                    _, actual_case, actual_size, phase, uptime, compilation = parts
                    assert (actual_case, int(actual_size)) == (case, size)
                    phases[phase] = dict(uptime_ms=int(uptime), compilation_ms=int(compilation))
                elif parts[0] == 'BENCH_DIAG':
                    _, actual_case, actual_size, batch, start, end, cpu, cb, ce, gb, ge, gtb, gte = parts
                    assert (actual_case, int(actual_size)) == (case, size)
                    diag[int(batch)] = dict(start_ms=int(start), end_ms=int(end), cpu_nanos=int(cpu),
                                           compilation_ms=int(ce)-int(cb), gc_count=int(ge)-int(gb), gc_ms=int(gte)-int(gtb))
                elif parts[0] == 'BULK_BENCH':
                    _, actual_mode, actual_case, actual_size, batch, iterations, nanos, allocated = parts
                    assert (actual_mode, actual_case, int(actual_size)) == (mode, case, size)
                    values[int(batch)] = dict(iterations=int(iterations), nanos=int(nanos), jvm_allocated_bytes=int(allocated))
            assert warm is not None and set(phases) == {'start', 'end'}
            assert set(values) == set(diag) == set(range(a.batches))
            for batch in values:
                s = dict(version=version, mode=mode, case=case, size=size, fork=fork, batch=batch,
                         **values[batch], **diag[batch])
                assert s['iterations'] > 0 and s['nanos'] > 0 and s['cpu_nanos'] > 0
                samples.append(s)
            lo, hi = phases['start']['uptime_ms'] / 1000, phases['end']['uptime_ms'] / 1000
            events = []
            for n in ET.parse(jit).getroot().iter('nmethod'):
                stamp = float(n.get('stamp', '-1'))
                if lo <= stamp <= hi:
                    events.append(dict(stamp=stamp, method=n.get('method', ''), compiler=n.get('compiler'), level=n.get('level')))
            relevant = [e for e in events if e['method'].startswith(('bench.', 'bench/', 'BenchKt', 'Case ', 'Operation '))]
            elapsed = time.time() - begin
            runs.append(dict(id=ident, version=version, mode=mode, case=case, size=size, fork=fork,
                             seconds=elapsed, command=command, warmup=warm, phases=phases,
                             effective_flags=[s for s in r.stdout.splitlines() if s.startswith("-XX:")],
                             compilations_during_measurement=events, binding_driver_compilations=relevant,
                             stdout_sha256=sha(raw / f'{ident}.log'), gc_sha256=sha(gc), jit_sha256=sha(jit)))
            (root / 'progress.json').write_text(json.dumps(dict(runs=runs, samples=samples)))
            print(f'{len(runs)}/{len(cells)*a.forks*2} {ident}: {elapsed:.1f}s, '
                  f'quiet={warm["stable"]}, target-compilations={len(relevant)}', flush=True)
summary = []
for mode, case, size in sorted(cells):
    row = dict(mode=mode, case=case, size=size)
    for version in info:
        means, cpus, bytes_, medians, forks, gcs = [], [], [], [], [], []
        for fork in range(a.forks):
            group = [s for s in samples if (s['version'], s['mode'], s['case'], s['size'], s['fork']) == (version, mode, case, size, fork)]
            assert len(group) == a.batches
            count = sum(s['iterations'] for s in group)
            mean = sum(s['nanos'] for s in group) / count
            cpu = sum(s['cpu_nanos'] for s in group) / count
            allocated = sum(s['jvm_allocated_bytes'] for s in group) / count
            means.append(mean); cpus.append(cpu); bytes_.append(allocated)
            median = statistics.median(s['nanos']/s['iterations'] for s in group)
            medians.append(median)
            gc_count, gc_ms = sum(s['gc_count'] for s in group), sum(s['gc_ms'] for s in group)
            gcs.append(gc_count)
            forks.append(dict(fork=fork, wall_ns=mean, cpu_ns=cpu, jvm_bytes=allocated,
                              batch_median_ns=median, gc_count=gc_count, gc_ms=gc_ms,
                              compilation_ms=sum(s['compilation_ms'] for s in group)))
        row[version] = dict(wall_ns=statistics.median(means), min_wall_ns=min(means), max_wall_ns=max(means),
                            cpu_ns=statistics.median(cpus), jvm_bytes=statistics.median(bytes_),
                            batch_median_ns=statistics.median(medians), forks=forks, total_gc=sum(gcs))
    row['wall_change_percent'] = (row['after']['wall_ns']/row['before']['wall_ns']-1)*100
    row['cpu_change_percent'] = (row['after']['cpu_ns']/row['before']['cpu_ns']-1)*100
    row['paired_wall_changes_percent'] = [(x['wall_ns']/y['wall_ns']-1)*100 for x,y in zip(row['after']['forks'],row['before']['forks'])]
    summary.append(row)
with (root / 'samples.csv').open('w') as f:
    w = csv.DictWriter(f, fieldnames=list(samples[0])); w.writeheader(); w.writerows(samples)
for d in info.values():
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
assert sha(driver['driver_jar']) == driver['driver_jar_sha256']
report = dict(driver=driver, settings=vars(a) | {'driver': str(a.driver), 'output': str(a.output)},
              cells=cells, jvm_flags=flags, warmup_environment={k:v for k,v in env.items() if k.startswith('PB_')},
              affinity=sorted(os.sched_getaffinity(0)), platform=platform.platform(),
              java_environment={k:os.environ[k] for k in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS','MALLOC_ARENA_MAX'] if k in os.environ},
              cpu=subprocess.run(['lscpu'],capture_output=True,text=True).stdout,
              cpu_quota=Path('/sys/fs/cgroup/cpu.max').read_text().strip() if Path('/sys/fs/cgroup/cpu.max').exists() else None,
              java=subprocess.run(['java','-version'],capture_output=True,text=True).stderr,
              runs=runs, summary=summary, runner_sha256=sha(__file__), samples_sha256=sha(root/'samples.csv'))
(root / 'results.json').write_text(json.dumps(report, indent=2)+'\n')
print('Finished:', root / 'results.json', flush=True)
