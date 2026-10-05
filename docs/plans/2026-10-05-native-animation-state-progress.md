# 通用原生動畫狀態接線（2026-10-05）

## 本輪計畫與完成項

1. 接通 persistent Buff overlay 到原生英雄動畫，不依賴歷史 toggle cue：已完成。
2. 用 Lua 生成 state-to-slot 綁定，新增角色不用寫對應 C++／Blueprint graph：已完成。
3. 修正這條動畫路徑的協商時脈與過期攻擊，局部確認：已完成；完整對局與画面驗收保留最後。

## 問題與決定

- Rust 已選出最高 priority、同順位最低 stable ID 的 overlay，generated bridge 也傳遞名稱，但 AOmHeroActor 原先只使用普通動作／idle變體，忽略 overlay／locomotion variant。新增通用 selector，以 ue.native_visual.state_slots 把任意名稱映射到已有 render.animation_sources／animations 片段；直接同名片段亦可使用。
- 建置階段限制最多64映射、非空 ASCII alphanumeric／underscore名稱、禁止 None、拒絕 FName大小寫重複來源與不存在目標，無 native model 不容許綁定。未知 native_visual 欄位也拒絕，不吞 typo。
- 攻擊／critical 優先於被動覆蓋；明確 locomotion 優先站姿；走路缺覆蓋只回普通 move，不拿站姿替代。移除 Buff 後回原 idle variant，缺 critical 可回 attack，缺普通片段可回 idle。沒有可用片段時保持原生 fallback，不製造玩法。
- 初始化快取可用 slot，沒有每 entity 每 frame 集合分配。缺失／錯 skeleton 的片段首次失敗後剔除，同一 frame 重新選擇；每次失敗至少刪一個 slot，不能無限迴圈，不重播 ability。
- Saika僅 Lua 設定 sniper_mode→sniper、sniper_walk→move。現有素材沒有獨立 sniper walk，不能用站姿片段冒充移動；未來新增片段與改Lua映射即可，selector沒有 Saika名稱。
- 原 bridge 動畫用tick/120、impact固定16.7ms，且已過 backswing 的 retained FX 仍維持 attack。改由 driver 的已協商 step_fps 交給唯一 LockstepTiming；未知／unsupported 無 action timing與idle週期，正常 locomotion／overlay仍可顯示。未到／過期 FX 不保持 action，windup依真正 impact時間，impact一個協商tick。

## 局部確認與限制

- 來源界線補充（E235）：60／90／120Hz相位測試使用合法AttackPhaseFx；formal IPC未提供完整權威攻擊時間，不能宣稱正式普攻consumer已有端到端來源。後續時間契約列入tasks，不由基礎asd猜測。

- codegen原生建構子測試1項通過，包含任意映射、無目標、None、非法／大小寫重複名稱與無 model拒絕。
- bridge animation_overlay_tests 4項通過；新時脈測試涵蓋60／90／120Hz相同100ms windup、impact duration、recovery、到期、未到與未知時脈，以及原 overlay priority／移除與未知內容fallback。
- 正式生成15檔／17個Lua輸入。最終資料 catalog_data_hash=2027e0ada2f76866；hero／ability identity=ff3ef5e2957aa89f、codegen content_hash=6460930eee2ba926不變。各雜湊 domain 不可互換。
- 最終生成 --check、OpenSpec strict與主repo／omfue whitespace檢查通過；不重跑全套驗收。
- native scoped OmRuntime＋OmGenerated＋OmEditor 首次13 actions／9.61秒成功；修正walk資料後最終3 actions／4.65秒成功。日誌：omfue/Saved/Logs/native-animation-state-modules-20261005.log、native-animation-state-final-modules-20261005.log。
- GenericAnimationOverlay原生測試補7個selector斷言；它們僅編譯，未執行。已知E224 BuildId mismatch不重試、不手改manifest。沒有 stage DLL、完整UE對局、MCP／PIE／像素驗收或omfx建置。
- OpenSpec仍21/31，6.1完整呈現仍未完成；這份紀錄不宣稱所有動畫美術或重連畫面驗收完成。
