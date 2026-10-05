# Bot 通用購裝目標（2026-10-05）

## 計畫與決策

1. 在既有正式商店、owner Gold／Inventory 與 Lua role 配方上新增可選 item_builds；不新增英雄專屬 Rust／C++／Blueprint。省略時維持原本行為。
2. 每個角色宣告有序「最終背包目標」，不是購買游標。最多五個角色、各一至六件、角色不重複、ID 必須在 compiled MOBA catalog；可重複裝備。嚴格反序列化仍拒絕未知欄位。
3. 每次從 owner 背包保留所有已完成目標，再依第一個未完成目標遞迴推導缺少材料，重複 recipe 需要不同件數。沒有生命期游標，因此不因合成消耗材料而重新買已完成大件，重生後依持久背包重新推導。
4. 使用 BTreeMap／宣告順序確保決定性；遞迴訪問集合拒絕循環，深度受 catalog 節點數限制。不讀敵方金錢／背包、隱藏敵人或私有商店狀態。
5. preview_buy_item 複製六格背包與 Gold 後直接呼叫正式 buy_item kernel，不自行重算價格、材料、六格與合成規則。此預覽不等於 authority admission；正式 dispatcher 仍核對 phase／pause、owner、活體、己方基地／距離。
6. 商店半徑 300 提取為共用常數。Bot 只在公開己方 home 範圍內、存活、未 Recall、committed disclosure 新鮮且含自身時，一次送一個正式 ItemBuy。購裝在基地低血等待前執行，不阻止恢復；不足金錢／背包阻塞不賣裝、不跳到較便宜後續目標。
7. 正式 plan compile 在建 World 前，以相同 planner／kernel 模擬空背包購裝，拒絕六格無法完成的優先順序。模擬限 4096 次，金錢使用最大合法餘額、不進遊戲世界；此項只在 preflight 執行，不增加每 think 的模擬熱路徑。
8. Lua 各角色共用既有被動 catalog：Carry／Jungle 優先物理大件，Top 先護甲，Mid／Support 先鞋或護甲。現無法術強度物品，不把 atk 當法術強度；這是可替換策略資料，不是最佳化平衡配裝。
9. 接著補每個 build 可選 return_to_shop={min_gold=950,threat_radius=1000}。省略不主動購物回城；金錢門檻須在 1..i32::MAX、距離在 1..10000。離開 shop、有可購買的未完成目標、餘額達門檻且附近沒有披露的活體敵方／中立單位，才送正式 Recall。低血續航撤退先處理，現有回城讀條不打斷；已完成目標不因累積金錢而再次回城。不把「無已披露威脅」當成未知敵人無法攻擊的保證。
10. 回城與購物共用 disclosed_threat 函式，不重寫敵人可見性或讀取隱藏 world。三原型與單真人 Lua recipe 繼承各角色 policy，完全由設定決定，不硬編碼英雄。

## 當前功能確認

- core 指定 role_bot_items 四項通過：重複劍材料、950 全價的大劍只補 250、完成目標不再買材料；保留既有終局目標、滿格可合成、低／負餘額、容量與循環拒絕、非法 ID／數量／角色、無法組裝順序預檢；serde 舊配方省略與 compile 複製新策略。另以測試 registry 的二階合成確認遞迴，不將測試 ID 加入 production catalog。新增 opt-in 經濟回城的金錢／範圍／披露敵方與中立／死體與友軍邊界。
- base_content 指定 role_bot_items 兩項通過：正式 60Hz driver／PlayerInput／商店結算，1250 依序買兩劍、大劍與鞋後為 0，背包只留兩件；planner 本身不改金錢或背包、對手資產不變；低血基地等待前購買、完成後停止、範圍外／pause／死亡不買。另一项從線外正式 Recall→完整讀條（不送中斷 input）→權威 home 傳送→兩劍與大劍結算 950→0；完成後線外持有 9999 仍不再次 Recall。
- 固定 Lua 混合單真人配方 prepare-only 成功：一真人九 Bot、60Hz，正式 Rust moba-config 接受全部新 item_builds。基地購裝版本輸出 target/role-ue-runs/1791138841-1/session/game.toml；加入 return_to_shop 的最終版本為 target/role-ue-runs/1791139211-1/session/game.toml。此為設定預檢，不是網路／Unreal 對局。
- OpenSpec strict 與 root／omb whitespace 檢查通過。未提交、推送、清理或修改無關使用者檔案。
- 不重跑全套驗收，不建 OmGame、不 stage 新 DLL、不動既有 Editor 或遊戲程序。

## 未完成與邊界

可調整裝備／出售策略、最佳化補刀與隊友保護、魔力與進階技能、100 場 headless、LAN、完整 Unreal 對局與最後效能驗收仍未完成。外來或真人變更背包造成容量不足時保守不購買，不偷偷出售；未知威脅仍可能中斷經濟回城，本批未加入最佳化回城路線。OpenSpec 維持 20/30，5.5 不勾選。錯誤與防再犯見 E167。
