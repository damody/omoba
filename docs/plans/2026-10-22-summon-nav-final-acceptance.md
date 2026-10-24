# 2026-10-22 召喚落點、跟隨與導航最終驗收

OpenSpec `complete-moba-summon-navigation` **5/5 完成**。接續 17 系統完整度盤點，先實作單機可驗證的共用空間行為。

## 實際變更

Lua／SummonStats／generated／runtime 增加 follow_start_distance、follow_stop_distance。0/0 關閉，拒絕非法、非有限及沒有正確距離門檻的設定；雜賀內容採 750／350，屬專案平衡值。

召喚在腳本接納前與正式生成時都檢查公開 polygon 與友軍體積，並納入同批先前生成。固定順序、有界候選落點在每軸 512 內尋找；每個接納點確認主人可達性。無合法位置則拒絕／退休 reservation，沒有瞬移穿牆或讀取隱藏敵人占位。

明確 MoveTarget 優先到完成；否則超出 start 後沿公開地形返回 stop 範圍。完整主人 generation 失效或死亡則取消自主返回，不自動接到新生命。原生導航快取 waypoint 250ms，每步仍 swept-circle；腳本追擊接既有共用 static_step_toward。Root／stun、buff 移速、pause 與期限沿既有規則。導航狀態跟隨 SummonedUnit 納入 authority digest，未新增每幀 UI 操作。

## 最後集中驗收

RustV1 **1706 通過／10 既有 ignored／0 失敗**：template61、core856、codegen57、base253、backend190/1、bridge127/1、client147/8、ABI15；另 generation 成功，9 工作均 exit0／退休。

新增 6 案已含於總數：metadata 邊界 1、核心 4（落點／無位置、60Hz waypoint／步長／重播、命令覆寫／root／pause／owner generation、繞牆／期限），正式 base_content 1。正式雜賀技能在牆內要求陣形，實際 generated handler 生成合法召喚，主人移動後返回；每 tick 不入牆、最後停在停止距離內，雙次 trace／digest 一致。

UE5.8 build-v1 與 backend-v1 成功，UBT **3.52 秒**；Lua 三契約通過。WorldV1 **33 成功**（32 Success＋1 SuccessWithWarnings，4 既有 preview-world 清理 warnings，0 Failed／NotRun）。兩 map 各 74 影格／6 片段，兩張輸出已目視；這是公開場景 fixture 回歸，沒有冒稱實際網路對局 ACK。

**3348 來源／資產**在最終接受前後集合、SHA、size、mtime 一致；最後比較等待 Rust 與來源捕捉 worker／watcher 完全退休。current build／staging／DLL 與 import library 另驗。**41 份原始證據**逐 byte／SHA 封存獨立複核。三分鐘監測 **6 次／最大 181 秒**，另讀 log 與 CPU 排除卡住；**7 個 workflow worker＋7 個 watcher**及全程 health monitor 全部退出。

## 仍未完成

完整召喚可見性複本生命期、更多命令／隊形、主人重生重新綁定的內容政策、不同生成批次競爭落點的整組原子回滾、退休來源腳本 callback；通用施法排程／中斷、特殊幾何、objective stack／傳承／小兵光環、營地分裂、塔皮／公告與專用美術。

原 MVP 仍 **33/44／11 外部待驗**。未新增網路對局／重設配額／commit／push，omfx 不維護；未重跑完整 209 native／458 IPC／69 Blueprint／五 PCM，不混算。錯誤 N01–N03 與已保留編譯 warning 在本輪錯誤 MD，詳細盤點見 summon-nav-audit.md；證據索引為 `docs/acceptance/2026-10-22-summon-nav/index.json`。
