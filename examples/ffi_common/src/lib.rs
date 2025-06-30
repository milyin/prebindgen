use prebindgen::{prebindgen, prebindgen_path};

prebindgen_path!(GENERATED_PATH);

#[prebindgen]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Foo {
    pub id: u64,
}

pub fn copy_foo(dst: &mut std::mem::MaybeUninit<Foo>, src: &Foo) {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
}
