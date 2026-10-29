//! Bulk C-representation transport fixtures.
use prebindgen_proc_macro::prebindgen;
#[prebindgen]
pub fn bulk_scalar(value: i64) -> i64 {
    value.wrapping_add(1)
}
#[prebindgen]
pub fn bulk_numbers(values: Vec<i64>) -> Vec<i64> {
    values
}
#[prebindgen]
pub fn bulk_nested(values: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    values
}
#[prebindgen]
pub fn bulk_text(value: String) -> String {
    value
}
#[prebindgen]
pub fn bulk_strings(values: Vec<String>) -> Vec<String> {
    values
}
#[prebindgen]
pub fn bulk_optional(value: Option<Vec<Option<String>>>) -> Option<Vec<Option<String>>> {
    value
}
#[prebindgen]
pub fn bulk_bytes(value: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    value
}
#[prebindgen]
#[repr(C)]
pub struct BulkPoint {
    pub x: i64,
    pub y: f64,
}
#[prebindgen]
pub fn bulk_point(value: BulkPoint) -> BulkPoint {
    value
}
#[prebindgen]
pub fn bulk_points(values: Vec<BulkPoint>) -> Vec<BulkPoint> {
    values
}
#[prebindgen]
#[repr(C, align(32))]
pub struct BulkNative {
    pub prefix: i32,
    pub number: i64,
    pub unsigned: u64,
}
#[prebindgen]
pub fn bulk_native(values: Vec<BulkNative>) -> Vec<BulkNative> {
    values
}
#[prebindgen]
pub struct BulkExpanded {
    pub number: i64,
    pub text: String,
}
#[prebindgen]
pub fn bulk_expanded_from(number: i64, text: String) -> BulkExpanded {
    BulkExpanded { number, text }
}
#[prebindgen]
pub fn bulk_expanded_number(value: &BulkExpanded) -> i64 {
    value.number
}
#[prebindgen]
pub fn bulk_expanded_text(value: &BulkExpanded) -> &str {
    &value.text
}
#[prebindgen]
pub fn bulk_expanded(value: BulkExpanded) -> BulkExpanded {
    value
}
#[prebindgen]
pub fn bulk_expanded_rows(values: Vec<BulkExpanded>) -> Vec<BulkExpanded> {
    values
}
static BULK_DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[prebindgen]
pub struct BulkHandle {
    pub value: i64,
}
impl Drop for BulkHandle {
    fn drop(&mut self) {
        BULK_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn bulk_handle_new(value: i64) -> BulkHandle {
    BulkHandle { value }
}
#[prebindgen]
pub fn bulk_handle_value(value: &BulkHandle) -> i64 {
    value.value
}
#[prebindgen]
pub fn bulk_drops() -> usize {
    BULK_DROPS.load(std::sync::atomic::Ordering::SeqCst)
}
#[prebindgen]
pub fn bulk_handles(values: Vec<BulkHandle>) -> Vec<BulkHandle> {
    values
}
#[prebindgen]
pub fn bulk_borrow(value: &BulkHandle) -> i64 {
    value.value
}
#[prebindgen]
pub fn bulk_checked(value: BulkHandle, number: u8) -> BulkHandle {
    let _ = number;
    value
}
#[prebindgen]
pub struct BulkFailure {
    pub code: i64,
}
#[prebindgen]
pub fn bulk_failure_code(value: &BulkFailure) -> i64 {
    value.code
}
#[prebindgen]
pub fn bulk_result(value: BulkExpanded, success: bool) -> Result<BulkExpanded, BulkFailure> {
    if success {
        Ok(value)
    } else {
        Err(BulkFailure { code: 17 })
    }
}
#[prebindgen]
pub fn bulk_callback(value: impl Fn(Vec<i64>) -> Vec<i64>) -> Vec<i64> {
    value(vec![1, 2, 3])
}
#[prebindgen]
pub fn bulk_handle_callback(value: impl Fn(Vec<BulkHandle>) -> Vec<BulkHandle>) -> Vec<BulkHandle> {
    value(vec![BulkHandle { value: 6 }, BulkHandle { value: 9 }])
}
#[prebindgen]
pub fn bulk_wide(values: Vec<i128>) -> Vec<i128> {
    values
}
#[prebindgen]
pub fn bulk_unsigned_wide(values: Vec<u128>) -> Vec<u128> {
    values
}
#[prebindgen]
pub struct BulkLate {
    pub panic: bool,
}
impl Drop for BulkLate {
    fn drop(&mut self) {
        assert!(!self.panic, "bulk late drop");
    }
}
#[prebindgen]
pub fn bulk_late(panic: bool) -> BulkLate {
    BulkLate { panic }
}
#[prebindgen]
pub fn bulk_late_handle(_owner: &BulkLate) -> BulkHandle {
    BulkHandle { value: 33 }
}
#[prebindgen]
pub fn bulk_handle_views(value: i64) -> BulkHandle {
    BulkHandle { value }
}

#[prebindgen]
pub fn bulk_array(value: [i64; 3]) -> [i64; 3] {
    value
}
#[prebindgen]
pub fn bulk_array_borrow(value: &[i64; 3]) -> i64 {
    value.iter().sum()
}
#[prebindgen]
pub fn bulk_point_array(value: [BulkPoint; 2]) -> [BulkPoint; 2] {
    value
}

#[prebindgen]
pub fn bulk_point_plain(value: BulkPoint) -> BulkPoint {
    value
}
#[prebindgen]
pub fn bulk_optional_handles(
    value: Option<Vec<Option<BulkHandle>>>,
) -> Option<Vec<Option<BulkHandle>>> {
    value
}
#[prebindgen]
pub fn bulk_slice(value: &[BulkPoint]) -> Vec<BulkPoint> {
    value.iter().map(|v| BulkPoint { x: v.x, y: v.y }).collect()
}
#[prebindgen]
pub fn bulk_select(value: i64) -> i64 {
    value
}
#[prebindgen]
#[repr(i32)]
pub enum BulkKind {
    A = -3,
    B = 9,
}
#[prebindgen]
pub fn bulk_kinds(value: Vec<BulkKind>) -> Vec<BulkKind> {
    value
}
#[prebindgen]
pub fn bulk_flags(value: Vec<bool>) -> Vec<bool> {
    value
}

#[prebindgen]
pub fn bulk_units(value: Vec<()>) -> Vec<()> {
    value
}

#[prebindgen]
pub struct BulkEnvelope {
    pub point: BulkPoint,
    pub tag: i64,
}
#[prebindgen]
pub fn bulk_envelope(value: Option<BulkEnvelope>) -> Option<BulkEnvelope> {
    value
}
#[prebindgen]
pub fn bulk_optional_wide(value: Option<i128>) -> Option<i128> {
    value
}
#[prebindgen]
pub fn bulk_optional_callback(value: impl Fn() -> Option<Vec<BulkHandle>>) -> usize {
    value().map_or(0, |v| v.len())
}
#[prebindgen]
pub struct BulkPrimitives {
    pub flag: bool,
    pub byte: i8,
    pub short: i16,
    pub octet: u8,
    pub word: u16,
    pub number: u32,
    pub scalar: char,
    pub address: usize,
}
#[prebindgen]
pub fn bulk_primitives(value: BulkPrimitives) -> BulkPrimitives {
    value
}
#[prebindgen]
pub fn bulk_callback_worker(value: impl Fn(Vec<i64>) -> Vec<i64> + Send + 'static) -> Vec<i64> {
    std::thread::spawn(move || value(vec![3, 4]))
        .join()
        .unwrap()
}
