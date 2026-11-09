//! Exact native destruction counts for the opt-in Kotlin Cleaner lifecycle.
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicUsize, Ordering};

static DROPS: [AtomicUsize; 128] = [const { AtomicUsize::new(0) }; 128];

pub struct GcResourceImpl {
    id: usize,
    value: u64,
}
impl Drop for GcResourceImpl {
    fn drop(&mut self) {
        DROPS[self.id].fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub type GcResource = GcResourceImpl;

#[prebindgen]
pub fn gc_resource_new(id: u64, value: u64) -> GcResource {
    GcResourceImpl {
        id: id as usize,
        value,
    }
}
#[prebindgen]
pub fn gc_resource_value(value: &GcResource) -> u64 {
    value.value
}
#[prebindgen]
pub fn gc_resource_consume(value: GcResource) -> u64 {
    value.value
}
#[prebindgen]
pub fn gc_resource_fail(value: GcResource) -> Result<u64, i64> {
    drop(value);
    Err(17)
}
#[prebindgen]
pub fn gc_resource_pair(first: GcResource, second: GcResource) -> u64 {
    first.value + second.value
}
#[prebindgen]
pub fn gc_resource_three(a: &GcResource, b: &GcResource, c: &GcResource) -> u64 {
    a.value + b.value + c.value
}
#[prebindgen]
pub fn gc_resource_consume_rows(values: Vec<GcResource>) -> u64 {
    values.into_iter().map(|value| value.value).sum()
}
#[prebindgen]
pub struct GcReadNumber {
    pub value: u64,
}
#[prebindgen]
pub fn gc_read_number(value: &GcResource) -> GcReadNumber {
    GcReadNumber { value: value.value }
}
#[prebindgen]
pub fn gc_resource_read_rows(values: Vec<GcReadNumber>) -> u64 {
    values.into_iter().map(|value| value.value).sum()
}
#[prebindgen]
pub fn gc_resource_drops(id: u64) -> u64 {
    DROPS[id as usize].load(Ordering::SeqCst) as u64
}
#[prebindgen]
pub fn gc_resource_emit(id: u64, value: u64, callback: impl Fn(GcResource)) {
    callback(gc_resource_new(id, value));
}
#[prebindgen]
pub fn gc_resource_emit_owned(id: u64, value: u64, callback: impl Fn(GcResource)) {
    callback(gc_resource_new(id, value));
}
#[prebindgen]
pub fn gc_resource_return(callback: impl Fn() -> GcResource) -> u64 {
    callback().value
}
#[prebindgen]
pub fn gc_resource_hold(value: &GcResource, callback: impl Fn()) -> u64 {
    callback();
    value.value
}
#[prebindgen]
pub fn gc_resource_rows(id: u64, count: u64) -> Vec<GcResource> {
    (0..count).map(|value| gc_resource_new(id, value)).collect()
}

/// A separate plain handle with the same payload for creation/read measurements.
#[prebindgen]
pub type PlainResource = GcResourceImpl;
#[prebindgen]
pub fn plain_resource_new(id: u64, value: u64) -> PlainResource {
    GcResourceImpl {
        id: id as usize,
        value,
    }
}
#[prebindgen]
pub fn plain_resource_value(value: &PlainResource) -> u64 {
    value.value
}
