//! Logical → physical touch-slot mapper.
//!
//! Profiles use logical slots `0..=20` (21 nodes in `pubg.json`); the runtime
//! supports [`MAX_TOUCHES`](crate::MAX_TOUCHES) (10) concurrent touches.
//! This pure mapper hands out physical `0-9` slots on demand and reclaims
//! them on release. Port of vendor `phantom/src/touch.rs:SlotAllocator`.
#![deny(missing_docs)]

use std::collections::HashMap;

use wd_core::{Result, WdError};

/// Maximum concurrent physical slots (mirrors [`crate::MAX_TOUCHES`]).
pub const MAX_PHYSICAL_SLOTS: usize = crate::MAX_TOUCHES;

/// Logical (`u8` from profile) → physical (`0-9`) slot allocator.
#[derive(Debug, Clone, Default)]
pub struct SlotAllocator {
    /// Logical → physical mapping.
    logical_to_physical: HashMap<u8, u8>,
    /// Physical → logical owner.
    physical_to_logical: [Option<u8>; MAX_PHYSICAL_SLOTS],
}

impl SlotAllocator {
    /// New empty allocator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Physical slot for a logical slot, if already mapped.
    #[must_use]
    pub fn physical_for(&self, logical_slot: u8) -> Option<u8> {
        self.logical_to_physical.get(&logical_slot).copied()
    }

    /// Ensure a physical slot for `logical_slot` (reuse or allocate).
    ///
    /// # Errors
    ///
    /// Returns [`WdError::Validation`](wd_core::WdError::Validation) when all
    /// 10 physical slots are in use.
    pub fn ensure_physical(&mut self, logical_slot: u8) -> Result<u8> {
        if let Some(slot) = self.physical_for(logical_slot) {
            return Ok(slot);
        }
        let Some((physical, owner)) = self
            .physical_to_logical
            .iter_mut()
            .enumerate()
            .find(|(_, owner)| owner.is_none())
        else {
            return Err(WdError::Validation(format!(
                "too many concurrent touches: logical slot {logical_slot} would exceed the {MAX_PHYSICAL_SLOTS}-touch runtime limit"
            )));
        };
        *owner = Some(logical_slot);
        let physical = u8::try_from(physical).map_err(|_| {
            WdError::Validation(format!(
                "slot index {physical} exceeds u8 range (limit {MAX_PHYSICAL_SLOTS})"
            ))
        })?;
        self.logical_to_physical.insert(logical_slot, physical);
        tracing::debug!(logical = logical_slot, physical, "wd-input: slot mapped");
        Ok(physical)
    }

    /// Release a logical slot, returning its physical slot if mapped.
    pub fn release(&mut self, logical_slot: u8) -> Option<u8> {
        let physical = self.logical_to_physical.remove(&logical_slot)?;
        self.physical_to_logical[usize::from(physical)] = None;
        tracing::debug!(logical = logical_slot, physical, "wd-input: slot released");
        Some(physical)
    }

    /// Number of active mappings.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.logical_to_physical.len()
    }

    /// Active physical slots, ascending.
    #[must_use]
    pub fn active_physical_slots(&self) -> Vec<u8> {
        self.physical_to_logical
            .iter()
            .enumerate()
            .filter_map(|(idx, owner)| {
                // `idx < MAX_TOUCHES = 10`, so the `u8` cast cannot truncate;
                // `u8::try_from` documents the bound without panicking.
                owner.and_then(|_| u8::try_from(idx).ok())
            })
            .collect()
    }

    /// Active `(logical, physical)` pairs.
    #[must_use]
    pub fn active_pairs(&self) -> Vec<(u8, u8)> {
        let mut out: Vec<(u8, u8)> = self
            .logical_to_physical
            .iter()
            .map(|(&logical, &physical)| (logical, physical))
            .collect();
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_existing_mapping() {
        let mut a = SlotAllocator::new();
        let first = a.ensure_physical(12).unwrap();
        assert_eq!(a.ensure_physical(12).unwrap(), first);
    }

    #[test]
    fn releases_and_reuses_physical() {
        let mut a = SlotAllocator::new();
        let first = a.ensure_physical(12).unwrap();
        let _ = a.ensure_physical(13).unwrap();
        assert_eq!(a.release(12), Some(first));
        assert_eq!(a.ensure_physical(14).unwrap(), first);
    }

    #[test]
    fn enforces_runtime_limit() {
        let mut a = SlotAllocator::new();
        let limit = usize::from(u8::MAX).min(MAX_PHYSICAL_SLOTS);
        for logical in 0..limit {
            let logical = u8::try_from(logical).expect("loop bounded by u8::MAX");
            a.ensure_physical(logical).unwrap();
        }
        assert!(a.ensure_physical(99).is_err());
    }

    #[test]
    fn maps_pubg_logical_range_into_pool() {
        let mut a = SlotAllocator::new();
        for logical in 0..10u8 {
            assert!(a.ensure_physical(logical).is_ok());
        }
        assert_eq!(a.active_count(), 10);
        for logical in 0..10u8 {
            a.release(logical);
        }
        for logical in 10..20u8 {
            assert!(a.ensure_physical(logical).is_ok());
        }
    }
}
