"""Compare prebuilt baseline and optimized collection-input bindings."""
import argparse
import csv
import hashlib
import json
import os
import platform
import statistics
import subprocess
import time
from pathlib import Path

p = argparse.ArgumentParser(__doc__)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--after', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = a.output
root.mkdir(parents=True, exist_ok=True)
assert not list(root.iterdir()), 'Use an empty output directory'
info = {key: json.loads((path / 'build.json').read_text())
        for key, path in [('before', a.before), ('after', a.after)]}
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
for field in ['fixture_sha256', 'rustc', 'kotlin', 'dependency_lock_sha256',
              'cargo_config_sha256', 'cargo_profile', 'profile_environment']:
    assert info['before'][field] == info['after'][field], field
for d in info.values():
    assert not d['dirty_paths']
    assert sha(d['jar']) == d['jar_sha256']
    assert sha(d['library']) == d['native_sha256']
flags = ['-Xms256m', '-Xmx256m', '-XX:+UseG1GC',
         '-XX:ActiveProcessorCount=2', '-XX:+PrintCommandLineFlags']
cases = 'control,record_input,numbers_input,records_input,handles_input,handle_records_input,owned_handles_roundtrip'
orders = [['before-plain', 'after-plain', 'after-bulk', 'before-bulk'],
          ['after-plain', 'before-bulk', 'before-plain', 'after-bulk'],
          ['before-bulk', 'after-bulk', 'after-plain', 'before-plain'],
          ['after-bulk', 'before-plain', 'before-bulk', 'after-plain']]
samples = []
runs = []
for trial, order in enumerate(orders):
    for treatment in order:
        version, mode = treatment.split('-')
        d = info[version]
        command = ['java', *flags, '-Djava.library.path=' + str(Path(d['library']).parent),
                   '-jar', d['jar'], mode, '0,1,8,64,512', '6', '9', '20',
                   str(trial), '64,4096,65536', cases]
        start = time.time()
        result = subprocess.run(command, capture_output=True, text=True)
        (root / f'jvm-{trial}-{treatment}.log').write_text(result.stdout + result.stderr)
        result.check_returncode()
        assert 'SINK,' in result.stdout
        assert 'WARNING in native method' not in result.stdout + result.stderr
        count = 0
        for line in result.stdout.splitlines():
            if not line.startswith('BULK_BENCH,'):
                continue
            _, actual, case, size, batch, iterations, nanos, allocated = line.split(',')
            assert actual == mode
            samples.append(dict(version=version, mode=mode, case=case, size=int(size),
                                trial=trial, batch=int(batch), iterations=int(iterations),
                                nanos=int(nanos), jvm_allocated_bytes=int(allocated)))
            count += 1
        assert count == 27 * 9, (treatment, count)
        runs.append(dict(trial=trial, treatment=treatment,
                         seconds=time.time() - start, command=command))
        print(f'Round {trial + 1}/4 {treatment}: {time.time() - start:.1f}s', flush=True)
        (root / 'progress.json').write_text(json.dumps(samples))
summary = []
for mode, case, size in sorted({(s['mode'], s['case'], s['size']) for s in samples}):
    row = dict(mode=mode, case=case, size=size)
    for version in info:
        medians, allocated = [], []
        for trial in range(4):
            group = [s for s in samples if
                     (s['mode'], s['case'], s['size'], s['version'], s['trial']) ==
                     (mode, case, size, version, trial)]
            assert len(group) == 9
            medians.append(statistics.median(s['nanos'] / s['iterations'] for s in group))
            allocated.append(statistics.median(s['jvm_allocated_bytes'] / s['iterations'] for s in group))
        row[version] = dict(ns=statistics.median(medians), min_ns=min(medians),
                            max_ns=max(medians), trial_medians=medians,
                            jvm_bytes=statistics.median(allocated))
    row['speedup'] = row['before']['ns'] / row['after']['ns']
    row['latency_change_percent'] = (row['after']['ns'] / row['before']['ns'] - 1) * 100
    row['paired_speedups'] = [x / y for x, y in zip(row['before']['trial_medians'], row['after']['trial_medians'])]
    summary.append(row)
with (root / 'samples.csv').open('w') as f:
    writer = csv.DictWriter(f, fieldnames=list(samples[0]))
    writer.writeheader()
    writer.writerows(samples)
report = dict(builds=info, affinity=sorted(os.sched_getaffinity(0)), platform=platform.platform(),
              cpu=subprocess.run(['lscpu'], capture_output=True, text=True).stdout,
              java=subprocess.run(['java', '-version'], capture_output=True, text=True).stderr,
              jvm_flags=flags, orders=runs, warmup=6, batches=9, batch_target_ms=20,
              samples_sha256=sha(root / 'samples.csv'), runner_sha256=sha(__file__), summary=summary)
(root / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
for d in info.values():
    assert sha(d['jar']) == d['jar_sha256'] and sha(d['library']) == d['native_sha256']
print('Comparison complete: ' + str(root / 'results.json'), flush=True)
