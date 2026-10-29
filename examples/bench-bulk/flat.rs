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
