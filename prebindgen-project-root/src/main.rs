use std::{env, fs, path::{Path, PathBuf}};

use anyhow::{anyhow, bail, Context, Result};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

const USAGE: &str = "prebindgen-project-root

Usage:
  prebindgen-project-root help
  prebindgen-project-root install <path>

Commands:
  help                 Show this help.
  install <path>       Install local copy of prebindgen-project-root crate into the given Cargo workspace.

Details:
  - <path> may be either a path to a workspace root directory (containing Cargo.toml with [workspace])
    or a path directly to that workspace's Cargo.toml file.
  - This will:
      * add a new member crate named 'prebindgen-project-root' inside the workspace
      * place build.rs and lib.rs into that crate (no main.rs)
      * add a [patch.crates-io] to point prebindgen-project-root to the local path
";

fn main() {
    if let Err(err) = real_main() {
        eprintln!("error: {:#}", err);
        std::process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || matches!(args[0].as_str(), "help" | "-h" | "--help") {
        println!("{}", USAGE);
        return Ok(());
    }

    match args.remove(0).as_str() {
        "install" => {
            let path = args.get(0).ok_or_else(|| anyhow!("install requires <path>"))?;
            install(Path::new(path))
        }
        other => bail!("unknown command: {}\n\n{}", other, USAGE),
    }
}

fn install(input: &Path) -> Result<()> {
    let (ws_root, ws_manifest_path) = resolve_workspace_root_and_manifest(input)?;

    // 1) Ensure destination crate directory
    let local_crate_dir = ws_root.join("prebindgen-project-root");
    if local_crate_dir.exists() {
        // If exists, ensure it's a directory and has src/. If not, bail to avoid clobbering.
        if !local_crate_dir.is_dir() {
            bail!("Target path exists and is not a directory: {}", local_crate_dir.display());
        }
    } else {
        fs::create_dir_all(local_crate_dir.join("src")).with_context(|| format!(
            "creating crate dir {}",
            local_crate_dir.display()
        ))?;
    }

    // 2) Write Cargo.toml for the local crate (library only)
    let local_cargo = local_crate_dir.join("Cargo.toml");
    if !local_cargo.exists() {
        let cargo_toml = r#"[package]
name = "prebindgen-project-root"
version = "0.4.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Utility to expose the workspace project root at build time"

[lib]
name = "prebindgen_project_root"
path = "src/lib.rs"

[build-dependencies]
project-root = "0.2"
"#;
        fs::write(&local_cargo, cargo_toml).with_context(|| format!("writing {}", local_cargo.display()))?;
    }

    // 3) Write lib.rs and build.rs content from this package's templates
    // We embed our own canonical lib.rs and build.rs to install.
    let lib_rs = include_str!("./lib.rs.template");
    let build_rs = include_str!("./build.rs.template");

    fs::write(local_crate_dir.join("src/lib.rs"), lib_rs)
        .with_context(|| format!("writing {}", local_crate_dir.join("src/lib.rs").display()))?;
    fs::write(local_crate_dir.join("build.rs"), build_rs)
        .with_context(|| format!("writing {}", local_crate_dir.join("build.rs").display()))?;

    // 4) Update workspace Cargo.toml: add member and [patch.crates-io]
    add_member_and_patch(&ws_manifest_path, &local_crate_dir)?;

    println!(
        "Installed local 'prebindgen-project-root' crate at: {}\nUpdated workspace at: {}",
        local_crate_dir.display(),
        ws_manifest_path.display()
    );
    Ok(())
}

fn resolve_workspace_root_and_manifest(input: &Path) -> Result<(PathBuf, PathBuf)> {
    let p = if input.is_dir() { input.join("Cargo.toml") } else { input.to_path_buf() };
    if !p.exists() {
        bail!("Path does not exist: {}", p.display());
    }
    if p.is_dir() {
        bail!("Expected a Cargo.toml file or a directory containing one: {}", p.display());
    }
    // Read and parse
    let text = fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
    let doc: DocumentMut = text
        .parse()
        .with_context(|| format!("parsing TOML at {}", p.display()))?;
    if !doc.contains_key("workspace") {
        bail!("Cargo.toml is not a workspace manifest; missing [workspace] at {}", p.display());
    }

    // Determine workspace root directory
    let ws_root = p.parent().ok_or_else(|| anyhow!("manifest has no parent: {}", p.display()))?.to_path_buf();
    Ok((ws_root, p))
}

fn add_member_and_patch(ws_manifest_path: &Path, local_crate_dir: &Path) -> Result<()> {
    let mut text = fs::read_to_string(ws_manifest_path)
        .with_context(|| format!("reading {}", ws_manifest_path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("parsing TOML at {}", ws_manifest_path.display()))?;

    // Ensure [workspace]
    let ws = doc["workspace"].or_insert(Item::Table(Table::new()));
    let ws_tbl = ws.as_table_mut().expect("workspace to be a table");

    // Ensure members array includes "prebindgen-project-root"
    let members = ws_tbl.entry("members").or_insert(Item::Value(Value::Array(Array::default())));
    if let Some(arr) = members.as_array_mut() {
        let exists = arr.iter().any(|v| v.as_str() == Some("prebindgen-project-root"));
        if !exists {
            arr.push("prebindgen-project-root");
        }
    } else {
        // Replace with array if not array
        let mut arr = Array::default();
        arr.push("prebindgen-project-root");
        ws_tbl["members"] = Item::Value(Value::Array(arr));
    }

    // Add [patch.crates-io]
    let patch = doc["patch"].or_insert(Item::Table(Table::new()));
    let patch_tbl = patch.as_table_mut().unwrap();
    let crates_io = patch_tbl.entry("crates-io").or_insert(Item::Table(Table::new()));
    let crates_io_tbl = crates_io.as_table_mut().unwrap();

    let rel_path = pathdiff::diff_paths(local_crate_dir, ws_manifest_path.parent().unwrap())
        .unwrap_or_else(|| local_crate_dir.to_path_buf());
    let rel_str = rel_path.to_string_lossy().replace('\\', "/");

    // prebindgen-project-root = { path = "..." }
    let mut path_table = Table::new();
    path_table.insert("path", toml_edit::value(rel_str));
    crates_io_tbl.insert("prebindgen-project-root", Item::Table(path_table));

    // Write back
    text = doc.to_string();
    fs::write(ws_manifest_path, text)
        .with_context(|| format!("writing {}", ws_manifest_path.display()))?;
    Ok(())
}

