//! Owned buffer identities alongside observing output views.
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicUsize, Ordering};
static BUFFER_ADDRESS: AtomicUsize = AtomicUsize::new(0);

#[prebindgen]
pub fn buffer_views_text() -> String {
    let value = Box::<str>::from("move text").into_string();
    BUFFER_ADDRESS.store(value.as_ptr() as usize, Ordering::Relaxed);
    value
}
#[prebindgen]
pub fn buffer_views_text_len(value: &String) -> usize {
    assert_eq!(
        value.as_ptr() as usize,
        BUFFER_ADDRESS.load(Ordering::Relaxed)
    );
    value.len()
}
#[prebindgen]
pub fn buffer_views_bytes() -> Vec<u8> {
    let value: Box<[u8]> = Box::new([1, 2, 3, 4]);
    let value = value.into_vec();
    BUFFER_ADDRESS.store(value.as_ptr() as usize, Ordering::Relaxed);
    value
}
#[prebindgen]
pub fn buffer_views_bytes_len(value: &Vec<u8>) -> usize {
    assert_eq!(
        value.as_ptr() as usize,
        BUFFER_ADDRESS.load(Ordering::Relaxed)
    );
    value.len()
}
#[prebindgen]
pub fn buffer_views_address() -> usize {
    BUFFER_ADDRESS.load(Ordering::Relaxed)
}

#[prebindgen]
pub fn buffer_views_optional_text(present: bool) -> Option<String> {
    present.then(buffer_views_text)
}
#[prebindgen]
pub fn buffer_views_optional_text_len(value: &Option<String>) -> usize {
    value.as_ref().map(buffer_views_text_len).unwrap_or(0)
}
#[prebindgen]
pub fn buffer_views_optional_bytes(present: bool) -> Option<Vec<u8>> {
    present.then(buffer_views_bytes)
}
#[prebindgen]
pub fn buffer_views_optional_bytes_len(value: &Option<Vec<u8>>) -> usize {
    value.as_ref().map(buffer_views_bytes_len).unwrap_or(0)
}

static COLLECTION_INPUT_CONSTRUCTIONS: AtomicUsize = AtomicUsize::new(0);

#[prebindgen]
pub fn collection_input_plain(values: Vec<i64>, marker: u8) -> i64 {
    values.into_iter().sum::<i64>() + i64::from(marker)
}
#[prebindgen]
pub fn collection_input_bulk(values: Vec<i64>, marker: u8) -> i64 {
    collection_input_plain(values, marker)
}
#[prebindgen]
pub fn collection_input_custom(values: Vec<i64>, marker: u8) -> i64 {
    collection_input_plain(values, marker)
}
#[prebindgen]
pub fn collection_input_custom_bulk(values: Vec<i64>, marker: u8) -> i64 {
    collection_input_plain(values, marker)
}
#[prebindgen]
pub fn collection_input_value(value: i64) -> i64 {
    COLLECTION_INPUT_CONSTRUCTIONS.fetch_add(1, Ordering::Relaxed);
    value
}
#[prebindgen]
pub fn collection_input_constructions() -> usize {
    COLLECTION_INPUT_CONSTRUCTIONS.load(Ordering::Relaxed)
}
#[prebindgen]
pub fn collection_input_slice(values: &[i64]) -> i64 {
    values.iter().sum()
}
#[prebindgen]
pub fn collection_input_array(values: [i64; 4]) -> i64 {
    values.into_iter().sum()
}
#[prebindgen]
pub fn collection_input_optional(values: Option<Vec<i64>>) -> i64 {
    values.map(|v| v.into_iter().sum()).unwrap_or(-1)
}
#[prebindgen]
pub fn collection_input_narrow(values: Vec<i64>) -> i64 {
    values.into_iter().sum()
}
#[prebindgen]
pub fn collection_input_narrow_bulk(values: Vec<i64>) -> i64 {
    collection_input_narrow(values)
}
#[prebindgen]
pub fn collection_input_narrow_value(value: u8) -> i64 {
    collection_input_value(i64::from(value))
}
