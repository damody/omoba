//! 目標排除（原作 `vscripts/library/common/dummy.lua:331-374` `_G.EXCLUDE_TARGET_NAME`）。
//!
//! - 原作：約 40 個單位名稱（將軍塔、兵營、地雷、dummy、土牆、忍者分身、A21 的武器……）。沒有全域
//!   強制，各技能、道具自己寫 `EXCLUDE_TARGET_NAME[name] == nil` 檢查，所以有的技能會漏掉。
//! - 這裡：改用單位 tag。單位模板或狀態把 tag 當成永久狀態（BuffStore 的狀態 id）帶在身上；
//!   selector 預設排除帶 [`UNTARGETABLE_BY_SPELLS`] 的單位，技能資料要選到它們必須明確宣告
//!   [`Inclusion::Everything`]。名稱比對只在建置期（單位模板）發生，執行期只看 tag。
use specs::Entity;

use crate::runtime::BuffStore;

/// 技能與道具的範圍、指定選取預設不會選到的單位（原作 EXCLUDE_TARGET_NAME）。
pub const UNTARGETABLE_BY_SPELLS: &str = "tag_untargetable_by_spells";

/// selector 的排除政策。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Inclusion {
    /// 預設：排除帶 [`UNTARGETABLE_BY_SPELLS`] 的單位。
    #[default]
    Default,
    /// 資料明確宣告要包含被排除的單位（原作少數技能沒有檢查清單）。
    Everything,
}

/// 單位是否帶某個 tag。
pub fn has_tag(buffs: &BuffStore, entity: Entity, tag: &str) -> bool { buffs.has(entity, tag) }

/// selector 是否要排除這個單位。
pub fn excluded(buffs: &BuffStore, entity: Entity, inclusion: Inclusion) -> bool {
    inclusion == Inclusion::Default && has_tag(buffs, entity, UNTARGETABLE_BY_SPELLS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn tagged_units_are_excluded_unless_the_data_asks_for_everything() {
        let mut world = World::new();
        let (dummy, hero) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        buffs.add(dummy, UNTARGETABLE_BY_SPELLS, Fixed64::from_raw(i64::MAX), serde_json::Value::Null);
        assert!(excluded(&buffs, dummy, Inclusion::Default));
        assert!(!excluded(&buffs, dummy, Inclusion::Everything));
        assert!(!excluded(&buffs, hero, Inclusion::Default));
    }
}
