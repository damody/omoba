//! Bounded, typed owner-team settlement. Never carries JSON, prices, actor
//! targets, or client-chosen balances. Float bit patterns preserve the existing
//! item component representation exactly; every decoded value is validated.
use super::comp::{Gold, Inventory, ItemEffects, ItemInstance};
use serde::{Deserialize, Serialize};

/// Persistent player-owned HUD data, independent of a live render entity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OwnerEconomyState {
    pub player_id: u32,
    pub economy: CommittedEconomyState,
    pub shop_available: bool,
}

impl OwnerEconomyState {
    pub const PAYLOAD_LEN: usize = 86;
    pub fn encode(&self) -> Vec<u8> {
        [
            self.player_id.to_le_bytes().as_slice(),
            self.economy.encode().as_slice(),
            &[u8::from(self.shop_available)],
        ]
        .concat()
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN || bytes[85] > 1 {
            return None;
        }
        let player_id = u32::from_le_bytes(bytes[..4].try_into().ok()?);
        if player_id == 0 {
            return None;
        }
        Some(Self {
            player_id,
            economy: CommittedEconomyState::decode(&bytes[4..85]).ok()?,
            shop_available: bytes[85] == 1,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CommittedEconomyState {
    pub gold: i32,
    pub item_ids: [u16; 6],
    pub cooldown_bits: [u32; 6],
    pub effect_bits: [u32; 10],
    pub dirty: bool,
}

impl CommittedEconomyState {
    pub const PAYLOAD_LEN: usize = 81;

    pub fn capture(
        gold: Gold,
        inventory: &Inventory,
        effects: ItemEffects,
    ) -> Result<Self, &'static str> {
        let mut state = Self {
            gold: gold.0,
            item_ids: [0; 6],
            cooldown_bits: [0; 6],
            effect_bits: [
                effects.bonus_atk,
                effects.bonus_hp,
                effects.bonus_mp,
                effects.bonus_ms,
                effects.bonus_armor,
                effects.bonus_mp_regen,
                effects.applied_atk,
                effects.applied_hp,
                effects.applied_ms,
                effects.applied_armor,
            ]
            .map(f32::to_bits),
            dirty: effects.dirty,
        };
        for (slot, item) in inventory.items() {
            state.item_ids[slot] = omoba_template_ids::MOBA_ITEM_CATALOG
                .iter()
                .find(|entry| entry.id == item.item_id)
                .ok_or("unknown MOBA catalog item")?
                .catalog_id;
            state.cooldown_bits[slot] = item.cooldown_remaining.to_bits();
        }
        state.components()?;
        Ok(state)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::PAYLOAD_LEN);
        bytes.extend(self.gold.to_le_bytes());
        for slot in 0..6 {
            bytes.extend(self.item_ids[slot].to_le_bytes());
            bytes.extend(self.cooldown_bits[slot].to_le_bytes());
        }
        for bits in self.effect_bits {
            bytes.extend(bits.to_le_bytes());
        }
        bytes.push(u8::from(self.dirty));
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != Self::PAYLOAD_LEN || bytes[80] > 1 {
            return Err("malformed economy payload");
        }
        let mut state = Self {
            gold: i32::from_le_bytes(bytes[..4].try_into().unwrap()),
            item_ids: [0; 6],
            cooldown_bits: [0; 6],
            effect_bits: [0; 10],
            dirty: bytes[80] != 0,
        };
        for slot in 0..6 {
            let offset = 4 + slot * 6;
            state.item_ids[slot] =
                u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
            state.cooldown_bits[slot] =
                u32::from_le_bytes(bytes[offset + 2..offset + 6].try_into().unwrap());
        }
        for (index, value) in state.effect_bits.iter_mut().enumerate() {
            let offset = 40 + index * 4;
            *value = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        }
        state.components()?;
        Ok(state)
    }

    pub fn components(&self) -> Result<(Gold, Inventory, ItemEffects), &'static str> {
        if self.gold < 0 {
            return Err("negative economy balance");
        }
        let mut inventory = Inventory::default();
        for slot in 0..6 {
            let cooldown = f32::from_bits(self.cooldown_bits[slot]);
            if !cooldown.is_finite() || cooldown < 0.0 || cooldown > 86400.0 {
                return Err("invalid item cooldown");
            }
            if self.item_ids[slot] == 0 {
                if self.cooldown_bits[slot] != 0 {
                    return Err("empty slot has cooldown");
                }
            } else {
                let item = omoba_template_ids::MOBA_ITEM_CATALOG
                    .iter()
                    .find(|item| item.catalog_id == self.item_ids[slot])
                    .ok_or("unknown catalog id")?;
                inventory.slots[slot] = Some(ItemInstance {
                    item_id: item.id.into(),
                    cooldown_remaining: cooldown,
                });
            }
        }
        let values = self.effect_bits.map(f32::from_bits);
        if values
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0 || *v > 600_000.0)
        {
            return Err("invalid equipment effects");
        }
        let effects = ItemEffects {
            bonus_atk: values[0],
            bonus_hp: values[1],
            bonus_mp: values[2],
            bonus_ms: values[3],
            bonus_armor: values[4],
            bonus_mp_regen: values[5],
            applied_atk: values[6],
            applied_hp: values[7],
            applied_ms: values[8],
            applied_armor: values[9],
            dirty: self.dirty,
        };
        Ok((Gold(self.gold), inventory, effects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_codec_is_bounded_and_validates_identity_and_flag() {
        let state = OwnerEconomyState {
            player_id: 7,
            shop_available: true,
            economy: CommittedEconomyState::capture(
                Gold(5),
                &Inventory::default(),
                ItemEffects::default(),
            )
            .unwrap(),
        };
        assert_eq!(
            OwnerEconomyState::decode(&state.encode()),
            Some(state.clone())
        );
        assert!(OwnerEconomyState::decode(&state.encode()[..85]).is_none());
        let mut bytes = state.encode();
        bytes[85] = 2;
        assert!(OwnerEconomyState::decode(&bytes).is_none());
        let mut bytes = state.encode();
        bytes[..4].fill(0);
        assert!(OwnerEconomyState::decode(&bytes).is_none());
    }
    #[test]
    fn economy_codec_roundtrip_and_fail_closed() {
        let mut inventory = Inventory::default();
        inventory.slots[5] = Some(ItemInstance {
            item_id: "moba_sword".into(),
            cooldown_remaining: 0.25,
        });
        let state = CommittedEconomyState::capture(
            Gold(500),
            &inventory,
            ItemEffects {
                bonus_atk: 10.0,
                applied_atk: 10.0,
                dirty: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            CommittedEconomyState::decode(&state.encode()).unwrap(),
            state
        );
        for offset in [0, 4, 40, 80] {
            let mut bytes = state.encode();
            match offset {
                0 => bytes[..4].copy_from_slice(&(-1i32).to_le_bytes()),
                4 => bytes[4..6].copy_from_slice(&u16::MAX.to_le_bytes()),
                40 => bytes[40..44].copy_from_slice(&f32::NAN.to_bits().to_le_bytes()),
                _ => bytes[80] = 2,
            }
            assert!(CommittedEconomyState::decode(&bytes).is_err());
        }
        assert!(CommittedEconomyState::decode(&state.encode()[..80]).is_err());
        let mut invalid = state;
        invalid.cooldown_bits[0] = 1.0f32.to_bits();
        assert!(invalid.components().is_err());
    }
}
