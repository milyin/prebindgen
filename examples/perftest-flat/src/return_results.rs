//! Return-capture reentry and cleanup after native ownership commit.
// The wide error intentionally tests chunking across the JVM argument limit.
#![allow(clippy::result_large_err)]
use crate::{option_resource_new, prebindgen, OwnedResource};

pub struct ReturnOwnerImpl {
    value: i64,
    panic: bool,
    callback: Box<dyn Fn() + Send + Sync>,
}
impl Drop for ReturnOwnerImpl {
    fn drop(&mut self) {
        (self.callback)();
        assert!(!self.panic, "return owner drop panic");
    }
}
#[prebindgen]
pub type ReturnOwner = ReturnOwnerImpl;
#[prebindgen]
pub fn return_ownership(
    value: i64,
    panic: bool,
    callback: impl Fn() + Send + Sync + 'static,
) -> ReturnOwner {
    ReturnOwnerImpl {
        value,
        panic,
        callback: Box::new(callback),
    }
}
#[prebindgen]
pub fn return_single_ownership(
    value: i64,
    panic: bool,
    callback: impl Fn() + Send + Sync + 'static,
) -> ReturnOwner {
    return_ownership(value, panic, callback)
}
#[prebindgen]
pub fn return_owner_value(value: &ReturnOwner) -> i64 {
    value.value
}
#[prebindgen]
pub fn return_owner_resource(_value: &ReturnOwner) -> OwnedResource {
    option_resource_new(true).unwrap()
}

pub struct ReturnFailureImpl {
    value: i64,
    panic: bool,
    callback: Box<dyn Fn() + Send + Sync>,
}
impl Drop for ReturnFailureImpl {
    fn drop(&mut self) {
        (self.callback)();
        assert!(!self.panic, "return error drop panic");
    }
}
#[prebindgen]
pub type ReturnFailure = ReturnFailureImpl;
#[prebindgen]
pub fn return_failure_code(value: &ReturnFailure) -> i64 {
    value.value
}
#[prebindgen]
pub fn return_domain(
    value: i64,
    panic: bool,
    callback: impl Fn() + Send + Sync + 'static,
) -> Result<i64, ReturnFailure> {
    Err(ReturnFailureImpl {
        value,
        panic,
        callback: Box::new(callback),
    })
}
#[prebindgen]
pub struct ReturnPair {
    pub first: i64,
    pub second: i64,
}
#[prebindgen]
pub fn return_pair(value: i64) -> ReturnPair {
    ReturnPair {
        first: value,
        second: value + 1,
    }
}
#[prebindgen]
pub fn return_pair_first(value: &ReturnPair) -> i64 {
    value.first
}
#[prebindgen]
pub fn return_pair_second(value: &ReturnPair) -> i64 {
    value.second
}
#[prebindgen]
pub fn return_wide(value: i64) -> i64 {
    value
}

#[prebindgen]
pub struct ReturnArgA {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: i64,
    pub v9: i64,
    pub v10: i64,
    pub v11: i64,
    pub v12: i64,
    pub v13: i64,
    pub v14: i64,
    pub v15: i64,
    pub v16: i64,
    pub v17: i64,
    pub v18: i64,
    pub v19: i64,
    pub v20: i64,
    pub v21: i64,
    pub v22: i64,
    pub v23: i64,
    pub v24: i64,
    pub v25: i64,
    pub v26: i64,
    pub v27: i64,
    pub v28: i64,
    pub v29: i64,
    pub v30: i64,
    pub v31: i64,
    pub v32: i64,
    pub v33: i64,
    pub v34: i64,
    pub v35: i64,
    pub v36: i64,
    pub v37: i64,
    pub v38: i64,
    pub v39: i64,
    pub v40: i64,
    pub v41: i64,
    pub v42: i64,
    pub v43: i64,
    pub v44: i64,
    pub v45: i64,
    pub v46: i64,
    pub v47: i64,
    pub v48: i64,
    pub v49: i64,
    pub v50: i64,
    pub v51: i64,
    pub v52: i64,
    pub v53: i64,
    pub v54: i64,
    pub v55: i64,
    pub v56: i64,
    pub v57: i64,
    pub v58: i64,
    pub v59: i64,
    pub v60: i64,
    pub v61: i64,
    pub v62: i64,
}

