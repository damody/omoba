# A 線：道具上架盤點（2026-11-08）

## 總數

以 3b147d1a（catalog 到 236）之後的 live，加上 A 線這次的產生器修正來數。

- 道具文件 170 件（docs/character_pipeline/items），原作目錄 source_items.lua 157 件。
- 6 件原作道具看起來沒有 v3 文件，其實只是 id 大小寫不同：Hannya_Onimaru 和 hannya_onimaru 這類，文件都在。
- 能產生的有 134 件。其餘 36 件分成兩類：
  - 18 件產生器只回報不在商店。E 查證後：3 件能上架，10 件原作沒有 KV 不移植，4 件等配方零件，1 件等 host 支援 unit_or_point 目標。見下節。
  - 18 件卡在 ability_kit／host（B）。
- A 線產生器這一層的缺口已清空。

查法：`tools\lua\lua.exe target\lineA\item_all.lua <root>` 列出每件道具的 blockers；`v3_codegen_coverage.lua` 是同一份資料。

## A 線這次補的（v3_rust.lua，land 中）

- `apply_status` 帶 `source` 施加的吸收盾（apply_status_from）：盾屬於那個 source，盾量以 source 為 caster 來讀。解開 item_devil_supressor_armor（惡鬼鎮壓甲）。
- 沒有自身效果的道具（例如合成零件破魔寶珠 item_hama_orb）不再擋在「rust_handler without behavior」。它本來就已上架，只是沒有腳本。

## 產生器只回報「不在商店」的 18 件（E 線查證後修正）

產生器和 kit 都不擋，阻擋只剩 `catalog: … is not in the shop (templates/moba_items.lua)`。E 線逐件查過，實際狀況如下。

**能上架（3 件）**
- item_devil_supressor_armor（制魔腹卷，237）：E land 中。
- item_nannbann_armor（南蠻胴具足，238）：E land 中。
- item_samurais_badge（侍的徽章，205 原本已上架但不是 scripted）：原作 key 是 `item_samurai's_badge`，content_catalog 比對 id 時不收撇號。A 線已修，修好後 E 改成 scripted。

**不能上架**
- **原作沒有 KV 定義，依 D13 不移植（10 件）**：shintogo_kunimitsu、ancient_doll、ghost_gloves、gold_to_prestige、hachiman_bell、ice_book、kat_steel、ksitigarbha、magic_ring、poisonous_ring。source_items 也沒有它們。
- **配方零件還沒上架（4 件）**：item_sources_check 要求零件先上架。
  - insight_gem 要洞察碎片 insight_gem_shard，碎片卡在 B 的 on_owner_death。
  - cross_spear、prophet_armor 都用 insight_gem。
  - prophet_akazonae_armor 用 prophet_armor。
  - B 補好 on_owner_death 後，碎片、寶石、十字槍、真知短甲、真視赤備挂甲依序就能上架。
- **host 的道具目錄沒有對應的目標形狀（1 件）**：great_dragon 的 target 是 unit_or_point，目錄只有 unit 和 point，要 B 支援。

## 卡在 ability_kit／host（B 線）：18 件

