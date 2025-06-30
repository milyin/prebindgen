// Include the copied ffi_common.rs file from OUT_DIR
include!(concat!(env!("OUT_DIR"), "/ffi_common.rs"));

// Demonstrate that we can use the included Foo struct
pub fn create_foo(id: u64) -> Foo {
    Foo { id }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo_struct() {
        let foo = create_foo(42);
        assert_eq!(foo.id, 42);
    }
}