#[prebindgen]
pub struct ReturnArgB {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: i64,
    pub v9: i64,
    pub v10: i64,
    pub v11: i64,
    pub v12: i64,
    pub v13: i64,
    pub v14: i64,
    pub v15: i64,
    pub v16: i64,
    pub v17: i64,
    pub v18: i64,
    pub v19: i64,
    pub v20: i64,
    pub v21: i64,
    pub v22: i64,
    pub v23: i64,
    pub v24: i64,
    pub v25: i64,
    pub v26: i64,
    pub v27: i64,
    pub v28: i64,
    pub v29: i64,
    pub v30: i64,
    pub v31: i64,
    pub v32: i64,
    pub v33: i64,
    pub v34: i64,
    pub v35: i64,
    pub v36: i64,
    pub v37: i64,
    pub v38: i64,
    pub v39: i64,
    pub v40: i64,
    pub v41: i64,
    pub v42: i64,
    pub v43: i64,
    pub v44: i64,
    pub v45: i64,
    pub v46: i64,
    pub v47: i64,
    pub v48: i64,
    pub v49: i64,
    pub v50: i64,
    pub v51: i64,
    pub v52: i64,
    pub v53: i64,
    pub v54: i64,
    pub v55: i64,
    pub v56: i64,
    pub v57: i64,
    pub v58: i64,
    pub v59: i64,
    pub v60: i64,
    pub v61: i64,
    pub v62: i64,
    pub v63: i64,
}

#[prebindgen]
pub struct ReturnWideA {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: i64,
    pub v9: i64,
    pub v10: i64,
    pub v11: i64,
    pub v12: i64,
    pub v13: i64,
    pub v14: i64,
    pub v15: i64,
    pub v16: i64,
    pub v17: i64,
    pub v18: i64,
    pub v19: i64,
    pub v20: i64,
    pub v21: i64,
    pub v22: i64,
    pub v23: i64,
    pub v24: i64,
    pub v25: i64,
    pub v26: i64,
    pub v27: i64,
    pub v28: i64,
    pub v29: i64,
    pub v30: i64,
    pub v31: i64,
    pub v32: i64,
    pub v33: i64,
    pub v34: i64,
    pub v35: i64,
    pub v36: i64,
    pub v37: i64,
    pub v38: i64,
    pub v39: i64,
    pub v40: i64,
    pub v41: i64,
    pub v42: i64,
    pub v43: i64,
    pub v44: i64,
    pub v45: i64,
    pub v46: i64,
    pub v47: i64,
    pub v48: i64,
    pub v49: i64,
    pub v50: i64,
    pub v51: i64,
    pub v52: i64,
    pub v53: i64,
    pub v54: i64,
    pub v55: i64,
    pub v56: i64,
    pub v57: i64,
    pub v58: i64,
    pub v59: i64,
    pub v60: i64,
    pub v61: i64,
    pub v62: i64,
    pub v63: i64,
}

#[prebindgen]
pub struct ReturnWideB {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: i64,
    pub v9: i64,
    pub v10: i64,
    pub v11: i64,
    pub v12: i64,
    pub v13: i64,
    pub v14: i64,
    pub v15: i64,
    pub v16: i64,
    pub v17: i64,
    pub v18: i64,
    pub v19: i64,
    pub v20: i64,
    pub v21: i64,
    pub v22: i64,
    pub v23: i64,
    pub v24: i64,
    pub v25: i64,
    pub v26: i64,
    pub v27: i64,
    pub v28: i64,
    pub v29: i64,
    pub v30: i64,
    pub v31: i64,
    pub v32: i64,
    pub v33: i64,
    pub v34: i64,
    pub v35: i64,
    pub v36: i64,
    pub v37: i64,
    pub v38: i64,
    pub v39: i64,
    pub v40: i64,
    pub v41: i64,
    pub v42: i64,
    pub v43: i64,
    pub v44: i64,
    pub v45: i64,
    pub v46: i64,
    pub v47: i64,
    pub v48: i64,
    pub v49: i64,
    pub v50: i64,
    pub v51: i64,
    pub v52: i64,
    pub v53: i64,
    pub v54: i64,
    pub v55: i64,
    pub v56: i64,
    pub v57: i64,
    pub v58: i64,
    pub v59: i64,
    pub v60: i64,
    pub v61: i64,
    pub v62: i64,
    pub v63: i64,
    pub v64: i64,
    pub v65: i64,
}

