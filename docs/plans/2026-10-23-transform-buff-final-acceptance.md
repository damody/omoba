# 自身變身技能共用校驗與最終驗收

完成 `complete-transform-buff-validation`。火繩銃與三段擊改用同一個有界資料解析／living-self 執行器，先檢查等級、JSON、時間、數值、型別與整數發數，再提交唯一 buff。省略 extras 才使用相同 authored rank；顯式錯值不再靜默補零／fallback。一般屬性維持 Q10 整數，attack_stun_* 維持主機實際讀取的小數格式，未變更七個原始 rank 的平衡數值、ABI 或 Unreal C++。

新測試驗證所有 rank、負值／超限／錯型別／null／低於量子／非法目標／死亡或退休來源均無部分效果、recast 單項 refresh 與精確 expiry。正式 60Hz PlayerInput 驗證七個 rank 只扣一次魔力／冷卻、pause 不倒數、冷卻拒絕不刷新、真正普攻吃到增益，三發視覺只有一顆有傷害與 normal-attack 資格，實際飛行命中後總傷害只一次，變身到期恢復原攻擊。

## 最終結果

- Rust V2：**1966 passed／10 既有 ignored／0 failed**，12 工作含內容生成；base_content 295。四項新增測試的 focused probe 4/4 通過。
- 四項 Lua 契約、backend V1、UE5.8 build V1 成功；UBT 保留 -NoUBTMakefiles，增量執行 1.66 秒。7 個 staged DLL 與 import library SHA 均符合建置輸出。
- World V2：**56 成功／0 Failed／0 NotRun**（55 Success、1 SuccessWithWarnings）；4 個既有 RuntimePresentationReset／Untitled DestroyActor warning。
- **3404 來源／資產**集合、SHA、size、mtime 前後一致；與第一輪 native 建置輸入另比較，只有 Rust 測試檔的 lazy-resource fixture 修正不同，其餘正式輸入 SHA／size 一致。
- 封存 **151 份證據**並另行逐 byte／SHA 校驗；World worker／watcher 退出後目視 **16 張 PNG**。既有 layered 地圖標籤擁擠、first-kill 下方 HUD fixture 裁切與 objective 窄 strip 限制保留；本批沒有新增 UI 視覺功能。兩地圖各 74 影格／6 片段、authority_ack=false。
- 健康採樣 **7 次，最大間隔 171 秒**；7 組最終及 5 組歷史／probe worker／watcher 與 health monitor 全數退休。

## 錯誤與範圍

Rust V1 294/1 的唯一失敗為新測試提前讀取尚未由首次投射物建立的 ProjectileOrigins；所有讀取者退出後，僅把 prior set 讀取改為 optional，未放寬傷害或數量斷言。另修正 World runner 複製時誤改的既有可選截圖參數，V2 補齊畫面。失敗及原始證據均保留，詳見 2026-10-23-transform-buff-errors.md，總登錄 E1497。

全遊戲盤點見 2026-10-21-whole-game-completeness-audit.md 與本批 audit.md。持續引導／區域、定時位移／殘留、特殊幾何完整預覽、召喚整組 commit 原子回滾、完整 combat replica 生命期接受與更多內容／美術音效仍待補齊。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持 pending。零新增真網路對局、quota reset、commit／push，omfx 未維護。

證據索引：`../acceptance/2026-10-23-transform-buff/index.json`。
