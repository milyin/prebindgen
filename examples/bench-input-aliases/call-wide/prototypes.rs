//! Benchmark-only alternatives. Production converters and runtime are unchanged.
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};
type Result<T> = std::result::Result<T, String>;
#[derive(Default)]
struct AddressHasher(u64);
impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }
    fn write_usize(&mut self, v: usize) {
        let mut x = v as u64;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        self.0 = x ^ (x >> 31);
    }
}
#[derive(Default)]
pub struct OnlineHash {
    values: HashMap<usize, bool, BuildHasherDefault<AddressHasher>>,
    reserve: usize,
}
impl OnlineHash {
    pub fn needs_group(&self) -> bool {
        true
    }
    pub fn reserve_group(&mut self, n: usize) {
        self.reserve = n;
    }
    pub fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        if self.reserve != 0 {
            self.values.reserve(self.reserve);
            self.reserve = 0;
        }
        use std::collections::hash_map::Entry;
        match self.values.entry(address) {
            Entry::Vacant(e) => {
                e.insert(exclusive);
                Ok(())
            }
            Entry::Occupied(e) if !(*e.get() || exclusive) => Ok(()),
            Entry::Occupied(_) => Err("conflicting input aliases".into()),
        }
    }
}
pub fn check_sorted(entries: &mut [(usize, bool)]) -> Result<()> {
    entries.sort_unstable_by_key(|e| e.0);
    if entries
        .windows(2)
        .any(|p| p[0].0 == p[1].0 && (p[0].1 || p[1].1))
    {
        return Err("conflicting input aliases".into());
    }
    Ok(())
}
