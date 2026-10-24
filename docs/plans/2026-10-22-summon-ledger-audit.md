# 2026-10-22 共用召喚落點預約盤點

本輪接續全遊戲17系統盤點，優先可用同一台電腦驗證的權威召喚完整度。地圖、兵線、野區、防禦建築、技能提示、HUD既有正式接受繼續回歸；不新增網路對局。

## 根因與實作

原先spawn_summoned_unit只合併已存在的友軍體積與同一adapter的outcomes。不同event／on_tick各有adapter，同tick可向腳本承諾相同位置；正式commit再逐一移位，讓返回handle當時可讀到的位置與實際生成不同。

新增World範圍PlacementReservations，Arc／Mutex保護的BTreeMap以完整Entity含generation記錄team、位置與半徑。既有canonical serial重放負責分配，speculative平行hook不預約。每次spawn使用相同team所有未提交預約與既有友軍體積；不同team互不影響，不洩漏隱藏敵方佔位。全World4096與每invocation128上限。

finish將預約交給outcome；discard_cast／Drop撤銷自己的預約及保留handle，正式handle_summoned_spawn先退休對應預約，再沿既有owner／公開地形檢查生成。新cache清除已退休generation的預約；無World資源的精簡harness使用cache範圍fallback。StateInitializer所有正式World安裝共用資源。無ABI／protobuf／Lua內容格式變更。

## 接受

新增跨cache預約、取消後位置可重用、Drop撤銷、不同team隔離、退休generation清除與全域容量守門。既有8單位canonical平行重放測試新增兩兩不重疊，並真的settle outcomes驗證每個公開返回位置與實際Pos完全一致、ledger歸零。完整Rust與既有45項Unreal World集中回歸，來源／DLL staging與證據檢查另記。

## 邊界與後續

這次解決跨invocation／cache的落點承諾，任意中間outcome移動世界／刪除owner後的整組施法commit原子回滾仍需額外交易設計。退休來源callback需完整發射時上下文，不能只補script ID。持續引導、特殊技能幾何、完整combat replica、更多內容及美術保留。MVP33/44與11項外部ACK／LAN／DPI／效能條件不因此勾完。

最終結果以2026-10-22-summon-ledger-final-acceptance.md為準；錯誤與決策保存在summon-ledger-errors.md。
