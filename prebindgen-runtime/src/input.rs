//! Call-scoped resource conflicts for inputs constructed as they are decoded.
use crate::Result;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::{BuildHasherDefault, Hasher},
};

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
/// Address history for a call with dynamic collections. One complete batch and
/// small incremental histories retain sorted storage. Larger incremental sets
/// use an exact integer-hashed index rather than repeatedly moving old entries.
#[derive(Default)]
pub struct DynamicInputs {
    history: InputHistory,
}

// A single batch can reuse its input allocation without building another index.
// The threshold applies only when accepting additional distinct accesses/groups.
const INDEX_THRESHOLD: usize = 128;
enum InputHistory {
    Sorted(Vec<(usize, bool)>),
    Indexed(InputIndex),
}
impl Default for InputHistory {
    fn default() -> Self {
        Self::Sorted(Vec::new())
    }
}
#[derive(Default)]
struct InputIndex {
    // Low bit: exclusive access. Other bits identify the group that inserted
    // the address, so a rejected group can roll back without a second vector.
    values: HashMap<usize, u64, BuildHasherDefault<AddressHasher>>,
    generation: u64,
}
impl InputIndex {
    fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        match self.values.entry(address) {
            Entry::Vacant(entry) => {
                entry.insert(self.generation | u64::from(exclusive));
                Ok(())
            }
            Entry::Occupied(entry) if (entry.get() | u64::from(exclusive)) & 1 == 0 => Ok(()),
            Entry::Occupied(_) => Err("conflicting input aliases".into()),
        }
    }
    fn accept_group(&mut self, group: InputGroup) -> Result<()> {
        self.generation = match self.generation.checked_add(2) {
            Some(next) => next,
            None => {
                // Reset only group tags; retain every address and its access mode.
                // This is unreachable for practical call sizes, but keeps the API
                // correct even if a validator is reused for more than 2^63 groups.
                for value in self.values.values_mut() {
                    *value &= 1;
                }
                2
            }
        };
        self.values.reserve(group.values.len());
        for (address, exclusive) in group.values {
            if let Err(error) = self.access(address, exclusive) {
                let generation = self.generation;
                self.values.retain(|_, value| *value & !1 != generation);
                return Err(error);
            }
        }
        Ok(())
    }
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
    fn promote(&mut self) {
        let previous = std::mem::take(&mut self.history);
        if let InputHistory::Sorted(values) = previous {
            let mut index = InputIndex {
                values: HashMap::with_capacity_and_hasher(values.len(), Default::default()),
                generation: 0,
            };
            for (address, exclusive) in values {
                index.values.insert(address, u64::from(exclusive));
            }
            self.history = InputHistory::Indexed(index);
        } else {
            self.history = previous;
        }
    }
    /// Reject conflicting accesses before constructing any owner or reference.
    /// A rejected group leaves previously accepted identities unchanged.
    pub fn accept_group(&mut self, group: InputGroup) -> Result<()> {
        if group.values.is_empty() {
            return Ok(());
        }
        if let InputHistory::Sorted(values) = &self.history {
            if !values.is_empty() && group.values.len() == 1 {
                let (a, e) = group.values[0];
                return self.access(a, e);
            }
            if !values.is_empty()
                && values.len().saturating_add(group.values.len()) > INDEX_THRESHOLD
            {
                self.promote();
            }
        }
        match &mut self.history {
            InputHistory::Sorted(values) => Self::accept_sorted(values, group),
            InputHistory::Indexed(index) => index.accept_group(group),
        }
    }
    fn accept_sorted(values: &mut Vec<(usize, bool)>, group: InputGroup) -> Result<()> {
        let mut group = group.values;
        if group.is_empty() {
            // Previously accepted accesses are already sorted and checked.
            return Ok(());
        }
        if group.len() == 1 && !values.is_empty() {
            // No internal duplicate is possible; only history can conflict.
            let (address, exclusive) = group[0];
            return Self::access_sorted(values, address, exclusive);
        }
        // Sort only the new addresses. Re-sorting/scanning the complete history
        // for every inner list makes nested input cost depend heavily on the
        // allocator's address order, even though earlier groups are unchanged.
        group.sort_unstable_by_key(|entry| entry.0);
        for pair in group.windows(2) {
            if pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1) {
                return Err("conflicting input aliases".into());
            }
        }
        group.dedup_by_key(|entry| entry.0);
        if values.is_empty() {
            // Reuse the first batch's allocation as the history.
            *values = group;
            return Ok(());
        }
        if group[0].0 > values.last().expect("nonempty history").0 {
            // Disjoint ascending batches can append without any history scan.
            values.append(&mut group);
            return Ok(());
        }
        let mut added = 0;
        for i in 0..group.len() {
            let (address, exclusive) = group[i];
            match values.binary_search_by_key(&address, |entry| entry.0) {
                Ok(previous) if exclusive || values[previous].1 => {
                    return Err("conflicting input aliases".into());
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
        Ok(())
    }
    /// Check an individual argument against previous accesses in this call.
    pub fn access(&mut self, address: usize, exclusive: bool) -> Result<()> {
        if let InputHistory::Sorted(values) = &mut self.history {
            let position = match values.binary_search_by_key(&address, |entry| entry.0) {
                Ok(i) => {
                    return if values[i].1 || exclusive {
                        Err("conflicting input aliases".into())
                    } else {
                        Ok(())
                    }
                }
                Err(position) => position,
            };
            if values.len() < INDEX_THRESHOLD {
                values.insert(position, (address, exclusive));
                return Ok(());
            }
            self.promote();
        }
        match &mut self.history {
            InputHistory::Indexed(index) => index.access(address, exclusive),
            InputHistory::Sorted(_) => unreachable!("promoted above"),
        }
    }
    fn access_sorted(
        values: &mut Vec<(usize, bool)>,
        address: usize,
        exclusive: bool,
    ) -> Result<()> {
        match values.binary_search_by_key(&address, |entry| entry.0) {
            Ok(i) if values[i].1 || exclusive => Err("conflicting input aliases".into()),
            Ok(_) => Ok(()),
            Err(i) => {
                values.insert(i, (address, exclusive));
                Ok(())
            }
        }
    }
    #[cfg(test)]
    fn snapshot(&self) -> Vec<(usize, bool)> {
        match &self.history {
            InputHistory::Sorted(values) => values.clone(),
            InputHistory::Indexed(index) => {
                let mut values = index
                    .values
                    .iter()
                    .map(|(&a, &tag)| (a, tag & 1 != 0))
                    .collect::<Vec<_>>();
                values.sort_unstable_by_key(|entry| entry.0);
                values
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
        assert!(matches!(&history.history, InputHistory::Sorted(values) if values.capacity()==0));
        history.access(8, true).unwrap();
        assert!(!history.snapshot().is_empty());
    }

    #[test]
    fn empty_and_singleton_groups_preserve_cross_group_conflicts() {
        for previous_exclusive in [false, true] {
            let mut history = DynamicInputs::default();
            let mut first = history.begin_group(3);
            first.push((24, previous_exclusive));
            first.push((8, false));
            first.push((16, false));
            history.accept_group(first).unwrap();
            history.accept_group(history.begin_group(0)).unwrap();
            for address in [32, 4, 20] {
                let mut next = history.begin_group(1);
                next.push((address, true));
                history.accept_group(next).unwrap();
            }
            let mut shared = history.begin_group(1);
            shared.push((24, false));
            assert_eq!(history.accept_group(shared).is_err(), previous_exclusive);
            for (address, exclusive) in [(24, true), (32, false), (4, true), (20, false)] {
                let mut next = history.begin_group(1);
                next.push((address, exclusive));
                assert!(history.accept_group(next).is_err());
            }
            assert_eq!(
                history.snapshot(),
                vec![
                    (4, true),
                    (8, false),
                    (16, false),
                    (20, true),
                    (24, previous_exclusive),
                    (32, true)
                ]
            );
        }
    }

    #[test]
    fn merge_compacts_shared_repeats_and_inserts_new_endpoints() {
        let mut history = DynamicInputs::default();
        history.access(16, false).unwrap();
        history.access(32, false).unwrap();
        let mut group = history.begin_group(5);
        for value in [
            (32, false),
            (4, true),
            (16, false),
            (40, false),
            (24, false),
        ] {
            group.push(value);
        }
        history.accept_group(group).unwrap();
        assert_eq!(
            history.snapshot(),
            vec![
                (4, true),
                (16, false),
                (24, false),
                (32, false),
                (40, false)
            ]
        );
        let previous = history.snapshot();
        let mut bad = history.begin_group(2);
        bad.push((8, false));
        bad.push((32, true));
        assert!(history.accept_group(bad).is_err());
        assert_eq!(history.snapshot(), previous);
    }

    #[test]
    fn descending_batches_merge_into_long_history() {
        let mut history = DynamicInputs::default();
        let mut expected = Vec::new();
        for batch in (0..128).rev() {
            let mut group = history.begin_group(4);
            for field in 0..=batch % 4 {
                let value = ((batch * 4 + field + 1) * 8, batch % 3 == 0);
                group.push(value);
                expected.push(value);
            }
            history.accept_group(group).unwrap();
            expected.sort_unstable_by_key(|entry| entry.0);
            assert_eq!(history.snapshot(), expected);
        }
    }

    #[test]
    fn mixed_groups_match_combined_sort_validation() {
        for initial_seed in 0u64..128 {
            let mut seed = initial_seed;
            let mut history = DynamicInputs::default();
            for step in 0..64 {
                let len = step % 5;
                let mut group = history.begin_group(len);
                let previous = history.snapshot();
                let mut expected = previous.clone();
                for _ in 0..len {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let value = (((seed >> 32) as usize % 512) * 8, seed % 7 == 0);
                    group.push(value);
                    expected.push(value);
                }
                expected.sort_unstable_by_key(|entry| entry.0);
                let conflict = expected
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1));
                assert_eq!(history.accept_group(group).is_err(), conflict);
                if conflict {
                    assert_eq!(history.snapshot(), previous);
                    // Generated conversion returns immediately on conflict.
                    history = DynamicInputs::default();
                } else {
                    expected.dedup_by_key(|entry| entry.0);
                    assert_eq!(history.snapshot(), expected);
                }
            }
        }
    }

    #[test]
    fn one_large_batch_reuses_sorted_storage_until_another_identity_arrives() {
        let mut history = DynamicInputs::default();
        let mut group = history.begin_group(2048);
        for i in (0..2048).rev() {
            group.push((i * 64, i % 4 == 0));
        }
        history.accept_group(group).unwrap();
        assert!(matches!(history.history, InputHistory::Sorted(_)));
        history.accept_group(history.begin_group(0)).unwrap();
        history.access(64, false).unwrap();
        assert!(history.access(0, false).is_err());
        assert!(matches!(history.history, InputHistory::Sorted(_)));
        history.access(2048 * 64, true).unwrap();
        assert!(matches!(history.history, InputHistory::Indexed(_)));
        assert_eq!(history.snapshot().len(), 2049);
    }

    #[test]
    fn indexed_groups_roll_back_new_shared_and_exclusive_entries_only() {
        let mut history = DynamicInputs::default();
        for i in 0..512 {
            history.access(i * 64, i % 4 == 0).unwrap();
        }
        assert!(matches!(history.history, InputHistory::Indexed(_)));
        let before = history.snapshot();
        for conflict in [(0, false), (64, true), (0, true)] {
            let mut group = history.begin_group(5);
            for entry in [
                (100000, true),
                (100064, false),
                (64, false),
                (100064, false),
                conflict,
            ] {
                group.push(entry);
            }
            assert!(history.accept_group(group).is_err());
            assert_eq!(history.snapshot(), before);
        }
        let mut group = history.begin_group(3);
        for entry in [(100000, false), (100000, false), (100000, true)] {
            group.push(entry);
        }
        assert!(history.accept_group(group).is_err());
        assert_eq!(history.snapshot(), before);
        history.access(100000, true).unwrap();
        assert!(history.access(100000, false).is_err());
    }

    #[test]
    fn indexed_generation_wrap_preserves_older_modes_and_rollback() {
        let mut history = DynamicInputs::default();
        for i in 0..512 {
            history.access(i * 64, i % 4 == 0).unwrap();
        }
        let InputHistory::Indexed(index) = &mut history.history else {
            panic!("index expected")
        };
        index.generation = u64::MAX - 1;
        let before = history.snapshot();
        let mut group = history.begin_group(2);
        group.push((100000, true));
        group.push((0, false));
        assert!(history.accept_group(group).is_err());
        assert_eq!(history.snapshot(), before);
        history.access(64, false).unwrap();
        assert!(history.access(64, true).is_err());
        history.access(100000, true).unwrap();
    }

    #[test]
    fn mixed_incremental_calls_match_exact_model_across_promotion_and_failures() {
        use std::collections::BTreeMap;
        for initial in 0..16u64 {
            let mut history = DynamicInputs::default();
            let mut model = BTreeMap::new();
            let mut seed = initial;
            for step in 0..512 {
                let mut group = history.begin_group(step % 11);
                let mut expected = model.clone();
                let mut conflict = false;
                for _ in 0..step % 11 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let address = ((seed >> 32) as usize % 4096) * 64;
                    let exclusive = seed % 13 == 0;
                    group.push((address, exclusive));
                    if !conflict {
                        if expected
                            .get(&address)
                            .is_some_and(|e: &bool| *e || exclusive)
                        {
                            conflict = true;
                        } else {
                            expected.insert(address, exclusive);
                        }
                    }
                }
                assert_eq!(history.accept_group(group).is_err(), conflict);
                if !conflict {
                    model = expected;
                }
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let address = ((seed >> 32) as usize % 4096) * 64;
                let exclusive = step % 3 == 0;
                let conflict = model.get(&address).is_some_and(|e| *e || exclusive);
                assert_eq!(history.access(address, exclusive).is_err(), conflict);
                if !conflict {
                    model.insert(address, exclusive);
                }
                assert_eq!(
                    history.snapshot(),
                    model.iter().map(|(&a, &e)| (a, e)).collect::<Vec<_>>()
                );
            }
            assert!(matches!(history.history, InputHistory::Indexed(_)));
        }
    }
}
