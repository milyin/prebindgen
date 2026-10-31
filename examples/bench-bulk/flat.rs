use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();

#[prebindgen]
pub struct Row {
    pub a: i64,
    pub b: i64,
    pub c: i64,
    pub d: i64,
    pub e: i64,
    pub f: i64,
    pub g: i64,
    pub h: i64,
}
impl Row {
    fn new(value: i64) -> Self {
        Self {
            a: value,
            b: value + 1,
            c: value + 2,
            d: value + 3,
            e: value + 4,
            f: value + 5,
            g: value + 6,
            h: value + 7,
        }
    }
    fn sum(&self) -> i64 {
        self.a + self.b + self.c + self.d + self.e + self.f + self.g + self.h
    }
    fn checksum(&self) -> i64 {
        1 * self.a
            + 2 * self.b
            + 3 * self.c
            + 4 * self.d
            + 5 * self.e
            + 6 * self.f
            + 7 * self.g
            + 8 * self.h
    }
}
#[prebindgen]
pub fn scalar(value: i64) -> i64 {
    std::hint::black_box(value) + 1
}
#[prebindgen]
pub fn row_sum(value: Row) -> i64 {
    std::hint::black_box(value).sum()
}
#[prebindgen]
pub fn numbers_sum(values: Vec<i64>) -> i64 {
    std::hint::black_box(values).iter().sum()
}
#[prebindgen]
pub fn rows_sum(values: &[Row]) -> i64 {
    std::hint::black_box(values).iter().map(Row::sum).sum()
}
#[prebindgen]
pub fn rows_make(count: i64, seed: i64) -> Vec<Row> {
    (0..std::hint::black_box(count))
        .map(|i| Row::new(seed + i))
        .collect()
}
#[prebindgen]
pub fn rows_echo(values: Vec<Row>) -> Vec<Row> {
    std::hint::black_box(values)
}
#[prebindgen]
pub type Handler = HandlerImpl;
pub struct HandlerImpl(Box<dyn Fn(Vec<Row>) -> i64 + Send + Sync>);
#[prebindgen]
pub fn handler_new(callback: impl Fn(Vec<Row>) -> i64 + Send + Sync + 'static) -> Handler {
    HandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn fire(handler: &Handler, count: i64, seed: i64) -> i64 {
    (handler.0)(rows_make(count, seed))
}

#[prebindgen]
pub struct Packet {
    pub header: Row,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn packet_sum(packet: Packet) -> i64 {
    let packet = std::hint::black_box(packet);
    packet.header.sum() + packet.bytes.iter().map(|b| i64::from(*b)).sum::<i64>()
}

// Separate entry point keeps the ordinary materialized-list baseline untouched.
#[prebindgen]
#[inline]
pub fn rows_for_reduce(count: i64, seed: i64) -> Vec<Row> {
    rows_make(count, seed)
}

// Single-record cases contain no source arrays or collections.
#[prebindgen]
pub fn row_make(seed: i64) -> Row {
    Row::new(std::hint::black_box(seed))
}
#[prebindgen]
pub struct WideRow {
    pub f0: i64,
    pub f1: i64,
    pub f2: i64,
    pub f3: i64,
    pub f4: i64,
    pub f5: i64,
    pub f6: i64,
    pub f7: i64,
    pub f8: i64,
    pub f9: i64,
    pub f10: i64,
    pub f11: i64,
    pub f12: i64,
    pub f13: i64,
    pub f14: i64,
    pub f15: i64,
    pub f16: i64,
    pub f17: i64,
    pub f18: i64,
    pub f19: i64,
    pub f20: i64,
    pub f21: i64,
    pub f22: i64,
    pub f23: i64,
    pub f24: i64,
    pub f25: i64,
    pub f26: i64,
    pub f27: i64,
    pub f28: i64,
    pub f29: i64,
    pub f30: i64,
    pub f31: i64,
}
impl WideRow {
    fn new(seed: i64) -> Self {
        Self {
            f0: seed + 0,
            f1: seed + 1,
            f2: seed + 2,
            f3: seed + 3,
            f4: seed + 4,
            f5: seed + 5,
            f6: seed + 6,
            f7: seed + 7,
            f8: seed + 8,
            f9: seed + 9,
            f10: seed + 10,
            f11: seed + 11,
            f12: seed + 12,
            f13: seed + 13,
            f14: seed + 14,
            f15: seed + 15,
            f16: seed + 16,
            f17: seed + 17,
            f18: seed + 18,
            f19: seed + 19,
            f20: seed + 20,
            f21: seed + 21,
            f22: seed + 22,
            f23: seed + 23,
            f24: seed + 24,
            f25: seed + 25,
            f26: seed + 26,
            f27: seed + 27,
            f28: seed + 28,
            f29: seed + 29,
            f30: seed + 30,
            f31: seed + 31,
        }
    }
    fn checksum(&self) -> i64 {
        1 * self.f0
            + 2 * self.f1
            + 3 * self.f2
            + 4 * self.f3
            + 5 * self.f4
            + 6 * self.f5
            + 7 * self.f6
            + 8 * self.f7
            + 9 * self.f8
            + 10 * self.f9
            + 11 * self.f10
            + 12 * self.f11
            + 13 * self.f12
            + 14 * self.f13
            + 15 * self.f14
            + 16 * self.f15
            + 17 * self.f16
            + 18 * self.f17
            + 19 * self.f18
            + 20 * self.f19
            + 21 * self.f20
            + 22 * self.f21
            + 23 * self.f22
            + 24 * self.f23
            + 25 * self.f24
            + 26 * self.f25
            + 27 * self.f26
            + 28 * self.f27
            + 29 * self.f28
            + 30 * self.f29
            + 31 * self.f30
            + 32 * self.f31
    }
}
#[prebindgen]
pub fn wide_row_make(seed: i64) -> WideRow {
    WideRow::new(std::hint::black_box(seed))
}
#[prebindgen]
pub fn wide_row_sum(value: WideRow) -> i64 {
    std::hint::black_box(value).checksum()
}

#[prebindgen]
pub type RowReturnHandler = RowReturnHandlerImpl;
pub struct RowReturnHandlerImpl(Box<dyn Fn(i64) -> Row + Send + Sync>);
#[prebindgen]
pub fn row_return_handler_new(
    callback: impl Fn(i64) -> Row + Send + Sync + 'static,
) -> RowReturnHandler {
    RowReturnHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn row_return_fire(handler: &RowReturnHandler, seed: i64) -> i64 {
    std::hint::black_box((handler.0)(seed)).checksum()
}

#[prebindgen]
pub type WideRowReturnHandler = WideRowReturnHandlerImpl;
pub struct WideRowReturnHandlerImpl(Box<dyn Fn(i64) -> WideRow + Send + Sync>);
#[prebindgen]
pub fn wide_row_return_handler_new(
    callback: impl Fn(i64) -> WideRow + Send + Sync + 'static,
) -> WideRowReturnHandler {
    WideRowReturnHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn wide_row_return_fire(handler: &WideRowReturnHandler, seed: i64) -> i64 {
    std::hint::black_box((handler.0)(seed)).checksum()
}

// Identical in both bindings: a control for variation between fresh JVMs.
#[prebindgen]
pub fn control(value: i64) -> i64 {
    std::hint::black_box(value) + 1
}

#[prebindgen]
pub struct InputHandle { pub value: i64 }
#[prebindgen]
pub fn input_handle_new(value:i64)->InputHandle { InputHandle {value} }
#[prebindgen]
pub fn input_handle_value(value:&InputHandle)->i64 { value.value }
#[prebindgen]
pub fn input_handle_copy(value:&InputHandle)->InputHandle { InputHandle { value:value.value } }
#[prebindgen]
pub struct HandleRecord { pub handle:InputHandle, pub tag:i64 }
#[prebindgen]
pub fn handle_values_sum(values:Vec<i64>)->i64 { std::hint::black_box(values).into_iter().sum() }
#[prebindgen]
pub fn handle_records_sum(values:Vec<HandleRecord>)->i64 {
    std::hint::black_box(values).into_iter().map(|row|row.handle.value+row.tag).sum()
}

#[prebindgen]
pub fn owned_handles_echo(values:Vec<InputHandle>)->Vec<InputHandle> { std::hint::black_box(values) }
