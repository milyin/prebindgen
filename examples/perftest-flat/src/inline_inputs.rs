//! Inline-owner rows, including panicking replacement and row constructors.
use crate::{owned_resource_id, InlinePacket, OwnedResource};
use prebindgen_proc_macro::prebindgen;
use std::cell::Cell;
thread_local! {
    static EMPTY_CALLS: Cell<usize> = const { Cell::new(0) };
    static EMPTY_PANIC: Cell<usize> = const { Cell::new(0) };
    static CONSTRUCTORS: Cell<usize> = const { Cell::new(0) };
    static CONSTRUCTOR_PANIC: Cell<usize> = const { Cell::new(0) };
    static CALLS: Cell<usize> = const { Cell::new(0) };
}
pub(crate) fn empty_event() {
    let index = EMPTY_CALLS.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    assert!(
        !EMPTY_PANIC.with(|n| n.get() == index),
        "inline empty factory panic"
    );
}
fn constructor() {
    let index = CONSTRUCTORS.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    assert!(
        !CONSTRUCTOR_PANIC.with(|n| n.get() == index),
        "inline row constructor panic"
    );
}
fn packet(value: &InlinePacket) -> i64 {
    assert_eq!(value.bytes.capacity(), 4096);
    assert_eq!(value.bytes.as_slice(), b"a\0b");
    assert_eq!(value.child.as_ref().unwrap().id, 42);
    value.bytes.len() as i64
}
#[prebindgen]
pub fn inline_input_mixed(first: OwnedResource, value: InlinePacket, last: OwnedResource) -> i64 {
    constructor();
    owned_resource_id(&first) + packet(&value) + owned_resource_id(&last)
}
#[prebindgen]
pub fn inline_input_pair(first: InlinePacket, second: InlinePacket) -> i64 {
    constructor();
    packet(&first) + packet(&second)
}
#[prebindgen]
pub fn inline_input_borrowed(value: &InlinePacket) -> usize {
    constructor();
    packet(value) as usize
}
#[prebindgen]
pub fn inline_inputs_mixed(rows: Vec<i64>) -> Result<i64, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(rows.iter().sum())
}
#[prebindgen]
pub fn inline_inputs_mixed_fast(rows: Vec<i64>) -> Result<i64, String> {
    inline_inputs_mixed(rows)
}
#[prebindgen]
pub fn inline_inputs_pair(rows: Vec<i64>) -> Result<i64, String> {
    inline_inputs_mixed(rows)
}
#[prebindgen]
pub fn inline_inputs_pair_fast(rows: Vec<i64>) -> Result<i64, String> {
    inline_inputs_mixed(rows)
}
#[prebindgen]
pub fn inline_inputs_packets(rows: Vec<InlinePacket>) -> Result<usize, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(rows.iter().map(|v| packet(v) as usize).sum())
}
#[prebindgen]
pub fn inline_inputs_packets_fast(rows: Vec<InlinePacket>) -> Result<usize, String> {
    inline_inputs_packets(rows)
}
#[prebindgen]
pub fn inline_inputs_optional(rows: Option<Vec<Option<i64>>>) -> Result<i64, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(rows.map_or(-1, |rows| rows.iter().flatten().sum()))
}
#[prebindgen]
pub fn inline_inputs_borrow(
    receiver: &mut InlinePacket,
    rows: Vec<usize>,
) -> Result<usize, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(packet(receiver) as usize + rows.iter().sum::<usize>())
}
#[prebindgen]
pub fn inline_inputs_bytes(rows: Vec<Vec<u8>>) -> Result<usize, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(rows.iter().fold(0, |out, v| {
        assert_eq!(v.capacity(), 4096);
        out ^ (v.as_ptr() as usize)
    }))
}
#[prebindgen]
pub fn inline_inputs_texts(rows: Vec<String>) -> Result<usize, String> {
    CALLS.with(|n| n.set(n.get() + 1));
    Ok(rows.iter().fold(0, |out, v| {
        assert_eq!(v.capacity(), 4096);
        out ^ (v.as_ptr() as usize)
    }))
}
#[prebindgen]
pub fn inline_inputs_reset(empty_panic: usize, constructor_panic: usize) {
    EMPTY_CALLS.with(|n| n.set(0));
    EMPTY_PANIC.with(|n| n.set(empty_panic));
    CONSTRUCTORS.with(|n| n.set(0));
    CONSTRUCTOR_PANIC.with(|n| n.set(constructor_panic));
    CALLS.with(|n| n.set(0));
}
#[prebindgen]
pub fn inline_inputs_empty_calls() -> usize {
    EMPTY_CALLS.with(Cell::get)
}
#[prebindgen]
pub fn inline_inputs_constructor_calls() -> usize {
    CONSTRUCTORS.with(Cell::get)
}
#[prebindgen]
pub fn inline_inputs_calls() -> usize {
    CALLS.with(Cell::get)
}
