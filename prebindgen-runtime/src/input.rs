//! Call-scoped resource conflicts for inputs constructed as they are decoded.
use crate::Result;
use std::hash::Hasher;

/// No bookkeeping is needed for shared-only calls or a single fixed handle.
#[derive(Default)]
pub struct SharedInputs;
impl SharedInputs {
    /// Shared-only calls can construct rows directly without retaining a batch.
    #[inline(always)]
    pub fn needs_group(&self) -> bool {
        false
    }
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
/// A shared-only group needs neither storage nor comparisons.
pub struct SharedGroup;
impl SharedGroup {
    /// Ignore an access when the entire call uses shared borrows.
    #[inline(always)]
    pub fn push(&mut self, _: (usize, bool)) {}
}
impl SharedInputs {
    /// Begin a collection whose accesses cannot conflict.
    pub fn begin_group(&self, _: usize) -> SharedGroup {
        SharedGroup
    }
    /// Complete a shared-only collection check.
    pub fn accept_group(&mut self, _: SharedGroup) -> Result<()> {
        Ok(())
    }
}
/// Lazily allocated addresses from a collection that has not been acquired yet.
pub struct InputGroup {
    values: Vec<(usize, bool)>,
    capacity: usize,
}
impl InputGroup {
    /// Register an active resource without allocating for absent optional values.
    pub fn push(&mut self, value: (usize, bool)) {
        if self.values.capacity() == 0 {
            self.values.reserve_exact(self.capacity);
        }
        self.values.push(value);
    }
}
/// Sorted address history for dynamic collections. A collection checks all of
/// its addresses together, before creating any owners or Rust references.
#[derive(Default)]
pub struct DynamicInputs {
    values: Vec<(usize, bool)>,
}
impl DynamicInputs {
    /// Dynamic exclusive accesses need a checked batch before construction.
    #[inline(always)]
    pub fn needs_group(&self) -> bool {
        true
    }
    /// No allocation is needed until an active access or group is encountered.
    pub fn reserve(&mut self, _: usize) {}
    /// Collect a batch without allocating for absent optional resources.
    pub fn begin_group(&self, capacity: usize) -> InputGroup {
        InputGroup {
            values: Vec::new(),
            capacity,
        }
    }
    /// Sort the new batch together with previous accesses, then reject every
    /// shared/exclusive or exclusive/exclusive duplicate before using the batch.
    pub fn accept_group(&mut self, group: InputGroup) -> Result<()> {
        let mut group = group.values;
        if self.values.is_empty() {
            ::core::mem::swap(&mut self.values, &mut group);
        } else {
            self.values.append(&mut group);
        }
        self.values.sort_unstable_by_key(|entry| entry.0);
        for pair in self.values.windows(2) {
            if pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1) {
                return Err("conflicting input aliases".into());
            }
        }
        self.values.dedup_by_key(|entry| entry.0);
        Ok(())
    }
    /// Check an individual argument against the history of previous arguments.
    pub fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        match self.values.binary_search_by_key(&address, |entry| entry.0) {
            Ok(i) if self.values[i].1 || exclusive => Err("conflicting input aliases".into()),
            Ok(_) => Ok(()),
            Err(i) => {
                self.values.insert(i, (address, exclusive));
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
    fn groups_check_internal_conflicts_and_previous_arguments() {
        let mut history = DynamicInputs::default();
        history.access(8, false).unwrap();
        let mut group = history.begin_group(4);
        group.push((16, true));
        group.push((24, false));
        group.push((24, false));
        history.accept_group(group).unwrap();
        assert!(history.access(16, false).is_err());
        let mut next = history.begin_group(2);
        next.push((32, false));
        next.push((8, true));
        assert!(history.accept_group(next).is_err());
        let mut history = DynamicInputs::default();
        let mut duplicates = history.begin_group(2);
        duplicates.push((48, true));
        duplicates.push((48, true));
        assert!(history.accept_group(duplicates).is_err());
    }
    #[test]
    fn inactive_dynamic_input_does_not_allocate() {
        let mut history = DynamicInputs::default();
        history.reserve(1000);
        assert_eq!(history.values.capacity(), 0);
        history.access(8, true).unwrap();
        assert!(history.values.capacity() > 0);
    }
}
