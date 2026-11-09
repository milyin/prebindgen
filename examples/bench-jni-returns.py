#!/usr/bin/env python3
"""Compare JNI return allocation/latency with another checkout, in separate JVMs."""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def measure(root, target, label):
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"], cwd=root, env=environment))
    package = next(p["id"] for p in metadata["packages"] if p["name"] == "covertest-kotlin")
    build = subprocess.run(["cargo", "build", "--offline", "--release", "-p", "covertest-kotlin", "--message-format=json"],
                           cwd=root, env=environment, text=True, stdout=subprocess.PIPE)
    output = None
    for line in build.stdout.splitlines():
        event = json.loads(line)
        if event["reason"] == "build-script-executed" and event["package_id"] == package:
            output = Path(event["out_dir"]) / "kotlin"
        elif event["reason"] == "compiler-message":
            print(event["message"]["rendered"], end="")
    if build.returncode:
        raise RuntimeError(f"{label} Cargo build failed")
    if output is None:
        raise RuntimeError("Cargo did not report the generated Kotlin directory")
    sources = sorted(output.rglob("*.kt")) + [
        root / "examples/covertest-kotlin/kotlin/io/prebindgen/covertest/NativeLibrary.kt",
        ROOT / "examples/return-bench/Bench.kt",
    ]
    destination = target / "return-bench"
    destination.mkdir(exist_ok=True)
    jar = destination / (label + ".jar")
    subprocess.run([os.environ.get("KOTLINC", "kotlinc"), *map(str, sources), "-include-runtime", "-d", str(jar)], check=True)
    result = subprocess.check_output(["java", "-Xms128m", "-Xmx256m", "-Djava.library.path=" + str(target / "release"), "-jar", str(jar)], text=True)
    print(label + "\n" + result, flush=True)
    measurements = {}
    for line in result.splitlines():
        if line.startswith("RETURN_BENCH "):
            _, case, nanos, allocated = line.split()
            measurements[case] = {"ns": float(nanos.split("=")[1]), "bytes": float(allocated.split("=")[1])}
    if set(measurements) != {"scalar", "unit", "expanded", "handle"}:
        raise RuntimeError("benchmark did not report every return shape")
    return measurements


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True, type=Path, help="checkout of the packet-based generator")
    parser.add_argument("--target-dir", required=True, type=Path, help="parent for isolated Cargo output directories")
    args = parser.parse_args()
    args.target_dir.mkdir(parents=True, exist_ok=True)
    baseline = measure(args.baseline.resolve(), args.target_dir.resolve() / "baseline", "baseline")
    current = measure(ROOT, args.target_dir.resolve() / "current", "current")
    if current["scalar"]["bytes"] >= 0.01 or current["unit"]["bytes"] >= 0.01:
        raise RuntimeError("scalar/unit success unexpectedly allocates JVM objects after warmup")
    if any(current[case]["bytes"] >= baseline[case]["bytes"] for case in baseline):
        raise RuntimeError("return transport did not reduce allocation in every measured case")
    for case in baseline:
        old, new = baseline[case], current[case]
        print(f'{case}: {old["bytes"]:.1f} -> {new["bytes"]:.1f} JVM bytes/call; '
              f'{old["ns"]:.1f} -> {new["ns"]:.1f} ns/call')
    (args.target_dir / "results.json").write_text(json.dumps({"baseline": baseline, "current": current}, indent=2) + "\n")


if __name__ == "__main__":
    main()
