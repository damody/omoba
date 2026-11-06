# B21 馬場信房（舊版）推定紀錄

原作 `resource/addon_tchinese.txt` 沒有任何 `B21*_old` 的說明，四招說明都依 KV 與腳本還原。中文招式名取 KV 註解（`npc/B_Unified/B21.txt:399`「波動三連斬」、`:468`「殺意」、`:574`「領軍如神」、`:654`「不死鬼美濃」）。英文名：殺意、不死鬼美濃沿用本體；另外兩招自訂（Triple Wave Slash、Divine Command）。

- **b21_old_triple_wave（B21W_old 波動三連斬）**：說明依 `npc/B_Unified/B21.txt:399-467` 與 `vscripts/heroes/B_Unified/B21.lua:212-279`。
  - 四個 Timer（0、0.4、0.8、1.2 秒）各自檢查 `IsChanneling`，引導 1 秒，所以實際三道；說明顯示的道數由 `waves` 從引導時間與間隔算出。
  - 每道都從施放當下的位置與方向發射（投射物表只建立一次）。
- **b21_old_killing_intent（B21E_old 殺意）**：說明依 `npc/B_Unified/B21.txt:468-573` 與 `B21.lua:281-305`。
  - `ApplyDataDrivenModifier(..., "modifier_stunned")` 套引擎內建名稱，照專案慣例推定有效（原作大量使用）。
  - `BASEDAMAGEOUTGOING_PERCENTAGE` 寫成 StatKey `base_attack_damage_pct`。
- **b21_old_divine_command（B21R_old 領軍如神）**：說明依 `npc/B_Unified/B21.txt:574-653`。
  - 光環半徑取 `Aura_Radius` 800，`AoERadius` 900 只是指示器。
  - 光環殘留 0.5 秒是 Dota datadriven 預設（推定）。
- **b21_old_undying_demon（B21T_old 不死鬼美濃）**：說明依 `npc/B_Unified/B21.txt:654-790` 與 `B21.lua:319-399`。
  - 「每秒最多損失 hp_threshold2」是由 `caster.nexthp` 的 SetHealth 拉回機制推得（`B21.lua:324,338-341,376`），原作沒有文字說明；說明照實際效果描述。
  - `caster.nexthp` 寫成 `scope = "hero"` 的 `unit_state b21_old_next_hp`，初值與解除值以 -1 代替原作的 nil／-10。
  - OnHealthGained 推定與 v3 `on_heal_received` 相同（同本體 B21 的推定）。
  - periodic 內一秒後的清除寫成 `delay`，推定狀態移除後仍會執行（原作 Timer 與 modifier 無關）。
  - KV 沒寫 RequiredLevel，照引擎大招預設 6/12/18。
