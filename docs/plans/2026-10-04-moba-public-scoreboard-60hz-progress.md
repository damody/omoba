# 公開計分板與三玩家 60Hz 驗收

## 本輪計畫與決策

1. 權威端發布持久玩家名單、隊伍與 K／D／A，不依可見角色推算。
2. 將公開資料穿過安全投影、runtime persistent IPC、C ABI 與 Unreal 共用原生面板。
3. 用真實三玩家 KCP 封包、非法資料回歸、Editor 自動化與完整建置驗證；保留完整框架尚未完成項目。

- 新增獨立公開 metrics，原 owner score、Gold、物品與 Recall audience 不放寬；不發布位置、entity 身分或助攻帳本。
- runtime 僅接受完整 2–10 人、兩隊各最多五人、唯一非零 player／team、四項 u32 值及一致 count，固定 team／player 排序。缺少或錯誤代表不支援，不製造零分。
- bridge 再核對配置 owner 所屬隊伍及私人 KDA；ABI 升至 9，拒絕 8，固定十列 frame-owned array。Unreal int64 支援完整 u32。
- 共用 Slate 計分板唯讀、無英雄 C++ 或 Blueprint graph。有效資料即使英雄死亡仍顯示；完整快照沒有有效 HUD、Stop 清空，control-only frame 保留。700px 面板允許換行，但不據此宣稱所有解析度已做畫面 QA。

## 已驗證

- `target/interactive-runs/moba-runtime-1791079034/moba-runtime-smoke-report.json`：真實 60Hz KCP 三 runtime，正式 Move／AttackTarget intent，沒有注入傷害；settlement tick 1021，player 1 助攻、player 3 擊殺、player 2 死亡。
- opt-in `real_three_player_owner_score_capture` 通過：player 1／2／3 分別 1399／1294／1262 筆 IPC 快照，公開計分板逐 tick 對照原始 authority frame；相同 tick 三玩家公開列一致。snapshot tick 明確減 1 對照 wire，不使用最近樣本容錯。
- report cleanup 與獨立 Lua process.inspect 核對 PID 88760／91084／2532／82760 全部退出。
- base_content 84、core 330、server 154、runtime 59 lib＋3 main、bridge 51 lib＋2 integration 通過。runtime 5／bridge 2 opt-in 預設忽略，實際 capture 測試另行執行。Fyrox workspace `cargo check --tests` 通過。
- Unreal 完整編譯成功；Blueprint validation `compile-1791079407/report.json` 11 項 gate 通過；同 Editor `NativeVisual/report.json` 兩輪各 15／15 通過，包含 `Om.Runtime.NativeScoreboard` 真正 Slate visibility、十人及完整 u32、非法值與 Stop 清除。
- 串行 `pie-smoke-report.json` 通過；它驗證既有原生英雄／資產與 PIE，不是三玩家計分板實戰像素證據。
- 最後 `build_ue_moba.lua --build-only` 通過，bridge 產物與 staged DLL SHA-256 一致；codegen `--check` 確認 11 檔／15 Lua 輸入，content hash `de9c7fcfc98d6479`。OpenSpec strict validation 與本輪 scoped diff whitespace 檢查通過。
- 兩次 owned Editor PID 98052、23080 都經 MCP 保存 dirty assets、核對 exe 後正常關窗退出；沒有強制關閉使用者其他 Editor。

## 尚未完成

- 真實多人 Unreal 視窗的計分板畫面、十人完整對局與各解析度像素驗收。
- 5.3 完整玩法、6.2 選角到結算整體 UI 不因本次部分功能而勾選；OpenSpec 維持 19／30。
- 三路／野區、五位置 Bot、LAN 及效能門檻仍須原本後續驗收。

查找路徑與首次 patch context 命中錯誤、完整快照殘留風險均記錄於 `unreal-moba-error-register.md` E113。未提交、推送或清理既有變更。
