use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use std::process::Command;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tempfile::{Builder as TempDirBuilder, TempDir};
use toml_edit::DocumentMut;

fn crate_root() -> PathBuf {
    // This test file is in the prebindgen-project-root crate
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn tests_resources_dir() -> PathBuf {
    crate_root().join("tests").join("resources")
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn keep_tmp_enabled() -> bool {
    env::var("PREBINDGEN_KEEP_TMP")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn make_tmp(prefix: &str) -> (TempDir, PathBuf) {
    let tmp = TempDirBuilder::new()
        .prefix(prefix)
        .tempdir_in(Path::new("/tmp"))
        .expect("tempdir in /tmp");
    let tmp_path = tmp.path().to_path_buf();
    (tmp, tmp_path)
}

fn copy_fixture(tmp_root: &Path, fixture: &str) -> PathBuf {
    let src = tests_resources_dir().join(fixture);
    let dst = tmp_root.join(fixture);
    copy_dir_all(&src, &dst).expect("copy fixture");
    eprintln!("[test] {} copied to: {}", fixture, dst.display());
    dst
}

fn run_install(manifest: &Path) {
    let bin = cargo_bin("cargo-prebindgen-project-root");
    Command::new(&bin)
        .arg("install")
        .arg(manifest)
        .assert()
        .success();
}

fn read_manifest_doc(manifest: &Path) -> DocumentMut {
    let manifest_text = fs::read_to_string(manifest).expect("read manifest");
    manifest_text.parse().expect("parse manifest as toml")
}

fn assert_workspace_members(doc: &DocumentMut, expected: &[&str]) {
    assert!(
        doc.contains_key("workspace"),
        "workspace section should exist"
    );
    let members = doc["workspace"]["members"]
        .as_array()
        .expect("members array");
    for m in expected {
        assert!(
            members.iter().any(|v| v.as_str() == Some(m)),
            "workspace members should include '{}'",
            m
        );
    }
}

fn assert_helper_patch(doc: &DocumentMut) {
    let patch_tbl = doc["patch"]["crates-io"]
        .as_table()
        .expect("patch.crates-io table");
    let helper = patch_tbl
        .get("prebindgen-project-root")
        .expect("patch entry for helper crate");
    let helper_tbl = helper.as_table().expect("helper patch table");
    let path_value = helper_tbl
        .get("path")
        .and_then(|i| i.as_str())
        .expect("path value");
    eprintln!(
        "[test] patch crates-io.prebindgen-project-root.path = {}",
        path_value
    );
    assert!(
        path_value.contains("prebindgen-project-root"),
        "path should reference 'prebindgen-project-root'"
    );
}

fn assert_helper_files_exist(base_dir: &Path) {
    let local_helper_dir = base_dir.join("prebindgen-project-root");
    eprintln!(
        "[test] installed helper dir: {}",
        local_helper_dir.display()
    );
    assert!(
        local_helper_dir.join("src/lib.rs").exists(),
        "src/lib.rs should exist"
    );
    assert!(
        local_helper_dir.join("build.rs").exists(),
        "build.rs should exist"
    );
}

fn maybe_keep_tmp(tmp: TempDir) {
    if keep_tmp_enabled() {
        let kept_path = tmp.keep();
        eprintln!("[test] kept temp dir at: {}", kept_path.display());
    }
}

fn cargo_run_and_assert_workspace(dir: &Path, pkg: Option<&str>) {
    let pkg_suffix = pkg.map(|p| format!(" -p {}", p)).unwrap_or_default();
    eprintln!(
        "[test] running `cargo run{}` in {}",
        pkg_suffix,
        dir.display()
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet");
    if let Some(p) = pkg {
        cmd.arg("-p").arg(p);
    }
    let output = cmd
        .current_dir(dir)
        .output()
        .expect("failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo run failed.\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr
    );

    let printed = stdout.trim();
    let expected = fs::canonicalize(dir).expect("canonicalize workspace dir");
    let expected_str = expected.to_string_lossy();
    eprintln!("[test] program printed: {}", printed);
    eprintln!("[test] expected root  : {}", expected_str);
    assert_eq!(
        printed,
        expected_str,
        "printed workspace root does not match expected"
    );
}

#[test]
fn installs_into_standalone_crate_from_resources() {
    let (tmp, tmp_root) = make_tmp("prebindgen-test-");
    let dst_pkg = copy_fixture(&tmp_root, "simple-package");
    let manifest = dst_pkg.join("Cargo.toml");

    run_install(&manifest);

    let doc: DocumentMut = read_manifest_doc(&manifest);
    assert_workspace_members(&doc, &[".", "prebindgen-project-root"]);
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_pkg);

    // Running the package should print its workspace root path (the package dir itself)
    cargo_run_and_assert_workspace(&dst_pkg, None);

    maybe_keep_tmp(tmp);
}

#[test]
fn installs_into_existing_workspace_from_resources() {
    let (tmp, tmp_root) = make_tmp("prebindgen-test-");
    let dst_ws = copy_fixture(&tmp_root, "workspace-package");
    let ws_manifest = dst_ws.join("Cargo.toml");

    run_install(&ws_manifest);

    let doc: DocumentMut = read_manifest_doc(&ws_manifest);
    assert_workspace_members(&doc, &["simple-member", "prebindgen-project-root"]);
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_ws);

    // Running the workspace member should print the workspace root path (the workspace dir)
    cargo_run_and_assert_workspace(&dst_ws, Some("simple_member"));

    maybe_keep_tmp(tmp);
}

#[test]
fn installs_into_simple_workspace_root_from_resources() {
    // Workspace with [workspace] in the root manifest and a root package
    let (tmp, tmp_root) = make_tmp("prebindgen-test-");
    let dst_ws = copy_fixture(&tmp_root, "simple-workspace-package");
    let ws_manifest = dst_ws.join("Cargo.toml");

    run_install(&ws_manifest);

    let doc: DocumentMut = read_manifest_doc(&ws_manifest);
    // Expect the helper crate to be added as a workspace member
    assert_workspace_members(&doc, &["prebindgen-project-root"]);
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_ws);

    // Running the root package should print the workspace root path (the workspace dir)
    cargo_run_and_assert_workspace(&dst_ws, None);

    maybe_keep_tmp(tmp);
}
