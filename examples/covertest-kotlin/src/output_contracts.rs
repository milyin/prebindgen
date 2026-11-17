pub struct DurationPair {
    pub a: i64,
    pub b: i64,
}
pub fn duration_echo(milliseconds: i64) -> i64 {
    milliseconds
}
pub fn duration_packet(milliseconds: i64) -> i64 {
    milliseconds
}
pub fn duration_series() -> Vec<i64> {
    vec![3, -5, 8]
}
pub fn duration_pair() -> DurationPair {
    DurationPair { a: 17, b: -29 }
}
pub fn pair_a(pair: &DurationPair) -> i64 {
    pair.a
}
pub fn pair_b(pair: &DurationPair) -> i64 {
    pair.b
}
pub fn duration_optional(present: bool) -> Option<i64> {
    present.then_some(37)
}
pub fn duration_result(milliseconds: i64) -> Result<i64, i64> {
    if milliseconds < 0 {
        Err(-milliseconds)
    } else {
        Ok(milliseconds)
    }
}
pub fn plain_result(milliseconds: i64) -> Result<i64, i64> {
    duration_result(milliseconds)
}
pub fn duration_callback(callback: impl Fn(i64) -> i64) -> i64 {
    callback(43)
}
pub fn plain_callback(callback: impl Fn(i64) -> i64) -> i64 {
    callback(43)
}
