//! Execute the generated ABI in Rust as well, so Miri can check provenance.
use super::*;
use perftest_flat::{GetterChild, GetterOwner};

unsafe fn finish(owner: *mut GetterOwner, before: &[usize]) {
    perftest_drop_getter_owner(owner);
    let after = perftest_flat::getter_counts();
    assert_eq!(after[0], before[0] + 1);
    assert_eq!(after[1], before[1]); // No leaf or child clone is requested.
}

#[test]
fn owner_backed_outputs_preserve_provenance_and_cleanup() {
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe {
        let before = perftest_flat::getter_counts();
        let mut child = std::ptr::null();
        let owner = perftest_owner_view_shared(1, &mut child);
        assert!(!owner.is_null() && !child.is_null());
        assert_eq!(perftest_getter_child_number(child), 71);
        let name = perftest_getter_child_name(child);
        assert!(std::slice::from_raw_parts(name.data, name.len).starts_with(b"h"));
        perftest_free_buffer(name);
        assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
        finish(owner, &before);

        let before = perftest_flat::getter_counts();
        let mut owner = std::ptr::null_mut();
        let child = perftest_owner_view_reordered(0, &mut owner);
        assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
        finish(owner, &before);

        let before = perftest_flat::getter_counts();
        let mut child = std::ptr::null_mut();
        let owner = perftest_owner_view_mutable(0, &mut child);
        perftest_getter_child_set_number(child, 88);
        // Reading the owner ends the child loan. Do not use child again.
        assert_eq!(perftest_getter_owner_id(owner), 88);
        finish(owner, &before);

        for present in [0, 1] {
            let before = perftest_flat::getter_counts();
            let mut child = std::ptr::null();
            let owner = perftest_owner_view_optional(present, &mut child);
            assert_eq!(child.is_null(), present == 0);
            if present != 0 {
                assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
            }
            finish(owner, &before);

            let before = perftest_flat::getter_counts();
            let mut child = std::ptr::null_mut();
            let owner = perftest_owner_view_optional_mut(present, &mut child);
            assert_eq!(child.is_null(), present == 0);
            if present != 0 {
                perftest_getter_child_set_number(child, 89);
            }
            assert_eq!(
                perftest_getter_owner_id(owner),
                if present != 0 { 89 } else { 71 }
            );
            finish(owner, &before);
        }

        let mut child = std::ptr::null();
        assert!(perftest_owner_view_maybe(0, &mut child).is_null());
        assert!(child.is_null());
        let before = perftest_flat::getter_counts();
        let owner = perftest_owner_view_maybe(1, &mut child);
        assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
        finish(owner, &before);

        let before = perftest_flat::getter_counts();
        let text = b"temporary";
        let owner = perftest_owner_view_from_temporary(
            __PbBuffer {
                data: text.as_ptr(),
                len: text.len(),
            },
            &mut child,
        );
        assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
        finish(owner, &before);

        for fail in [0, 1] {
            let before = perftest_flat::getter_counts();
            let mut child: *const GetterChild = std::ptr::null();
            let mut code = 123;
            let mut error = std::ptr::null_mut();
            let owner = perftest_owner_view_checked(fail, &mut child, &mut code, &mut error);
            if fail == 0 {
                assert!(error.is_null());
                assert_eq!(code, 0);
                assert_eq!(perftest_getter_owner_child_matches(owner, child), 1);
                finish(owner, &before);
            } else {
                assert!(owner.is_null() && child.is_null() && !error.is_null());
                assert_eq!(code, 123);
                perftest_free_error(error);
                assert_eq!(perftest_flat::getter_counts()[0], before[0] + 1);
            }
        }

        // A source temporary panics while dropping AFTER ownership commit.
        // Result rollback must free the owner and the separate owned resource,
        // but never the interior child pointer stored in the result packet.
        let before = perftest_flat::getter_counts();
        let resources = perftest_flat::owned_resource_drops();
        let mut child = std::ptr::null();
        let mut resource = std::ptr::null_mut();
        let mut error = std::ptr::null_mut();
        assert!(perftest_owner_view_rollback(&mut child, &mut resource, &mut error).is_null());
        assert!(child.is_null() && resource.is_null() && !error.is_null());
        perftest_free_error(error);
        assert_eq!(perftest_flat::getter_counts()[0], before[0] + 1);
        assert_eq!(perftest_flat::owned_resource_drops(), resources + 1);
    }
}
