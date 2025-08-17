use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use std::{env, fs, path::{Path, PathBuf}};
use std::process::Command;
use tempfile::Builder as TempDirBuilder;
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

#[test]
fn installs_into_standalone_crate_from_resources() {
    // 1) Copy dummy simple package into a temp dir
    let tmp = TempDirBuilder::new()
        .prefix("prebindgen-test-")
        .tempdir_in(Path::new("/tmp"))
        .expect("tempdir in /tmp");
    // Use owned PathBuf so we can later move `tmp` via into_path() safely.
    let tmp_path = tmp.path().to_path_buf();

    let src_pkg = tests_resources_dir().join("simple-package");
    let dst_pkg = tmp_path.join("simple-package");
    copy_dir_all(&src_pkg, &dst_pkg).expect("copy simple package");

    let manifest = dst_pkg.join("Cargo.toml");
    eprintln!("[test] simple package copied to: {}", dst_pkg.display());

    // 2) Run installer binary against the crate manifest (creates workspace, installs helper)
    let bin = cargo_bin("prebindgen-project-root");
    Command::new(&bin)
        .arg("install")
        .arg(&manifest)
        .assert()
        .success();

    // 3) Validate changes: workspace created with '.' + helper member, patch present
    let manifest_text = fs::read_to_string(&manifest).expect("read manifest");
    let doc: DocumentMut = manifest_text.parse().expect("parse manifest as toml");
    assert!(doc.contains_key("workspace"), "workspace section should exist");
    let members = doc["workspace"]["members"].as_array().expect("members array");
    let has_dot = members.iter().any(|v| v.as_str() == Some("."));
    let has_helper_member = members.iter().any(|v| v.as_str() == Some("prebindgen-project-root"));
    assert!(has_dot, "workspace members should include '.'");
    assert!(has_helper_member, "workspace members should include 'prebindgen-project-root'");

    let patch_tbl = doc["patch"]["crates-io"].as_table().expect("patch.crates-io table");
    let helper = patch_tbl.get("prebindgen-project-root").expect("patch entry for helper crate");
    let helper_tbl = helper.as_table().expect("helper patch table");
    let path_value = helper_tbl.get("path").and_then(|i| i.as_str()).expect("path value");
    eprintln!("[test] patch crates-io.prebindgen-project-root.path = {}", path_value);
    assert!(path_value.contains("prebindgen-project-root"), "path should reference 'prebindgen-project-root'");

    // Files exist in the new local crate under the dummy crate dir
    let local_helper_dir = dst_pkg.join("prebindgen-project-root");
    eprintln!("[test] installed helper dir: {}", local_helper_dir.display());
    assert!(local_helper_dir.join("src/lib.rs").exists(), "src/lib.rs should exist");
    assert!(local_helper_dir.join("build.rs").exists(), "build.rs should exist");

    // Optionally keep the temp dir for manual inspection
    if keep_tmp_enabled() {
        let kept_path = tmp.keep();
        eprintln!("[test] kept temp dir at: {}", kept_path.display());
    }
}

#[test]
fn installs_into_existing_workspace_from_resources() {
    // 1) Copy dummy workspace into a temp dir
    let tmp = TempDirBuilder::new()
        .prefix("prebindgen-test-")
        .tempdir_in(Path::new("/tmp"))
        .expect("tempdir in /tmp");
    // Use owned PathBuf so we can later move `tmp` via into_path() safely.
    let tmp_path = tmp.path().to_path_buf();

    let src_ws = tests_resources_dir().join("workspace-package");
    let dst_ws = tmp_path.join("workspace-package");
    copy_dir_all(&src_ws, &dst_ws).expect("copy workspace package");

    let ws_manifest = dst_ws.join("Cargo.toml");
    eprintln!("[test] workspace package copied to: {}", dst_ws.display());

    // 2) Run installer binary against workspace manifest (adds helper member + patch)
    let bin = cargo_bin("prebindgen-project-root");
    Command::new(&bin)
        .arg("install")
        .arg(&ws_manifest)
        .assert()
        .success();

    // 3) Validate changes: workspace members include existing member and helper, patch present
    let manifest_text = fs::read_to_string(&ws_manifest).expect("read manifest");
    let doc: DocumentMut = manifest_text.parse().expect("parse manifest as toml");
    assert!(doc.contains_key("workspace"), "workspace section should exist");
    let members = doc["workspace"]["members"].as_array().expect("members array");
    let has_member = members.iter().any(|v| v.as_str() == Some("simple-member"));
    let has_helper_member = members.iter().any(|v| v.as_str() == Some("prebindgen-project-root"));
    assert!(has_member, "workspace members should include 'simple-member'");
    assert!(has_helper_member, "workspace members should include 'prebindgen-project-root'");

    let patch_tbl = doc["patch"]["crates-io"].as_table().expect("patch.crates-io table");
    let helper = patch_tbl.get("prebindgen-project-root").expect("patch entry for helper crate");
    let helper_tbl = helper.as_table().expect("helper patch table");
    let path_value = helper_tbl.get("path").and_then(|i| i.as_str()).expect("path value");
    eprintln!("[test] patch crates-io.prebindgen-project-root.path = {}", path_value);
    assert!(path_value.contains("prebindgen-project-root"), "path should reference 'prebindgen-project-root'");

    // Files exist under workspace root
    let local_helper_dir = dst_ws.join("prebindgen-project-root");
    eprintln!("[test] installed helper dir: {}", local_helper_dir.display());
    assert!(local_helper_dir.join("src/lib.rs").exists(), "src/lib.rs should exist");
    assert!(local_helper_dir.join("build.rs").exists(), "build.rs should exist");

    // Optionally keep the temp dir for manual inspection
    if keep_tmp_enabled() {
        let kept_path = tmp.keep();
        eprintln!("[test] kept temp dir at: {}", kept_path.display());
    }
}
