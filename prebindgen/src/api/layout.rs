//! Where a captured record lives inside the prebindgen output directory.
//!
//! ```text
//! {OUT_DIR}/prebindgen/
//!     crate_name.txt
//!     features.txt
//!     g_{group}/                        <- one directory per group, encoded
//!         {name}_{digest(record)}.jsonl <- one file per captured record
//! ```
//!
//! Nothing here is named after the process or the compilation that wrote it:
//! every path is derived from the data it holds. The proc-macro computes these
//! paths when writing and [`Source`](crate::Source) computes them when reading,
//! so they live here rather than in either crate.
//!
//! That is what keeps the layout loss-proof: a name determines its contents, so
//! two writers either write identical bytes to one path or different bytes to
//! different paths — never a partial overwrite of one record's capture by
//! another's.
//!
//! Real filesystems are far more opinionated about names than Rust is about
//! identifiers and string literals, and a group name is an arbitrary string
//! literal. Four hazards have to be closed, or two distinct groups end up in
//! one directory on somebody's machine:
//!
//! - **Case folding.** macOS (APFS by default) and Windows treat `Foo` and
//!   `foo` as one name, while `#[prebindgen("Foo")]` and `#[prebindgen("foo")]`
//!   are two groups.
//! - **Reserved names.** Windows cannot create `CON`, `PRN`, `AUX`, `NUL`,
//!   `COM1`…`LPT9`, with or without an extension, nor a component ending in a
//!   dot or a space.
//! - **Separators and traversal.** `"a/b"` or `".."` must not escape the
//!   capture directory or name its parent.
//! - **Unicode normalization.** Non-ASCII names are stored decomposed on macOS
//!   and composed elsewhere, so one literal would spell two directories.
//!
//! [`group_dir_name`] closes all four by encoding rather than by restricting
//! what a group may be called: only `a-z`, `0-9` and `_` survive verbatim,
//! every other byte becomes `-` plus two lowercase hex digits, and the whole
//! sits behind a `g_` prefix. The result holds no uppercase (so case folding
//! cannot merge two encodings), no separator, no dot, and is never a bare
//! device name.
//!
//! The encoding is reversible, which is what lets [`decode_group_dir_name`]
//! recover the exact group name a consumer selects by — no sidecar file, and no
//! way for a directory and the name it claims to disagree.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Marks a subdirectory as a group's captures, and keeps the encoded name off
/// the reserved-device list (`CON` encodes to `con`, which Windows refuses).
const GROUP_DIR_PREFIX: &str = "g_";

/// Bytes a single path component may hold. 255 is the limit on ext4, APFS and
/// NTFS alike; the encoding inflates a name at most threefold, so it is only
/// reachable with a deliberately absurd group name.
pub const MAX_COMPONENT_LEN: usize = 255;

/// Longest run of a record's name kept in its file name; the digest beside it
/// carries the identity, this part is only there to be read by a human.
const READABLE_LIMIT: usize = 48;

/// 16 lowercase hexadecimal digits identifying `value`.
fn digest(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// The directory holding one group's captures: `g_` plus the encoded name.
///
/// Escaping to `-XX` rather than to `_XX` is what keeps the common case
/// readable: `default` stays `g_default` and `my_group` stays `g_my_group`,
/// because `_` is the character group names actually contain. `Foo` becomes
/// `g_-46oo`.
///
/// See the module docs for why this encodes instead of validating.
pub fn group_dir_name(group: &str) -> String {
    use std::fmt::Write;

    let mut encoded = String::from(GROUP_DIR_PREFIX);
    for byte in group.bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'_' => encoded.push(byte as char),
            _ => {
                // Infallible: writing to a String.
                let _ = write!(encoded, "-{byte:02x}");
            }
        }
    }
    encoded
}

/// The group name a directory produced by [`group_dir_name`] stands for, or
/// `None` if it was not produced by it.
pub fn decode_group_dir_name(dir_name: &str) -> Option<String> {
    let encoded = dir_name.strip_prefix(GROUP_DIR_PREFIX)?;

    let mut bytes = Vec::with_capacity(encoded.len());
    let mut rest = encoded.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'_' => {
                bytes.push(byte);
                rest = tail;
            }
            b'-' if tail.len() >= 2 => {
                let (hex, tail) = tail.split_at(2);
                bytes.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                rest = tail;
            }
            // Anything else cannot have come out of `group_dir_name`.
            _ => return None,
        }
    }
    String::from_utf8(bytes).ok()
}

