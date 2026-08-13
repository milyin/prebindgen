use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn cargo(target: &Path, args: &[&str]) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    Command::new(cargo)
        .current_dir(workspace_root())
        .env("CARGO_TARGET_DIR", target)
        .env("CARGO_TERM_COLOR", "never")
        .args(args)
        .output()
        .unwrap()
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn visit_directories(path: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let path = entry.path();
            if path.file_name().is_some_and(|name| name == "prebindgen")
                && fs::read_to_string(path.join("crate_name.txt"))
                    .map(|crate_name| crate_name == "example-flat")
                    .unwrap_or(false)
            {
                found.push(path);
            } else {
                visit_directories(&path, found);
            }
        }
    }
}

fn example_capture_dirs(target: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    visit_directories(target, &mut found);
    found.sort();
    found
}

fn capture_files(directory: &Path) -> BTreeMap<OsString, Vec<u8>> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_owned(),
                fs::read(path).unwrap(),
            )
        })
        .collect()
}

#[test]
fn feature_hashes_clean_regenerate_and_stay_warm() {
    let target = tempfile::tempdir().unwrap();
    for features in [None, Some("unstable"), Some("internal")] {
        let mut args = vec!["check", "-p", "example-flat"];
        if let Some(features) = features {
            args.extend(["--features", features]);
        }
        let output = cargo(target.path(), &args);
        assert_success(&output, &format!("initial build with {features:?}"));
    }
    let output = cargo(
        target.path(),
        &["check", "-p", "example-flat", "--all-features"],
    );
    assert_success(&output, "initial all-features build");
    assert_eq!(example_capture_dirs(target.path()).len(), 4);

    let cleaner = env!("CARGO_BIN_EXE_cargo-prebindgen");
    let dry_run = Command::new(cleaner)
        .args(["clean", "--dry-run", "--target-dir"])
        .arg(target.path())
        .output()
        .unwrap();
    assert_success(&dry_run, "cleanup dry-run");
    assert_eq!(example_capture_dirs(target.path()).len(), 4);

    let clean = Command::new(cleaner)
        .args(["clean", "--target-dir"])
        .arg(target.path())
        .output()
        .unwrap();
    assert_success(&clean, "cleanup");
    assert!(example_capture_dirs(target.path()).is_empty());

    let rebuilt = cargo(
        target.path(),
        &[
            "check",
            "-p",
            "example-flat",
            "--features",
            "unstable",
            "-vv",
        ],
    );
    assert_success(&rebuilt, "post-clean rebuild");
    let diagnostics = String::from_utf8_lossy(&rebuilt.stderr);
    assert!(
        diagnostics.contains("Dirty example-flat")
            && diagnostics.contains(".prebindgen-capture-state-v1-"),
        "producer was not invalidated by a state slot:\n{diagnostics}"
    );

    let directories = example_capture_dirs(target.path());
    assert_eq!(directories.len(), 1);
    let before = capture_files(&directories[0]);
    assert_eq!(
        before
            .keys()
            .filter(|name| Path::new(name)
                .extension()
                .is_some_and(|ext| ext == "jsonl"))
            .count(),
        2
    );
    for (name, contents) in &before {
        if Path::new(name)
            .extension()
            .is_some_and(|ext| ext == "jsonl")
        {
            assert!(!contents.is_empty());
            for line in contents
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
            {
                serde_json::from_slice::<serde_json::Value>(line).unwrap();
            }
        }
    }

    let warm = cargo(
        target.path(),
        &[
            "check",
            "-p",
            "example-flat",
            "--features",
            "unstable",
            "-vv",
        ],
    );
    assert_success(&warm, "warm post-clean rebuild");
    assert!(
        String::from_utf8_lossy(&warm.stderr).contains("Fresh example-flat"),
        "producer was unexpectedly rebuilt:\n{}",
        String::from_utf8_lossy(&warm.stderr)
    );
    assert_eq!(example_capture_dirs(target.path()).len(), 1);
    assert_eq!(capture_files(&directories[0]), before);
}
