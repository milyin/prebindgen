#!/usr/bin/env python3
"""Build both adapters, then execute actual C and Kotlin/JNI callers."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"]))
ids = {p["id"]: p["name"] for p in metadata["packages"]}
outputs = {}
# Cargo reports the exact output paths; never select a stale build directory.
process = subprocess.Popen(["cargo", "build", "--offline", "-p", "demo-c", "-p", "covertest-kotlin", "--message-format=json"], stdout=subprocess.PIPE, text=True)
for line in process.stdout:
    message = json.loads(line)
    if message["reason"] == "build-script-executed" and message["package_id"] in ids:
        outputs[ids[message["package_id"]]] = Path(message["out_dir"])
    elif message["reason"] == "compiler-message":
        print(message["message"]["rendered"], file=sys.stderr)
if process.wait():
    sys.exit(process.returncode)
build = Path(metadata["target_directory"]) / "boundary-tests"
build.mkdir(exist_ok=True)
lib = Path(metadata["target_directory"]) / "debug"
cc = [os.environ.get("CC", "cc"), "-std=c11", "-Wall", "-Wextra", "-Werror"]
if os.environ.get("SANITIZE") == "1":
    cc += ["-fsanitize=address,undefined", "-fno-omit-frame-pointer"]
subprocess.run(cc + ["-I" + str(outputs["demo-c"]), "examples/demo-c/test.c", "-L" + str(lib), "-ldemo_c", "-Wl,-rpath," + str(lib), "-o", str(build / "c-test")], check=True)
subprocess.run([str(build / "c-test")], check=True)
kotlin = sorted(outputs["covertest-kotlin"].joinpath("kotlin").rglob("*.kt"))
subprocess.run([os.environ.get("KOTLINC", "kotlinc"), *map(str, kotlin), "examples/covertest-kotlin/Test.kt", "-include-runtime", "-d", str(build / "test.jar")], check=True)
subprocess.run(["java", "-Xcheck:jni", "-Djava.library.path=" + str(lib), "-jar", str(build / "test.jar")], check=True)
