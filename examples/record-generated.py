#!/usr/bin/env python3
"""Record or verify the example bindings compiled by Cargo for PR review."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
FILES = {
    "example-cbindgen": ("bindings.rs", "bindings.h"),
    "perftest-c": ("bindings.rs", "bindings.h", "owners.rs", "owners.h"),
    "covertest-kotlin": ("bindings.rs", "kotlin"),
    "perftest-kotlin": ("bindings.rs", "kotlin"),
    "emitcheck": ("bindings.rs",),
}


def build_outputs(root):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"], cwd=root))
    names = {package["id"]: package["name"] for package in metadata["packages"]}
    process = subprocess.Popen(
        ["cargo", "build", "--offline", "--workspace", "--message-format=json"],
        cwd=root, stdout=subprocess.PIPE, text=True)
    outputs = {}
    for line in process.stdout:
        message = json.loads(line)
        if message["reason"] == "build-script-executed":
            name = names.get(message["package_id"])
            if name in FILES:
                outputs[name] = Path(message["out_dir"])
        elif message["reason"] == "compiler-message":
            print(message["message"]["rendered"], file=sys.stderr)
    if process.wait():
        raise SystemExit(process.returncode)
    missing = FILES.keys() - outputs.keys()
    if missing:
        raise RuntimeError("Cargo did not report generated output for: " + ", ".join(sorted(missing)))
    return outputs


def generated_files(outputs):
    files = {}
    for name, entries in FILES.items():
        destination = Path("examples") / name / "generated"
        for entry in entries:
            if entry == "kotlin":
                # Build scripts replace this generator-owned tree before writing.
                for path in sorted((outputs[name] / entry).rglob("*.kt")):
                    relative = path.relative_to(outputs[name] / entry)
                    files[destination / entry / relative] = path.read_bytes()
            else:
                files[destination / entry] = (outputs[name] / entry).read_bytes()
    return files


def existing_files(root):
    return {path.relative_to(root)
            for name in FILES
            for path in (root / "examples" / name / "generated").rglob("*")
            if path.is_file()}


def record(root, files):
    # These five generated/ trees are owned by this command; handwritten files
    # live outside them. Remove obsolete output when a declaration is removed.
    for stale in existing_files(root) - files.keys():
        (root / stale).unlink()
    for relative, content in files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.is_file() or path.read_bytes() != content:
            path.write_bytes(content)


def check(root, files):
    existing = existing_files(root)
    missing = files.keys() - existing
    stale = existing - files.keys()
    changed = {path for path in existing & files.keys()
               if (root / path).read_bytes() != files[path]}
    for label, paths in [("missing", missing), ("obsolete", stale), ("changed", changed)]:
        for path in sorted(paths):
            print(f"{label}: {path}", file=sys.stderr)
    if missing or stale or changed:
        print("Run python3 examples/record-generated.py and commit the generated changes.", file=sys.stderr)
        return False
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify snapshots without changing them")
    args = parser.parse_args()
    files = generated_files(build_outputs(ROOT))
    if args.check:
        if not check(ROOT, files):
            raise SystemExit(1)
        print(f"Generated bindings match: {len(files)} files")
    else:
        record(ROOT, files)
        print(f"Recorded generated bindings: {len(files)} files")


if __name__ == "__main__":
    main()
