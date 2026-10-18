2026-10-12 Codex自行權威戰鬥設定範圍接受：`.ai-collab/combat-settings-accepted-v1/summary.json`。acquisition／idle-autoattack由authenticated owner更新名冊，selective10／renderer8／ABI27，完整投影與hash／bootstrap／rebase／死亡／重連守衛接通；Unreal兩個真Slate checkbox保存desired profile，按owner／connection／epoch單次同步，只有實際投影相符才顯示已套用。既有純本機英雄限定不混入權威偏好。

Rust v7真core734／client-wire2／backend-wire1／bridge120（1既有ignored）及四all-target checks成功；此批Rust沒有歷史前後SHA證據，不補造。ordinary build-v3 NoEngineChanges／NoUBA／nocache成功；五項native-v1全Success且五warning完整保留，之後僅JSONL測試gate變更，fresh observer兩項零warning及fresh owned-v2 selection17cases／4intents／4stages／零warning／cleanup成功。69Blueprint零compiler error／warning／load failure、package SHA／mtime不變。435來源在fresh observer／owned／Blueprint期間核對不變。三張當次original圖目視，另四張設定頁縮放圖亦檢視。E1001–E1006原失敗與修正保留；無新authority game／ACK，非full414或完整世界驗收，父項12/44與10/10 quota不變。完整驗收最後，接英雄個別設定與WASD正式權威輸入。

2026-10-12 權威戰鬥偏好名冊基礎範圍接受：`.ai-collab/combat-roster-accepted-v1/summary.json`。Codex自行實作，無子代理。實際 MobaHeroSlot 保存 acquisition／idle-autoattack，constructor 預設、authenticated-owner 更新與 phase／非法 raw／重複 owner 拒絕；死亡與真 spawn replacement 保留、舊 source 退休、兩 owner 隔離、replay diagnostic digest 與實際 hero_tick／hero_command_tick consumer 驗證通過。權威名冊優先於 replica 注入 cache，暫時只選英雄不持久化。

fresh Rust v3：core-all 728 passed／0 ignored，targeted 5 與 core 重疊不加總；bridge IPC 41 passed／1既有 ignored；四個 maintained all-target checks 全成功，六來源全程凍結核對。E994／E995 原失敗 logs／helper／summary 保留。無新 UE build、Blueprint、對局或 ACK；Unreal stage 尚未重新包含這批 Rust 變更。傳輸、bootstrap／reconnect 與設定介面尚未接完，父項12/44／quota10/10保持，完整驗收最後。

# 權威戰鬥偏好

## 正式傳輸與 Unreal 接線計畫（2026-10-12）

1. selective protocol 由實際目前的 9 升為 10，renderer IPC 7 升為 8；新 schema 1 設定僅含 acquisition／idle-autoattack，owner 沿 authenticated InputSubmit。舊 client 明確版本拒絕，不默默忽略改變決定性選敵的新狀態。
2. 每幀發布己方名冊與 committed visibility 內敵方英雄的完整有序偏好。不存在的敵方 source 不發布。死亡己方仍保留；暫時只選英雄不入 wire。合法更新不改移動／攻擊 queue 或 cooldown。
3. bootstrap／continuing join／rebase 保留同一份偏好；新 epoch 必須重綁。rebase manifest 與 authority／replica canonical team hash 都綁偏好。非法 schema、epoch、owner 0、重複／亂序 owner、數值溢位要在安裝前整批拒絕。
4. filtered stepper 在本幀戰鬥前安裝完整 admitted owner map，缺投影則清掉舊 map，不能把舊 epoch／已隱藏敵方偏好留下；設定不是 actor-life gameplay 指令，replica 不重播該 input，權威則由正式輸入 router 改名冊。
5. renderer 必須具有當前 epoch 的合法自身名冊投影才可提交；死亡不需要假 actor，missing projection 不代表允許。owner／epoch／shape 檢查通過前不得配置 input ID。
6. Unreal 正式 ABI 新增專用設定 command 欄位與 owner-only frame view；不濫用 ability/item slot 或 queued bit 傳偏好。ABI 升版後由既有 cbindgen 工具產 header，驗 offset／size／舊版拒絕。UI 使用現有原子 profile 保存與主／副快捷鍵系統，兩個 checkbox 顯示 desired profile；純 cached status 區分已保存／等待連線／等待對局／送出／權威已採用。
7. 設定同步以 player／connection／view epoch 與 desired tuple 管理單次提交。按鍵焦點或 hero replacement 不把名冊設定重置；重連重新核對權威投影。modal 設定頁允許提交偏好，但不放寬移動或施法輸入守衛；Slate bound getter 不做 widget-path 查詢。
8. 完成正式 Rust／Unreal／真 Slate／owned IPC fixture 驗證再接受當前功能。既有 10/10 authority-game quota 保持，不把 synthetic ACK 或 renderer queued 當成權威 ACK；完整專案驗收最後執行。

Codex 自行實作，無子代理。使用既有 deterministic combat_controls 與實際 hero_command_tick／hero_tick，不以客戶端定時送攻擊或移動模擬偏好。

1. 每個 MobaHeroSlot 保存合法 owner 的 acquisition／idle-autoattack 設定，預設 hero-origin／開啟 idle；死亡及真 replacement 保留原名冊設定，納入 replay diagnostic digest。
2. 更新必須核對 authenticated owner、名冊唯一身分、合法 acquisition 與 playing phase；錯誤不得改任何設定或摘要。暫時只選英雄狀態維持本機指標策略，不混成全局權威設定。
3. 實際權威 consumer 直接讀對應名冊；不建立第二份 mutable authority cache。既有 filtered replica 的 admitted resource seam 保留，不能覆蓋權威名冊。Cursor 只影響明確 A 選敵；idle 仍依英雄位置，關閉 idle 不禁止明確 A 或 AttackTarget。
4. 先驗真 production consumers／兩 owner／非法更新／死亡重生／digest，再接 authenticated wire、bootstrap／reconnect、Unreal 設定與實際輸入。基礎完成不宣稱設定已對玩家生效；每輪錯誤寫入 register，完整驗收最後。
