// Standalone experiment; run.py supplies the actual baseline/current runtime module.
use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasherDefault, Hasher},
    hint::black_box,
    time::Instant,
};
type Result<T> = std::result::Result<T, String>;
type Address = (usize, bool);
trait Validator: Default {
    type Group;
    fn begin(&self, capacity: usize) -> Self::Group;
    fn push(group: &mut Self::Group, entry: Address);
    fn accept(&mut self, group: Self::Group) -> bool;
    fn access(&mut self, entry: Address) -> bool;
}
#[derive(Default)]
struct Current(runtime::DynamicInputs);
impl Validator for Current {
    type Group = runtime::InputGroup;
    fn begin(&self, capacity: usize) -> Self::Group {
        self.0.begin_group(capacity)
    }
    fn push(group: &mut Self::Group, entry: Address) {
        group.push(entry);
    }
    fn accept(&mut self, group: Self::Group) -> bool {
        self.0.accept_group(group).is_ok()
    }
    fn access(&mut self, entry: Address) -> bool {
        self.0.access(entry.0, entry.1).is_ok()
    }
}
#[derive(Default)]
struct Selected<const LIMIT: usize = {production::DEFAULT_INPUT_ALIAS_INDEX_THRESHOLD}>(production::DynamicInputsWithThreshold<LIMIT>);
impl<const LIMIT: usize> Validator for Selected<LIMIT> {
    type Group = production::InputGroup;
    fn begin(&self, n: usize) -> Self::Group {
        self.0.begin_group(n)
    }
    fn push(g: &mut Self::Group, e: Address) {
        g.push(e);
    }
    fn accept(&mut self, g: Self::Group) -> bool {
        self.0.accept_group(g).is_ok()
    }
    fn access(&mut self, e: Address) -> bool {
        self.0.access(e.0, e.1).is_ok()
    }
}
struct Group {
    entries: Vec<Address>,
    capacity: usize,
}
impl Group {
    fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }
    fn push(&mut self, entry: Address) {
        if self.entries.capacity() == 0 {
            self.entries.reserve_exact(self.capacity);
        }
        self.entries.push(entry);
    }
}
fn internal(entries: &mut Vec<Address>) -> bool {
    entries.sort_unstable_by_key(|e| e.0);
    if entries
        .windows(2)
        .any(|p| p[0].0 == p[1].0 && (p[0].1 || p[1].1))
    {
        return false;
    }
    entries.dedup_by_key(|e| e.0);
    true
}
#[derive(Default)]
struct Linear(Vec<Address>);
impl Validator for Linear {
    type Group = Group;
    fn begin(&self, capacity: usize) -> Group {
        Group::new(capacity)
    }
    fn push(group: &mut Group, entry: Address) {
        group.push(entry);
    }
    fn accept(&mut self, mut group: Group) -> bool {
        if !internal(&mut group.entries) {
            return false;
        }
        let mut added = 0;
        for i in 0..group.entries.len() {
            let e = group.entries[i];
            if let Some(p) = self.0.iter().find(|p| p.0 == e.0) {
                if p.1 || e.1 {
                    return false;
                }
            } else {
                group.entries[added] = e;
                added += 1;
            }
        }
        group.entries.truncate(added);
        if self.0.is_empty() {
            self.0 = group.entries;
        } else {
            self.0.extend(group.entries);
        }
        true
    }
    fn access(&mut self, e: Address) -> bool {
        if let Some(p) = self.0.iter().find(|p| p.0 == e.0) {
            return !(p.1 || e.1);
        }
        self.0.push(e);
        true
    }
}
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
#[derive(Default)]
struct AddressHasher(u64);
impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only usize keys")
    }
    fn write_usize(&mut self, value: usize) {
        self.0 = mix(value as u64);
    }
}
type FastMap = HashMap<usize, u64, BuildHasherDefault<AddressHasher>>;
trait Store: Default {
    fn add(&mut self, address: usize, stamp: u64) -> bool;
    fn rollback(&mut self, stamp: u64);
    fn reserve(&mut self, capacity: usize);
}
macro_rules! store {
    ($ty:ty, $entry:path, $reserve:expr) => {
        impl Store for $ty {
            fn add(&mut self, address: usize, stamp: u64) -> bool {
                use $entry as Entry;
                match self.entry(address) {
                    Entry::Vacant(v) => {
                        v.insert(stamp);
                        true
                    }
                    Entry::Occupied(v) => (v.get() | stamp) & 1 == 0,
                }
            }
            fn rollback(&mut self, stamp: u64) {
                self.retain(|_, value| *value & !1 != stamp);
            }
            fn reserve(&mut self, capacity: usize) {
                ($reserve)(self, capacity);
            }
        }
    };
}
store!(HashMap<usize, u64>, std::collections::hash_map::Entry, |map: &mut HashMap<usize,u64>,n| map.reserve(n));
store!(
    FastMap,
    std::collections::hash_map::Entry,
    |map: &mut FastMap, n| map.reserve(n)
);
store!(BTreeMap<usize, u64>, std::collections::btree_map::Entry, |_: &mut BTreeMap<usize,u64>,_| {});
struct Indexed<S> {
    map: S,
    stamp: u64,
}
impl<S: Default> Default for Indexed<S> {
    fn default() -> Self {
        Self {
            map: S::default(),
            stamp: 0,
        }
    }
}
impl<S: Store> Validator for Indexed<S> {
    type Group = Group;
    fn begin(&self, capacity: usize) -> Group {
        Group::new(capacity)
    }
    fn push(group: &mut Group, entry: Address) {
        group.push(entry);
    }
    fn accept(&mut self, group: Group) -> bool {
        self.stamp += 2;
        self.map.reserve(group.entries.len());
        for (address, exclusive) in group.entries {
            if !self.map.add(address, self.stamp | u64::from(exclusive)) {
                self.map.rollback(self.stamp);
                return false;
            }
        }
        true
    }
    fn access(&mut self, entry: Address) -> bool {
        self.map.add(entry.0, self.stamp | u64::from(entry.1))
    }
}
#[cfg(metrics)]
mod metrics {
    use std::sync::atomic::AtomicUsize;
    pub static QUERIES: AtomicUsize = AtomicUsize::new(0);
    pub static POSITIVES: AtomicUsize = AtomicUsize::new(0);
    pub static FALSE_POSITIVES: AtomicUsize = AtomicUsize::new(0);
    pub static EXACT_VISITS: AtomicUsize = AtomicUsize::new(0);
}
fn filtered_lookup(entries: &[Address], address: usize, maybe: bool) -> Option<bool> {
    #[cfg(metrics)]
    {
        metrics::QUERIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    if !maybe {
        return None;
    }
    let position = entries.iter().position(|p| p.0 == address);
    #[cfg(metrics)]
    {
        use std::sync::atomic::Ordering::Relaxed;
        metrics::POSITIVES.fetch_add(1, Relaxed);
        metrics::EXACT_VISITS.fetch_add(position.map_or(entries.len(), |i| i + 1), Relaxed);
        if position.is_none() {
            metrics::FALSE_POSITIVES.fetch_add(1, Relaxed);
        }
    }
    position.map(|i| entries[i].1)
}
struct Bloom<const WORDS: usize, const BLOCKED: bool> {
    bits: [u64; WORDS],
    entries: Vec<Address>,
}
impl<const N: usize, const B: bool> Default for Bloom<N, B> {
    fn default() -> Self {
        Self {
            bits: [0; N],
            entries: Vec::new(),
        }
    }
}
impl<const N: usize, const B: bool> Bloom<N, B> {
    fn positions(address: usize) -> [(usize, u64); 3] {
        let h = mix(address as u64);
        if B {
            let word = (h >> 32) as usize & (N - 1);
            [
                (word, 1 << (h & 63)),
                (word, 1 << ((h >> 6) & 63)),
                (word, 1 << ((h >> 12) & 63)),
            ]
        } else {
            [0, 21, 42].map(|shift| {
                let bit = (h >> shift) as usize & (N * 64 - 1);
                (bit >> 6, 1 << (bit & 63))
            })
        }
    }
    fn may_contain(&self, address: usize) -> bool {
        Self::positions(address)
            .iter()
            .all(|&(w, b)| self.bits[w] & b != 0)
    }
    fn add(&mut self, entry: Address) {
        for (w, b) in Self::positions(entry.0) {
            self.bits[w] |= b;
        }
        self.entries.push(entry);
    }
    fn conflict(&self, entry: Address) -> bool {
        self.may_contain(entry.0)
            && self
                .entries
                .iter()
                .any(|p| p.0 == entry.0 && (p.1 || entry.1))
    }
}
impl<const N: usize, const B: bool> Validator for Bloom<N, B> {
    type Group = Group;
    fn begin(&self, capacity: usize) -> Group {
        Group::new(capacity)
    }
    fn push(group: &mut Group, entry: Address) {
        group.push(entry);
    }
    fn accept(&mut self, mut group: Group) -> bool {
        if !internal(&mut group.entries) {
            return false;
        }
        let mut added = 0;
        for i in 0..group.entries.len() {
            let entry = group.entries[i];
            let previous = filtered_lookup(&self.entries, entry.0, self.may_contain(entry.0));
            if let Some(p) = previous {
                if p || entry.1 {
                    return false;
                }
            } else {
                group.entries[added] = entry;
                added += 1;
            }
        }
        group.entries.truncate(added);
        for entry in group.entries {
            self.add(entry);
        }
        true
    }
    fn access(&mut self, entry: Address) -> bool {
        if let Some(previous) = filtered_lookup(&self.entries, entry.0, self.may_contain(entry.0)) {
            return !(previous || entry.1);
        }
        self.add(entry);
        true
    }
}
// Lazy, growing Bloom filter over an exact append-only vector. The first
// complete batch retains its allocation without building a filter.
#[derive(Default)]
struct GrowingBloom {
    bits: Vec<u64>,
    entries: Vec<Address>,
}
impl GrowingBloom {
    fn positions(&self, a: usize) -> [(usize, u64); 3] {
        let h = mix(a as u64);
        let mask = self.bits.len() * 64 - 1;
        [0, 21, 42].map(|s| {
            let bit = (h >> s) as usize & mask;
            (bit >> 6, 1 << (bit & 63))
        })
    }
    fn insert_bits(&mut self, a: usize) {
        for (w, b) in self.positions(a) {
            self.bits[w] |= b;
        }
    }
    fn ensure(&mut self, additional: usize) {
        let expected = self.entries.len().saturating_add(additional);
        if self.bits.is_empty() || expected * 12 > self.bits.len() * 64 {
            let words = expected
                .saturating_mul(16)
                .div_ceil(64)
                .max(8)
                .next_power_of_two();
            self.bits = vec![0; words];
            for i in 0..self.entries.len() {
                self.insert_bits(self.entries[i].0);
            }
        }
    }
    fn previous(&self, a: usize) -> Option<bool> {
        let maybe = self
            .positions(a)
            .iter()
            .all(|&(w, b)| self.bits[w] & b != 0);
        filtered_lookup(&self.entries, a, maybe)
    }
}
impl Validator for GrowingBloom {
    type Group = Group;
    fn begin(&self, c: usize) -> Group {
        Group::new(c)
    }
    fn push(g: &mut Group, e: Address) {
        g.push(e);
    }
    fn accept(&mut self, mut g: Group) -> bool {
        if !internal(&mut g.entries) {
            return false;
        }
        if self.entries.is_empty() {
            self.entries = g.entries;
            return true;
        }
        if g.entries.is_empty() {
            return true;
        }
        self.ensure(g.entries.len());
        let mut added = 0;
        for i in 0..g.entries.len() {
            let e = g.entries[i];
            if let Some(p) = self.previous(e.0) {
                if p || e.1 {
                    return false;
                }
            } else {
                g.entries[added] = e;
                added += 1;
            }
        }
        g.entries.truncate(added);
        for e in g.entries {
            self.insert_bits(e.0);
            self.entries.push(e);
        }
        true
    }
    fn access(&mut self, e: Address) -> bool {
        self.ensure(1);
        if let Some(p) = self.previous(e.0) {
            return !(p || e.1);
        }
        self.insert_bits(e.0);
        self.entries.push(e);
        true
    }
}
fn sorted_accept(values: &mut Vec<Address>, group: Vec<Address>) -> bool {
    let mut group = group;
    if group.is_empty() {
        // Previously accepted accesses are already sorted and checked.
        return true;
    }
    if group.len() == 1 && !values.is_empty() {
        // No internal duplicate is possible; only history can conflict.
        let (address, exclusive) = group[0];
        return sorted_access(values, (address, exclusive));
    }
    // Sort only the new addresses. Re-sorting/scanning the complete history
    // for every inner list makes nested input cost depend heavily on the
    // allocator's address order, even though earlier groups are unchanged.
    group.sort_unstable_by_key(|entry| entry.0);
    for pair in group.windows(2) {
        if pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1) {
            return false;
        }
    }
    group.dedup_by_key(|entry| entry.0);
    if values.is_empty() {
        // Reuse the first batch's allocation as the history.
        *values = group;
        return true;
    }
    if group[0].0 > values.last().expect("nonempty history").0 {
        // Disjoint ascending batches can append without any history scan.
        values.append(&mut group);
        return true;
    }
    let mut added = 0;
    for i in 0..group.len() {
        let (address, exclusive) = group[i];
        match values.binary_search_by_key(&address, |entry| entry.0) {
            Ok(previous) if exclusive || values[previous].1 => {
                return false;
            }
            Ok(_) => {} // A repeated shared access needs no new entry.
            Err(_) => {
                group[added] = group[i];
                added += 1;
            }
        }
    }
    group.truncate(added);
    // Merge backwards into the existing allocation. The already ordered
    // prefix stays in place, and no temporary merged vector is needed.
    let mut previous = values.len();
    values.resize(previous + added, (0, false));
    let mut write = values.len();
    for entry in group.into_iter().rev() {
        while previous > 0 && values[previous - 1].0 > entry.0 {
            previous -= 1;
            write -= 1;
            values[write] = values[previous];
        }
        write -= 1;
        values[write] = entry;
    }
    true
}

