# Lua 生成通用技能 cue fallback

## 本批計畫與決定

1. 實際缺口：OmUnitActor::OnAbilityCue 直接比較四個技能 ID，日後同類技能也得修改 C++。改 Lua ue.ability_cue→codegen→原生 registry→通用形狀繪製，Actor 只轉送 payload 與自身 transform。
2. 支援 sphere／toggle／formation／cone／fan 五種通用形狀。可選 RGB／停用 RGB、半徑、持續時間、射線長度、間距、數量、半角與 payload_distance_scale；全部有共用預設，未宣告／未知 ID 為 cyan sphere。不推斷技能執行語義。
3. 尺寸宣告為 cm；事件既有距離由明確 payload_distance_scale 換算，預設1，原四個 legacy 技能宣告100。不再用 AoeRadius<50 猜單位。angle 宣告為 degree，事件 ArcHalfAngleRadians 轉為 degree，DrawDebugCone 邊界才轉回 radians。
4. 色值必須三個 u8、未知欄位拒絕；半徑與射線長度≤10000、持續時間≤10、間距≤1000、半角≤89、倍率≤100，均為有限正數；count1..12、fan最多8。事件 count 截限、非有限／非正距離用已驗證預設、有限距離截到10000，沒有把非法資料帶入繪製。
5. 模組啟動安裝 compiled registry／卸載清空，game-thread lookup 回傳值。Runtime 不反向依賴 Generated、不讀 Lua／JSON、不建立第二份 World。
6. 保留 Blueprint OnAbilityCue 介面與 payload／C ABI；只修改 fallback 外觀，不調整效果傷害、碰撞、作用範圍、視野、cue 去重、ACK 或權威規則。formation與fan副圖元統一用宣告主色、球形細分／線寬由共用模板處理；不是像素完全不變的相容宣告。

## 作者入口

```lua
ue = { ability_cue = {
  shape = "formation",
  color_rgb = {12, 34, 56},
  radius_cm = 420,
  count = 5,
  payload_distance_scale = 1,
} }
```

不需要角色専屬 C++ 或 Blueprint graph。這是既有 DrawDebug fallback 的通用化，不是正式美術特效、Shipping 可視品質或所有技能 cue 的權威發布。

## 當前功能確認

- 生成器指定1/1成功：任意技能 ID生成、沒有作者宣告的技能不註冊、check冪等、12種非法欄位／數值拒絕且不覆寫既有生成檔。
- 正式來源生成15檔成功，完整catalog data hash23c4614509fdbf2d；identity ff3ef5e2957aa89f與既有presentation hash91320001f1ba2429不變。必須以full catalog handshake判斷完整內容，不能只比舊presentation hash。
- 原生增量build保留-NoEngineChanges，12個project actions編譯／連結成功；log：omfue/Saved/Logs/ability-style-build-1791183304.log。沒有修改共享引擎來源。
- UnrealEditor-Cmd -NullRHI單項Om.Generated.AbilityCueStyle為1/1 Success、0 errors／warnings、exit0，報告：omfue/Saved/McpAutomation/AbilityStyle-1791183357/index.json。確認生成toggle資料與啟停色、任意形狀ID、明確倍率／count截限／formation override、非有限值fallback、角度轉換與registry清空；scope guard最後恢復正式catalog。
- 這不是特效像素／實際技能發布／完整对局確認；沒有重新stage Rust DLL或跑雙UE长測。之後正式build workflow需生成／建置／部署同一full catalog。

## 剩餘項目

- 2.2b仍有typed相容介面與其他歷史來源需盤點；Actor內留下的Saika比較只用於舊插值log，不是本批技能派發。不能聲稱全部角色特例移除。
- 6.1完整動畫／所有技能cue／正式美術與Shipping品質、最後整合對局／LAN／效能仍待。21/31保持，不把局部功能確認當整項完成。
- 只維護omfue，不修／建／測omfx，不commit／push。
- 問題與防錯規則記E216。
