# Authored recovery skill — final acceptance

既有晨光學徒的 R「定心療癒」已改為真正的站定引導：2 秒內每 0.5 秒恢復 35/55/75/95 生命，共 4 次，總計 140/220/300/380。沿用原本 ID、配裝、圖示、魔力 90/100/110/120 與冷卻 25/23/21/19。晨光導師的原本瞬發回復保持原樣。

共用資料使用封閉 RecoveryChannelSelf 與 typed HostAbilityExecution；effects_preview 仍只供說明。作者資料、Rust schema 與固定 Lua 生成器都拒絕混合效果、不完整等級、非精確 fixed-point 時間及不整除脈衝。原生 host 在施法交易內預檢精確 owner/entity generation、生命、階段與控制；拒絕不扣費，接受後共用道具既有 recovery clock、治療結算及中斷。GameWorld stable ABI 方法表沒有增加，沒有依英雄或營地名稱特判。

移動、停止、普攻、其他施法、生命受傷、暈眩或沉默會中斷；定身允許站定引導，暫停凍結時鐘，治療沿用最大生命與治療削減。正式引擎先排施法後排移動，因此同 tick Stop 在兩種輸入清單順序都取消；下一 tick 可正常重施。共用提示從實際 pulse/duration/interval 自動產生完整各等級資訊，經 generated registry 和身分綁定 HUD payload 進入既有 tooltip。

## 集中驗收

- V3 八個正式維護 Rust 套件：1980 passed、0 failed、9 個既有 ignored；另有 probeV4 的 12 個聚焦通過案例，未合併重複計數。
- Lua 作者／生成合約 18/18，加召喚提示回歸 19/19，共 37；使用固定 tools/lua/lua.exe。
- 實際作者技能走正式 60Hz：脈衝、MP/CD、root、pause、傷害與 Stop 邊界、控制拒絕、paired replay 通過。
- 正式後端、moba-config、UE5.8 建置及 DLL/import library staging SHA 核對通過；離線 plan-lock 的權威與 generated catalog data hash 相符，catalog IDs 與 class surface 保持一致，未建立真網路對局。
- 六項原生驗收全部 Success、0 warnings：AuthoredRecoverySkill、RecoveryChannel、AbilityUpgradePreview、SummonGuidance、OwnedChannelProgress、PublicChannelDuration。
- AuthoredRecoverySkill 檢查實際 generated metadata 與正式 HUD producer／tooltip；其 2 秒公開狀態展示是明確標示的 synthetic presentation fixture。共用 public ABI 另外以既有 3 秒真 export fixture 驗證。不能把 synthetic 截圖宣稱為這個新技能的完整 ABI 端到端錄影。
- recovery-skill.png 已實際檢視：名稱、各等級脈衝／總量、中斷／root／pause、升級 MP/CD、剩餘 1.5 秒顯示可讀，無裁字。畫面中的英雄資源是 presentation fixture，不是正式對局資源證據。

## 來源與失敗保留

最終 3479 個正式來源／資產完整 fingerprint 在所有建置後一致；父基準為已封存 summon-guidance。55 個既有候選原始內容及 1 個新增 Lua 測試皆明列並凍結。V3 相對 V2 只多納入 HUD 圖示清單的作者名稱同步；V2 相對 V1 只有舊 registry 測試的單一斷言更新。原始四個 probe、失敗 acceptV1（舊 preview 假設）、失敗 acceptV2（建置時 label projection）及成功 acceptV3 共七個 owned workers 保留 owner/waiter、原始輸出、固定輸入、health、exit receipt、retirement、cleanup 與原始 API reap。錯誤根因／修正／預防見 2026-10-24-recovery-skill-errors.md；未降低來源守門或製造假的瞬發 preview。

證據索引：docs/acceptance/2026-10-24-recovery-skill/index.json，含 sealer actual API reap 與 post-sealer independent rehash。正式來源下一批基準：omfue/Saved/Automation/RecoverySkillV3/source-before.json；凍結 inputs.json 與 recovery-skill-parent-v3.json 一併保留。

## 下一個具體缺口

Bot 目前仍只允許 instant ability policy，且決策只跳過 recall，尚未等待 recovery channel。本批沒有宣稱 channel-aware bots。接續優先加入封閉的 channel-recovery intent、根據已公開觀察安全開始，以及引導期間等待／結束後恢復；同時涵蓋既有道具與技能，避免全域放寬未知 cast type。之後還有 projectile/multistage、玩家召喚物控制及內容／美術／平衡。MVP 33/44、11 個外部驗收不變。

無人值守期限維持 2026-10-26 23:59:59 Asia/Taipei；未新增真網路對局、重置 quota、commit/push、維護 omfx 或終止無關程序。

Archive independently rehashed files: 761; source unchanged 3454, changed 24, added 1.
