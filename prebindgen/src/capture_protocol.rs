//! Internal protocol shared by capture producers and the cargo-prebindgen binary.
//!
//! This module is public only so the proc-macro crate and the binary shipped in
//! this package can share the protocol. It is not a supported user API.

use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};

pub const PREBINDGEN_DIR: &str = "prebindgen";
pub const CRATE_NAME_FILE: &str = "crate_name.txt";
pub const FEATURES_FILE: &str = "features.txt";
pub const STATE_SLOT_FILES: [&str; 2] = [
    ".prebindgen-capture-state-v1-a.json",
    ".prebindgen-capture-state-v1-b.json",
];

const PROTOCOL: &str = "prebindgen-capture-v1";
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureState {
    protocol: String,
    pub generation: u64,
    pub crate_name: String,
    pub features: Vec<String>,
}

impl CaptureState {
    fn new(generation: u64, crate_name: &str, features: &BTreeSet<String>) -> Self {
        Self {
            protocol: PROTOCOL.to_string(),
            generation,
            crate_name: crate_name.to_string(),
            features: features.iter().cloned().collect(),
        }
    }

    fn validate(&self) -> bool {
        self.protocol == PROTOCOL
            && !self.crate_name.is_empty()
            && !self.crate_name.contains(['\n', '\r'])
            && self
                .features
                .iter()
                .all(|feature| !feature.is_empty() && !feature.contains(['\n', '\r']))
            && self.features.windows(2).all(|pair| pair[0] < pair[1])
    }
}

pub fn state_slot_paths(out_dir: &Path) -> [PathBuf; 2] {
    STATE_SLOT_FILES.map(|file| out_dir.join(file))
}

pub fn capture_dir(out_dir: &Path) -> PathBuf {
    out_dir.join(PREBINDGEN_DIR)
}

pub fn get_out_dir() -> io::Result<PathBuf> {
    std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))
}

