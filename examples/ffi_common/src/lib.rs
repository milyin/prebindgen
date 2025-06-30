prebindgen_path!(GENERATED_PATH);

#[prebindgen]
#[repr(C)]
pub struct Foo {
    pub id: u64,
}

#[prebindgen]
pub fn copy_foo(dst: &MaybeUninit<Foo>, src: &Foo) {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
}
