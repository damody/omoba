# 選角 TCP 與正式設定工具整合

## 計畫與決定

1. 將前一批 Rust 最終配方分發編譯進正式 release moba-config。
2. 使用既有 connection worker／Invitation／JSON-line transport 確認兩位真人的交付路徑，不另建房間或測試版規則。
3. 子代理若未交付不拖住整合，primary 接手；保存可重現指令、錯誤及明確驗收邊界。

## 完成結果

下列建置成功，正式主機／選角 proxy 已包含 E335 的只讀 final plan 分發：

```text
cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only --release
```

omb/src/bin/moba_config/selection_network.rs 新增 NetworkPeer fixture：真實 loopback TCP、2 秒 client 時限、RAII 關閉／join，使用正式 connection 而非替身。私人 token 錯誤必須 PermissionDenied，沒有選角回覆、席位占用或 revision 變更。兩真人收到尚未完成的初始配方為 null，各自 lock 後才 finalize；host take 已消耗後，follower read 及席位重新 admission 仍得到相同完整配方／snapshot，host take 不再產生新 ticket，最後 registry 清空。Bot 席位保留。

```text
cargo test --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only selection_network_ -- --nocapture
```

3 項通過；新增 1 項實際 TCP 多席位案例及 2 項既有 framing／原子 publication。格式化後重新確認也通過，計數不重複。既有 dead_code 及 protoc 使用受版控 fallback warning 不當作錯誤成功或遊戲功能結論。

## 子代理與審查

使用 Grok Build 委派技能提供單檔有界修改與獨立審查契約。job run-muwjfkw0-w21bq7、thread 5a874ee0-7ee6-431f-b063-f63944a18868；203 秒停在讀取文字、沒有產碼。primary 取消，追蹤 terminal cancelled／三 PID null 並另確認原 101476／43324 不存在後實作。沒有接受 Grok 補丁；成本／API 時間／根因未知。stop exit1/taskkill128 不是單獨清理證明。

## 尚未宣稱完成的部分

這是 production worker 的 loopback socket 局部確認，不是完整 executable host accept loop／proxy stdout pipe／Unreal renderer，也不是兩台實機 LAN。僅 moba-config 正式 release 新建，其他正式遊戲 DLL／runtime／Unreal 最後仍需一致部署與整體驗收。沒有啟動遊戲／Editor／場次模擬、沒有修改 omfx 或執行 commit／push。OpenSpec 保持 25/31，六個完整驗收項不因這批勾選。防錯紀錄 E336。
