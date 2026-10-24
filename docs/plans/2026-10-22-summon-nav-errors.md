# 召喚導航錯誤紀錄

- N01：生成直接寫入要求 Pos，可能在 polygon 內；移動只做碰撞滑動，缺繞行與通用跟隨。以共用公開地形幾何、固定順序落點與有界 waypoint 規劃修正，不使用隱藏敵方位置。
- N02：OpenSpec CLI 前輪已確認不可用；依既有結構維護 proposal/design/tasks，依使用者授權直接實作並記錄決策。

- N03：再次推測 native/mod.rs 路徑而讀取失敗；正式入口是 runtime/native.rs。後續一律先 rg --files，再讀定位結果。另 apply_patch 的 hunks 必須按檔內順序，反向 hunk 已拆分後成功。


本輪保留的非致命編譯 warning：既有模板 dead_code／unused import、protoc 使用 generated fallback；另 summon_tick 測試 fixture 的 world 有多餘 mut（行為不受影響）。不把有 warning 的建置宣稱為零警告。

