//! Public Kotlin parameter names across direct and expanded input shapes.
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub struct ParameterPoint {
    pub x: i64,
    pub y: i64,
}
#[prebindgen]
pub fn parameter_point(x: i64, y: i64) -> ParameterPoint {
    ParameterPoint { x, y }
}
#[prebindgen]
pub fn parameter_sum(point: ParameterPoint) -> i64 {
    point.x + point.y
}
#[prebindgen]
pub fn parameter_bulk_sum(point: ParameterPoint) -> i64 {
    parameter_sum(point)
}
#[prebindgen]
pub fn parameter_sum_object(point: ParameterPoint) -> i64 {
    parameter_sum(point)
}
#[prebindgen]
pub fn parameter_optional(point: Option<ParameterPoint>) -> i64 {
    point.map(parameter_sum).unwrap_or(-1)
}
#[prebindgen]
pub fn parameter_variants(point: ParameterPoint) -> i64 {
    parameter_sum(point)
}

#[prebindgen]
#[allow(non_snake_case)]
pub fn parameter_collisions(
    foo_bar: i64,
    fooBar: i64,
    on_error: i64,
    return_capture: i64,
    r#when: i64,
    io: i64,
) -> i64 {
    foo_bar + 10 * fooBar + 100 * on_error + 1000 * return_capture + 10_000 * r#when + 100_000 * io
}

#[prebindgen]
pub struct WireNamedFields {
    pub p0_w1: i64,
    pub p1_w1: String,
}
#[prebindgen]
pub fn parameter_record(value: WireNamedFields) -> WireNamedFields {
    value
}
#[prebindgen]
pub fn parameter_records(items: Vec<WireNamedFields>) -> Vec<WireNamedFields> {
    items
}

#[prebindgen]
pub fn parameter_external(org: i64, com: i64, callback: impl Fn(ParameterPoint)) -> ParameterPoint {
    callback(ParameterPoint { x: org, y: com });
    ParameterPoint { x: org, y: com }
}
