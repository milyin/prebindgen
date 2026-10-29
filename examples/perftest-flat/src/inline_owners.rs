//! Native ownership fixtures: spare capacity and padded, non-Clone resources.
use prebindgen_proc_macro::prebindgen;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
thread_local! {
    static BYTES: RefCell<Option<Vec<u8>>> = const {RefCell::new(None)};
    static TEXT: RefCell<Option<String>> = const {RefCell::new(None)};
    static NUMBERS: RefCell<Option<Vec<i64>>> = const {RefCell::new(None)};
}
static DROPS: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
#[derive(Default)]
pub struct InlineChild {
    pub id: u64,
}
#[prebindgen]
pub type InlineChildBox = Option<Box<InlineChild>>;
#[prebindgen]
pub struct InlinePacket {
    pub tag: u8,
    #[cfg(feature = "layout-wide")]
    pub wide: usize,
    pub bytes: Vec<u8>,
    pub text: String,
    pub child: InlineChildBox,
}
impl Drop for InlinePacket {
    fn drop(&mut self) {
        if self.tag != 0 {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
}
/// Prepare outside allocation-counted exports.
pub fn inline_stage(bytes: Vec<u8>, text: String, numbers: Vec<i64>) {
    BYTES.with(|v| *v.borrow_mut() = Some(bytes));
    TEXT.with(|v| *v.borrow_mut() = Some(text));
    NUMBERS.with(|v| *v.borrow_mut() = Some(numbers));
}
#[prebindgen]
pub fn inline_bytes() -> Vec<u8> {
    BYTES.with(|v| v.borrow_mut().take().unwrap())
}
#[prebindgen]
pub fn inline_text() -> String {
    TEXT.with(|v| v.borrow_mut().take().unwrap())
}
#[prebindgen]
pub fn inline_numbers() -> Vec<i64> {
    NUMBERS.with(|v| v.borrow_mut().take().unwrap())
}
#[prebindgen]
pub fn inline_bytes_new() -> Vec<u8> {
    let mut v = Vec::with_capacity(4096);
    v.extend_from_slice(b"a\0b");
    v
}
#[prebindgen]
pub fn inline_text_new() -> String {
    let mut v = String::with_capacity(4096);
    v.push_str("héllo");
    v
}
#[prebindgen]
pub fn inline_numbers_new() -> Vec<i64> {
    let mut v = Vec::with_capacity(4096);
    v.extend_from_slice(&[7, 9]);
    v
}
#[prebindgen]
pub fn inline_bytes_capacity(value: &Vec<u8>) -> usize {
    value.capacity()
}
#[prebindgen]
pub fn inline_text_capacity(value: &String) -> usize {
    value.capacity()
}
#[prebindgen]
pub fn inline_numbers_capacity(value: &Vec<i64>) -> usize {
    value.capacity()
}
#[prebindgen]
pub fn inline_bytes_take(value: Vec<u8>) -> usize {
    value.capacity()
}
#[prebindgen]
pub fn inline_text_take(value: String) -> usize {
    value.capacity()
}
#[prebindgen]
pub fn inline_numbers_take(value: Vec<i64>) -> i64 {
    value.into_iter().sum()
}
#[prebindgen]
pub fn inline_bytes_alias(value: Vec<u8>, borrow: &Vec<u8>) -> Result<usize, String> {
    Ok(value.len() + borrow.len())
}
#[prebindgen]
pub fn inline_packet_new() -> InlinePacket {
    InlinePacket {
        tag: 1,
        #[cfg(feature = "layout-wide")]
        wide: 0,
        bytes: inline_bytes_new(),
        text: inline_text_new(),
        child: Some(Box::new(InlineChild { id: 42 })),
    }
}
#[prebindgen]
pub fn inline_packet_len(value: &InlinePacket) -> usize {
    value.bytes.len()
}
#[prebindgen]
pub fn inline_packet_ref(value: &InlinePacket) -> &InlineChild {
    value.child.as_ref().unwrap()
}
#[prebindgen]
pub fn inline_child_id(value: &InlineChild) -> u64 {
    value.id
}
#[prebindgen]
pub fn inline_packet_take(value: InlinePacket) -> usize {
    value.bytes.capacity()
}
#[prebindgen]
pub fn inline_packet_drops() -> usize {
    DROPS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn inline_packet_panic(_value: &InlinePacket) -> usize {
    panic!("expected post-owner conversion failure")
}
#[prebindgen]
pub fn inline_packet_result() -> Result<InlinePacket, String> {
    Ok(inline_packet_new())
}
#[prebindgen]
pub fn inline_packet_optional(present: bool) -> Option<InlinePacket> {
    present.then(inline_packet_new)
}
#[prebindgen]
pub fn inline_packet_callback(callback: impl Fn(InlinePacket) -> InlinePacket) -> usize {
    callback(inline_packet_new()).bytes.capacity()
}
#[prebindgen]
pub fn inline_bytes_callback(callback: impl Fn(Vec<u8>) -> Vec<u8>) -> usize {
    callback(inline_bytes_new()).capacity()
}
#[prebindgen]
pub fn inline_packet_error(value: &String) -> String {
    value.clone()
}
#[prebindgen]
pub struct InlineReturn {
    pub packet: InlinePacket,
    pub letter: char,
}
#[prebindgen]
pub fn inline_return_new(packet: InlinePacket, letter: char) -> InlineReturn {
    InlineReturn { packet, letter }
}
#[prebindgen]
pub fn inline_return_catch(callback: impl Fn() -> InlineReturn) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).is_err()
}
#[prebindgen]
pub fn inline_rows_catch(callback: impl Fn() -> Vec<InlineReturn>) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).is_err()
}

#[prebindgen]
pub fn inline_packet_empty() -> InlinePacket {
    InlinePacket {
        tag: 0,
        #[cfg(feature = "layout-wide")]
        wide: 0,
        bytes: Vec::new(),
        text: String::new(),
        child: None,
    }
}
#[prebindgen]
pub fn inline_boxed_numbers_new() -> Vec<i32> {
    vec![7, 9]
}
#[prebindgen]
pub fn inline_boxed_numbers_len(value: &Vec<i32>) -> usize {
    value.len()
}
#[prebindgen]
pub fn inline_boxed_numbers_take(value: Vec<i32>) -> i32 {
    value.into_iter().sum()
}

#[prebindgen]
pub fn inline_bytes_clear(value: &mut Vec<u8>) -> usize {
    value.clear();
    value.capacity()
}
#[prebindgen]
pub fn inline_bytes_mutable() -> Vec<u8> {
    inline_bytes_new()
}
#[prebindgen]
pub fn inline_packets_take(values: Vec<InlinePacket>) -> usize {
    values.iter().map(|v| v.bytes.len()).sum()
}
#[prebindgen]
pub fn inline_packets_rows() -> Result<i64, String> {
    Ok(7)
}
#[prebindgen]
pub fn inline_packets_getter(_value: &i64) -> Vec<InlinePacket> {
    vec![inline_packet_new(), inline_packet_new()]
}
#[prebindgen]
pub fn inline_packets_late(_value: &i64) -> usize {
    panic!("expected failure after preparing native owner rows")
}
