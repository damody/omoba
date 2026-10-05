# 安全入傷觀測與 Carry 目前可擊殺優先

## 計畫與決策

1. 查清披露生命週期與更新路徑：baseline 並不是每 tick replica 狀態同步，因此不能只加欄位。
2. 新增通用聚合觀測與正式 committed fact；使用原有視野 policy，Bot 與 filtered 客戶端使用同一合法資訊，不新增私有敵方讀取。
3. Carry 普攻選擇共用 E194 正式 packet 公式；只確認這個增量，完整验收留最後。

## 實作

- `DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID=0x464f470f`：v1 單個 signed Q10 bonus，8-byte big endian。與既有 40-byte property 分離，沒有 buff 身分或來源資訊。
- MOBA baseline 從 BuffStore 聚合 DamageTakenBonus；`CommittedIncomingDamage=27` 使用 8-byte little-endian sanitized event。finish hook 在正式 fact barrier 前比較上一提交值，只發 changed absolute state，刪除生命從 cache 移除。
- 納入共用 allowlist、baseline restore 驗證、projector 與 replica event apply。既有 visibility policy 管理 audience、Hide／Reveal；沒有 ComponentRepair 更新捷徑。舊客戶端不支援新 schema，需正常同步建置，不宣稱舊二進位可混用。
- Carry 只檢查已披露存活敵兵，使用本人 `UnitStats.final_atk`、`final_attack_range` 與 accuracy。可擊殺者依 HP／距離／canonical ID 穩定排序；缺少倍率、傷害／range 非正或 accuracy 有 miss 時回到既有策略。
- 範圍保持既有局部 550 上限。優先順序只用在普通 AttackTarget，作者技能政策與其他角色 focus 不變。
- 不保證 windup／投射物到達時仍能尾刀，不預測其他單位傷害或未來 buff；這是目前披露狀態下的可擊殺優先。

## 單項確認

- core `--lib incoming_damage_observation`：2 passed。免傷低 HP 兵不搶可擊殺兵、順序穩定、unknown／隊伍／種類／死亡／距離排除；事件 absolute 重播冪等、非法 payload 原子拒絕。
- base_content 同一篩選：2 passed。正常 Production60Hz driver 的 Carry 正式 AttackTarget、未提交敵方 buff 變更不改決策、重新提交後更新、移除 current 身分不讀 cache。
- filtered 增量：兩隊各 6 steps（12 total），正／負／免傷／歸零／Hide／Reveal；每步 canonical hash 與權威重新 bootstrap 一致，零 ComponentRepair。Hide 時敵隊拿不到倍率變更，owner 仍有合法事件。
- 測試命令當次 TEMP/TMP 指向 D 槽既有 scripts/target/debug，退出還原，以避開 C 槽已滿（E194）。本批真正失敗與修正記於 E195。

## 剩餘

本增量不等於完整 Carry farming／波控或三原型 Bot 完整性；未跑全套、100 場、UE、LAN 或效能驗收。OpenSpec 5.5 維持未勾選、總計 20/30；E177 共享引擎變更阻擋保持。沒有改 Lua 内容／full content hash、Unreal C++／BP 或部署 DLL。
