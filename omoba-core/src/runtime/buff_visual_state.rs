//! Only catalog identity and remaining time; never arbitrary payload or source.
use omoba_template_ids::{buff_by_name, try_buff_id_str, BuffId};
pub const SCHEMA_ID: u32 = 0x464f4711;
pub const MAX_VISUAL_BUFFS: usize = 64;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuffVisualState(pub Vec<(u16, i64)>);
impl BuffVisualState {
    pub fn from_store(store: &crate::runtime::BuffStore, entity: specs::Entity) -> Self {
        let mut values = std::collections::BTreeMap::<u16, i64>::new();
        for (id, remaining) in store.iter_for(entity).filter_map(|(name, entry)| {
            let id = buff_by_name(name)
                .or_else(|| {
                    let (ability, stat) = omb_script_abi::buff_ids::declarative_buff_source(name)?;
                    omoba_template_ids::declarative_buff_visual_id(ability, stat)
                })?
                .raw();
            let raw = entry.remaining.raw();
            (raw != 0).then_some((id, if raw < 0 { -1 } else { raw }))
        }) {
            // A public icon means at least one source remains; no source count,
            // caster identity or internal key crosses the presentation boundary.
            values
                .entry(id)
                .and_modify(|old| {
                    *old = if *old == -1 || remaining == -1 {
                        -1
                    } else {
                        (*old).max(remaining)
                    };
                })
                .or_insert(remaining);
        }
        Self(values.into_iter().take(MAX_VISUAL_BUFFS).collect())
    }
    pub fn encode(&self) -> Option<Vec<u8>> {
        if self.0.len() > MAX_VISUAL_BUFFS {
            return None;
        }
        let mut bytes = b"BVS1".to_vec();
        bytes.extend((self.0.len() as u16).to_le_bytes());
        let mut previous = 0;
        for &(id, raw) in &self.0 {
            if id <= previous || try_buff_id_str(BuffId(id)).is_none() || (raw != -1 && raw <= 0) {
                return None;
            }
            previous = id;
            bytes.extend(id.to_le_bytes());
            bytes.extend(raw.to_le_bytes());
        }
        Some(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 6 || &bytes[..4] != b"BVS1" {
            return None;
        }
        let count = u16::from_le_bytes(bytes[4..6].try_into().ok()?) as usize;
        if count > MAX_VISUAL_BUFFS || bytes.len() != 6 + 10 * count {
            return None;
        }
        let state = Self(
            bytes[6..]
                .chunks_exact(10)
                .map(|row| {
                    (
                        u16::from_le_bytes(row[..2].try_into().unwrap()),
                        i64::from_le_bytes(row[2..].try_into().unwrap()),
                    )
                })
                .collect(),
        );
        state.encode()?;
        Some(state)
    }
    pub fn render_buffs(&self) -> Vec<crate::runtime::BuffSnapshot> {
        self.0
            .iter()
            .filter_map(|&(id, raw)| {
                Some(crate::runtime::BuffSnapshot {
                    buff_id: try_buff_id_str(BuffId(id))?.into(),
                    remaining_secs: if raw == -1 {
                        -1.0
                    } else {
                        omoba_sim::Fixed64::from_raw(raw).to_f32_for_render()
                    },
                    payload_json: String::new(),
                })
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, WorldExt};
    #[test]
    fn buff_visual_state_codec_is_strict() {
        let id = buff_by_name("slow").unwrap().raw();
        let state = BuffVisualState(vec![(id, 1024)]);
        let bytes = state.encode().unwrap();
        assert_eq!(BuffVisualState::decode(&bytes), Some(state));
        for length in 0..bytes.len() {
            assert!(BuffVisualState::decode(&bytes[..length]).is_none());
        }
        let mut extra = bytes;
        extra.push(0);
        assert!(BuffVisualState::decode(&extra).is_none());
        for rows in [
            vec![(0, 1)],
            vec![(id, 0)],
            vec![(id, -2)],
            vec![(id, 1), (id, 2)],
            vec![(u16::MAX, 1)],
            vec![(id, 1); 65],
        ] {
            assert!(BuffVisualState(rows).encode().is_none());
        }
        assert!(BuffVisualState(vec![(id, -1)]).encode().is_some());
        assert_eq!(
            BuffVisualState::decode(&BuffVisualState(vec![]).encode().unwrap()),
            Some(BuffVisualState(vec![]))
        );
    }
    #[test]
    fn buff_visual_state_excludes_private_payload_and_dynamic_names() {
        let mut world = specs::World::new();
        let entity = world.create_entity().build();
        let mut store = crate::runtime::BuffStore::default();
        store.add(
            entity,
            "slow",
            omoba_sim::Fixed64::from_i32(2),
            serde_json::json!({"source":999,"secret":"private"}),
        );
        store.add(
            entity,
            "generic_mana:hidden:123:456",
            omoba_sim::Fixed64::ONE,
            serde_json::json!({}),
        );
        let state = BuffVisualState::from_store(&store, entity);
        assert_eq!(state.0.len(), 1);
        let buffs = state.render_buffs();
        assert_eq!(buffs[0].buff_id, "slow");
        assert!(buffs[0].payload_json.is_empty());
        assert_eq!(buffs[0].remaining_secs, 2.0);
    }

    #[test]
    fn buff_visual_state_declared_sources_merge_without_private_identity() {
        let mut world = specs::World::new();
        let entity = world.create_entity().build();
        let mut store = crate::runtime::BuffStore::default();
        for (key, seconds) in [
            ("slow", 2),
            ("generic_slow:ranger_shot:100:3", 4),
            ("generic_slow:ranger_shot:101:3", 3),
            ("generic_mana:ranger_patch:mana_regen_constant:100:3", 5),
            ("generic_mana:vanguard_recover:mana_bonus:100:3", 6),
            ("generic_mana:ranger_patch:mana_bonus:100:3", 9),
            ("generic_slow:missing:100:3", 9),
            ("generic_slow:ranger_shot:0100:3", 9),
        ] {
            store.add(
                entity,
                key,
                omoba_sim::Fixed64::from_i32(seconds),
                serde_json::json!({"secret":999}),
            );
        }
        let state = BuffVisualState::from_store(&store, entity);
        assert_eq!(state.0.len(), 3);
        assert!(
            state.encode().is_some(),
            "merged sources keep strict unique catalog IDs"
        );
        let buffs = state.render_buffs();
        assert!(buffs
            .iter()
            .all(|b| b.payload_json.is_empty() && !b.buff_id.contains(':')));
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "slow")
                .unwrap()
                .remaining_secs,
            4.0
        );
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "mana_regeneration")
                .unwrap()
                .remaining_secs,
            5.0
        );
        assert_eq!(
            buffs
                .iter()
                .find(|b| b.buff_id == "mana_capacity")
                .unwrap()
                .remaining_secs,
            6.0
        );
        store.remove(entity, "generic_slow:ranger_shot:100:3");
        assert_eq!(
            BuffVisualState::from_store(&store, entity)
                .render_buffs()
                .iter()
                .find(|b| b.buff_id == "slow")
                .unwrap()
                .remaining_secs,
            3.0
        );
        store.tick(omoba_sim::Fixed64::from_i32(6));
        assert!(BuffVisualState::from_store(&store, entity).0.is_empty());
    }
}
