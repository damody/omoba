# 通用 native 英雄呈現：實作與驗收

## 計畫

1. 以生成器提供全部英雄的美術引用與動畫配置，通用 actor 處理呈現。
2. 以 Editor automation 驗證模型、材質、動畫選擇、重複 frame、不重播相同 action、fallback 與既有 Blueprint 相容。
3. 以 PIE 真實 frame／日誌／截圖檢查呈現，再以資產 SHA-256 驗證配方重跑。
4. 每次錯誤立即寫入 `unreal-moba-error-register.md`，修正後重新驗證，不只重跑失敗指令。

## 已實作

- AOmHeroActor 提供 NativeHeroMesh，於 BeginPlay 載入 soft asset；生成 CDO 只持有引用、不載入美術。
- 生成的所有英雄 constructor 填入模型、材質、來源動畫及 Lua clip 的開始／結束秒數、loop。缺少模型的英雄維持 fallback。
- 動畫採通用 idle／move／attack／critical 槽；同一 action instance 不重播，新 instance 重播。播放只改變呈現，不執行傷害或權威規則。
- 畫面血條 bounds 使用實際模型；插值保留美術初始 offset，不再抹掉 Lua 設定的高度。
- 已有 skeletal mesh 的 Blueprint 保留原呈現路徑，不建立第二套可見 mesh；Saika 相容事件保留，2.2b 尚未完成。
- 生成器 version 3／class surface signature 帶模板版本，避免把新增 constructor 的表面變更當作舊版本。Lua canonical hash 不變。
- native soft object path 使用完整 `Package.Asset`；MCP 配方使用 package path，兩者不可混用。
- 資產配方增加 SkeletalMesh usage 旗標，讀回／graph／引用皆需通過。

## Lua 美術設定

既有 render 的 scale 以 meter→centimeter 邊界換算；未指定 scale（共用 schema 的 0）在 native 層採 UE identity。UE FBX importer 已轉 Z-up，native 預設 pitch／roll 為 0，不重套 Fyrox 原始來源軸向。需要不同姿態時可直接配置：

```lua
ue = {
  native_only = true,
  native_visual = {
    scale = 1.2,
    pitch_deg = 0,
    yaw_deg = -90,
    roll_deg = 0,
    z_offset_cm = 0,
  },
}
```

欄位皆可省略；scale 必須正且所有數值有限。此設定不修改角色專屬 C++／Blueprint graph。

## 已確認的驗收

- `cargo test --manifest-path omfue/codegen/Cargo.toml`：22/22 通過，涵蓋 full object path、來源／clip、預設 UE 軸向及 Lua override。
- 軸向、rest offset、soft path 修正後 full build 通過；4/4 Editor automation 通過，零 error／warning，既有 BlueprintSurface／AnimationStateSmoke／SaikaEventDispatch 保持通過。
- fresh PIE native transform 與 fallback hero spawn 斷言通過，NativeHeroMesh.WasRecentlyRendered=True，截圖可見模型與材質。工具依實際玩家視點與旋轉配置測試位置，不再使用原點假設。
- 資產配方復驗 10 jobs、22 套件 SHA-256 不變；SkeletalMesh usage 讀回通過，PIE 無對應 missing usage 警告。
- Lua planner／module tests、OpenSpec strict validation 通過。
- 同一 Editor session 重跑發現 BlueprintSurface 暫存名撞名崩潰；已改 unique name，完整重建後連跑兩次皆 4/4 通過。驗收工具固定兩輪，防止只驗 fresh session。

## 可重現指令

```text
tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8
tools\lua\lua.exe scripts\ue_native_visual_smoke.lua
tools\lua\lua.exe scripts\ue_pie_smoke.lua
tools\lua\lua.exe scripts\tests\ue_asset_recipe_editor_test.lua
```

NativeVisual 報告位於 `omfue/Saved/McpAutomation/NativeVisual/report.json`，PIE 報告位於 `omfue/Saved/McpAutomation/pie-smoke-report.json`。工具會檢查 modal／PIE／測試 discovery／完整 verdict，不採用僅 started:true 的假成功。

## 尚未完成

整體仍為 14/30：此輪先完成 6.1 的 native 動畫基礎，但技能 cue、完整視野與地圖尚未驗收，故不勾選整項。雙隊端到端、MOBA 規則、完整 UI、美術替換實測、LAN 與效能仍須接續。
