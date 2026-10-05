# 魔力上限 Buff 增量（2026-10-05）

## 計畫與決策

1. 共用 UnitStats.checked_mana_capacity 以英雄原始 MOBA Lua容量＋ManaBonus＋ExtraManaBonus求有效上限，沿上一輪checked_sum_add與explicit family規則，避免額外角色欄位或C++。
2. 有效容量最低0，最高沿既有作者envelope 1,000,000；非法payload／加總溢位／超過上限時拒絕Buff結果，host退回英雄原始容量，不給極端值自動截成百萬池。
3. 使用既有ManaPool.set_maximum：增加上限保留絕對目前魔力、不補滿；縮小時截限，目前值達上限時清除餘數。等級成長與Buff都重算原始base，不累加先前effective maximum。
4. 在共用run_script_dispatch開始、快取與施法准入前同步容量，包含已由正常Buff系統到期的加成；正式finish在效果Outcome／升級等結算後再次用同一helper同步，再回魔與發布CommittedMana。
5. 同步只處理opt-in mana_enabled、Playing、正active delta、未pause的match slots與存活非lethal英雄，不為None建立池、不影響TD或未啟用MOBA。
6. 同一hook新加Buff仍為deferred outcome，不能立即影響該hook的restore上限；在本tick finish生效、下次dispatch能讀到。這與既有Buff outcome可見性一致，不額外製造第二種即時Buff模型。
7. 新生命仍沿原始Lua容量滿池，舊生命的臨時Buff不跨死亡繼承；這輪不改出生規則、最大容量的智力舊公式或新版協定。

## 作者介面

既有Rust add_stat_buff payload可使用：

```json
{"mana_bonus":102400,"extra_mana_bonus":20480}
```

整數為Q10 raw，合計＋120容量。欲互斥同組來源可提供__aggregation_family，按每個stat最強者計算；沒有family的不同來源加總。只增加容量不是恢復魔力，要恢復另用既有restore_mana或Lua restore_mana_self。Lua持續Buff宣告仍待，不把這份payload當成已完成生成器支援。

## 當前功能確認

- core `--lib mana_capacity`：新1 passed。兩種容量加成、family強弱／移除、负值歸零、極端整數／非法型別／超envelope拒絕。
- base_content `--lib mana_capacity`：正式Production60Hz新1 passed。原容量280、剩餘200加120→仍200/max400；pause不改；等級2在新base上加120不補滿。短Buff到期後於施法前截回base，再扣正式metadata45，而非先消費已到期的超額魔力。大負值→0/0/0；invalid→0/base/0、不補滿。
- base_content `--lib mana_lifecycle`：既有3 passed，包括出生／active再生／pause／growth／Finished、warmup／opt-in／死亡新身分復活滿池、短60Hz安全投影hash一致且無repair。
- 共5個不同測試passed，只確認當前容量功能與直接受影響生命週期。没有full suite、真實network、DLL stage、UE build／MCP／PIE或100場。

## 問題與防錯

- 只有finish同步不够：到期Buff的舊上限會在該tick仍允許腳本使用超額余额，然後才縮池。新增共用派發前同步，正式測試的base－45證明順序，不能只檢查tick結尾current<=max。
- 搜尋native/mod.rs失敗，實際re-export在native.rs；先以rg找到pub use，不在錯誤路徑假定API不可用。
- 插入helper時原finish的commit hook comment暫時附到refresh function；diff檢查後移回finish。避免文件指出錯誤生命週期。
- 這輪沒有編譯或測試失敗；問題／決策登錄E184，不用成功測試掩蓋未完成作者介面。

## 下一步

Lua持續Buff宣告與duration／refresh契約尚待；UI、LAN、重連、三原型100場及效能仍未完整驗收。OpenSpec保持20/30，5.5／6.2整項不勾選。E177共享引擎baseline阻礙未解除，最後再完整驗收。
