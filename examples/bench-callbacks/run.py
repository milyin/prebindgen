#!/usr/bin/env python3
"""Release JNI callback microbenchmark with a selectable generator checkout."""
import argparse
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile

FIXTURE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--checkout", type=Path, default=FIXTURE.parents[1])
parser.add_argument("--output", type=Path)
parser.add_argument("--jvms", type=int, default=3)
parser.add_argument("--iterations", type=int, default=200000)
parser.add_argument("--warmup", type=int, default=10)
parser.add_argument("--batches", type=int, default=9)
args = parser.parse_args()
if min(args.jvms, args.iterations, args.warmup, args.batches) < 1:
    parser.error("all measurement counts must be positive")
checkout = args.checkout.resolve()
output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="callback-bench-"))
output.mkdir(parents=True, exist_ok=True)
project = output / "fixture"
(project / "flat/src").mkdir(parents=True, exist_ok=True)
(project / "src").mkdir(exist_ok=True)
declarations = "prebindgen-declarations" if (checkout / "prebindgen-declarations").is_dir() else "prebindgen-registry"

def dependencies(names):
    return "\n".join(f"{name}={{path={json.dumps(str(checkout / name))}}}" for name in names)

manifest = '''[package]
name="callback-bench"
version="0.1.0"
edition="2021"
[lib]
name="callback_bench"
crate-type=["cdylib"]
[dependencies]
bench-flat={path="flat"}
jni="0.21.1"
konst="0.3.0"
tracing="0.1"
'''
manifest += dependencies(["prebindgen", "prebindgen-jni-runtime"])
if (checkout / "prebindgen-runtime").is_dir():
    manifest += "\n" + dependencies(["prebindgen-runtime"])
manifest += '\n[build-dependencies]\nsyn={version="2.0",features=["full"]}\nbench-flat={path="flat"}\n'
manifest += dependencies(["prebindgen-jni", declarations]) + "\n[workspace]\n"
(project / "Cargo.toml").write_text(manifest)
(project / "flat/Cargo.toml").write_text('''[package]
name="bench-flat"
version="0.1.0"
edition="2021"
[dependencies]
''' + dependencies(["prebindgen-proc-macro"]) + "\n[build-dependencies]\n" + dependencies(["prebindgen"]) + "\n")
shutil.copyfile(FIXTURE / "flat.rs", project / "flat/src/lib.rs")
(project / "flat/build.rs").write_text("fn main(){prebindgen::init_prebindgen_out_dir();}\n")
(project / "src/lib.rs").write_text('include!(concat!(env!("OUT_DIR"),"/bindings.rs"));\n')
(project / "build.rs").write_text((FIXTURE / "bindings.rs").read_text().replace("DECLARATIONS_CRATE", declarations.replace("-", "_")))
env = dict(os.environ, CARGO_TARGET_DIR=str(output / "target"))
print(f"Building {checkout}; results in {output}", flush=True)
with (output / "cargo.jsonl").open("w") as log:
    subprocess.run(["cargo", "build", "--offline", "--release", "--message-format=json"], cwd=project, env=env, stdout=log, check=True)
messages = [json.loads(line) for line in (output / "cargo.jsonl").read_text().splitlines()]
out = next(Path(m["out_dir"]) for m in messages if m.get("reason") == "build-script-executed" and m["package_id"].endswith("#callback-bench@0.1.0"))
jar = output / "bench.jar"
subprocess.run([os.environ.get("KOTLINC", "kotlinc"), *map(str, sorted((out / "kotlin").rglob("*.kt"))), str(FIXTURE / "CallbackBench.kt"), "-include-runtime", "-d", str(jar)], check=True)
samples = {case: [] for case in ("scalar", "record", "owned")}
for trial in range(args.jvms):
    result = subprocess.run(["java", "-Xms256m", "-Xmx256m", f"-Djava.library.path={output / 'target/release'}", "-jar", str(jar), str(args.iterations), str(args.warmup), str(args.batches)], text=True, capture_output=True, check=True)
    (output / f"run-{trial}.log").write_text(result.stdout + result.stderr)
    for line in result.stdout.splitlines():
        if line.startswith("BENCH,"):
            _, case, count, nanos = line.split(",")
            samples[case].append(int(nanos) / int(count))
    for case in samples:
        if len(samples[case]) != (trial + 1) * args.batches:
            raise RuntimeError(f"incomplete measurements for {case}")
    print(f"JVM {trial + 1}/{args.jvms} complete", flush=True)
report = {
    "checkout": str(checkout),
    "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip(),
    "dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=checkout)),
    "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
    "java": subprocess.run(["java", "-version"], capture_output=True, text=True, check=True).stderr.strip(),
    "kotlin": subprocess.run([os.environ.get("KOTLINC", "kotlinc"), "-version"], capture_output=True, text=True, check=True).stderr.strip(),
    "iterations": args.iterations, "warmup": args.warmup, "batches": args.batches, "jvms": args.jvms,
    "median_ns_per_callback": {case: statistics.median(values) for case, values in samples.items()},
    "samples_ns_per_callback": samples,
}
(output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report["median_ns_per_callback"], indent=2))
