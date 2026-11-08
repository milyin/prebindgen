//! Shared C inputs retain foreign storage; temporary reconstruction is explicit.
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub fn shared_input_checked(values: &[i64], expected: usize) -> Result<usize, String> {
    if values.len() != expected {
        return Err("unexpected input length".into());
    }
    Ok(values.as_ptr().addr())
}
#[prebindgen]
pub fn shared_input_echo(values: &[i64]) -> &[i64] {
    values
}
#[prebindgen]
pub fn shared_input_fixed(values: &[f64; 3]) -> Result<f64, String> {
    Ok(values.iter().sum())
}
#[prebindgen]
pub fn shared_input_optional(values: Option<&[i64]>) -> usize {
    values.map_or(usize::MAX, |v| v.len())
}
#[prebindgen]
pub fn shared_input_copy(values: &[i64]) -> usize {
    values.as_ptr().addr()
}
#[prebindgen]
pub fn shared_input_vector(values: &Vec<i64>) -> usize {
    values.as_ptr().addr()
}
#[prebindgen]
pub fn shared_input_string(value: &String) -> usize {
    value.as_ptr().addr()
}
#[prebindgen]
pub fn shared_input_bytes(value: &Vec<u8>) -> usize {
    value.as_ptr().addr()
}
#[prebindgen]
pub fn shared_input_optional_vector(values: Option<&Vec<i64>>) -> usize {
    values.map_or(usize::MAX, Vec::len)
}
#[prebindgen]
pub fn shared_input_text(value: &str) -> usize {
    value.as_ptr().addr()
}
#[prebindgen]
pub fn shared_input_qualified_bytes(values: &[::core::primitive::u8]) -> usize {
    values.as_ptr().addr()
}
#[prebindgen]
pub struct SharedInputRow {
    pub len: usize,
    pub first: i64,
}
#[prebindgen]
pub fn shared_input_row_new(values: &[i64]) -> SharedInputRow {
    SharedInputRow {
        len: values.len(),
        first: values.first().copied().unwrap_or(0),
    }
}
#[prebindgen]
pub fn shared_input_rows(rows: Vec<SharedInputRow>) -> i64 {
    rows.iter().map(|row| row.len as i64 + row.first).sum()
}

#[prebindgen]
pub fn shared_input_i8(values: &[i8]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_i16(values: &[i16]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_i32(values: &[i32]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_i64(values: &[i64]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_isize(values: &[isize]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_u16(values: &[u16]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_u32(values: &[u32]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_u64(values: &[u64]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_usize(values: &[usize]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_f32(values: &[f32]) -> usize {
    values.as_ptr().addr()
}

#[prebindgen]
pub fn shared_input_f64(values: &[f64]) -> usize {
    values.as_ptr().addr()
}
