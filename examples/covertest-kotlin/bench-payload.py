#!/usr/bin/env python3
"""Measure the payload covertest cases with a small, matching generated library."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--target-dir", type=Path)
parser.add_argument("--kotlinc", default=os.environ.get("KOTLINC", "kotlinc"))
parser.add_argument("--forks", type=int, default=3)
parser.add_argument("--count-native", action="store_true",
                    help="Separate instrumented allocation run; not a latency measurement")
args = parser.parse_args()
out = args.output.resolve()
fixture = out / "fixture"
(fixture / "src").mkdir(parents=True, exist_ok=True)

def dependency(name, path=None):
    return f'{name}={{path={json.dumps(str(ROOT / (path or name)))}}}\n'

(fixture / "Cargo.toml").write_text('''[package]
name="payload-bench"
version="0.0.0"
edition="2021"
[lib]
crate-type=["cdylib"]
[features]
native-allocations=[]
[dependencies]
konst="0.3.0"
''' + dependency("perftest-flat", "examples/perftest-flat")
    + dependency("prebindgen-runtime") + dependency("prebindgen-jni-runtime")
    + '[build-dependencies]\n' + dependency("perftest-flat", "examples/perftest-flat")
    + dependency("prebindgen-jni") + dependency("prebindgen-declarations")
    + '[workspace]\n')
(fixture / "build.rs").write_text(r'''
use prebindgen_jni::{JniGen,package,ptr_class,sealed_class,fun,JniInput as Input};
use prebindgen_jni::prelude::*;
fn main() {
    let generated=JniGen::builder().source(perftest_flat::PREBINDGEN_OUT_DIR)
        .set_package_prefix("io.prebindgen.covertest")
        .set_jni_native_init(r#"System.loadLibrary("payload_bench")"#)
        .package(package!("callbackreturns").class(ptr_class!(CallbackReturnResource)))
        .package(package!("payloadenums")
            .class(sealed_class!(WireBench)).class(sealed_class!(WireOwnedEvent))
            .fun(fun!(wire_manual_total)).fun(fun!(wire_bench_total))
            .fun(fun!(wire_bench_scalar_bulk).expand_param("value",Input::value().bulk()))
            .fun(fun!(wire_manual_owned_bench)).fun(fun!(wire_owned_bench))
            .fun(fun!(wire_bench_objects))
            .fun(fun!(wire_bench_bulk).expand_param("values",Input::sequence(Input::value()).bulk())))
        .build().unwrap();
    generated.write_rust("bindings.rs").unwrap();
    generated.write_kotlin(&std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("kotlin")).unwrap();
}
''')
(fixture / "src/lib.rs").write_text('''#![allow(non_camel_case_types,unused_variables)]
mod native_allocations;
include!(concat!(env!("OUT_DIR"),"/bindings.rs"));
''')
shutil.copyfile(ROOT / "examples/bench-bulk/native_allocations.rs",
                fixture / "src/native_allocations.rs")
env = dict(os.environ, CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0",
           CARGO_TARGET_DIR=str(args.target_dir.resolve() if args.target_dir else out / "target"))
command = ["cargo", "build", "--release", "--offline", "--message-format=json"]
if args.count_native:
    command += ["--features", "native-allocations"]
with (out / "cargo.jsonl").open("w") as log:
    subprocess.run(command, cwd=fixture, env=env, stdout=log, check=True)
messages = [json.loads(line) for line in (out / "cargo.jsonl").read_text().splitlines()]
package = next(m["package_id"] for m in messages if m.get("target", {}).get("name") == "payload_bench")
generated = next(Path(m["out_dir"]) for m in messages if m.get("reason") == "build-script-executed" and m["package_id"] == package)
artifact = next(m for m in messages if m.get("reason") == "compiler-artifact" and m["package_id"] == package and "cdylib" in m["target"]["kind"])
if artifact["profile"]["opt_level"] != "3" or artifact["profile"]["debug_assertions"]:
    raise RuntimeError("benchmark requires the optimized release profile")
library = Path(artifact["filenames"][0])
shutil.copyfile(library, out / library.name)
kotlin = out / "kotlin"
shutil.rmtree(kotlin, ignore_errors=True)
shutil.copytree(generated / "kotlin", kotlin)
shutil.copyfile(ROOT / "examples/covertest-kotlin/kotlin/io/prebindgen/covertest/PayloadBench.kt", kotlin / "PayloadBench.kt")
(kotlin / "Counters.kt").write_text('''package bench.allocations
internal object Counters {
    @JvmStatic external fun start():Boolean
    @JvmStatic external fun stop():LongArray
}
''')
(kotlin / "Main.kt").write_text('''package io.prebindgen.covertest
fun main(args:Array<String>) {
    if(args.isEmpty()) { benchmarkPayloadEnums(); return }
    for((name,_,call) in payloadBenchCases()) {
        repeat(1000) { call() }
        check(bench.allocations.Counters.start())
        repeat(10000) { call() }
        val counts=bench.allocations.Counters.stop()
        println("$name: ${counts[0]/10000.0} native allocations/call, ${counts[1]/10000.0} reallocations/call, ${counts[2]/10000.0} native bytes/call")
    }
}
''')
jar = out / "payload.jar"
subprocess.run([args.kotlinc, *map(str, sorted(kotlin.rglob("*.kt"))), "-include-runtime", "-d", str(jar)],
               env=dict(os.environ, JAVA_OPTS="-Xms128m -Xmx768m"), check=True)
for fork in range(args.forks):
    command = ["java", "-Xms512m", "-Xmx512m", "-Djava.library.path=" + str(out), "-jar", str(jar)]
    if args.count_native:
        command += ["allocations"]
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
    (out / f"fork-{fork}.log").write_text(result.stdout)
    print(f"fork {fork}\n{result.stdout}", flush=True)
