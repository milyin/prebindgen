//! C row replay with borrowed buffers, ownership transfer and failing decoding.
use crate::{owned_resource_id, OwnedResource, ResultFailure};
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicUsize, Ordering};

static CONSTRUCTORS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

#[prebindgen]
pub struct BufferInputRow {
    pub resource: OwnedResource,
    pub text: String,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn buffer_row_new(resource: OwnedResource, text: String, bytes: Vec<u8>) -> BufferInputRow {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    assert_ne!(
        owned_resource_id(&resource),
        -2,
        "buffer row constructor panic"
    );
    BufferInputRow {
        resource,
        text,
        bytes,
    }
}
fn sum(row: &BufferInputRow) -> i64 {
    owned_resource_id(&row.resource)
        + row.text.len() as i64
        + row.bytes.iter().map(|b| i64::from(*b)).sum::<i64>()
}
#[prebindgen]
pub fn buffer_rows_sum(rows: Vec<BufferInputRow>) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows.iter().map(sum).sum())
}
#[prebindgen]
pub fn buffer_rows_optional(
    rows: Option<Vec<Option<BufferInputRow>>>,
) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows
        .map(|rows| rows.iter().flatten().map(sum).sum())
        .unwrap_or(-1))
}
#[prebindgen]
pub fn buffer_row_borrow(resource: &OwnedResource, text: &str, bytes: &[u8]) -> i64 {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    owned_resource_id(resource)
        + text.len() as i64
        + bytes.iter().map(|b| i64::from(*b)).sum::<i64>()
}
#[prebindgen]
pub fn buffer_rows_borrowed(
    receiver: &mut OwnedResource,
    rows: Vec<i64>,
) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(owned_resource_id(receiver) + rows.iter().sum::<i64>())
}
#[prebindgen]
pub fn buffer_row_temporary(resource: OwnedResource, text: &String, bytes: &Vec<u8>) -> i64 {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    owned_resource_id(&resource)
        + text.len() as i64
        + bytes.iter().map(|b| i64::from(*b)).sum::<i64>()
}
#[prebindgen]
pub fn buffer_rows_temporaries(rows: Vec<i64>) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows.iter().sum())
}
#[prebindgen]
pub fn buffer_row_reset_counts() {
    CONSTRUCTORS.store(0, Ordering::SeqCst);
    CALLS.store(0, Ordering::SeqCst);
}
#[prebindgen]
pub fn buffer_row_constructor_calls() -> usize {
    CONSTRUCTORS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn buffer_row_function_calls() -> usize {
    CALLS.load(Ordering::SeqCst)
}
