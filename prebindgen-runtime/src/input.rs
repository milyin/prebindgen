//! Call-scoped resource conflicts for inputs constructed as they are decoded.
use crate::Result;
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

/// No bookkeeping is needed for shared-only calls or a single fixed handle.
#[derive(Default)]
pub struct SharedInputs;
impl SharedInputs {
    /// Accept an access that cannot conflict within this generated call.
    #[inline(always)]
    pub fn access(&mut self, _: usize, _: bool) -> Result<()> {
        Ok(())
    }
    /// Ignore a collection capacity hint.
    #[inline(always)]
    pub fn reserve(&mut self, _: usize) {}
}
/// Heap-free history for fixed accesses; larger sets use a stack hash table.
pub struct FixedInputs<const N: usize> {
    values: [(usize, bool); N],
    len: usize,
}
impl<const N: usize> Default for FixedInputs<N> {
    fn default() -> Self {
        Self {
            values: [(0, false); N],
            len: 0,
        }
    }
}
impl<const N: usize> FixedInputs<N> {
    /// Check this access before creating an owner or reference.
    #[inline]
    pub fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        if N <= 8 {
            if self.values[..self.len]
                .iter()
                .any(|&(p, e)| p == address && (e || exclusive))
            {
                return Err("conflicting input aliases".into());
            }
            if self.len == N {
                return Err("input access history capacity exceeded".into());
            }
            self.values[self.len] = (address, exclusive);
            self.len += 1;
        } else {
            let mut hasher = AddressHasher::default();
            hasher.write_usize(address);
            let mut slot = hasher.finish() as usize % N;
            for _ in 0..N {
                let (previous, used_exclusively) = self.values[slot];
                if previous == 0 {
                    self.values[slot] = (address, exclusive);
                    return Ok(());
                }
                if previous == address {
                    if used_exclusively || exclusive {
                        return Err("conflicting input aliases".into());
                    }
                    return Ok(());
                }
                slot = (slot + 1) % N;
            }
            return Err("input access history capacity exceeded".into());
        }
        Ok(())
    }
    /// Fixed input histories need no collection allocation.
    pub fn reserve(&mut self, _: usize) {}
}
/// Hash native addresses without hashing arbitrary foreign data.
#[derive(Default)]
struct AddressHasher(u64);
impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        let mut word = 0u64;
        for (i, b) in bytes.iter().take(8).enumerate() {
            word |= u64::from(*b) << (i * 8);
        }
        self.write_u64(word);
    }
    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }
    fn write_u64(&mut self, mut value: u64) {
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        self.0 = value ^ (value >> 31);
    }
}
/// History for dynamic collections, retaining addresses after their objects move.
#[derive(Default)]
pub struct DynamicInputs {
    values: HashMap<usize, bool, BuildHasherDefault<AddressHasher>>,
    hint: usize,
}
impl DynamicInputs {
    /// Delay allocating the capacity hint until the first active resource.
    pub fn reserve(&mut self, additional: usize) {
        self.hint = self.hint.max(self.values.len().saturating_add(additional));
    }
    /// Reject a conflicting current or previous access before acquisition.
    pub fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        if self.values.capacity() < self.hint {
            self.values
                .reserve(self.hint.saturating_sub(self.values.len()));
        }
        match self.values.entry(address) {
            std::collections::hash_map::Entry::Occupied(e) if *e.get() || exclusive => {
                Err("conflicting input aliases".into())
            }
            std::collections::hash_map::Entry::Occupied(_) => Ok(()),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(exclusive);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_and_dynamic_histories_check_before_every_use() {
        for exclusive in [false, true] {
            let mut fixed = FixedInputs::<4>::default();
            let mut dynamic = DynamicInputs::default();
            dynamic.reserve(4);
            for (address, access) in [(8, false), (16, exclusive), (8, false)] {
                fixed.access(address, access).unwrap();
                dynamic.access(address, access).unwrap();
            }
            assert!(fixed.access(8, true).is_err());
            assert!(dynamic.access(8, true).is_err());
            assert!(fixed.access(16, true).is_err());
            assert!(dynamic.access(16, true).is_err());
        }
    }
    #[test]
    fn large_fixed_history_accepts_shared_aliases_and_detects_moves() {
        let mut history = FixedInputs::<32>::default();
        for i in 1..=24 {
            history.access(i * 8, false).unwrap();
        }
        for _ in 0..100 {
            history.access(8, false).unwrap();
        }
        assert!(history.access(8, true).is_err());
        history.access(800, true).unwrap();
        assert!(history.access(800, false).is_err());
    }
    #[test]
    fn inactive_dynamic_input_does_not_allocate() {
        let mut history = DynamicInputs::default();
        history.reserve(1000);
        assert_eq!(history.values.capacity(), 0);
        history.access(8, true).unwrap();
        assert!(history.values.capacity() >= 1000);
    }
}
