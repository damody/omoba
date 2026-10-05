# 通用宣告式定身／沉默（2026-10-05）

## 計畫與實作決定

1. 新增共用 `ControlEffectKind` 閉合集合與 Lua `control_enemy` 效果，只接受 `stun`／`root`／`silence`。不開放任意作者字串成為權威 Buff；原 `stun_enemy` 宣告保留，執行時降低至相同 ControlEnemy 計畫。
2. 共用 schema 與固定 Lua registry 生成器驗證每級 duration_key：完整 ranks、有限 1/1024–60 秒、enemy unit、正值有界射程、敵我效果不衝突。Rust 全 plan 預檢後才提交傷害與控制，不讓後段無效控制造成前段傷害部分生效。
3. 唯一通用 sink 依 typed enum 呼叫既有標準 BuffId。定身只禁移動，沉默只禁技能；不把這兩者当減速／暈眩，不凍結普攻時鐘。持續時間沿標準 BuffStore 最大剩餘 TTL，不新增多來源驅散契約。
4. Lua 先鋒磐岩重擊附加定身、遊俠逐風終箭附加沉默，rank1–4 分別 1／1.25／1.5／1.75 秒。既有技能 ID、原始伤害／射程／成本／冷卻不變，沒有英雄專用 Rust handler、C++ 或 Blueprint graph。
5. 追加 root／silence Buff catalog 身分与顯示名稱，以 native_only 生成 UOmBuffRootVisual／UOmBuffSilenceVisual，公開視覺沿既有註冊 Buff 管線且無任意 payload。首次生成檢查發現預設 Blueprint 路徑，修正 Lua 明確 native_only 後重新生成與增量編譯，不建立無必要的 Blueprint 資產。

作者範例（僅建置期）：

```lua
extras = { duration = {1, 1.25, 1.5, 1.75} },
effects = {{kind = 'control_enemy', control = 'root', duration_key = 'duration'}},
```

技能還須提供合法 instant active／ultimate、unit target 與完整每級 range。正式執行仍為編譯 Rust 與生成 Unreal C++，沒有 runtime Lua。

## 局部確認

- 新 Lua control 邊界與非法種類案例 50/50，原 stun 15/15、include／混合敵我 8/8 通過。
- Rust 共 7 項直接相關確認通過：新模型1、效果原子預檢1、正式60Hz定身／沉默1（兩個實際生成技能子案例）、原暈眩2、混合敵我1、三原型十二技能1。
- 正式60Hz確認原傷害精確結算、標準Buff公開身分、定身只阻移動且允許自療、沉默允許移動但拒絕自療／不啟冷卻、兩者攻擊時鐘繼續、正常TTL到期後移動／施法恢復。
- Unreal codegen／--check 成功，15檔／17輸入；限定 OmRuntime＋OmGenerated＋OmEditor 的 NoEngineChanges build 首次12actions／10.39秒，native_only修正後5actions／6.00秒成功。沒有執行 Editor／PIE／原生畫面斷言。
- 最終 generator5、identity hash ff3ef5e2957aa89f，data hash26f34a124129cc46、呈現hash ca42ac58715f2f69。需最後共同建置部署，不能把舊stage DLL當新內容。C ABI14／selective wire5／IPC4不變。

## 問題、防錯與剩餘

- 大段 skill／status／code 合併輸出再次截斷；後續縮小範圍，不以截斷輸出當完整審阅證據。native_only省略會產生BP路徑，已修正並核對最終manifest為空；記錄E249。
- 本輪沒有編譯／斷言失敗，既有td_rounds dead_code警告不擴大修復。沒有部署 DLL、改引擎BuildId、維護omfx或commit／push。
- 這是5.5英雄通用效果子功能，完整5.5／100場、UE完整對局、LAN與最後效能驗收仍未完成；全框架21/31保持，不用局部成功代替整體完成。
