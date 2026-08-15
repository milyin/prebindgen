//! Serialization utilities for reading and writing records.

use std::{
    borrow::Borrow,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::api::record::Record;

/// Publish `contents` at `file_path` atomically, once.
///
/// Every path in the capture directory is named after what it holds (see
/// [`layout`](crate::layout)), so any writer producing this path produces these
/// exact bytes. Two compilations of the same source therefore write the same
/// file rather than two copies of it — but they may do so concurrently, so the
/// content is written to a private temporary file in the same directory and
/// `rename`d over the destination. A reader consequently sees either the
/// previous complete file or the new complete file, never a partial one, and a
/// lost race costs nothing because the loser wrote identical bytes.
pub fn publish_file<P: AsRef<Path>>(
    file_path: P,
    contents: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = file_path.as_ref();

    // The name determines the content, so an existing file already holds it:
    // skip the write rather than re-publishing identical bytes. Only a complete
    // file can exist at this path (they arrive by rename), but a zero-length
    // one from an older layout or a truncated copy is rewritten.
    if fs::metadata(file_path).is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(());
    }

    let directory = file_path
        .parent()
        .ok_or_else(|| format!("capture path {} has no parent", file_path.display()))?;
    fs::create_dir_all(directory)?;

    // Unique per process and per call: two writers must never share a temporary.
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let temporary = directory.join(format!(
        ".{}.{}.{}.tmp",
        file_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));

    let write_temporary = || -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    };
    if let Err(error) = write_temporary() {
        let _ = fs::remove_file(&temporary);
        return Err(format!("failed to write {}: {error}", temporary.display()).into());
    }

    if let Err(error) = fs::rename(&temporary, file_path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("failed to publish {}: {error}", file_path.display()).into());
    }
    Ok(())
}

/// Write a collection of records to a file in JSON-lines format
pub fn write_to_jsonl_file<P: AsRef<Path>, R: Borrow<Record>>(
    file_path: P,
    records: &[R],
) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) else {
        return Err("Failed to open file".into());
    };
    // Check if file is empty (just created or was deleted)
    for record in records {
        let json_line = record.borrow().to_jsonl_string()?;
        writeln!(file, "{json_line}")?;
    }
    file.flush()?;
    Ok(())
}

/// Read records from a JSON-lines file
pub fn read_jsonl_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&file_path)?;
    let mut records = Vec::new();

    // Parse JSON-lines format: each line is a separate JSON object
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue; // Skip empty lines
        }

        let record: Record = serde_json::from_str(line)
            .map_err(|e| format!("{}:{}: {}", file_path.as_ref().display(), line_num + 1, e))?;

        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests;
