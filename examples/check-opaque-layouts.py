#!/usr/bin/env python3
"""Exercise external opaque-layout artifacts without building a probe tool."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)
HOST = next(
    line.split(": ", 1)[1]
    for line in subprocess.check_output(["rustc", "-vV"], text=True).splitlines()
    if line.startswith("host: ")
)
METADATA = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"], text=True
))
PACKAGES = {p["id"]: p["name"] for p in METADATA["packages"]}


def check(label, target, artifact=None, *, features=False, failure=None, package="perftest-c", binding_target=None):
    env = os.environ.copy()
    env.pop("PREBINDGEN_OPAQUE_LAYOUTS", None)
    env.pop("PREBINDGEN_BINDING_TARGET", None)
    if binding_target:
        env["PREBINDGEN_BINDING_TARGET"] = binding_target
    if artifact:
        env["PREBINDGEN_OPAQUE_LAYOUTS"] = str(artifact)
    command = ["cargo", "check", "--offline", "-p", package,
               "--target", target, "--message-format=json"]
    if features:
        command += ["--features", "layout-wide"]
    result = subprocess.run(command, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if failure:
        if result.returncode == 0 or failure not in result.stdout:
            raise RuntimeError(f"{label}: expected {failure!r}\n{result.stdout}")
    elif result.returncode:
        raise RuntimeError(f"{label}:\n{result.stdout}")
    header = None
    if result.returncode == 0:
        outputs = {}
        for line in result.stdout.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("reason") == "build-script-executed":
                outputs[PACKAGES.get(event["package_id"])] = Path(event["out_dir"])
        header = outputs[package] / ("owners.h" if package == "perftest-c" else "bindings.h")
        measurements = (json.loads(Path(artifact).read_text())
                        if artifact and package == "perftest-c" else None)
        verify_header(target, header, measurements)
    print(label + " PASS", flush=True)
    return header


def compile_header(target, language, source):
    clang = shutil.which("clang")
    if not clang:
        raise RuntimeError("clang is required to check target C/C++ header layouts")
    command = [clang, "--target=" + target, "-ffreestanding",
               "-std=" + ("c++11" if language == "c++" else "c11"),
               "-Wall", "-Wextra", "-Werror", "-fsyntax-only", "-x", language, "-"]
    return subprocess.run(command, input=source, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)


def verify_header(target, header, artifact):
    if "_TARGET " + json.dumps(target) not in header.read_text():
        raise RuntimeError("generated header has no matching target identity")
    for language in ["c", "c++"]:
        lines = ['#include "' + str(header) + '"']
        if artifact:
            for owner, key in [("bytes", "std.vec"), ("numbers", "std.vec"),
                               ("text", "std.string"), ("packet", "packet")]:
                layout = artifact["layouts"][key]
                assertion = "static_assert" if language == "c++" else "_Static_assert"
                alignment = "alignof" if language == "c++" else "_Alignof"
                lines.append(f'{assertion}(sizeof(owner_{owner})=={layout["size"]},"owner size");')
                lines.append(f'{assertion}({alignment}(owner_{owner})=={layout["alignment"]},"owner alignment");')
        result = compile_header(target, language, "\n".join(lines) + "\n")
        if result.returncode:
            raise RuntimeError(f"target header {target}:\n{result.stdout}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", action="append", default=[], help="installed cross target to check")
    args = parser.parse_args()
    fixture = ROOT / "examples/perftest-c/layouts" / (HOST + ".json")
    if not fixture.exists():
        raise RuntimeError(f"No checked example layout fixture for host {HOST}; supply one for this test platform")
    with tempfile.TemporaryDirectory(prefix="prebindgen-layout-check-") as temporary:
        directory = Path(temporary)
        data = json.loads(fixture.read_text())
        check("native automatic layout", HOST)
        check("native external layout", HOST, fixture)
        check("native feature-dependent automatic layout", HOST, features=True)
        check("probe feature mismatch", HOST, fixture, features=True, failure="opaque layout features")
        wide = json.loads(fixture.read_text())
        wide["sources"]["perftest-flat"]["features"] = ["layout-wide"]
        stale = directory / "stale.json"
        stale.write_text(json.dumps(wide))
        check("stale measurements despite matching features", HOST, stale, features=True,
              failure="opaque owner size differs from its Rust type")
        # These are checked fixture measurements, not a sizeof probe implementation.
        wide["layouts"]["packet"]["size"] += wide["layouts"]["packet"]["alignment"]
        valid = directory / "wide.json"
        valid.write_text(json.dumps(wide))
        check("matching feature-dependent external layout", HOST, valid, features=True)
        incomplete = directory / "incomplete.json"
        del data["layouts"]["std.string"]
        incomplete.write_text(json.dumps(data))
        check("incomplete native artifact", HOST, incomplete, failure="missing opaque layout `std.string`")
        for target in args.target:
            if target == HOST:
                continue
            artifact = fixture.with_name(target + ".json")
            if not artifact.exists():
                raise RuntimeError(f"No checked example layout fixture for {target}")
            check("pointer-only cross build " + target, target, package="example-cbindgen")
            check("Rust rejects a different binding target " + target, HOST,
                  package="example-cbindgen", binding_target=target,
                  failure="C binding target differs from the Rust compilation target")
            check("cross build requires external data " + target, target,
                  failure="requires an external opaque layout artifact")
            check("cross target mismatch " + target, target, fixture, failure="opaque layout target mismatch")
            header = check("cross external layout " + target, target, artifact)
            cross_data = json.loads(artifact.read_text())
            host_data = json.loads(fixture.read_text())
            if cross_data["layouts"]["std.vec"]["size"] != host_data["layouts"]["std.vec"]["size"]:
                for language in ["c", "c++"]:
                    result = compile_header(HOST, language, '#include "' + str(header) + '"\n')
                    if result.returncode == 0 or "header pointer width differs" not in result.stdout:
                        raise RuntimeError("wrong-width header was not rejected: " + result.stdout)
                print("wrong-width C/C++ header rejected " + target + " PASS", flush=True)
            cross_data["sources"]["perftest-flat"]["features"] = ["layout-wide"]
            cross_data["layouts"]["packet"]["size"] += cross_data["layouts"]["packet"]["alignment"]
            wide_path = directory / (target + "-wide.json")
            wide_path.write_text(json.dumps(cross_data))
            check("cross feature-dependent external layout " + target, target, wide_path, features=True)
            bad = json.loads(artifact.read_text())
            del bad["layouts"]["packet"]
            path = directory / (target + "-incomplete.json")
            path.write_text(json.dumps(bad))
            check("cross incomplete artifact " + target, target, path, failure="missing opaque layout `packet`")
    print("Opaque layout build checks PASS", flush=True)


if __name__ == "__main__":
    main()
