//! Source `OnAbilityExecuted` filter for hero `UnitScript::on_skill_cast`
//! hooks. The host calls that hook for every committed ability cast (skill id
//! = ability id) and every committed item active use (skill id =
//! `ITEM_SKILL_PREFIX + item id`, `ScriptEvent::ItemUsed`). Source scripts skip
//! toggle abilities (`ability:IsToggle()`) and a per-script list of item names;
//! the game item id is the source key without its `item_` prefix.
use omoba_core::runtime::scripting::ITEM_SKILL_PREFIX;
use omoba_template_ids::{AbilityTypeC,ability_by_name,active_ability_const};

/// B08R_Action（B08_Asai_Nagamasa.lua:252-255）：item_logging、item_tpscroll、item_samurai_cloth。
pub(crate) const B08R_EXCLUDED_ITEMS:&[&str]=&["logging","tpscroll","samurai_cloth"];
/// new_C17D（C17.lua:49-54）：再加 item_flash_shoes、item_flash_ring。
pub(crate) const C17D_EXCLUDED_ITEMS:&[&str]=&["logging","tpscroll","flash_shoes","flash_ring","samurai_cloth"];

/// The item id of an item-use hook call, `None` for an ability cast.
pub(crate) fn used_item(skill_id:&str)->Option<&str> {skill_id.strip_prefix(ITEM_SKILL_PREFIX)}

/// Whether a hook call counts as an executed ability for the source passive.
pub(crate) fn counts(skill_id:&str,excluded_items:&[&str])->bool {
    match used_item(skill_id) {
        Some(item)=>!excluded_items.contains(&item),
        None=>!ability_by_name(skill_id).and_then(active_ability_const).is_some_and(|c|c.ability_type==AbilityTypeC::Toggle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn items_and_abilities_follow_the_source_exclusions() {
        assert!(counts("b08_fang_slash",B08R_EXCLUDED_ITEMS));
        assert!(counts("item:moba_stride_charm",B08R_EXCLUDED_ITEMS));
        for item in ["logging","tpscroll","samurai_cloth"] {
            assert!(!counts(&format!("item:{item}"),B08R_EXCLUDED_ITEMS));
            assert!(!counts(&format!("item:{item}"),C17D_EXCLUDED_ITEMS));
        }
        assert!(counts("item:flash_ring",B08R_EXCLUDED_ITEMS),"B08R does not skip the flash ring");
        assert!(!counts("item:flash_ring",C17D_EXCLUDED_ITEMS));
        assert!(!counts("item:flash_shoes",C17D_EXCLUDED_ITEMS));
        assert_eq!(used_item("item:rations"),Some("rations"));
        assert_eq!(used_item("c17_light_chain"),None);
        // Every authored toggle is skipped.
        let toggles:Vec<_>=(1..=u16::MAX).map_while(|raw|omoba_template_ids::try_ability_id_str(omoba_template_ids::AbilityId(raw)).map(|s|(raw,s)))
            .filter(|(raw,_)|active_ability_const(omoba_template_ids::AbilityId(*raw)).is_some_and(|c|c.ability_type==AbilityTypeC::Toggle)).collect();
        for (_,id) in toggles {assert!(!counts(id,C17D_EXCLUDED_ITEMS),"{id}");}
    }
}
