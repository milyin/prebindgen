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

// No explicit bulk expansion: the JNI adapter chooses the measured array default.
#[prebindgen]
pub struct DefaultBulkRow {
    pub number: i64,
    pub fraction: f64,
}
#[prebindgen]
pub fn default_bulk_echo(values: Vec<DefaultBulkRow>) -> Vec<DefaultBulkRow> {
    values
}
#[prebindgen]
pub fn default_bulk_array() -> [DefaultBulkRow; 2] {
    [
        DefaultBulkRow {
            number: i64::MIN,
            fraction: 1.5,
        },
        DefaultBulkRow {
            number: i64::MAX,
            fraction: -2.0,
        },
    ]
}
#[prebindgen]
pub fn default_bulk_optional(mode: i64) -> Option<Vec<DefaultBulkRow>> {
    match mode {
        0 => None,
        1 => Some(Vec::new()),
        _ => Some(default_bulk_array().into_iter().collect()),
    }
}
#[prebindgen]
pub fn default_bulk_numbers() -> Vec<i64> {
    vec![i64::MIN, 0, i64::MAX]
}
#[prebindgen]
pub fn default_bulk_plain() -> Vec<DefaultBulkRow> {
    default_bulk_array().into_iter().collect()
}
#[prebindgen]
pub fn default_bulk_callback(callback: impl Fn(Vec<DefaultBulkRow>)) {
    callback(default_bulk_plain());
}

#[prebindgen]
pub fn default_bulk_nested() -> Vec<Vec<i64>> {
    vec![vec![i64::MIN, 0, i64::MAX], Vec::new()]
}
#[prebindgen]
pub fn default_bulk_borrowed() -> &'static [DefaultBulkRow] {
    static ROWS: [DefaultBulkRow; 2] = [
        DefaultBulkRow {
            number: i64::MIN,
            fraction: 1.5,
        },
        DefaultBulkRow {
            number: i64::MAX,
            fraction: -2.0,
        },
    ];
    &ROWS
}

#[prebindgen]
pub fn default_bulk_callback_mixed(callback: impl Fn(Vec<Vec<i64>>, Vec<Vec<i64>>)) {
    callback(default_bulk_nested(), default_bulk_nested());
}

#[prebindgen]
pub fn default_bulk_units() -> Vec<()> {
    vec![(), ()]
}
#[prebindgen]
pub fn default_bulk_texts() -> Vec<String> {
    vec![String::new(), "héllo 世界\0".into()]
}
#[prebindgen]
pub fn default_bulk_buffers() -> Vec<Vec<u8>> {
    vec![vec![0, 255, 128], Vec::new()]
}

// Borrow text from an owned optional payload: packing must finish before Some drops.
#[prebindgen]
pub fn default_bulk_optional_texts() -> Vec<Option<String>> {
    vec![Some("héllo 世界".into()), None, Some(String::new())]
}

#[prebindgen]
pub struct BulkHandleText {
    pub handle: BulkHandle,
    pub text: String,
}
#[prebindgen]
pub fn bulk_handle_texts(values: Vec<BulkHandleText>) -> Vec<BulkHandleText> {
    values
}
#[prebindgen]
pub fn bulk_handle_text_callback(callback: impl Fn() -> Vec<BulkHandleText>) -> usize {
    callback().len()
}
#[prebindgen]
pub fn bulk_handle_nested(values: Vec<Vec<BulkHandle>>) -> Vec<Vec<BulkHandle>> {
    values
}

#[prebindgen]
pub struct BulkPatchRow {
    pub first: Option<BulkHandle>,
    pub children: Option<Vec<Option<Vec<Option<BulkHandle>>>>>,
    pub last: BulkHandle,
}
#[prebindgen]
pub fn bulk_patch_row_new(
    first: Option<BulkHandle>,
    children: Option<Vec<Option<Vec<Option<BulkHandle>>>>>,
    last: BulkHandle,
) -> BulkPatchRow {
    BulkPatchRow {
        first,
        children,
        last,
    }
}
#[prebindgen]
pub fn bulk_patch_rows(row: BulkPatchRow) -> Vec<i64> {
    let mut values = Vec::new();
    values.push(row.first.as_ref().map_or(-1, |h| h.value));
    if let Some(rows) = &row.children {
        values.push(rows.len() as i64);
        for children in rows {
            if let Some(children) = children {
                values.push(children.len() as i64);
                values.extend(children.iter().map(|h| h.as_ref().map_or(-3, |h| h.value)));
            } else {
                values.push(-4);
            }
        }
    } else {
        values.push(-2);
    }
    values.push(row.last.value);
    values
}
#[prebindgen]
pub fn bulk_patch_callback(
    callback: impl Fn() -> Vec<Option<Vec<Option<BulkHandle>>>>,
) -> Vec<i64> {
    let mut values = Vec::new();
    for children in callback() {
        if let Some(children) = children {
            values.push(children.len() as i64);
            values.extend(children.iter().map(|h| h.as_ref().map_or(-3, |h| h.value)));
        } else {
            values.push(-4);
        }
    }
    values
}
#[prebindgen]
pub struct BulkBorrowPair {
    pub number: i64,
}
#[prebindgen]
pub fn bulk_borrow_pair_new(handle: &BulkHandle, number: i64) -> BulkBorrowPair {
    BulkBorrowPair {
        number: handle.value + number,
    }
}
#[prebindgen]
pub fn bulk_optional_borrow_pair(value: Option<BulkBorrowPair>) -> i64 {
    value.map(|v| v.number).unwrap_or(-1)
}

