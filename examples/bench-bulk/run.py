#!/usr/bin/env python3
"""Compare ordinary and bulk generated JNI wrappers over identical Rust functions."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import tempfile

FIXTURE = Path(__file__).resolve().parent
MODES = ("plain", "bulk")
SINGLE_CASES = ("control", "scalar", "record_input", "record_return", "wide_record_input", "wide_record_return", "callback_record_return", "callback_wide_record_return")
KINDS = ("numbers_input", "records_input", "records_output", "records_roundtrip", "callback_records", "records_reduce")


def command_version(command, cwd=None):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=True)
    return (result.stdout + result.stderr).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_revisions(metadata, project):
    fixture_repo = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], cwd=FIXTURE, text=True).strip())
    repositories = {fixture_repo: []}
    for package in metadata["packages"]:
        if package["source"] is not None:
            continue
        root = Path(package["manifest_path"]).parent.resolve()
        if root.is_relative_to(project):
            continue
        repo = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], cwd=root, text=True).strip())
        repositories.setdefault(repo, []).append(package["name"])
    result = []
    for repo, packages in sorted(repositories.items()):
        dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=repo, text=True)
        if dirty:
            raise RuntimeError(f"path dependency repository must be committed and clean: {repo}\n{dirty}")
        result.append({"repository": str(repo), "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(), "packages": sorted(packages), "contains_fixture": repo == fixture_repo})
    return result


def cargo_configs(project):
    home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    paths = [home / "config", home / "config.toml"]
    for parent in [project, *project.parents]:
        paths += [parent / ".cargo/config", parent / ".cargo/config.toml"]
    return {str(path): digest(path) for path in paths if path.is_file()}


def build(checkout, output, kotlinc, record, check, fetch):
    project = output / "fixture"
    (project / "flat/src").mkdir(parents=True)
    (project / "src").mkdir()

    def dependencies(names):
        return "\n".join(f"{name}={{path={json.dumps(str(checkout / name))}}}" for name in names)

    (project / "Cargo.toml").write_text('''[package]
name="bulk-bench"
version="0.1.0"
edition="2021"
[lib]
name="bulk_bench"
crate-type=["cdylib"]
[dependencies]
bench-flat={path="flat"}
konst="0.3.0"
''' + dependencies(["prebindgen-runtime", "prebindgen-jni-runtime"]) + '''
[build-dependencies]
bench-flat={path="flat"}
''' + dependencies(["prebindgen-jni", "prebindgen-declarations"]) + '\n[workspace]\n')
    (project / "flat/Cargo.toml").write_text('''[package]
name="bench-flat"
version="0.1.0"
edition="2021"
[dependencies]
''' + dependencies(["prebindgen-proc-macro"]) + '\n[build-dependencies]\n' + dependencies(["prebindgen"]) + '\n')
    shutil.copyfile(FIXTURE / "flat.rs", project / "flat/src/lib.rs")
    shutil.copyfile(FIXTURE / "bindings.rs", project / "build.rs")
    (project / "flat/build.rs").write_text('fn main(){prebindgen::init_prebindgen_out_dir();}\n')
    (project / "src/lib.rs").write_text('''#![allow(non_camel_case_types, unused_variables)]
mod plain { include!(concat!(env!("OUT_DIR"),"/plain.rs")); }
mod bulk { include!(concat!(env!("OUT_DIR"),"/bulk.rs")); }
''')
    seed = FIXTURE / "Cargo.lock.snapshot"
    shutil.copyfile(seed, project / "Cargo.lock")
    env = dict(os.environ, CARGO_TARGET_DIR=str(output / "target"))
    if fetch:
        subprocess.run(["cargo", "fetch", "--locked"], cwd=project, env=env, check=True)
    rustc = command_version([os.environ.get("RUSTC", "rustc"), "-vV"], cwd=project)
    host = next(line.split(": ", 1)[1] for line in rustc.splitlines() if line.startswith("host: "))
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--offline", "--locked", "--filter-platform", host, "--format-version=1"], cwd=project, env=env))
    revisions = source_revisions(metadata, project)
    package = next(p["id"] for p in metadata["packages"] if p["name"] == "bulk-bench")
    with (output / "cargo.jsonl").open("w") as log:
        subprocess.run(["cargo", "build", "--offline", "--locked", "--release", "--message-format=json"], cwd=project, env=env, stdout=log, check=True)
    messages = [json.loads(line) for line in (output / "cargo.jsonl").read_text().splitlines()]
    out = next(Path(m["out_dir"]) for m in messages if m.get("reason") == "build-script-executed" and m["package_id"] == package)
    cdylib = next(m for m in messages if m.get("reason") == "compiler-artifact" and m["package_id"] == package and "cdylib" in m["target"]["kind"])
    if cdylib["profile"]["opt_level"] != "3" or cdylib["profile"]["debug_assertions"]:
        raise RuntimeError(f"unexpected native benchmark profile: {cdylib['profile']}")
    jar = output / "bench.jar"
    subprocess.run([kotlinc, *map(str, sorted((out / "kotlin").rglob("*.kt"))), str(FIXTURE / "Bench.kt"), "-nowarn", "-include-runtime", "-d", str(jar)], check=True)
    libraries = [output / "target/release" / name for name in ("libbulk_bench.so", "libbulk_bench.dylib", "bulk_bench.dll")]
    library, = [p for p in libraries if p.is_file()]
    if record or check:
        destination = FIXTURE / "generated"
        files = {Path("kotlin") / path.relative_to(out / "kotlin"): path.read_bytes() for path in (out / "kotlin").rglob("*.kt")}
        files.update({Path(name): (out / name).read_bytes() for name in ("plain.rs", "bulk.rs")})
        if check:
            existing={path.relative_to(destination) for path in destination.rglob("*") if path.is_file()}
            if existing != files.keys() or any((destination / path).read_bytes() != data for path,data in files.items()):
                raise RuntimeError("generated files differ; rebuild with --record-generated")
    shutil.copyfile(project / "Cargo.lock", output / "Cargo.lock.snapshot")
    if source_revisions(metadata, project) != revisions:
        raise RuntimeError("path dependency revisions changed during build")
    build_info = {
        "checkout": str(checkout),
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip(),
        "dirty_paths": subprocess.check_output(["git", "status", "--porcelain"], cwd=checkout, text=True).splitlines(),
        "profile": "release",
        "profile_environment": {key: value for key, value in os.environ.items() if key.startswith(("CARGO_PROFILE_", "CARGO_BUILD_", "RUSTC")) or key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN")},
        "rustc": rustc,
        "path_dependency_repositories": revisions,
        "cargo_config_sha256": cargo_configs(project),
        "cargo": command_version(["cargo", "-V"], cwd=project),
        "dependency_lock_sha256": digest(project / "Cargo.lock"),
        "cargo_profile": cdylib["profile"],
        "kotlin": command_version([kotlinc, "-version"]),
        "fixture_sha256": {name: digest(FIXTURE / name) for name in ("flat.rs", "bindings.rs", "Bench.kt", "run.py", "Cargo.lock.snapshot")},
        "jar": str(jar), "library": str(library),
        "jar_sha256": digest(jar), "native_sha256": digest(library),
    }
    (output / "build.json").write_text(json.dumps(build_info, indent=2) + "\n")
    # Snapshot writes are intentional mutations, after all build-source checks.
    if record and destination.exists():
        for stale in destination.rglob("*"):
            if stale.is_file() and stale.relative_to(destination) not in files:
                stale.unlink()
    if record:
        for relative, content in files.items():
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
    return build_info


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, default=FIXTURE.parents[1])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--prepared", type=Path, help="reuse build.json/JAR/library from an earlier run")
    parser.add_argument("--jvms", type=int, default=4, help="fresh JVMs per mode; even for balanced order")
    parser.add_argument("--warmup", type=int, default=5)
    parser.add_argument("--batches", type=int, default=9)
    parser.add_argument("--target-ms", type=int, default=30, help="calibrated batch target duration")
    parser.add_argument("--sizes", default="0,1,8,64,512")
    parser.add_argument("--payloads", default="64,4096,65536", help="byte payload sizes for packet inputs")
    parser.add_argument("--cases", help="comma-separated case names; default runs all cases")
    parser.add_argument("--record-generated", action="store_true")
    parser.add_argument("--check-generated", action="store_true")
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--fetch", action="store_true", help="fetch the pinned fixture dependencies before building")
    parser.add_argument("--check-jni", action="store_true", help="correctness smoke run; not suitable for timing")
    args = parser.parse_args()
    if min(args.jvms, args.warmup, args.batches, args.target_ms) < 1 or args.jvms % 2:
        parser.error("counts must be positive; JVM count must be even to balance mode order")
    try:
        sizes = [int(n) for n in args.sizes.split(',')]
        payloads = [int(n) for n in args.payloads.split(',')]
    except ValueError:
        parser.error("sizes must be comma-separated integers")
    if not sizes or min(sizes) < 0 or max(sizes) > 100000 or len(set(sizes)) != len(sizes):
        parser.error("sizes must be unique and between 0 and 100000")
    if not payloads or min(payloads) < 0 or max(payloads) > 1000000 or len(set(payloads)) != len(payloads):
        parser.error("payload sizes must be unique and between 0 and 1000000")
    if args.record_generated and args.check_generated:
        parser.error("choose --record-generated or --check-generated")
    if args.prepared and (args.record_generated or args.check_generated):
        parser.error("generated-file checks/recording require a fresh build")
    available = (*SINGLE_CASES, *KINDS, "packet_input")
    selected = args.cases.split(',') if args.cases else list(available)
    if not selected or len(set(selected)) != len(selected) or set(selected) - set(available):
        parser.error("cases must be unique names from " + ','.join(available))
    output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="bulk-bench-"))
    if output.exists() and any(output.iterdir()):
        parser.error("use a new empty output directory to preserve earlier results")
    output.mkdir(parents=True, exist_ok=True)
    kotlinc = os.environ.get("KOTLINC", "kotlinc")
    info = json.loads((args.prepared.resolve() / "build.json").read_text()) if args.prepared else build(args.checkout.resolve(), output, kotlinc, args.record_generated, args.check_generated, args.fetch)
    jar, library = Path(info["jar"]), Path(info["library"])
    if digest(jar) != info["jar_sha256"] or digest(library) != info["native_sha256"]:
        raise RuntimeError("prepared benchmark artifact changed")
    if args.build_only:
        print(f"Release build ready: {output}")
        return
    keys = [(kind, 1) for kind in SINGLE_CASES] + [(kind, n) for n in sizes for kind in KINDS] + [("packet_input", n) for n in payloads]
    keys = [(kind, n) for kind, n in keys if kind in selected]
    medians = {mode: {f"{kind}/{n}": [] for kind, n in keys} for mode in MODES}
    runs = []
    jvm_args = ["-Xms256m", "-Xmx256m", "-XX:+UseG1GC", "-XX:ActiveProcessorCount=2", "-XX:+PrintCommandLineFlags"] + (["-Xcheck:jni"] if args.check_jni else [])
    print(f"Build ready; measuring without concurrent builds. Output: {output}", flush=True)
    for trial in range(args.jvms):
        for mode in (MODES if trial % 2 == 0 else MODES[::-1]):
            command = ["java", *jvm_args, f"-Djava.library.path={library.parent}", "-jar", str(jar), mode, args.sizes,
                       str(args.warmup), str(args.batches), str(args.target_ms), str(trial), args.payloads, ",".join(selected)]
            result = subprocess.run(command, text=True, capture_output=True)
            (output / f"run-{trial}-{mode}.log").write_text(result.stdout + result.stderr)
            result.check_returncode()
            if "WARNING in native method" in result.stdout + result.stderr or "exceeds capacity" in result.stdout + result.stderr:
                raise RuntimeError("JNI warning; see run log")
            samples = {f"{kind}/{n}": [] for kind, n in keys}
            case_order = []
            for line in result.stdout.splitlines():
                if line.startswith("BULK_BENCH,"):
                    _, actual, kind, size, batch, iterations, nanos, allocated = line.split(',')
                    key = f"{kind}/{size}"
                    iterations, nanos, allocated, batch = map(int, (iterations, nanos, allocated, batch))
                    if actual != mode or key not in samples or batch != len(samples[key]) or iterations <= 0 or nanos <= 0 or allocated < 0:
                        raise RuntimeError(f"invalid measurement: {line}")
                    if batch == 0:
                        case_order.append(key)
                    samples[key].append({"iterations": iterations, "elapsed_ns": nanos, "jvm_allocated_bytes": allocated,
                                         "ns_per_call": nanos / iterations, "jvm_bytes_per_call": allocated / iterations})
            if not any(line.startswith("SINK,") for line in result.stdout.splitlines()) or any(len(s) != args.batches for s in samples.values()):
                raise RuntimeError("incomplete measurement; see run log")
            for key, values in samples.items():
                medians[mode][key].append({"ns_per_call": statistics.median(v["ns_per_call"] for v in values),
                                           "jvm_bytes_per_call": statistics.median(v["jvm_bytes_per_call"] for v in values),
                                           "batch_ms": statistics.median(v["elapsed_ns"] for v in values) / 1000000})
            runs.append({"trial": trial, "mode": mode, "samples": samples, "case_order": case_order, "effective_flags": [line for line in result.stdout.splitlines() if line.startswith("-XX:")]})
            print(f"JVM {trial+1}/{args.jvms}: {mode} complete", flush=True)
    if digest(jar) != info["jar_sha256"] or digest(library) != info["native_sha256"]:
        raise RuntimeError("benchmark artifacts changed while measuring")
    summary = {}
    for key in medians["plain"]:
        summary[key] = {}
        for mode in MODES:
            values = medians[mode][key]
            times = [v["ns_per_call"] for v in values]
            summary[key][mode] = {"ns_per_call": statistics.median(times), "jvm_median_ns_min": min(times), "jvm_median_ns_max": max(times),
                                  "jvm_bytes_per_call": statistics.median(v["jvm_bytes_per_call"] for v in values),
                                  "median_batch_ms": statistics.median(v["batch_ms"] for v in values)}
        summary[key]["plain_fold_vs_bulk_list" if key.startswith("records_reduce/") else "bulk_speedup"] = summary[key]["plain"]["ns_per_call"] / summary[key]["bulk"]["ns_per_call"]
    cpu_model = next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')), None) if Path('/proc/cpuinfo').exists() else platform.processor()
    sample_file=output / "samples.csv"
    with sample_file.open("w", newline="") as stream:
        writer=csv.writer(stream)
        writer.writerow(["trial","mode","case","size","batch","iterations","elapsed_ns","jvm_allocated_bytes"])
        for run in runs:
            for key, samples in run["samples"].items():
                kind,size=key.split("/")
                for batch, sample in enumerate(samples):
                    writer.writerow([run["trial"],run["mode"],kind,size,batch,sample["iterations"],sample["elapsed_ns"],sample["jvm_allocated_bytes"]])
    report = {"build": info, "runner_sha256": digest(Path(__file__).resolve()), "host": {"platform": platform.platform(), "cpu_model": cpu_model,
              "affinity_cpus": sorted(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else None,
              "cpu_quota": Path('/sys/fs/cgroup/cpu.max').read_text().strip() if Path('/sys/fs/cgroup/cpu.max').exists() else None},
              "java": command_version(["java", "-version"]), "jvm_args": jvm_args, "sizes": sizes, "payload_bytes": payloads,
              "cases": selected, "jvms_per_mode": args.jvms, "warmup_batches": args.warmup, "measurement_batches": args.batches,
              "target_ms": args.target_ms, "correctness_only": args.check_jni,
              "run_order": [{"trial": r["trial"], "mode": r["mode"], "cases": r["case_order"], "effective_flags": r["effective_flags"]} for r in runs],
              "summary": summary, "jvm_medians": medians, "samples_csv": "samples.csv", "samples_sha256": digest(sample_file)}
    if args.check_jni:
        del report["summary"], report["jvm_medians"]
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    if args.check_jni:
        print(f"Correctness only: {2 * args.jvms} JVMs passed with -Xcheck:jni; timing summary suppressed.")
        return
    print("case/size | plain ns | bulk ns | plain/bulk | plain JVM B | bulk JVM B")
    for key, values in summary.items():
        ratio=values.get("bulk_speedup",values.get("plain_fold_vs_bulk_list"))
        label=key+(" [fold/list]" if key.startswith("records_reduce/") else "")
        print(f'{label}: {values["plain"]["ns_per_call"]:.1f} | {values["bulk"]["ns_per_call"]:.1f} | {ratio:.2f}x | '
              f'{values["plain"]["jvm_bytes_per_call"]:.1f} | {values["bulk"]["jvm_bytes_per_call"]:.1f}')


if __name__ == "__main__":
    main()
