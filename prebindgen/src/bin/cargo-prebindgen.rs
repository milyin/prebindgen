use std::{
    collections::BTreeSet,
    env,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use fs2::FileExt;
use prebindgen::capture_protocol::{advance_state, validate_capture_dir, PREBINDGEN_DIR};
use serde::Deserialize;

const CARGO_LOCKS_IN_ORDER: [&str; 2] = [".cargo-build-lock", ".cargo-lock"];
const TOMBSTONE_PREFIX: &str = ".prebindgen-cleaning-v1-";
static TOMBSTONE_ID: AtomicU64 = AtomicU64::new(0);

fn main() {
    if let Err(error) = run() {
        eprintln!("cargo-prebindgen: {error}");
        std::process::exit(2);
    }
}

#[derive(Debug, Default)]
struct Options {
    dry_run: bool,
    version_only: bool,
    manifest_path: Option<PathBuf>,
    roots: Vec<PathBuf>,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let Some(options) = parse_args(env::args_os().skip(1).collect())? else {
        print_help();
        return Ok(());
    };
    if options.version_only {
        return Ok(());
    }

    let roots = if options.roots.is_empty() {
        vec![metadata_target_dir(options.manifest_path.as_deref())?]
    } else {
        options.roots
    };
    let report = clean_roots(&roots, options.dry_run)?;

    let verb = if options.dry_run {
        "Would remove"
    } else {
        "Removed"
    };
    println!(
        "{verb} {} managed prebindgen capture director{}.",
        report.managed,
        if report.managed == 1 { "y" } else { "ies" }
    );
    if report.interrupted != 0 {
        println!(
            "{} interrupted cleanup tombstone{} included.",
            report.interrupted,
            if report.interrupted == 1 {
                " was"
            } else {
                "s were"
            }
        );
    }
    if report.skipped != 0 {
        println!(
            "Skipped {} unrecognized or malformed prebindgen director{}.",
            report.skipped,
            if report.skipped == 1 { "y" } else { "ies" }
        );
    }
    for path in report.paths {
        println!("  {}", path.display());
    }
    Ok(())
}

fn parse_args(mut args: Vec<OsString>) -> Result<Option<Options>, String> {
    if args.first().is_some_and(|arg| arg == "prebindgen") {
        args.remove(0);
    }
    if args.is_empty()
        || args
            .first()
            .is_some_and(|arg| arg == "-h" || arg == "--help")
    {
        return Ok(None);
    }
    if args
        .first()
        .is_some_and(|arg| arg == "-V" || arg == "--version")
    {
        println!("cargo-prebindgen {}", env!("CARGO_PKG_VERSION"));
        return Ok(Some(Options {
            version_only: true,
            ..Options::default()
        }));
    }
    if args.remove(0) != "clean" {
        return Err("expected clean (try cargo prebindgen --help)".to_string());
    }

    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].to_str() {
            Some("--dry-run") => options.dry_run = true,
            Some("--target-dir" | "--build-dir") => {
                index += 1;
                let path = args.get(index).ok_or_else(|| {
                    "missing path after target/build directory option".to_string()
                })?;
                options.roots.push(PathBuf::from(path));
            }
            Some("--manifest-path") => {
                index += 1;
                let path = args
                    .get(index)
                    .ok_or_else(|| "missing path after --manifest-path".to_string())?;
                if options.manifest_path.replace(PathBuf::from(path)).is_some() {
                    return Err("--manifest-path may be specified only once".to_string());
                }
            }
            Some("-h" | "--help") => return Ok(None),
            Some(other) => return Err(format!("unknown option {other}")),
            None => return Err("command options must be valid UTF-8".to_string()),
        }
        index += 1;
    }
    if options.manifest_path.is_some() && !options.roots.is_empty() {
        return Err(
            "--manifest-path cannot be combined with an explicit target/build directory"
                .to_string(),
        );
    }
    Ok(Some(options))
}

