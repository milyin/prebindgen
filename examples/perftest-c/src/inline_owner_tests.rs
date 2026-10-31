#[cfg(test)]
mod tests {
    use super::*;
    static OWNERS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    unsafe extern "C" fn echo_packet(
        _: *mut core::ffi::c_void,
        packet: __PbOwned_packet,
    ) -> __PbOwned_packet {
        packet
    }
    unsafe extern "C" fn bad_pair(
        _: *mut core::ffi::c_void,
    ) -> owner_callback_e857880e5782d2cb_result {
        owner_callback_e857880e5782d2cb_result {
            v0: owner_inline_packet_new(),
            v1: 0xd800,
        }
    }
    unsafe extern "C" fn bad_rows(context: *mut core::ffi::c_void) -> __PbRows {
        let rows = &*(context as *const Vec<__PbRow_callback_1782e472ad837f0b_callback_return_w1>);
        __PbRows {
            data: rows.as_ptr().cast(),
            len: rows.len(),
        }
    }
    unsafe extern "C" fn mixed_rows(context: *mut core::ffi::c_void) -> __PbRows {
        let rows = &*(context as *const Vec<__PbRow_callback_d1e1f55a20f3880c_callback_return_w1>);
        __PbRows { data: rows.as_ptr().cast(), len: rows.len() }
    }
    #[test]
    fn grouped_rows_move_inline_storage_before_deriving_its_pointer() {
        let _guard = OWNERS.lock().unwrap();
        unsafe {
            let before = owner_inline_packet_drops();
            let mut rows: Vec<_> = (0..32).map(|i| __PbRow_callback_d1e1f55a20f3880c_callback_return_w1 {
                v0: owner_inline_packet_new(),
                v1: Box::into_raw(Box::new(perftest_flat::InlineChild { id: i })),
            }).collect();
            let total = owner_inline_mixed_rows_sum(owner_callback_d1e1f55a20f3880c {
                run: Some(mixed_rows), context: (&mut rows as *mut Vec<_>).cast(),
                drop: None, release_result: None,
            });
            assert_eq!(total, (0..32).sum::<u64>() + 32 * 3);
            assert_eq!(owner_inline_packet_drops(), before + 32);
        }
    }
    #[test]
    fn padded_native_owners_survive_wire_moves_callbacks_and_rollback() {
        let _guard = OWNERS.lock().unwrap();
        unsafe {
            let mut capacity = 0;
            let mut bytes = owner_inline_bytes_mutable(&mut capacity);
            assert_eq!(owner_bytes_len(&bytes), 0);
            assert_eq!(capacity, 4096);
            owner_drop_bytes(&mut bytes);
            let mut error = core::mem::MaybeUninit::uninit();
            assert!(!crate::perftest_c_result_panic_error(error.as_mut_ptr()));
            let mut error = error.assume_init();
            assert_eq!(crate::perftest_owned_string_len(&error.v0), 0);
            assert!(!crate::perftest_owned_string_view(&error.v0).data.is_null());
            crate::perftest_drop_panic_error(&mut error);
            crate::perftest_drop_panic_error(&mut error);
            let before = owner_inline_packet_drops();
            let packet = owner_inline_packet_new();
            // Moving the raw carrier models C returning/storing its opaque value.
            let mut moved = ::core::mem::MaybeUninit::<__PbOwned_packet>::uninit();
            ::core::ptr::copy_nonoverlapping(&packet, moved.as_mut_ptr(), 1);
            let mut moved = moved.assume_init();
            let child = owner_inline_packet_ref(&moved);
            assert_eq!(owner_inline_child_id(child), 42);
            assert_eq!(moved.as_rust().bytes.capacity(), 4096);
            assert_eq!(owner_inline_packet_take(&mut moved), 4096);
            owner_drop_packet(&mut moved);
            owner_drop_packet(&mut moved);
            assert_eq!(owner_inline_packet_drops(), before + 1);
            assert_eq!(
                owner_inline_packet_callback(owner_callback_7a0d2e4eb3c7fbb1 {
                    run: Some(echo_packet),
                    context: core::ptr::null_mut(),
                    drop: None
                }),
                4096
            );
            assert_eq!(owner_inline_packet_drops(), before + 2);
            assert_eq!(
                owner_inline_return_catch(owner_callback_e857880e5782d2cb {
                    run: Some(bad_pair),
                    context: core::ptr::null_mut(),
                    drop: None
                }),
                1
            );
            assert_eq!(owner_inline_packet_drops(), before + 3);
            let mut output = ::core::mem::MaybeUninit::uninit();
            let mut len = 0;
            let mut error = core::ptr::null_mut();
            assert!(!owner_inline_packet_result(
                output.as_mut_ptr(),
                &mut len,
                &mut error
            ));
            assert!(!error.is_null());
            owner_free_error(error);
            assert_eq!(owner_inline_packet_drops(), before + 4);
            let mut rows = vec![
                __PbRow_callback_1782e472ad837f0b_callback_return_w1 {
                    v0: owner_inline_packet_new(),
                    v1: 'a' as u32,
                },
                __PbRow_callback_1782e472ad837f0b_callback_return_w1 {
                    v0: owner_inline_packet_new(),
                    v1: 0xd800,
                },
            ];
            assert_eq!(
                owner_inline_rows_catch(owner_callback_1782e472ad837f0b {
                    run: Some(bad_rows),
                    context: (&mut rows as *mut Vec<_>).cast(),
                    drop: None,
                    release_result: None
                }),
                1
            );
            assert_eq!(owner_inline_packet_drops(), before + 6);
            let mut rows = __PbRows {
                data: core::ptr::null(),
                len: 0,
            };
            let mut count = 123;
            let mut error = core::ptr::null_mut();
            assert!(!owner_inline_packets_rows(
                &mut rows, &mut count, &mut error
            ));
            assert!(rows.data.is_null());
            assert_eq!(count, 123);
            owner_free_error(error);
            assert_eq!(owner_inline_packet_drops(), before + 8);
            let mut first = owner_inline_packet_new();
            let mut second = owner_inline_packet_new();
            let rows = [
                __PbRow_inline_packets_take_p0_w1 { v0: &mut first },
                __PbRow_inline_packets_take_p0_w1 { v0: &mut second },
            ];
            assert_eq!(
                owner_inline_packets_take(__PbRows {
                    data: rows.as_ptr().cast(),
                    len: rows.len()
                }),
                6
            );
            owner_drop_packet(&mut first);
            owner_drop_packet(&mut second);
            assert_eq!(owner_inline_packet_drops(), before + 10);
        }
    }
}