#[prebindgen]
pub struct BulkPairText {
    pub borrowed_handle: BulkHandle,
    pub owned_handle: BulkHandle,
    pub text: String,
}
#[prebindgen]
pub fn bulk_handle_copy(value: &BulkHandle) -> BulkHandle {
    BulkHandle { value: value.value }
}
#[prebindgen]
pub fn bulk_pair_text_callback(callback: impl Fn() -> Vec<BulkPairText>) -> usize {
    callback().len()
}

#[prebindgen]
pub fn bulk_collection_lock_groups(
    first: Option<Vec<BulkHandle>>,
    second: Vec<BulkHandle>,
    probe: impl Fn() -> bool,
) -> bool {
    let result = probe();
    drop((first, second));
    result
}

#[prebindgen]
pub struct BulkInputEnvelope {
    pub values: Vec<Vec<i64>>,
}
#[prebindgen]
pub fn bulk_nested_envelope(values: Vec<BulkInputEnvelope>) -> i64 {
    values.into_iter().flat_map(|v| v.values).flatten().sum()
}

#[prebindgen]
pub fn bulk_input_envelope_new(values: Vec<Vec<i64>>) -> BulkInputEnvelope {
    BulkInputEnvelope { values }
}

// Row-local borrowed conversions used to verify bulk replay lifetimes and cleanup.
thread_local! {
    static REPLAY_TEXTS: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
    static REPLAY_BYTES: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
    static REPLAY_KINDS: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
    static REPLAY_CALLS: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
    static REPLAY_FAIL: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
}
#[prebindgen]
pub struct BulkReplayRow {
    pub first: BulkHandle,
    pub text: Option<String>,
    pub bytes: Option<Vec<u8>>,
    pub kind: Option<BulkKind>,
    pub last: BulkHandle,
}
#[prebindgen]
pub fn bulk_replay_text(value: &String) -> String {
    let call = REPLAY_TEXTS.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    assert!(
        !REPLAY_FAIL.with(|n| n.get() == call),
        "expected bulk row constructor panic"
    );
    value.clone()
}
#[prebindgen]
pub fn bulk_replay_bytes(value: &Vec<u8>) -> Vec<u8> {
    REPLAY_BYTES.with(|n| n.set(n.get() + 1));
    value.clone()
}
#[prebindgen]
pub fn bulk_replay_kind(value: &BulkKind) -> BulkKind {
    REPLAY_KINDS.with(|n| n.set(n.get() + 1));
    match value {
        BulkKind::A => BulkKind::A,
        BulkKind::B => BulkKind::B,
    }
}
#[prebindgen]
pub fn bulk_replay_rows(rows: Vec<BulkReplayRow>) -> usize {
    REPLAY_CALLS.with(|n| n.set(n.get() + 1));
    rows.iter()
        .map(|r| {
            r.first.value as usize
                + r.last.value as usize
                + r.text.as_ref().map_or(0, String::len)
                + r.bytes.as_ref().map_or(0, Vec::len)
                + r.kind.as_ref().map_or(0, |v| match v {
                    BulkKind::A => 1,
                    BulkKind::B => 2,
                })
        })
        .sum()
}
#[prebindgen]
pub fn bulk_replay_callback(callback: impl Fn() -> Vec<BulkReplayRow>) -> usize {
    bulk_replay_rows(callback())
}
#[prebindgen]
pub fn bulk_replay_reset(fail_text: usize) {
    REPLAY_TEXTS.with(|n| n.set(0));
    REPLAY_BYTES.with(|n| n.set(0));
    REPLAY_KINDS.with(|n| n.set(0));
    REPLAY_CALLS.with(|n| n.set(0));
    REPLAY_FAIL.with(|n| n.set(fail_text));
}
#[prebindgen]
pub fn bulk_replay_count(which: usize) -> usize {
    match which {
        0 => REPLAY_TEXTS.with(std::cell::Cell::get),
        1 => REPLAY_BYTES.with(std::cell::Cell::get),
        2 => REPLAY_KINDS.with(std::cell::Cell::get),
        _ => REPLAY_CALLS.with(std::cell::Cell::get),
    }
}
#[prebindgen]
pub fn bulk_replay_staged_rows(rows: Vec<BulkReplayRow>) -> usize {
    bulk_replay_rows(rows)
}

#[prebindgen]
pub fn bulk_replay_object_rows(rows: Vec<BulkReplayRow>) -> usize {
    bulk_replay_rows(rows)
}

#[prebindgen]
pub fn bulk_cleanup_nested_callback(
    callback: impl Fn() -> Vec<Option<Vec<Option<BulkHandle>>>> + Send + 'static,
    retry: bool,
    worker: bool,
) -> Vec<i64> {
    let call = move || {
        if retry {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback));
        }
        bulk_patch_callback(callback)
    };
    if worker {
        std::thread::spawn(call)
            .join()
            .unwrap_or_else(|failure| std::panic::resume_unwind(failure))
    } else {
        call()
    }
}

#[prebindgen]
pub fn bulk_cleanup_pair_callback(callback: impl Fn() -> BulkPairText) -> usize {
    let _ = callback();
    1
}