fn print_help() {
    println!(
        "Scoped cleanup for prebindgen captures

Usage:
  cargo prebindgen clean [OPTIONS]

Options:
      --dry-run              Validate and list captures without removing them
      --target-dir <PATH>    Scan an explicit Cargo target directory
      --build-dir <PATH>     Scan an explicit Cargo build directory (repeatable)
      --manifest-path <PATH> Resolve the target directory through cargo metadata
  -h, --help                 Print help
  -V, --version              Print version

The command acquires every discovered Cargo profile lock before changing
anything. It removes only state-validated */build/*/out/prebindgen directories.
Cargo build directories configured separately from target-dir must be supplied
with --build-dir."
    );
}

#[derive(Deserialize)]
struct Metadata {
    target_directory: PathBuf,
}

fn metadata_target_dir(manifest_path: Option<&Path>) -> Result<PathBuf, String> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut command = Command::new(cargo);
    command.args(["metadata", "--format-version=1", "--no-deps"]);
    if let Some(path) = manifest_path {
        command.arg("--manifest-path").arg(path);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice::<Metadata>(&output.stdout)
        .map(|metadata| metadata.target_directory)
        .map_err(|error| format!("failed to decode cargo metadata: {error}"))
}

#[derive(Debug, Default)]
struct CleanReport {
    managed: usize,
    interrupted: usize,
    skipped: usize,
    paths: Vec<PathBuf>,
}

fn clean_roots(roots: &[PathBuf], dry_run: bool) -> io::Result<CleanReport> {
    let profiles = discover_profiles(roots)?;
    let _locks = lock_profiles(&profiles)?;

    let mut candidates = BTreeSet::new();
    for profile in &profiles {
        scan_profile(profile, &mut candidates)?;
    }

    let mut managed = Vec::new();
    let mut skipped = 0;
    for candidate in candidates {
        if validate_capture_dir(&candidate.path)?.is_some() {
            managed.push(candidate);
        } else {
            skipped += 1;
        }
    }

    let mut report = CleanReport {
        managed: managed.len(),
        skipped,
        ..CleanReport::default()
    };
    for candidate in managed {
        report.paths.push(candidate.path.clone());
        if candidate.interrupted {
            report.interrupted += 1;
        }
        if dry_run {
            continue;
        }
        if candidate.interrupted {
            fs::remove_dir_all(&candidate.path)?;
        } else {
            advance_state(&candidate.out_dir)?;
            let tombstone = next_tombstone(&candidate.out_dir);
            fs::rename(&candidate.path, &tombstone)?;
            fs::remove_dir_all(tombstone)?;
        }
    }
    Ok(report)
}

fn discover_profiles(roots: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut profiles = BTreeSet::new();
    for root in roots {
        let root = fs::canonicalize(root).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("cannot scan build root {}: {error}", root.display()),
            )
        })?;
        discover_profiles_at(&root, 0, &mut profiles)?;
    }
    Ok(profiles.into_iter().collect())
}

fn discover_profiles_at(
    directory: &Path,
    depth: usize,
    profiles: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    if has_profile_lock(directory)? {
        profiles.insert(directory.to_path_buf());
        return Ok(());
    }
    if depth == 2 {
        return Ok(());
    }

    let mut children = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let metadata = child.file_type()?;
        if metadata.is_dir() && !metadata.is_symlink() {
            discover_profiles_at(&child.path(), depth + 1, profiles)?;
        }
    }
    Ok(())
}

fn has_profile_lock(directory: &Path) -> io::Result<bool> {
    let mut found = false;
    for name in CARGO_LOCKS_IN_ORDER {
        let path = directory.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Cargo lock path is not a regular file: {}", path.display()),
                ));
            }
            Ok(_) => found = true,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(found)
}

struct LockedProfile {
    _path: PathBuf,
    _locks: Vec<File>,
}

fn lock_profiles(profiles: &[PathBuf]) -> io::Result<Vec<LockedProfile>> {
    for profile in profiles {
        if is_nfs(profile)? {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "refusing cleanup on NFS because Cargo cannot provide a reliable build lock: {}",
                    profile.display()
                ),
            ));
        }
    }

    let mut locked = Vec::new();
    for profile in profiles {
        let mut files = Vec::new();
        for name in CARGO_LOCKS_IN_ORDER {
            let path = profile.join(name);
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Cargo lock path is not a regular file: {}", path.display()),
                ));
            }

            let file = OpenOptions::new().read(true).write(true).open(&path)?;
            match FileExt::try_lock_exclusive(&file) {
                Ok(()) => files.push(file),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!(
                            "Cargo profile is active; no captures were removed: {}",
                            profile.display()
                        ),
                    ));
                }
                Err(error) => {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("cannot lock {}: {error}", path.display()),
                    ));
                }
            }
        }
        locked.push(LockedProfile {
            _path: profile.clone(),
            _locks: files,
        });
    }
    Ok(locked)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    path: PathBuf,
    out_dir: PathBuf,
    interrupted: bool,
}

fn scan_profile(profile: &Path, candidates: &mut BTreeSet<Candidate>) -> io::Result<()> {
    let build = profile.join("build");
    if !is_directory(&build)? {
        return Ok(());
    }

    for first in directory_children(&build)? {
        if !is_directory(&first)? {
            continue;
        }
        scan_out_dir(&first.join("out"), candidates)?;
        for second in directory_children(&first)? {
            if is_directory(&second)? {
                scan_out_dir(&second.join("out"), candidates)?;
            }
        }
    }
    Ok(())
}

fn scan_out_dir(out_dir: &Path, candidates: &mut BTreeSet<Candidate>) -> io::Result<()> {
    if !is_directory(out_dir)? {
        return Ok(());
    }

    let current = out_dir.join(PREBINDGEN_DIR);
    match fs::symlink_metadata(&current) {
        Ok(_) => {
            candidates.insert(Candidate {
                path: current,
                out_dir: out_dir.to_path_buf(),
                interrupted: false,
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    for path in directory_children(out_dir)? {
        if path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with(TOMBSTONE_PREFIX))
        {
            candidates.insert(Candidate {
                path,
                out_dir: out_dir.to_path_buf(),
                interrupted: true,
            });
        }
    }
    Ok(())
}

fn directory_children(directory: &Path) -> io::Result<Vec<PathBuf>> {
    match fs::read_dir(directory) {
        Ok(entries) => {
            let mut paths = entries
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, _>>()?;
            paths.sort();
            Ok(paths)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn is_directory(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn next_tombstone(out_dir: &Path) -> PathBuf {
    loop {
        let id = TOMBSTONE_ID.fetch_add(1, Ordering::Relaxed);
        let path = out_dir.join(format!("{TOMBSTONE_PREFIX}{}-{id}", std::process::id()));
        if !path.exists() {
            return path;
        }
    }
}

#[cfg(target_os = "linux")]
fn is_nfs(path: &Path) -> io::Result<bool> {
    use std::{ffi::CString, mem, os::unix::ffi::OsStrExt};

    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path contains a NUL byte: {}", path.display()),
        )
    })?;
    let mut status = unsafe { mem::zeroed::<libc::statfs>() };
    if unsafe { libc::statfs(path.as_ptr(), &mut status) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(status.f_type as u64 == libc::NFS_SUPER_MAGIC as u64)
}

#[cfg(not(target_os = "linux"))]
fn is_nfs(_path: &Path) -> io::Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        fs::{self, OpenOptions},
        io,
        path::{Path, PathBuf},
        sync::{Arc, Barrier},
        thread,
    };

    use fs2::FileExt;
    use prebindgen::capture_protocol::{ensure_capture_dir_at, initialize_state, state_slot_paths};

    use super::{clean_roots, TOMBSTONE_PREFIX};

    fn profile(root: &Path, name: &str) -> PathBuf {
        let profile = root.join(name);
        fs::create_dir_all(profile.join("build")).unwrap();
        fs::write(profile.join(".cargo-build-lock"), []).unwrap();
        fs::write(profile.join(".cargo-lock"), []).unwrap();
        profile
    }

    fn managed_at(out_dir: &Path, crate_name: &str) -> PathBuf {
        fs::create_dir_all(out_dir).unwrap();
        let features = BTreeSet::from(["feature-a".to_string()]);
        initialize_state(out_dir, crate_name, &features).unwrap();
        let capture = ensure_capture_dir_at(out_dir).unwrap();
        fs::write(capture.join("default_deadbeef.jsonl"), b"{}\n").unwrap();
        capture
    }

    fn legacy_capture(profile: &Path, unit: &str) -> PathBuf {
        managed_at(&profile.join("build").join(unit).join("out"), "source")
    }

    #[test]
    fn cleans_valid_legacy_new_and_cross_target_layouts_only() {
        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        let legacy = legacy_capture(&debug, "source-0123456789abcdef");
        let modern = managed_at(
            &debug
                .join("build")
                .join("other-source")
                .join("fedcba9876543210")
                .join("out"),
            "other-source",
        );
        let cross_profile = profile(&root.path().join("wasm32-unknown-unknown"), "release");
        let cross = legacy_capture(&cross_profile, "cross-source-0123456789abcdef");
        let degraded = legacy_capture(&debug, "degraded-0123456789abcdef");
        let degraded_out = degraded.parent().unwrap();
        fs::remove_file(&state_slot_paths(degraded_out)[0]).unwrap();

        let malformed = debug
            .join("build")
            .join("malformed-0123456789abcdef")
            .join("out")
            .join("prebindgen");
        fs::create_dir_all(&malformed).unwrap();
        fs::write(malformed.join("crate_name.txt"), b"not-managed").unwrap();

        let unrelated = debug.join("unrelated/out/prebindgen");
        fs::create_dir_all(&unrelated).unwrap();
        fs::write(unrelated.join("keep"), b"keep").unwrap();
        let cargo_owned = legacy.parent().unwrap().join("cargo-output");
        fs::write(&cargo_owned, b"keep").unwrap();

        let report = clean_roots(&[root.path().to_path_buf()], false).unwrap();
        assert_eq!(report.managed, 3);
        assert_eq!(report.skipped, 2);
        assert!(!legacy.exists());
        assert!(!modern.exists());
        assert!(!cross.exists());
        assert!(malformed.exists());
        assert!(degraded.exists());
        assert!(unrelated.exists());
        assert_eq!(fs::read(cargo_owned).unwrap(), b"keep");
        for path in state_slot_paths(legacy.parent().unwrap()) {
            assert!(path.is_file());
        }
    }

    #[test]
    fn active_profile_aborts_all_profiles_before_any_deletion() {
        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        let release = profile(root.path(), "release");
        let debug_capture = legacy_capture(&debug, "source-a");
        let release_capture = legacy_capture(&release, "source-b");

        let ready = Arc::new(Barrier::new(2));
        let release_lock = Arc::new(Barrier::new(2));
        let lock_path = debug.join(".cargo-lock");
        let worker = {
            let ready = Arc::clone(&ready);
            let release_lock = Arc::clone(&release_lock);
            thread::spawn(move || {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(lock_path)
                    .unwrap();
                FileExt::lock_shared(&file).unwrap();
                ready.wait();
                release_lock.wait();
            })
        };
        ready.wait();

        let error = clean_roots(&[root.path().to_path_buf()], false).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(debug_capture.exists());
        assert!(release_capture.exists());

        release_lock.wait();
        worker.join().unwrap();
        let report = clean_roots(&[root.path().to_path_buf()], false).unwrap();
        assert_eq!(report.managed, 2);
    }

    #[test]
    fn active_separate_build_directory_lock_blocks_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        fs::remove_file(debug.join(".cargo-lock")).unwrap();
        let capture = legacy_capture(&debug, "source-a");

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(debug.join(".cargo-build-lock"))
            .unwrap();
        FileExt::lock_shared(&lock).unwrap();

        let error = clean_roots(&[root.path().to_path_buf()], false).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(capture.exists());

        FileExt::unlock(&lock).unwrap();
        let report = clean_roots(&[root.path().to_path_buf()], false).unwrap();
        assert_eq!(report.managed, 1);
        assert!(!capture.exists());
    }
    #[test]
    fn dry_run_validates_and_preserves_capture_and_generation() {
        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        let capture = legacy_capture(&debug, "source-a");
        let out = capture.parent().unwrap();
        let generation = prebindgen::capture_protocol::load_latest_state(out)
            .unwrap()
            .0
            .generation;

        let report = clean_roots(&[root.path().to_path_buf()], true).unwrap();
        assert_eq!(report.managed, 1);
        assert!(capture.exists());
        assert_eq!(
            prebindgen::capture_protocol::load_latest_state(out)
                .unwrap()
                .0
                .generation,
            generation
        );
    }

    #[test]
    fn interrupted_tombstone_is_recovered_without_touching_other_output() {
        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        let capture = legacy_capture(&debug, "source-a");
        let out = capture.parent().unwrap();
        let tombstone = out.join(format!("{TOMBSTONE_PREFIX}old"));
        fs::rename(&capture, &tombstone).unwrap();
        let keep = out.join("keep");
        fs::write(&keep, b"keep").unwrap();

        let report = clean_roots(&[root.path().to_path_buf()], false).unwrap();
        assert_eq!(report.managed, 1);
        assert_eq!(report.interrupted, 1);
        assert!(!tombstone.exists());
        assert_eq!(fs::read(keep).unwrap(), b"keep");
    }

    #[cfg(unix)]
    #[test]
    fn capture_symlink_is_never_followed() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let debug = profile(root.path(), "debug");
        let out = debug.join("build/source-a/out");
        fs::create_dir_all(&out).unwrap();
        let real = root.path().join("outside");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("keep"), b"keep").unwrap();
        symlink(&real, out.join("prebindgen")).unwrap();

        let report = clean_roots(&[root.path().to_path_buf()], false).unwrap();
        assert_eq!(report.managed, 0);
        assert_eq!(report.skipped, 1);
        assert_eq!(fs::read(real.join("keep")).unwrap(), b"keep");
    }
}
