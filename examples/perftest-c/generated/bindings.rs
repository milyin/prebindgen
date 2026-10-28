#[allow(dead_code)]
struct __PbOutputOwner<T: ?Sized>(*mut T);
impl<T: ?Sized> Drop for __PbOutputOwner<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0));
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __Pb128 {
    pub hi: u64,
    pub lo: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRows {
    pub data: *const ::core::ffi::c_void,
    pub len: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbBuffer {
    pub data: *const u8,
    pub len: usize,
}
unsafe fn __pb_report_message(message: String, error: *mut *mut std::ffi::c_char) {
    if !error.is_null() {
        std::ptr::write(
            error,
            std::ffi::CString::new(message.replace('\0', "?")).unwrap().into_raw(),
        );
    }
}
/**# Safety
The pointer must be null or an unfreed error string returned by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_free_error(error: *mut std::ffi::c_char) {
    if !error.is_null() {
        drop(std::ffi::CString::from_raw(error));
    }
}
unsafe fn __pb_read<'a>(b: __PbBuffer) -> Result<&'a [u8], String> {
    if b.len == 0 {
        return Ok(&[]);
    }
    if b.data.is_null() || b.len > isize::MAX as usize {
        return Err("invalid buffer".into());
    }
    Ok(std::slice::from_raw_parts(b.data, b.len))
}
/**# Safety
    The buffer must have been returned by this library and not already freed.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_free_buffer(b: __PbBuffer) {
    if !b.data.is_null() {
        drop(
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8, b.len)),
        );
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_getter_owner(
    ptr: *mut perftest_flat::GetterOwner,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_getter_child(
    ptr: *mut perftest_flat::GetterChild,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_getter_leaf(ptr: *mut perftest_flat::GetterLeaf) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_panic_output(
    ptr: *mut perftest_flat::CPanicOutput,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_callback_return_resource(
    ptr: *mut perftest_flat::CallbackReturnResource,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_callback_return_job(
    ptr: *mut perftest_flat::CallbackReturnJob,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_callback_return_handler(
    ptr: *mut perftest_flat::CallbackReturnHandler,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_enum_payload(
    ptr: *mut perftest_flat::PayloadHandle,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_storage(ptr: *mut perftest_flat::Storage) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_blob(ptr: *mut perftest_flat::BlobValue) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_report(ptr: *mut perftest_flat::Report) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_report_struct(
    ptr: *mut perftest_flat::ReportStruct,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_summary(ptr: *mut perftest_flat::Summary) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_unsigned(ptr: *mut perftest_flat::Unsigned) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_archive(ptr: *mut perftest_flat::Archive) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_owned_resource(
    ptr: *mut perftest_flat::OwnedResource,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_niche_choice(
    ptr: *mut perftest_flat::NicheChoiceHandle,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_option_ctor(ptr: *mut perftest_flat::OptionCtor) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_option_choice(
    ptr: *mut perftest_flat::OptionChoice,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
struct __PbCallbackResult<T: Copy> {
    value: T,
    context: *mut ::core::ffi::c_void,
    release: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *const T)>,
}
impl<T: Copy> Drop for __PbCallbackResult<T> {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            unsafe { release(self.context, &self.value) }
        }
    }
}
struct __PbCallbackOwner {
    context: *mut ::core::ffi::c_void,
    drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
unsafe impl Send for __PbCallbackOwner {}
unsafe impl Sync for __PbCallbackOwner {}
impl __PbCallbackOwner {
    fn new(
        context: *mut ::core::ffi::c_void,
        drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    ) -> Self {
        Self { context, drop }
    }
    fn context(&self) -> *mut ::core::ffi::c_void {
        self.context
    }
}
impl Drop for __PbCallbackOwner {
    fn drop(&mut self) {
        if let Some(drop) = self.drop {
            unsafe { drop(self.context) }
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_0e60b44d7236d10d {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> *mut perftest_flat::CallbackReturnResource,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_15f1a05217a2ceea {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> i64>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_24a5c7966da050a9_result {
    pub v0: *mut perftest_flat::CallbackReturnResource,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_24a5c7966da050a9 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> perftest_callback_24a5c7966da050a9_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_321f7c4288706a27 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64) -> ()>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_490413898821d50d {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u8) -> u8>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_589eef1af0230719_result {
    pub v0: i64,
    pub v1: __PbBuffer,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_589eef1af0230719 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            i64,
            __PbBuffer,
        ) -> perftest_callback_589eef1af0230719_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *const perftest_callback_589eef1af0230719_result,
        ),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_5f675dc767965255_result {
    pub v0: *mut perftest_flat::CallbackReturnResource,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_5f675dc767965255 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> perftest_callback_5f675dc767965255_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_5fdca16b0dadfb25_callback_arg_0_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data
                        as *mut __PbRow_callback_5fdca16b0dadfb25_callback_arg_0_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_5fdca16b0dadfb25_callback_return_w1 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_5fdca16b0dadfb25 {
    pub run: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbRows) -> __PbRows,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_6a62e9f8e6773c45 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, f32) -> f32>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_6e20e8b97ebb3c88 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> *mut perftest_flat::CallbackReturnResource,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_73c3c177e4a73331 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u32) -> u32>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_8831734aa5203b29 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> u32>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_9067021b3e3252ad {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __Pb128) -> __Pb128>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_91e558069da45345 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __Pb128) -> __Pb128>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_b0a36b18d83434ed_callback_return_w1 {
    pub v0: *mut perftest_flat::CallbackReturnResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_b0a36b18d83434ed {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_cef6210f15ef4535 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64) -> i64>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_d58f9fdf78f56c8d {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, f64) -> f64>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_de793b4a6fb814fd {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i32) -> i32>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_df547e71faee3c03 {
    pub run: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbBuffer) -> __PbBuffer,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbBuffer),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_e51dd6cac1e6761d {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i16) -> i16>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_ea174d88bac9b485 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u64) -> u64>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_ea6b56ec3d11492d_result {
    pub v0: i32,
    pub v1: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_ea6b56ec3d11492d {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            i32,
            i64,
        ) -> perftest_callback_ea6b56ec3d11492d_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_efc3c673f144160f {
    pub run: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbBuffer) -> __PbBuffer,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbBuffer),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct perftest_callback_f5f6b40488a06bd3 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i8) -> i8>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
pub struct __PbError_result_error {
    pub v0: i64,
    pub v1: __PbBuffer,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_result_error(
    source_error: perftest_flat::CResultError,
    error_output: *mut __PbError_result_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1 = ret_w0.code;
        let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w0.message)
            .into_boxed_slice();
        let ret_w2 = Box::into_raw(ret_w2);
        let ret_w2_owner = __PbOutputOwner(ret_w2);
        let result = __PbError_result_error {
            v0: ret_w1,
            v1: __PbBuffer {
                data: ret_w2 as *const u8,
                len: ret_w2.len(),
            },
        };
        std::ptr::write(error_output, result);
        ::core::mem::forget(ret_w2_owner);
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_result_error(error: *mut __PbError_result_error) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_1 = std::mem::replace(
                &mut (*error).v1,
                __PbBuffer {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            if !(owned_1).data.is_null() {
                drop(
                    Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (owned_1).data as *mut u8,
                            (owned_1).len,
                        ),
                    ),
                );
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
pub struct __PbError_panic_error {
    pub v0: __PbBuffer,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_panic_error(
    source_error: perftest_flat::CPanicError,
    error_output: *mut __PbError_panic_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1 = ::core::convert::Into::<Vec<u8>>::into(ret_w0.message)
            .into_boxed_slice();
        let ret_w1 = Box::into_raw(ret_w1);
        let ret_w1_owner = __PbOutputOwner(ret_w1);
        let result = __PbError_panic_error {
            v0: __PbBuffer {
                data: ret_w1 as *const u8,
                len: ret_w1.len(),
            },
        };
        std::ptr::write(error_output, result);
        ::core::mem::forget(ret_w1_owner);
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_panic_error(error: *mut __PbError_panic_error) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_0 = std::mem::replace(
                &mut (*error).v0,
                __PbBuffer {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            if !(owned_0).data.is_null() {
                drop(
                    Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (owned_0).data as *mut u8,
                            (owned_0).len,
                        ),
                    ),
                );
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
pub struct __PbError_rich_error {
    pub v0: i32,
    pub v1: i32,
    pub v2: __PbBuffer,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_rich_error(
    source_error: perftest_flat::CRichError,
    error_output: *mut __PbError_rich_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1: i32 = match &(ret_w0.kind) {
            perftest_flat::SparseCode::Bottom => -2147483648i32,
            perftest_flat::SparseCode::MinusTwo => -2i32,
            perftest_flat::SparseCode::MinusOne => -1i32,
            perftest_flat::SparseCode::Zero => 0i32,
            perftest_flat::SparseCode::Seven => 7i32,
            perftest_flat::SparseCode::Eight => 8i32,
            perftest_flat::SparseCode::Top => 2147483647i32,
        };
        let (ret_w2, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0
            .message
        {
            let ret_w4 = ::core::convert::Into::<Vec<u8>>::into(ret_w3)
                .into_boxed_slice();
            let ret_w4 = Box::into_raw(ret_w4);
            let ret_w4_owner = __PbOutputOwner(ret_w4);
            let ret_w2_values = (
                __PbBuffer {
                    data: ret_w4 as *const u8,
                    len: ret_w4.len(),
                },
            );
            (
                1i32,
                ret_w2_values,
                Some(move || {
                    ::core::mem::forget(ret_w4_owner);
                }),
            )
        } else {
            (
                0i32,
                (
                    __PbBuffer {
                        data: std::ptr::null(),
                        len: 0,
                    },
                ),
                None,
            )
        };
        let result = __PbError_rich_error {
            v0: ret_w1,
            v1: ret_w2,
            v2: ret_w2_value0,
        };
        std::ptr::write(error_output, result);
        if let Some(ret_w2_action) = ret_w2_commit {
            ret_w2_action();
        }
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_rich_error(error: *mut __PbError_rich_error) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_2 = std::mem::replace(
                &mut (*error).v2,
                __PbBuffer {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            if !(owned_2).data.is_null() {
                drop(
                    Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (owned_2).data as *mut u8,
                            (owned_2).len,
                        ),
                    ),
                );
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
pub struct __PbError_drop_error {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_drop_error(
    source_error: perftest_flat::CDropError,
    error_output: *mut __PbError_drop_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1 = perftest_flat::c_panic_output_value(&ret_w0.resource);
        let ret_w2: *mut perftest_flat::OwnedResource = Box::into_raw(Box::new(ret_w1));
        let ret_w2_owner = __PbOutputOwner(ret_w2);
        let result = __PbError_drop_error { v0: ret_w2 };
        std::ptr::write(error_output, result);
        ::core::mem::forget(ret_w2_owner);
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_drop_error(error: *mut __PbError_drop_error) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_0 = std::mem::replace(&mut (*error).v0, std::ptr::null_mut());
            if !(owned_0).is_null() {
                drop(Box::from_raw((owned_0) as *mut perftest_flat::OwnedResource));
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
pub struct __PbError_resource_error {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbBuffer,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_resource_error(
    source_error: perftest_flat::CResourceError,
    error_output: *mut __PbError_resource_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1: *mut perftest_flat::OwnedResource = Box::into_raw(
            Box::new(ret_w0.resource),
        );
        let ret_w1_owner = __PbOutputOwner(ret_w1);
        let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w0.message)
            .into_boxed_slice();
        let ret_w2 = Box::into_raw(ret_w2);
        let ret_w2_owner = __PbOutputOwner(ret_w2);
        let result = __PbError_resource_error {
            v0: ret_w1,
            v1: __PbBuffer {
                data: ret_w2 as *const u8,
                len: ret_w2.len(),
            },
        };
        std::ptr::write(error_output, result);
        ::core::mem::forget(ret_w1_owner);
        ::core::mem::forget(ret_w2_owner);
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_resource_error(
    error: *mut __PbError_resource_error,
) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_0 = std::mem::replace(&mut (*error).v0, std::ptr::null_mut());
            let owned_1 = std::mem::replace(
                &mut (*error).v1,
                __PbBuffer {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            if !(owned_0).is_null() {
                drop(Box::from_raw((owned_0) as *mut perftest_flat::OwnedResource));
            }
            if !(owned_1).data.is_null() {
                drop(
                    Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (owned_1).data as *mut u8,
                            (owned_1).len,
                        ),
                    ),
                );
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_error_collection_error_ret_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_error_collection_error_ret_w1_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_error_collection_error_ret_w1,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_error_collection_error_ret_w10 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_error_collection_error_ret_w10_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_error_collection_error_ret_w10,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_error_collection_error_ret_w6 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_error_collection_error_ret_w6_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_error_collection_error_ret_w6,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct __PbError_collection_error {
    pub v0: __PbRows,
    pub v1: __PbRows,
}
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
unsafe fn __pb_report_collection_error(
    source_error: perftest_flat::CollectionError,
    error_output: *mut __PbError_collection_error,
    reported: &mut bool,
) -> Result<(), String> {
    if error_output.is_null() {
        return Ok(());
    }
    {
        let ret_w0 = source_error;
        let ret_w1_source = ret_w0.resources;
        let ret_w3 = ret_w1_source.len();
        let mut ret_w1_commits = Vec::with_capacity(ret_w3);
        let mut ret_w1: Vec<__PbRow_error_collection_error_ret_w1> = Vec::with_capacity(
            ret_w3,
        );
        for (ret_w4, ret_w2) in ret_w1_source.into_iter().enumerate() {
            let ret_w5: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(ret_w2),
            );
            let ret_w5_owner = __PbOutputOwner(ret_w5);
            ret_w1
                .push(__PbRow_error_collection_error_ret_w1 {
                    v0: ret_w5,
                });
            ret_w1_commits
                .push(move || {
                    ::core::mem::forget(ret_w5_owner);
                });
        }
        let ret_w1 = ret_w1.into_boxed_slice();
        let ret_w6_source = ret_w0.codes;
        let ret_w8 = ret_w6_source.len();
        let mut ret_w6_commits = Vec::with_capacity(ret_w8);
        let mut ret_w6: Vec<__PbRow_error_collection_error_ret_w6> = Vec::with_capacity(
            ret_w8,
        );
        for (ret_w9, ret_w7) in ret_w6_source.into_iter().enumerate() {
            let ret_w10_source = ret_w7;
            let ret_w12 = ret_w10_source.len();
            let mut ret_w10_commits = Vec::with_capacity(ret_w12);
            let mut ret_w10: Vec<__PbRow_error_collection_error_ret_w10> = Vec::with_capacity(
                ret_w12,
            );
            for (ret_w13, ret_w11) in ret_w10_source.into_iter().enumerate() {
                let ret_w14 = ret_w11;
                ret_w10
                    .push(__PbRow_error_collection_error_ret_w10 {
                        v0: ret_w14,
                    });
                ret_w10_commits.push(move || {});
            }
            let ret_w10 = ret_w10.into_boxed_slice();
            ret_w6
                .push(__PbRow_error_collection_error_ret_w6 {
                    v0: __PbRows {
                        data: if ret_w10.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w10.as_ptr().cast()
                        },
                        len: ret_w10.len(),
                    },
                });
            ret_w6_commits
                .push(move || {
                    let _ = Box::into_raw(ret_w10);
                    for commit in ret_w10_commits {
                        commit();
                    }
                });
        }
        let ret_w6 = ret_w6.into_boxed_slice();
        let result = __PbError_collection_error {
            v0: __PbRows {
                data: if ret_w1.is_empty() {
                    ::core::ptr::null()
                } else {
                    ret_w1.as_ptr().cast()
                },
                len: ret_w1.len(),
            },
            v1: __PbRows {
                data: if ret_w6.is_empty() {
                    ::core::ptr::null()
                } else {
                    ret_w6.as_ptr().cast()
                },
                len: ret_w6.len(),
            },
        };
        std::ptr::write(error_output, result);
        let _ = Box::into_raw(ret_w1);
        for commit in ret_w1_commits {
            commit();
        }
        let _ = Box::into_raw(ret_w6);
        for commit in ret_w6_commits {
            commit();
        }
        *reported = true;
        Ok(())
    }
}
/**# Safety
The pointer must be null or writable storage containing a live error produced by this library.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_drop_collection_error(
    error: *mut __PbError_collection_error,
) {
    if let Err(message) = ::prebindgen_runtime::boundary(|| {
        if !error.is_null() {
            let owned_0 = std::mem::replace(
                &mut (*error).v0,
                __PbRows {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            let owned_1 = std::mem::replace(
                &mut (*error).v1,
                __PbRows {
                    data: std::ptr::null(),
                    len: 0,
                },
            );
            if (owned_0).len != 0 {
                let rows = Box::from_raw(
                    std::ptr::slice_from_raw_parts_mut(
                        (owned_0).data as *mut __PbRow_error_collection_error_ret_w1,
                        (owned_0).len,
                    ),
                );
                for row in rows.iter() {
                    if !(row.v0).is_null() {
                        drop(
                            Box::from_raw((row.v0) as *mut perftest_flat::OwnedResource),
                        );
                    }
                }
            }
            if (owned_1).len != 0 {
                let rows = Box::from_raw(
                    std::ptr::slice_from_raw_parts_mut(
                        (owned_1).data as *mut __PbRow_error_collection_error_ret_w6,
                        (owned_1).len,
                    ),
                );
                for row in rows.iter() {
                    if (row.v0).len != 0 {
                        let rows = Box::from_raw(
                            std::ptr::slice_from_raw_parts_mut(
                                (row.v0).data
                                    as *mut __PbRow_error_collection_error_ret_w10,
                                (row.v0).len,
                            ),
                        );
                        for row in rows.iter() {}
                    }
                }
            }
        }
        Ok(())
    }) {
        panic!("C error destructor failed: {}", message);
    }
}
#[repr(C)]
pub struct perftest_api_record_new_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_api_record_new() -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_api_record_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::api_record_new();
            let ret_w0 = perftest_flat::api_record_into(source_value);
            let ret_w1 = ret_w0;
            let ret_w2 = ret_w1.count;
            let result = perftest_api_record_new_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_archive_latest_result {
    pub v0: *const perftest_flat::Summary,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_archive_latest(
    p0_w1: *const perftest_flat::Archive,
) -> *const perftest_flat::Summary {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_archive_latest_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Archive
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Archive = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::archive_latest(__prebindgen_argument_0);
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: *const perftest_flat::Summary = ret_w1
                    as *const perftest_flat::Summary;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null(),), None)
            };
            let result = perftest_archive_latest_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_archive_new_result {
    pub v0: *mut perftest_flat::Archive,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_archive_new() -> *mut perftest_flat::Archive {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_archive_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::archive_new();
            let ret_w0: *mut perftest_flat::Archive = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_archive_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_archive_store_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_archive_store(
    p0_w1: *mut perftest_flat::Archive,
    p1_w1: i32,
    p1_w4: i64,
    p1_w6: f64,
    p1_w8: *mut perftest_flat::Summary,
) {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_archive_store_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Archive
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            if !(0..2i32).contains(&p1_w1) {
                return Err("invalid selector".into());
            }
            let p1_w3 = if p1_w1 == 0i32 { Some(p1_w4) } else { None };
            let p1_w5 = if p1_w1 == 0i32 { Some(p1_w6) } else { None };
            let p1_w8_pointer = if p1_w1 == 1i32 {
                p1_w8 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if p1_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p1_w8_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if p1_w1 == 1i32 {
                    for identity in ::core::iter::once((p1_w8_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p1_w7 = if p1_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p1_w8_pointer) })
            } else {
                None
            };
            let p0_w0: &mut perftest_flat::Archive = unsafe { &mut (*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: perftest_flat::Summary = match p1_w1 {
                0i32 => {
                    let p1_w3: i64 = ::core::convert::Into::into(
                        p1_w3.expect("active value"),
                    );
                    let p1_w5: f64 = ::core::convert::Into::into(
                        p1_w5.expect("active value"),
                    );
                    let p1_w2: perftest_flat::Summary = perftest_flat::summary_new(
                        p1_w3,
                        p1_w5,
                    );
                    p1_w2
                }
                1i32 => {
                    let p1_w7: perftest_flat::Summary = *p1_w7
                        .expect("active allocation");
                    p1_w7
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::archive_store(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let result = perftest_archive_store_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct perftest_blob_value_echo_result {
    pub v0: *mut perftest_flat::BlobValue,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_blob_value_echo(
    p0_w1: *mut perftest_flat::BlobValue,
) -> *mut perftest_flat::BlobValue {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_blob_value_echo_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::BlobValue
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w0 = if true {
                Some(unsafe { Box::from_raw(p0_w1_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::BlobValue = *p0_w0.expect("active allocation");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::blob_value_echo(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::BlobValue = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_blob_value_echo_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_blob_value_new_result {
    pub v0: *mut perftest_flat::BlobValue,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_blob_value_new(
    p0_w1: i64,
    p1_w1: __PbBuffer,
) -> *mut perftest_flat::BlobValue {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_blob_value_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(unsafe { __pb_read(p1_w1)? }) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: Vec<u8> = ::core::convert::Into::into(
                p1_w0.expect("active value"),
            );
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::blob_value_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0: *mut perftest_flat::BlobValue = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_blob_value_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_c_panic_output_new_result {
    pub v0: *mut perftest_flat::CPanicOutput,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_panic_output_new() -> *mut perftest_flat::CPanicOutput {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_panic_output_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::c_panic_output_new();
            let ret_w0: *mut perftest_flat::CPanicOutput = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_c_panic_output_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_c_plain_panic_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_plain_panic() -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_c_plain_panic_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::c_plain_panic();
            let ret_w0 = source_value;
            let result = perftest_c_plain_panic_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_c_result_bool_result {
    pub v0: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_bool(
    p0_w1: u8,
    out0: *mut u8,
    error: *mut __PbError_result_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_c_result_bool_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::c_result_bool(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = u8::from(ret_w0);
            let result = perftest_c_result_bool_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_drop_panic_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_drop_panic(
    error: *mut __PbError_result_error,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_drop_panic_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::c_result_drop_panic();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::c_panic_output_value(&ret_w0);
            let ret_w3: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(ret_w2),
            );
            let ret_w3_owner = __PbOutputOwner(ret_w3);
            let result = perftest_c_result_drop_panic_result {
                v0: ret_w3,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w3_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw(
                                (value.v0) as *mut perftest_flat::OwnedResource,
                            ),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_error_drop_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_error_drop(
    error: *mut __PbError_drop_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_error_drop_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::c_result_error_drop();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_drop_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_c_result_error_drop_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_drop_error(
                            <perftest_flat::CDropError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_late_niche_result {
    pub v0: i64,
    pub v1: u32,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_late_niche(
    p0_w1: u8,
    out0: *mut i64,
    error: *mut __PbError_result_error,
) -> u32 {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_late_niche_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::c_result_late_niche(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::c_result_pair_into(ret_w0);
            let ret_w3 = ret_w2;
            let ret_w4 = ret_w3.number;
            let ret_w5 = (ret_w3.character) as u32;
            let result = perftest_c_result_late_niche_result {
                v0: ret_w4,
                v1: ret_w5,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            value.v1
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            1114112
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_optional_handle_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_optional_handle(
    p0_w1: i32,
    out0: *mut *mut perftest_flat::OwnedResource,
    error: *mut __PbError_result_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_optional_handle_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::c_result_optional_handle(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w3),
                );
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                let ret_w2_values = (ret_w4,);
                (
                    1i32,
                    ret_w2_values,
                    Some(move || {
                        ::core::mem::forget(ret_w4_owner);
                    }),
                )
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_c_result_optional_handle_result {
                v0: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw(
                                (value.v0) as *mut perftest_flat::OwnedResource,
                            ),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_panic_error_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_panic_error(
    error: *mut __PbError_panic_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_panic_error_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::c_result_panic_error();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_panic_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_c_result_panic_error_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_panic_error(
                            <perftest_flat::CPanicError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_resource_error_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_resource_error(
    p0_w1: u8,
    error: *mut __PbError_resource_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_resource_error_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::c_result_resource_error(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_resource_error(
                                ret_w1,
                                error,
                                &mut error_reported,
                            )
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_c_result_resource_error_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_resource_error(
                            <perftest_flat::CResourceError as From<
                                String,
                            >>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_rich_error_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_rich_error(
    p0_w1: u8,
    error: *mut __PbError_rich_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_rich_error_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::c_result_rich_error(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_rich_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_c_result_rich_error_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_rich_error(
                            <perftest_flat::CRichError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_c_result_structured_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_c_result_structured(
    p0_w1: __PbBuffer,
    p1_w1: i32,
    out0: *mut i64,
    error: *mut __PbError_result_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_c_result_structured_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let __prebindgen_argument_0 = ::core::convert::AsRef::<
                str,
            >::as_ref(p0_w0.as_ref().expect("active value"));
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::c_result_structured(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = perftest_c_result_structured_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_callback_return_bool_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_bool(
    p0_callback: perftest_callback_490413898821d50d,
) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_bool_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: bool| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = u8::from(callback_source_0);
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1 != 0)
                    } else {
                        None
                    };
                    let callback_return_w0: bool = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_490413898821d50d",
                            "(bool)->bool",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_bool(
                __prebindgen_argument_0,
            );
            let ret_w0 = u8::from(source_value);
            let result = perftest_callback_return_bool_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_borrowed_pair_catch_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_borrowed_pair_catch(
    p0_callback: perftest_callback_5f675dc767965255,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_borrowed_pair_catch_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w2 = raw_result.v0;
                    let callback_return_w4 = raw_result.v1;
                    {
                        let callback_return_w2_pointer = if true {
                            callback_return_w2
                                as *mut perftest_flat::CallbackReturnResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(
                                callback_return_w2_pointer,
                            )?;
                        }
                        let callback_return_w3 = if true {
                            Some(
                                char::from_u32(callback_return_w4)
                                    .ok_or("invalid Unicode scalar")?,
                            )
                        } else {
                            None
                        };
                        let callback_return_w1: &mut perftest_flat::CallbackReturnResource = unsafe {
                            &mut (*callback_return_w2_pointer)
                        };
                        let callback_return_w3: char = ::core::convert::Into::into(
                            callback_return_w3.expect("active value"),
                        );
                        let callback_return_w0: perftest_flat::CallbackReturnBorrowedPair = perftest_flat::callback_return_borrowed_pair_new(
                            callback_return_w1,
                            callback_return_w3,
                        );
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_5f675dc767965255",
                            "()->perftest_flat :: CallbackReturnBorrowedPair",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_borrowed_pair_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_borrowed_pair_catch_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_bytes_result {
    pub v0: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_bytes(
    p0_callback: perftest_callback_df547e71faee3c03,
) -> __PbBuffer {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_bytes_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: Vec<u8>| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = ::core::convert::Into::<
                        Vec<u8>,
                    >::into(callback_source_0)
                        .into_boxed_slice();
                    let callback_arg_0_w0 = Box::into_raw(callback_arg_0_w0);
                    let callback_arg_0_w0_owner = __PbOutputOwner(callback_arg_0_w0);
                    let callback_argument_0 = __PbBuffer {
                        data: callback_arg_0_w0 as *const u8,
                        len: callback_arg_0_w0.len(),
                    };
                    {
                        ::core::mem::forget(callback_arg_0_w0_owner);
                    }
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(unsafe { __pb_read(callback_return_w1)? })
                    } else {
                        None
                    };
                    let callback_return_w0: Vec<u8> = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_df547e71faee3c03",
                            "(Vec < u8 >)->Vec < u8 >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_bytes(
                __prebindgen_argument_0,
            );
            let ret_w0 = ::core::convert::Into::<Vec<u8>>::into(source_value)
                .into_boxed_slice();
            let ret_w0 = Box::into_raw(ret_w0);
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_callback_return_bytes_result {
                v0: __PbBuffer {
                    data: ret_w0 as *const u8,
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_c_result_result {
    pub v0: u32,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_c_result(
    p0_callback: perftest_callback_73c3c177e4a73331,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> u32 {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_c_result_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: char| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = (callback_source_0) as u32;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            char::from_u32(callback_return_w1)
                                .ok_or("invalid Unicode scalar")?,
                        )
                    } else {
                        None
                    };
                    let callback_return_w0: char = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_73c3c177e4a73331",
                            "(char)->char",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_c_result(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = (ret_w0) as u32;
            let result = perftest_callback_return_c_result_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            1114112
        }
    }
}
#[repr(C)]
pub struct perftest_callback_return_catch_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_catch(
    p0_callback: perftest_callback_cef6210f15ef4535,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_catch_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_cef6210f15ef4535",
                            "(i64)->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_catch_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_char_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_char(
    p0_callback: perftest_callback_73c3c177e4a73331,
) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_char_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: char| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = (callback_source_0) as u32;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            char::from_u32(callback_return_w1)
                                .ok_or("invalid Unicode scalar")?,
                        )
                    } else {
                        None
                    };
                    let callback_return_w0: char = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_73c3c177e4a73331",
                            "(char)->char",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_char(
                __prebindgen_argument_0,
            );
            let ret_w0 = (source_value) as u32;
            let result = perftest_callback_return_char_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_char_catch_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_char_catch(
    p0_callback: perftest_callback_8831734aa5203b29,
) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_char_catch_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w1 = raw_result;
                    {
                        let callback_return_w0 = if true {
                            Some(
                                char::from_u32(callback_return_w1)
                                    .ok_or("invalid Unicode scalar")?,
                            )
                        } else {
                            None
                        };
                        let callback_return_w0: char = ::core::convert::Into::into(
                            callback_return_w0.expect("active value"),
                        );
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_8831734aa5203b29",
                            "()->char",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_char_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = (source_value) as u32;
            let result = perftest_callback_return_char_catch_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_enum_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_enum(
    p0_callback: perftest_callback_de793b4a6fb814fd,
) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_enum_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |
                callback_source_0: perftest_flat::CallbackReturnChoice|
            {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0: i32 = match &(callback_source_0) {
                        perftest_flat::CallbackReturnChoice::First => 0i32,
                        perftest_flat::CallbackReturnChoice::Second => 1i32,
                    };
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            match callback_return_w1 {
                                0i32 => perftest_flat::CallbackReturnChoice::First,
                                1i32 => perftest_flat::CallbackReturnChoice::Second,
                                _ => return Err("invalid enum discriminant".into()),
                            },
                        )
                    } else {
                        None
                    };
                    let callback_return_w0 = callback_return_w0.expect("active enum");
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_de793b4a6fb814fd",
                            "(perftest_flat :: CallbackReturnChoice)->perftest_flat :: CallbackReturnChoice",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_enum(
                __prebindgen_argument_0,
            );
            let ret_w0 = u8::from(source_value);
            let result = perftest_callback_return_enum_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_f32_result {
    pub v0: f32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_f32(
    p0_callback: perftest_callback_6a62e9f8e6773c45,
) -> f32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_f32_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: f32| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: f32 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_6a62e9f8e6773c45",
                            "(f32)->f32",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_f32(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_f32_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_float_result {
    pub v0: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_float(
    p0_callback: perftest_callback_d58f9fdf78f56c8d,
) -> f64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_float_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: f64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: f64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_d58f9fdf78f56c8d",
                            "(f64)->f64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_float(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_float_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_handler_call_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_handler_call(
    p0_w1: *const perftest_flat::CallbackReturnHandler,
    p1_w1: i64,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_handler_call_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::CallbackReturnHandler
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = ();
            let p0_w0: &perftest_flat::CallbackReturnHandler = unsafe {
                &(*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::callback_return_handler_call(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_handler_call_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_return_handler_many_p0_w1 {
    pub v0: perftest_callback_cef6210f15ef4535,
}
#[repr(C)]
pub struct perftest_callback_return_handler_many_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_handler_many(p0_w1: __PbRows) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_handler_many_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_callback_return_handler_many_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe {
                        (*((p0_w1.data
                            as *const __PbRow_callback_return_handler_many_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w6_run = if true {
                        Some((p0_w6).run.ok_or("null callback function")?)
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            let p0_w6_owner = if true {
                                Some(__PbCallbackOwner::new((p0_w6).context, (p0_w6).drop))
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::CallbackReturnHandler,
                                String,
                            > {
                                let p0_w6_run = p0_w6_run
                                    .expect("validated active callback");
                                let p0_w6_owner = p0_w6_owner
                                    .expect("validated active callback");
                                let p0_w5 = move |callback_source_0: i64| {
                                    let result = ::prebindgen_runtime::boundary(|| {
                                        let callback_arg_0_w0 = callback_source_0;
                                        let callback_argument_0 = callback_arg_0_w0;
                                        {}
                                        let raw_result = unsafe {
                                            p0_w6_run(p0_w6_owner.context(), callback_argument_0)
                                        };
                                        let callback_return_w1 = raw_result;
                                        let callback_return_w0 = if true {
                                            Some(callback_return_w1)
                                        } else {
                                            None
                                        };
                                        let callback_return_w0: i64 = ::core::convert::Into::into(
                                            callback_return_w0.expect("active value"),
                                        );
                                        let callback_return_value = callback_return_w0;
                                        Ok(callback_return_value)
                                    });
                                    result
                                        .unwrap_or_else(|error| {
                                            ::prebindgen_runtime::report_callback_error(
                                                "perftest_callback_cef6210f15ef4535",
                                                "(i64)->i64",
                                                &error,
                                            );
                                            panic!("callback failed: {}", error)
                                        })
                                };
                                let p0_w4: perftest_flat::CallbackReturnHandler = perftest_flat::callback_return_handler_wrap(
                                    p0_w5,
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::CallbackReturnHandler> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::callback_return_handler_many(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_handler_many_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_handler_new_result {
    pub v0: *mut perftest_flat::CallbackReturnHandler,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_handler_new(
    p0_callback: perftest_callback_cef6210f15ef4535,
    p1_w1: u8,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::CallbackReturnHandler {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_handler_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_cef6210f15ef4535",
                            "(i64)->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::callback_return_handler_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::CallbackReturnHandler = Box::into_raw(
                Box::new(ret_w0),
            );
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let result = perftest_callback_return_handler_new_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw(
                                (value.v0) as *mut perftest_flat::CallbackReturnHandler,
                            ),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_callback_return_handler_value_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_handler_value(
    p0_w2: perftest_callback_cef6210f15ef4535,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_handler_value_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w2_run = if true {
                Some(p0_w2.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w2_owner = if true {
                Some(__PbCallbackOwner::new(p0_w2.context, p0_w2.drop))
            } else {
                None
            };
            let p0_w2_run = p0_w2_run.expect("validated active callback");
            let p0_w2_owner = p0_w2_owner.expect("validated active callback");
            let p0_w1 = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_w2_run(p0_w2_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_cef6210f15ef4535",
                            "(i64)->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let p0_w0: perftest_flat::CallbackReturnHandler = perftest_flat::callback_return_handler_wrap(
                p0_w1,
            );
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::callback_return_handler_value(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_handler_value_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_i16_result {
    pub v0: i16,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_i16(
    p0_callback: perftest_callback_e51dd6cac1e6761d,
) -> i16 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_i16_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i16| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i16 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_e51dd6cac1e6761d",
                            "(i16)->i16",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_i16(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_i16_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_i64_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_i64(
    p0_callback: perftest_callback_cef6210f15ef4535,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_i64_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_cef6210f15ef4535",
                            "(i64)->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_i64(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_i64_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_i8_result {
    pub v0: i8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_i8(
    p0_callback: perftest_callback_f5f6b40488a06bd3,
) -> i8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_i8_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i8| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i8 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_f5f6b40488a06bd3",
                            "(i8)->i8",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_i8(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_i8_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_join_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_join(
    p0_w1: *mut perftest_flat::CallbackReturnJob,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_join_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::CallbackReturnJob
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w0 = if true {
                Some(unsafe { Box::from_raw(p0_w1_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::CallbackReturnJob = *p0_w0
                .expect("active allocation");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::callback_return_join(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_join_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_optional_result {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_optional(
    p0_callback: perftest_callback_ea6b56ec3d11492d,
    p1_w1: u8,
    out1: *mut i64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: Option<i64>| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let (
                        callback_arg_0_w0,
                        (callback_arg_0_w0_value0,),
                        callback_arg_0_w0_commit,
                    ) = if let Some(callback_arg_0_w1) = callback_source_0 {
                        let callback_arg_0_w2 = callback_arg_0_w1;
                        let callback_arg_0_w0_values = (callback_arg_0_w2,);
                        (1i32, callback_arg_0_w0_values, Some(move || {}))
                    } else {
                        (0i32, (0,), None)
                    };
                    let callback_argument_0 = callback_arg_0_w0;
                    let callback_argument_1 = callback_arg_0_w0_value0;
                    {
                        if let Some(callback_arg_0_w0_action) = callback_arg_0_w0_commit {
                            callback_arg_0_w0_action();
                        }
                    }
                    let raw_result = unsafe {
                        p0_callback_run(
                            p0_callback_owner.context(),
                            callback_argument_0,
                            callback_argument_1,
                        )
                    };
                    let callback_return_w1 = raw_result.v0;
                    let callback_return_w3 = raw_result.v1;
                    if callback_return_w1 != 0 && callback_return_w1 != 1 {
                        return Err("invalid option presence".into());
                    }
                    let callback_return_w2 = if callback_return_w1 == 1i32 {
                        Some(callback_return_w3)
                    } else {
                        None
                    };
                    let callback_return_w0 = if callback_return_w1 == 1i32 {
                        let callback_return_w2: i64 = ::core::convert::Into::into(
                            callback_return_w2.expect("active value"),
                        );
                        Some(callback_return_w2)
                    } else {
                        None
                    };
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_ea6b56ec3d11492d",
                            "(Option < i64 >)->Option < i64 >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::callback_return_optional(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_callback_return_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_owned_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_owned(
    p0_callback: perftest_callback_0e60b44d7236d10d,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_owned_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_owner = if callback_return_w1.is_null() {
                        None
                    } else {
                        ::prebindgen_runtime::validate_pointer(callback_return_w1)?;
                        Some(unsafe {
                            Box::<
                                perftest_flat::CallbackReturnResource,
                            >::from_raw(callback_return_w1)
                        })
                    };
                    {
                        let callback_return_w1_pointer = if true {
                            callback_return_w1
                                as *mut perftest_flat::CallbackReturnResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(
                                callback_return_w1_pointer,
                            )?;
                        }
                        let _ = callback_return_owner.map(Box::into_raw);
                        let callback_return_w0 = if true {
                            Some(unsafe { Box::from_raw(callback_return_w1_pointer) })
                        } else {
                            None
                        };
                        let callback_return_w0: perftest_flat::CallbackReturnResource = *callback_return_w0
                            .expect("active allocation");
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_0e60b44d7236d10d",
                            "()->perftest_flat :: CallbackReturnResource",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_owned(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_owned_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_owned_optional_result {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_owned_optional(
    p0_callback: perftest_callback_6e20e8b97ebb3c88,
    out1: *mut i64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_owned_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w3 = raw_result;
                    let callback_return_owner = if callback_return_w3.is_null() {
                        None
                    } else {
                        ::prebindgen_runtime::validate_pointer(callback_return_w3)?;
                        Some(unsafe {
                            Box::<
                                perftest_flat::CallbackReturnResource,
                            >::from_raw(callback_return_w3)
                        })
                    };
                    {
                        let callback_return_w1: i32 = i32::from(
                            (callback_return_w3 as usize) != 0,
                        );
                        let callback_return_w3_pointer = if callback_return_w1 == 1i32 {
                            callback_return_w3
                                as *mut perftest_flat::CallbackReturnResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if callback_return_w1 == 1i32 {
                            ::prebindgen_runtime::validate_pointer(
                                callback_return_w3_pointer,
                            )?;
                        }
                        let _ = callback_return_owner.map(Box::into_raw);
                        let callback_return_w2 = if callback_return_w1 == 1i32 {
                            Some(unsafe { Box::from_raw(callback_return_w3_pointer) })
                        } else {
                            None
                        };
                        let callback_return_w0 = if callback_return_w1 == 1i32 {
                            let callback_return_w2: perftest_flat::CallbackReturnResource = *callback_return_w2
                                .expect("active allocation");
                            Some(callback_return_w2)
                        } else {
                            None
                        };
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_6e20e8b97ebb3c88",
                            "()->Option < perftest_flat :: CallbackReturnResource >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_owned_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_callback_return_owned_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_owned_vector_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_owned_vector(
    p0_callback: perftest_callback_b0a36b18d83434ed,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_owned_vector_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w1 = raw_result;
                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::default();
                    ::prebindgen_runtime::validate_slice(
                        callback_return_w1.data
                            as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1,
                        callback_return_w1.len,
                    )?;
                    for index in 0..callback_return_w1.len {
                        let callback_return_w5 = unsafe {
                            (*((callback_return_w1.data
                                as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1)
                                .add(index)))
                                .v0
                        };
                        unsafe {
                            callback_return_owners
                                .receive(
                                    callback_return_w5
                                        as *mut perftest_flat::CallbackReturnResource,
                                );
                        }
                    }
                    callback_return_owners.check()?;
                    {
                        let mut callback_return_w1_identities_0: Vec<usize> = Vec::new();
                        let mut callback_return_w1_staged = Vec::new();
                        if true {
                            let callback_return_w2 = callback_return_w1.len;
                            ::prebindgen_runtime::validate_slice(
                                callback_return_w1.data
                                    as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1,
                                callback_return_w2,
                            )?;
                            for callback_return_w3 in 0..callback_return_w2 {
                                let callback_return_w5 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1)
                                        .add(callback_return_w3)))
                                        .v0
                                };
                                let callback_return_w5_pointer = if true {
                                    (callback_return_w5)
                                        as *mut perftest_flat::CallbackReturnResource
                                } else {
                                    ::core::ptr::null_mut()
                                };
                                if true {
                                    ::prebindgen_runtime::validate_pointer(
                                        callback_return_w5_pointer,
                                    )?;
                                }
                                if true {
                                    callback_return_w1_identities_0
                                        .push((callback_return_w5_pointer.addr()) as usize);
                                }
                                callback_return_w1_staged
                                    .push(move || {
                                        let callback_return_w4 = if true {
                                            Some(unsafe { Box::from_raw(callback_return_w5_pointer) })
                                        } else {
                                            None
                                        };
                                        move || -> ::core::result::Result<
                                            perftest_flat::CallbackReturnResource,
                                            String,
                                        > {
                                            let callback_return_w4: perftest_flat::CallbackReturnResource = *callback_return_w4
                                                .expect("active allocation");
                                            let callback_return_w0 = callback_return_w4;
                                            Ok(callback_return_w0)
                                        }
                                    });
                            }
                        }
                        let _: () = {
                            let mut __prebindgen_parameter_resources: Vec<
                                (usize, bool),
                            > = Vec::new();
                            let _: () = {
                                if true {
                                    for identity in callback_return_w1_identities_0
                                        .iter()
                                        .copied()
                                    {
                                        __prebindgen_parameter_resources.push((identity, true));
                                    }
                                }
                                __prebindgen_parameter_resources
                                    .sort_unstable_by_key(|entry| entry.0);
                                if __prebindgen_parameter_resources
                                    .windows(2)
                                    .any(|pair| {
                                        pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1)
                                    })
                                {
                                    return Err("conflicting input aliases".into());
                                }
                            };
                        };
                        callback_return_owners.commit();
                        let callback_return_w1_acquired: Vec<_> = callback_return_w1_staged
                            .into_iter()
                            .map(|acquire| acquire())
                            .collect();
                        let callback_return_w0: Vec<
                            perftest_flat::CallbackReturnResource,
                        > = callback_return_w1_acquired
                            .into_iter()
                            .map(|construct| construct())
                            .collect::<::core::result::Result<_, String>>()?;
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_b0a36b18d83434ed",
                            "()->Vec < perftest_flat :: CallbackReturnResource >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_owned_vector(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_owned_vector_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_owned_vector_catch_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_owned_vector_catch(
    p0_callback: perftest_callback_b0a36b18d83434ed,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_owned_vector_catch_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w1 = raw_result;
                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::default();
                    ::prebindgen_runtime::validate_slice(
                        callback_return_w1.data
                            as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1,
                        callback_return_w1.len,
                    )?;
                    for index in 0..callback_return_w1.len {
                        let callback_return_w5 = unsafe {
                            (*((callback_return_w1.data
                                as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1)
                                .add(index)))
                                .v0
                        };
                        unsafe {
                            callback_return_owners
                                .receive(
                                    callback_return_w5
                                        as *mut perftest_flat::CallbackReturnResource,
                                );
                        }
                    }
                    callback_return_owners.check()?;
                    {
                        let mut callback_return_w1_identities_0: Vec<usize> = Vec::new();
                        let mut callback_return_w1_staged = Vec::new();
                        if true {
                            let callback_return_w2 = callback_return_w1.len;
                            ::prebindgen_runtime::validate_slice(
                                callback_return_w1.data
                                    as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1,
                                callback_return_w2,
                            )?;
                            for callback_return_w3 in 0..callback_return_w2 {
                                let callback_return_w5 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_b0a36b18d83434ed_callback_return_w1)
                                        .add(callback_return_w3)))
                                        .v0
                                };
                                let callback_return_w5_pointer = if true {
                                    (callback_return_w5)
                                        as *mut perftest_flat::CallbackReturnResource
                                } else {
                                    ::core::ptr::null_mut()
                                };
                                if true {
                                    ::prebindgen_runtime::validate_pointer(
                                        callback_return_w5_pointer,
                                    )?;
                                }
                                if true {
                                    callback_return_w1_identities_0
                                        .push((callback_return_w5_pointer.addr()) as usize);
                                }
                                callback_return_w1_staged
                                    .push(move || {
                                        let callback_return_w4 = if true {
                                            Some(unsafe { Box::from_raw(callback_return_w5_pointer) })
                                        } else {
                                            None
                                        };
                                        move || -> ::core::result::Result<
                                            perftest_flat::CallbackReturnResource,
                                            String,
                                        > {
                                            let callback_return_w4: perftest_flat::CallbackReturnResource = *callback_return_w4
                                                .expect("active allocation");
                                            let callback_return_w0 = callback_return_w4;
                                            Ok(callback_return_w0)
                                        }
                                    });
                            }
                        }
                        let _: () = {
                            let mut __prebindgen_parameter_resources: Vec<
                                (usize, bool),
                            > = Vec::new();
                            let _: () = {
                                if true {
                                    for identity in callback_return_w1_identities_0
                                        .iter()
                                        .copied()
                                    {
                                        __prebindgen_parameter_resources.push((identity, true));
                                    }
                                }
                                __prebindgen_parameter_resources
                                    .sort_unstable_by_key(|entry| entry.0);
                                if __prebindgen_parameter_resources
                                    .windows(2)
                                    .any(|pair| {
                                        pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1)
                                    })
                                {
                                    return Err("conflicting input aliases".into());
                                }
                            };
                        };
                        callback_return_owners.commit();
                        let callback_return_w1_acquired: Vec<_> = callback_return_w1_staged
                            .into_iter()
                            .map(|acquire| acquire())
                            .collect();
                        let callback_return_w0: Vec<
                            perftest_flat::CallbackReturnResource,
                        > = callback_return_w1_acquired
                            .into_iter()
                            .map(|construct| construct())
                            .collect::<::core::result::Result<_, String>>()?;
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_b0a36b18d83434ed",
                            "()->Vec < perftest_flat :: CallbackReturnResource >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_owned_vector_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_owned_vector_catch_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_pair_catch_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_pair_catch(
    p0_callback: perftest_callback_24a5c7966da050a9,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_pair_catch_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w2 = raw_result.v0;
                    let callback_return_w4 = raw_result.v1;
                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::default();
                    unsafe {
                        callback_return_owners
                            .receive(
                                callback_return_w2
                                    as *mut perftest_flat::CallbackReturnResource,
                            );
                    }
                    callback_return_owners.check()?;
                    {
                        let callback_return_w2_pointer = if true {
                            callback_return_w2
                                as *mut perftest_flat::CallbackReturnResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(
                                callback_return_w2_pointer,
                            )?;
                        }
                        let callback_return_w3 = if true {
                            Some(
                                char::from_u32(callback_return_w4)
                                    .ok_or("invalid Unicode scalar")?,
                            )
                        } else {
                            None
                        };
                        callback_return_owners.commit();
                        let callback_return_w1 = if true {
                            Some(unsafe { Box::from_raw(callback_return_w2_pointer) })
                        } else {
                            None
                        };
                        let callback_return_w1: perftest_flat::CallbackReturnResource = *callback_return_w1
                            .expect("active allocation");
                        let callback_return_w3: char = ::core::convert::Into::into(
                            callback_return_w3.expect("active value"),
                        );
                        let callback_return_w0: perftest_flat::CallbackReturnPair = perftest_flat::callback_return_pair_new(
                            callback_return_w1,
                            callback_return_w3,
                        );
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_24a5c7966da050a9",
                            "()->perftest_flat :: CallbackReturnPair",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_pair_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_pair_catch_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_record_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_record(
    p0_callback: perftest_callback_589eef1af0230719,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_record_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |
                callback_source_0: perftest_flat::CallbackReturnRecord|
            {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = perftest_flat::callback_return_record_into(
                        callback_source_0,
                    );
                    let callback_arg_0_w1 = callback_arg_0_w0;
                    let callback_arg_0_w2 = callback_arg_0_w1.number;
                    let callback_arg_0_w3 = ::core::convert::Into::<
                        Vec<u8>,
                    >::into(callback_arg_0_w1.label)
                        .into_boxed_slice();
                    let callback_arg_0_w3 = Box::into_raw(callback_arg_0_w3);
                    let callback_arg_0_w3_owner = __PbOutputOwner(callback_arg_0_w3);
                    let callback_argument_0 = callback_arg_0_w2;
                    let callback_argument_1 = __PbBuffer {
                        data: callback_arg_0_w3 as *const u8,
                        len: callback_arg_0_w3.len(),
                    };
                    {
                        ::core::mem::forget(callback_arg_0_w3_owner);
                    }
                    let raw_result = unsafe {
                        p0_callback_run(
                            p0_callback_owner.context(),
                            callback_argument_0,
                            callback_argument_1,
                        )
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w2 = raw_result.v0;
                    let callback_return_w4 = raw_result.v1;
                    let callback_return_w1 = if true {
                        Some(callback_return_w2)
                    } else {
                        None
                    };
                    let callback_return_w3 = if true {
                        Some(
                            std::str::from_utf8(unsafe {
                                    __pb_read(callback_return_w4)?
                                })
                                .map_err(|_| "invalid UTF-8")?,
                        )
                    } else {
                        None
                    };
                    let callback_return_w1: i64 = ::core::convert::Into::into(
                        callback_return_w1.expect("active value"),
                    );
                    let callback_return_w3: String = ::core::convert::Into::into(
                        callback_return_w3.expect("active value"),
                    );
                    let callback_return_w0: perftest_flat::CallbackReturnRecord = perftest_flat::callback_return_record_new(
                        callback_return_w1,
                        callback_return_w3,
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_589eef1af0230719",
                            "(perftest_flat :: CallbackReturnRecord)->perftest_flat :: CallbackReturnRecord",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_record(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_record_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_resource_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_resource_id(
    p0_w1: *const perftest_flat::CallbackReturnResource,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_resource_id_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::CallbackReturnResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::CallbackReturnResource = unsafe {
                &(*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::callback_return_resource_id(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_resource_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_resource_live_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_resource_live() -> u64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_resource_live_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::callback_return_resource_live();
            let ret_w0 = source_value;
            let result = perftest_callback_return_resource_live_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_resource_new_result {
    pub v0: *mut perftest_flat::CallbackReturnResource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_resource_new(
    p0_w1: i64,
) -> *mut perftest_flat::CallbackReturnResource {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_resource_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::callback_return_resource_new(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::CallbackReturnResource = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_callback_return_resource_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_spawn_result {
    pub v0: *mut perftest_flat::CallbackReturnJob,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_spawn(
    p0_callback: perftest_callback_cef6210f15ef4535,
) -> *mut perftest_flat::CallbackReturnJob {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_spawn_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: i64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_cef6210f15ef4535",
                            "(i64)->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_spawn(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::CallbackReturnJob = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_callback_return_spawn_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_text_result {
    pub v0: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_text(
    p0_callback: perftest_callback_efc3c673f144160f,
) -> __PbBuffer {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_text_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: String| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = ::core::convert::Into::<
                        Vec<u8>,
                    >::into(callback_source_0)
                        .into_boxed_slice();
                    let callback_arg_0_w0 = Box::into_raw(callback_arg_0_w0);
                    let callback_arg_0_w0_owner = __PbOutputOwner(callback_arg_0_w0);
                    let callback_argument_0 = __PbBuffer {
                        data: callback_arg_0_w0 as *const u8,
                        len: callback_arg_0_w0.len(),
                    };
                    {
                        ::core::mem::forget(callback_arg_0_w0_owner);
                    }
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            std::str::from_utf8(unsafe {
                                    __pb_read(callback_return_w1)?
                                })
                                .map_err(|_| "invalid UTF-8")?,
                        )
                    } else {
                        None
                    };
                    let callback_return_w0: String = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_efc3c673f144160f",
                            "(String)->String",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_text(
                __prebindgen_argument_0,
            );
            let ret_w0 = ::core::convert::Into::<Vec<u8>>::into(source_value)
                .into_boxed_slice();
            let ret_w0 = Box::into_raw(ret_w0);
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_callback_return_text_result {
                v0: __PbBuffer {
                    data: ret_w0 as *const u8,
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_u128_result {
    pub v0: __Pb128,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_u128(
    p0_callback: perftest_callback_91e558069da45345,
) -> __Pb128 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_u128_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: u128| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = {
                        let bits = (callback_source_0) as u128;
                        __Pb128 {
                            hi: (bits >> 64) as u64,
                            lo: bits as u64,
                        }
                    };
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            ((u128::from(callback_return_w1.hi) << 64)
                                | u128::from(callback_return_w1.lo)) as u128,
                        )
                    } else {
                        None
                    };
                    let callback_return_w0: u128 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_91e558069da45345",
                            "(u128)->u128",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_u128(
                __prebindgen_argument_0,
            );
            let ret_w0 = {
                let bits = (source_value) as u128;
                __Pb128 {
                    hi: (bits >> 64) as u64,
                    lo: bits as u64,
                }
            };
            let result = perftest_callback_return_u128_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_unit_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_unit(
    p0_callback: perftest_callback_321f7c4288706a27,
) {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_unit_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    Ok(())
                });
                if let Err(error) = result {
                    ::prebindgen_runtime::report_callback_error(
                        "perftest_callback_321f7c4288706a27",
                        "(i64)->()",
                        &error,
                    );
                }
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_unit(
                __prebindgen_argument_0,
            );
            let result = perftest_callback_return_unit_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct perftest_callback_return_unsigned_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_unsigned(
    p0_callback: perftest_callback_ea174d88bac9b485,
) -> u64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_unsigned_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: u64| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = callback_source_0;
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(callback_return_w1)
                    } else {
                        None
                    };
                    let callback_return_w0: u64 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_ea174d88bac9b485",
                            "(u64)->u64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_unsigned(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_unsigned_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_return_vector_ret_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_vector_ret_w0_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_callback_return_vector_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_callback_return_vector_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_vector(
    p0_callback: perftest_callback_5fdca16b0dadfb25,
) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_vector_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: Vec<i64>| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0_source = callback_source_0;
                    let callback_arg_0_w2 = callback_arg_0_w0_source.len();
                    let mut callback_arg_0_w0_commits = Vec::with_capacity(
                        callback_arg_0_w2,
                    );
                    let mut callback_arg_0_w0: Vec<
                        __PbRow_callback_5fdca16b0dadfb25_callback_arg_0_w0,
                    > = Vec::with_capacity(callback_arg_0_w2);
                    for (callback_arg_0_w3, callback_arg_0_w1) in callback_arg_0_w0_source
                        .into_iter()
                        .enumerate()
                    {
                        let callback_arg_0_w4 = callback_arg_0_w1;
                        callback_arg_0_w0
                            .push(__PbRow_callback_5fdca16b0dadfb25_callback_arg_0_w0 {
                                v0: callback_arg_0_w4,
                            });
                        callback_arg_0_w0_commits.push(move || {});
                    }
                    let callback_arg_0_w0 = callback_arg_0_w0.into_boxed_slice();
                    let callback_argument_0 = __PbRows {
                        data: if callback_arg_0_w0.is_empty() {
                            ::core::ptr::null()
                        } else {
                            callback_arg_0_w0.as_ptr().cast()
                        },
                        len: callback_arg_0_w0.len(),
                    };
                    {
                        let _ = Box::into_raw(callback_arg_0_w0);
                        for commit in callback_arg_0_w0_commits {
                            commit();
                        }
                    }
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_result = __PbCallbackResult {
                        value: raw_result,
                        context: p0_callback_owner.context(),
                        release: p0_callback_release_result,
                    };
                    let raw_result = callback_result.value;
                    let callback_return_w1 = raw_result;
                    let mut callback_return_w1_staged = Vec::new();
                    if true {
                        let callback_return_w2 = callback_return_w1.len;
                        ::prebindgen_runtime::validate_slice(
                            callback_return_w1.data
                                as *const __PbRow_callback_5fdca16b0dadfb25_callback_return_w1,
                            callback_return_w2,
                        )?;
                        for callback_return_w3 in 0..callback_return_w2 {
                            let callback_return_w5 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_5fdca16b0dadfb25_callback_return_w1)
                                    .add(callback_return_w3)))
                                    .v0
                            };
                            let callback_return_w4 = if true {
                                Some(callback_return_w5)
                            } else {
                                None
                            };
                            callback_return_w1_staged
                                .push(move || {
                                    move || -> ::core::result::Result<i64, String> {
                                        let callback_return_w4: i64 = ::core::convert::Into::into(
                                            callback_return_w4.expect("active value"),
                                        );
                                        let callback_return_w0 = callback_return_w4;
                                        Ok(callback_return_w0)
                                    }
                                });
                        }
                    }
                    let callback_return_w1_acquired: Vec<_> = callback_return_w1_staged
                        .into_iter()
                        .map(|acquire| acquire())
                        .collect();
                    let callback_return_w0: Vec<i64> = callback_return_w1_acquired
                        .into_iter()
                        .map(|construct| construct())
                        .collect::<::core::result::Result<_, String>>()?;
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_5fdca16b0dadfb25",
                            "(Vec < i64 >)->Vec < i64 >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_vector(
                __prebindgen_argument_0,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_callback_return_vector_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                ret_w0
                    .push(__PbRow_callback_return_vector_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_callback_return_vector_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_wide_result {
    pub v0: __Pb128,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_wide(
    p0_callback: perftest_callback_9067021b3e3252ad,
) -> __Pb128 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_wide_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move |callback_source_0: i128| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = {
                        let bits = (callback_source_0) as u128;
                        __Pb128 {
                            hi: (bits >> 64) as u64,
                            lo: bits as u64,
                        }
                    };
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
                    let callback_return_w0 = if true {
                        Some(
                            ((u128::from(callback_return_w1.hi) << 64)
                                | u128::from(callback_return_w1.lo)) as i128,
                        )
                    } else {
                        None
                    };
                    let callback_return_w0: i128 = ::core::convert::Into::into(
                        callback_return_w0.expect("active value"),
                    );
                    let callback_return_value = callback_return_w0;
                    Ok(callback_return_value)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_9067021b3e3252ad",
                            "(i128)->i128",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_wide(
                __prebindgen_argument_0,
            );
            let ret_w0 = {
                let bits = (source_value) as u128;
                __Pb128 {
                    hi: (bits >> 64) as u64,
                    lo: bits as u64,
                }
            };
            let result = perftest_callback_return_wide_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_callback_return_zero_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_callback_return_zero(
    p0_callback: perftest_callback_15f1a05217a2ceea,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_callback_return_zero_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_callback_owner = if true {
                Some(__PbCallbackOwner::new(p0_callback.context, p0_callback.drop))
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("validated active callback");
            let p0_callback_owner = p0_callback_owner
                .expect("validated active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w1 = raw_result;
                    {
                        let callback_return_w0 = if true {
                            Some(callback_return_w1)
                        } else {
                            None
                        };
                        let callback_return_w0: i64 = ::core::convert::Into::into(
                            callback_return_w0.expect("active value"),
                        );
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    }
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "perftest_callback_15f1a05217a2ceea",
                            "()->i64",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::callback_return_zero(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_callback_return_zero_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_alias_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_alias_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_alias(
    p0_w1: __PbRows,
    p1_w1: *const perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_alias_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_alias_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_alias_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w5_pointer = if true {
                        (p0_w5) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w5_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w4 = if true {
                                Some(unsafe { Box::from_raw(p0_w5_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w4: perftest_flat::OwnedResource = *p0_w4
                                    .expect("active allocation");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w1_pointer = if true {
                p1_w1 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p1_w1_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in p0_w1_identities_0.iter().copied() {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p1_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: &perftest_flat::OwnedResource = unsafe { &(*p1_w1_pointer) };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_alias(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = perftest_collection_alias_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_borrow_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_borrow_ret_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_borrow_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_borrow_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_borrow_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_borrow(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_borrow_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_borrow_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_borrow_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = &p0_w0;
            let source_value = perftest_flat::collection_borrow(__prebindgen_argument_0);
            let ret_w0_source = &(source_value);
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_borrow_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.iter().enumerate() {
                let ret_w4: &i64 = ret_w1;
                let ret_w4 = *ret_w4;
                ret_w0
                    .push(__PbRow_collection_borrow_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_borrow_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_borrow_each_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
pub struct perftest_collection_borrow_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_borrow_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_borrow_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_borrow_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_borrow_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = &p0_w0;
            let source_value = perftest_flat::collection_borrow(__prebindgen_argument_0);
            let ret_w0_source = &(source_value);
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.iter().enumerate() {
                let ret_w4: &i64 = ret_w1;
                let ret_w4 = *ret_w4;
                let __pb_callback_value_0 = ret_w4;
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_borrow_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_bundle_ret_w2 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_bundle_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_bundle_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_bundle_result {
    pub v0: __PbRows,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_bundle(out1: *mut i64) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_bundle_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::collection_bundle();
            let ret_w0 = perftest_flat::collection_bundle_into(source_value);
            let ret_w1 = ret_w0;
            let ret_w2_source = ret_w1.values;
            let ret_w4 = ret_w2_source.len();
            let mut ret_w2_commits = Vec::with_capacity(ret_w4);
            let mut ret_w2: Vec<__PbRow_collection_bundle_ret_w2> = Vec::with_capacity(
                ret_w4,
            );
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w3),
                );
                let ret_w6_owner = __PbOutputOwner(ret_w6);
                ret_w2
                    .push(__PbRow_collection_bundle_ret_w2 {
                        v0: ret_w6,
                    });
                ret_w2_commits
                    .push(move || {
                        ::core::mem::forget(ret_w6_owner);
                    });
            }
            let ret_w2 = ret_w2.into_boxed_slice();
            let ret_w7 = ret_w1.tag;
            let result = perftest_collection_bundle_result {
                v0: __PbRows {
                    data: if ret_w2.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w2.as_ptr().cast()
                    },
                    len: ret_w2.len(),
                },
                v1: ret_w7,
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w2);
                for commit in ret_w2_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_bytes_p0_w1 {
    pub v0: __PbBuffer,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_bytes_ret_w0 {
    pub v0: __PbBuffer,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_bytes_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_bytes_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_bytes_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_bytes(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_bytes_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_bytes_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_bytes_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(unsafe { __pb_read(p0_w5)? })
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<Vec<u8>, String> {
                                let p0_w4: Vec<u8> = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<u8>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_bytes(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_bytes_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w4 = Box::into_raw(ret_w4);
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                ret_w0
                    .push(__PbRow_collection_bytes_ret_w0 {
                        v0: __PbBuffer {
                            data: ret_w4 as *const u8,
                            len: ret_w4.len(),
                        },
                    });
                ret_w0_commits
                    .push(move || {
                        ::core::mem::forget(ret_w4_owner);
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_bytes_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_bytes_each_p0_w1 {
    pub v0: __PbBuffer,
}
#[repr(C)]
pub struct perftest_collection_bytes_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_bytes_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbBuffer)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_bytes_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_bytes_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_bytes_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(unsafe { __pb_read(p0_w5)? })
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<Vec<u8>, String> {
                                let p0_w4: Vec<u8> = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<u8>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_bytes(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w4 = Box::into_raw(ret_w4);
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                let __pb_callback_value_0 = __PbBuffer {
                    data: ret_w4 as *const u8,
                    len: ret_w4.len(),
                };
                ::core::mem::forget(ret_w4_owner);
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_bytes_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_construct_all_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_construct_all_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_construct_all(p0_w1: __PbRows) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_construct_all_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_construct_all_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_construct_all_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w6_pointer = if true {
                        (p0_w6) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w6_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5 = if true {
                                Some(unsafe { Box::from_raw(p0_w6_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::CollectionItem,
                                String,
                            > {
                                let p0_w5: perftest_flat::OwnedResource = *p0_w5
                                    .expect("active allocation");
                                let p0_w4: perftest_flat::CollectionItem = perftest_flat::collection_construct(
                                    p0_w5,
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::CollectionItem> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_construct_all(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_collection_construct_all_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_enum_p0_w1 {
    pub v0: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_enum_ret_w0 {
    pub v0: i32,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_enum_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_enum_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_enum_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_enum(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_collection_enum_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_enum_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_enum_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(
                            match p0_w5 {
                                -2147483648i32 => perftest_flat::SparseCode::Bottom,
                                -2i32 => perftest_flat::SparseCode::MinusTwo,
                                -1i32 => perftest_flat::SparseCode::MinusOne,
                                0i32 => perftest_flat::SparseCode::Zero,
                                7i32 => perftest_flat::SparseCode::Seven,
                                8i32 => perftest_flat::SparseCode::Eight,
                                2147483647i32 => perftest_flat::SparseCode::Top,
                                _ => return Err("invalid enum discriminant".into()),
                            },
                        )
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<
                                perftest_flat::SparseCode,
                                String,
                            > {
                                let p0_w4 = p0_w4.expect("active enum");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::SparseCode> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_enum(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_enum_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4: i32 = match &(ret_w1) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                ret_w0
                    .push(__PbRow_collection_enum_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_enum_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_enum_each_p0_w1 {
    pub v0: i32,
}
#[repr(C)]
pub struct perftest_collection_enum_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_enum_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i32)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_enum_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_enum_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_enum_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(
                            match p0_w5 {
                                -2147483648i32 => perftest_flat::SparseCode::Bottom,
                                -2i32 => perftest_flat::SparseCode::MinusTwo,
                                -1i32 => perftest_flat::SparseCode::MinusOne,
                                0i32 => perftest_flat::SparseCode::Zero,
                                7i32 => perftest_flat::SparseCode::Seven,
                                8i32 => perftest_flat::SparseCode::Eight,
                                2147483647i32 => perftest_flat::SparseCode::Top,
                                _ => return Err("invalid enum discriminant".into()),
                            },
                        )
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<
                                perftest_flat::SparseCode,
                                String,
                            > {
                                let p0_w4 = p0_w4.expect("active enum");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::SparseCode> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_enum(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4: i32 = match &(ret_w1) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                let __pb_callback_value_0 = ret_w4;
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_enum_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_collection_fail_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_fail(
    error: *mut __PbError_collection_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_collection_fail_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::collection_fail();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_collection_error(
                                ret_w1,
                                error,
                                &mut error_reported,
                            )
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_collection_fail_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_collection_error(
                            <perftest_flat::CollectionError as From<
                                String,
                            >>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_handles_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_handles_ret_w2 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_handles_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_handles_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_handles_result {
    pub v0: __PbRows,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_handles(
    p0_w1: __PbRows,
    p1_w1: u32,
    p2_w1: i32,
    out0: *mut __PbRows,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_handles_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_handles_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_handles_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w5_pointer = if true {
                        (p0_w5) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w5_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w4 = if true {
                                Some(unsafe { Box::from_raw(p0_w5_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w4: perftest_flat::OwnedResource = *p0_w4
                                    .expect("active allocation");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true {
                Some(char::from_u32(p1_w1).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: char = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: i32 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let source_value = perftest_flat::collection_handles(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            let mut ret_w2_commits = Vec::with_capacity(ret_w4);
            let mut ret_w2: Vec<__PbRow_collection_handles_ret_w2> = Vec::with_capacity(
                ret_w4,
            );
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w3),
                );
                let ret_w6_owner = __PbOutputOwner(ret_w6);
                ret_w2
                    .push(__PbRow_collection_handles_ret_w2 {
                        v0: ret_w6,
                    });
                ret_w2_commits
                    .push(move || {
                        ::core::mem::forget(ret_w6_owner);
                    });
            }
            let ret_w2 = ret_w2.into_boxed_slice();
            let result = perftest_collection_handles_result {
                v0: __PbRows {
                    data: if ret_w2.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w2.as_ptr().cast()
                    },
                    len: ret_w2.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w2);
                for commit in ret_w2_commits {
                    commit();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v0).data as *mut __PbRow_collection_handles_ret_w2,
                            (value.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        if !(row.v0).is_null() {
                            let _ = ::prebindgen_runtime::boundary(|| {
                                drop(
                                    Box::from_raw((row.v0) as *mut perftest_flat::OwnedResource),
                                );
                                Ok(())
                            });
                        }
                    }
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_handles_each_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_handles_each_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_handles_each(
    p0_w1: __PbRows,
    p1_w1: u32,
    p2_w1: i32,
    __pb_callback: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut perftest_flat::OwnedResource),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out0: *mut usize,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_handles_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_handles_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_handles_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w5_pointer = if true {
                        (p0_w5) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w5_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w4 = if true {
                                Some(unsafe { Box::from_raw(p0_w5_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w4: perftest_flat::OwnedResource = *p0_w4
                                    .expect("active allocation");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true {
                Some(char::from_u32(p1_w1).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: char = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: i32 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let source_value = perftest_flat::collection_handles(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w3),
                );
                let ret_w6_owner = __PbOutputOwner(ret_w6);
                let __pb_callback_value_0 = ret_w6;
                ::core::mem::forget(ret_w6_owner);
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_handles_each_result {
                v0: ret_w4,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_i64_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_i64_ret_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_i64_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_i64_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_i64_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_i64(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_collection_i64_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_i64_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_i64_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_i64(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_i64_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                ret_w0
                    .push(__PbRow_collection_i64_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_i64_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_i64_each_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
pub struct perftest_collection_i64_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_i64_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_i64_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_i64_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_i64_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_i64(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                let __pb_callback_value_0 = ret_w4;
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_i64_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_late_drop_ret_w7 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_late_drop_ret_w7_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_late_drop_ret_w7,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_late_drop_ret_w3 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_late_drop_ret_w3_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_late_drop_ret_w3,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_late_drop_result {
    pub v0: __PbRows,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_late_drop(
    out0: *mut __PbRows,
    error: *mut __PbError_result_error,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_late_drop_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let source_value = perftest_flat::collection_late_drop();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        unsafe {
                            __pb_report_result_error(ret_w1, error, &mut error_reported)
                        }?;
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::collection_late_values(&ret_w0);
            let ret_w3_source = ret_w2;
            let ret_w5 = ret_w3_source.len();
            let mut ret_w3_commits = Vec::with_capacity(ret_w5);
            let mut ret_w3: Vec<__PbRow_collection_late_drop_ret_w3> = Vec::with_capacity(
                ret_w5,
            );
            for (ret_w6, ret_w4) in ret_w3_source.into_iter().enumerate() {
                let ret_w7_source = ret_w4;
                let ret_w9 = ret_w7_source.len();
                let mut ret_w7_commits = Vec::with_capacity(ret_w9);
                let mut ret_w7: Vec<__PbRow_collection_late_drop_ret_w7> = Vec::with_capacity(
                    ret_w9,
                );
                for (ret_w10, ret_w8) in ret_w7_source.into_iter().enumerate() {
                    let ret_w11: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w8),
                    );
                    let ret_w11_owner = __PbOutputOwner(ret_w11);
                    ret_w7
                        .push(__PbRow_collection_late_drop_ret_w7 {
                            v0: ret_w11,
                        });
                    ret_w7_commits
                        .push(move || {
                            ::core::mem::forget(ret_w11_owner);
                        });
                }
                let ret_w7 = ret_w7.into_boxed_slice();
                ret_w3
                    .push(__PbRow_collection_late_drop_ret_w3 {
                        v0: __PbRows {
                            data: if ret_w7.is_empty() {
                                ::core::ptr::null()
                            } else {
                                ret_w7.as_ptr().cast()
                            },
                            len: ret_w7.len(),
                        },
                    });
                ret_w3_commits
                    .push(move || {
                        let _ = Box::into_raw(ret_w7);
                        for commit in ret_w7_commits {
                            commit();
                        }
                    });
            }
            let ret_w3 = ret_w3.into_boxed_slice();
            let result = perftest_collection_late_drop_result {
                v0: __PbRows {
                    data: if ret_w3.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w3.as_ptr().cast()
                    },
                    len: ret_w3.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w3);
                for commit in ret_w3_commits {
                    commit();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v0).data as *mut __PbRow_collection_late_drop_ret_w3,
                            (value.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        if (row.v0).len != 0 {
                            let rows = Box::from_raw(
                                std::ptr::slice_from_raw_parts_mut(
                                    (row.v0).data as *mut __PbRow_collection_late_drop_ret_w7,
                                    (row.v0).len,
                                ),
                            );
                            for row in rows.iter() {
                                if !(row.v0).is_null() {
                                    let _ = ::prebindgen_runtime::boundary(|| {
                                        drop(
                                            Box::from_raw((row.v0) as *mut perftest_flat::OwnedResource),
                                        );
                                        Ok(())
                                    });
                                }
                            }
                        }
                    }
                }
            }
            if !error_reported && !error.is_null() {
                let mut delivered = false;
                if ::prebindgen_runtime::boundary(|| unsafe {
                        __pb_report_result_error(
                            <perftest_flat::CResultError as From<String>>::from(message),
                            error,
                            &mut delivered,
                        )
                    })
                    .is_err() && !delivered
                {
                    std::ptr::write(error, std::mem::zeroed());
                }
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_make_items_ret_w0 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_make_items_ret_w0_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_make_items_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_make_items_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_make_items() -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_make_items_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::collection_make_items();
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_make_items_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = perftest_flat::collection_item_into(ret_w1);
                let ret_w5 = ret_w4;
                let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w5.resource),
                );
                let ret_w6_owner = __PbOutputOwner(ret_w6);
                ret_w0
                    .push(__PbRow_collection_make_items_ret_w0 {
                        v0: ret_w6,
                    });
                ret_w0_commits
                    .push(move || {
                        ::core::mem::forget(ret_w6_owner);
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_make_items_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_p0_w5 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_p0_w1 {
    pub v0: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_ret_w4 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_ret_w4_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_ret_w0 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_nested_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_nested_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_nested_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_nested_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let mut p0_w5_staged = Vec::new();
                    if true {
                        let p0_w6 = (p0_w5).len;
                        ::prebindgen_runtime::validate_slice(
                            (p0_w5).data as *const __PbRow_collection_nested_p0_w5,
                            p0_w6,
                        )?;
                        for p0_w7 in 0..p0_w6 {
                            let p0_w9 = unsafe {
                                (*(((p0_w5).data as *const __PbRow_collection_nested_p0_w5)
                                    .add(p0_w7)))
                                    .v0
                            };
                            let p0_w8 = if true { Some(p0_w9) } else { None };
                            p0_w5_staged
                                .push(move || {
                                    move || -> ::core::result::Result<i64, String> {
                                        let p0_w8: i64 = ::core::convert::Into::into(
                                            p0_w8.expect("active value"),
                                        );
                                        let p0_w4 = p0_w8;
                                        Ok(p0_w4)
                                    }
                                });
                        }
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5_acquired: Vec<_> = p0_w5_staged
                                .into_iter()
                                .map(|acquire| acquire())
                                .collect();
                            move || -> ::core::result::Result<Vec<i64>, String> {
                                let p0_w4: Vec<i64> = p0_w5_acquired
                                    .into_iter()
                                    .map(|construct| construct())
                                    .collect::<::core::result::Result<_, String>>()?;
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<i64>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_nested(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_nested_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4_source = ret_w1;
                let ret_w6 = ret_w4_source.len();
                let mut ret_w4_commits = Vec::with_capacity(ret_w6);
                let mut ret_w4: Vec<__PbRow_collection_nested_ret_w4> = Vec::with_capacity(
                    ret_w6,
                );
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8 = ret_w5;
                    ret_w4
                        .push(__PbRow_collection_nested_ret_w4 {
                            v0: ret_w8,
                        });
                    ret_w4_commits.push(move || {});
                }
                let ret_w4 = ret_w4.into_boxed_slice();
                ret_w0
                    .push(__PbRow_collection_nested_ret_w0 {
                        v0: __PbRows {
                            data: if ret_w4.is_empty() {
                                ::core::ptr::null()
                            } else {
                                ret_w4.as_ptr().cast()
                            },
                            len: ret_w4.len(),
                        },
                    });
                ret_w0_commits
                    .push(move || {
                        let _ = Box::into_raw(ret_w4);
                        for commit in ret_w4_commits {
                            commit();
                        }
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_nested_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_each_p0_w5 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_each_p0_w1 {
    pub v0: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_each_ret_w4 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_each_ret_w4_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_each_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_nested_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbRows)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_nested_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_nested_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_nested_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let mut p0_w5_staged = Vec::new();
                    if true {
                        let p0_w6 = (p0_w5).len;
                        ::prebindgen_runtime::validate_slice(
                            (p0_w5).data as *const __PbRow_collection_nested_each_p0_w5,
                            p0_w6,
                        )?;
                        for p0_w7 in 0..p0_w6 {
                            let p0_w9 = unsafe {
                                (*(((p0_w5).data
                                    as *const __PbRow_collection_nested_each_p0_w5)
                                    .add(p0_w7)))
                                    .v0
                            };
                            let p0_w8 = if true { Some(p0_w9) } else { None };
                            p0_w5_staged
                                .push(move || {
                                    move || -> ::core::result::Result<i64, String> {
                                        let p0_w8: i64 = ::core::convert::Into::into(
                                            p0_w8.expect("active value"),
                                        );
                                        let p0_w4 = p0_w8;
                                        Ok(p0_w4)
                                    }
                                });
                        }
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5_acquired: Vec<_> = p0_w5_staged
                                .into_iter()
                                .map(|acquire| acquire())
                                .collect();
                            move || -> ::core::result::Result<Vec<i64>, String> {
                                let p0_w4: Vec<i64> = p0_w5_acquired
                                    .into_iter()
                                    .map(|construct| construct())
                                    .collect::<::core::result::Result<_, String>>()?;
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<i64>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_nested(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4_source = ret_w1;
                let ret_w6 = ret_w4_source.len();
                let mut ret_w4_commits = Vec::with_capacity(ret_w6);
                let mut ret_w4: Vec<__PbRow_collection_nested_each_ret_w4> = Vec::with_capacity(
                    ret_w6,
                );
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8 = ret_w5;
                    ret_w4
                        .push(__PbRow_collection_nested_each_ret_w4 {
                            v0: ret_w8,
                        });
                    ret_w4_commits.push(move || {});
                }
                let ret_w4 = ret_w4.into_boxed_slice();
                let __pb_callback_value_0 = __PbRows {
                    data: if ret_w4.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w4.as_ptr().cast()
                    },
                    len: ret_w4.len(),
                };
                let _ = Box::into_raw(ret_w4);
                for commit in ret_w4_commits {
                    commit();
                }
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_nested_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_p0_w5 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_p0_w1 {
    pub v0: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_ret_w4 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_handles_ret_w4_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_handles_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_ret_w0 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_handles_ret_w0_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_handles_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_nested_handles_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_handles(
    p0_w1: __PbRows,
) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_nested_handles_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_nested_handles_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_nested_handles_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let mut p0_w5_identities_0: Vec<usize> = Vec::new();
                    let mut p0_w5_staged = Vec::new();
                    if true {
                        let p0_w6 = (p0_w5).len;
                        ::prebindgen_runtime::validate_slice(
                            (p0_w5).data
                                as *const __PbRow_collection_nested_handles_p0_w5,
                            p0_w6,
                        )?;
                        for p0_w7 in 0..p0_w6 {
                            let p0_w9 = unsafe {
                                (*(((p0_w5).data
                                    as *const __PbRow_collection_nested_handles_p0_w5)
                                    .add(p0_w7)))
                                    .v0
                            };
                            let p0_w9_pointer = if true {
                                (p0_w9) as *mut perftest_flat::OwnedResource
                            } else {
                                ::core::ptr::null_mut()
                            };
                            if true {
                                ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                            }
                            if true {
                                p0_w5_identities_0.push((p0_w9_pointer.addr()) as usize);
                            }
                            p0_w5_staged
                                .push(move || {
                                    let p0_w8 = if true {
                                        Some(unsafe { Box::from_raw(p0_w9_pointer) })
                                    } else {
                                        None
                                    };
                                    move || -> ::core::result::Result<
                                        perftest_flat::OwnedResource,
                                        String,
                                    > {
                                        let p0_w8: perftest_flat::OwnedResource = *p0_w8
                                            .expect("active allocation");
                                        let p0_w4 = p0_w8;
                                        Ok(p0_w4)
                                    }
                                });
                        }
                    }
                    if true {
                        p0_w1_identities_0.extend(p0_w5_identities_0.iter().copied());
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5_acquired: Vec<_> = p0_w5_staged
                                .into_iter()
                                .map(|acquire| acquire())
                                .collect();
                            move || -> ::core::result::Result<
                                Vec<perftest_flat::OwnedResource>,
                                String,
                            > {
                                let p0_w4: Vec<perftest_flat::OwnedResource> = p0_w5_acquired
                                    .into_iter()
                                    .map(|construct| construct())
                                    .collect::<::core::result::Result<_, String>>()?;
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<perftest_flat::OwnedResource>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_nested_handles(
                __prebindgen_argument_0,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_nested_handles_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4_source = ret_w1;
                let ret_w6 = ret_w4_source.len();
                let mut ret_w4_commits = Vec::with_capacity(ret_w6);
                let mut ret_w4: Vec<__PbRow_collection_nested_handles_ret_w4> = Vec::with_capacity(
                    ret_w6,
                );
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    ret_w4
                        .push(__PbRow_collection_nested_handles_ret_w4 {
                            v0: ret_w8,
                        });
                    ret_w4_commits
                        .push(move || {
                            ::core::mem::forget(ret_w8_owner);
                        });
                }
                let ret_w4 = ret_w4.into_boxed_slice();
                ret_w0
                    .push(__PbRow_collection_nested_handles_ret_w0 {
                        v0: __PbRows {
                            data: if ret_w4.is_empty() {
                                ::core::ptr::null()
                            } else {
                                ret_w4.as_ptr().cast()
                            },
                            len: ret_w4.len(),
                        },
                    });
                ret_w0_commits
                    .push(move || {
                        let _ = Box::into_raw(ret_w4);
                        for commit in ret_w4_commits {
                            commit();
                        }
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_nested_handles_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_each_p0_w5 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_each_p0_w1 {
    pub v0: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_nested_handles_each_ret_w4 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_handles_each_ret_w4_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_nested_handles_each_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_nested_handles_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_nested_handles_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbRows)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_nested_handles_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_nested_handles_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data
                            as *const __PbRow_collection_nested_handles_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let mut p0_w5_identities_0: Vec<usize> = Vec::new();
                    let mut p0_w5_staged = Vec::new();
                    if true {
                        let p0_w6 = (p0_w5).len;
                        ::prebindgen_runtime::validate_slice(
                            (p0_w5).data
                                as *const __PbRow_collection_nested_handles_each_p0_w5,
                            p0_w6,
                        )?;
                        for p0_w7 in 0..p0_w6 {
                            let p0_w9 = unsafe {
                                (*(((p0_w5).data
                                    as *const __PbRow_collection_nested_handles_each_p0_w5)
                                    .add(p0_w7)))
                                    .v0
                            };
                            let p0_w9_pointer = if true {
                                (p0_w9) as *mut perftest_flat::OwnedResource
                            } else {
                                ::core::ptr::null_mut()
                            };
                            if true {
                                ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                            }
                            if true {
                                p0_w5_identities_0.push((p0_w9_pointer.addr()) as usize);
                            }
                            p0_w5_staged
                                .push(move || {
                                    let p0_w8 = if true {
                                        Some(unsafe { Box::from_raw(p0_w9_pointer) })
                                    } else {
                                        None
                                    };
                                    move || -> ::core::result::Result<
                                        perftest_flat::OwnedResource,
                                        String,
                                    > {
                                        let p0_w8: perftest_flat::OwnedResource = *p0_w8
                                            .expect("active allocation");
                                        let p0_w4 = p0_w8;
                                        Ok(p0_w4)
                                    }
                                });
                        }
                    }
                    if true {
                        p0_w1_identities_0.extend(p0_w5_identities_0.iter().copied());
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5_acquired: Vec<_> = p0_w5_staged
                                .into_iter()
                                .map(|acquire| acquire())
                                .collect();
                            move || -> ::core::result::Result<
                                Vec<perftest_flat::OwnedResource>,
                                String,
                            > {
                                let p0_w4: Vec<perftest_flat::OwnedResource> = p0_w5_acquired
                                    .into_iter()
                                    .map(|construct| construct())
                                    .collect::<::core::result::Result<_, String>>()?;
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Vec<perftest_flat::OwnedResource>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_nested_handles(
                __prebindgen_argument_0,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4_source = ret_w1;
                let ret_w6 = ret_w4_source.len();
                let mut ret_w4_commits = Vec::with_capacity(ret_w6);
                let mut ret_w4: Vec<__PbRow_collection_nested_handles_each_ret_w4> = Vec::with_capacity(
                    ret_w6,
                );
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    ret_w4
                        .push(__PbRow_collection_nested_handles_each_ret_w4 {
                            v0: ret_w8,
                        });
                    ret_w4_commits
                        .push(move || {
                            ::core::mem::forget(ret_w8_owner);
                        });
                }
                let ret_w4 = ret_w4.into_boxed_slice();
                let __pb_callback_value_0 = __PbRows {
                    data: if ret_w4.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w4.as_ptr().cast()
                    },
                    len: ret_w4.len(),
                };
                let _ = Box::into_raw(ret_w4);
                for commit in ret_w4_commits {
                    commit();
                }
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_nested_handles_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_numbers_ret_w2 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_numbers_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_numbers_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_numbers_result {
    pub v0: __PbRows,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_numbers(out1: *mut i64) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_numbers_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::collection_numbers();
            let ret_w0 = perftest_flat::collection_numbers_view(&source_value);
            let ret_w1 = &(ret_w0);
            let ret_w2_source = &(&ret_w1.values);
            let ret_w4 = ret_w2_source.len();
            let mut ret_w2_commits = Vec::with_capacity(ret_w4);
            let mut ret_w2: Vec<__PbRow_collection_numbers_ret_w2> = Vec::with_capacity(
                ret_w4,
            );
            for (ret_w5, ret_w3) in ret_w2_source.iter().enumerate() {
                let ret_w6: &i64 = ret_w3;
                let ret_w6 = *ret_w6;
                ret_w2
                    .push(__PbRow_collection_numbers_ret_w2 {
                        v0: ret_w6,
                    });
                ret_w2_commits.push(move || {});
            }
            let ret_w2 = ret_w2.into_boxed_slice();
            let ret_w7: &i64 = &ret_w1.tag;
            let ret_w7 = *ret_w7;
            let result = perftest_collection_numbers_result {
                v0: __PbRows {
                    data: if ret_w2.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w2.as_ptr().cast()
                    },
                    len: ret_w2.len(),
                },
                v1: ret_w7,
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w2);
                for commit in ret_w2_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_p0_w3 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_ret_w2 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_optional_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_optional_result {
    pub v0: i32,
    pub v1: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional(
    p0_w1: i32,
    p0_w3: __PbRows,
    out1: *mut __PbRows,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data as *const __PbRow_collection_optional_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w6 = if true { Some(p0_w7) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w6: i64 = ::core::convert::Into::into(
                                    p0_w6.expect("active value"),
                                );
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<i64> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                let mut ret_w2_commits = Vec::with_capacity(ret_w4);
                let mut ret_w2: Vec<__PbRow_collection_optional_ret_w2> = Vec::with_capacity(
                    ret_w4,
                );
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6 = ret_w3;
                    ret_w2
                        .push(__PbRow_collection_optional_ret_w2 {
                            v0: ret_w6,
                        });
                    ret_w2_commits.push(move || {});
                }
                let ret_w2 = ret_w2.into_boxed_slice();
                let ret_w0_values = (
                    __PbRows {
                        data: if ret_w2.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w2.as_ptr().cast()
                        },
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        let _ = Box::into_raw(ret_w2);
                        for commit in ret_w2_commits {
                            commit();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbRows {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_collection_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_each_p0_w3 {
    pub v0: i64,
}
#[repr(C)]
pub struct perftest_collection_optional_each_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_each(
    p0_w1: i32,
    p0_w3: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_each_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data as *const __PbRow_collection_optional_each_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w6 = if true { Some(p0_w7) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w6: i64 = ::core::convert::Into::into(
                                    p0_w6.expect("active value"),
                                );
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<i64> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6 = ret_w3;
                    let __pb_callback_value_0 = ret_w6;
                    unsafe {
                        __pb_callback
                            .expect(
                                "validated callback",
                            )(__pb_callback_context, __pb_callback_value_0);
                    }
                }
                let ret_w0_values = (ret_w4,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_collection_optional_each_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_handles_p0_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_handles_ret_w2 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_handles_ret_w2_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_optional_handles_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_optional_handles_result {
    pub v0: i32,
    pub v1: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_handles(
    p0_w1: i32,
    p0_w3: __PbRows,
    out1: *mut __PbRows,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_handles_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_identities_0: Vec<usize> = Vec::new();
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_handles_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data
                            as *const __PbRow_collection_optional_handles_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w7_pointer = if true {
                        (p0_w7) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w7_pointer)?;
                    }
                    if true {
                        p0_w3_identities_0.push((p0_w7_pointer.addr()) as usize);
                    }
                    p0_w3_staged
                        .push(move || {
                            let p0_w6 = if true {
                                Some(unsafe { Box::from_raw(p0_w7_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w6: perftest_flat::OwnedResource = *p0_w6
                                    .expect("active allocation");
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if p0_w1 == 1i32 {
                        for identity in p0_w3_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<perftest_flat::OwnedResource> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional_handles(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                let mut ret_w2_commits = Vec::with_capacity(ret_w4);
                let mut ret_w2: Vec<__PbRow_collection_optional_handles_ret_w2> = Vec::with_capacity(
                    ret_w4,
                );
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w3),
                    );
                    let ret_w6_owner = __PbOutputOwner(ret_w6);
                    ret_w2
                        .push(__PbRow_collection_optional_handles_ret_w2 {
                            v0: ret_w6,
                        });
                    ret_w2_commits
                        .push(move || {
                            ::core::mem::forget(ret_w6_owner);
                        });
                }
                let ret_w2 = ret_w2.into_boxed_slice();
                let ret_w0_values = (
                    __PbRows {
                        data: if ret_w2.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w2.as_ptr().cast()
                        },
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        let _ = Box::into_raw(ret_w2);
                        for commit in ret_w2_commits {
                            commit();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbRows {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_collection_optional_handles_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_handles_each_p0_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_optional_handles_each_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_handles_each(
    p0_w1: i32,
    p0_w3: __PbRows,
    __pb_callback: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut perftest_flat::OwnedResource),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_handles_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_identities_0: Vec<usize> = Vec::new();
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_handles_each_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data
                            as *const __PbRow_collection_optional_handles_each_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w7_pointer = if true {
                        (p0_w7) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w7_pointer)?;
                    }
                    if true {
                        p0_w3_identities_0.push((p0_w7_pointer.addr()) as usize);
                    }
                    p0_w3_staged
                        .push(move || {
                            let p0_w6 = if true {
                                Some(unsafe { Box::from_raw(p0_w7_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w6: perftest_flat::OwnedResource = *p0_w6
                                    .expect("active allocation");
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if p0_w1 == 1i32 {
                        for identity in p0_w3_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<perftest_flat::OwnedResource> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional_handles(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w3),
                    );
                    let ret_w6_owner = __PbOutputOwner(ret_w6);
                    let __pb_callback_value_0 = ret_w6;
                    ::core::mem::forget(ret_w6_owner);
                    unsafe {
                        __pb_callback
                            .expect(
                                "validated callback",
                            )(__pb_callback_context, __pb_callback_value_0);
                    }
                }
                let ret_w0_values = (ret_w4,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_collection_optional_handles_each_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_items_p0_w3 {
    pub v0: i32,
    pub v1: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_items_ret_w2 {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_items_ret_w2_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_optional_items_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_optional_items_result {
    pub v0: i32,
    pub v1: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_items(
    p0_w1: i32,
    p0_w3: __PbRows,
    out1: *mut __PbRows,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_items_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_items_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data as *const __PbRow_collection_optional_items_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w9 = unsafe {
                        (*((p0_w3.data as *const __PbRow_collection_optional_items_p0_w3)
                            .add(p0_w5)))
                            .v1
                    };
                    if (p0_w7) != 0 && (p0_w7) != 1 {
                        return Err("invalid option presence".into());
                    }
                    let p0_w8 = if (p0_w7) == 1i32 { Some(p0_w9) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<Option<i64>, String> {
                                let p0_w6 = if (p0_w7) == 1i32 {
                                    let p0_w8: i64 = ::core::convert::Into::into(
                                        p0_w8.expect("active value"),
                                    );
                                    Some(p0_w8)
                                } else {
                                    None
                                };
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<Option<i64>> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional_items(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                let mut ret_w2_commits = Vec::with_capacity(ret_w4);
                let mut ret_w2: Vec<__PbRow_collection_optional_items_ret_w2> = Vec::with_capacity(
                    ret_w4,
                );
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let (ret_w6, (ret_w6_value0,), ret_w6_commit) = if let Some(
                        ret_w7,
                    ) = ret_w3 {
                        let ret_w8 = ret_w7;
                        let ret_w6_values = (ret_w8,);
                        (1i32, ret_w6_values, Some(move || {}))
                    } else {
                        (0i32, (0,), None)
                    };
                    ret_w2
                        .push(__PbRow_collection_optional_items_ret_w2 {
                            v0: ret_w6,
                            v1: ret_w6_value0,
                        });
                    ret_w2_commits
                        .push(move || {
                            if let Some(ret_w6_action) = ret_w6_commit {
                                ret_w6_action();
                            }
                        });
                }
                let ret_w2 = ret_w2.into_boxed_slice();
                let ret_w0_values = (
                    __PbRows {
                        data: if ret_w2.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w2.as_ptr().cast()
                        },
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        let _ = Box::into_raw(ret_w2);
                        for commit in ret_w2_commits {
                            commit();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbRows {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_collection_optional_items_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_optional_items_each_p0_w3 {
    pub v0: i32,
    pub v1: i64,
}
#[repr(C)]
pub struct perftest_collection_optional_items_each_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_optional_items_each(
    p0_w1: i32,
    p0_w3: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i32, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_optional_items_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w3_staged = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_collection_optional_items_each_p0_w3,
                    p0_w4,
                )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = unsafe {
                        (*((p0_w3.data
                            as *const __PbRow_collection_optional_items_each_p0_w3)
                            .add(p0_w5)))
                            .v0
                    };
                    let p0_w9 = unsafe {
                        (*((p0_w3.data
                            as *const __PbRow_collection_optional_items_each_p0_w3)
                            .add(p0_w5)))
                            .v1
                    };
                    if (p0_w7) != 0 && (p0_w7) != 1 {
                        return Err("invalid option presence".into());
                    }
                    let p0_w8 = if (p0_w7) == 1i32 { Some(p0_w9) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<Option<i64>, String> {
                                let p0_w6 = if (p0_w7) == 1i32 {
                                    let p0_w8: i64 = ::core::convert::Into::into(
                                        p0_w8.expect("active value"),
                                    );
                                    Some(p0_w8)
                                } else {
                                    None
                                };
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<Option<i64>> = p0_w3_acquired
                    .into_iter()
                    .map(|construct| construct())
                    .collect::<::core::result::Result<_, String>>()?;
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_optional_items(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let (ret_w6, (ret_w6_value0,), ret_w6_commit) = if let Some(
                        ret_w7,
                    ) = ret_w3 {
                        let ret_w8 = ret_w7;
                        let ret_w6_values = (ret_w8,);
                        (1i32, ret_w6_values, Some(move || {}))
                    } else {
                        (0i32, (0,), None)
                    };
                    let __pb_callback_value_0 = ret_w6;
                    let __pb_callback_value_1 = ret_w6_value0;
                    if let Some(ret_w6_action) = ret_w6_commit {
                        ret_w6_action();
                    }
                    unsafe {
                        __pb_callback
                            .expect(
                                "validated callback",
                            )(
                            __pb_callback_context,
                            __pb_callback_value_0,
                            __pb_callback_value_1,
                        );
                    }
                }
                let ret_w0_values = (ret_w4,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_collection_optional_items_each_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_options_p0_w1 {
    pub v0: i32,
    pub v1: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_options_ret_w0 {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_options_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_options_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_options_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_options(
    p0_w1: __PbRows,
    p1_w1: i16,
) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_options_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_options_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_options_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w7 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_options_p0_w1)
                            .add(p0_w3)))
                            .v1
                    };
                    if (p0_w5) != 0 && (p0_w5) != 1 {
                        return Err("invalid option presence".into());
                    }
                    let p0_w6 = if (p0_w5) == 1i32 { Some(p0_w7) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<Option<i64>, String> {
                                let p0_w4 = if (p0_w5) == 1i32 {
                                    let p0_w6: i64 = ::core::convert::Into::into(
                                        p0_w6.expect("active value"),
                                    );
                                    Some(p0_w6)
                                } else {
                                    None
                                };
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Option<i64>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i16 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_options(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_options_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let (ret_w4, (ret_w4_value0,), ret_w4_commit) = if let Some(ret_w5) = ret_w1 {
                    let ret_w6 = ret_w5;
                    let ret_w4_values = (ret_w6,);
                    (1i32, ret_w4_values, Some(move || {}))
                } else {
                    (0i32, (0,), None)
                };
                ret_w0
                    .push(__PbRow_collection_options_ret_w0 {
                        v0: ret_w4,
                        v1: ret_w4_value0,
                    });
                ret_w0_commits
                    .push(move || {
                        if let Some(ret_w4_action) = ret_w4_commit {
                            ret_w4_action();
                        }
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_options_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_options_each_p0_w1 {
    pub v0: i32,
    pub v1: i64,
}
#[repr(C)]
pub struct perftest_collection_options_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_options_each(
    p0_w1: __PbRows,
    p1_w1: i16,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i32, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_options_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_options_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_options_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w7 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_options_each_p0_w1)
                            .add(p0_w3)))
                            .v1
                    };
                    if (p0_w5) != 0 && (p0_w5) != 1 {
                        return Err("invalid option presence".into());
                    }
                    let p0_w6 = if (p0_w5) == 1i32 { Some(p0_w7) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<Option<i64>, String> {
                                let p0_w4 = if (p0_w5) == 1i32 {
                                    let p0_w6: i64 = ::core::convert::Into::into(
                                        p0_w6.expect("active value"),
                                    );
                                    Some(p0_w6)
                                } else {
                                    None
                                };
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<Option<i64>> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i16 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_options(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let (ret_w4, (ret_w4_value0,), ret_w4_commit) = if let Some(ret_w5) = ret_w1 {
                    let ret_w6 = ret_w5;
                    let ret_w4_values = (ret_w6,);
                    (1i32, ret_w4_values, Some(move || {}))
                } else {
                    (0i32, (0,), None)
                };
                let __pb_callback_value_0 = ret_w4;
                let __pb_callback_value_1 = ret_w4_value0;
                if let Some(ret_w4_action) = ret_w4_commit {
                    ret_w4_action();
                }
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(
                        __pb_callback_context,
                        __pb_callback_value_0,
                        __pb_callback_value_1,
                    );
                }
            }
            let result = perftest_collection_options_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_probe_p0_w1 {
    pub v0: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_probe_ret_w0 {
    pub v0: u32,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_probe_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_probe_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_probe_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_probe(
    p0_w1: __PbRows,
    p1_w1: __PbBuffer,
) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_probe_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_probe_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_probe_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(char::from_u32(p0_w5).ok_or("invalid Unicode scalar")?)
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<char, String> {
                                let p0_w4: char = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p1_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<char> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let __prebindgen_argument_1 = ::core::convert::AsRef::<
                str,
            >::as_ref(p1_w0.as_ref().expect("active value"));
            let source_value = perftest_flat::collection_probe(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_probe_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = (ret_w1) as u32;
                ret_w0
                    .push(__PbRow_collection_probe_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_probe_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_resource_ids_p0_w1 {
    pub v0: *const perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_resource_ids_ret_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_resource_ids_ret_w0_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_resource_ids_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_resource_ids_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_resource_ids(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_resource_ids_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_resource_ids_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_resource_ids_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w6_pointer = if true {
                        (p0_w6) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w6_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w5: &perftest_flat::OwnedResource = unsafe {
                                    &(*p0_w6_pointer)
                                };
                                let p0_w4: perftest_flat::OwnedResource = perftest_flat::collection_resource_clone(
                                    p0_w5,
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = ();
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0.as_slice();
            let source_value = perftest_flat::collection_resource_ids(
                __prebindgen_argument_0,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_resource_ids_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                ret_w0
                    .push(__PbRow_collection_resource_ids_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_resource_ids_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_collection_resource_new_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_resource_new(
    p0_w1: i64,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_resource_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_resource_new(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_collection_resource_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_ret_w2 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_result_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_result_result {
    pub v0: __PbRows,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result(
    p0_w1: __PbRows,
    p1_w1: u8,
    out0: *mut __PbRows,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_result_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_result_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_result_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_result(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            let mut ret_w2_commits = Vec::with_capacity(ret_w4);
            let mut ret_w2: Vec<__PbRow_collection_result_ret_w2> = Vec::with_capacity(
                ret_w4,
            );
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6 = ret_w3;
                ret_w2
                    .push(__PbRow_collection_result_ret_w2 {
                        v0: ret_w6,
                    });
                ret_w2_commits.push(move || {});
            }
            let ret_w2 = ret_w2.into_boxed_slice();
            let result = perftest_collection_result_result {
                v0: __PbRows {
                    data: if ret_w2.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w2.as_ptr().cast()
                    },
                    len: ret_w2.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w2);
                for commit in ret_w2_commits {
                    commit();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v0).data as *mut __PbRow_collection_result_ret_w2,
                            (value.v0).len,
                        ),
                    );
                    for row in rows.iter() {}
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_each_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
pub struct perftest_collection_result_each_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result_each(
    p0_w1: __PbRows,
    p1_w1: u8,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out0: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_result_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_result_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_result_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_result(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6 = ret_w3;
                let __pb_callback_value_0 = ret_w6;
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_result_each_result {
                v0: ret_w4,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_optional_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_optional_ret_w4 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result_optional_ret_w4_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_result_optional_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_result_optional_result {
    pub v0: i32,
    pub v1: __PbRows,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result_optional(
    p0_w1: __PbRows,
    p1_w1: i32,
    out0: *mut i32,
    out1: *mut __PbRows,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_result_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_result_optional_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data
                            as *const __PbRow_collection_result_optional_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w5_pointer = if true {
                        (p0_w5) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w5_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w4 = if true {
                                Some(unsafe { Box::from_raw(p0_w5_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w4: perftest_flat::OwnedResource = *p0_w4
                                    .expect("active allocation");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_result_optional(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (ret_w2, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4_source = ret_w3;
                let ret_w6 = ret_w4_source.len();
                let mut ret_w4_commits = Vec::with_capacity(ret_w6);
                let mut ret_w4: Vec<__PbRow_collection_result_optional_ret_w4> = Vec::with_capacity(
                    ret_w6,
                );
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    ret_w4
                        .push(__PbRow_collection_result_optional_ret_w4 {
                            v0: ret_w8,
                        });
                    ret_w4_commits
                        .push(move || {
                            ::core::mem::forget(ret_w8_owner);
                        });
                }
                let ret_w4 = ret_w4.into_boxed_slice();
                let ret_w2_values = (
                    __PbRows {
                        data: if ret_w4.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w4.as_ptr().cast()
                        },
                        len: ret_w4.len(),
                    },
                );
                (
                    1i32,
                    ret_w2_values,
                    Some(move || {
                        let _ = Box::into_raw(ret_w4);
                        for commit in ret_w4_commits {
                            commit();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbRows {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_collection_result_optional_result {
                v0: ret_w2,
                v1: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            std::ptr::write(out1, value.v1);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v1).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v1).data
                                as *mut __PbRow_collection_result_optional_ret_w4,
                            (value.v1).len,
                        ),
                    );
                    for row in rows.iter() {
                        if !(row.v0).is_null() {
                            let _ = ::prebindgen_runtime::boundary(|| {
                                drop(
                                    Box::from_raw((row.v0) as *mut perftest_flat::OwnedResource),
                                );
                                Ok(())
                            });
                        }
                    }
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_result_optional_each_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_result_optional_each_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_result_optional_each(
    p0_w1: __PbRows,
    p1_w1: i32,
    __pb_callback: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut perftest_flat::OwnedResource),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out0: *mut i32,
    out1: *mut usize,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_result_optional_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_result_optional_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data
                            as *const __PbRow_collection_result_optional_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w5_pointer = if true {
                        (p0_w5) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w5_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w4 = if true {
                                Some(unsafe { Box::from_raw(p0_w5_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::OwnedResource,
                                String,
                            > {
                                let p0_w4: perftest_flat::OwnedResource = *p0_w4
                                    .expect("active allocation");
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::OwnedResource> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::collection_result_optional(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (ret_w2, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4_source = ret_w3;
                let ret_w6 = ret_w4_source.len();
                for (ret_w7, ret_w5) in ret_w4_source.into_iter().enumerate() {
                    let ret_w8: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    let __pb_callback_value_0 = ret_w8;
                    ::core::mem::forget(ret_w8_owner);
                    unsafe {
                        __pb_callback
                            .expect(
                                "validated callback",
                            )(__pb_callback_context, __pb_callback_value_0);
                    }
                }
                let ret_w2_values = (ret_w6,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_collection_result_optional_each_result {
                v0: ret_w2,
                v1: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            std::ptr::write(out1, value.v1);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_slice_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_slice_ret_w0 {
    pub v0: i64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_slice_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_slice_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_slice_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_slice(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_slice_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_slice_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_slice_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0.as_slice();
            let source_value = perftest_flat::collection_slice(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_slice_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                ret_w0
                    .push(__PbRow_collection_slice_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_slice_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_slice_each_p0_w1 {
    pub v0: i64,
}
#[repr(C)]
pub struct perftest_collection_slice_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_slice_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, i64)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_slice_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_slice_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_slice_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true { Some(p0_w5) } else { None };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0.as_slice();
            let source_value = perftest_flat::collection_slice(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                let __pb_callback_value_0 = ret_w4;
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_slice_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_text_p0_w1 {
    pub v0: __PbBuffer,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_text_ret_w0 {
    pub v0: __PbBuffer,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_text_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_text_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_text_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_text(p0_w1: __PbRows) -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_collection_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_text_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_text_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(
                            std::str::from_utf8(unsafe { __pb_read(p0_w5)? })
                                .map_err(|_| "invalid UTF-8")?,
                        )
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<String, String> {
                                let p0_w4: String = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<String> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_text(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_collection_text_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w4 = Box::into_raw(ret_w4);
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                ret_w0
                    .push(__PbRow_collection_text_ret_w0 {
                        v0: __PbBuffer {
                            data: ret_w4 as *const u8,
                            len: ret_w4.len(),
                        },
                    });
                ret_w0_commits
                    .push(move || {
                        ::core::mem::forget(ret_w4_owner);
                    });
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_collection_text_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_text_each_p0_w1 {
    pub v0: __PbBuffer,
}
#[repr(C)]
pub struct perftest_collection_text_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_text_each(
    p0_w1: __PbRows,
    __pb_callback: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbBuffer)>,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_text_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_text_each_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_collection_text_each_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w4 = if true {
                        Some(
                            std::str::from_utf8(unsafe { __pb_read(p0_w5)? })
                                .map_err(|_| "invalid UTF-8")?,
                        )
                    } else {
                        None
                    };
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<String, String> {
                                let p0_w4: String = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<String> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_text(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w4 = Box::into_raw(ret_w4);
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                let __pb_callback_value_0 = __PbBuffer {
                    data: ret_w4 as *const u8,
                    len: ret_w4.len(),
                };
                ::core::mem::forget(ret_w4_owner);
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_collection_text_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_try_construct_all_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct perftest_collection_try_construct_all_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_try_construct_all(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_try_construct_all_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let mut p0_w1_identities_0: Vec<usize> = Vec::new();
            let mut p0_w1_staged = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_try_construct_all_p0_w1,
                    p0_w2,
                )?;
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe {
                        (*((p0_w1.data
                            as *const __PbRow_collection_try_construct_all_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w6_pointer = if true {
                        (p0_w6) as *mut perftest_flat::OwnedResource
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if true {
                        ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                    }
                    if true {
                        p0_w1_identities_0.push((p0_w6_pointer.addr()) as usize);
                    }
                    p0_w1_staged
                        .push(move || {
                            let p0_w5 = if true {
                                Some(unsafe { Box::from_raw(p0_w6_pointer) })
                            } else {
                                None
                            };
                            move || -> ::core::result::Result<
                                perftest_flat::CollectionItem,
                                String,
                            > {
                                let p0_w5: perftest_flat::OwnedResource = *p0_w5
                                    .expect("active allocation");
                                let p0_w4: perftest_flat::CollectionItem = perftest_flat::collection_construct(
                                    p0_w5,
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let _: () = {
                let mut __prebindgen_parameter_resources: Vec<(usize, bool)> = Vec::new();
                let _: () = {
                    if true {
                        for identity in p0_w1_identities_0.iter().copied() {
                            __prebindgen_parameter_resources.push((identity, true));
                        }
                    }
                    __prebindgen_parameter_resources
                        .sort_unstable_by_key(|entry| entry.0);
                    if __prebindgen_parameter_resources
                        .windows(2)
                        .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                    {
                        return Err("conflicting input aliases".into());
                    }
                };
            };
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<perftest_flat::CollectionItem> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_try_construct_all(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = perftest_collection_try_construct_all_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_try_make_items_ret_w2 {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_try_make_items_ret_w2_rows_free(
    rows: __PbRows,
) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_collection_try_make_items_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_collection_try_make_items_result {
    pub v0: __PbRows,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_collection_try_make_items(
    out0: *mut __PbRows,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_collection_try_make_items_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let source_value = perftest_flat::collection_try_make_items();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            let mut ret_w2_commits = Vec::with_capacity(ret_w4);
            let mut ret_w2: Vec<__PbRow_collection_try_make_items_ret_w2> = Vec::with_capacity(
                ret_w4,
            );
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6 = perftest_flat::collection_item_into(ret_w3);
                let ret_w7 = ret_w6;
                let ret_w8: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w7.resource),
                );
                let ret_w8_owner = __PbOutputOwner(ret_w8);
                ret_w2
                    .push(__PbRow_collection_try_make_items_ret_w2 {
                        v0: ret_w8,
                    });
                ret_w2_commits
                    .push(move || {
                        ::core::mem::forget(ret_w8_owner);
                    });
            }
            let ret_w2 = ret_w2.into_boxed_slice();
            let result = perftest_collection_try_make_items_result {
                v0: __PbRows {
                    data: if ret_w2.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w2.as_ptr().cast()
                    },
                    len: ret_w2.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w2);
                for commit in ret_w2_commits {
                    commit();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v0).data
                                as *mut __PbRow_collection_try_make_items_ret_w2,
                            (value.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        if !(row.v0).is_null() {
                            let _ = ::prebindgen_runtime::boundary(|| {
                                drop(
                                    Box::from_raw((row.v0) as *mut perftest_flat::OwnedResource),
                                );
                                Ok(())
                            });
                        }
                    }
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_enum_names_echo_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_enum_names_echo(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_enum_names_echo_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    match p0_w1 {
                        4i32 => perftest_flat::EnumNames::Companion,
                        9i32 => perftest_flat::EnumNames::wireValue,
                        12i32 => perftest_flat::EnumNames::fromWire,
                        14i32 => perftest_flat::EnumNames::r#type,
                        16i32 => perftest_flat::EnumNames::when,
                        20i32 => perftest_flat::EnumNames::name,
                        21i32 => perftest_flat::EnumNames::ordinal,
                        22i32 => perftest_flat::EnumNames::entries,
                        24i32 => perftest_flat::EnumNames::value,
                        25i32 => perftest_flat::EnumNames::kotlin,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = p0_w0.expect("active enum");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::enum_names_echo(__prebindgen_argument_0);
            let ret_w0: i32 = match &(source_value) {
                perftest_flat::EnumNames::Companion => 4i32,
                perftest_flat::EnumNames::wireValue => 9i32,
                perftest_flat::EnumNames::fromWire => 12i32,
                perftest_flat::EnumNames::r#type => 14i32,
                perftest_flat::EnumNames::when => 16i32,
                perftest_flat::EnumNames::name => 20i32,
                perftest_flat::EnumNames::ordinal => 21i32,
                perftest_flat::EnumNames::entries => 22i32,
                perftest_flat::EnumNames::value => 24i32,
                perftest_flat::EnumNames::kotlin => 25i32,
            };
            let result = perftest_enum_names_echo_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_enum_payload_new_result {
    pub v0: *mut perftest_flat::PayloadHandle,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_enum_payload_new(
    p0_w1: f64,
) -> *mut perftest_flat::PayloadHandle {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_enum_payload_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: f64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::enum_payload_new(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::PayloadHandle = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_enum_payload_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_enum_record_echo_result {
    pub v0: i32,
    pub v1: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_enum_record_echo(
    p0_w2: i32,
    p0_w6: i32,
    out1: *mut i32,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_enum_record_echo_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1 = if true {
                Some(
                    match p0_w2 {
                        0i32 => perftest_flat::Priority::Low,
                        1i32 => perftest_flat::Priority::Normal,
                        2i32 => perftest_flat::Priority::High,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w4: i32 = i32::from(p0_w6 != -3);
            let p0_w5 = if p0_w4 == 1i32 {
                Some(
                    match p0_w6 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w1 = p0_w1.expect("active enum");
            let p0_w3 = if p0_w4 == 1i32 {
                let p0_w5 = p0_w5.expect("active enum");
                Some(p0_w5)
            } else {
                None
            };
            let p0_w0: perftest_flat::EnumRecord = perftest_flat::enum_record_new(
                p0_w1,
                p0_w3,
            );
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::enum_record_echo(__prebindgen_argument_0);
            let ret_w0 = perftest_flat::enum_record_into(source_value);
            let ret_w1 = ret_w0;
            let ret_w2: i32 = match &(ret_w1.priority) {
                perftest_flat::Priority::Low => 0i32,
                perftest_flat::Priority::Normal => 1i32,
                perftest_flat::Priority::High => 2i32,
            };
            let (_, (ret_w3_value0,), ret_w3_commit) = if let Some(ret_w4) = ret_w1.code
            {
                let ret_w5: i32 = match &(ret_w4) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                let ret_w3_values = (ret_w5,);
                (1i32, ret_w3_values, Some(move || {}))
            } else {
                (0i32, (-3,), None)
            };
            let result = perftest_enum_record_echo_result {
                v0: ret_w2,
                v1: ret_w3_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w3_action) = ret_w3_commit {
                    ret_w3_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_getter_child_name_result {
    pub v0: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_child_name(
    p0_w1: *const perftest_flat::GetterChild,
) -> __PbBuffer {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_child_name_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterChild
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterChild = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_child_name(__prebindgen_argument_0);
            let ret_w0 = ::core::convert::Into::<
                Vec<u8>,
            >::into((source_value).to_string())
                .into_boxed_slice();
            let ret_w0 = Box::into_raw(ret_w0);
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_getter_child_name_result {
                v0: __PbBuffer {
                    data: ret_w0 as *const u8,
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_child_number_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_child_number(
    p0_w1: *const perftest_flat::GetterChild,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_child_number_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterChild
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterChild = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_child_number(
                __prebindgen_argument_0,
            );
            let ret_w0: &i64 = source_value;
            let ret_w0 = *ret_w0;
            let result = perftest_getter_child_number_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_child_set_number_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_child_set_number(
    p0_w1: *mut perftest_flat::GetterChild,
    p1_w1: i64,
) {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_child_set_number_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterChild
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = ();
            let p0_w0: &mut perftest_flat::GetterChild = unsafe {
                &mut (*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_child_set_number(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let result = perftest_getter_child_set_number_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_getter_counts_ret_w0 {
    pub v0: usize,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_counts_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_getter_counts_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_getter_counts_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_counts() -> __PbRows {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_getter_counts_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::getter_counts();
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let mut ret_w0: Vec<__PbRow_getter_counts_ret_w0> = Vec::with_capacity(
                ret_w2,
            );
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = ret_w1;
                ret_w0
                    .push(__PbRow_getter_counts_ret_w0 {
                        v0: ret_w4,
                    });
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = ret_w0.into_boxed_slice();
            let result = perftest_getter_counts_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                for commit in ret_w0_commits {
                    commit();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_fields_new_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: i64,
    pub v2: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_fields_new(
    out1: *mut i64,
    out2: *mut __PbBuffer,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_fields_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::getter_fields_new();
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_details(unsafe { &*(ret_w0) });
            let ret_w2 = &(ret_w1);
            let ret_w3: &i64 = &ret_w2.number;
            let ret_w3 = *ret_w3;
            let ret_w4 = ::core::convert::Into::<
                Vec<u8>,
            >::into(::core::clone::Clone::clone(&ret_w2.text))
                .into_boxed_slice();
            let ret_w4 = Box::into_raw(ret_w4);
            let ret_w4_owner = __PbOutputOwner(ret_w4);
            let result = perftest_getter_fields_new_result {
                v0: ret_w0,
                v1: ret_w3,
                v2: __PbBuffer {
                    data: ret_w4 as *const u8,
                    len: ret_w4.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
                ::core::mem::forget(ret_w4_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    value.v0
}
#[repr(C)]
pub struct perftest_getter_leaf_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_leaf_id(
    p0_w1: *const perftest_flat::GetterLeaf,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_getter_leaf_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterLeaf
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterLeaf = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_leaf_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_getter_leaf_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_child_result {
    pub v0: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_child(
    p0_w1: *const perftest_flat::GetterOwner,
) -> *const perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_child_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_child(
                __prebindgen_argument_0,
            );
            let ret_w0: *const perftest_flat::GetterChild = source_value
                as *const perftest_flat::GetterChild;
            let result = perftest_getter_owner_child_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_child_matches_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_child_matches(
    p0_w1: *const perftest_flat::GetterOwner,
    p1_w1: *const perftest_flat::GetterChild,
) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_child_matches_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w1_pointer = if true {
                p1_w1 as *mut perftest_flat::GetterChild
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p1_w1_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p1_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: &perftest_flat::GetterChild = unsafe { &(*p1_w1_pointer) };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_owner_child_matches(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = u8::from(source_value);
            let result = perftest_getter_owner_child_matches_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_child_mut_result {
    pub v0: *mut perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_child_mut(
    p0_w1: *mut perftest_flat::GetterOwner,
) -> *mut perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_child_mut_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &mut perftest_flat::GetterOwner = unsafe {
                &mut (*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_child_mut(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterChild = source_value
                as *mut perftest_flat::GetterChild;
            let result = perftest_getter_owner_child_mut_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_id(
    p0_w1: *const perftest_flat::GetterOwner,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_getter_owner_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_getter_owner_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_new_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: i64,
    pub v2: __PbBuffer,
    pub v3: *mut perftest_flat::GetterLeaf,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_new(
    p0_w1: u8,
    out1: *mut i64,
    out2: *mut __PbBuffer,
    out3: *mut *mut perftest_flat::GetterLeaf,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        if out3.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_new(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_child(unsafe { &*(ret_w0) });
            let ret_w2 = perftest_flat::getter_child_number(ret_w1);
            let ret_w3: &i64 = ret_w2;
            let ret_w3 = *ret_w3;
            let ret_w4 = perftest_flat::getter_child_name(ret_w1);
            let ret_w5 = ::core::convert::Into::<Vec<u8>>::into((ret_w4).to_string())
                .into_boxed_slice();
            let ret_w5 = Box::into_raw(ret_w5);
            let ret_w5_owner = __PbOutputOwner(ret_w5);
            let ret_w6 = perftest_flat::getter_owner_leaf(unsafe { &*(ret_w0) });
            let ret_w7: *mut perftest_flat::GetterLeaf = Box::into_raw(
                Box::new(::core::clone::Clone::clone(ret_w6)),
            );
            let ret_w7_owner = __PbOutputOwner(ret_w7);
            let result = perftest_getter_owner_new_result {
                v0: ret_w0,
                v1: ret_w3,
                v2: __PbBuffer {
                    data: ret_w5 as *const u8,
                    len: ret_w5.len(),
                },
                v3: ret_w7,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
                ::core::mem::forget(ret_w5_owner);
                ::core::mem::forget(ret_w7_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    std::ptr::write(out3, value.v3);
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_optional_result {
    pub v0: i32,
    pub v1: i64,
    pub v2: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_optional(
    p0_w1: *const perftest_flat::GetterOwner,
    out1: *mut i64,
    out2: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0, ret_w0_value1), ret_w0_commit) = if let Some(
                ret_w1,
            ) = source_value {
                let ret_w2 = perftest_flat::getter_child_number(ret_w1);
                let ret_w3: &i64 = ret_w2;
                let ret_w3 = *ret_w3;
                let ret_w4 = perftest_flat::getter_child_name(ret_w1);
                let ret_w5 = ::core::convert::Into::<Vec<u8>>::into((ret_w4).to_string())
                    .into_boxed_slice();
                let ret_w5 = Box::into_raw(ret_w5);
                let ret_w5_owner = __PbOutputOwner(ret_w5);
                let ret_w0_values = (
                    ret_w3,
                    __PbBuffer {
                        data: ret_w5 as *const u8,
                        len: ret_w5.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w5_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        0,
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_getter_owner_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
                v2: ret_w0_value1,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_optional_mut_result {
    pub v0: *mut perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_optional_mut(
    p0_w1: *mut perftest_flat::GetterOwner,
) -> *mut perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_optional_mut_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &mut perftest_flat::GetterOwner = unsafe {
                &mut (*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_optional_mut(
                __prebindgen_argument_0,
            );
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: *mut perftest_flat::GetterChild = ret_w1
                    as *mut perftest_flat::GetterChild;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_getter_owner_optional_mut_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_optional_pointer_result {
    pub v0: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_optional_pointer(
    p0_w1: *const perftest_flat::GetterOwner,
) -> *const perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_optional_pointer_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_optional_pointer(
                __prebindgen_argument_0,
            );
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: *const perftest_flat::GetterChild = ret_w1
                    as *const perftest_flat::GetterChild;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null(),), None)
            };
            let result = perftest_getter_owner_optional_pointer_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_read_result {
    pub v0: i64,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_read(
    p0_w1: *const perftest_flat::GetterOwner,
    out1: *mut __PbBuffer,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_read_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::getter_owner_read(__prebindgen_argument_0);
            let ret_w0 = perftest_flat::getter_child_number(source_value);
            let ret_w1: &i64 = ret_w0;
            let ret_w1 = *ret_w1;
            let ret_w2 = perftest_flat::getter_child_name(source_value);
            let ret_w3 = ::core::convert::Into::<Vec<u8>>::into((ret_w2).to_string())
                .into_boxed_slice();
            let ret_w3 = Box::into_raw(ret_w3);
            let ret_w3_owner = __PbOutputOwner(ret_w3);
            let result = perftest_getter_owner_read_result {
                v0: ret_w1,
                v1: __PbBuffer {
                    data: ret_w3 as *const u8,
                    len: ret_w3.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w3_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_getter_owner_try_child_result {
    pub v0: *const perftest_flat::GetterChild,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_try_child(
    p0_w1: *const perftest_flat::GetterOwner,
    p1_w1: u8,
    error: *mut *mut std::ffi::c_char,
) -> *const perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_try_child_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_owner_try_child(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *const perftest_flat::GetterChild = ret_w0
                as *const perftest_flat::GetterChild;
            let result = perftest_getter_owner_try_child_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null()
        }
    }
}
#[repr(C)]
pub struct perftest_getter_owner_try_child_mut_result {
    pub v0: *mut perftest_flat::GetterChild,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_try_child_mut(
    p0_w1: *mut perftest_flat::GetterOwner,
    p1_w1: u8,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_try_child_mut_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1 != 0) } else { None };
            let _: () = ();
            let p0_w0: &mut perftest_flat::GetterOwner = unsafe {
                &mut (*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: bool = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_owner_try_child_mut(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::GetterChild = ret_w0
                as *mut perftest_flat::GetterChild;
            let result = perftest_getter_owner_try_child_mut_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_getter_owner_try_optional_child_result {
    pub v0: *const perftest_flat::GetterChild,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_try_optional_child(
    p0_w1: *const perftest_flat::GetterOwner,
    p1_w1: i32,
    out0: *mut *const perftest_flat::GetterChild,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_try_optional_child_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = ();
            let p0_w0: &perftest_flat::GetterOwner = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_owner_try_optional_child(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4: *const perftest_flat::GetterChild = ret_w3
                    as *const perftest_flat::GetterChild;
                let ret_w2_values = (ret_w4,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null(),), None)
            };
            let result = perftest_getter_owner_try_optional_child_result {
                v0: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_getter_owner_try_optional_mut_result {
    pub v0: *mut perftest_flat::GetterChild,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_owner_try_optional_mut(
    p0_w1: *mut perftest_flat::GetterOwner,
    p1_w1: i32,
    out0: *mut *mut perftest_flat::GetterChild,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_owner_try_optional_mut_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::GetterOwner
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = ();
            let p0_w0: &mut perftest_flat::GetterOwner = unsafe {
                &mut (*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::getter_owner_try_optional_mut(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4: *mut perftest_flat::GetterChild = ret_w3
                    as *mut perftest_flat::GetterChild;
                let ret_w2_values = (ret_w4,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_getter_owner_try_optional_mut_result {
                v0: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_getter_static_child_result {
    pub v0: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_static_child(
    p0_w1: __PbBuffer,
) -> *const perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_static_child_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    ::core::convert::Into::<
                        String,
                    >::into(
                        std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                            .map_err(|_| "invalid UTF-8")?,
                    ),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.as_ref().expect("active value");
            let source_value = perftest_flat::getter_static_child(
                __prebindgen_argument_0,
            );
            let ret_w0: *const perftest_flat::GetterChild = source_value
                as *const perftest_flat::GetterChild;
            let result = perftest_getter_static_child_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_getter_two_clones_new_result {
    pub v0: *mut perftest_flat::GetterLeaf,
    pub v1: *mut perftest_flat::GetterLeaf,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_getter_two_clones_new(
    out1: *mut *mut perftest_flat::GetterLeaf,
) -> *mut perftest_flat::GetterLeaf {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_getter_two_clones_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::getter_two_clones_new();
            let ret_w0 = perftest_flat::getter_owner_leaf(&source_value);
            let ret_w1: *mut perftest_flat::GetterLeaf = Box::into_raw(
                Box::new(::core::clone::Clone::clone(ret_w0)),
            );
            let ret_w1_owner = __PbOutputOwner(ret_w1);
            let ret_w2: *mut perftest_flat::GetterLeaf = Box::into_raw(
                Box::new(::core::clone::Clone::clone(ret_w0)),
            );
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let result = perftest_getter_two_clones_new_result {
                v0: ret_w1,
                v1: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w1_owner);
                ::core::mem::forget(ret_w2_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_niche_choice_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_niche_choice_id(
    p0_w1: i32,
    p0_w3: i32,
    p0_w8: u32,
    p0_w10: *mut perftest_flat::NicheChoiceHandle,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_niche_choice_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            if (p0_w1 == 1i32) && (!(0..2i32).contains(&p0_w3)) {
                return Err("invalid selector".into());
            }
            let p0_w6: i32 = i32::from(p0_w8 != 1114112);
            let p0_w7 = if ((p0_w1 == 1i32) && (p0_w3 == 0i32)) && (p0_w6 == 1i32) {
                Some(char::from_u32(p0_w8).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w10_pointer = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                p0_w10 as *mut perftest_flat::NicheChoiceHandle
            } else {
                ::core::ptr::null_mut()
            };
            if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                ::prebindgen_runtime::validate_pointer(p0_w10_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w9 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(unsafe { Box::from_raw(p0_w10_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: perftest_flat::NicheChoiceHandle = match p0_w3 {
                    0i32 => {
                        let p0_w5 = if ((p0_w1 == 1i32) && (p0_w3 == 0i32))
                            && (p0_w6 == 1i32)
                        {
                            let p0_w7: char = ::core::convert::Into::into(
                                p0_w7.expect("active value"),
                            );
                            Some(p0_w7)
                        } else {
                            None
                        };
                        let p0_w4: perftest_flat::NicheChoiceHandle = perftest_flat::niche_choice_new(
                            p0_w5,
                        );
                        p0_w4
                    }
                    1i32 => {
                        let p0_w9: perftest_flat::NicheChoiceHandle = *p0_w9
                            .expect("active allocation");
                        p0_w9
                    }
                    _ => unreachable!("validated selector"),
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::niche_choice_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_niche_choice_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_niche_choice_new_result {
    pub v0: *mut perftest_flat::NicheChoiceHandle,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_niche_choice_new(
    p0_w3: u32,
) -> *mut perftest_flat::NicheChoiceHandle {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_niche_choice_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w3 != 1114112);
            let p0_w2 = if p0_w1 == 1i32 {
                Some(char::from_u32(p0_w3).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: char = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::niche_choice_new(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::NicheChoiceHandle = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_niche_choice_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_bool_result {
    pub v0: i32,
    pub v1: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_bool(
    p0_w1: i32,
    p0_w3: u8,
    out1: *mut u8,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_bool_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 { Some(p0_w3 != 0) } else { None };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: bool = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_bool(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = u8::from(ret_w1);
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_bool_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_bytes_result {
    pub v0: i32,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_bytes(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_bytes_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(unsafe { __pb_read(p0_w3)? })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: Vec<u8> = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_bytes(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w2 = Box::into_raw(ret_w2);
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (
                    __PbBuffer {
                        data: ret_w2 as *const u8,
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_option_bytes_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_char_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_char(p0_w3: u32) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_char_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w3 != 1114112);
            let p0_w2 = if p0_w1 == 1i32 {
                Some(char::from_u32(p0_w3).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: char = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_char(__prebindgen_argument_0);
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = (ret_w1) as u32;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (1114112,), None)
            };
            let result = perftest_option_char_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_choice_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_choice_id(
    p0_w1: i32,
    p0_w3: i32,
    p0_w6: i64,
    p0_w8: *mut perftest_flat::OptionChoice,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_choice_id_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            if (p0_w1 == 1i32) && (!(0..2i32).contains(&p0_w3)) {
                return Err("invalid selector".into());
            }
            let p0_w5 = if (p0_w1 == 1i32) && (p0_w3 == 0i32) {
                Some(p0_w6)
            } else {
                None
            };
            let p0_w8_pointer = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                p0_w8 as *mut perftest_flat::OptionChoice
            } else {
                ::core::ptr::null_mut()
            };
            if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                ::prebindgen_runtime::validate_pointer(p0_w8_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w7 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(unsafe { Box::from_raw(p0_w8_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: perftest_flat::OptionChoice = match p0_w3 {
                    0i32 => {
                        let p0_w5: i64 = ::core::convert::Into::into(
                            p0_w5.expect("active value"),
                        );
                        let p0_w4: perftest_flat::OptionChoice = perftest_flat::option_choice_new(
                            p0_w5,
                        );
                        p0_w4
                    }
                    1i32 => {
                        let p0_w7: perftest_flat::OptionChoice = *p0_w7
                            .expect("active allocation");
                        p0_w7
                    }
                    _ => unreachable!("validated selector"),
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_choice_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_option_choice_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_ctor_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_ctor_id(
    p0_w1: i32,
    p0_w4: i32,
    p0_w6: i64,
    p0_w8: *mut perftest_flat::OptionCtor,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_ctor_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            if (p0_w1 == 0i32) && (p0_w4 != 0 && p0_w4 != 1) {
                return Err("invalid option presence".into());
            }
            let p0_w5 = if (p0_w1 == 0i32) && (p0_w4 == 1i32) {
                Some(p0_w6)
            } else {
                None
            };
            let p0_w8_pointer = if p0_w1 == 1i32 {
                p0_w8 as *mut perftest_flat::OptionCtor
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w8_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w7 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w8_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::OptionCtor = match p0_w1 {
                0i32 => {
                    let p0_w3 = if (p0_w1 == 0i32) && (p0_w4 == 1i32) {
                        let p0_w5: i64 = ::core::convert::Into::into(
                            p0_w5.expect("active value"),
                        );
                        Some(p0_w5)
                    } else {
                        None
                    };
                    let p0_w2: perftest_flat::OptionCtor = perftest_flat::option_ctor_new(
                        p0_w3,
                    );
                    p0_w2
                }
                1i32 => {
                    let p0_w7: perftest_flat::OptionCtor = *p0_w7
                        .expect("active allocation");
                    p0_w7
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_ctor_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_option_ctor_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_ctor_new_result {
    pub v0: *mut perftest_flat::OptionCtor,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_ctor_new(
    p0_w1: i32,
    p0_w3: i64,
) -> *mut perftest_flat::OptionCtor {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_ctor_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 { Some(p0_w3) } else { None };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: i64 = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_ctor_new(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::OptionCtor = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_option_ctor_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_i64_result {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_i64(
    p0_w1: i32,
    p0_w3: i64,
    out1: *mut i64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_i64_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 { Some(p0_w3) } else { None };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: i64 = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_i64(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_i64_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_i64_ref_result {
    pub v0: i32,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_i64_ref(
    p0_w1: i32,
    p0_w3: i64,
    out1: *mut i64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_i64_ref_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(::core::convert::Into::<i64>::into(p0_w3))
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(p0_w2.as_ref().expect("active value"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_i64_ref(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_i64_ref_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_nested_result {
    pub v0: i32,
    pub v1: i32,
    pub v2: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_nested(
    p0_w1: i32,
    p0_w3: i32,
    p0_w5: i64,
    out1: *mut i32,
    out2: *mut i64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_nested_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            if (p0_w1 == 1i32) && (p0_w3 != 0 && p0_w3 != 1) {
                return Err("invalid option presence".into());
            }
            let p0_w4 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(p0_w5)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                    let p0_w4: i64 = ::core::convert::Into::into(
                        p0_w4.expect("active value"),
                    );
                    Some(p0_w4)
                } else {
                    None
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_nested(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0, ret_w0_value1), ret_w0_commit) = if let Some(
                ret_w1,
            ) = source_value {
                let (ret_w2, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                    let ret_w4 = ret_w3;
                    let ret_w2_values = (ret_w4,);
                    (1i32, ret_w2_values, Some(move || {}))
                } else {
                    (0i32, (0,), None)
                };
                let ret_w0_values = (ret_w2, ret_w2_value0);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w2_action) = ret_w2_commit {
                            ret_w2_action();
                        }
                    }),
                )
            } else {
                (0i32, (0, 0), None)
            };
            let result = perftest_option_nested_result {
                v0: ret_w0,
                v1: ret_w0_value0,
                v2: ret_w0_value1,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    value.v0
}
#[repr(C)]
pub struct perftest_option_nested_char_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_nested_char(p0_w5: u32) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_nested_char_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w5 != 1114113);
            let p0_w3: i32 = i32::from(p0_w5 != 1114112);
            let p0_w4 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(char::from_u32(p0_w5).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                    let p0_w4: char = ::core::convert::Into::into(
                        p0_w4.expect("active value"),
                    );
                    Some(p0_w4)
                } else {
                    None
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_nested_char(
                __prebindgen_argument_0,
            );
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                    let ret_w4 = (ret_w3) as u32;
                    let ret_w2_values = (ret_w4,);
                    (1i32, ret_w2_values, Some(move || {}))
                } else {
                    (0i32, (1114112,), None)
                };
                let ret_w0_values = (ret_w2_value0,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w2_action) = ret_w2_commit {
                            ret_w2_action();
                        }
                    }),
                )
            } else {
                (0i32, (1114113,), None)
            };
            let result = perftest_option_nested_char_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_nested_resource_result {
    pub v0: i32,
    pub v1: *mut perftest_flat::OwnedResource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_nested_resource(
    p0_w1: i32,
    p0_w5: *mut perftest_flat::OwnedResource,
    out1: *mut *mut perftest_flat::OwnedResource,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_nested_resource_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w3: i32 = i32::from((p0_w5 as usize) != 0);
            let p0_w5_pointer = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                p0_w5 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w4 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(unsafe { Box::from_raw(p0_w5_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                    let p0_w4: perftest_flat::OwnedResource = *p0_w4
                        .expect("active allocation");
                    Some(p0_w4)
                } else {
                    None
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_nested_resource(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                    let ret_w4: *mut perftest_flat::OwnedResource = Box::into_raw(
                        Box::new(ret_w3),
                    );
                    let ret_w4_owner = __PbOutputOwner(ret_w4);
                    let ret_w2_values = (ret_w4,);
                    (
                        1i32,
                        ret_w2_values,
                        Some(move || {
                            ::core::mem::forget(ret_w4_owner);
                        }),
                    )
                } else {
                    (0i32, (std::ptr::null_mut(),), None)
                };
                let ret_w0_values = (ret_w2_value0,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w2_action) = ret_w2_commit {
                            ret_w2_action();
                        }
                    }),
                )
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_option_nested_resource_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_nested_u32_result {
    pub v0: i32,
    pub v1: i32,
    pub v2: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_nested_u32(
    p0_w1: i32,
    p0_w3: i32,
    p0_w5: u32,
    out1: *mut i32,
    out2: *mut u32,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_nested_u32_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            if (p0_w1 == 1i32) && (p0_w3 != 0 && p0_w3 != 1) {
                return Err("invalid option presence".into());
            }
            let p0_w4 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(p0_w5)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                    let p0_w4: u32 = ::core::convert::Into::into(
                        p0_w4.expect("active value"),
                    );
                    Some(p0_w4)
                } else {
                    None
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_nested_u32(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0, ret_w0_value1), ret_w0_commit) = if let Some(
                ret_w1,
            ) = source_value {
                let (ret_w2, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                    let ret_w4 = ret_w3;
                    let ret_w2_values = (ret_w4,);
                    (1i32, ret_w2_values, Some(move || {}))
                } else {
                    (0i32, (0,), None)
                };
                let ret_w0_values = (ret_w2, ret_w2_value0);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w2_action) = ret_w2_commit {
                            ret_w2_action();
                        }
                    }),
                )
            } else {
                (0i32, (0, 0), None)
            };
            let result = perftest_option_nested_u32_result {
                v0: ret_w0,
                v1: ret_w0_value0,
                v2: ret_w0_value1,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    value.v0
}
#[repr(C)]
pub struct perftest_option_resource_echo_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_resource_echo(
    p0_w3: *mut perftest_flat::OwnedResource,
    p1_w1: u32,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_resource_echo_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from((p0_w3 as usize) != 0);
            let p0_w3_pointer = if p0_w1 == 1i32 {
                p0_w3 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w3_pointer)?;
            }
            let p1_w0 = if true {
                Some(char::from_u32(p1_w1).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w2 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w3_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: perftest_flat::OwnedResource = *p0_w2
                    .expect("active allocation");
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: char = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::option_resource_echo(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w1),
                );
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (ret_w2,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_option_resource_echo_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_resource_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_resource_id(
    p0_w3: *const perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_resource_id_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from((p0_w3 as usize) != 0);
            let p0_w3_pointer = if p0_w1 == 1i32 {
                p0_w3 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w3_pointer)?;
            }
            let _: () = ();
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: &perftest_flat::OwnedResource = unsafe { &(*p0_w3_pointer) };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_resource_id(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_option_resource_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_resource_new_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_resource_new(
    p0_w1: u8,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_resource_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_resource_new(
                __prebindgen_argument_0,
            );
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w1),
                );
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (ret_w2,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_option_resource_new_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_resource_pair_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_resource_pair(
    p0_w3: *mut perftest_flat::OwnedResource,
    p1_w3: *mut perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_resource_pair_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from((p0_w3 as usize) != 0);
            let p0_w3_pointer = if p0_w1 == 1i32 {
                p0_w3 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w3_pointer)?;
            }
            let p1_w1: i32 = i32::from((p1_w3 as usize) != 0);
            let p1_w3_pointer = if p1_w1 == 1i32 {
                p1_w3 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p1_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p1_w3_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if p0_w1 == 1i32 {
                    for identity in ::core::iter::once((p0_w3_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if p1_w1 == 1i32 {
                    for identity in ::core::iter::once((p1_w3_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w2 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w3_pointer) })
            } else {
                None
            };
            let p1_w2 = if p1_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p1_w3_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: perftest_flat::OwnedResource = *p0_w2
                    .expect("active allocation");
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0 = if p1_w1 == 1i32 {
                let p1_w2: perftest_flat::OwnedResource = *p1_w2
                    .expect("active allocation");
                Some(p1_w2)
            } else {
                None
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::option_resource_pair(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = perftest_option_resource_pair_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_option_slice_echo_result {
    pub v0: i32,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_slice_echo(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_slice_echo_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(unsafe { __pb_read(p0_w3)? })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(
                    ::core::convert::AsRef::<
                        [u8],
                    >::as_ref(p0_w2.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_slice_echo(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ::core::convert::Into::<Vec<u8>>::into((ret_w1).to_vec())
                    .into_boxed_slice();
                let ret_w2 = Box::into_raw(ret_w2);
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (
                    __PbBuffer {
                        data: ret_w2 as *const u8,
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_option_slice_echo_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_slice_len_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_slice_len(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_slice_len_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(unsafe { __pb_read(p0_w3)? })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(
                    ::core::convert::AsRef::<
                        [u8],
                    >::as_ref(p0_w2.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_slice_len(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_slice_len_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_str_echo_result {
    pub v0: i32,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_str_echo(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_str_echo_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w3)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(
                    ::core::convert::AsRef::<
                        str,
                    >::as_ref(p0_w2.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_str_echo(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ::core::convert::Into::<Vec<u8>>::into((ret_w1).to_string())
                    .into_boxed_slice();
                let ret_w2 = Box::into_raw(ret_w2);
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (
                    __PbBuffer {
                        data: ret_w2 as *const u8,
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_option_str_echo_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_str_len_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_str_len(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_str_len_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w3)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(
                    ::core::convert::AsRef::<
                        str,
                    >::as_ref(p0_w2.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_str_len(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_str_len_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_string_len_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_string_len(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_option_string_len_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    ::core::convert::Into::<
                        String,
                    >::into(
                        std::str::from_utf8(unsafe { __pb_read(p0_w3)? })
                            .map_err(|_| "invalid UTF-8")?,
                    ),
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                Some(p0_w2.as_ref().expect("active value"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_string_len(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_string_len_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_text_result {
    pub v0: i32,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_text(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w3)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: String = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_text(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w2 = Box::into_raw(ret_w2);
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (
                    __PbBuffer {
                        data: ret_w2 as *const u8,
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_option_text_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_u8_result {
    pub v0: i32,
    pub v1: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_u8(
    p0_w1: i32,
    p0_w3: u8,
    out1: *mut u8,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_u8_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 { Some(p0_w3) } else { None };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: u8 = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_u8(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_option_u8_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_option_unit_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_option_unit(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_option_unit_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w0 = if p0_w1 == 1i32 { Some(()) } else { None };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::option_unit(__prebindgen_argument_0);
            let (ret_w0, (), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w0_values = ();
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (), None)
            };
            let result = perftest_option_unit_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_optional_fields_echo_result {
    pub v0: i32,
    pub v1: i64,
    pub v2: i32,
    pub v3: __PbBuffer,
    pub v4: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_optional_fields_echo(
    p0_w4: i32,
    p0_w6: i64,
    p0_w8: i32,
    p0_w10: __PbBuffer,
    p0_w14: u32,
    out1: *mut i64,
    out2: *mut i32,
    out3: *mut __PbBuffer,
    out4: *mut u32,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_optional_fields_echo_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        if out3.is_null() {
            return Err("null success output".into());
        }
        if out4.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1: i32 = i32::from(p0_w14 != 1114113);
            if (p0_w1 == 1i32) && (p0_w4 != 0 && p0_w4 != 1) {
                return Err("invalid option presence".into());
            }
            let p0_w5 = if (p0_w1 == 1i32) && (p0_w4 == 1i32) {
                Some(p0_w6)
            } else {
                None
            };
            if (p0_w1 == 1i32) && (p0_w8 != 0 && p0_w8 != 1) {
                return Err("invalid option presence".into());
            }
            let p0_w9 = if (p0_w1 == 1i32) && (p0_w8 == 1i32) {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w10)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w12: i32 = i32::from(p0_w14 != 1114112);
            let p0_w13 = if (p0_w1 == 1i32) && (p0_w12 == 1i32) {
                Some(char::from_u32(p0_w14).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w3 = if (p0_w1 == 1i32) && (p0_w4 == 1i32) {
                    let p0_w5: i64 = ::core::convert::Into::into(
                        p0_w5.expect("active value"),
                    );
                    Some(p0_w5)
                } else {
                    None
                };
                let p0_w7 = if (p0_w1 == 1i32) && (p0_w8 == 1i32) {
                    let p0_w9: String = ::core::convert::Into::into(
                        p0_w9.expect("active value"),
                    );
                    Some(p0_w9)
                } else {
                    None
                };
                let p0_w11 = if (p0_w1 == 1i32) && (p0_w12 == 1i32) {
                    let p0_w13: char = ::core::convert::Into::into(
                        p0_w13.expect("active value"),
                    );
                    Some(p0_w13)
                } else {
                    None
                };
                let p0_w2: perftest_flat::OptionalFields = perftest_flat::optional_fields_new(
                    p0_w3,
                    p0_w7,
                    p0_w11,
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::optional_fields_echo(
                __prebindgen_argument_0,
            );
            let (
                _,
                (
                    ret_w0_value0,
                    ret_w0_value1,
                    ret_w0_value2,
                    ret_w0_value3,
                    ret_w0_value4,
                ),
                ret_w0_commit,
            ) = if let Some(ret_w1) = source_value {
                let ret_w2 = perftest_flat::optional_fields_into(ret_w1);
                let ret_w3 = ret_w2;
                let (ret_w4, (ret_w4_value0,), ret_w4_commit) = if let Some(ret_w5) = ret_w3
                    .count
                {
                    let ret_w6 = ret_w5;
                    let ret_w4_values = (ret_w6,);
                    (1i32, ret_w4_values, Some(move || {}))
                } else {
                    (0i32, (0,), None)
                };
                let (ret_w7, (ret_w7_value0,), ret_w7_commit) = if let Some(ret_w8) = ret_w3
                    .text
                {
                    let ret_w9 = ::core::convert::Into::<Vec<u8>>::into(ret_w8)
                        .into_boxed_slice();
                    let ret_w9 = Box::into_raw(ret_w9);
                    let ret_w9_owner = __PbOutputOwner(ret_w9);
                    let ret_w7_values = (
                        __PbBuffer {
                            data: ret_w9 as *const u8,
                            len: ret_w9.len(),
                        },
                    );
                    (
                        1i32,
                        ret_w7_values,
                        Some(move || {
                            ::core::mem::forget(ret_w9_owner);
                        }),
                    )
                } else {
                    (
                        0i32,
                        (
                            __PbBuffer {
                                data: std::ptr::null(),
                                len: 0,
                            },
                        ),
                        None,
                    )
                };
                let (_, (ret_w10_value0,), ret_w10_commit) = if let Some(ret_w11) = ret_w3
                    .letter
                {
                    let ret_w12 = (ret_w11) as u32;
                    let ret_w10_values = (ret_w12,);
                    (1i32, ret_w10_values, Some(move || {}))
                } else {
                    (0i32, (1114112,), None)
                };
                let ret_w0_values = (
                    ret_w4,
                    ret_w4_value0,
                    ret_w7,
                    ret_w7_value0,
                    ret_w10_value0,
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w4_action) = ret_w4_commit {
                            ret_w4_action();
                        }
                        if let Some(ret_w7_action) = ret_w7_commit {
                            ret_w7_action();
                        }
                        if let Some(ret_w10_action) = ret_w10_commit {
                            ret_w10_action();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        0,
                        0,
                        0,
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                        1114113,
                    ),
                    None,
                )
            };
            let result = perftest_optional_fields_echo_result {
                v0: ret_w0_value0,
                v1: ret_w0_value1,
                v2: ret_w0_value2,
                v3: ret_w0_value3,
                v4: ret_w0_value4,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    std::ptr::write(out3, value.v3);
    std::ptr::write(out4, value.v4);
    value.v0
}
#[repr(C)]
pub struct perftest_owned_resource_drops_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owned_resource_drops() -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owned_resource_drops_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::owned_resource_drops();
            let ret_w0 = source_value;
            let result = perftest_owned_resource_drops_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_checked_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
    pub v2: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_checked(
    p0_w1: u8,
    out1: *mut *const perftest_flat::GetterChild,
    out2: *mut i64,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_checked_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        if out2.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_checked(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(ret_w0),
            );
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let ret_w3 = perftest_flat::getter_owner_child(unsafe { &*(ret_w2) });
            let ret_w4: *const perftest_flat::GetterChild = ret_w3
                as *const perftest_flat::GetterChild;
            let ret_w5 = perftest_flat::owner_view_check(unsafe { &*(ret_w2) });
            let ret_w6 = ret_w5;
            let result = perftest_owner_view_checked_result {
                v0: ret_w2,
                v1: ret_w4,
                v2: ret_w6,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out1, value.v1);
            std::ptr::write(out2, value.v2);
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw((value.v0) as *mut perftest_flat::GetterOwner),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_owner_view_from_temporary_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_from_temporary(
    p0_w1: __PbBuffer,
    out1: *mut *const perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_from_temporary_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true {
                Some(
                    ::core::convert::Into::<
                        String,
                    >::into(
                        std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                            .map_err(|_| "invalid UTF-8")?,
                    ),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.as_ref().expect("active value");
            let source_value = perftest_flat::owner_view_from_temporary(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_child(unsafe { &*(ret_w0) });
            let ret_w2: *const perftest_flat::GetterChild = ret_w1
                as *const perftest_flat::GetterChild;
            let result = perftest_owner_view_from_temporary_result {
                v0: ret_w0,
                v1: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_maybe_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_maybe(
    p0_w1: u8,
    out1: *mut *const perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_maybe_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_maybe(__prebindgen_argument_0);
            let (_, (ret_w0_value0, ret_w0_value1), ret_w0_commit) = if let Some(
                ret_w1,
            ) = source_value {
                let ret_w2: *mut perftest_flat::GetterOwner = Box::into_raw(
                    Box::new(ret_w1),
                );
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w3 = perftest_flat::getter_owner_child(unsafe { &*(ret_w2) });
                let ret_w4: *const perftest_flat::GetterChild = ret_w3
                    as *const perftest_flat::GetterChild;
                let ret_w0_values = (ret_w2, ret_w4);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (0i32, (std::ptr::null_mut(), std::ptr::null()), None)
            };
            let result = perftest_owner_view_maybe_result {
                v0: ret_w0_value0,
                v1: ret_w0_value1,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_mutable_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *mut perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_mutable(
    p0_w1: u8,
    out1: *mut *mut perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_mutable_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_mutable(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_child_mut(unsafe {
                &mut *(ret_w0)
            });
            let ret_w2: *mut perftest_flat::GetterChild = ret_w1
                as *mut perftest_flat::GetterChild;
            let result = perftest_owner_view_mutable_result {
                v0: ret_w0,
                v1: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_optional_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_optional(
    p0_w1: u8,
    out1: *mut *const perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_optional(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_optional(unsafe { &*(ret_w0) });
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                let ret_w4: *const perftest_flat::GetterChild = ret_w3
                    as *const perftest_flat::GetterChild;
                let ret_w2_values = (ret_w4,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null(),), None)
            };
            let result = perftest_owner_view_optional_result {
                v0: ret_w0,
                v1: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_optional_mut_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *mut perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_optional_mut(
    p0_w1: u8,
    out1: *mut *mut perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_optional_mut_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_optional_mut(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_optional_mut(unsafe {
                &mut *(ret_w0)
            });
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                let ret_w4: *mut perftest_flat::GetterChild = ret_w3
                    as *mut perftest_flat::GetterChild;
                let ret_w2_values = (ret_w4,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (std::ptr::null_mut(),), None)
            };
            let result = perftest_owner_view_optional_mut_result {
                v0: ret_w0,
                v1: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_reordered_result {
    pub v0: *const perftest_flat::GetterChild,
    pub v1: *mut perftest_flat::GetterOwner,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_reordered(
    p0_w1: u8,
    out1: *mut *mut perftest_flat::GetterOwner,
) -> *const perftest_flat::GetterChild {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_reordered_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_reordered(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_child(unsafe { &*(ret_w0) });
            let ret_w2: *const perftest_flat::GetterChild = ret_w1
                as *const perftest_flat::GetterChild;
            let result = perftest_owner_view_reordered_result {
                v0: ret_w2,
                v1: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_owner_view_rollback_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
    pub v2: *mut perftest_flat::OwnedResource,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_rollback(
    out1: *mut *const perftest_flat::GetterChild,
    out2: *mut *mut perftest_flat::OwnedResource,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_rollback_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        if out2.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let source_value = perftest_flat::owner_view_rollback();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::callback_return_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(ret_w0),
            );
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let ret_w3 = perftest_flat::getter_owner_child(unsafe { &*(ret_w2) });
            let ret_w4: *const perftest_flat::GetterChild = ret_w3
                as *const perftest_flat::GetterChild;
            let ret_w5 = perftest_flat::owner_view_late(unsafe { &*(ret_w2) });
            let ret_w6 = perftest_flat::c_panic_output_value(&ret_w5);
            let ret_w7: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(ret_w6),
            );
            let ret_w7_owner = __PbOutputOwner(ret_w7);
            let result = perftest_owner_view_rollback_result {
                v0: ret_w2,
                v1: ret_w4,
                v2: ret_w7,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                ::core::mem::forget(ret_w7_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out1, value.v1);
            std::ptr::write(out2, value.v2);
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw((value.v0) as *mut perftest_flat::GetterOwner),
                        );
                        Ok(())
                    });
                }
                if !(value.v2).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw(
                                (value.v2) as *mut perftest_flat::OwnedResource,
                            ),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_owner_view_shared_result {
    pub v0: *mut perftest_flat::GetterOwner,
    pub v1: *const perftest_flat::GetterChild,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_owner_view_shared(
    p0_w1: u8,
    out1: *mut *const perftest_flat::GetterChild,
) -> *mut perftest_flat::GetterOwner {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_owner_view_shared_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owner_view_shared(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::GetterOwner = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::getter_owner_child(unsafe { &*(ret_w0) });
            let ret_w2: *const perftest_flat::GetterChild = ret_w1
                as *const perftest_flat::GetterChild;
            let result = perftest_owner_view_shared_result {
                v0: ret_w0,
                v1: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_payload_new_result {
    pub v0: i64,
    pub v1: i32,
    pub v2: f64,
    pub v3: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_payload_new(
    p0_w1: i64,
    p1_w1: i32,
    p2_w1: f64,
    p3_w1: u8,
    out1: *mut i32,
    out2: *mut f64,
    out3: *mut u8,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_payload_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        if out3.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let p3_w0 = if true { Some(p3_w1 != 0) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: f64 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let p3_w0: bool = ::core::convert::Into::into(p3_w0.expect("active value"));
            let __prebindgen_argument_3 = p3_w0;
            let source_value = perftest_flat::payload_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
                __prebindgen_argument_3,
            );
            let ret_w0 = perftest_flat::payload_to_struct(&source_value);
            let ret_w1 = &(ret_w0);
            let ret_w2: &i64 = &ret_w1.id;
            let ret_w2 = *ret_w2;
            let ret_w3: &i32 = &ret_w1.seq;
            let ret_w3 = *ret_w3;
            let ret_w4: &f64 = &ret_w1.value;
            let ret_w4 = *ret_w4;
            let ret_w5: &bool = &ret_w1.flag;
            let ret_w5 = u8::from(*ret_w5);
            let result = perftest_payload_new_result {
                v0: ret_w2,
                v1: ret_w3,
                v2: ret_w4,
                v3: ret_w5,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    std::ptr::write(out3, value.v3);
    value.v0
}
#[repr(C)]
pub struct perftest_payload_priority_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_payload_priority(
    p0_w1: *const perftest_flat::PayloadHandle,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_payload_priority_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::PayloadHandle
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::PayloadHandle = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::payload_priority(__prebindgen_argument_0);
            let ret_w0: i32 = match &(source_value) {
                perftest_flat::Priority::Low => 0i32,
                perftest_flat::Priority::Normal => 1i32,
                perftest_flat::Priority::High => 2i32,
            };
            let result = perftest_payload_priority_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_plain_note_echo_result {
    pub v0: i32,
    pub v1: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_plain_note_echo(
    p0_w1: i32,
    p0_w3: __PbBuffer,
    out1: *mut __PbBuffer,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_plain_note_echo_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w3)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: String = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::plain_note_echo(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w1)
                    .into_boxed_slice();
                let ret_w2 = Box::into_raw(ret_w2);
                let ret_w2_owner = __PbOutputOwner(ret_w2);
                let ret_w0_values = (
                    __PbBuffer {
                        data: ret_w2 as *const u8,
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2_owner);
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbBuffer {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_plain_note_echo_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_bool_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_bool(p0_w1: u8) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_bool_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_bool(__prebindgen_argument_0);
            let ret_w0 = u8::from(source_value);
            let result = perftest_primitive_bool_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_char_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_char(p0_w1: u32) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_char_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(char::from_u32(p0_w1).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let p0_w0: char = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_char(__prebindgen_argument_0);
            let ret_w0 = (source_value) as u32;
            let result = perftest_primitive_char_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_f32_result {
    pub v0: f32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_f32(p0_w1: f32) -> f32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_f32_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: f32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_f32(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_f32_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_f64_result {
    pub v0: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_f64(p0_w1: f64) -> f64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_f64_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: f64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_f64(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_f64_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_i128_result {
    pub v0: __Pb128,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_i128(p0_w1: __Pb128) -> __Pb128 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_i128_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(((u128::from(p0_w1.hi) << 64) | u128::from(p0_w1.lo)) as i128)
            } else {
                None
            };
            let p0_w0: i128 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_i128(__prebindgen_argument_0);
            let ret_w0 = {
                let bits = (source_value) as u128;
                __Pb128 {
                    hi: (bits >> 64) as u64,
                    lo: bits as u64,
                }
            };
            let result = perftest_primitive_i128_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_i16_result {
    pub v0: i16,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_i16(p0_w1: i16) -> i16 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_i16_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i16 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_i16(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_i16_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_i32_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_i32(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_i32_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_i32(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_i32_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_i64_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_i64(p0_w1: i64) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_i64_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_i64(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_i64_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_i8_result {
    pub v0: i8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_i8(p0_w1: i8) -> i8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_i8_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i8 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_i8(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_i8_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_isize_result {
    pub v0: isize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_isize(p0_w1: isize) -> isize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_isize_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: isize = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_isize(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_isize_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_u128_result {
    pub v0: __Pb128,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_u128(p0_w1: __Pb128) -> __Pb128 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_u128_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(((u128::from(p0_w1.hi) << 64) | u128::from(p0_w1.lo)) as u128)
            } else {
                None
            };
            let p0_w0: u128 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_u128(__prebindgen_argument_0);
            let ret_w0 = {
                let bits = (source_value) as u128;
                __Pb128 {
                    hi: (bits >> 64) as u64,
                    lo: bits as u64,
                }
            };
            let result = perftest_primitive_u128_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_u16_result {
    pub v0: u16,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_u16(p0_w1: u16) -> u16 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_u16_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: u16 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_u16(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_u16_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_u32_result {
    pub v0: u32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_u32(p0_w1: u32) -> u32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_u32_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: u32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_u32(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_u32_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_u64_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_u64(p0_w1: u64) -> u64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_u64_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: u64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_u64(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_u64_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_u8_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_u8(p0_w1: u8) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_u8_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: u8 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_u8(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_u8_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_primitive_unit_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_unit() {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_unit_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::primitive_unit();
            let result = perftest_primitive_unit_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct perftest_primitive_usize_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_primitive_usize(p0_w1: usize) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_primitive_usize_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: usize = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::primitive_usize(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_primitive_usize_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_priority_or_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_priority_or(p0_w3: i32, p1_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_priority_or_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w3 != -1);
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    match p0_w3 {
                        0i32 => perftest_flat::Priority::Low,
                        1i32 => perftest_flat::Priority::Normal,
                        2i32 => perftest_flat::Priority::High,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p1_w0 = if true {
                Some(
                    match p1_w1 {
                        0i32 => perftest_flat::Priority::Low,
                        1i32 => perftest_flat::Priority::Normal,
                        2i32 => perftest_flat::Priority::High,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = p0_w2.expect("active enum");
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0 = p1_w0.expect("active enum");
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::priority_or(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0: i32 = match &(source_value) {
                perftest_flat::Priority::Low => 0i32,
                perftest_flat::Priority::Normal => 1i32,
                perftest_flat::Priority::High => 2i32,
            };
            let result = perftest_priority_or_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_priority_weight_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_priority_weight(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_priority_weight_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    match p0_w1 {
                        0i32 => perftest_flat::Priority::Low,
                        1i32 => perftest_flat::Priority::Normal,
                        2i32 => perftest_flat::Priority::High,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = p0_w0.expect("active enum");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::priority_weight(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_priority_weight_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_report_into_struct_result {
    pub v0: *mut perftest_flat::ReportStruct,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_report_into_struct(
    p0_w1: i32,
    p0_w3: *mut perftest_flat::Report,
    p0_w6: i64,
    p0_w8: f64,
    p0_w10: __PbBuffer,
) -> *mut perftest_flat::ReportStruct {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_report_into_struct_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            let p0_w3_pointer = if p0_w1 == 0i32 {
                p0_w3 as *mut perftest_flat::Report
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 0i32 {
                ::prebindgen_runtime::validate_pointer(p0_w3_pointer)?;
            }
            let p0_w5 = if p0_w1 == 1i32 { Some(p0_w6) } else { None };
            let p0_w7 = if p0_w1 == 1i32 { Some(p0_w8) } else { None };
            let p0_w9 = if p0_w1 == 1i32 {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w10)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w2 = if p0_w1 == 0i32 {
                Some(unsafe { Box::from_raw(p0_w3_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::Report = match p0_w1 {
                0i32 => {
                    let p0_w2: perftest_flat::Report = *p0_w2
                        .expect("active allocation");
                    p0_w2
                }
                1i32 => {
                    let p0_w5: i64 = ::core::convert::Into::into(
                        p0_w5.expect("active value"),
                    );
                    let p0_w7: f64 = ::core::convert::Into::into(
                        p0_w7.expect("active value"),
                    );
                    let p0_w9: String = ::core::convert::Into::into(
                        p0_w9.expect("active value"),
                    );
                    let p0_w4: perftest_flat::Report = perftest_flat::report_new(
                        p0_w5,
                        p0_w7,
                        p0_w9,
                    );
                    p0_w4
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::report_into_struct(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::ReportStruct = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_report_into_struct_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_report_label_result {
    pub v0: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_report_label(
    p0_w2: i32,
    p0_w5: i64,
    p0_w7: f64,
    p0_w9: *mut perftest_flat::Summary,
    p0_w11: __PbBuffer,
) -> __PbBuffer {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_report_label_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if !(0..2i32).contains(&p0_w2) {
                return Err("invalid selector".into());
            }
            let p0_w4 = if p0_w2 == 0i32 { Some(p0_w5) } else { None };
            let p0_w6 = if p0_w2 == 0i32 { Some(p0_w7) } else { None };
            let p0_w9_pointer = if p0_w2 == 1i32 {
                p0_w9 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w2 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
            }
            let p0_w10 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w11)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w8 = if p0_w2 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w9_pointer) })
            } else {
                None
            };
            let p0_w1: perftest_flat::Summary = match p0_w2 {
                0i32 => {
                    let p0_w4: i64 = ::core::convert::Into::into(
                        p0_w4.expect("active value"),
                    );
                    let p0_w6: f64 = ::core::convert::Into::into(
                        p0_w6.expect("active value"),
                    );
                    let p0_w3: perftest_flat::Summary = perftest_flat::summary_new(
                        p0_w4,
                        p0_w6,
                    );
                    p0_w3
                }
                1i32 => {
                    let p0_w8: perftest_flat::Summary = *p0_w8
                        .expect("active allocation");
                    p0_w8
                }
                _ => unreachable!("validated selector"),
            };
            let p0_w0: perftest_flat::Report = perftest_flat::report_from_summary(
                p0_w1,
                ::core::convert::AsRef::<
                    str,
                >::as_ref(p0_w10.as_ref().expect("active value")),
            );
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::report_label(__prebindgen_argument_0);
            let ret_w0 = ::core::convert::Into::<Vec<u8>>::into(source_value)
                .into_boxed_slice();
            let ret_w0 = Box::into_raw(ret_w0);
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_report_label_result {
                v0: __PbBuffer {
                    data: ret_w0 as *const u8,
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_report_new_result {
    pub v0: i64,
    pub v1: f64,
    pub v2: i64,
    pub v3: i64,
    pub v4: __PbBuffer,
    pub v5: *mut perftest_flat::Report,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_report_new(
    p0_w1: i64,
    p1_w1: f64,
    p2_w1: __PbBuffer,
    out1: *mut f64,
    out2: *mut i64,
    out3: *mut i64,
    out4: *mut __PbBuffer,
    out5: *mut *mut perftest_flat::Report,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_report_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        if out3.is_null() {
            return Err("null success output".into());
        }
        if out4.is_null() {
            return Err("null success output".into());
        }
        if out5.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p2_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: f64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: String = ::core::convert::Into::into(
                p2_w0.expect("active value"),
            );
            let __prebindgen_argument_2 = p2_w0;
            let source_value = perftest_flat::report_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0: *mut perftest_flat::Report = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let ret_w1 = perftest_flat::report_to_struct(unsafe { &*(ret_w0) });
            let ret_w2 = &(ret_w1);
            let ret_w3 = perftest_flat::summary_count(&ret_w2.summary);
            let ret_w4 = ret_w3;
            let ret_w5 = perftest_flat::summary_total(&ret_w2.summary);
            let ret_w6 = ret_w5;
            let ret_w7 = perftest_flat::stamp_secs(&ret_w2.origin);
            let ret_w8 = ret_w7;
            let ret_w9 = perftest_flat::stamp_nanos(&ret_w2.origin);
            let ret_w10 = ret_w9;
            let ret_w11 = ::core::convert::Into::<
                Vec<u8>,
            >::into(::core::clone::Clone::clone(&ret_w2.label))
                .into_boxed_slice();
            let ret_w11 = Box::into_raw(ret_w11);
            let ret_w11_owner = __PbOutputOwner(ret_w11);
            let result = perftest_report_new_result {
                v0: ret_w4,
                v1: ret_w6,
                v2: ret_w8,
                v3: ret_w10,
                v4: __PbBuffer {
                    data: ret_w11 as *const u8,
                    len: ret_w11.len(),
                },
                v5: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w11_owner);
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    std::ptr::write(out3, value.v3);
    std::ptr::write(out4, value.v4);
    std::ptr::write(out5, value.v5);
    value.v0
}
#[repr(C)]
pub struct perftest_resource_choice_id_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_resource_choice_id(
    p0_w4: *mut perftest_flat::OwnedResource,
    p0_w6: i64,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_resource_choice_id_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from((p0_w4 as usize) != 0);
            let p0_w4_pointer = if p0_w1 == 1i32 {
                p0_w4 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w4_pointer)?;
            }
            let p0_w5 = if p0_w1 == 1i32 { Some(p0_w6) } else { None };
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w3 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w4_pointer) })
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w3: perftest_flat::OwnedResource = *p0_w3
                    .expect("active allocation");
                let p0_w5: i64 = ::core::convert::Into::into(
                    p0_w5.expect("active value"),
                );
                let p0_w2: perftest_flat::ResourceChoice = perftest_flat::resource_choice_new(
                    p0_w3,
                    p0_w5,
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::resource_choice_id(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = perftest_resource_choice_id_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_result_error_drops_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_error_drops() -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_result_error_drops_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::result_error_drops();
            let ret_w0 = source_value;
            let result = perftest_result_error_drops_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_result_optional_result {
    pub v0: i32,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_optional(
    p0_w1: i32,
    error: *mut *mut std::ffi::c_char,
) -> i32 {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_optional_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_optional(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w0 {
                let ret_w4: i32 = match &(ret_w3) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                let ret_w2_values = (ret_w4,);
                (1i32, ret_w2_values, Some(move || {}))
            } else {
                (0i32, (-3,), None)
            };
            let result = perftest_result_optional_result {
                v0: ret_w2_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w2_action) = ret_w2_commit {
                    ret_w2_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            -4
        }
    }
}
#[repr(C)]
pub struct perftest_result_other_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_other(
    p0_w1: u8,
    out0: *mut i64,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_other_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_other(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_string_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = perftest_result_other_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_result_owned_result {
    pub v0: *mut perftest_flat::OwnedResource,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_owned(
    p0_w1: *mut perftest_flat::OwnedResource,
    p1_w1: i32,
    p2_w1: u32,
    ownership: *mut bool,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_owned_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true {
                Some(char::from_u32(p2_w1).ok_or("invalid Unicode scalar")?)
            } else {
                None
            };
            let _: () = ();
            let _: () = {
                ownership_accepted = true;
            };
            let p0_w0 = if true {
                Some(unsafe { Box::from_raw(p0_w1_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::OwnedResource = *p0_w0.expect("active allocation");
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: char = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let source_value = perftest_flat::result_owned(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(ret_w0),
            );
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let result = perftest_result_owned_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted);
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw(
                                (value.v0) as *mut perftest_flat::OwnedResource,
                            ),
                        );
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_result_record_result {
    pub v0: i32,
    pub v1: i32,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_record(
    p0_w1: u8,
    out1: *mut i32,
    error: *mut *mut std::ffi::c_char,
) -> i32 {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_record_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_record(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::enum_record_into(ret_w0);
            let ret_w3 = ret_w2;
            let ret_w4: i32 = match &(ret_w3.priority) {
                perftest_flat::Priority::Low => 0i32,
                perftest_flat::Priority::Normal => 1i32,
                perftest_flat::Priority::High => 2i32,
            };
            let (_, (ret_w5_value0,), ret_w5_commit) = if let Some(ret_w6) = ret_w3.code
            {
                let ret_w7: i32 = match &(ret_w6) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                let ret_w5_values = (ret_w7,);
                (1i32, ret_w5_values, Some(move || {}))
            } else {
                (0i32, (-3,), None)
            };
            let result = perftest_result_record_result {
                v0: ret_w4,
                v1: ret_w5_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w5_action) = ret_w5_commit {
                    ret_w5_action();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out1, value.v1);
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            -1
        }
    }
}
#[repr(C)]
pub struct perftest_result_scalar_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_scalar(
    p0_w1: i64,
    out0: *mut i64,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_scalar_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_scalar(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = perftest_result_scalar_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_result_text_result {
    pub v0: __PbBuffer,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_text(
    p0_w1: u8,
    out0: *mut __PbBuffer,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_text_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_text(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ::core::convert::Into::<Vec<u8>>::into(ret_w0)
                .into_boxed_slice();
            let ret_w2 = Box::into_raw(ret_w2);
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let result = perftest_result_text_result {
                v0: __PbBuffer {
                    data: ret_w2 as *const u8,
                    len: ret_w2.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).data.is_null() {
                    drop(
                        Box::from_raw(
                            std::ptr::slice_from_raw_parts_mut(
                                (value.v0).data as *mut u8,
                                (value.v0).len,
                            ),
                        ),
                    );
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_result_unit_result {
    pub unused: u8,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_result_unit(
    p0_w1: u8,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<perftest_result_unit_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            let p0_w0: bool = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::result_unit(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let result = perftest_result_unit_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_sequence_late_drop_each_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sequence_late_drop_each(
    __pb_callback: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut perftest_flat::OwnedResource),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out0: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_sequence_late_drop_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let source_value = perftest_flat::sequence_late_drop();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::sequence_late_view(&ret_w0);
            let ret_w3_source = ret_w2;
            let ret_w5 = ret_w3_source.len();
            for (ret_w6, ret_w4) in ret_w3_source.into_iter().enumerate() {
                let ret_w7: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w4),
                );
                let ret_w7_owner = __PbOutputOwner(ret_w7);
                let __pb_callback_value_0 = ret_w7;
                ::core::mem::forget(ret_w7_owner);
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(__pb_callback_context, __pb_callback_value_0);
                }
            }
            let result = perftest_sequence_late_drop_each_result {
                v0: ret_w5,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_sequence_parts_each_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sequence_parts_each(
    p0_w1: i32,
    __pb_callback: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            i64,
            __PbBuffer,
            *mut perftest_flat::OwnedResource,
        ),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out0: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_sequence_parts_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i32 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::sequence_parts(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2_source = ret_w0;
            let ret_w4 = ret_w2_source.len();
            for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                let ret_w6 = perftest_flat::sequence_part_into(ret_w3);
                let ret_w7 = ret_w6;
                let ret_w8 = ret_w7.number;
                let ret_w9 = ::core::convert::Into::<Vec<u8>>::into(ret_w7.text)
                    .into_boxed_slice();
                let ret_w9 = Box::into_raw(ret_w9);
                let ret_w9_owner = __PbOutputOwner(ret_w9);
                let ret_w10: *mut perftest_flat::OwnedResource = Box::into_raw(
                    Box::new(ret_w7.resource),
                );
                let ret_w10_owner = __PbOutputOwner(ret_w10);
                let __pb_callback_value_0 = ret_w8;
                let __pb_callback_value_1 = __PbBuffer {
                    data: ret_w9 as *const u8,
                    len: ret_w9.len(),
                };
                let __pb_callback_value_2 = ret_w10;
                ::core::mem::forget(ret_w9_owner);
                ::core::mem::forget(ret_w10_owner);
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(
                        __pb_callback_context,
                        __pb_callback_value_0,
                        __pb_callback_value_1,
                        __pb_callback_value_2,
                    );
                }
            }
            let result = perftest_sequence_parts_each_result {
                v0: ret_w4,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct perftest_sequence_scalars_each_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sequence_scalars_each(
    __pb_callback: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            u8,
            i8,
            i16,
            i32,
            i64,
            f32,
            f64,
            __Pb128,
            i32,
            __PbBuffer,
        ),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_sequence_scalars_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let source_value = perftest_flat::sequence_scalars();
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w4 = perftest_flat::sequence_scalars_into(ret_w1);
                let ret_w5 = ret_w4;
                let ret_w6 = u8::from(ret_w5.flag);
                let ret_w7 = ret_w5.byte;
                let ret_w8 = ret_w5.short;
                let ret_w9 = ret_w5.number;
                let ret_w10 = ret_w5.long;
                let ret_w11 = ret_w5.float;
                let ret_w12 = ret_w5.double;
                let ret_w13 = {
                    let bits = (ret_w5.wide) as u128;
                    __Pb128 {
                        hi: (bits >> 64) as u64,
                        lo: bits as u64,
                    }
                };
                let (ret_w14, (ret_w14_value0,), ret_w14_commit) = if let Some(
                    ret_w15,
                ) = ret_w5.text
                {
                    let ret_w16 = ::core::convert::Into::<Vec<u8>>::into(ret_w15)
                        .into_boxed_slice();
                    let ret_w16 = Box::into_raw(ret_w16);
                    let ret_w16_owner = __PbOutputOwner(ret_w16);
                    let ret_w14_values = (
                        __PbBuffer {
                            data: ret_w16 as *const u8,
                            len: ret_w16.len(),
                        },
                    );
                    (
                        1i32,
                        ret_w14_values,
                        Some(move || {
                            ::core::mem::forget(ret_w16_owner);
                        }),
                    )
                } else {
                    (
                        0i32,
                        (
                            __PbBuffer {
                                data: std::ptr::null(),
                                len: 0,
                            },
                        ),
                        None,
                    )
                };
                let __pb_callback_value_0 = ret_w6;
                let __pb_callback_value_1 = ret_w7;
                let __pb_callback_value_2 = ret_w8;
                let __pb_callback_value_3 = ret_w9;
                let __pb_callback_value_4 = ret_w10;
                let __pb_callback_value_5 = ret_w11;
                let __pb_callback_value_6 = ret_w12;
                let __pb_callback_value_7 = ret_w13;
                let __pb_callback_value_8 = ret_w14;
                let __pb_callback_value_9 = ret_w14_value0;
                if let Some(ret_w14_action) = ret_w14_commit {
                    ret_w14_action();
                }
                unsafe {
                    __pb_callback
                        .expect(
                            "validated callback",
                        )(
                        __pb_callback_context,
                        __pb_callback_value_0,
                        __pb_callback_value_1,
                        __pb_callback_value_2,
                        __pb_callback_value_3,
                        __pb_callback_value_4,
                        __pb_callback_value_5,
                        __pb_callback_value_6,
                        __pb_callback_value_7,
                        __pb_callback_value_8,
                        __pb_callback_value_9,
                    );
                }
            }
            let result = perftest_sequence_scalars_each_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_sparse_borrowed_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sparse_borrowed(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_sparse_borrowed_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    match p0_w1 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.as_ref().expect("active enum");
            let source_value = perftest_flat::sparse_borrowed(__prebindgen_argument_0);
            let ret_w0: i32 = match &(source_value) {
                perftest_flat::SparseCode::Bottom => -2147483648i32,
                perftest_flat::SparseCode::MinusTwo => -2i32,
                perftest_flat::SparseCode::MinusOne => -1i32,
                perftest_flat::SparseCode::Zero => 0i32,
                perftest_flat::SparseCode::Seven => 7i32,
                perftest_flat::SparseCode::Eight => 8i32,
                perftest_flat::SparseCode::Top => 2147483647i32,
            };
            let result = perftest_sparse_borrowed_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_sparse_echo_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sparse_echo(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_sparse_echo_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    match p0_w1 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = p0_w0.expect("active enum");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::sparse_echo(__prebindgen_argument_0);
            let ret_w0: i32 = match &(source_value) {
                perftest_flat::SparseCode::Bottom => -2147483648i32,
                perftest_flat::SparseCode::MinusTwo => -2i32,
                perftest_flat::SparseCode::MinusOne => -1i32,
                perftest_flat::SparseCode::Zero => 0i32,
                perftest_flat::SparseCode::Seven => 7i32,
                perftest_flat::SparseCode::Eight => 8i32,
                perftest_flat::SparseCode::Top => 2147483647i32,
            };
            let result = perftest_sparse_echo_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_sparse_nested_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sparse_nested(p0_w5: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_sparse_nested_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w5 != -4);
            let p0_w3: i32 = i32::from(p0_w5 != -3);
            let p0_w4 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                Some(
                    match p0_w5 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = if (p0_w1 == 1i32) && (p0_w3 == 1i32) {
                    let p0_w4 = p0_w4.expect("active enum");
                    Some(p0_w4)
                } else {
                    None
                };
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::sparse_nested(__prebindgen_argument_0);
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let (_, (ret_w2_value0,), ret_w2_commit) = if let Some(ret_w3) = ret_w1 {
                    let ret_w4: i32 = match &(ret_w3) {
                        perftest_flat::SparseCode::Bottom => -2147483648i32,
                        perftest_flat::SparseCode::MinusTwo => -2i32,
                        perftest_flat::SparseCode::MinusOne => -1i32,
                        perftest_flat::SparseCode::Zero => 0i32,
                        perftest_flat::SparseCode::Seven => 7i32,
                        perftest_flat::SparseCode::Eight => 8i32,
                        perftest_flat::SparseCode::Top => 2147483647i32,
                    };
                    let ret_w2_values = (ret_w4,);
                    (1i32, ret_w2_values, Some(move || {}))
                } else {
                    (0i32, (-3,), None)
                };
                let ret_w0_values = (ret_w2_value0,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        if let Some(ret_w2_action) = ret_w2_commit {
                            ret_w2_action();
                        }
                    }),
                )
            } else {
                (0i32, (-4,), None)
            };
            let result = perftest_sparse_nested_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_sparse_optional_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sparse_optional(p0_w3: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_sparse_optional_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1: i32 = i32::from(p0_w3 != -3);
            let p0_w2 = if p0_w1 == 1i32 {
                Some(
                    match p0_w3 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2 = p0_w2.expect("active enum");
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::sparse_optional(__prebindgen_argument_0);
            let (_, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2: i32 = match &(ret_w1) {
                    perftest_flat::SparseCode::Bottom => -2147483648i32,
                    perftest_flat::SparseCode::MinusTwo => -2i32,
                    perftest_flat::SparseCode::MinusOne => -1i32,
                    perftest_flat::SparseCode::Zero => 0i32,
                    perftest_flat::SparseCode::Seven => 7i32,
                    perftest_flat::SparseCode::Eight => 8i32,
                    perftest_flat::SparseCode::Top => 2147483647i32,
                };
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (-3,), None)
            };
            let result = perftest_sparse_optional_result {
                v0: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_sparse_ref_result {
    pub v0: i32,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_sparse_ref(p0_w1: i32) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_sparse_ref_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    match p0_w1 {
                        -2147483648i32 => perftest_flat::SparseCode::Bottom,
                        -2i32 => perftest_flat::SparseCode::MinusTwo,
                        -1i32 => perftest_flat::SparseCode::MinusOne,
                        0i32 => perftest_flat::SparseCode::Zero,
                        7i32 => perftest_flat::SparseCode::Seven,
                        8i32 => perftest_flat::SparseCode::Eight,
                        2147483647i32 => perftest_flat::SparseCode::Top,
                        _ => return Err("invalid enum discriminant".into()),
                    },
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.as_ref().expect("active enum");
            let source_value = perftest_flat::sparse_ref(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_sparse_ref_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_stamp_new_result {
    pub v0: i64,
    pub v1: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_stamp_new(
    p0_w1: i64,
    p1_w1: i64,
    out1: *mut i64,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_stamp_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::stamp_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = perftest_flat::stamp_secs(&source_value);
            let ret_w1 = ret_w0;
            let ret_w2 = perftest_flat::stamp_nanos(&source_value);
            let ret_w3 = ret_w2;
            let result = perftest_stamp_new_result {
                v0: ret_w1,
                v1: ret_w3,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_storage_contains_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_contains(
    p0_w1: *const perftest_flat::Storage,
    p1_w1: i64,
) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_contains_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::storage_contains(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = u8::from(source_value);
            let result = perftest_storage_contains_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_storage_get_result {
    pub v0: i32,
    pub v1: i64,
    pub v2: i32,
    pub v3: f64,
    pub v4: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_get(
    p0_w1: *const perftest_flat::Storage,
    out1: *mut i64,
    out2: *mut i32,
    out3: *mut f64,
    out4: *mut u8,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_storage_get_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        if out3.is_null() {
            return Err("null success output".into());
        }
        if out4.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_get(__prebindgen_argument_0);
            let (
                ret_w0,
                (ret_w0_value0, ret_w0_value1, ret_w0_value2, ret_w0_value3),
                ret_w0_commit,
            ) = if let Some(ret_w1) = source_value {
                let ret_w2 = perftest_flat::payload_to_struct(&ret_w1);
                let ret_w3 = &(ret_w2);
                let ret_w4: &i64 = &ret_w3.id;
                let ret_w4 = *ret_w4;
                let ret_w5: &i32 = &ret_w3.seq;
                let ret_w5 = *ret_w5;
                let ret_w6: &f64 = &ret_w3.value;
                let ret_w6 = *ret_w6;
                let ret_w7: &bool = &ret_w3.flag;
                let ret_w7 = u8::from(*ret_w7);
                let ret_w0_values = (ret_w4, ret_w5, ret_w6, ret_w7);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0, 0, 0.0, 0), None)
            };
            let result = perftest_storage_get_result {
                v0: ret_w0,
                v1: ret_w0_value0,
                v2: ret_w0_value1,
                v3: ret_w0_value2,
                v4: ret_w0_value3,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    std::ptr::write(out3, value.v3);
    std::ptr::write(out4, value.v4);
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_storage_get_vec_ret_w2 {
    pub v0: i64,
    pub v1: i32,
    pub v2: f64,
    pub v3: u8,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_get_vec_ret_w2_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_storage_get_vec_ret_w2,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct perftest_storage_get_vec_result {
    pub v0: i32,
    pub v1: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_get_vec(
    p0_w1: *const perftest_flat::Storage,
    out1: *mut __PbRows,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_storage_get_vec_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_get_vec(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                let mut ret_w2_commits = Vec::with_capacity(ret_w4);
                let mut ret_w2: Vec<__PbRow_storage_get_vec_ret_w2> = Vec::with_capacity(
                    ret_w4,
                );
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6 = perftest_flat::payload_to_struct(&ret_w3);
                    let ret_w7 = &(ret_w6);
                    let ret_w8: &i64 = &ret_w7.id;
                    let ret_w8 = *ret_w8;
                    let ret_w9: &i32 = &ret_w7.seq;
                    let ret_w9 = *ret_w9;
                    let ret_w10: &f64 = &ret_w7.value;
                    let ret_w10 = *ret_w10;
                    let ret_w11: &bool = &ret_w7.flag;
                    let ret_w11 = u8::from(*ret_w11);
                    ret_w2
                        .push(__PbRow_storage_get_vec_ret_w2 {
                            v0: ret_w8,
                            v1: ret_w9,
                            v2: ret_w10,
                            v3: ret_w11,
                        });
                    ret_w2_commits.push(move || {});
                }
                let ret_w2 = ret_w2.into_boxed_slice();
                let ret_w0_values = (
                    __PbRows {
                        data: if ret_w2.is_empty() {
                            ::core::ptr::null()
                        } else {
                            ret_w2.as_ptr().cast()
                        },
                        len: ret_w2.len(),
                    },
                );
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        let _ = Box::into_raw(ret_w2);
                        for commit in ret_w2_commits {
                            commit();
                        }
                    }),
                )
            } else {
                (
                    0i32,
                    (
                        __PbRows {
                            data: std::ptr::null(),
                            len: 0,
                        },
                    ),
                    None,
                )
            };
            let result = perftest_storage_get_vec_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_storage_get_vec_each_result {
    pub v0: i32,
    pub v1: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_get_vec_each(
    p0_w1: *const perftest_flat::Storage,
    __pb_callback: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, i64, i32, f64, u8),
    >,
    __pb_callback_context: *mut ::core::ffi::c_void,
    out1: *mut usize,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_get_vec_each_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if __pb_callback.is_none() {
            return Err("null sequence callback".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_get_vec(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2_source = ret_w1;
                let ret_w4 = ret_w2_source.len();
                for (ret_w5, ret_w3) in ret_w2_source.into_iter().enumerate() {
                    let ret_w6 = perftest_flat::payload_to_struct(&ret_w3);
                    let ret_w7 = &(ret_w6);
                    let ret_w8: &i64 = &ret_w7.id;
                    let ret_w8 = *ret_w8;
                    let ret_w9: &i32 = &ret_w7.seq;
                    let ret_w9 = *ret_w9;
                    let ret_w10: &f64 = &ret_w7.value;
                    let ret_w10 = *ret_w10;
                    let ret_w11: &bool = &ret_w7.flag;
                    let ret_w11 = u8::from(*ret_w11);
                    let __pb_callback_value_0 = ret_w8;
                    let __pb_callback_value_1 = ret_w9;
                    let __pb_callback_value_2 = ret_w10;
                    let __pb_callback_value_3 = ret_w11;
                    unsafe {
                        __pb_callback
                            .expect(
                                "validated callback",
                            )(
                            __pb_callback_context,
                            __pb_callback_value_0,
                            __pb_callback_value_1,
                            __pb_callback_value_2,
                            __pb_callback_value_3,
                        );
                    }
                }
                let ret_w0_values = (ret_w4,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_storage_get_vec_each_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_storage_len_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_len(
    p0_w1: *const perftest_flat::Storage,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_storage_len_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_len(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_storage_len_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_storage_new_result {
    pub v0: *mut perftest_flat::Storage,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_new() -> *mut perftest_flat::Storage {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_storage_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::storage_new();
            let ret_w0: *mut perftest_flat::Storage = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_storage_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_storage_put_by_take_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_put_by_take(
    p0_w1: *mut perftest_flat::Storage,
    p1_w2: i64,
    p1_w4: i32,
    p1_w6: f64,
    p1_w8: u8,
) {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_put_by_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w1 = if true { Some(p1_w2) } else { None };
            let p1_w3 = if true { Some(p1_w4) } else { None };
            let p1_w5 = if true { Some(p1_w6) } else { None };
            let p1_w7 = if true { Some(p1_w8 != 0) } else { None };
            let _: () = ();
            let p0_w0: &mut perftest_flat::Storage = unsafe { &mut (*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w1: i64 = ::core::convert::Into::into(p1_w1.expect("active value"));
            let p1_w3: i32 = ::core::convert::Into::into(p1_w3.expect("active value"));
            let p1_w5: f64 = ::core::convert::Into::into(p1_w5.expect("active value"));
            let p1_w7: bool = ::core::convert::Into::into(p1_w7.expect("active value"));
            let p1_w0: perftest_flat::Payload = perftest_flat::payload_new(
                p1_w1,
                p1_w3,
                p1_w5,
                p1_w7,
            );
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::storage_put_by_take(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let result = perftest_storage_put_by_take_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct perftest_storage_put_opt_result {
    pub v0: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_put_opt(
    p0_w1: *mut perftest_flat::Storage,
    p1_w1: i32,
    p1_w4: i64,
    p1_w6: i32,
    p1_w8: f64,
    p1_w10: u8,
) -> u8 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_storage_put_opt_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            if p1_w1 != 0 && p1_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p1_w3 = if p1_w1 == 1i32 { Some(p1_w4) } else { None };
            let p1_w5 = if p1_w1 == 1i32 { Some(p1_w6) } else { None };
            let p1_w7 = if p1_w1 == 1i32 { Some(p1_w8) } else { None };
            let p1_w9 = if p1_w1 == 1i32 { Some(p1_w10 != 0) } else { None };
            let _: () = ();
            let p0_w0: &mut perftest_flat::Storage = unsafe { &mut (*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0 = if p1_w1 == 1i32 {
                let p1_w3: i64 = ::core::convert::Into::into(
                    p1_w3.expect("active value"),
                );
                let p1_w5: i32 = ::core::convert::Into::into(
                    p1_w5.expect("active value"),
                );
                let p1_w7: f64 = ::core::convert::Into::into(
                    p1_w7.expect("active value"),
                );
                let p1_w9: bool = ::core::convert::Into::into(
                    p1_w9.expect("active value"),
                );
                let p1_w2: perftest_flat::Payload = perftest_flat::payload_new(
                    p1_w3,
                    p1_w5,
                    p1_w7,
                    p1_w9,
                );
                Some(p1_w2)
            } else {
                None
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::storage_put_opt(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = u8::from(source_value);
            let result = perftest_storage_put_opt_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_storage_put_slice_p1_w1 {
    pub v0: i64,
    pub v1: i32,
    pub v2: f64,
    pub v3: u8,
}
#[repr(C)]
pub struct perftest_storage_put_slice_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_put_slice(
    p0_w1: *mut perftest_flat::Storage,
    p1_w1: __PbRows,
) {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_put_slice_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let mut p1_w1_staged = Vec::new();
            if true {
                let p1_w2 = p1_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p1_w1.data as *const __PbRow_storage_put_slice_p1_w1,
                    p1_w2,
                )?;
                for p1_w3 in 0..p1_w2 {
                    let p1_w6 = unsafe {
                        (*((p1_w1.data as *const __PbRow_storage_put_slice_p1_w1)
                            .add(p1_w3)))
                            .v0
                    };
                    let p1_w8 = unsafe {
                        (*((p1_w1.data as *const __PbRow_storage_put_slice_p1_w1)
                            .add(p1_w3)))
                            .v1
                    };
                    let p1_w10 = unsafe {
                        (*((p1_w1.data as *const __PbRow_storage_put_slice_p1_w1)
                            .add(p1_w3)))
                            .v2
                    };
                    let p1_w12 = unsafe {
                        (*((p1_w1.data as *const __PbRow_storage_put_slice_p1_w1)
                            .add(p1_w3)))
                            .v3
                    };
                    let p1_w5 = if true { Some(p1_w6) } else { None };
                    let p1_w7 = if true { Some(p1_w8) } else { None };
                    let p1_w9 = if true { Some(p1_w10) } else { None };
                    let p1_w11 = if true { Some((p1_w12) != 0) } else { None };
                    p1_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<
                                perftest_flat::Payload,
                                String,
                            > {
                                let p1_w5: i64 = ::core::convert::Into::into(
                                    p1_w5.expect("active value"),
                                );
                                let p1_w7: i32 = ::core::convert::Into::into(
                                    p1_w7.expect("active value"),
                                );
                                let p1_w9: f64 = ::core::convert::Into::into(
                                    p1_w9.expect("active value"),
                                );
                                let p1_w11: bool = ::core::convert::Into::into(
                                    p1_w11.expect("active value"),
                                );
                                let p1_w4: perftest_flat::Payload = perftest_flat::payload_new(
                                    p1_w5,
                                    p1_w7,
                                    p1_w9,
                                    p1_w11,
                                );
                                let p1_w0 = p1_w4;
                                Ok(p1_w0)
                            }
                        });
                }
            }
            let _: () = ();
            let p1_w1_acquired: Vec<_> = p1_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: &mut perftest_flat::Storage = unsafe { &mut (*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: Vec<perftest_flat::Payload> = p1_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_1 = p1_w0.as_slice();
            let source_value = perftest_flat::storage_put_slice(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let result = perftest_storage_put_slice_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct perftest_storage_summary_handle_result {
    pub v0: *mut perftest_flat::Summary,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_summary_handle(
    p0_w1: *const perftest_flat::Storage,
) -> *mut perftest_flat::Summary {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_summary_handle_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_summary_handle(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::Summary = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_storage_summary_handle_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_storage_total_len_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_total_len(
    p0_w1: *const perftest_flat::Storage,
    p1_w1: *const perftest_flat::Storage,
    p2_w1: *const perftest_flat::Storage,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_total_len_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w1_pointer = if true {
                p1_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p1_w1_pointer)?;
            }
            let p2_w1_pointer = if true {
                p2_w1 as *mut perftest_flat::Storage
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p2_w1_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p1_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p2_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: &perftest_flat::Storage = unsafe { &(*p1_w1_pointer) };
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: &perftest_flat::Storage = unsafe { &(*p2_w1_pointer) };
            let __prebindgen_argument_2 = p2_w0;
            let source_value = perftest_flat::storage_total_len(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0 = source_value;
            let result = perftest_storage_total_len_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_storage_try_with_label_result {
    pub v0: *mut perftest_flat::Storage,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_try_with_label(
    p0_w1: __PbBuffer,
    error: *mut *mut std::ffi::c_char,
) -> *mut perftest_flat::Storage {
    #[allow(unused_mut)]
    let mut ownership_accepted = false;
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_try_with_label_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = ::core::convert::AsRef::<
                str,
            >::as_ref(p0_w0.as_ref().expect("active value"));
            let source_value = perftest_flat::storage_try_with_label(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::storage_error_message(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2: *mut perftest_flat::Storage = Box::into_raw(Box::new(ret_w0));
            let ret_w2_owner = __PbOutputOwner(ret_w2);
            let result = perftest_storage_try_with_label_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2_owner);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            value.v0
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if !(value.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(Box::from_raw((value.v0) as *mut perftest_flat::Storage));
                        Ok(())
                    });
                }
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            std::ptr::null_mut()
        }
    }
}
#[repr(C)]
pub struct perftest_storage_with_payload_result {
    pub v0: *mut perftest_flat::Storage,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_storage_with_payload(
    p0_w2: i64,
    p0_w4: i32,
    p0_w6: f64,
    p0_w8: u8,
) -> *mut perftest_flat::Storage {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_storage_with_payload_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1 = if true { Some(p0_w2) } else { None };
            let p0_w3 = if true { Some(p0_w4) } else { None };
            let p0_w5 = if true { Some(p0_w6) } else { None };
            let p0_w7 = if true { Some(p0_w8 != 0) } else { None };
            let p0_w1: i64 = ::core::convert::Into::into(p0_w1.expect("active value"));
            let p0_w3: i32 = ::core::convert::Into::into(p0_w3.expect("active value"));
            let p0_w5: f64 = ::core::convert::Into::into(p0_w5.expect("active value"));
            let p0_w7: bool = ::core::convert::Into::into(p0_w7.expect("active value"));
            let p0_w0: perftest_flat::Payload = perftest_flat::payload_new(
                p0_w1,
                p0_w3,
                p0_w5,
                p0_w7,
            );
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_with_payload(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::Storage = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_storage_with_payload_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_string_len_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_string_len(p0_w1: __PbBuffer) -> usize {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_string_len_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    ::core::convert::Into::<
                        String,
                    >::into(
                        std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                            .map_err(|_| "invalid UTF-8")?,
                    ),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.as_ref().expect("active value");
            let source_value = perftest_flat::string_len(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_string_len_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_string_new_result {
    pub v0: __PbBuffer,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_string_new(p0_w1: __PbBuffer) -> __PbBuffer {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_string_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p0_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = ::core::convert::AsRef::<
                str,
            >::as_ref(p0_w0.as_ref().expect("active value"));
            let source_value = perftest_flat::string_new(__prebindgen_argument_0);
            let ret_w0 = ::core::convert::Into::<Vec<u8>>::into(source_value)
                .into_boxed_slice();
            let ret_w0 = Box::into_raw(ret_w0);
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_string_new_result {
                v0: __PbBuffer {
                    data: ret_w0 as *const u8,
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_summary_merge_result {
    pub v0: i64,
    pub v1: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_summary_merge(
    p0_w1: i32,
    p0_w4: i64,
    p0_w6: f64,
    p0_w8: *mut perftest_flat::Summary,
    p1_w1: i32,
    p1_w4: i64,
    p1_w6: f64,
    p1_w8: *mut perftest_flat::Summary,
    out1: *mut f64,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_summary_merge_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            let p0_w3 = if p0_w1 == 0i32 { Some(p0_w4) } else { None };
            let p0_w5 = if p0_w1 == 0i32 { Some(p0_w6) } else { None };
            let p0_w8_pointer = if p0_w1 == 1i32 {
                p0_w8 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w8_pointer)?;
            }
            if !(0..2i32).contains(&p1_w1) {
                return Err("invalid selector".into());
            }
            let p1_w3 = if p1_w1 == 0i32 { Some(p1_w4) } else { None };
            let p1_w5 = if p1_w1 == 0i32 { Some(p1_w6) } else { None };
            let p1_w8_pointer = if p1_w1 == 1i32 {
                p1_w8 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if p1_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p1_w8_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if p0_w1 == 1i32 {
                    for identity in ::core::iter::once((p0_w8_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                if p1_w1 == 1i32 {
                    for identity in ::core::iter::once((p1_w8_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w7 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w8_pointer) })
            } else {
                None
            };
            let p1_w7 = if p1_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p1_w8_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::Summary = match p0_w1 {
                0i32 => {
                    let p0_w3: i64 = ::core::convert::Into::into(
                        p0_w3.expect("active value"),
                    );
                    let p0_w5: f64 = ::core::convert::Into::into(
                        p0_w5.expect("active value"),
                    );
                    let p0_w2: perftest_flat::Summary = perftest_flat::summary_new(
                        p0_w3,
                        p0_w5,
                    );
                    p0_w2
                }
                1i32 => {
                    let p0_w7: perftest_flat::Summary = *p0_w7
                        .expect("active allocation");
                    p0_w7
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: perftest_flat::Summary = match p1_w1 {
                0i32 => {
                    let p1_w3: i64 = ::core::convert::Into::into(
                        p1_w3.expect("active value"),
                    );
                    let p1_w5: f64 = ::core::convert::Into::into(
                        p1_w5.expect("active value"),
                    );
                    let p1_w2: perftest_flat::Summary = perftest_flat::summary_new(
                        p1_w3,
                        p1_w5,
                    );
                    p1_w2
                }
                1i32 => {
                    let p1_w7: perftest_flat::Summary = *p1_w7
                        .expect("active allocation");
                    p1_w7
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::summary_merge(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = perftest_flat::summary_count(&source_value);
            let ret_w1 = ret_w0;
            let ret_w2 = perftest_flat::summary_total(&source_value);
            let ret_w3 = ret_w2;
            let result = perftest_summary_merge_result {
                v0: ret_w1,
                v1: ret_w3,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_summary_new_result {
    pub v0: i64,
    pub v1: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_summary_new(
    p0_w1: i64,
    p1_w1: f64,
    out1: *mut f64,
) -> i64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_summary_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: f64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::summary_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = perftest_flat::summary_count(&source_value);
            let ret_w1 = ret_w0;
            let ret_w2 = perftest_flat::summary_total(&source_value);
            let ret_w3 = ret_w2;
            let result = perftest_summary_new_result {
                v0: ret_w1,
                v1: ret_w3,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_summary_total_result {
    pub v0: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_summary_total(
    p0_w1: *const perftest_flat::Summary,
) -> f64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<perftest_summary_total_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Summary = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::summary_total(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_summary_total_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_summary_total_raw_result {
    pub v0: f64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_summary_total_raw(
    p0_w1: i32,
    p0_w4: i64,
    p0_w6: f64,
    p0_w8: *mut perftest_flat::Summary,
) -> f64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_summary_total_raw_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            let p0_w3 = if p0_w1 == 0i32 { Some(p0_w4) } else { None };
            let p0_w5 = if p0_w1 == 0i32 { Some(p0_w6) } else { None };
            let p0_w8_pointer = if p0_w1 == 1i32 {
                p0_w8 as *mut perftest_flat::Summary
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w8_pointer)?;
            }
            let _: () = ();
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w7 = if p0_w1 == 1i32 {
                Some(unsafe { Box::from_raw(p0_w8_pointer) })
            } else {
                None
            };
            let p0_w0: perftest_flat::Summary = match p0_w1 {
                0i32 => {
                    let p0_w3: i64 = ::core::convert::Into::into(
                        p0_w3.expect("active value"),
                    );
                    let p0_w5: f64 = ::core::convert::Into::into(
                        p0_w5.expect("active value"),
                    );
                    let p0_w2: perftest_flat::Summary = perftest_flat::summary_new(
                        p0_w3,
                        p0_w5,
                    );
                    p0_w2
                }
                1i32 => {
                    let p0_w7: perftest_flat::Summary = *p0_w7
                        .expect("active allocation");
                    p0_w7
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::summary_total_raw(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = perftest_summary_total_raw_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct perftest_unsigned_data_maybe_result {
    pub v0: i32,
    pub v1: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_unsigned_data_maybe(
    p0_w1: *const perftest_flat::Unsigned,
    out1: *mut u64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_unsigned_data_maybe_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::Unsigned
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let _: () = ();
            let p0_w0: &perftest_flat::Unsigned = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::unsigned_data_maybe(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_unsigned_data_maybe_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_unsigned_optional_result {
    pub v0: i32,
    pub v1: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_unsigned_optional(
    p0_w1: i32,
    p0_w3: u64,
    out1: *mut u64,
) -> i32 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_unsigned_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p0_w2 = if p0_w1 == 1i32 { Some(p0_w3) } else { None };
            let p0_w0 = if p0_w1 == 1i32 {
                let p0_w2: u64 = ::core::convert::Into::into(
                    p0_w2.expect("active value"),
                );
                Some(p0_w2)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::unsigned_optional(__prebindgen_argument_0);
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            let result = perftest_unsigned_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct perftest_unsigned_round_trip_result {
    pub v0: *mut perftest_flat::Unsigned,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn perftest_unsigned_round_trip(
    p0_w1: u8,
    p1_w1: u16,
    p2_w1: u32,
    p3_w1: u64,
    p4_w1: i32,
    p4_w3: u64,
) -> *mut perftest_flat::Unsigned {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        perftest_unsigned_round_trip_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let p3_w0 = if true { Some(p3_w1) } else { None };
            if p4_w1 != 0 && p4_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p4_w2 = if p4_w1 == 1i32 { Some(p4_w3) } else { None };
            let p0_w0: u8 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: u16 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: u32 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let p3_w0: u64 = ::core::convert::Into::into(p3_w0.expect("active value"));
            let __prebindgen_argument_3 = p3_w0;
            let p4_w0 = if p4_w1 == 1i32 {
                let p4_w2: u64 = ::core::convert::Into::into(
                    p4_w2.expect("active value"),
                );
                Some(p4_w2)
            } else {
                None
            };
            let __prebindgen_argument_4 = p4_w0;
            let source_value = perftest_flat::unsigned_round_trip(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
                __prebindgen_argument_3,
                __prebindgen_argument_4,
            );
            let ret_w0: *mut perftest_flat::Unsigned = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = perftest_unsigned_round_trip_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::ApiRecord > () != 0,
    "flat object `ApiRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Archive > () != 0,
    "flat object `Archive` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::BlobValue > () != 0,
    "flat object `BlobValue` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CDropError > () != 0,
    "flat object `CDropError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CPanicError > () != 0,
    "flat object `CPanicError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CPanicOutput > () != 0,
    "flat object `CPanicOutput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CResourceError > () != 0,
    "flat object `CResourceError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CResultError > () != 0,
    "flat object `CResultError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CResultPair > () != 0,
    "flat object `CResultPair` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CRichError > () != 0,
    "flat object `CRichError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnBorrowedPair > () != 0,
    "flat object `CallbackReturnBorrowedPair` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnChoice > () != 0,
    "flat object `CallbackReturnChoice` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnHandler > () != 0,
    "flat object `CallbackReturnHandler` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnJob > () != 0,
    "flat object `CallbackReturnJob` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnPair > () != 0,
    "flat object `CallbackReturnPair` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnRecord > () != 0,
    "flat object `CallbackReturnRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CallbackReturnResource > () != 0,
    "flat object `CallbackReturnResource` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CollectionBundle > () != 0,
    "flat object `CollectionBundle` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CollectionError > () != 0,
    "flat object `CollectionError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CollectionItem > () != 0,
    "flat object `CollectionItem` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CollectionNumbers > () != 0,
    "flat object `CollectionNumbers` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::EnumNames > () != 0,
    "flat object `EnumNames` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::EnumRecord > () != 0,
    "flat object `EnumRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::GetterChild > () != 0,
    "flat object `GetterChild` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::GetterDetails > () != 0,
    "flat object `GetterDetails` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::GetterLeaf > () != 0,
    "flat object `GetterLeaf` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::GetterOwner > () != 0,
    "flat object `GetterOwner` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::NicheChoiceHandle > () != 0,
    "flat object `NicheChoiceHandle` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OptionChoice > () != 0,
    "flat object `OptionChoice` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OptionCtor > () != 0,
    "flat object `OptionCtor` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OptionalFields > () != 0,
    "flat object `OptionalFields` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OwnedResource > () != 0,
    "flat object `OwnedResource` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Payload > () != 0,
    "flat object `Payload` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::PayloadHandle > () != 0,
    "flat object `PayloadHandle` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Priority > () != 0,
    "flat object `Priority` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Report > () != 0,
    "flat object `Report` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::ReportStruct > () != 0,
    "flat object `ReportStruct` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::ResourceChoice > () != 0,
    "flat object `ResourceChoice` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::ResultFailure > () != 0,
    "flat object `ResultFailure` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::SequencePart > () != 0,
    "flat object `SequencePart` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::SequenceScalars > () != 0,
    "flat object `SequenceScalars` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::SparseCode > () != 0,
    "flat object `SparseCode` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Stamp > () != 0,
    "flat object `Stamp` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Storage > () != 0,
    "flat object `Storage` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::StorageError > () != 0,
    "flat object `StorageError` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Summary > () != 0,
    "flat object `Summary` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Unsigned > () != 0,
    "flat object `Unsigned` must have nonzero size"
);
const _: () = assert!(
    perftest_flat::CallbackReturnChoice::First as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::CallbackReturnChoice::Second as i128 == 1i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::Priority::Low as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::Priority::Normal as i128 == 1i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::Priority::High as i128 == 2i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Bottom as i128 == - 2147483648i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::MinusTwo as i128 == - 2i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::MinusOne as i128 == - 1i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Zero as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Seven as i128 == 7i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Eight as i128 == 8i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Top as i128 == 2147483647i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::Companion as i128 == 4i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::wireValue as i128 == 9i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::fromWire as i128 == 12i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::r#type as i128 == 14i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::when as i128 == 16i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::name as i128 == 20i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::ordinal as i128 == 21i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::entries as i128 == 22i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::value as i128 == 24i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::EnumNames::kotlin as i128 == 25i32 as i128,
    "enum discriminant differs from captured source"
);
