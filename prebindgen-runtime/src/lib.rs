//! Owned, typed handles and cleanup shared by generated C and JNI bindings.
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

pub type Result<T> = std::result::Result<T, String>;
type Value = Box<dyn Any + Send>;
#[derive(Default)]
struct Store {
    next: u64,
    values: HashMap<u64, Value>,
}
fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(Mutex::default)
}

/// A wrong-type handle is rejected without consuming it. Removed values are
/// dropped outside the lock, including on error paths.
pub fn take<T: Any + Send>(id: u64) -> Result<T> {
    let value = {
        let mut store = store().lock().map_err(|_| "handle store poisoned")?;
        let value = store
            .values
            .get(&id)
            .ok_or_else(|| format!("invalid or consumed handle {id}"))?;
        if (**value).type_id() != TypeId::of::<T>() {
            return Err("wrong handle type".into());
        }
        store.values.remove(&id).unwrap()
    };
    Ok(*value.downcast::<T>().map_err(|_| "wrong handle type")?)
}
pub fn is_alive(id: u64) -> bool {
    store()
        .lock()
        .map(|s| s.values.contains_key(&id))
        .unwrap_or(false)
}
pub fn release<T: Any + Send>(id: u64) -> Result<()> {
    let value = {
        let mut store = store().lock().map_err(|_| "handle store poisoned")?;
        let Some(value) = store.values.get(&id) else {
            return Ok(());
        };
        if (**value).type_id() != TypeId::of::<T>() {
            return Err("wrong handle type".into());
        }
        store.values.remove(&id)
    };
    drop(value);
    Ok(())
}

/// Output handles become caller-owned only after the whole result is encoded.
/// Until then, a failed conversion or failed foreign delivery releases them.
#[derive(Default)]
pub struct Outputs {
    pending: Vec<u64>,
}
impl Outputs {
    pub fn insert<T: Any + Send>(&mut self, value: T) -> Result<u64> {
        let mut store = store().lock().map_err(|_| "handle store poisoned")?;
        let id = store
            .next
            .checked_add(1)
            .filter(|id| *id <= i64::MAX as u64)
            .ok_or("handle space exhausted")?;
        store.next = id;
        store.values.insert(id, Box::new(value));
        self.pending.push(id);
        Ok(id)
    }
    pub fn commit(mut self) {
        self.pending.clear();
    }
}
impl Drop for Outputs {
    fn drop(&mut self) {
        let removed = if let Ok(mut store) = store().lock() {
            self.pending
                .drain(..)
                .filter_map(|id| store.values.remove(&id))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        drop(removed);
    }
}

/// An exported boundary never unwinds through a foreign caller.
pub fn boundary<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("Rust operation panicked".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handles_are_typed_and_consumed_once() {
        let mut out = Outputs::default();
        let id = out.insert(String::from("abc")).unwrap();
        out.commit();
        assert!(take::<Vec<u8>>(id).is_err());
        assert_eq!(take::<String>(id).unwrap(), "abc");
        assert!(take::<String>(id).is_err());
    }
    #[test]
    fn failed_output_reclaims_handles() {
        let id;
        {
            let mut out = Outputs::default();
            id = out.insert(42u32).unwrap();
        }
        assert!(take::<u32>(id).is_err());
    }
    #[test]
    fn panics_do_not_escape() {
        assert!(boundary::<()>(|| panic!("test")).is_err());
    }
}