| 道具 | 文件 | 缺的 |
|---|---|---|
| item_death_of_scepter、item_muramasa_katana | pilot_b | 道具的被動 on_death、on_respawn（同上，沒有 learn marker） |
| item_insight_gem_shard、item_insight_gem_ninja | batch_I1 | event:on_owner_death |
| item_insight_gem_hero | batch_I1 | event:on_owner_death；aura.types building+hero |
| item_moon_mirror | batch_I2 | 道具實例範圍（scope item_instance）的 unit_state，型別 bool 和 unit。`cue.at of a err` 是讀這個 unit 狀態的連帶阻擋，補好就會消失 |
| item_reito_lantern | pilot_a | 同 moon_mirror |
| item_soul_adder | batch_I3 | refund mana 或 health |
| item_the_spear_of_saddle | batch_I5 | passive.orb_slot：只支援「開關技能在開啟時佔用、在狀態結束時釋放」的形式，這件不是 |
| item_the_dark_yellow_teeth_and_zimbabwe | pilot_a | damage.type:event（沿用事件傷害的類型） |
| item_earth_wall_book | pilot_a | item.active.cost.charges；unit_template blocks_pathing 但沒有 collision_radius |
| item_the_great_sword_of_invincible | pilot_b | stat.when `not primary_is(self, agi／int／str)` |
| item_ronin_jitte | pilot_b | projectile.lifetime |
| item_the_great_sword_of_teleport | pilot_b | item.active.cast.interruptible（host）；另有兩項產生器缺口：has_tag(courier) 和 target.types 含 courier，要 host 先有 courier 單位 tag 與型別，A 線再接 |

按數量排，B 線一次能解開最多件的是：
1. remove_status 的 exclude_source_tags／min_duration：3 件。
2. 道具被動的 on_death／on_respawn／on_ability_executed：3 件。
3. on_owner_death：3 件。
4. item_instance 範圍的 unit_state：2 件。
B 補好一項，A 線就接產生器那一側，再請 E 上架。

## 2026-11-08 更新（A 線 d5763558）

remove_status 來源篩選（B c69d3adf）與道具被動 on_ability_executed 已在產生器接上，下列 4 件從 B 層移到 E（只差上架）：

- item_frozen_book（pilot_a）
- item_ice_storm_book（batch_I1）
- item_the_great_sword_of_drain（batch_I4）
- item_bizennosafunekanemitsu（batch_I0）：hp_regen 文件寫 10、來源目錄 30，上架前要決定（加 source_differs 或改文件）

已在影子暫時上架這 4 件跑過 items_impl 測試。

## 2026-11-08 更新 2（A 線，接 B 7b32a3e1 on_owner_death）

- 產生器支援道具 active `events.on_owner_death` 和 passive `on_death`（marker mask 524288，kind 22）、`item_count`，pick_item 也收 `slots = "all"`（主機沒有背包，等於物品欄）。
- item_insight_gem_shard、item_insight_gem_ninja：產生器沒有阻擋了，只剩上架。但 ninja（和 hero）在 source_items 沒有原作 KV 列，碎片的配方零件上不了架，所以碎片也上不了。這部分要 E／main 決定。
- item_insight_gem_hero：只剩 aura.types building+hero（B）。
- item_death_of_scepter、item_muramasa_katana：on_death 已經可以用，只剩 on_respawn（B 的下一項）。
- 已請 B 處理：host 的 item_grant::source_id／item_in_slot 回傳原作 key（例如 item_insight_gem_Shard），和 v3 id 對不上。產生器生成的比對會失敗。受影響的有：Hannya_Onimaru、Oochidorijuumonnjiyari、Sukemune、Sukezane、insight_gem_Shard、prophet_Akazonae_armor、samurai's_badge。

## D 線查證：insight_gem_hero／ninja 有原作 KV（2026-11-08）
- 原作 KV：真視碎片 `item_insight_gem_hero` 在 `npc/npc_items_custom_load.txt:16091-16237`（ItemCost 250、AbilityCooldown 30、AOERadius 1200、光環 600）；真知碎片 `item_insight_gem_ninja` 在 `:16238-16385`（ItemCost 150、AbilityCooldown 30、AOERadius 1200）。兩件都在商店 `shops/nobu_shops.txt:13-14、40-41、238-239`（consumables／pregame／sideshop1）。
- 兩件都有 catalog 列：`source-index/source-item-catalog.lua:6839`、`:7037`（listed = true）。也都有 source_items 列：`scripts/lua_data/templates/source_items.lua:631`（cost 250）、`:642`（cost 150），source_ref `history_nobu@f966720b…:item_insight_gem_hero／ninja`。不需要補資料，前面寫的「source_items 沒有原作 KV 列」不正確。
- 碎片上不了架的真正原因：`item_sources_check.lua:72-82` 要求配方零件必須是已上架的 MOBA 道具（by_id），而 hero／ninja 還沒上架。依上面的清單，ninja 已沒有阻擋，可以直接上架；hero 只剩 aura.types building+hero（B 線）。之後依序是碎片 → 寶石 → 十字槍／真知短甲 → 真視赤備挂甲。另外，碎片的 host source_id 大小寫（item_insight_gem_Shard）問題，前面已請 B 線處理。

