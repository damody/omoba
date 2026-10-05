# 宣告式技能共用施法驗證（2026-10-05）

## 計畫與決策

1. 延續任務 5.5；檢查通用單體 Damage 發現只有目標生命／敵方關係檢查，没有像 AreaDamage 驗證實際施法距離。不以 Bot 的距離篩選或 Unreal 防呆代替權威技能檢查。
2. 共用內容模型把 bounded cast range 規則套到所有有目標的宣告式效果：每個 rank 在 [1/1024,10000]、有限；無目標 HealSelf 仍允許零 range。Rust template IDs、runtime content 與 UE codegen 使用同一驗證；固定 Lua FFI 生成器對 Damage／AreaDamage 也檢查完整 per-rank range。
3. ResolvedEffect::Damage 帶内部 cast_range，單體為當前 rank 的 Some(range)；範圍展開的受害者為 None，因中心點已通過距離驗證。這不是新 ABI 欄位，也沒有新增腳本 GameWorld 方法。
4. 共用 apply_effects 先驗 caster 的有效 handle／正 HP 生命，再完整驗所有單體受害者的生命、敵方關係、兩側位置與 Q10 平方距離。精確距離邊界合法；超界不輸出 Damage／Heal，整个序列在提交前拒絕，不讓排在前面的治療部分生效。
5. HealSelf 在 resolve 階段明確要求 Target::None，不能接受指向自己或別人的 Entity，也不能帶 Point。不存在目標補救、偷偷換目標或降低規則的 fallback。
6. 冷卻仍由既有 scripting dispatch 在 execute 成功後啟動，沒有手動改 Hero cooldown。正式輸入接受／排入 ScriptCast 不等於效果生效；拒絕效果不代表撤銷输入 admission，也不宣稱恢復 handler 已清除的舊命令。
7. 範圍技能只限制中心點，不把每個受害者再當單體檢查 cast range；合法範圍中心能傷害半徑內、但超過自身 cast range 的敵人。這是宣告式瞬發效果共同規則，不擴大到既有特殊 Rust handler、持續技能或任意投射物。
8. 檢查盟友治療前置時发现 GameWorld::faction_of 尚為 RNone；本批不把「不是敵人」反過來當「友軍」，也不為了快速新增治療功能擅自改 ABI。先修已存在的施法驗證缺口。

## 當前功能確認

- 新 base_content cast_preflight 2/2：純執行器距離邊界、超出一個 Q10 raw unit、死 caster 與錯誤 self target、全序列原子拒絕；正式 60Hz ECS 超距單體不扣 HP／CD，entity／point 治療不生效，精確邊界 80 傷害與 None 治療 70 正常且啟動 CD。
- 新 shared model cast_preflight 1/1：第二 rank 的零／負／低於 Q10／過大／Infinity／NaN range 拒絕，合法兩 rank 通過，無目標治療零 range 保持合法。
- 更新既有 AoE 60Hz fixture 1/1：point 中心選 x590，敵方 x790 超過 cast range700但在 radius220內，仍正確受傷；同隊與半徑外 x1000 不受傷。證明未把單體限制誤套到 area hits。
- 同批既有 generic_effects 3/3：原目標／rank 拒絕、四個 Lua 共用效果與原 AoE 原子展開通過。這是直接相關確認，不是完整對局驗收。
- 固定 Lua include 功能 5/5 通過；合法測試 fixture 補齊 levels={{range=600}}，不讓舊的缺省 range 測試資料繞過新規則。
- UE codegen --check 成功：11檔／17 Lua inputs、content_hash=5638c23df5bba3c7，既有生成資料無變更。沒有新英雄 C++／Blueprint，也沒有本輪 OmGame／DLL stage／Editor／PIE。

## 未完成

盟友治療、Mana、位移／控場／持續區域、全部特殊 Rust 技能驗證、完整 filtered／KCP／UE、100 場／LAN／效能與最後驗收仍待完成。OpenSpec 維持 20/30，5.5 不勾選；錯誤與防再犯見 E169。未提交、推送或清理既有工作樹。
