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