/// The file holding one record: `{name}_{digest}.jsonl`.
///
/// `serialized` is the record's JSON form — the exact bytes the file will
/// hold — so equal names mean equal files. Unlike a group name, a record name
/// never has to be recovered from its path, so the readable part is a lossy
/// ASCII reduction and the digest beside it carries the identity. The digest is
/// what keeps `#[cfg(test)] fn f` apart from the crate's own `fn f`, whose
/// units compile in parallel, and `struct Foo` apart from `fn foo` on a
/// case-folding filesystem.
pub fn capture_file_name(name: &str, serialized: &str) -> String {
    let readable: String = name
        .chars()
        .take(READABLE_LIMIT)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    format!("{readable}_{}.jsonl", digest(serialized))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names Windows refuses as a path component, with or without extension.
    const WINDOWS_DEVICE_NAMES: [&str; 8] =
        ["CON", "PRN", "AUX", "NUL", "COM1", "COM9", "LPT1", "LPT9"];

    /// Group names a filesystem would otherwise refuse, mangle, or merge.
    const HOSTILE_GROUP_NAMES: [&str; 16] = [
        "Foo",
        "foo",
        "CON",
        "NUL",
        "a/b",
        "a\\b",
        "..",
        ".",
        "../../etc",
        "",
        "  ",
        "a:b",
        "a\0b",
        "trailing.",
        "grüppe",
        "🙂",
    ];

    #[test]
    fn a_group_name_survives_the_round_trip() {
        for group in HOSTILE_GROUP_NAMES
            .iter()
            .copied()
            .chain(["default", "structs", "my_group", "my-group"])
        {
            let dir = group_dir_name(group);
            assert_eq!(
                decode_group_dir_name(&dir).as_deref(),
                Some(group),
                "{dir} decoded wrong"
            );
        }
    }

    #[test]
    fn every_char_survives_the_round_trip() {
        for code_point in (0..=0x2ff_u32).chain([0x1f600, 0x10ffff]) {
            let Some(character) = char::from_u32(code_point) else {
                continue;
            };
            let group = format!("a{character}b");
            assert_eq!(
                decode_group_dir_name(&group_dir_name(&group)).as_deref(),
                Some(group.as_str())
            );
        }
    }

    #[test]
    fn only_directories_this_produced_decode() {
        for foreign in [
            "default",
            "incremental",
            "g",
            "g_A",
            "g_a-",
            "g_a-zz",
            "g_a.",
        ] {
            assert_eq!(decode_group_dir_name(foreign), None, "{foreign}");
        }
    }

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
        }
    }

    #[test]
    fn group_directories_are_one_contained_component() {
        for hostile in HOSTILE_GROUP_NAMES {
            let dir = group_dir_name(hostile);
            assert!(dir.is_ascii(), "{dir:?} from {hostile:?}");
            assert!(
                !dir.contains(['/', '\\', ':', '.', ' ', '\0']),
                "{dir:?} from {hostile:?}"
            );
            assert!(
                !dir.chars().any(char::is_uppercase),
                "{dir:?} folds on a case-insensitive filesystem"
            );
            assert!(dir != "." && dir != "..", "{dir:?}");
        }
    }

    #[test]
    fn the_encoding_inflates_a_name_at_most_threefold() {
        // What keeps MAX_COMPONENT_LEN out of reach of any sane group name.
        for group in HOSTILE_GROUP_NAMES {
            let encoded = group_dir_name(group).len() - GROUP_DIR_PREFIX.len();
            assert!(encoded <= group.len() * 3, "{group:?} -> {encoded}");
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
    fn a_capture_file_name_is_ascii_and_bounded() {
        let unicode = capture_file_name("Ünïcödé", r#"{"name":"unicode"}"#);
        assert!(unicode.is_ascii(), "{unicode}");

        let long = capture_file_name(&"a".repeat(READABLE_LIMIT * 4), r#"{"name":"long"}"#);
        assert_eq!(long.len(), READABLE_LIMIT + 1 + 16 + ".jsonl".len());
        assert!(long.len() <= MAX_COMPONENT_LEN);
    }
}
