use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();
use std::sync::atomic::{AtomicI64, Ordering};
static LIVE: AtomicI64 = AtomicI64::new(0);
#[prebindgen]
pub fn live() -> i64 {
    LIVE.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn control(value: i64) -> i64 {
    std::hint::black_box(value) + 1
}
#[prebindgen]
pub struct A {
    pub value: i64,
}
impl Drop for A {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn a_new(value: i64) -> A {
    LIVE.fetch_add(1, Ordering::SeqCst);
    A { value }
}
#[prebindgen]
pub fn a_read(value: &A) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub fn a_mut(value: &mut A) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub struct B {
    pub value: i64,
}
impl Drop for B {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn b_new(value: i64) -> B {
    LIVE.fetch_add(1, Ordering::SeqCst);
    B { value }
}
#[prebindgen]
pub fn b_read(value: &B) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub fn b_mut(value: &mut B) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub struct C {
    pub value: i64,
}
impl Drop for C {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn c_new(value: i64) -> C {
    LIVE.fetch_add(1, Ordering::SeqCst);
    C { value }
}
#[prebindgen]
pub fn c_read(value: &C) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub fn c_mut(value: &mut C) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub struct D {
    pub value: i64,
}
impl Drop for D {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn d_new(value: i64) -> D {
    LIVE.fetch_add(1, Ordering::SeqCst);
    D { value }
}
#[prebindgen]
pub fn d_read(value: &D) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub fn d_mut(value: &mut D) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub struct Token {
    pub value: i64,
}
impl Drop for Token {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn token_new(value: i64) -> Token {
    LIVE.fetch_add(1, Ordering::SeqCst);
    Token { value }
}
#[prebindgen]
pub fn token_read(value: &Token) -> i64 {
    std::hint::black_box(value).value
}
#[prebindgen]
pub fn token_mut(value: &mut Token) -> i64 {
    std::hint::black_box(value).value
}

#[prebindgen]
pub fn read_one(values: Vec<i64>) -> i64 {
    std::hint::black_box(values).into_iter().sum()
}
#[prebindgen]
pub fn mut_one(values: Vec<i64>) -> i64 {
    std::hint::black_box(values).into_iter().sum()
}
#[prebindgen]
pub fn mut_nested(values: Vec<Vec<i64>>) -> i64 {
    std::hint::black_box(values).into_iter().flatten().sum()
}
#[prebindgen]
pub fn mixed_shared(a: Vec<i64>, b: Vec<i64>, c: Vec<i64>, d: Vec<i64>, token: &mut Token) -> i64 {
    std::hint::black_box((a, b, c, d, token)).pipe_sum()
}
trait Sum {
    fn pipe_sum(self) -> i64;
}
impl Sum for (Vec<i64>, Vec<i64>, Vec<i64>, Vec<i64>, &mut Token) {
    fn pipe_sum(self) -> i64 {
        self.0
            .into_iter()
            .chain(self.1)
            .chain(self.2)
            .chain(self.3)
            .sum::<i64>()
            + self.4.value
    }
}
#[prebindgen]
pub fn mixed_mut(a: Vec<i64>, b: Vec<i64>, c: Vec<i64>, d: Vec<i64>) -> i64 {
    let (a, b, c, d) = std::hint::black_box((a, b, c, d));
    a.into_iter().chain(b).chain(c).chain(d).sum()
}
#[prebindgen]
pub fn mixed_same(reads: Vec<i64>, writes: Vec<i64>) -> i64 {
    std::hint::black_box((reads, writes)).0.into_iter().sum()
}
#[prebindgen]
pub fn owned_echo(values: Vec<A>) -> Vec<A> {
    std::hint::black_box(values)
}
