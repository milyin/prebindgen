#!/usr/bin/env python3
"""Export compact aggregate measurements and build provenance without raw logs."""
import argparse
import json
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('runs', nargs='+', help='label=directory containing input-compare.py results.json')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
builds = {}
summary = []
metadata = {}
for entry in a.runs:
    label, directory = entry.split('=', 1)
    r = json.loads((Path(directory) / 'results.json').read_text())
    metadata[label] = {k: r[k] for k in ['settings', 'affinity', 'platform', 'cpu', 'loader_environment', 'samples_sha256', 'runner_sha256', 'driver_sha256']}
    metadata[label]['libraries'] = {v: b['library_sha256'] for v,b in r['effective_builds'].items()}
    for b in r['builds'].values():
        builds[b['library_sha256']] = b
    for row in r['summary']:
        row = dict(run=label, **row)
        if 'paired_wall_changes_percent' in row:
            row['paired_change_median_percent'] = statistics.median(row['paired_wall_changes_percent'])
        summary.append(row)
a.output.mkdir(parents=True, exist_ok=True)
(a.output/'provenance.json').write_text(json.dumps(dict(builds=builds, runs=metadata), indent=2)+'\n')
(a.output/'summary.json').write_text('{\n  "summary": [\n'+',\n'.join('    '+json.dumps(row,separators=(',', ':')) for row in summary)+'\n  ]\n}\n')
