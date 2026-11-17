//! Benchmark-only, calling-thread Rust allocation counts. Never enabled for timing.
#[cfg(feature="count-allocations")]
mod counter {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static VALUES: Cell<[u64;3]> = const { Cell::new([0;3]) };
    }
    fn record(size:usize, reallocation:bool) {
        if ACTIVE.try_with(Cell::get).unwrap_or(false) {
            let _=VALUES.try_with(|values| {
                let mut v=values.get();
                v[usize::from(reallocation)]=v[usize::from(reallocation)].wrapping_add(1);
                v[2]=v[2].wrapping_add(size as u64);
                values.set(v);
            });
        }
    }
    struct Allocator;
    #[global_allocator] static ALLOCATOR: Allocator=Allocator;
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout:Layout)->*mut u8 {
            let p=unsafe {System.alloc(layout)};
            if !p.is_null(){record(layout.size(),false);} p
        }
        unsafe fn alloc_zeroed(&self, layout:Layout)->*mut u8 {
            let p=unsafe {System.alloc_zeroed(layout)};
            if !p.is_null(){record(layout.size(),false);} p
        }
        unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe {System.dealloc(p,layout)}}
        unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8 {
            let p=unsafe {System.realloc(p,layout,size)};
            if !p.is_null(){record(size,true);} p
        }
    }
    pub fn start(){VALUES.with(|v|v.set([0;3]));ACTIVE.with(|v|v.set(true));}
    pub fn stop()->[u64;3]{ACTIVE.with(|v|v.set(false));VALUES.with(Cell::get)}
}
#[no_mangle] pub extern "C" fn bench_counter_enabled()->bool {cfg!(feature="count-allocations")}
#[no_mangle] pub extern "C" fn bench_counter_start(){
    #[cfg(feature="count-allocations")] counter::start();
}
#[no_mangle] pub unsafe extern "C" fn bench_counter_stop(out:*mut u64){
    #[cfg(feature="count-allocations")] unsafe {core::ptr::copy_nonoverlapping(counter::stop().as_ptr(),out,3);}
    #[cfg(not(feature="count-allocations"))] let _=out;
}