#[prebindgen]
pub struct ReturnArgOuter {
    pub a: ReturnArgA,
    pub b: ReturnArgB,
}
#[prebindgen]
pub fn return_limited(value: ReturnArgOuter, flag: bool) -> i64 {
    value.a.v0 + value.a.v62 + value.b.v0 + value.b.v63 + i64::from(flag)
}
#[prebindgen]
pub struct ReturnWideError {
    pub a: ReturnWideA,
    pub b: ReturnWideB,
}
#[prebindgen]
pub fn return_wide_error() -> Result<(), ReturnWideError> {
    Err(ReturnWideError {
        a: ReturnWideA {
            v0: 0,
            v1: 1,
            v2: 2,
            v3: 3,
            v4: 4,
            v5: 5,
            v6: 6,
            v7: 7,
            v8: 8,
            v9: 9,
            v10: 10,
            v11: 11,
            v12: 12,
            v13: 13,
            v14: 14,
            v15: 15,
            v16: 16,
            v17: 17,
            v18: 18,
            v19: 19,
            v20: 20,
            v21: 21,
            v22: 22,
            v23: 23,
            v24: 24,
            v25: 25,
            v26: 26,
            v27: 27,
            v28: 28,
            v29: 29,
            v30: 30,
            v31: 31,
            v32: 32,
            v33: 33,
            v34: 34,
            v35: 35,
            v36: 36,
            v37: 37,
            v38: 38,
            v39: 39,
            v40: 40,
            v41: 41,
            v42: 42,
            v43: 43,
            v44: 44,
            v45: 45,
            v46: 46,
            v47: 47,
            v48: 48,
            v49: 49,
            v50: 50,
            v51: 51,
            v52: 52,
            v53: 53,
            v54: 54,
            v55: 55,
            v56: 56,
            v57: 57,
            v58: 58,
            v59: 59,
            v60: 60,
            v61: 61,
            v62: 62,
            v63: 63,
        },
        b: ReturnWideB {
            v0: 0,
            v1: 1,
            v2: 2,
            v3: 3,
            v4: 4,
            v5: 5,
            v6: 6,
            v7: 7,
            v8: 8,
            v9: 9,
            v10: 10,
            v11: 11,
            v12: 12,
            v13: 13,
            v14: 14,
            v15: 15,
            v16: 16,
            v17: 17,
            v18: 18,
            v19: 19,
            v20: 20,
            v21: 21,
            v22: 22,
            v23: 23,
            v24: 24,
            v25: 25,
            v26: 26,
            v27: 27,
            v28: 28,
            v29: 29,
            v30: 30,
            v31: 31,
            v32: 32,
            v33: 33,
            v34: 34,
            v35: 35,
            v36: 36,
            v37: 37,
            v38: 38,
            v39: 39,
            v40: 40,
            v41: 41,
            v42: 42,
            v43: 43,
            v44: 44,
            v45: 45,
            v46: 46,
            v47: 47,
            v48: 48,
            v49: 49,
            v50: 50,
            v51: 51,
            v52: 52,
            v53: 53,
            v54: 54,
            v55: 55,
            v56: 56,
            v57: 57,
            v58: 58,
            v59: 59,
            v60: 60,
            v61: 61,
            v62: 62,
            v63: 63,
            v64: 64,
            v65: 65,
        },
    })
}