## 2026-11-08 更新 3（A 線 57bfb4d5、1c37f120）

- 57bfb4d5：道具的 on_owner_death 和 on_death、item_count、pick_item slots all 已 land。
- 1c37f120：道具被動 on_respawn 已 land。
- 交給 E（只差上架）：
  - item_insight_gem_ninja
  - item_death_of_scepter：spell_lifesteal_pct 在 shelf 沒有數字；mana_regen 文件 5、來源 5.5
  - item_muramasa_katana：lifesteal_pct 在 shelf 沒有數字；attack 文件 30、來源 31
- item_insight_gem_shard：產生器沒有阻擋，要等 hero 上架。hero 只剩 aura.types building+hero（B）。
- B e2965050 已修好 item_grant 的 v3 id，前面提到的 id 對應問題結案。

## A 線更新 4（2026-11-08）

- 9b7bf2d8：item_instance 狀態 bool／unit 讀寫（B 4b32ae10）。月輪鏡 item_moon_mirror、靈都宮燈 item_reito_lantern 可產生，已通知 E 上架。
- 還沒解決：道具註冊 id。entry_head 遇到 source_id:lower()==id 時用原作 source key 註冊，host item_script_id 也查原作 key；samurais_badge 這種帶撇號的會對不上。統一改用 v3 id 的方案已提給 B，等回覆。
- 剩下的阻擋：soul_adder refund、spear_of_saddle orb_slot、dark_yellow_teeth damage.type:event、earth_wall_book charges／blocks_pathing、great_sword_of_invincible stat.when、ronin_jitte projectile.lifetime、great_sword_of_teleport courier 等、insight_gem_hero aura types。

## A 線更新 5（2026-11-09）

- 94f671d9：select／aura types building+hero → HeroesAndBuildings，真視寶石（英雄）可產生；forms 測試的逝者之笏 marker 改成只比對 mask。
- c38bd3db：道具一律以 v3 id 註冊（拿掉 entry_head 的特例，配 B 039cc0df），侍的徽章找得到腳本。
- a9ae7133：refund mana 在道具施放內放行，魂蛇可產生。
- 1b8af10c：projectile.lifetime → projectile_expiring，浪人十手可產生。
- 38f10132：NotPrimary、damage type event、道具 orb_slot release=unequip；不敗大劍、暗黃之牙與辛巴威、鞍切可產生。
- 以上都已通知 E 上架。
- 還卡在 B：
  - 土牆之書 cost.charges（道具充能）。它的 blocks_pathing 只是文件少了 collision_radius，A 線補 collision 80 並重生 v3_summons；這會改內容 hash，等 charges 到了再一起 land，patch 在 target/lineA/wall_patch.lua。
  - 千里大劍：courier 標籤、cast.interruptible、target types 非英雄＋信使。
- 影子 cargo target 已用 cargo_target_retention 清掉 9.9 GB；之後影子 cargo 一律加 DEV／TEST debug=0。
- 2026-11-09 E 要求：之後通知 E 可上架前，在影子用 temp_list.lua 暫時加上架列，再跑 tools/lua/lua.exe scripts/gen_item_v3.lua --check 比對數值（v2_key、kv、source_differs），只做 TEMP 編譯不夠。

## A 線全面盤點（2026-11-09，live）

