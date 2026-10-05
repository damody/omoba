# 通用建築披露與 Bot 攻城選擇

## 計畫與決策

1. 檢查建築披露及權威解鎖，修正塔／基地被當成兵的判斷來源。
2. 獨立 safe structure observation 與 changed absolute fact，沿原視野規則，Bot 不查 hidden prerequisite／HP。
3. 一般戰鬥優先、無戰鬥目標時才選範圍內合法建築；只確認本功能，完整驗收最後執行。

## 實作

- MOBA `disclosed_structure_states` 共用正式 structure_unlocked 與 Playing phase 判斷，只輸出角色與當前可受傷 flag，不公開前置塔身分／隱藏 HP。
- `DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID=0x464f4710` v1 固定兩個 bytes：[role1塔／2基地, admitted0／1]。既有 render.kind、40-byte property 與 Unreal schema 不變。
- `CommittedStructure=28` 發布絕對、change-only observation；勝負結算後、fact barrier 前擷取。allowlist、restore validation、projector／replica apply 全部接線；invalid role／flag／length 在寫入前拒絕。
- Birth／Reveal 最新 baseline、Hide／Forget 原生命週期沿用，沒有每 tick ComponentRepair，也沒有 Bot 特權 cache。新 wire schema 要整套正常建置，未宣稱舊 DLL／client 可混用。
- Bot perception 内部分類4塔／5基地／6鎖定，只從 current safe payload 取得；這不是改 renderer kind。兵／英雄戰鬥優先於建築，建築依距離／canonical ID穩定決勝。
- Top／Mid／Carry／Support 可在無普通戰鬥目標時提交近距合法建築 AttackTarget；Jungle 保留原 camp／支援策略。既有護衛、療傷、回城與尾刀優先不變。
- Carry 只對 kind2 兵計算尾刀，建築不再誤用 farming 保留；進攻技能 intention 仍按原英雄／兵候選，不新增任意技能攻塔假設。

## 當前確認

- core `--lib disclosed_s`：3 passed，其中2個新功能（合法建築優先／兵與英雄先／locked與隊伍range排除／Carry不把建築當兵；absolute apply冪等／非法payload原子拒絕）及1既有Support測試。
- base `--lib disclosed_structure_state`：1 passed。正常 Production60Hz layered map，nearer locked inner tower不取代outer；outer HP0尚未退休不解鎖，權威退休後新披露讓Bot選inner。
- 兩隊各4正式步驟（8 total）：unlock、Hide、Reveal、Finished；每步完整canonical hash與權威重新bootstrap一致，零ComponentRepair。終局fixture用 forced base Death隔離結算時序，不冒充合法擊破鎖定基地或完整對局。
- 首次fixture移動靜態tower導致hash不符，改英雄移動進出視野後通過，見E197。當次TEMP/TMP沿E194使用D槽建置目录並還原；未清理C槽或使用者檔案。

## 尚未完成

不等於完整攻城戰略／兵線護送／塔傷害評估或五位置Bot全部完成，也不是Unreal建築外觀／HUD驗收。未跑100場、全套、UE或LAN；未修改Lua內容hash、生成C++／BP或部署DLL。OpenSpec 5.5與整體20/30保持未完整完成；E177共享引擎變更阻擋仍保留。
