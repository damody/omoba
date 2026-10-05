# Lua 生成通用投射物 fallback 樣式

## 計畫與決定

1. 6.1 實際缺口：OmProjectileVisual 與 OmUnitActor 各自依投射物 ID 選色／判斷爆炸，新增投射物仍需要 C++ 特例，兩份邏輯也有半徑差異。
2. 新可選 ue.projectile_cue 宣告 color_rgb（三個 u8）、trail_thickness_cm（有限正數，最多 100）、impact_radius_cm（有限正數，最多 10000）。完整欄位與未知欄位嚴格驗證；非法內容在寫生成檔前拒絕。沒有宣告的投射物不進 registry，以共用預設處理。
3. codegen 生成 OmProjectileStyles.h/cpp；OmGenerated 模組啟動安裝完整原生 registry、卸載清空。OmRuntime 不反向依賴 OmGenerated、不解析 Lua／JSON、不建立玩法 World。lookup 回傳值、不暴露容器指標，限制在 game thread。
4. Actor 與 UObject 的投射物 cue，以及塔的投射物顏色共用同一 registry。原 Blueprint 事件與 payload 不變；未調整 cue 去重、ACK、視野或輸入規則。
5. 既有八種有特殊外觀的投射物改成 Lua 宣告，顏色沿用原值。爆炸半徑統一為 120 cm、線寬統一用宣告（原 Actor bomb 110 cm、bomb_frag 38 cm／固定線寬 6 cm 與 UObject 不一致）；這是純 fallback 外觀收斂，不修改傷害／碰撞／作用範圍。
6. 未知／未宣告／卸載後共用暖黃色、5 cm 線寬、38 cm 半徑；不依角色或名稱猜顏色。仍是既有 DrawDebug 呈現 fallback，不能冒充正式特效美術或 Shipping 特效品質。

## 本批確認

- 生成器指定測試 1/1 通過：任意 ID、完整宣告生成、未宣告 fallback、tombstone 排除、check 冪等，以及七種非法資料不覆寫既有生成檔。
- 正式來源生成 13 檔成功。完整 Lua catalog data hash 更新為 40641b573c2890e9；identity hash ff3ef5e2957aa89f 不變。既有 content_hash 91320001f1ba2429 不包含此新呈現設定，不能以它單獨判斷本批完整內容一致；正式 full catalog handshake 仍必要。
- restart build（不加 with-bridge）保留 -NoEngineChanges，15 個 project actions 全部編譯／連結成功，沒有還原或建置共享引擎來源。log：omfue/Saved/Logs/projectile-style-build-1791182806.log。
- UnrealEditor-Cmd -NullRHI 的 Om.Generated.ProjectileCueStyle 單項 1/1 Success、0 errors／warnings、程序 exit0；報告 omfue/Saved/McpAutomation/ProjectileStyle-1791182856/index.json。確認 compiled Lua 的 bomb RGB／9 cm 線寬／120 cm 半徑，任意 ID、registry 原子替換不留舊值、清空後 fallback；scope guard 最後重新安裝正式 catalog。不重跑完整 NativeVisual／雙 UE 長測，也沒有取樣特效像素。
- 正式生成 --check（13 檔）、OpenSpec strict validation、主 repo／omfue whitespace check 通過。

## 尚未完成

- 未重新 stage Rust bridge／script DLL，未聲稱目前部署可開完整對局；之後正式 build workflow 必須生成／建置並部署同一 full catalog。
- 2.2b 的 legacy ability fallback 與 typed 相容介面、6.1 完整技能／動畫呈現仍待，不因本批移除投射物 ID 分支而勾整項。
- 最後集中完整对局／LAN／效能驗收。僅維護 omfue，不碰 omfx，不 commit／push。
- 問題與防重犯規則記 E215。
