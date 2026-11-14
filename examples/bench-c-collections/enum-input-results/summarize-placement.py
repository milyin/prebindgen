#!/usr/bin/env python3
"""Export placement.py measurements without raw per-batch or console logs."""
import argparse
import json
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('run', type=Path)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
r = json.loads((a.run / 'results.json').read_text())
probes = {}
probe_file = a.run / 'allocation-probe.csv'
if probe_file.exists():
    for line in probe_file.read_text().splitlines():
        fields = line.split(',')
        if fields[0] == 'PROBE':
            probes.setdefault(int(fields[4]), {})[int(fields[1])] = fields[5:]
    for versions in probes.values():
        assert versions[0] == versions[1] and versions[0][-1] == '1'
metadata = {key: r[key] for key in ['builds', 'driver_sha256', 'settings', 'source_sha256']}
metadata['allocation_probe'] = [dict(size=512, slots=slots,
    source_addresses_match=True, destination_addresses_match=True,
    destination_sized_malloc_calls_each=1) for slots in probes]
for row in r['summary']:
    changes = [score['change_percent'] for score in row['forks']]
    row['paired_change_median_percent'] = statistics.median(changes)
lines = ['{', '  "metadata": ' + json.dumps(metadata, separators=(',', ':')) + ',', '  "summary": [']
lines += ['    ' + json.dumps(row, separators=(',', ':')) + (',' if i + 1 < len(r['summary']) else '')
          for i, row in enumerate(r['summary'])]
lines += ['  ]', '}']
a.output.write_text('\n'.join(lines) + '\n')
