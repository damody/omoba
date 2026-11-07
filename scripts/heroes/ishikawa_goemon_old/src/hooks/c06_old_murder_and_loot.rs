//! c06_old_murder_and_loot（C06E_old 殺人越貨，舊版）：手寫的 hook（m3-rust-codegen-design.md §2.4）。
//!
//! 為什麼不能共用：給錢量是目標的金錢賞金乘 `bounty_mult`（原作 GetGoldBounty() * 4，C06.lua:303），
//! 詞彙表讀不到單位的賞金；host 的 `unit.gold_bounty`（工作線 B，edc90f0f）由 kit 的
//! `gold_bounty_of` 讀。行為與預期結果寫在 v3 `hooks.c06e_old_murder_and_loot_hook`。
//!
//! 目標不是英雄時：先讀賞金、給五右衛門 `bounty_mult` 倍的金錢，再以目標最大生命的純粹傷害擊殺它
//! （擊殺本身照常給一份賞金）；目標是英雄時只有音效與特效。
use super::*;

pub struct Impl;

impl Hook for Impl {
    fn on_cast(cx: &mut Cx<'_, '_>) -> Res {
        use super::super::values::c06_old_murder_and_loot as v;
        let target = cx.target_unit();
        cx.cue_event(super::super::ids::Cue::C06eOldSellSound.as_str(), target.into(), None, None, None);
        cx.cue_event(super::super::ids::Cue::C06eOldCoins.as_str(), target.into(), None, None, None);
        if !target.is_valid() || cx.is_hero(target) { return Ok(()); }
        // Read before the kill: a dead unit pays nothing more.
        let loot = cx.gold_bounty_of(target) * v::BOUNTY_MULT.at(cx.rank);
        let caster = cx.caster;
        cx.give_gold(caster, loot)?;
        let hp = cx.max_hp(target);
        cx.damage(target, hp, DamageKind::Pure, Dmg { require_source_alive: false, no_amplify: false })?;
        Ok(())
    }
}
