# Mana 共用施法扣費進度（2026-10-05）

## 計畫與決策

1. [x] 共用 dispatch 依 serial SkillCast queue 順序維護本批英雄 Mana／CD ledger，不修改 immutable cache、不靠平行 Mutex 排程。
2. [x] 明確 mana_pool=Some 的英雄先保留扣費候選，handler 成功才提交 Mana outcome；失敗保留原帳本並丟棄本次 adapter 效果與施法 visual。
3. [x] 正式 60Hz PlayerInput 與同批內部事件確認成功／拒絕與延後狀態邊界。
4. [x] 記錄結果、修正計畫及 E174；全套驗收集中最後。

沿 OpenSpec 5.5，目前仍 20/30；不是完整 Mana 或完整原型對局封關。

## 共用實作

- `prepare_mana_cast` 只處理明確啟用法力池的 Hero；其餘模式維持原本施法語意。驗證 caster 存活／正 HP、學習 rank、非 passive、當級 metadata、CD 與合法有限成本。
- base cost 轉 Q10 round-nearest；使用既有 UnitStats mana_cost_mult。i128 計算修正成本，正小數成本向上取整，zero multiplier 明確免費；無法表示的成本拒絕，不讓 overflow 改成免費。
- ledger 按既有事件順序保存每名 caster 成功後的 Hero Mana／CD，generation 完整 Entity 作 key。同 tick 不同技能不能超支；同招成功後不可再因舊 cache CD=0 重施。
- handler `execute` 成功才提交 ScriptSetMana 與既有 cooldown outcome。失敗丟棄該 fresh adapter 全部 deferred outcomes、位置／面向／攻擊計時 overlay，以及本次施法 visual；不回滾其他已成功事件。
- 已捕捉的 unit hook panic 會標記本次 cast 失敗；execute 的既有 panic 捕捉也不標成功。這不是跨 ABI abort、程序 crash 或外部 I/O rollback 保證。
- managed caster 的 current_mana 在事件內讀本批施法前 ledger 餘額；其他 enabled Hero 讀真實 pool。未啟用的 legacy 模式仍保留原 max-mana 相容路徑。
- host 已擁有 metadata 扣費，enabled Hero 的 explicit script spend_mana 暫時明確拒絕，不能沿舊 stub 虛報成功或另扣一遍。任意腳本資源變動／restore_mana 尚未接線，不宣稱該 API 已完整。

不寫 hero ID 特例，不加角色 C++／Blueprint，不修改 GameWorld ABI。

## 直接相關确认

- base_content `mana_cast` 2/2：正常 60Hz PlayerInput 敵對拒絕不扣／不啟 CD、成功兩招各扣45、零魔治療拒絕；同 tick shot→finisher→shot，以45與135兩組餘額驗超支和重複 CD gate、精確 HP／Mana。
- core adapter `mana_cast` 1/1：丟棄位置／冷卻／Mana deferred mutations、清除 ledger read view 並恢復 cache view；managed script spend 明確拒絕。
- 最後 core 修改後 base 2/2 再確認通過；保留既有 td_rounds warnings。沒有 full suite／DLL stage／OmGame／Editor／網路驗收。
- 初次 `mana_cast` 篩選在新測試加入前只編譯成功、running 0 tests，不能算測試通過；後續兩個實際案例均已執行。

## 下一步仍必須實作

正式 Lua／match opt-in、出生／重生與升級容量／active dt 再生、ordered 任意脚本 Mana mutation、committed Mana facts／baseline／fresh bootstrap／filtered gameplay hash、owner HUD／IPC／UE 與 Bot 自身預算。安全投影與版本契約接妥前不自動啟用一般網路對局；目前新池只由專用功能 fixture 明確啟用。
