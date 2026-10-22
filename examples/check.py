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
process = subprocess.Popen(["cargo", "build", "--offline", "--workspace", "--message-format=json"], stdout=subprocess.PIPE, text=True)
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
for name in ["example-cbindgen", "perftest-c"]:
    executable = build / name
    subprocess.run(cc + ["-I" + str(outputs[name]), f"examples/{name}/c/smoke.c", "-L" + str(lib), "-l" + name.replace("-", "_"), "-Wl,-rpath," + str(lib), "-o", str(executable)], check=True)
    subprocess.run([str(executable)], check=True)
# The C compiler must reject crossing distinct opaque pointer types.
wrong_type = build / "wrong-pointer.c"
wrong_type.write_text('#include "bindings.h"\nvoid wrong(perftest_storage *p) { perftest_drop_blob(p); }\n')
rejected = subprocess.run(cc + ["-fsyntax-only", "-I" + str(outputs["perftest-c"]), str(wrong_type)], capture_output=True, text=True)
if rejected.returncode == 0 or "incompatible" not in rejected.stderr:
    raise RuntimeError("C compiler did not diagnose incompatible opaque pointers: " + rejected.stderr)
for name in ["covertest-kotlin", "perftest-kotlin"]:
    kotlin = sorted(outputs[name].joinpath("kotlin").rglob("*.kt"))
    kotlin += sorted(Path(f"examples/{name}/kotlin").rglob("*.kt"))
    jar = build / (name + ".jar")
    subprocess.run([os.environ.get("KOTLINC", "kotlinc"), *map(str, kotlin), "-include-runtime", "-d", str(jar)], check=True)
    subprocess.run(["java", "-Xcheck:jni", "-Djava.library.path=" + str(lib), "-jar", str(jar)], check=True)

# Java callers can recover through the same typed domain-error callback.
java_test = build / "ResultJava.java"
java_test.write_text("""public class ResultJava {
    public static void main(String[] args) {
        long recovered = io.prebindgen.covertest.results.FunctionsKt.resultScalar(
            -7L, message -> { throw new AssertionError(message); }, code -> -code);
        if (recovered != 7L) throw new AssertionError(recovered);
    }
}
""")
classpath = str(build / "covertest-kotlin.jar")
subprocess.run(["javac", "-encoding", "UTF-8", "-cp", classpath, "-d", str(build), str(java_test)], check=True)
subprocess.run(["java", "-Xcheck:jni", "-Djava.library.path=" + str(lib), "-cp", classpath + os.pathsep + str(build), "ResultJava"], check=True)
