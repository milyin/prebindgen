//! Actual generated C calls: completed rows and partial native outputs own once.
use super::*;

macro_rules! close_rows {
    ($rows:expr, $row:ty, $free:ident) => {{
        let rows = $rows;
        if rows.len != 0 {
            for row in core::slice::from_raw_parts(rows.data.cast::<$row>(), rows.len) {
                perftest_drop_c_output_resource(row.v0);
                let mut text = row.v1;
                perftest_drop_owned_string(&mut text);
                perftest_drop_c_output_resource(row.v2);
            }
        }
        $free(rows);
    }};
}
unsafe fn close_nested(rows: __PbRows) {
    if rows.len != 0 {
        for row in core::slice::from_raw_parts(
            rows.data.cast::<__PbRow_c_output_nested_ret_w2>(),
            rows.len,
        ) {
            close_rows!(
                row.v0,
                __PbRow_c_output_nested_ret_w6,
                perftest_c_output_nested_ret_w6_rows_free
            );
        }
    }
    perftest_c_output_nested_ret_w2_rows_free(rows);
}
unsafe extern "C" fn receive(context: *mut core::ffi::c_void, rows: __PbRows) {
    *(context as *mut usize) += 1;
    if rows.len != 0 {
        for row in core::slice::from_raw_parts(
            rows.data
                .cast::<__PbRow_callback_4cf53ee6dc85aa46_callback_arg_0_w0>(),
            rows.len,
        ) {
            close_rows!(
                row.v0,
                __PbRow_callback_4cf53ee6dc85aa46_callback_arg_0_w4,
                perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w4_rows_free
            );
        }
    }
    perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w0_rows_free(rows);
}
fn sentinel() -> __PbRows {
    __PbRows {
        data: core::ptr::dangling::<u8>().cast(),
        len: usize::MAX,
    }
}
fn untouched(rows: __PbRows) {
    assert_eq!(rows.data, sentinel().data);
    assert_eq!(rows.len, usize::MAX);
}

#[test]
fn generated_buffered_outputs_release_partial_nested_and_independent_rows() {
    unsafe {
        assert_eq!(perftest_callback_row_resource_live(), 0);
        for fail in [-1, 0, 1, 2] {
            let mut error = core::ptr::null_mut();
            let mut rows = sentinel();
            let ok = perftest_c_output_rows(fail, &mut rows, &mut error);
            assert_eq!(ok, fail < 0);
            if ok {
                assert_eq!(rows.len, 3);
                close_rows!(
                    rows,
                    __PbRow_c_output_rows_ret_w2,
                    perftest_c_output_rows_ret_w2_rows_free
                );
            } else {
                untouched(rows);
                assert!(!error.is_null());
                perftest_free_error(error);
            }
            assert_eq!(perftest_callback_row_resource_live(), 0);
            rows = sentinel();
            error = core::ptr::null_mut();
            let ok = perftest_c_output_nested(fail, &mut rows, &mut error);
            assert_eq!(ok, fail < 0);
            if ok {
                assert_eq!(rows.len, 3);
                close_nested(rows);
            } else {
                untouched(rows);
                assert!(!error.is_null());
                perftest_free_error(error);
            }
            assert_eq!(perftest_callback_row_resource_live(), 0);
            let mut first = sentinel();
            let mut second = sentinel();
            error = core::ptr::null_mut();
            let ok = perftest_c_output_pair(fail, &mut first, &mut second, &mut error);
            assert_eq!(ok, fail < 0);
            if ok {
                assert_eq!(first.len, 3);
                assert_eq!(second.len, 3);
                close_rows!(
                    first,
                    __PbRow_c_output_pair_ret_w3,
                    perftest_c_output_pair_ret_w3_rows_free
                );
                close_rows!(
                    second,
                    __PbRow_c_output_pair_ret_w13,
                    perftest_c_output_pair_ret_w13_rows_free
                );
            } else {
                untouched(first);
                untouched(second);
                assert!(!error.is_null());
                perftest_free_error(error);
            }
            assert_eq!(perftest_callback_row_resource_live(), 0);
            let mut calls = 0usize;
            perftest_c_output_callback(
                fail,
                perftest_callback_4cf53ee6dc85aa46 {
                    run: Some(receive),
                    context: (&mut calls as *mut usize).cast(),
                    drop: None,
                },
            );
            assert_eq!(calls, usize::from(fail < 0));
            assert_eq!(perftest_callback_row_resource_live(), 0);
        }
    }
}
