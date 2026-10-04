# 通用小地圖隊伍辨識

## 計畫與決策

- [x] 唯一隊伍來源為權威公開scoreboard roster，不以player ID、owner正負或生成順序猜team。
- [x] roster原子驗證2..10列、唯一非零player、兩隊各1..5人、合法team及本地owner存在；無效即不使用隊伍對應。
- [x] 保留configured player精確核對，共用顏色：自己青色、隊友綠色、敌方紅色、未知灰色。
- [x] roster不產生新的單位標記，只替已經披露的live標記取得隊伍。
- [x] remembered型別仍沒有team／owner，不猜記憶隊伍；control保留，full reset清空。
- [x] 以十人、非連續ID、player7/team2、high-bit owner與非法roster fixture加入MinimapTeams測試。
- [x] 建置與此功能單輪確認；完整驗收留最後。

## 問題與限制

OmFrameEntity.owner實際是玩家ID，先前Slate以owner>0把其他玩家全部畫紅，無法正確呈現同隊玩家。改為與安全公開roster映射；缺少或不合法roster時未知，不把非自己當作敵人。

ABI owner仍是i32，但Rust既有投影保存u32位元模式；查roster時轉回u32並測high-bit ID。0／-1保留既有未知／無owner語意；u32::MAX與-1哨兵的既有ABI歧義未在這批改ABI，不猜該值。未知NPC顯示灰色，尚無獨立NPC隊伍欄位，不能依位置或player ID推導其隊伍。

所有變更在共用native adapter／Slate，無每英雄或地圖C++／Blueprint graph。沒有改Rust玩法、KCP或視野資料範圍。

## 當前功能確認

- `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` exit0、OmGameEditor Succeeded。ABI10及bridge未變，stage SHA仍為 `554332de168326f01ac36999ae7fa8af1fa40ef57a35f549515b5f76c1903f40`。
- `ue_native_visual_smoke.lua --test Om.Runtime.MinimapTeams --runs 1 --out-dir omfue/Saved/McpAutomation/MinimapTeams` exit0／success=true，1 passed／0 failed／error_count0。
- 測試確認十人roster不新增hidden位置、非連續ID／player7/team2、自己／隊友／敵方／未知顏色、configured player錯配、high-bit owner、重複／oversized／非法team／缺owner／legacy fallback、control與reset。
- 本批diff check通過。沒有執行其他全套automation／PIE／双UE／60Hz長測；完整UI與框架仍未完成。
