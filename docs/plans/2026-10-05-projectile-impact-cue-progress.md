# 通用投射物命中 cue 增量（2026-10-05）

## 計畫與決定

1. 真正命中以權威 `Outcome::ProjectileHit` 為來源；投射物移除、死亡或逾時不是命中。
2. 新增獨立 `ProjectileImpact` fact，僅攜帶受擊者 canonical identity。經投影只留下 visible 且 Disclosed 的 replica subject 與固定 `HIT1` 標記，不公開攻擊者、種類或座標。
3. 共用 `PresentationCue` 新增精確 28-byte `IMP1`：tick、target replica、disclosure epoch；復用既有 watch 保留、送出 ACK、Hide／Forget／reset／reconnect 與 bridge lease 管理，不新建一套傳輸。
4. C ABI 13 結構不變：FxImpact cue_id=2 與傷害的 cue_id=1 分開。native 共用去重，呼叫 `OnProjectileImpactCue` 的受擊者位置 fallback；不虛構碰撞點或彈道、不需要英雄專屬 C++ 或 Blueprint graph。

## 範圍與限制

- 零傷害接觸仍是命中，傷害結算仍由原管線處理；不能把 impact 當作扣血通知。
- fallback 錨定目前公開 actor，不宣稱是精確碰撞座標。已隱藏、忘記、移除或同 tick 致死後不再 live 的受擊者不能補推位置，cue 依共用 live dependency 退役。
- 後續若要世界座標命中特效，必須新增權威結果／安全披露契約，不能從輸入或最新位置猜測。
- 本批不 stage DLL、不改 engine／project BuildId、不啟動完整雙 UE／60Hz 驗收、不維護 omfx。E224 的 native automation 啟動基線仍未修復。

## 局部確認

- core 嚴格 codec／可見性 2/2 與來源 outcome 1/1 通過；補強來源 fact buffer 的實際 emit／tick／target 接線後，該測試再確認 1/1 通過。
- client 共用 cue 局部 7/7 通過，實際 IPC 測試涵蓋 IMP1、watch overwrite、ACK 與 reconnect；補強 public event 到 snapshot、隱藏 target 排除後，該測試再確認 1/1 通過。
- bridge target anchor／獨立 cue_id 1/1 通過。
- native OmRuntime／OmGenerated／OmEditor 14 actions 編譯成功：`omfue/Saved/Logs/projectile-impact-modules-20261005.log`。未執行 native automation。
- OpenSpec 全項仍 21/31，不以增量取代完整 6.1 驗收。
- OpenSpec strict 與主 repo／omfue whitespace 檢查通過。合計 11 個不同局部測試通過；兩個補強測試的重跑不重複計數。

## 防錯

見 `unreal-moba-error-register.md` E226。