- 英雄技能：192 個英雄、868 個條目，產生器阻擋 0（target/lineA/hero_all.lua）。
- 道具：170 件中 155 件沒有阻擋。13 件只差上架（catalog:），E 正在照 gen_item_v3 --check 流程處理。
- summon 模板：只有 earth_wall 一項（blocks_pathing 缺 collision_radius，A 線補丁待 land）。
- 還有 B 層／產生器阻擋的只剩 2 件，B 依序在做：
  1. 土牆之書：item.active.cost.charges（B 做道具充能 host）→ A 接產生器，加上 wall_patch.lua（collision 80）並重生 v3_summons。
  2. 千里大劍：item.active.cast.interruptible、fn has_tag(u, courier)、target.types building+courier+creep+neutral+summon → B 做可打斷施放和信使 tag／目標類型，A 接產生器。
- 沒有新的 B 層缺口。
- 9ef90d98（2026-11-09）：道具充能接上（B 1f8fecee）。gen_item_v3 輸出 catalog charges，v3_rust 放行，石牆補 collision 80 並重生 v3_summons。土牆之書只差上架，已交 E；gen_item_v3 --check 已在影子用暫時上架列驗過。現在只剩千里大劍等 B（interruptible、courier）。
- 73478294（2026-11-09）：道具前搖接上（B 2be4ab76）。gen_item_v3 輸出 catalog cast，v3_rust 拿掉 cast gap，item_shop_v3 有 28 件帶前搖。千里大劍只剩 courier tag／非英雄＋信使目標，等 B。
- e5f6446c（2026-11-09）：千里大劍接上（B 88af54a8）。courier tag、道具非英雄目標，gen_item_v3 輸出 exclude_heroes 和 range 0。live 上 170 件道具已無產生器／B 層阻擋，11 件只差上架（shintogo_kunimitsu、ancient_doll、ghost_gloves、gold_to_prestige、hachiman_bell、ice_book、kat_steel、ksitigarbha、magic_ring、poisonous_ring、the_great_sword_of_teleport），已交 E。
- E 確認（2026-11-09）：千里大劍（原作名稱「陸奧守吉行．銘刀」，catalog 261）land 後，158 件原作道具全部上架。其餘 10 件（shintogo_kunimitsu、ancient_doll、ghost_gloves、gold_to_prestige、hachiman_bell、ice_book、kat_steel、ksitigarbha、magic_ring、poisonous_ring）原作沒有 KV，依 D13 不移植也不上架，item_all 的 catalog: 列可以忽略。A 線道具工作完成。
- 8c11d5ca（2026-11-09）：巨龍、扇子的 bot 改成作者指定 enemy_line。90 件 scripted 主動道具的 bot 策略主機都能用。道具施放轉向由 B 9ceb5d9e 修好。待辦：英雄技能 unit_or_point 照實發布，等 B 在 host 加 TargetType UnitOrPoint。受影響 6 個：B08W_old、B04W_old、A06D、C10W、A14D_old、A32T。
- b8e4f3c0（2026-11-09）：v3_bot 新 intent ally_anchor_unit，千里大劍 bot 改用它（host 端由 B 做）。待辦：英雄 unit_or_point 照實發布，等 B or18 之後。
- 5f232ab3（2026-11-09）：信使 summon courier（馬），SummonStats.courier／magic_immune，技能殼 courier_deliver／courier_return。前三次 land 失敗：漏了 v3_test_heroes 重生、空 on_cast 的 unused cx（deny warnings）、items 註冊測試的已知技能清單。教訓：送 land 前在影子跑三個 generator --check、scripts workspace cargo check、items_impl／英雄 DLL 載入測試。
- a1b7a6a0（2026-11-09）：英雄技能照實發布 unit_or_point，7 個技能。待辦：B or22 land 後補 unit_target／targets_heroes_only（hero_author TAKES_UNIT，patch 在 target/lineA/uop_patch.lua）。
