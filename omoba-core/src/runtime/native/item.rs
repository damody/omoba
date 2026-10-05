use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ItemBonus {
    #[serde(default)]
    pub atk: f32,
    #[serde(default)]
    pub hp: f32,
    #[serde(default)]
    pub mp: f32,
    #[serde(default)]
    pub ms: f32,
    #[serde(default)]
    pub armor: f32,
    #[serde(default)]
    pub mp_regen: f32,
}

/// Native active-effect schema. Timed and one-shot modifiers use BuffStore;
/// RestoreMana uses the managed pool. Shield absorbs settled damage, not healing.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActiveEffect {
    /// Arm a one-shot flat physical bonus consumed by the next normal attack launch.
    HeadshotNext { bonus_damage: f32 },
    /// Timed damage absorption; remaining capacity is distinct from HP.
    Shield { amount: f32, duration: f32 },
    /// Restore managed mana, clamped to the current checked capacity.
    RestoreMana { amount: f32 },
    /// 短時間增加移速
    SprintBuff { ms_bonus: f32, duration: f32 },
    /// Timed all-packet reduction through the shared incoming damage modifier.
    DamageReduce { percent: f32, duration: f32 },
}

impl From<omoba_template_ids::MobaItemActiveConst> for ActiveEffect {
    fn from(value: omoba_template_ids::MobaItemActiveConst) -> Self {
        use omoba_template_ids::MobaItemActiveConst as Generated;
        match value {
            Generated::Shield { amount, duration } => Self::Shield { amount: amount.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::SprintBuff { ms_bonus, duration } => Self::SprintBuff { ms_bonus: ms_bonus.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::RestoreMana { amount } => Self::RestoreMana { amount: amount.to_f32_for_render() },
            Generated::DamageReduce { percent, duration } => Self::DamageReduce { percent: percent.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::HeadshotNext { bonus_damage } => Self::HeadshotNext { bonus_damage: bonus_damage.to_f32_for_render() },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemConfig {
    pub id: String,
    pub name: String,
    pub cost: i32,
    #[serde(default)]
    pub bonus: ItemBonus,
    #[serde(default)]
    pub active: Option<ActiveEffect>,
    #[serde(default)]
    pub cooldown: f32,
    /// 升級所需組件 item_id（可為空）。購買時會從背包消耗這些組件
    #[serde(default)]
    pub recipe: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ItemRegistry {
    pub items: HashMap<String, Arc<ItemConfig>>,
}

impl ItemRegistry {
    /// No runtime file IO or independent JSON authoring for MOBA equipment.
    pub fn generated_moba() -> Self {
        Self::from_configs(omoba_template_ids::MOBA_ITEM_CATALOG.iter().map(|item| ItemConfig {
            id: item.id.into(), name: item.name.into(), cost: item.cost,
            bonus: ItemBonus {
                atk: item.atk.to_f32_for_render(), hp: item.hp.to_f32_for_render(),
                ms: item.ms.to_f32_for_render(), armor: item.armor.to_f32_for_render(),
                ..ItemBonus::default()
            },
            active: item.active.map(ActiveEffect::from), cooldown: item.cooldown.to_f32_for_render(),
            recipe: item.recipe.iter().map(|id| (*id).into()).collect(),
        }).collect())
    }

    pub fn from_configs(list: Vec<ItemConfig>) -> Self {
        let mut items = HashMap::new();
        for cfg in list {
            items.insert(cfg.id.clone(), Arc::new(cfg));
        }
        ItemRegistry { items }
    }

    pub fn get(&self, id: &str) -> Option<Arc<ItemConfig>> {
        self.items.get(id).cloned()
    }
}

/// 回收售價（50%）
pub fn sell_price(cost: i32) -> i32 {
    (cost as f32 * 0.5) as i32
}
