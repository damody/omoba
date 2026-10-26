-- Provenance for H01. raw_sha256 values pin the exact source blocks read; the
-- validator rejects the package when the source catalog no longer matches.
return function(ctx)
  return {
    schema_version = 2,
    provenance = {
      repository = "https://github.com/damody/nobugg_official",
      commit = "ca6ee8602473f6015a397cc121dac6baeb70b2e5",
      catalog = "docs/plans/2026-10-24-omfue-month-mvp/source-index/source-catalog.lua",
      source_hero_id = "B01",
      variant = "current",
      language = "zh-TW",
      unit_file = { path = "npc/B_Unified/B01_2.txt", line = 4,
        raw_sha256 = "e306356d716c68c6decb55af88fbccc731a47b00ef9f66aa150d9a96c4014d3b" },
    },
    abilities = {
      { source_ability_id = "B01W", path = "npc/B_Unified/B01.txt", line = 15,
        raw_sha256 = "a8a9b2edc0078f966f324dbf952bee79377a67e54b7b690ef3f7880451c7f226",
        tooltip_sha256 = "0e038da6e72006091e3ea265f76acac3696f2562ae12bb31cae52612bb92d55a" },
      { source_ability_id = "B01E", path = "npc/B_Unified/B01.txt", line = 313,
        raw_sha256 = "10c5e4960b784cef4e88b1f217c30838de2ea47e155845aac59dedc9e3c4a153",
        tooltip_sha256 = "280b268800f06da9794c64c837a8e396732e10cf8dfa8a4a108ab53be296a555" },
      { source_ability_id = "B01R", path = "npc/B_Unified/B01.txt", line = 219,
        raw_sha256 = "d87a398674d8ac730617928643bc7d54951531158179fbc15e120d716c1928ee",
        tooltip_sha256 = "207e71119222616b5811a4fb3ce2da6f6270e7a6c54c9095db1c52a9da9210bc" },
      { source_ability_id = "B01T", path = "npc/B_Unified/B01.txt", line = 457,
        raw_sha256 = "2af903b3e7ff36f2ae5f9acc6765cc55ae0aa85799fd33f918552c0adffdbf1a",
        tooltip_sha256 = "41ec1e18c368695161ee87dfbf69813cab48e30c40fe3305a93bfd0dd3485d6e" },
    },
    assumptions = {
      { id = "distance_units", kind = "assumed",
        text = "1 個來源距離單位 = 1 個本遊戲世界單位（與本地英雄 700～900 射程同量級）；換算集中在 package，不散落在 handler。" },
      { id = "base_stats_from_local", kind = "adaptation",
        text = "來源生命／法力由 Dota 三圍推導；英雄模板雖有力量／敏捷／智力欄位，但 MOBA 戰鬥使用 authored 數值，舊屬性公式（comp/hero.rs）與 StatsAgilityBonus 在 MOBA 路徑沒有消費者；基礎生命 580、法力 300、攻擊 52、護甲 1.5 沿用本地 saika_magoichi。射程 700、移速 350 取來源（本地是 900、320）。" },
      { id = "night_vision_not_applicable", kind = "not_applicable",
        text = "B01W 的夜晚視野 +1000：本遊戲沒有日夜循環，此效果不改變玩法，記錄為不適用，不實作。" },
      { id = "w_range_text_conflict", kind = "kv_text_conflict",
        text = "B01W 描述寫「攻擊距離提升至 900/1050/1200/1400」（即 AbilityCastRange），KV modifier 是 bonus_atk_rng 200/350/500/700。採用 KV 加成值；700 基礎射程下最終射程為 900/1050/1200/1400，兩者一致。" },
      { id = "gunner_stats_from_local", kind = "assumed",
        text = "B01E 召喚物的生命、攻擊、射程、存活時間在缺失的 vscript 中；新模板 b01_saika_corps_gunner 的數值沿用本地 saika_gunner（生命 400、攻擊 45、射程 450），存活 60 秒，攻擊依英雄等級 +10（取描述）、不能移動。主人死亡時保留到期（persist）。" },
      { id = "gunner_reserve_start", kind = "assumed",
        text = "B01E 學會時儲存 1 位（來源 OnUpgrade 由 vscript 決定，缺失）；之後每 15 秒 +1，最多 2/3/4/5。" },
      { id = "three_stage_no_multishot", kind = "adaptation",
        text = "B01T「三段技法」來源 KV 只有 BASEDAMAGEOUTGOING_PERCENTAGE +200%，沒有連發判定；只做增傷。本地 three_stage_technique 的 3 發視覺可保留為呈現，但不得多判定傷害。" },
      { id = "local_kit_differences", kind = "adaptation",
        text = "與本地 B01_saika_magoichi 的差異：sniper_mode 本地是 toggle（無 CD／法力、降攻速移速），來源 current 是 12 秒限時增益（CD 22、法力 100）；saika_reinforcements 本地是兩排陣型一次召喚多位，來源是儲存制、每次 1 位、不能移動；rain_iron_cannon 本地是扇形真實濺射，來源是依目標類別的物理加成；three_stage_technique 本地 5 秒、4 級，來源 7/11/13 秒、3 級、CD 135/123/111。本地實作只當程式起點，新技能以新 ID 附加，不覆蓋舊技能。" },
    },
  }
end
