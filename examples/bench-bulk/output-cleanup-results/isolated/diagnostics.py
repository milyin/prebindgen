"""Summarize measured-phase deoptimizations and sampling-loop compilation."""
import argparse
import hashlib
import json
import xml.etree.ElementTree as ET
from pathlib import Path

p = argparse.ArgumentParser(__doc__)
p.add_argument('measurement', type=Path)
a = p.parse_args()
data = json.loads((a.measurement / 'results.json').read_text())
rows = []
for run in data['runs']:
    log = a.measurement / 'raw' / (run['id'] + '-jit.xml')
    assert hashlib.sha256(log.read_bytes()).hexdigest() == run['jit_sha256']
    root = ET.parse(log).getroot()
    methods = {n.get('compile_id'): dict(n.attrib) for n in root.iter('nmethod')}
    lo = run['phases']['start']['uptime_ms'] / 1000
    hi = run['phases']['end']['uptime_ms'] / 1000
    retired = {n.get('compile_id') for n in root.iter('make_not_entrant')
               if float(n.get('stamp', 'inf')) <= lo}
    active_sample_c2 = [m for key, m in methods.items() if
                        'BenchKt main$sample ' in m.get('method', '')
                        and m.get('level') == '4' and 'compile_kind' not in m
                        and float(m['stamp']) <= lo and key not in retired]
    events = []
    for tag in ['uncommon_trap', 'deoptimized', 'make_not_entrant']:
        for n in root.iter(tag):
            if lo <= float(n.get('stamp', '-1')) <= hi:
                events.append(dict(kind=tag, **n.attrib,
                                   compiled_method=methods.get(n.get('compile_id')),
                                   frames=[dict(f.attrib) for f in n.iter('jvms')]))
    rows.append(dict(id=run['id'], active_sample_c2_at_start=active_sample_c2,
                     measured_phase_deoptimization_events=events))
result = dict(runs=rows, analyzer_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(a.measurement / 'diagnostics.json').write_text(json.dumps(result, indent=2) + '\n')
print('Runs with active C2 sampling method:', sum(bool(r['active_sample_c2_at_start']) for r in rows), '/', len(rows))
print('Deoptimization events during measurement:', sum(len(r['measured_phase_deoptimization_events']) for r in rows))
