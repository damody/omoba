-- Package schema v3 cross-document registry (v3.md §12.11, v3.2).
--
-- Names that several documents must spell the same way to mean the same thing:
--   unit_state  shared variables (scope hero / unit / match) — same name, same type, same variable
--   status_tags tags other documents read (has_status_tag, remove_status tags, immune_to tag:x)
--   item_tags   tags an item declares (items[].tags) for other documents (modify_cooldown item_tag)
--   cooldown_groups  shared cooldown group names; a series is named <series>_family
-- The validator rejects a shared name that is not here, and a tag read that no document
-- declares here or on its own status. Adding a row needs the source behind it (`source`).
-- Paths are under history_nobu game/dota_addons/nobu/scripts/ (commit f966720).
-- 2026-10-28: merged census-v3/registry-additions-{A,B,C,items}.lua (M4 cleanup).
return {
  unit_state = {
    -- 原作 unit.AphoticShieldRemaining：A04、A24、B08、B36、C06、C12 與河童道具共用的護盾剩餘量
    aphotic_shield_remaining = { type = "fixed", scope = "unit",
      source = "vscripts/heroes/A_Oda/A04_Takenaka_Juji.lua:246-283；vscripts/heroes/C_Neutral/C06.lua:168" },
    -- 原作 caster.raikiri：舞雷刀、雷切、大千鳥共用的保底計數
    raikiri = { type = "int", scope = "hero", source = "vscripts/items/Addon_Items/item_lightning_scroll.lua:224-239" },
    -- 原作 caster.nobuorb1：英雄層級的法球槽（v3 以 passive.orb_slot 表達；這裡登錄名稱給仍需讀它的條目）
    nobuorb1 = { type = "ability", scope = "hero", source = "vscripts/heroes/A_Oda/A21.lua:49,69" },
    -- 原作 caster.great_sword_of_disease：池田鬼神丸國重裝備時設 1、卸下時設 nil；A04W 水式神、A28T 鳳凰、
    -- 林之卷系列召喚（I4 林之章、B16 等）讀它決定生命 ×1.5
    great_sword_of_disease = { type = "bool", scope = "hero",
      source = "vscripts/items/Addon_Items/item_the_great_sword_of_disease.lua:5,32；vscripts/heroes/A_Oda/A04_Takenaka_Juji.lua:437；vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:321；vscripts/items/Addon_Items/item_forest_scroll.lua:157-159,183-185" },
    -- 原作 caster.swiftness_boots：流風捷履（pilot_b）與流風捷履・改（I4）共用的未受傷計數
    swiftness_calm = { type = "int", scope = "hero",
      source = "vscripts/items/Addon_Items/item_swiftness_boots.lua:5-11,20-21,32-33" },
    -- 原作全域變數 tsum（沒有 local）：B30T 的點燃補套與 B27T 的緩速補套共用同一個值、互相改寫
    -- （B30.lua 每命中重設為 0.1、每 0.1 秒加 0.1；B27.lua 施放時重設為 0.1、0.1 秒後加 0.1）。其餘英雄的 tsum 都是 local
    tsum = { type = "fixed", scope = "match",
      source = "vscripts/heroes/B_Unified/B30.lua:176-212；vscripts/heroes/B_Unified/B27.lua:168-178" },
  },
  status_tags = {
    -- rules layer (v3.md §12.3)
    slow = { source = "rules: 原作 dummy.lua Slow 系統的緩速狀態" },
    invisible = { source = "rules: 隱形狀態" },
    engine_invisible = { source = "rules: 原作引擎 modifier_invisible" },
    true_sight = { source = "rules: modifier_truesight" },
    cleanse_protected = { source = "rules: _G.EXCLUDE_MODIFIER_NAME（vscripts/library/common/dummy.lua:376-392）" },
    -- rules layer: set on every status whose caster still exists and whose source is an ability that is not an item
    from_ability = { source = "rules: 原作 CreateMirror 只重套 IsValidEntity(v:GetCaster()) 且 not v:GetAbility():IsItem() 的 modifier（vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:428-435；vscripts/heroes/C_Neutral/C02.lua:67-74,150-157；vscripts/heroes/C_Neutral/C16.lua:85-92）；A27W、C02W／E／R、C16D、B02W 讀取" },
    -- cross-document reads recorded by the census
    perceive_wine = { source = "item_perceive_wine 系列的魔免狀態；B22R 移除、天狗之葉等讀取" },
    isvoid = { source = "A32 的虛化狀態（vscripts/heroes/A_Oda/A32.lua:258,473）；鬼面、村正吸血排除" },
    record = { source = "modifier_record；B22T barrier 排除" },
    a09t_tentacle = { source = "modifier_A09T_tentacle；B22T barrier 排除" },
    c25e_mark = { source = "C25E 的標記；流風捷履排除" },
    zimbabwe = { source = "闇牙黃泉津的反射狀態；C02T 讀取" },
    a32r_void = { source = "A32R 的虛空狀態；B35R 讀取" },
    b13e_sleep = { source = "modifier_B13E_sleep（npc/B_Unified/B13.txt:239；vscripts/heroes/B_Unified/B13.lua:352,365）；vscripts/items/Addon_Items/item_flame_armor.lua:10 讀取（I0 鱗片甲、狐火大鎧，I4 吉光骨食）" },
    c08t_bleeding = { source = "modifier_C08T_bleeding（npc/C_Neutral/C08.txt:430）；vscripts/items/Addon_Items/item_rockfall_book.lua:32、item_storm_book.lua:102、item_the_overflame_art_of_war.lua:106 讀取" },
    mountain_scroll_lockout = { source = "modifier_item_mountain_scroll_resistance（npc/npc_items_custom_load.txt:1202,13643,13930,14145）；pilot_b、I4、I5 宣告並以 has_status_tag 讀取" },
    swiftness_calm = { source = "modifier_UntakeDamage（npc/npc_items_custom_load.txt:12840,12991）；pilot_b、I4 宣告，remove_status tags 讀取" },
    reito_lantern_lockout = { source = "modifier_arc_reito_lantern_cd（npc/npc_items_custom_load.txt:10852,13287）；pilot_a、I2 宣告，pilot_a 以 has_status_tag 讀取" },
    flame_armor_aura = { source = "modifier_flame_armor_aura（npc/npc_items_custom_load.txt:3045,6625,11319；vscripts/items/Addon_Items/item_flame_armor.lua）；I0、I4 宣告並讀取" },
    go_home = { source = "modifier_wantgohome／modifier_gohomelua（vscripts/items/Addon_Items/item_tpscroll.lua:9-21）；vscripts/items/Addon_Items/item_soul_adder.lua:34-36 移除" },
  },
  item_tags = {
    blink = { source = "閃現類道具（item_flash_ring、item_flash_shoes 等）；C13T 以 modify_cooldown item_tag 讓它們進冷卻" },
  },
  cooldown_groups = {
    -- series share cooldowns by group; every series name ends in _family (AbilitySharedCooldown in npc/npc_items_custom_load.txt)
    fire_family = { source = "item_fire_scroll 系列（vscripts/items/Addon_Items/item_fire_scroll.lua:27-35）" },
    wind_family = { source = "item_wind_scroll 系列（item_wind_scroll.lua:69-77）" },
    mountain_family = { source = "item_mountain_scroll 系列（item_mountain_scroll.lua:5-13）" },
    boots_family = { source = "鞋類 AbilitySharedCooldown 群組（source_items shares_cooldown_with）" },
    forest_family = { source = "share_forest_scroll：item_forest_scroll 14697、item_the_art_of_war_forest_chapter 1557、item_the_lost_art_of_war_1 13663、item_the_art_of_war 13978" },
    lightning_family = { source = "share_lightning_scroll：item_lightning_scroll 1663、item_lightning_katana 3635、item_raikiri 10383、item_the_red_lightning_chapter 143、item_the_red_lightning_chapter_hyper 20" },
    flash_family = { source = "share_flash_ring：item_flash_ring 1009、item_flash_shoes 16431" },
    flood_family = { source = "share_item_flood_book：item_flood_book 14365" },
    ice_storm_family = { source = "item_ice_storm_book：item_storm_book 14573" },
    yubashiri_family = { source = "item_yubashiri：item_kokumo 10182 的 AbilitySharedCooldown；item_yubashiri 本身沒有宣告，推定以名稱相符而共用（batch_I5 quirk）" },
    overflame_family = { source = "share_the_overflame_art_of_war：item_the_overflame_art_of_war 550、item_the_overflame_art_of_war_hyper 435" },
    ronin_jitte_family = { source = "share_ronin_jitte：item_ronin_jitte 7040" },
  },
}
