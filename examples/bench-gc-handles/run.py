#!/usr/bin/env python3
"""Measure explicit Cleaner opt-in costs using the generated covertest bindings."""
import argparse
import csv
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def build():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"], cwd=ROOT))
    package = next(p["id"] for p in metadata["packages"] if p["name"] == "covertest-kotlin")
    build = subprocess.Popen(["cargo", "build", "--offline", "--release", "-p", "covertest-kotlin",
                              "--message-format=json"], cwd=ROOT, stdout=subprocess.PIPE, text=True)
    output = None
    for line in build.stdout:
        message = json.loads(line)
        if message["reason"] == "build-script-executed" and message["package_id"] == package:
            output = Path(message["out_dir"])
        elif message["reason"] == "compiler-message":
            print(message["message"]["rendered"], end="")
    if build.wait() or output is None:
        raise RuntimeError("covertest build failed or did not report its output")
    target = Path(metadata["target_directory"])
    jar = target / "gc-handle-bench.jar"
    kotlin = sorted(output.joinpath("kotlin").rglob("*.kt"))
    kotlin += sorted(p for p in (ROOT / "examples/covertest-kotlin/kotlin").rglob("*.kt") if p.name != "Test.kt")
    kotlin += [Path(__file__).parent / "Bench.kt"]
    subprocess.run([os.environ.get("KOTLINC", "kotlinc"), *map(str, kotlin), "-include-runtime", "-d", str(jar)], check=True)
    return target, jar


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepared", type=Path,
                        help="Reuse the target directory from a build of this exact fixture revision.")
    parser.add_argument("--processes", type=int, default=6)
    parser.add_argument("--output", type=Path, default=Path(__file__).parent / "results.json")
    parser.add_argument("--wait-for-file", type=Path,
                        help="After building, wait for this file before starting timing.")
    args = parser.parse_args()
    if args.prepared is None:
        target, jar = build()
    else:
        target = args.prepared.resolve()
        jar = target / "gc-handle-bench.jar"
        if not jar.is_file() or not (target / "release").is_dir():
            parser.error("--prepared needs the target directory of an earlier fixture build")
    if args.wait_for_file is not None:
        print("Build complete; waiting for " + str(args.wait_for_file), flush=True)
        while not args.wait_for_file.exists():
            time.sleep(1)
    cases = ["plain-read", "gc-read", "plain-read-aa", "plain-close", "gc-close", "plain-close-aa"]
    samples = []
    rng = random.Random(871)
    for process in range(args.processes):
        order = cases.copy()
        rng.shuffle(order)
        for case in order:
            result = subprocess.check_output(["java", "-Djava.library.path=" + str(target / "release"),
                                              "-jar", str(jar), case], text=True)
            for row in csv.reader(result.splitlines()):
                name, batch, count, elapsed, allocated = row
                samples.append(dict(case=name, process=process, batch=int(batch), count=int(count),
                                    elapsed_ns=int(elapsed), allocated_bytes=int(allocated)))
            print(f"process {process + 1}/{args.processes}: {case}", flush=True)
    summary = []
    for case in cases:
        rows = [s for s in samples if s["case"] == case]
        times, allocations = [], []
        for process in range(args.processes):
            part = [s for s in rows if s["process"] == process]
            count = sum(s["count"] for s in part)
            times.append(sum(s["elapsed_ns"] for s in part) / count)
            allocations.append(sum(s["allocated_bytes"] for s in part) / count)
        summary.append(dict(case=case, median_ns=statistics.median(times), min_ns=min(times),
                            max_ns=max(times), median_bytes=statistics.median(allocations)))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(dict(
        revision=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        dirty=bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True)),
        java=subprocess.run(["java", "-version"], capture_output=True, text=True).stderr,
        platform=platform.platform(), processes=args.processes, reused_artifacts=args.prepared is not None,
        summary=summary, samples=samples), indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
