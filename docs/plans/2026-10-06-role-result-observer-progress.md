# 角色啟動器有界結算觀測

2026-10-06 增量：正式launcher改append-only reader，每poll每player最多131072 bytes、待完成行最多65536 bytes與一筆精簡已驗結果；跨poll保留玩家獨立狀態與winner/tick矛盾檢查。檔案截短／已觀測檔消失／讀取失敗／超長行fail closed並沿既有owned cleanup；不重讀整場log，不改native結果來源。新增partial lines、跨poll、backlog分兩poll、矛盾、截短與超長案例，observer由7擴充11。初版greedy newline pattern在無newline長行產生平方回溯，改reverse一次+find的線性有界搜索，不只是調大timeout；實際修後確認見E322。無真實程序／場次／完整驗收。

決策：正常互動模式不變；新增選用`--finish-timeout-seconds 1..7200`。讀取各宣告本機玩家真正可見的原生結算面板紀錄，核對player/team/winner/outcome/tick與一致性，等待截圖及兩秒顯示後只清理本run建立的程序。逾時/矛盾/提早退出為失敗，保存errors/lifecycle；不強制winner、不改HP、不重啟後端。

`--selection-smoke-hero HERO`只在interactive-selection與有界觀測同時指定時允許，沿既有真實native pointer select/lock/finalize路徑，不繞過Rust鎖定配方，也不自動取代取消選角。選角fixture既有直接設定field保留。

結果檔scope明確`native-visible-result-only`，不能當authority完整hash、全部HUD互動、LAN或GPU驗收。沒有啟動`-om-match-smoke`自動攻擊；`-om-result-ui-smoke`只是已存在的唯讀可見面板logging/screenshot gate。

專屬fixture涵蓋缺玩家、其他玩家、任意非連續player ID、雙team、Draw、重複穩定、非法identity/winner/outcome/tick、逆序/矛盾及winner不一致、CLI有界防護；補實際launcher mock走成功/逾時與反向owned清理。當前功能確認後完整選角到自然結算留最後，不將此fixture勾6.2。
