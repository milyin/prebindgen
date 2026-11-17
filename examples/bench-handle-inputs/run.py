#!/usr/bin/env python3
"""Compare two unmodified generators through their generated JNI/Kotlin wrappers.

Builds and raw measurements stay in --output, outside the source tree. Each JVM
checks native results and closes every fixture owner, including roundtrip results.
"""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import statistics
import subprocess

HERE = Path(__file__).resolve().parent


def run(command, *, log, **kwargs):
    completed = subprocess.run(command, text=True, capture_output=True, **kwargs)
    log.write_text(completed.stdout + completed.stderr)
    completed.check_returncode()
    return completed.stdout


def revision(repo):
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
    diff = subprocess.check_output(['git', 'diff', 'HEAD'], cwd=repo)
    return {'commit': head, 'dirty_diff_sha256': hashlib.sha256(diff).hexdigest() if diff else None}


def build(repo, destination, args):
    project = destination / 'fixture'
    (project / 'flat' / 'src').mkdir(parents=True, exist_ok=True)
    (project / 'src').mkdir(exist_ok=True)

    def deps(names):
        return '\n'.join(f'{name}={{path={json.dumps(str(repo / name))}}}' for name in names)

    (project / 'Cargo.toml').write_text('''[package]
name="type-group-bench"
version="0.1.0"
edition="2021"
[lib]
name="type_group_bench"
crate-type=["cdylib"]
[dependencies]
type-group-flat={path="flat"}
konst="0.3.0"
''' + deps(['prebindgen-runtime', 'prebindgen-jni-runtime']) + '''
[build-dependencies]
type-group-flat={path="flat"}
''' + deps(['prebindgen-jni', 'prebindgen-declarations']) + '\n[workspace]\n')
    (project / 'flat' / 'Cargo.toml').write_text('''[package]
name="type-group-flat"
version="0.1.0"
edition="2021"
[dependencies]
''' + deps(['prebindgen-proc-macro']) + '\n[build-dependencies]\n' + deps(['prebindgen']) + '\n')
    shutil.copy2(HERE / 'flat.rs', project / 'flat' / 'src' / 'lib.rs')
    shutil.copy2(HERE / 'build.rs', project / 'build.rs')
    (project / 'flat' / 'build.rs').write_text('fn main(){prebindgen::init_prebindgen_out_dir();}\n')
    (project / 'src' / 'lib.rs').write_text('#![allow(non_camel_case_types,unused_variables)]\ninclude!(concat!(env!("OUT_DIR"),"/bindings.rs"));\n')
    shutil.copy2(repo / 'examples' / 'bench-bulk' / 'Cargo.lock.snapshot', project / 'Cargo.lock')
    target = destination / 'target'
    command = ['cargo', 'build', '--offline', '--release', '--message-format=json']
    source = run(command, cwd=project, env=os.environ | {'CARGO_TARGET_DIR': str(target), 'CARGO_BUILD_JOBS': '2'}, log=destination / 'build.log')
    messages = [json.loads(line) for line in source.splitlines() if line.startswith('{')]
    package = next(m['package_id'] for m in messages if m.get('reason') == 'compiler-artifact' and m['target']['name'] == 'type_group_bench')
    out = Path(next(m['out_dir'] for m in messages if m.get('reason') == 'build-script-executed' and m['package_id'] == package))
    kotlin = sorted((out / 'kotlin').rglob('*.kt'))
    shutil.copytree(out / 'kotlin', destination / 'kotlin', dirs_exist_ok=True)
    shutil.copy2(out / 'bindings.rs', destination / 'bindings.rs')
    library = destination / 'libtype_group_bench.so'
    shutil.copy2(target / 'release' / library.name, library)
    jar = destination / 'bench.jar'
    run([args.kotlinc, *map(str, kotlin), str(HERE / 'Bench.kt'), '-include-runtime', '-nowarn', '-d', str(jar)],
        env=os.environ | {'JAVA_OPTS': '-Xms128m -Xmx1g'}, log=destination / 'kotlin.log')
    print('built', destination.name, flush=True)
    return {'repository': str(repo), **revision(repo), 'library_sha256': hashlib.sha256(library.read_bytes()).hexdigest(),
            'jar_sha256': hashlib.sha256(jar.read_bytes()).hexdigest(), 'build_command': command}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--java', default='java')
    parser.add_argument('--kotlinc', default='kotlinc')
    parser.add_argument('--sizes', default='1,128,8192,32768')
    parser.add_argument('--forks', type=int, default=4)
    parser.add_argument('--skip-build', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    repos = {'baseline': args.baseline.resolve(), 'candidate': args.candidate.resolve()}
    if not args.skip_build:
        builds = {variant: build(repo, output / variant, args) for variant, repo in repos.items()}
        (output / 'builds.json').write_text(json.dumps(builds, indent=2) + '\n')
    else:
        builds = json.loads((output / 'builds.json').read_text())
    flags = ['-Xms512m', '-Xmx512m', '-Xss32m', '-XX:+UseG1GC', '-XX:ActiveProcessorCount=2', '-Xbatch']
    core = min(os.sched_getaffinity(0))
    rows, commands = [], []
    for fork in range(args.forks):
        variants = list(repos)
        random.Random(20261107 + fork // 2).shuffle(variants)
        if fork % 2:
            variants.reverse()
        for order in (['ordered', 'shuffled'] if fork % 2 == 0 else ['shuffled', 'ordered']):
            for variant in variants:
                directory = output / variant
                command = ['taskset', '-c', str(core), args.java, *flags, '-Djava.library.path=' + str(directory),
                           '-cp', str(directory / 'bench.jar'), 'BenchKt', args.sizes, order, str(fork), '5', '7', '12']
                source = run(command, log=output / f'{fork}-{order}-{variant}.log', timeout=240)
                count = 0
                for line in source.splitlines():
                    if line.startswith('TYPED_BENCH,'):
                        _, case, size, batch, iterations, ns, allocated = line.split(',')
                        rows.append(dict(fork=fork, variant=variant, order=order, case=case, size=int(size), batch=int(batch),
                                         ns_per_call=int(ns) / int(iterations), jvm_bytes_per_call=int(allocated) / int(iterations)))
                        count += 1
                assert count == (1 + 6 * len(args.sizes.split(','))) * 7, (variant, count)
                commands.append(command)
                with (output / 'samples.csv').open('w') as f:
                    writer = csv.DictWriter(f, fieldnames=list(rows[0])); writer.writeheader(); writer.writerows(rows)
                print(f'fork {fork}: {order} {variant}', flush=True)
    summary = []
    for order, case, size in sorted({(r['order'], r['case'], r['size']) for r in rows}):
        item = dict(order=order, case=case, size=size)
        for metric in ['ns_per_call', 'jvm_bytes_per_call']:
            by_variant = {variant: [statistics.median(r[metric] for r in rows if
                (r['variant'], r['fork'], r['order'], r['case'], r['size']) == (variant, fork, order, case, size))
                for fork in range(args.forks)] for variant in repos}
            for variant, values in by_variant.items():
                item[f'{variant}_{metric}'] = statistics.median(values)
            item[f'ratio_{metric}'] = statistics.median(b / a if a else 1.0 for a, b in zip(by_variant['baseline'], by_variant['candidate']))
        summary.append(item)
    with (output / 'summary.csv').open('w') as f:
        writer = csv.DictWriter(f, fieldnames=list(summary[0])); writer.writeheader(); writer.writerows(summary)
    report = {'builds': builds, 'java': subprocess.run([args.java, '-version'], capture_output=True, text=True).stderr,
              'commands': commands, 'core': core, 'warmup_calls_size8': 20000, 'summary': summary,
              'sources': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [HERE / 'Bench.kt', HERE / 'build.rs', HERE / 'flat.rs', Path(__file__)]}}
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
