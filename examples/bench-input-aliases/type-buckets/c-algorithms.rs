//! Additional exact-address checkers for the C benchmark, not production APIs.
pub type Address = (usize, bool);
fn scan(values: &[Address]) -> Result<(), String> {
    if values
        .windows(2)
        .any(|w| w[0].0 == w[1].0 && (w[0].1 || w[1].1))
    {
        Err("conflicting input aliases".into())
    } else {
        Ok(())
    }
}
pub fn merge_original(groups: &[Vec<Address>; 5]) -> Result<(), String> {
    let mut tree = [None::<(usize, usize)>; 16];
    let mut pos = [0usize; 5];
    for (i, g) in groups.iter().enumerate() {
        tree[8 + i] = g.first().map(|e| (e.0, i));
    }
    fn least(a: Option<(usize, usize)>, b: Option<(usize, usize)>) -> Option<(usize, usize)> {
        match (a, b) {
            (None, b) => b,
            (a, None) => a,
            (Some(a), Some(b)) => Some(a.min(b)),
        }
    }
    for i in (1..8).rev() {
        tree[i] = least(tree[i * 2], tree[i * 2 + 1]);
    }
    let mut previous = None::<Address>;
    while let Some((address, g)) = tree[1] {
        let value = groups[g][pos[g]];
        if let Some(p) = previous {
            if p.0 == address && (p.1 || value.1) {
                return Err("conflicting input aliases".into());
            }
        }
        previous = Some(value);
        pos[g] += 1;
        let mut node = 8 + g;
        tree[node] = groups[g].get(pos[g]).map(|e| (e.0, g));
        while node > 1 {
            node /= 2;
            tree[node] = least(tree[node * 2], tree[node * 2 + 1]);
        }
    }
    Ok(())
}
/// One global scan checks both local and cross-group aliases. Empty leaves are
/// (MAX, MAX), so a real MAX address still precedes the sentinel by group index.
pub fn merge_compact(groups: &[Vec<Address>; 5]) -> Result<(), String> {
    let active: usize = groups.iter().filter(|g| !g.is_empty()).count();
    if active <= 1 {
        for group in groups {
            scan(group)?;
        }
        return Ok(());
    }
    let leaves = active.next_power_of_two();
    let empty = (usize::MAX, usize::MAX);
    let mut tree = [empty; 16];
    let mut pos = [0usize; 5];
    let mut leaf = [0usize; 5];
    let mut next = leaves;
    let mut total = 0usize;
    for (g, values) in groups.iter().enumerate() {
        total += values.len();
        if let Some(v) = values.first() {
            tree[next] = (v.0, g);
            leaf[g] = next;
            next += 1;
        }
    }
    for i in (1..leaves).rev() {
        tree[i] = tree[i * 2].min(tree[i * 2 + 1]);
    }
    let mut previous = None::<Address>;
    for _ in 0..total {
        let (address, g) = tree[1];
        let value = groups[g][pos[g]];
        if let Some(p) = previous {
            if p.0 == address && (p.1 || value.1) {
                return Err("conflicting input aliases".into());
            }
        }
        previous = Some(value);
        pos[g] += 1;
        let mut node = leaf[g];
        tree[node] = groups[g].get(pos[g]).map_or(empty, |v| (v.0, g));
        while node > 1 {
            node /= 2;
            tree[node] = tree[node * 2].min(tree[node * 2 + 1]);
        }
    }
    Ok(())
}
pub fn compact(groups: &mut [Vec<Address>; 5]) -> Result<(), String> {
    for group in groups.iter_mut() {
        group.sort_unstable_by_key(|v| v.0);
    }
    merge_compact(groups)
}
/// Check intersections directly: no merged output is needed. Shared-only pairs
/// cannot conflict. Disjoint address ranges skip the two-cursor intersection.
pub fn intersections(groups: &[Vec<Address>; 5]) -> Result<(), String> {
    let exclusive = groups.each_ref().map(|g| g.iter().any(|v| v.1));
    for a in 0..5 {
        for b in a + 1..5 {
            let left = &groups[a];
            let right = &groups[b];
            if left.is_empty() || right.is_empty() || (!exclusive[a] && !exclusive[b]) {
                continue;
            }
            if left.last().unwrap().0 < right[0].0 || right.last().unwrap().0 < left[0].0 {
                continue;
            }
            if left.len() == 1 || right.len() == 1 {
                let (one, many) = if left.len() == 1 {
                    (left[0], right)
                } else {
                    (right[0], left)
                };
                if let Ok(i) = many.binary_search_by_key(&one.0, |v| v.0) {
                    if one.1 || many[i].1 {
                        return Err("conflicting input aliases".into());
                    }
                }
                continue;
            }
            let (mut i, mut j) = (0, 0);
            while i < left.len() && j < right.len() {
                match left[i].0.cmp(&right[j].0) {
                    std::cmp::Ordering::Less => i += 1,
                    std::cmp::Ordering::Greater => j += 1,
                    std::cmp::Ordering::Equal => {
                        if left[i].1 || right[j].1 {
                            return Err("conflicting input aliases".into());
                        }
                        i += 1;
                        j += 1;
                    }
                }
            }
        }
    }
    Ok(())
}
pub fn pairs(groups: &mut [Vec<Address>; 5]) -> Result<(), String> {
    for group in groups.iter_mut() {
        group.sort_unstable_by_key(|v| v.0);
        scan(group)?;
    }
    intersections(groups)
}
/// Alternative global sort that can exploit runs; its scratch allocation is
/// measured separately, rather than assumed to be free.
pub fn stable(values: &mut [Address]) -> Result<(), String> {
    values.sort_by_key(|v| v.0);
    scan(values)
}
