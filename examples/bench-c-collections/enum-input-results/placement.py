#!/usr/bin/env python3
"""Investigate native sum-function placement in frozen Linux/x86-64 benchmarks.

This is an isolated diagnostic, not a binding-generator option. Only the known
152-byte native sum implementation is accepted. Clones alter private process
memory; library files, converters and the production generator remain unchanged.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import re
import shlex
import statistics
import subprocess

if not __debug__:
    raise RuntimeError('This diagnostic requires Python validation assertions')

HERE = Path(__file__).resolve().parent
SHAPE = '5ca6ca2bfc339098658999a2f37f6d96dab59b5aabe28937ba8fe6af1795fd11'
MODES = dict(original=None, common0=(0, None), separate0=(0, 0),
             separate16=(16, 16), separate32=(32, 32), separate48=(48, 48),
             reproduce=(0, 48), swap=(48, 0))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
output = lambda *args: subprocess.check_output(args, text=True)


def inspect(directory):
    build = json.loads((directory / 'build.json').read_text())
    library = Path(build['library']).resolve()
    assert not build['count_allocations'] and build['alias_checks']
    assert sha(library) == build['library_sha256'], 'Frozen library changed'
    symbols = output('nm', '-SC', str(library)).splitlines()
    native = next(s.split(maxsplit=3) for s in symbols
                  if s.endswith(' c_collection_flat::take_numbers'))
    boundary = next(s.split(maxsplit=3) for s in symbols
                    if 'prebindgen_runtime::boundary::' in s
                    and 'bench_take_numbers::{closure#0}' in s)
    address, size = int(native[0], 16), int(native[1], 16)
    assert size == 0x98, 'Unsupported native sum function size'
    sections = output('readelf', '-SW', str(library))
    section = re.search(r'\]\s+\.text\s+PROGBITS\s+([a-f0-9]+)\s+([a-f0-9]+)', sections)
    text_address, file_offset = (int(s, 16) for s in section.groups())
    start = file_offset + address - text_address
    code = bytearray(library.read_bytes()[start:start + size])
    assert code[0x8d:0x8f] == b'\xff\x15'
    code[0x8f:0x93] = bytes(4)  # The sole RIP-relative deallocation call.
    assert hashlib.sha256(code).hexdigest() == SHAPE, 'Unsupported native sum instruction shape'
    # Find the native-function pointer actually used by the numbers converter.
    relocations = output('readelf', '-rW', str(library))
    slots = {int(m[0], 16) for m in re.findall(
        r'^([0-9a-f]+)\s+[0-9a-f]+\s+R_X86_64_RELATIVE\s+([0-9a-f]+)\s*$',
        relocations, re.M) if int(m[1], 16) == address}
    begin, length = int(boundary[0], 16), int(boundary[1], 16)
    assembly = output('objdump', '-d', f'--start-address={begin}',
                      f'--stop-address={begin + length}', str(library))
    references = {int(s, 16) for s in re.findall(r'#\s+([0-9a-f]+)\b', assembly)}
    selected = slots & references
    assert len(selected) == 1, 'Cannot identify numbers converter native call'
    slot = selected.pop()
    # The C helper restores this page to its original read-only protection.
    segments = output('readelf', '-lW', str(library))
    relro = re.search(r'GNU_RELRO\s+\S+\s+(0x[0-9a-f]+)\s+\S+\s+\S+\s+(0x[0-9a-f]+)', segments)
    low, length = (int(s, 16) for s in relro.groups())
    assert low <= (slot & ~4095) and (slot & ~4095) + 4096 <= low + length
    return dict(library=str(library), library_sha256=sha(library),
                native=address, native_size=size, numbers_got=slot, shape_sha256=SHAPE)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--before', type=Path, required=True, help='frozen build.py output')
    p.add_argument('--after', type=Path, required=True, help='frozen build.py output')
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--cpu', type=int, default=0)
    p.add_argument('--forks', type=int, default=8)
    p.add_argument('--batches', type=int, default=8)
    p.add_argument('--elements', type=int, default=33554432)
    p.add_argument('--smoke', action='store_true')
    p.add_argument('--probe-allocations', action='store_true', help='separate untimed GNU libc address probe')
    a = p.parse_args()
    assert platform.system() == 'Linux' and platform.machine() == 'x86_64'
    assert os.sysconf('SC_PAGE_SIZE') == 4096
    assert a.forks > 0 and a.batches > 0 and a.elements >= 4096
    root = a.output.resolve(); root.mkdir(parents=True, exist_ok=True)
    assert not list(root.iterdir()), 'Use an empty output directory'
    (root / 'raw').mkdir()
    builds = [inspect(d.resolve()) for d in [a.before, a.after]]
    header = ''.join(f'#define {side}_{key} 0x{build[field]:x}\n'
                     for side, build in zip(['BEFORE', 'AFTER'], builds)
                     for key, field in [('NATIVE', 'native'), ('GOT', 'numbers_got')])
    (root / 'placement.h').write_text(header)
    command = [*shlex.split(os.environ.get('CC', 'cc')), '-std=gnu11', '-O3',
               '-Wall', '-Wextra', '-Werror', '-I' + str(root),
               '-I' + str(HERE.parent / 'generated'), str(HERE / 'placement.c'), '-ldl']
    subprocess.run([*command, '-o', str(root / 'driver')], check=True)
    if a.probe_allocations:
        subprocess.run([*command, '-DPROBE_MALLOC', '-rdynamic', '-o', str(root / 'probe')], check=True)
        env = dict(os.environ, SOURCE_OFFSET='32'); env.pop('SUM_ALIGN', None); env.pop('SUM_ALIGN_OTHER', None); env.pop('SOURCE_STREAM', None)
        probe = subprocess.check_output(['taskset', '-c', str(a.cpu), str(root / 'probe'),
                    *(b['library'] for b in builds), '0', '1', '4096', '512'], env=env, text=True)
        (root / 'allocation-probe.csv').write_text(probe)
        addresses = {}
        for line in probe.splitlines():
            fields = line.split(',')
            if fields[0] == 'PROBE':
                addresses.setdefault(fields[4], {})[fields[1]] = fields[5:]
        assert len(addresses) == 2
        for pair in addresses.values():
            assert pair['0'] == pair['1'] and pair['0'][-1] == '1'

    forks, batches, elements = (1, 2, 1048576) if a.smoke else (a.forks, a.batches, a.elements)
    modes = {k: v for k, v in MODES.items() if not a.smoke or k in ['original', 'separate0', 'reproduce', 'swap']}
    samples = []
    for fork in range(forks):
        cells = [(mode, size) for mode in modes for size in [512, 4096]]
        random.Random(20261106 + fork).shuffle(cells)
        for mode, size in cells:
            env = dict(os.environ, SOURCE_OFFSET='32'); env.pop('SOURCE_STREAM', None)
            env.pop('SUM_ALIGN', None); env.pop('SUM_ALIGN_OTHER', None)
            if modes[mode] is not None:
                env['SUM_ALIGN'] = str(modes[mode][0])
                if modes[mode][1] is not None: env['SUM_ALIGN_OTHER'] = str(modes[mode][1])
            command = ['taskset', '-c', str(a.cpu), str(root / 'driver'),
                       *(b['library'] for b in builds), str(fork % 2), str(batches), str(elements), str(size)]
            result = subprocess.run(command, env=env, text=True, capture_output=True, check=True)
            (root / 'raw' / f'{fork}-{mode}-{size}.csv').write_text(result.stdout)
            for line in result.stdout.splitlines():
                fields = line.split(',')
                if fields[0] == 'PAIR':
                    n, offset, slots, batch, iterations, bw, aw, bc, ac = map(int, fields[1:])
                    assert n == size and offset == 32
                    samples.append(dict(fork=fork, mode=mode, size=size, slots=slots,
                        batch=batch, iterations=iterations, before_wall=bw / iterations,
                        after_wall=aw / iterations, before_cpu=bc / iterations, after_cpu=ac / iterations))
            print(fork, mode, size, flush=True)
    summary = []
    metrics = ['before_wall', 'after_wall', 'before_cpu', 'after_cpu']
    for mode in modes:
        for size in [512, 4096]:
            for slots in [1, 262144 // size]:
                scores = []
                for fork in range(forks):
                    rows = [s for s in samples if (s['mode'], s['size'], s['slots'], s['fork']) == (mode, size, slots, fork)]
                    assert len(rows) == batches
                    score = {key: statistics.mean(r[key] for r in rows) for key in metrics}
                    score['change_percent'] = (score['after_wall'] / score['before_wall'] - 1) * 100
                    scores.append(score)
                median = {key: statistics.median(s[key] for s in scores) for key in metrics}
                summary.append(dict(mode=mode, size=size, slots=slots, **median,
                    change_percent=(median['after_wall'] / median['before_wall'] - 1) * 100, forks=scores))
    report = dict(summary=summary, builds=builds, driver_sha256=sha(root / 'driver'),
                  settings=dict(forks=forks, batches=batches, elements=elements, cpu=a.cpu),
                  source_sha256={name: sha(HERE / name) for name in ['placement.py', 'placement.c']})
    (root / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    for build in builds: assert sha(build['library']) == build['library_sha256']


if __name__ == '__main__':
    main()
