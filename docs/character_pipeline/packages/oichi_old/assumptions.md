# C17_old 阿市（舊版）假設

四招都有原作繁中說明（`panorama/localization/addon_tchinese.txt:990-998`，UTF-16），數值以 KV 與腳本為準（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C17"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:333`），choose_11 逐一 `AddAbility("C17?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C17"]["11"] = ""`（`addon_game_mode.lua:436`）。沒有 D。
2. **靜滯陷阱（C17R_old）**：原作說明寫「造成 425 範圍敵人暈眩」，腳本 `C17.lua:392` 讀 `radius_stun` 500，採腳本。陷阱光環的 `DISABLE_HEALING`（`npc/C_Neutral/C17.txt:701`）因產生器不支援 `cannot_be_healed` 未寫；陷阱本來沒有回復，只影響友方治療技能對陷阱的效果。多人同時觸發推定只爆一次。兩個陷阱一秒內先後炸到同一單位時原作延後 0.1 秒結算（`C17.lua:424-441`），資料不延後。
3. **極光守護（C17T_old）**：KV `delay`（`C17.txt:828`）沒有被腳本讀取，延遲就是前搖 `AbilityCastPoint`，所以 `delay` 不列入。
