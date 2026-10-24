# 通用Buff層數錯誤紀錄

目前沒有建置／測試失敗。沿用前批程序與watcher完全退休才允許重試的守門。

- B01：讀取BuffStore時誤省略native路徑。以rg --files定位為runtime/native/ability_runtime/buff_store.rs；未更動程式。
- B02：查C ABI header時誤猜bridge/include。改以rg --files定位；避免把猜測路徑帶入驗收。

設計注意：先過owner／generation／duration條件才解析JSON，避免將完整場景Buff數量帶入本機HUD解碼熱路徑；payload bytes與字串邊界先驗證。
