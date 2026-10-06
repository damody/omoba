# 主動物品 Unreal 通用資料與冷卻 HUD（2026-10-06）

## 計畫與決定

1. 將 moba_items 作者型別、五種 active 閉集、數值／配方驗證移入 `omoba-content-model/src/moba_items.rs`；Rust 與 Unreal 生成入口共用，不複製另一份 C++ 作者 schema。
2. 生成 `OmItemCatalog.h/.cpp` 的通用 FindItemMetadata，包含穩定 catalog ID、物品 ID、主動種類、效果量／時間與總冷卻；未知 ID 清空輸出且失敗。數值沿同一 1/1024 量化，不建立遊戲期 Lua。
3. 靜態 metadata 由生成 C++ 提供；六格剩餘冷卻只讀既有 owner-local 安全快照，不新增 ABI 或重複 simulation。
4. HUD 區分 passive／ready／remaining-total／unavailable，商店 tooltip 顯示生成效果與總冷卻。是否可用不依商店距離；需當前自己的存活英雄、進行中對局與零權威冷卻。metadata 未知、非法冷卻不標 ready。
5. 完整快照缺自己 owner 時發布六格清空，control-only 不改既有物品 baseline；StopRuntime 也清空經濟與六格事件，不保留上一場冷卻。

## 實作與局部確認

- UE 的 FOmShopCatalogItemPayload／FOmItemHotbarStatePayload／FOmEconomyHudStatePayload 加通用 metadata／六格型別化狀態；原生 widget 與既有 ItemHotbarStateChanged 不需新增 Blueprint graph。
- `cargo test --manifest-path omfue/codegen/Cargo.toml item_metadata -- --nocapture`：2/2 通過。五種效果共享驗證、量化／未知欄位與非法冷卻拒絕；真正 Lua fixture 生成 C++，manifest 追蹤新檔且 check 通過。
- `cargo test --manifest-path omoba-template-ids/Cargo.toml --features runtime-lua-content --lib moba_item -- --nocapture`：2/2 通過，確認作者模型移動後原契約仍成立；此 feature 僅作者測試，不加入正式遊戲。
- 正式 Lua 內容生成17檔／17輸入，`--check` 通過；既有四個被動商品與平衡未變。identity=`ff3ef5e2957aa89f`、data=`26f34a124129cc46` 保持；presentation hash 因納入物品 metadata 變為 `e8bdc0929625fdd2`。
- 相關 Unreal 模組指令：`Build.bat OmGameEditor Win64 Development -Project=D:\code\omoba\omfue\om.uproject -Module=OmRuntime+OmGenerated+OmEditor -WaitMutex -NoHotReloadFromIDE -NoEngineChanges`。首次 unity 撞名失敗，修正後7 actions／11.23秒，Result: Succeeded。
- 日誌：`omfue/Saved/Logs/native-item-metadata-modules-20261006.log`。NativeShopInput 增加 readiness／主被動／remaining-total／NaN／owner 不可用斷言，只編譯未執行 Editor automation；未驗證真實畫面或 PIE。

## 問題與修正

- 新 parse 未註記 Vec 型別，被 slice 驗證 API 推導為 unsized slice，7個 E0277／E0308；明確 `Vec<MobaItemEntry>` 修正。
- 既有 OmAbilityCueStyle／OmProjectileCueStyle 在匿名 namespace 都叫 CompiledStyles，unity 合併產生 C2371。改為 CompiledAbilityStyles／CompiledProjectileStyles，不關閉 unity、不改 BuildId／引擎設定。修正後編譯通過；未另外強制 full unity／release。
- 調查猜測不存在的 omfue/bridge/include/om_bridge.h；實際來源在 ThirdParty/OmBridge/include，先 rg --files 再讀取。截斷輸出不當完整審閱。記錄 E263。

## 尚未完成與邊界

- 護盾餘量沒有安全 owner 投影；不從任意 Buff payload 或敵方狀態讀取、不用生成的初始量假裝當前盾量。此項待獨立安全投影後接 HUD。
- 正式商品仍四個被動品；新主動商品、原生使用按鍵／按鈕與權威結果閉環待下一段，不能以 metadata 代替完整可玩。
- ABI14／wire5／IPC4 保持；bridge／後端 DLL 未完整重建或部署，新 presentation hash 需最後統一建置／stage，不能拿舊 binary 冒稱可直接玩。
- 沒有新增角色專屬 C++／Blueprint graph、維護 omfx、runtime Lua、提交／推送或完整驗收。OpenSpec仍21/31。
