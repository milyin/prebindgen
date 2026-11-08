//! Isolated validator lifetimes; fixtures and pointer ordering are outside timing.
use std::{hint::black_box, time::Instant};
type Result<T> = std::result::Result<T, String>;
#[path = "prototypes.rs"]
mod experiment;
#[path = "../../../prebindgen-runtime/src/input.rs"]
mod production;
type Address = (usize, bool);
#[derive(Clone)]
struct Case {
    groups: Vec<Vec<Address>>,
    prefix: Vec<Address>,
    shared_only: bool,
}
#[inline(always)]
fn run(kind: &str, c: &Case) -> bool {
    if c.shared_only {
        return true;
    }
    match kind {
        "current" => {
            let mut history = production::DynamicInputs::default();
            for &(a, e) in &c.prefix {
                if history.access(a, e).is_err() {
                    return false;
                }
            }
            for g in &c.groups {
                let mut pending = history.begin_group(g.len());
                for &entry in g {
                    pending.push(entry);
                }
                if history.accept_group(pending).is_err() {
                    return false;
                }
            }
            black_box(history);
            true
        }
        "hash" => {
            let mut history = experiment::OnlineHash::default();
            for &(a, e) in &c.prefix {
                if history.access(a, e).is_err() {
                    return false;
                }
            }
            for g in &c.groups {
                history.reserve_group(g.len());
                for &(a, e) in g {
                    if history.access(a, e).is_err() {
                        return false;
                    }
                }
            }
            black_box(history);
            true
        }
        "sort" | "sort_sized" => {
            let mut values = Vec::new();
            if kind == "sort_sized" {
                values.reserve_exact(c.prefix.len() + c.groups.iter().map(Vec::len).sum::<usize>());
            }
            for &value in &c.prefix {
                values.push(value);
            }
            for g in &c.groups {
                for &entry in g {
                    values.push(entry);
                }
            }
            let valid = experiment::check_sorted(&mut values).is_ok();
            black_box(values);
            valid
        }
        _ => panic!("unknown candidate"),
    }
}
fn make(n: usize, shape: &str, order: &str) -> Case {
    let mut entries: Vec<_> = (0..n).map(|i| ((i + 1) * 64, shape != "shared")).collect();
    match order {
        "descending" => entries.reverse(),
        "shuffled" => {
            let mut state = 20261108u64;
            for i in (1..n).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                entries.swap(i, state as usize % (i + 1));
            }
        }
        "ascending" => {}
        _ => panic!(),
    }
    let chunk = match shape {
        "flat" => n.max(1),
        "groups8" => 8,
        _ => 1,
    };
    Case {
        groups: entries.chunks(chunk).map(|g| g.to_vec()).collect(),
        prefix: if shape == "shared" {
            vec![(n.saturating_add(2) * 64, true)]
        } else {
            vec![]
        },
        shared_only: shape == "readonly",
    }
}
fn verify() {
    let mut state = 7u64;
    let mut valid_count = 0;
    let mut invalid_count = 0;
    for test in 0..10000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let n = state as usize % 1023 + 2;
        let shape = match test % 3 {
            0 => "flat",
            1 => "singleton",
            _ => "groups8",
        };
        let mut c = make(n, shape, "shuffled");
        let valid = test % 2 == 0;
        if !valid {
            let first = c.groups[0][0];
            *c.groups.last_mut().unwrap().last_mut().unwrap() = first;
            invalid_count += 1;
        } else {
            valid_count += 1;
        }
        for kind in ["current", "hash", "sort", "sort_sized"] {
            assert_eq!(run(kind, &c), valid, "{kind}");
        }
    }
    assert_eq!((valid_count, invalid_count), (5000, 5000));
    for n in [128, 129, 512, 8192] {
        let mut shared = make(n, "shared", "shuffled");
        let first = shared.groups[0][0];
        shared.groups.last_mut().unwrap()[0] = first;
        for kind in ["current", "hash", "sort", "sort_sized"] {
            assert!(run(kind, &shared));
        }
        shared.groups.last_mut().unwrap()[0] = shared.prefix[0];
        for kind in ["current", "hash", "sort", "sort_sized"] {
            assert!(!run(kind, &shared));
        }
    }
    println!("VERIFY,5000 valid unique cases,5000 single-conflict cases,and shared repeats across thresholds");
}
#[cfg(counts)]
mod allocation {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };
    thread_local! {static ACTIVE:Cell<bool>=const{Cell::new(false)};static VALUES:Cell<[usize;5]>=const{Cell::new([0;5])};}
    struct Counter;
    #[global_allocator]
    static ALLOCATOR: Counter = Counter;
    fn update(old: usize, new: usize, realloc: bool) {
        if ACTIVE.try_with(Cell::get).unwrap_or(false) {
            let _ = VALUES.try_with(|s| {
                let mut v = s.get();
                if new != 0 {
                    v[usize::from(realloc)] += 1;
                    v[2] += new;
                }
                v[3] = v[3] + new - old;
                v[4] = v[4].max(v[3]);
                s.set(v);
            });
        }
    }
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let p = unsafe { System.alloc(l) };
            if !p.is_null() {
                update(0, l.size(), false);
            }
            p
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            let p = unsafe { System.alloc_zeroed(l) };
            if !p.is_null() {
                update(0, l.size(), false);
            }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            update(l.size(), 0, false);
            unsafe { System.dealloc(p, l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            let q = unsafe { System.realloc(p, l, n) };
            if !q.is_null() {
                update(l.size(), n, true);
            }
            q
        }
    }
    pub fn start() {
        VALUES.with(|s| s.set([0; 5]));
        ACTIVE.with(|s| s.set(true));
    }
    pub fn stop() -> [usize; 5] {
        ACTIVE.with(|s| s.set(false));
        VALUES.with(Cell::get)
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args[1] == "verify" {
        verify();
        return;
    }
    let kind = &args[2];
    let n: usize = args[3].parse().unwrap();
    let c = make(n, &args[4], &args[5]);
    assert!(run(kind, &c));
    let action: fn(&Case) -> bool = match kind.as_str() {
        "current" => |c| run("current", c),
        "hash" => |c| run("hash", c),
        "sort" => |c| run("sort", c),
        "sort_sized" => |c| run("sort_sized", c),
        _ => panic!(),
    };
    #[cfg(counts)]
    {
        assert_eq!(args[1], "count");
        allocation::start();
        assert!(run(kind, &c));
        let action: fn(&Case) -> bool = match kind.as_str() {
            "current" => |c| run("current", c),
            "hash" => |c| run("hash", c),
            "sort" => |c| run("sort", c),
            "sort_sized" => |c| run("sort_sized", c),
            _ => panic!(),
        };
        let v = allocation::stop();
        assert_eq!(v[3], 0);
        println!("COUNT,{},{},{},{}", v[0], v[1], v[2], v[4]);
    }
    #[cfg(not(counts))]
    {
        assert_eq!(args[1], "time");
        let target = args[7].parse::<u64>().unwrap() * 1_000_000;
        let batches: usize = args[6].parse().unwrap();
        let mut it = 8usize;
        for _ in 0..3 {
            let t = Instant::now();
            for _ in 0..it {
                assert!(black_box(action)(black_box(&c)));
            }
            let scale = (target as f64 / t.elapsed().as_nanos().max(1) as f64).clamp(0.25, 8.0);
            it = ((it as f64 * scale) as usize).clamp(1, 1_000_000);
        }
        for _ in 0..3 {
            for _ in 0..it {
                assert!(black_box(action)(black_box(&c)));
            }
        }
        for batch in 0..batches {
            let t = Instant::now();
            for _ in 0..it {
                assert!(black_box(action)(black_box(&c)));
            }
            println!("TIME,{batch},{it},{}", t.elapsed().as_nanos());
        }
    }
}