fn sorted_access(values: &mut Vec<Address>, e: Address) -> bool {
    match values.binary_search_by_key(&e.0, |p| p.0) {
        Ok(i) => !(values[i].1 || e.1),
        Err(i) => {
            values.insert(i, e);
            true
        }
    }
}
// Keep the exact sorted-vector algorithm for one batch/small histories, then
// promote to the integer-hashed map when another group exceeds the threshold.
enum Hybrid<const LIMIT: usize> {
    Sorted(Vec<Address>),
    Indexed(Indexed<FastMap>),
}
impl<const L: usize> Default for Hybrid<L> {
    fn default() -> Self {
        Self::Sorted(Vec::new())
    }
}
impl<const L: usize> Hybrid<L> {
    fn promote(&mut self) {
        if let Self::Sorted(values) = self {
            let mut index = Indexed::<FastMap>::default();
            index.map.reserve(values.len());
            for (a, e) in values.drain(..) {
                assert!(index.access((a, e)));
            }
            *self = Self::Indexed(index);
        }
    }
}
impl<const L: usize> Validator for Hybrid<L> {
    type Group = Group;
    fn begin(&self, capacity: usize) -> Group {
        Group::new(capacity)
    }
    fn push(group: &mut Group, entry: Address) {
        group.push(entry);
    }
    fn accept(&mut self, group: Group) -> bool {
        if let Self::Sorted(values) = self {
            if !values.is_empty()
                && !group.entries.is_empty()
                && values.len().saturating_add(group.entries.len()) > L
            {
                self.promote();
            }
        }
        match self {
            Self::Indexed(index) => index.accept(group),
            Self::Sorted(values) => sorted_accept(values, group.entries),
        }
    }
    fn access(&mut self, entry: Address) -> bool {
        if let Self::Sorted(values) = self {
            if let Ok(i) = values.binary_search_by_key(&entry.0, |p| p.0) {
                return !(values[i].1 || entry.1);
            }
            if values.len() >= L {
                self.promote();
            }
        }
        match self {
            Self::Indexed(index) => index.access(entry),
            Self::Sorted(values) => sorted_access(values, entry),
        }
    }
}
#[derive(Clone)]
struct Case {
    name: String,
    groups: Vec<Vec<Address>>,
    accesses: Vec<Address>,
    expected: bool,
}
fn run<V: Validator>(case: &Case) -> bool {
    let mut validator = V::default();
    for &entry in &case.accesses {
        if !validator.access(black_box(entry)) {
            return false;
        }
    }
    for entries in &case.groups {
        let mut group = validator.begin(entries.len());
        for &entry in entries {
            V::push(&mut group, black_box(entry));
        }
        if !validator.accept(group) {
            return false;
        }
    }
    black_box(&validator);
    true
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for n in [0, 1, 4, 16, 32, 64, 128, 256, 512, 1024, 2048, 8192] {
        for shape in ["batch", "quads", "singletons", "access"] {
            for order in ["ascending", "descending", "shuffled", "scattered"] {
                let mut entries = (0..n)
                    .map(|i| (0x100000 + i * 64, i % 4 == 0))
                    .collect::<Vec<_>>();
                if order == "scattered" {
                    for e in &mut entries {
                        e.0 = (mix(e.0 as u64) as usize) & !63;
                    }
                }
                if order == "descending" {
                    entries.reverse();
                }
                if order == "shuffled" {
                    let mut seed = 42u64;
                    for i in (1..n).rev() {
                        seed = mix(seed.wrapping_add(i as u64));
                        entries.swap(i, seed as usize % (i + 1));
                    }
                }
                let (accesses, groups) = if shape == "access" {
                    (entries, vec![])
                } else {
                    let size = match shape {
                        "batch" => n.max(1),
                        "quads" => 4,
                        _ => 1,
                    };
                    (vec![], entries.chunks(size).map(|r| r.to_vec()).collect())
                };
                cases.push(Case {
                    name: format!("{n}/{shape}/{order}/unique"),
                    accesses,
                    groups,
                    expected: true,
                });
            }
        }
        // Actual shared repeats in a call that also includes an exclusive input.
        let entries = (0..n)
            .map(|i| (0x100000 + (i % n.clamp(1, 16)) * 64, false))
            .collect::<Vec<_>>();
        cases.push(Case {
            name: format!("{n}/singletons/repeated/shared"),
            accesses: vec![(0x800000, true)],
            groups: entries.into_iter().map(|e| vec![e]).collect(),
            expected: true,
        });
    }
    for n in [16, 512, 8192] {
        for shape in ["batch", "singletons"] {
            let mut entries = (0..n)
                .map(|i| (0x100000 + i * 64, true))
                .collect::<Vec<_>>();
            entries.push(entries[0]);
            let size = if shape == "batch" { entries.len() } else { 1 };
            cases.push(Case {
                name: format!("{n}/{shape}/duplicate/exclusive"),
                accesses: vec![],
                groups: entries.chunks(size).map(|r| r.to_vec()).collect(),
                expected: false,
            });
        }
    }
    for n in [128, 512, 8192] {
        let first = (0..n)
            .map(|i| (0x100000 + i * 64, false))
            .collect::<Vec<_>>();
        let mut groups = vec![first.clone()];
        groups.extend(first.iter().rev().map(|&e| vec![e]));
        cases.push(Case {
            name: format!("{n}/history/repeated/shared"),
            accesses: vec![(0x800000, true)],
            groups,
            expected: true,
        });
        let first = (0..n)
            .map(|i| (0x100000 + i * 64, true))
            .collect::<Vec<_>>();
        let groups = vec![first, vec![(0x900000, false), (0x100000, false)]];
        cases.push(Case {
            name: format!("{n}/history/late/conflict"),
            accesses: vec![],
            groups,
            expected: false,
        });
        cases.push(Case {
            name: format!("{n}/mixed/shuffled/unique"),
            accesses: (0..16).map(|i| (0x800000 + i * 64, i % 4 == 0)).collect(),
            groups: (0..n)
                .map(|i| vec![(0x100000 + ((i * 17) % n) * 64, i % 4 == 0)])
                .collect(),
            expected: true,
        });
    }
    cases
}
macro_rules! dispatch {
    ($name:expr, $func:ident $(,$arg:expr)*) => { match $name {
        "selected" => $func::<Selected>($($arg),*),
        "limit0" => $func::<Selected<0>>($($arg),*),
        "limit64" => $func::<Selected<64>>($($arg),*),
        "limit128" => $func::<Selected<128>>($($arg),*),
        "limit256" => $func::<Selected<256>>($($arg),*),
        "limit512" => $func::<Selected<512>>($($arg),*),
        "limit1024" => $func::<Selected<1024>>($($arg),*),
        "limit2048" => $func::<Selected<2048>>($($arg),*),
        "limit4096" => $func::<Selected<4096>>($($arg),*),
        "limitmax" => $func::<Selected<{usize::MAX}>>($($arg),*),
        "sorted" => $func::<Current>($($arg),*),
        "linear" => $func::<Linear>($($arg),*),
        "hash" => $func::<Indexed<HashMap<usize,u64>>>($($arg),*),
        "hash_fast" => $func::<Indexed<FastMap>>($($arg),*),
        "btree" => $func::<Indexed<BTreeMap<usize,u64>>>($($arg),*),
        "bloom512" => $func::<Bloom<8,false>>($($arg),*),
        "bloom8192" => $func::<Bloom<128,false>>($($arg),*),
        "blocked8192" => $func::<Bloom<128,true>>($($arg),*),
        "bloom_grow" => $func::<GrowingBloom>($($arg),*),
        "hybrid64" => $func::<Hybrid<64>>($($arg),*),
        "hybrid128" => $func::<Hybrid<128>>($($arg),*),
        _ => panic!("unknown candidate")
    } };
}
#[cfg(counts)]
mod allocations {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    };
    pub static ENABLED: AtomicBool = AtomicBool::new(false);
    pub static ALLOCS: AtomicUsize = AtomicUsize::new(0);
    pub static REALLOCS: AtomicUsize = AtomicUsize::new(0);
    pub static BYTES: AtomicUsize = AtomicUsize::new(0);
    pub static LIVE: AtomicUsize = AtomicUsize::new(0);
    pub static PEAK: AtomicUsize = AtomicUsize::new(0);
    struct Counter;
    fn add(size: usize) {
        BYTES.fetch_add(size, Relaxed);
        let live = LIVE.fetch_add(size, Relaxed) + size;
        PEAK.fetch_max(live, Relaxed);
    }
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let p = System.alloc(l);
            if !p.is_null() && ENABLED.load(Relaxed) {
                ALLOCS.fetch_add(1, Relaxed);
                add(l.size());
            }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            if ENABLED.load(Relaxed) {
                LIVE.fetch_sub(l.size(), Relaxed);
            }
            System.dealloc(p, l);
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            let q = System.realloc(p, l, n);
            if !q.is_null() && ENABLED.load(Relaxed) {
                REALLOCS.fetch_add(1, Relaxed);
                LIVE.fetch_sub(l.size(), Relaxed);
                add(n);
            }
            q
        }
    }
    #[global_allocator]
    static ALLOCATOR: Counter = Counter;
}
fn sample<V: Validator>(case: &Case, target: u64) -> (u64, u64) {
    let probe = Instant::now();
    for _ in 0..4 {
        assert_eq!(black_box(run::<V>(black_box(case))), case.expected);
    }
    let cost = (probe.elapsed().as_nanos() as u64 / 4).max(1);
    let it = (target * 1_000_000 / cost).clamp(4, 2_000_000);
    let begin = Instant::now();
    for _ in 0..it {
        assert_eq!(black_box(run::<V>(black_box(case))), case.expected);
    }
    (it, begin.elapsed().as_nanos() as u64)
}
fn count<V: Validator>(case: &Case) {
    #[cfg(counts)]
    {
        use allocations::*;
        use std::sync::atomic::Ordering::Relaxed;
        for value in [&ALLOCS, &REALLOCS, &BYTES, &LIVE, &PEAK] {
            value.store(0, Relaxed);
        }
        #[cfg(metrics)]
        for value in [
            &metrics::QUERIES,
            &metrics::POSITIVES,
            &metrics::FALSE_POSITIVES,
            &metrics::EXACT_VISITS,
        ] {
            value.store(0, Relaxed);
        }
        ENABLED.store(true, Relaxed);
        let accepted = run::<V>(case);
        ENABLED.store(false, Relaxed);
        assert_eq!(accepted, case.expected);
        assert_eq!(LIVE.load(Relaxed), 0);
        #[cfg(metrics)]
        let stats = [
            metrics::QUERIES.load(Relaxed),
            metrics::POSITIVES.load(Relaxed),
            metrics::FALSE_POSITIVES.load(Relaxed),
            metrics::EXACT_VISITS.load(Relaxed),
        ];
        #[cfg(not(metrics))]
        let stats = [0; 4];
        println!(
            "COUNT,{},{},{},{},{},{},{},{},{}",
            ALLOCS.load(Relaxed),
            REALLOCS.load(Relaxed),
            BYTES.load(Relaxed),
            PEAK.load(Relaxed),
            std::mem::size_of::<V>(),
            stats[0],
            stats[1],
            stats[2],
            stats[3]
        );
    }
    #[cfg(not(counts))]
    {
        let _ = case;
        panic!("counts require separate binary");
    }
}
fn correctness<V: Validator>() {
    // Model checks include continuing after failed groups and direct accesses.
    for initial in 0..64u64 {
        let mut seed = initial;
        let mut v = V::default();
        let mut model = BTreeMap::<usize, bool>::new();
        let initial_group = (0..64).map(|i| (i * 64, i % 4 == 0)).collect::<Vec<_>>();
        let mut group = v.begin(initial_group.len());
        for &e in &initial_group {
            V::push(&mut group, e);
            model.insert(e.0, e.1);
        }
        assert!(v.accept(group));
        let mut bad = v.begin(2);
        V::push(&mut bad, (100000, false));
        V::push(&mut bad, (0, true));
        assert!(!v.accept(bad));
        assert!(v.access((100000, true)));
        model.insert(100000, true);
        for step in 0..96 {
            let mut values = Vec::new();
            for _ in 0..step % 7 {
                seed = mix(seed.wrapping_add(1));
                values.push((((seed >> 9) % 256) as usize * 64, seed % 9 == 0));
            }
            let previous = model.clone();
            let mut good = true;
            for &(a, e) in &values {
                if model.get(&a).is_some_and(|p| *p || e) {
                    good = false;
                    break;
                }
                model.insert(a, e);
            }
            if !good {
                model = previous;
            }
            let mut group = v.begin(values.len());
            for e in values {
                V::push(&mut group, e);
            }
            assert_eq!(v.accept(group), good);
            for probe in 0..4 {
                seed = mix(seed.wrapping_add(1));
                let a = ((seed >> 7) % 256) as usize * 64;
                let e = probe % 2 == 0;
                let expected = !model.get(&a).is_some_and(|p| *p || e);
                assert_eq!(v.access((a, e)), expected);
                if expected {
                    model.insert(a, e);
                }
            }
        }
    }
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let candidates = [
        "selected", "limit0", "limit64", "limit128", "limit256", "limit512", "limit1024", "limit2048", "limit4096", "limitmax",
        "sorted",
        "linear",
        "hash",
        "hash_fast",
        "btree",
        "bloom512",
        "bloom8192",
        "blocked8192",
        "bloom_grow",
        "hybrid64",
        "hybrid128",
    ];
    if args[1] == "verify" {
        for name in candidates {
            dispatch!(name, correctness);
            println!("verified {name}");
        }
        return;
    }
    let cases = cases();
    if args[1] == "list" {
        for c in cases {
            println!("{}", c.name);
        }
        return;
    }
    if args[1] == "countall" {
        for case in cases
            .iter()
            .filter(|c| args[2].split(';').any(|n| n == c.name))
        {
            for name in candidates {
                for _ in 0..3 {
                    println!("CELL,{name},{}", case.name);
                    dispatch!(name, count, case);
                }
            }
        }
        return;
    }
    let case = cases.iter().find(|c| c.name == args[3]).expect("case");
    if args[1] == "count" {
        dispatch!(args[2].as_str(), count, case);
        return;
    }
    let target = args[4].parse::<u64>().unwrap();
    let batches = args[5].parse::<usize>().unwrap();
    // One process per case; rotate candidate order each batch to balance drift.
    let fork = args[6].parse::<usize>().unwrap();
    let candidates = candidates
        .into_iter()
        .filter(|name| args[2] == "all" || *name == args[2])
        .collect::<Vec<_>>();
    for batch in 0..batches + 2 {
        for step in 0..candidates.len() {
            let i = if fork % 2 == 0 {
                (step + batch + fork) % candidates.len()
            } else {
                (candidates.len() - 1 - step + batch + fork) % candidates.len()
            };
            let name = candidates[i];
            let (it, ns) = dispatch!(name, sample, case, target);
            if batch >= 2 {
                println!("TIME,{name},{},{},{it},{ns}", case.name, batch - 2);
            }
        }
    }
}
