//! Checker-only decomposition: numeric addresses, never dereferenced.
#[path = "c-algorithms.rs"]
mod algorithms;
#[path = "../../bench-c-collections/allocations.rs"]
mod allocations;
#[path = "c-prototypes.rs"]
mod original;
use algorithms::Address;
use std::{hint::black_box, time::Instant};
type Groups = [Vec<Address>; 5];
fn scan(values: &[Address]) -> bool {
    !values
        .windows(2)
        .any(|w| w[0].0 == w[1].0 && (w[0].1 || w[1].1))
}
fn flatten(groups: &Groups) -> Vec<Address> {
    let mut v = Vec::with_capacity(groups.iter().map(Vec::len).sum());
    for g in groups {
        v.extend_from_slice(g);
    }
    v
}
fn check(kind: usize, groups: &Groups) -> bool {
    if kind == 6 {
        return algorithms::stable(&mut flatten(black_box(groups))).is_ok();
    }
    if kind == 0 {
        return original::sorted(&mut flatten(black_box(groups))).is_ok();
    }
    if kind == 5 {
        let token = groups[4][0].0;
        return groups[..4].iter().flatten().all(|v| v.0 != token);
    }
    let mut values = black_box(groups).clone();
    match kind {
        1 => original::buckets(&mut values, true).is_ok(),
        2 => {
            for g in &mut values {
                g.sort_unstable_by_key(|v| v.0);
            }
            algorithms::merge_original(&values).is_ok()
        }
        3 => algorithms::compact(&mut values).is_ok(),
        4 => algorithms::pairs(&mut values).is_ok(),
        _ => unreachable!(),
    }
}
fn equivalent(groups: &Groups) {
    let expected = check(0, groups);
    for kind in [1, 2, 3, 4, 6] {
        assert_eq!(check(kind, groups), expected, "checker {kind}: {groups:?}");
    }
}
fn tests() {
    for len in 0..=3 {
        let count = 30usize.pow(len);
        for mut code in 0..count {
            let mut g: Groups = std::array::from_fn(|_| Vec::new());
            for _ in 0..len {
                let v = code % 30;
                code /= 30;
                g[v % 5].push(([0, 1, usize::MAX][(v / 5) % 3], v / 15 != 0));
            }
            equivalent(&g);
        }
    }
    let mut rng = 51u64;
    for _ in 0..2000 {
        let mut g: Groups = std::array::from_fn(|_| Vec::new());
        for _ in 0..32 {
            let x = random(&mut rng);
            g[(x % 5) as usize].push((((x >> 8) % 32) as usize, x & 64 != 0));
        }
        equivalent(&g);
    }
    // Large valid sets exercise successful intersections, then exactly one
    // injected cross-group conflict. Both runs have more than one element.
    for seed in 0..1000u64 {
        let mut g: Groups = std::array::from_fn(|_| Vec::new());
        for i in 0..32 {
            g[(i % 5) as usize].push((16 + i * 16, (i as u64 + seed) % 3 == 0));
        }
        assert!(check(0, &g));
        equivalent(&g);
        let address = g[0][0].0;
        g[1][0] = (address, true);
        assert!(!check(0, &g));
        equivalent(&g);
        let mut token = input(32, 4, "shuffled", true, seed);
        assert_eq!(check(5, &token), check(0, &token));
        token[4][0].0 = token[2][0].0;
        assert!(!check(5, &token));
        assert_eq!(check(5, &token), check(0, &token));
    }
    println!("CHECKS_OK");
}
fn random(r: &mut u64) -> u64 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    *r
}
fn input(n: usize, k: usize, layout: &str, shared: bool, seed: u64) -> Groups {
    let mut groups: Groups = std::array::from_fn(|_| Vec::new());
    let mut rng = seed + 41;
    let spatial = layout == "random_sorted" || layout == "random_shuffled";
    let mut ranks: Vec<usize> = if spatial {
        (0..n).collect()
    } else {
        Vec::new()
    };
    if spatial {
        for i in (1..n).rev() {
            let j = random(&mut rng) as usize % (i + 1);
            ranks.swap(i, j);
        }
    }
    for (g, group) in groups[..k].iter_mut().enumerate() {
        let len = n / k + usize::from(g < n % k);
        for i in 0..len {
            let rank = if layout == "blocked" {
                g * (n / k) + g.min(n % k) + i
            } else {
                i * k + g
            };
            let rank = if spatial {
                ranks[g * (n / k) + g.min(n % k) + i]
            } else {
                rank
            };
            group.push((0x10000 + rank * 16, !shared));
        }
        match layout {
            "shuffled" => {
                for i in (1..len).rev() {
                    let j = (random(&mut rng) as usize) % (i + 1);
                    group.swap(i, j);
                }
            }
            "near" => {
                for _ in 0..len / 100 {
                    let a = random(&mut rng) as usize % len;
                    let b = random(&mut rng) as usize % len;
                    group.swap(a, b);
                }
            }
            "reverse" => group.reverse(),
            "random_sorted" => group.sort_unstable_by_key(|v| v.0),
            "random_shuffled" | "striped" | "blocked" => (),
            _ => panic!("unknown layout"),
        }
    }
    if shared {
        groups[4].push((0x10000 + (n / 2) * 16 + 8, true));
    }
    groups
}
fn stages(kind: usize, source: &Groups, it: usize) -> [u128; 5] {
    let mut result = [0; 5];
    for _ in 0..it {
        let begin = Instant::now();
        let mut groups: Groups = std::array::from_fn(|_| Vec::new());
        let mut flat = Vec::new();
        if kind == 0 {
            flat = flatten(black_box(source));
        } else {
            groups = black_box(source).clone();
        }
        result[0] += begin.elapsed().as_nanos();
        let begin = Instant::now();
        if kind == 0 {
            flat.sort_unstable_by_key(|v| v.0);
        } else {
            for g in &mut groups {
                g.sort_unstable_by_key(|v| v.0);
            }
        }
        result[1] += begin.elapsed().as_nanos();
        let begin = Instant::now();
        if kind == 0 {
            assert!(scan(black_box(&flat)));
        } else if kind == 1 || kind == 4 {
            for g in &groups {
                assert!(scan(black_box(g)));
            }
        }
        result[2] += begin.elapsed().as_nanos();
        let begin = Instant::now();
        let valid = match kind {
            0 => true,
            1 if groups.iter().filter(|g| !g.is_empty()).count() <= 1 => true,
            1 | 2 => algorithms::merge_original(&groups).is_ok(),
            3 => algorithms::merge_compact(&groups).is_ok(),
            4 => algorithms::intersections(&groups).is_ok(),
            _ => unreachable!(),
        };
        assert!(black_box(valid));
        result[3] += begin.elapsed().as_nanos();
        let begin = Instant::now();
        drop(black_box(groups));
        drop(black_box(flat));
        result[4] += begin.elapsed().as_nanos();
    }
    result
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 2 && args[1] == "--check" {
        tests();
        return;
    }
    assert_eq!(args.len(), 9);
    let n = args[1].parse::<usize>().unwrap();
    let k = args[2].parse::<usize>().unwrap();
    assert!(k <= 4 && k > 0);
    let layout = &args[3];
    let shared = &args[4] == "shared";
    let fork = args[5].parse::<usize>().unwrap();
    let batches = args[6].parse::<usize>().unwrap();
    let target = args[7].parse::<u128>().unwrap() * 1_000_000;
    let aa = &args[8] == "aa";
    let groups = input(n, k, layout, shared, fork as u64);
    for v in 0..5 {
        assert!(check(v, &groups));
    }
    if shared {
        assert!(check(5, &groups));
    }
    let mut flat = flatten(&groups);
    let mut flat_cmp = 0usize;
    flat.sort_unstable_by(|a, b| {
        flat_cmp += 1;
        a.0.cmp(&b.0)
    });
    let mut bucket_cmp = 0usize;
    for g in groups.clone().iter_mut() {
        g.sort_unstable_by(|a, b| {
            bucket_cmp += 1;
            a.0.cmp(&b.0)
        });
    }
    let active = groups.iter().filter(|g| !g.is_empty()).count();
    let mut stable_cmp = 0usize;
    let mut stable = flatten(&groups);
    stable.sort_by(|a, b| {
        stable_cmp += 1;
        a.0.cmp(&b.0)
    });
    println!("COUNTS,{flat_cmp},{bucket_cmp},{active},{stable_cmp}");
    let mut it = (10000 / (n + 1)).max(1);
    for _ in 0..4 {
        let start = Instant::now();
        for _ in 0..it {
            assert!(black_box(check(0, black_box(&groups))));
        }
        let ns = start.elapsed().as_nanos().max(1);
        it = ((it as f64 * (target as f64 / ns as f64).clamp(0.25, 8.0)) as usize)
            .clamp(1, 1_000_000);
    }
    let variants: Vec<usize> = if shared {
        vec![0, 1, 2, 3, 4, 5, 6]
    } else {
        vec![0, 1, 2, 3, 4, 6]
    };
    if allocations::bench_counter_enabled() {
        for &kind in &variants {
            check(kind, &groups);
            allocations::bench_counter_start();
            for _ in 0..10 {
                assert!(black_box(check(kind, &groups)));
            }
            let mut out = [0u64; 3];
            unsafe {
                allocations::bench_counter_stop(out.as_mut_ptr());
            }
            println!(
                "ALLOC,{kind},{},{},{}",
                out[0] / 10,
                out[1] / 10,
                out[2] / 10
            );
        }
        return;
    }
    for &v in &variants {
        for _ in 0..it {
            assert!(black_box(check(if aa { 0 } else { v }, &groups)));
        }
    }
    for batch in 0..batches {
        for step in 0..variants.len() {
            let index = (step + batch + fork) % variants.len();
            let v = variants[if (batch + fork) % 2 == 0 {
                index
            } else {
                variants.len() - 1 - index
            }];
            let kind = if aa { 0 } else { v };
            let start = Instant::now();
            for _ in 0..it {
                assert!(black_box(check(kind, black_box(&groups))));
            }
            println!("TIME,{v},{batch},{it},{}", start.elapsed().as_nanos());
        }
    }
    // Stage timers are separate diagnostics, never used as end-to-end latency.
    for v in 0..5 {
        let kind = if aa { 0 } else { v };
        let sample = stages(kind, &groups, it.min(50));
        println!(
            "STAGES,{v},{},{},{},{},{},{}",
            it.min(50),
            sample[0],
            sample[1],
            sample[2],
            sample[3],
            sample[4]
        );
    }
}
