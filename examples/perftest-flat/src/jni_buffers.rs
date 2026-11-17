//! Text/byte output fixtures, with native storage initialized outside measurement.
use crate::CallbackOwned;
use prebindgen_proc_macro::prebindgen;
use std::sync::OnceLock;

#[prebindgen]
pub struct JniBufferRow {
    pub text: String,
    pub bytes: Vec<u8>,
}
fn rows() -> &'static [JniBufferRow] {
    static ROWS: OnceLock<Vec<JniBufferRow>> = OnceLock::new();
    ROWS.get_or_init(|| {
        vec![
            JniBufferRow {
                text: "x".repeat(65536),
                bytes: vec![0x9b; 65536],
            },
            JniBufferRow {
                text: "héllo\0世界😀".into(),
                bytes: vec![0, 127, 128, 255],
            },
            JniBufferRow {
                text: String::new(),
                bytes: Vec::new(),
            },
        ]
    })
}
#[prebindgen]
pub fn jni_buffer_text() -> &'static str {
    &rows()[0].text
}
#[prebindgen]
pub fn jni_buffer_string() -> &'static String {
    &rows()[0].text
}
#[prebindgen]
pub fn jni_buffer_bytes() -> &'static [u8] {
    &rows()[0].bytes
}
#[prebindgen]
pub fn jni_buffer_vec() -> &'static Vec<u8> {
    &rows()[0].bytes
}

#[prebindgen]
pub fn jni_buffer_cow(owned: bool) -> std::borrow::Cow<'static, [u8]> {
    if owned {
        std::borrow::Cow::Owned(rows()[0].bytes.clone())
    } else {
        std::borrow::Cow::Borrowed(&rows()[0].bytes)
    }
}

#[prebindgen]
pub fn jni_buffer_optional_cow(
    present: bool,
    owned: bool,
) -> Option<std::borrow::Cow<'static, [u8]>> {
    present.then(|| jni_buffer_cow(owned))
}

#[prebindgen]
pub fn jni_buffer_cow_rows() -> Vec<std::borrow::Cow<'static, [u8]>> {
    vec![
        jni_buffer_cow(false),
        jni_buffer_cow(true),
        std::borrow::Cow::Borrowed(&[]),
    ]
}

#[prebindgen]
pub fn jni_buffer_cow_rows_plain() -> Vec<std::borrow::Cow<'static, [u8]>> {
    jni_buffer_cow_rows()
}

#[prebindgen]
pub fn jni_buffer_cow_callback(callback: impl Fn(std::borrow::Cow<'_, [u8]>) + Send + Sync) {
    // The callback's byte array must survive this temporary owner's destruction.
    let owned = rows()[1].bytes.clone();
    callback(std::borrow::Cow::Borrowed(&owned));
    callback(std::borrow::Cow::Owned(owned.clone()));
}

#[prebindgen]
pub fn jni_buffer_cow_from_slice(value: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    std::borrow::Cow::Borrowed(value)
}

#[prebindgen]
pub fn jni_buffer_cow_input(value: std::borrow::Cow<'_, [u8]>) -> usize {
    value.len()
}
#[prebindgen]
pub fn jni_buffer_optional_text(mode: i32) -> Option<Option<&'static str>> {
    match mode {
        0 => None,
        1 => Some(None),
        2 => Some(Some("")),
        _ => Some(Some(&rows()[1].text)),
    }
}
#[prebindgen]
pub fn jni_buffer_optional_bytes(mode: i32) -> Option<Option<&'static [u8]>> {
    match mode {
        0 => None,
        1 => Some(None),
        2 => Some(Some(&[])),
        _ => Some(Some(&rows()[1].bytes)),
    }
}
#[prebindgen]
pub fn jni_buffer_owned() -> JniBufferRow {
    JniBufferRow {
        text: rows()[1].text.clone(),
        bytes: rows()[1].bytes.clone(),
    }
}
#[prebindgen]
pub fn jni_buffer_owned_optional(mode: i32) -> Option<Option<JniBufferRow>> {
    match mode {
        0 => None,
        1 => Some(None),
        _ => Some(Some(jni_buffer_owned())),
    }
}
#[prebindgen]
pub fn jni_buffer_row_text(row: &JniBufferRow) -> &str {
    &row.text
}
#[prebindgen]
pub fn jni_buffer_row_bytes(row: &JniBufferRow) -> &[u8] {
    &row.bytes
}
#[prebindgen]
pub fn jni_buffer_rows() -> &'static [JniBufferRow] {
    rows()
}
#[prebindgen]
pub fn jni_buffer_callback(
    callback: impl Fn(&str, &[u8], Option<&str>, Option<&[u8]>) + Send + Sync,
) {
    for row in rows() {
        callback(&row.text, &row.bytes, Some(&row.text), Some(&row.bytes));
    }
    callback("", &[], None, None);
}
#[prebindgen]
pub fn jni_buffer_owned_callback(callback: impl Fn(Option<JniBufferRow>) + Send + Sync) {
    callback(None);
    callback(Some(jni_buffer_owned()));
}

#[prebindgen]
pub fn jni_buffer_failure(callback: impl Fn(CallbackOwned, &str, &[u8]) + Send + Sync) {
    let before = crate::callback_owned_drops();
    callback(
        crate::callback_owned_new(41),
        jni_buffer_text(),
        jni_buffer_bytes(),
    );
    assert_eq!(crate::callback_owned_drops(), before + 1);
    callback(
        crate::callback_owned_new(42),
        jni_buffer_text(),
        jni_buffer_bytes(),
    );
    assert_eq!(crate::callback_owned_drops(), before + 2);
}

#[prebindgen]
pub fn jni_buffer_wide() -> JniBufferRow {
    jni_buffer_owned()
}

#[prebindgen]
pub fn jni_buffer_worker(callback: impl Fn(&str, &[u8]) + Send + Sync + 'static) {
    std::thread::spawn(move || {
        for _ in 0..64 {
            let row = jni_buffer_owned();
            callback(&row.text, &row.bytes);
        }
    })
    .join()
    .unwrap();
}

#[prebindgen]
pub struct JniBufferResourceRow {
    pub resource: CallbackOwned,
    pub text: String,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn jni_buffer_resource() -> JniBufferResourceRow {
    JniBufferResourceRow {
        resource: crate::callback_owned_new(71),
        text: "buffer result".into(),
        bytes: vec![1, 2, 3],
    }
}
#[prebindgen]
pub fn jni_buffer_domain_failure() -> Result<(), JniBufferResourceRow> {
    Err(jni_buffer_resource())
}
#[prebindgen]
pub fn jni_buffer_resource_rows() -> Vec<JniBufferResourceRow> {
    (0..3).map(|_| jni_buffer_resource()).collect()
}
