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
