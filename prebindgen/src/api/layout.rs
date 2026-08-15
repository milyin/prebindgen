//! Where a captured record lives inside the prebindgen output directory.
//!
//! ```text
//! {OUT_DIR}/prebindgen/
//!     crate_name.txt
//!     features.txt
//!     {digest(group)}_{group}/          <- one directory per group
//!         group.txt                     <- the group's exact name
//!         {name}_{digest(record)}.jsonl <- one file per captured record
//! ```
//!
//! Both name components follow the same rule: **a digest carries the identity
//! and a sanitized spelling carries the readability.** The proc-macro computes
//! these paths when writing and [`Source`](crate::Source) computes them when
//! reading, so they live here rather than in either crate.
//!
//! Deriving a name from the data it holds is what keeps the layout loss-proof:
//! the name determines the contents, so two writers either write identical
//! bytes to one path or different bytes to different paths — never a partial
//! overwrite of one record's capture by another's.
//!
//! The digest is what makes that hold on real filesystems, which are far more
//! opinionated about names than Rust is about identifiers:
//!
//! - **Case folding.** macOS (APFS by default) and Windows treat `Foo` and
//!   `foo` as one name. Rust has both `struct Foo` and `fn foo`, and
//!   `#[prebindgen("Foo")]` and `#[prebindgen("foo")]` are two groups.
//! - **Reserved names.** Windows cannot create `CON`, `PRN`, `AUX`, `NUL`,
//!   `COM1`…`LPT9`, with or without an extension. A digest prefix means no
//!   component is ever a bare device name.
//! - **Separators and traversal.** A group name is an arbitrary string literal:
//!   `"a/b"` or `".."` must not escape the capture directory or name its
//!   parent.
//! - **Unicode normalization.** Rust identifiers and string literals admit
//!   non-ASCII characters, which filesystems normalize inconsistently
//!   (macOS stores NFD). The readable part is reduced to ASCII so a name's
//!   spelling never depends on the host.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Name of the file inside a group directory holding the group's exact name.
///
/// The directory name digests the group, which is not reversible, so the name
/// is recorded next to the records that belong to it. This is what lets
/// `Source` report the group a consumer selects by (`items_in_groups`) rather
/// than an approximation of it.
pub const GROUP_NAME_FILE: &str = "group.txt";

/// Longest run of a name kept in a path component; the digest beside it carries
/// the identity, this part is only there to be read by a human.
const READABLE_LIMIT: usize = 48;

/// 16 lowercase hexadecimal digits identifying `value`.
fn digest(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// The readable half of a path component: ASCII alphanumerics kept, everything
/// else folded to `_`, truncated to [`READABLE_LIMIT`] characters.
fn readable(value: &str) -> String {
    value
        .chars()
        .take(READABLE_LIMIT)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// The directory holding one group's captures: `{digest}_{group}`.
///
/// Leading with the digest makes the component collision-free for any two
/// distinct group names, on any filesystem — see the module docs for the four
/// ways a group name would otherwise be unrepresentable.
pub fn group_dir_name(group: &str) -> String {
    format!("{}_{}", digest(group), readable(group))
}

/// The file holding one record: `{name}_{digest}.jsonl`.
///
/// `serialized` is the record's JSON form — the exact bytes the file will
/// hold — so equal names mean equal files.
pub fn capture_file_name(name: &str, serialized: &str) -> String {
    format!("{}_{}.jsonl", readable(name), digest(serialized))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names Windows refuses as a path component, with or without extension.
    const WINDOWS_DEVICE_NAMES: [&str; 8] =
        ["CON", "PRN", "AUX", "NUL", "COM1", "COM9", "LPT1", "LPT9"];

    #[test]
    fn groups_differing_only_in_case_get_different_directories() {
        // The collision this guards: `.join(group)` puts `Foo` and `foo` in one
        // directory on macOS and Windows, merging two groups whose records the
        // API keeps apart.
        let upper = group_dir_name("Foo");
        let lower = group_dir_name("foo");
        assert_ne!(upper, lower);
        assert_ne!(
            upper.to_lowercase(),
            lower.to_lowercase(),
            "must differ on a case-insensitive filesystem too"
        );
    }

    #[test]
    fn group_directories_are_never_windows_device_names() {
        for device in WINDOWS_DEVICE_NAMES {
            let dir = group_dir_name(device);
            let base = dir.split('.').next().unwrap();
            assert!(
                !base.eq_ignore_ascii_case(device),
                "{dir} would be refused on Windows"
            );
            // Every component starts with the digest, so none can be a device.
            assert!(dir[..16].chars().all(|c| c.is_ascii_hexdigit()), "{dir}");
            assert_eq!(&dir[16..17], "_", "{dir}");
        }
    }

    #[test]
    fn group_directories_are_one_contained_component() {
        for hostile in [
            "a/b",
            "a\\b",
            "..",
            ".",
            "../../etc",
            "",
            "  ",
            "a:b",
            "a\0b",
        ] {
            let dir = group_dir_name(hostile);
            assert!(
                !dir.contains(['/', '\\', ':', '\0']),
                "{dir:?} from {hostile:?}"
            );
            assert!(dir != "." && dir != "..", "{dir:?}");
            assert!(dir.is_ascii(), "{dir:?}");
        }
    }

    #[test]
    fn distinct_group_names_get_distinct_directories() {
        let long = "a".repeat(READABLE_LIMIT * 2);
        let longer = "a".repeat(READABLE_LIMIT * 2 + 1);
        let names = [
            "default", "structs", "my_group", "my-group", "Grüppe", "grüppe", "", &long, &longer,
        ];
        let mut seen = std::collections::HashSet::new();
        for name in names {
            // Lowercased: two directories that fold together are a collision.
            assert!(
                seen.insert(group_dir_name(name).to_lowercase()),
                "{name:?} collides"
            );
        }
    }

    #[test]
    fn a_capture_file_name_is_determined_by_the_record() {
        assert_eq!(
            capture_file_name("Foo", r#"{"name":"Foo"}"#),
            capture_file_name("Foo", r#"{"name":"Foo"}"#)
        );
        assert_ne!(
            capture_file_name("Foo", r#"{"name":"Foo"}"#),
            capture_file_name("Foo", r#"{"name":"Foo","cfg":"test"}"#)
        );
        // macOS and Windows fold case, and `struct Foo` may live beside `fn foo`.
        assert_ne!(
            capture_file_name("Foo", r#"{"kind":"struct","name":"Foo"}"#).to_lowercase(),
            capture_file_name("foo", r#"{"kind":"function","name":"foo"}"#).to_lowercase()
        );
    }

    #[test]
    fn the_readable_part_is_ascii_and_bounded() {
        let unicode = capture_file_name("Ünïcödé", r#"{"name":"unicode"}"#);
        assert!(unicode.is_ascii(), "{unicode}");

        let long = capture_file_name(&"a".repeat(READABLE_LIMIT * 4), r#"{"name":"long"}"#);
        assert_eq!(long.len(), READABLE_LIMIT + 1 + 16 + ".jsonl".len());
    }
}
