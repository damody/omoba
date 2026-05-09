//! Sorted-by-id state hashing for desync detection.
//!
//! Hashes a stable subset of entity state (typically positions, HP, key flags) per tick —
//! cheaper than full snapshot serialization, sensitive enough to catch any drift between
//! server and client simulations. The sort step ensures hash is invariant under storage
//! ordering (BTreeMap iteration order, ECS join order, etc.).
//!
//! Used by Phase 2+ desync detection: server broadcasts hashes on the configured
//! lockstep interval, clients compare; mismatch = kick + log replay for analysis.

use fxhash::FxHasher64;
use std::hash::{Hash, Hasher};

/// Hashes a slice of items, sorted by id, into a single u64.
/// `id_of` extracts the sort/identity key from each item; ties on id produce non-deterministic
/// (input-order-dependent) results — caller must guarantee unique ids.
pub fn hash_sorted_by_id<T: Hash, F: Fn(&T) -> u32>(items: &[T], id_of: F) -> u64 {
    let mut indices: Vec<usize> = (0..items.len()).collect();
    indices.sort_by_key(|&i| id_of(&items[i]));
    let mut h = FxHasher64::default();
    items.len().hash(&mut h);
    for i in indices {
        items[i].hash(&mut h);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Hash)]
    struct Dummy {
        id: u32,
        x: i32,
        y: i32,
    }

    #[test]
    fn order_invariant() {
        let a = vec![
            Dummy {
                id: 2,
                x: 10,
                y: 20,
            },
            Dummy { id: 1, x: 5, y: 5 },
            Dummy {
                id: 3,
                x: 30,
                y: 30,
            },
        ];
        let b = vec![
            Dummy {
                id: 3,
                x: 30,
                y: 30,
            },
            Dummy {
                id: 2,
                x: 10,
                y: 20,
            },
            Dummy { id: 1, x: 5, y: 5 },
        ];
        assert_eq!(
            hash_sorted_by_id(&a, |d| d.id),
            hash_sorted_by_id(&b, |d| d.id)
        );
    }

    #[test]
    fn detects_change() {
        let a = vec![Dummy { id: 1, x: 5, y: 5 }];
        let b = vec![Dummy { id: 1, x: 6, y: 5 }];
        assert_ne!(
            hash_sorted_by_id(&a, |d| d.id),
            hash_sorted_by_id(&b, |d| d.id)
        );
    }

    #[test]
    fn empty_collection() {
        // Hash of empty Vec is well-defined (length 0 fed to hasher); not zero.
        let v: Vec<Dummy> = vec![];
        let h = hash_sorted_by_id(&v, |d| d.id);
        // Just verify it doesn't panic; specific value doesn't matter for this test.
        let _ = h;
    }

    #[test]
    fn detects_added_entity() {
        let a = vec![Dummy { id: 1, x: 5, y: 5 }];
        let b = vec![Dummy { id: 1, x: 5, y: 5 }, Dummy { id: 2, x: 0, y: 0 }];
        assert_ne!(
            hash_sorted_by_id(&a, |d| d.id),
            hash_sorted_by_id(&b, |d| d.id)
        );
    }
}