pub fn get_state_slot_paths() -> io::Result<[PathBuf; 2]> {
    Ok(state_slot_paths(&get_out_dir()?))
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn reject_symlink(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(invalid_data(format!(
            "refusing prebindgen state symlink {}",
            path.display()
        ))),
        Ok(metadata) if !metadata.is_file() => Err(invalid_data(format!(
            "prebindgen state path is not a file: {}",
            path.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn read_slot(path: &Path) -> io::Result<Option<CaptureState>> {
    reject_symlink(path)?;
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let Ok(state) = serde_json::from_slice::<CaptureState>(&bytes) else {
        return Ok(None);
    };
    Ok(state.validate().then_some(state))
}

fn read_slots(out_dir: &Path) -> io::Result<[Option<CaptureState>; 2]> {
    let [a, b] = state_slot_paths(out_dir);
    Ok([read_slot(&a)?, read_slot(&b)?])
}

fn latest_from_slots(slots: &[Option<CaptureState>; 2]) -> io::Result<(CaptureState, usize)> {
    let latest = slots
        .iter()
        .enumerate()
        .filter_map(|(index, state)| state.as_ref().map(|state| (state, index)))
        .max_by_key(|(state, _)| state.generation)
        .ok_or_else(|| invalid_data("no valid prebindgen capture state slot"))?;

    if let Some(other) = slots[1 - latest.1].as_ref() {
        if other.generation == latest.0.generation && other != latest.0 {
            return Err(invalid_data(
                "prebindgen capture state slots disagree at the same generation",
            ));
        }
    }
    Ok((latest.0.clone(), latest.1))
}

pub fn load_latest_state(out_dir: &Path) -> io::Result<(CaptureState, usize)> {
    latest_from_slots(&read_slots(out_dir)?)
}

fn write_slot(path: &Path, state: &CaptureState) -> io::Result<()> {
    reject_symlink(path)?;
    let mut bytes = serde_json::to_vec(state).map_err(|error| invalid_data(error.to_string()))?;
    bytes.push(b'\n');

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    let verified = read_slot(path)?;
    if verified.as_ref() != Some(state) {
        return Err(invalid_data(format!(
            "failed to verify prebindgen state slot {}",
            path.display()
        )));
    }
    Ok(())
}

/// Initialize both state slots before rustc starts.
pub fn initialize_state(
    out_dir: &Path,
    crate_name: &str,
    features: &BTreeSet<String>,
) -> io::Result<CaptureState> {
    let generation = read_slots(out_dir)?
        .iter()
        .flatten()
        .map(|state| state.generation)
        .max()
        .map_or(Ok(0), |generation| {
            generation
                .checked_add(1)
                .ok_or_else(|| invalid_data("prebindgen capture generation overflow"))
        })?;
    let state = CaptureState::new(generation, crate_name, features);
    for path in state_slot_paths(out_dir) {
        write_slot(&path, &state)?;
    }
    Ok(state)
}

/// Durably change one state slot while retaining the previous valid slot.
///
/// Cleanup calls this before renaming a capture directory. A crash before this
/// function returns leaves the old slot valid and the directory untouched; a
/// crash afterwards leaves rustc with a changed input dependency.
pub fn advance_state(out_dir: &Path) -> io::Result<CaptureState> {
    let slots = read_slots(out_dir)?;
    let (latest, latest_index) = latest_from_slots(&slots)?;
    let generation = latest
        .generation
        .checked_add(1)
        .ok_or_else(|| invalid_data("prebindgen capture generation overflow"))?;
    let next = CaptureState {
        generation,
        ..latest
    };
    let target_index = 1 - latest_index;
    write_slot(&state_slot_paths(out_dir)[target_index], &next)?;
    Ok(next)
}

pub fn feature_file_contents(features: &[String]) -> String {
    let mut contents = features.join("\n");
    if !contents.is_empty() {
        contents.push('\n');
    }
    contents
}

fn write_recovery_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if fs::read(path)
        .map(|existing| existing == bytes)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let id = TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("metadata");
    let temporary = path.with_file_name(format!(".{file_name}.tmp-{}-{id}", std::process::id()));
    let mut file = File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);

    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
            ) && fs::read(path)
                .map(|existing| existing == bytes)
                .unwrap_or(false) =>
        {
            fs::remove_file(temporary)
        }
        Err(error) => {
            let _ = fs::remove_file(temporary);
            Err(error)
        }
    }
}

/// Recreate metadata after scoped cleanup without requiring the build script
/// itself to rerun. The proc macro calls this before writing any JSONL record.
pub fn ensure_capture_dir() -> io::Result<PathBuf> {
    ensure_capture_dir_at(&get_out_dir()?)
}

pub fn ensure_capture_dir_at(out_dir: &Path) -> io::Result<PathBuf> {
    let (state, _) = load_latest_state(out_dir)?;
    let capture_dir = capture_dir(out_dir);
    fs::create_dir_all(&capture_dir)?;
    write_recovery_file(
        &capture_dir.join(CRATE_NAME_FILE),
        state.crate_name.as_bytes(),
    )?;
    write_recovery_file(
        &capture_dir.join(FEATURES_FILE),
        feature_file_contents(&state.features).as_bytes(),
    )?;
    Ok(capture_dir)
}

pub fn validate_capture_dir(path: &Path) -> io::Result<Option<CaptureState>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Ok(None);
    }
    let Some(out_dir) = path.parent() else {
        return Ok(None);
    };
    // Cleanup must update an already-existing inactive slot before deletion.
    // Creating a missing slot would also require syncing the parent directory
    // to make the new entry power-loss durable.
    for state_path in state_slot_paths(out_dir) {
        let metadata = match fs::symlink_metadata(state_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Ok(None);
        }
    }
    let (state, _) = match load_latest_state(out_dir) {
        Ok(state) => state,
        Err(error) if error.kind() == io::ErrorKind::InvalidData => return Ok(None),
        Err(error) => return Err(error),
    };

    let crate_name = path.join(CRATE_NAME_FILE);
    let features = path.join(FEATURES_FILE);
    for metadata_path in [&crate_name, &features] {
        let metadata = match fs::symlink_metadata(metadata_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Ok(None);
        }
    }
    if fs::read(&crate_name)? != state.crate_name.as_bytes()
        || fs::read(&features)? != feature_file_contents(&state.features).as_bytes()
    {
        return Ok(None);
    }
    Ok(Some(state))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        advance_state, capture_dir, ensure_capture_dir_at, initialize_state, load_latest_state,
        CRATE_NAME_FILE, FEATURES_FILE,
    };

    #[test]
    fn inactive_slot_advance_survives_a_corrupt_other_slot() {
        let dir = tempfile::tempdir().unwrap();
        let features = BTreeSet::from(["a".to_string(), "b".to_string()]);
        let initialized = initialize_state(dir.path(), "source", &features).unwrap();
        let advanced = advance_state(dir.path()).unwrap();
        assert_eq!(advanced.generation, initialized.generation + 1);

        let (_, latest_index) = load_latest_state(dir.path()).unwrap();
        let other = super::state_slot_paths(dir.path())[1 - latest_index].clone();
        std::fs::write(other, b"truncated").unwrap();
        assert_eq!(load_latest_state(dir.path()).unwrap().0, advanced);
    }

    #[test]
    fn macro_recovery_recreates_deleted_capture_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let features = BTreeSet::from(["unstable".to_string()]);
        initialize_state(dir.path(), "source", &features).unwrap();

        let recovered = ensure_capture_dir_at(dir.path()).unwrap();
        assert_eq!(recovered, capture_dir(dir.path()));
        assert_eq!(
            std::fs::read_to_string(recovered.join(CRATE_NAME_FILE)).unwrap(),
            "source"
        );
        assert_eq!(
            std::fs::read_to_string(recovered.join(FEATURES_FILE)).unwrap(),
            "unstable\n"
        );
    }
}
