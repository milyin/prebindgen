#!/usr/bin/env python3
"""Compare prepared callback benchmarks in rotating JVM order, without building."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import statistics
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--variant", action="append", required=True, metavar="NAME=DIRECTORY")
parser.add_argument("--output", required=True, type=Path)
parser.add_argument("--jvms", type=int, help="JVMs per variant; defaults to two complete rotations")
parser.add_argument("--iterations", type=int, default=200000)
parser.add_argument("--warmup", type=int, default=10)
parser.add_argument("--batches", type=int, default=9)
args = parser.parse_args()
if min(args.iterations, args.warmup, args.batches) < 1:
    parser.error("all measurement counts must be positive")
variants = {}
for value in args.variant:
    name, separator, directory = value.partition("=")
    if not separator or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", name):
        parser.error("variants must use NAME=DIRECTORY with a simple alphanumeric name")
    if name in variants:
        parser.error(f"duplicate variant: {name}")
    variants[name] = Path(directory).resolve()
if len(variants) < 2:
    parser.error("at least two variants are required")
if args.jvms is None:
    args.jvms = 2 * len(variants)
if args.jvms <= 0 or args.jvms % len(variants):
    parser.error("JVM count must be a positive multiple of the number of variants")
output = args.output.resolve()
if output.exists():
    parser.error("use a new output directory to preserve earlier measurements")

def artifacts(directory):
    libraries = [directory / "target/release" / name for name in
                 ("libcallback_bench.so", "libcallback_bench.dylib", "callback_bench.dll")]
    libraries = [library for library in libraries if library.is_file()]
    if len(libraries) != 1:
        raise RuntimeError(f"expected one native callback library in {directory}")
    return {
        "jar_sha256": hashlib.sha256((directory / "bench.jar").read_bytes()).hexdigest(),
        "native_sha256": hashlib.sha256(libraries[0].read_bytes()).hexdigest(),
    }

sources = {}
for name, directory in variants.items():
    metadata = json.loads((directory / "results.json").read_text())
    if metadata.get("profile") != "release" or "profile_environment" not in metadata:
        parser.error(f"rebuild {name} using the current run.py to record profile metadata")
    sources[name] = {
        "directory": str(directory),
        "build": {key: metadata[key] for key in
                  ("checkout", "commit", "dirty", "rustc", "kotlin", "java", "profile", "profile_environment")},
        **artifacts(directory),
    }
reference = next(iter(sources.values()))["build"]
for name, source in sources.items():
    for key in ("rustc", "kotlin", "profile", "profile_environment"):
        if source["build"][key] != reference[key]:
            parser.error(f"{name} differs in {key}; use matching tools and profile settings")
output.mkdir(parents=True)
cases = ("scalar", "record", "owned")
samples = {name: {case: [] for case in cases} for name in variants}
jvm_medians = {name: {case: [] for case in cases} for name in variants}
# Each batch invokes all three cases; Kotlin Long arithmetic wraps at 64 bits.
expected = (args.warmup + args.batches) * (args.iterations * (args.iterations - 1) // 2 + args.iterations ** 2 + args.iterations)
expected = (expected + 2 ** 63) % (2 ** 64) - 2 ** 63
for trial in range(args.jvms):
    names = list(variants)
    offset = trial % len(names)
    for name in names[offset:] + names[:offset]:
        directory = variants[name]
        result = subprocess.run([
            "java", "-Xms256m", "-Xmx256m",
            f"-Djava.library.path={directory / 'target/release'}", "-jar", str(directory / "bench.jar"),
            str(args.iterations), str(args.warmup), str(args.batches),
        ], capture_output=True, text=True)
        (output / f"run-{trial}-{name}.log").write_text(result.stdout + result.stderr)
        result.check_returncode()
        current = {case: [] for case in cases}
        sink = None
        for line in result.stdout.splitlines():
            if line.startswith("BENCH,"):
                _, case, count, nanos = line.split(",")
                if case not in cases or int(count) != args.iterations or int(nanos) <= 0:
                    raise RuntimeError(f"invalid measurement from {name}: {line}")
                current[case].append(int(nanos) / int(count))
            elif line.startswith("SINK,"):
                sink = int(line.split(",")[1])
        if sink != expected:
            raise RuntimeError(f"callback results differ for {name}: {sink} != {expected}")
        if any(len(values) != args.batches for values in current.values()):
            raise RuntimeError(f"incomplete measurements for {name}")
        for case, values in current.items():
            samples[name][case].extend(values)
            jvm_medians[name][case].append(statistics.median(values))
        print(f"JVM round {trial + 1}/{args.jvms}: {name}", flush=True)
for name, directory in variants.items():
    if any(sources[name][key] != value for key, value in artifacts(directory).items()):
        raise RuntimeError(f"benchmark artifact changed while measuring {name}")
report = {
    "variants": sources,
    "java": subprocess.run(["java", "-version"], capture_output=True, text=True, check=True).stderr.strip(),
    "iterations": args.iterations, "warmup": args.warmup, "batches": args.batches, "jvms": args.jvms,
    "order": "rotate variant order each JVM round; no builds during comparison",
    "summary": "median of per-JVM medians; batches within one JVM are not independent JVM samples",
    "median_ns_per_callback": {case: {name: statistics.median(jvm_medians[name][case]) for name in variants} for case in cases},
    "per_jvm_median_ns_per_callback": jvm_medians,
    "jvm_median_range_ns": {case: {name: [min(jvm_medians[name][case]), max(jvm_medians[name][case])] for name in variants} for case in cases},
    "samples_ns_per_callback": samples,
}
(output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report["median_ns_per_callback"], indent=2))
