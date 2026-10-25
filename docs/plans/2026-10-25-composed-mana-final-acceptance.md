# 技能組合：施法者回魔最終單機驗收

OpenSpec `complete-composed-caster-mana` 3/3完成。

既有「晨光援護」治療指定存活隊友或自身140/220/300/380 HP，現在施放成功後額外回復施法者10/15/20/25 MP；保留600施法距離、原有消耗與冷卻、所有英雄／技能ID。說明透過共用資料生成至既有技能提示。這是明確的支援技能內容調整，不宣稱複製LOL的數值或技能。

共用模型、固定Lua註冊產生器與執行器區分「選取目標」和「施法者自身效果」。RestoreManaSelf可搭配形狀／距離／陣營已驗證的目標效果，單獨回魔仍只接受無目標；其他自身消耗、治療、buff及原有exclusive幾何／位移／引導限制維持。所有效果先完成preflight才提交資源，沿用既有host serial ManaPool overlay與失敗rollback；必須先負擔metadata cost，不能用尚未取得的回魔支付消耗。回魔夾限至容量，治療對象不會因此獲得魔力，禁療只影響HP。沒有英雄／技能名稱特判、新wire欄位、額外每幀世界掃描。

聚焦Rust 5項與固定Lua 43項通過。任意技能名schema、隊友／敵人／點目標組合、純自身錯誤目標拒絕、遺漏資料、原有self-only限制與順序一致；generic四級、反向宣告順序、敵方／死亡／超距離先拒絕、不留下治療或回魔、cap與實際caster recipient；正式Production60Hz四級cost不足拒絕、成功一次結算、冷卻防重複、pause和paired replay；既有實際三玩家隊友治療bot案例同步驗證施法者回魔。host原有rollback／資源交易測試列入全套回歸。

FreshRust九套 **2104通過／0失敗／9既有ignored**；正式backend／moba-config、UE5.8、DLL／import staging、離線plan/catalog一致成功。原生4項：CombatAim.DisclosedAllyCard、UnitTargetPreview、Generated.SummonerManaIdentity、Runtime.SummonerPayload，0測試warnings。沒有新增截圖、真網路對局或physical device驗收宣稱。ABI36／presentation12／selective14維持。

**3489來源／資產一致**：相對上批3477未變／12變更／0新增；集中驗收前後hash、set、size、mtime一致。封存452份證據並獨立重核；index `docs/acceptance/2026-10-25-composed-mana/index.json`，ledger `omfue/Saved/BuildTemp/composed-mana-final-ledger-v1.json`。原始失敗／輸入、父來源、所有owned工作原始API退出與退休、health gap≤180秒均保留；CM01–CM06根因／修正／預防見errors MD。未commit／push／reset quota／維護omfx／終止無關程序。

MVP仍33/44、11項外部驗收待辦；本批只完成可在單台電腦驗證的技能組合與既有呈現路徑。
