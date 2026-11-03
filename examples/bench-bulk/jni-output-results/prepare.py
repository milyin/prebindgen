#!/usr/bin/env python3
"""Build the common fixture and snapshot its library before reusing build storage."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil

p = argparse.ArgumentParser(__doc__)
p.add_argument('--checkout', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--target', type=Path, required=True)
p.add_argument('--counts', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
a.target.mkdir(parents=True, exist_ok=True)
(a.output/'target').symlink_to(a.target.resolve(), target_is_directory=True)
runner = Path(__file__).resolve().parents[1]/'run.py'
spec = importlib.util.spec_from_file_location('fixture_runner', runner)
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)
info = fixture.build(a.checkout.resolve(), a.output.resolve(), os.environ.get('KOTLINC','kotlinc'), False, False, False, a.counts)
snapshot = a.output/'native'/Path(info['library']).name
snapshot.parent.mkdir()
shutil.copyfile(info['library'],snapshot)
info['library']=str(snapshot.resolve())
(a.output/'build.json').write_text(json.dumps(info,indent=2)+'\n')
