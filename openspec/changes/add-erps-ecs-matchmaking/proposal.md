## Why

目前 monorepo 沒有可獨立部署、能以大量玩家規模執行 Elo 配對與異質遊戲伺服器配置的服務。舊 ERPS 將房間、資料庫、MQTT 與配對生命週期耦合在單體程式中，無法直接滿足 Specs ECS 平行化、party ready check、加權容量與跨語言 client SDK 的需求。

## What Changes

- 新增獨立 process 的 ERPS，以本地 `specs` fork 管理 player、party、queue ticket、proposal、match 與 game server 狀態。
- 新增 deterministic、可分 shard 平行運算的 1v1、5v5 與八人自由混戰配對；依等待時間階梯式放寬 Elo 範圍。
- 新增 1～5 人 5v5 party、1～4 人八人自由混戰 party、CJK 房名與短效 invite token。
- 新增全員 ready check、拒絕／逾時信用處分及無責任玩家保留等待時間重新配對。
- 新增異質 game server 動態註冊、heartbeat、加權容量、1～100 instance 上限、reservation 與 launch-ready 配置流程。
- 新增 client、game server 與唯讀管理 gRPC contract，以及 TLS、版本協商、冪等 mutation 與 bounded backpressure。
- 新增 Rust async client library 與 Windows／Linux C ABI poll client library。
- 新增 property、ECS、gRPC、SDK 與 100,000 玩家 deterministic load test。
- ERPS 第一版採單一記憶體 authority；不提供 embedded `omb`、active-active 或 queue 重啟恢復。

## Capabilities

### New Capabilities

- `erps-matchmaking-core`: Specs ECS 資料模型、Elo、搜尋範圍、三種模式的 party-safe deterministic 平行配對與賽後 rating 更新。
- `erps-party-ready-credit`: Party 房間／邀請、ready check、重新排隊、斷線 grace period與信用分規則。
- `erps-game-server-placement`: 異質 game server 註冊、健康檢查、加權容量、instance 上限、reservation、launch 與失聯處理。
- `erps-grpc-services`: 獨立 ERPS process 的 client／game server／admin gRPC contract、認證、版本、冪等與背壓行為。
- `erps-client-sdks`: Rust async SDK 與 C ABI poll SDK 的公開介面、執行緒、記憶體與跨平台產物契約。
- `erps-load-validation`: 單元、property、ECS、gRPC、C smoke 與 100,000 玩家負載驗證及硬性 invariant。

### Modified Capabilities

無。

## Impact

- 新增 `erps`、`erps-proto`、`erps-client`、`erps-client-ffi` crates 與 `erps-server`、`erps-load-test` binaries。
- 新增 protobuf／tonic build pipeline、C header 與 Windows x64／Linux x86_64 動態 library 發行產物。
- `omb` 需以 `GameServerService` client 註冊、回報 heartbeat／instance，並接收 launch command；client 直接連線 ERPS。
- 新增可注入 `PlayerProfileProvider`、token validator、server-class policy 與 TLS／matching 設定。
- 不修改既有 gameplay simulation 的 deterministic state 或 script ABI；ERPS 使用獨立 Specs `World`。
