use prebindgen::{prebindgen, prebindgen_path};

pub const GENERATED_PATH: &str = prebindgen_path!();

#[prebindgen]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Foo {
    pub id: u64,
}

pub fn copy_foo(dst: &mut std::mem::MaybeUninit<Foo>, src: &Foo) {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
}
