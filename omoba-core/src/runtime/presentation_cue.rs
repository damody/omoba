//! One-shot presentation payloads contain replica identity only, never a hidden source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamagePresentationCue {
    pub tick: u64,
    pub target_id: u64,
    pub disclosure_epoch: u64,
    pub amount_milli: i64,
}

impl DamagePresentationCue {
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = b"DMG1".to_vec();
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.target_id.to_le_bytes());
        bytes.extend(self.disclosure_epoch.to_le_bytes());
        bytes.extend(self.amount_milli.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 36 || &bytes[..4] != b"DMG1" { return None; }
        let cue = Self {
            tick: u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            target_id: u64::from_le_bytes(bytes[12..20].try_into().ok()?),
            disclosure_epoch: u64::from_le_bytes(bytes[20..28].try_into().ok()?),
            amount_milli: i64::from_le_bytes(bytes[28..36].try_into().ok()?),
        };
        (cue.target_id != 0 && cue.disclosure_epoch != 0 && cue.amount_milli > 0).then_some(cue)
    }
}

/// Collision-free within one team/view stream. Reject overflow; never truncate an ID.
pub fn presentation_effect_id(tick: u64, sub_index: u32) -> Option<u64> {
    let tick = u32::try_from(tick).ok()?;
    Some((u64::from(tick) << 32) | u64::from(sub_index.checked_add(1)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_damage_payload_and_ids_fail_closed() {
        let cue = DamagePresentationCue {tick: 17, target_id: 4, disclosure_epoch: 3, amount_milli: 12000};
        let mut bytes = cue.encode();
        assert_eq!(DamagePresentationCue::decode(&bytes), Some(cue));
        bytes.push(0);
        assert_eq!(DamagePresentationCue::decode(&bytes), None);
        assert!(presentation_effect_id(u64::from(u32::MAX) + 1, 0).is_none());
        assert!(presentation_effect_id(1, u32::MAX).is_none());
        assert_ne!(presentation_effect_id(17, 0), presentation_effect_id(17, 1));
        assert_ne!(presentation_effect_id(17, 0), presentation_effect_id(18, 0));
    }
}
