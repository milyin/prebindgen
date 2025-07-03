#[no_mangle] pub unsafe extern "C" fn void_function (_x : i32) { example_ffi :: void_function (_x) ; }
#[no_mangle] pub unsafe extern "C" fn copy_foo (_dst : & mut std :: mem :: MaybeUninit < Foo > , _src : & Foo) { example_ffi :: copy_foo (unsafe { std :: mem :: transmute (_dst) } , unsafe { std :: mem :: transmute (_src) }) ; }
#[no_mangle] pub unsafe extern "C" fn another_test_function () -> bool { example_ffi :: another_test_function () }
#[no_mangle] pub unsafe extern "C" fn test_function (_a : i32 , _b : f64) -> i32 { example_ffi :: test_function (_a , _b) }
#[repr(C)] #[derive(Copy, Clone, Debug, PartialEq)] pub struct Foo
{
    #[cfg(target_arch = "x86_64")] pub x86_64_field : u64,
    #[cfg(target_arch = "aarch64")] pub aarch64_field : u64,
}
#[repr(C)] pub struct Bar { pub aarch64_field : u64 }
