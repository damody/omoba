# B36_old 直江兼續（舊版）假設

路徑相對 `game/dota_addons/nobu/scripts/`。W/E/R/T 的說明取自 `resource/addon_tchinese.txt:3206-3213`；B36D_old 沒有任何說明。

1. **槽位**：`heromap_skill["B36"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:315`），choose_11 依序 `AddAbility`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）。`heromap_autoskill["B36"]["11"] = "D"`（`addon_game_mode.lua:418`）會把 B36D_old 自動學一級，寫成 `hero.auto_learn`。D 照本體慣例放 D 槽。
2. **符咒攻擊（B36D_old）**：KV 沒有名稱與說明，名稱依效果推定為「符咒攻擊／Talisman Strike」。說明依 `npc/B_Unified/B36.txt:719-763` 與 `vscripts/heroes/B_Unified/B36.lua:325-328` 還原：+572 攻擊距離，並以 SetAttackCapability 改成遠程攻擊。產生器不支援 `passive.states`，所以改成學會時（`on_learn`）掛上永久隱藏狀態，帶 `ranged_attack` 與 `attack_range`。原作 modifier 寫了 IsPurgable 1 加 IsDebuff 1，可以被敵方驅散。照設計意圖修為不可驅散（quirk fixed，FC 待 D 編號）。
3. **符咒‧金縛（B36W_old）**：光環掛在 10 秒的 npc_dummy_unit 上（`b36w_old_seal_unit`）。光環給的 modifier_B36W_old2 寫了 Duration 4。推定光環給的 modifier 時間由光環管理，Duration 不生效，離開後殘留照 Dota 預設 0.5 秒。說明「持續 10 秒」指的是直江印本身。移速百分比與攻速點數讀同一個 `speed` 變數，所以拆成兩個數值。v3_rust 回報模板不是 content summon，這是預期內的，等 D 重產生召喚列。
4. **符咒‧護符障壁（B36E_old）**：每 0.2 秒隨機挑一名敵人。FIND_ANY_ORDER 加 RandomInt 寫成 `order = "random", limit = 1`。等級加成用命中當下的英雄等級。
5. **符咒．咒之符（B36R_old）**：照 A28W（羽柴秀吉本體）的變青蛙寫法。
6. **禁咒‧殺之陣（B36T_old）**：角度步長原作用 3.14 近似 π，資料寫 10°（差異小於 0.1°）。KV 沒寫 RequiredLevel，照大招預設。
