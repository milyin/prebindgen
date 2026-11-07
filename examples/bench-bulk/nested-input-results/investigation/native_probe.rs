#![allow(dead_code, unused_variables, non_camel_case_types, clippy::all)]
use std::{hint::black_box, time::Instant};
mod bench_flat {
    pub struct InputHandle {
        pub value: i64,
    }
}
include!("decoders.rs");
fn frame(n: usize, groups: usize, order: usize) -> Vec<u8> {
    let mut ptrs = (0..n)
        .map(|i| {
            Box::into_raw(Box::new(prebindgen_jni_runtime::Allocation {
                value: bench_flat::InputHandle { value: i as i64 },
            })) as usize
        })
        .collect::<Vec<_>>();
    ptrs.sort_unstable();
    if order == 1 {
        ptrs.reverse();
    }
    if order == 2 {
        let mut seed = 42u64;
        for i in (1..n).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            ptrs.swap(i, seed as usize % (i + 1));
        }
    }
    let start = 16 + 16 * groups;
    let mut bytes = vec![0; start + 8 * n];
    fn put(b: &mut [u8], at: usize, x: usize) {
        b[at..at + 8].copy_from_slice(&(x as u64).to_ne_bytes());
    }
    put(&mut bytes, 0, 16);
    put(&mut bytes, 8, groups);
    for i in 0..groups {
        put(&mut bytes, 16 + 16 * i, start + 8 * (i * n / groups));
        put(&mut bytes, 24 + 16 * i, n / groups);
    }
    for (i, ptr) in ptrs.into_iter().enumerate() {
        put(&mut bytes, start + 8 * i, ptr);
    }
    bytes
}
type Decode = fn(&[u8]) -> Result<Vec<Vec<bench_flat::InputHandle>>, String>;
fn main() {
    let cases: [(&str, Decode); 6] = [
        ("staged", staged::decode),
        ("reread", reread::decode),
        ("pointer_once", pointer_once::decode),
        ("checks_once", checks_once::decode),
        ("staged_no_alias", staged_no_alias::decode),
        ("reread_no_alias", reread_no_alias::decode),
    ];
    for batch in 0..9 {
        for (shape, n, g) in [
            ("one", 512, 1),
            ("pairs", 512, 256),
            ("quads", 512, 128),
            ("many", 512, 512),
        ] {
            for order in 0..3 {
                for step in 0..cases.len() {
                    let idx = if batch % 2 == 0 {
                        step
                    } else {
                        cases.len() - 1 - step
                    };
                    let (name, decode) = cases[idx];
                    let mut samples = 0u64;
                    let mut ns = 0u64;
                    while ns < 20_000_000 || samples < 30 {
                        let bytes = frame(n, g, order);
                        let begin = Instant::now();
                        let result = decode(black_box(&bytes)).unwrap();
                        let time = begin.elapsed();
                        ns += time.as_nanos() as u64;
                        samples += 1;
                        assert_eq!(result.iter().map(|r| r.len()).sum::<usize>(), n);
                        assert_eq!(
                            result.iter().flatten().map(|h| h.value).sum::<i64>(),
                            (n * (n - 1) / 2) as i64
                        );
                        black_box(&result);
                        drop(result);
                    }
                    println!("PROBE,{batch},{shape},{order},{name},{samples},{ns}");
                }
            }
        }
    }
}
