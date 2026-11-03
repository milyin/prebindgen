use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();

pub struct ResourceImpl(u64);
#[prebindgen]
pub type Resource = ResourceImpl;
#[prebindgen]
pub fn resource_value(value: &Resource) -> u64 { value.0 }
#[prebindgen]
pub struct Record { pub first: Resource, pub tag: u64, pub second: Resource }
#[prebindgen]
pub fn record_into(value: Record) -> Record { value }
#[prebindgen]
pub fn make_handles(n: usize) -> Vec<Resource> {
    std::hint::black_box((0..n).map(|i| ResourceImpl(i as u64)).collect())
}
#[prebindgen]
pub fn make_records(n: usize) -> Vec<Record> {
    std::hint::black_box((0..n).map(|i| Record {
        first: ResourceImpl(i as u64), tag: i as u64, second: ResourceImpl(i as u64 + 1)
    }).collect())
}
#[prebindgen]
pub fn make_nested(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box(vec![vec![], make_handles(n), vec![]])
}
#[prebindgen]
pub fn make_many(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box((0..n).map(|i| vec![ResourceImpl(i as u64)]).collect())
}
#[prebindgen]
pub fn make_numbers(n: usize) -> Vec<u64> {
    std::hint::black_box((0..n as u64).collect())
}
#[prebindgen]
pub fn control(n: usize) -> u64 { std::hint::black_box(n as u64) }
