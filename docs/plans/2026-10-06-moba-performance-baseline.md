# 正式 60Hz 本機双玩家效能基線

## 固定決策

使用 `baselines/moba-60hz-local-two-player-v1.json` 的十二個固定門檻。先以雙玩家 release、compiled-content-only、single_lane、雙1280×720 D3D11 renderer、60Hz與一次renderer重連建立樣本，再於固定門檻後另開全新同配置執行。不得因第二次不通過調大門檻；失敗須保留並診斷。

首次採樣 `target/interactive-runs/final-reconnect-20261006-project-bound`，修正後報告 `performance-baseline-capture-p1-fixed.json` 與 `performance-baseline-capture-p2.json`：六個量測完整、error/missing0，但 thresholds=not_evaluated，不能稱門檻通過。此前失敗 `performance-baseline-capture-p1.json` 保留。

| 量測 | 首次雙玩家 mean 最差 | 首次 max 最差 | 固定 mean / max 上限 |
|---|---:|---:|---:|
| server compute | 3.224ms | 65.351ms | 5 / 100ms |
| client replica apply | 0.381ms | 24.779ms | 2 / 50ms |
| IPC send message | 2336 bytes | 3350 bytes | 4096 / 8192 bytes |
| IPC receive message | 17 bytes | 32 bytes | 64 / 512 bytes |
| UE game-thread interval | 16.688ms | 53.020ms | 18.5 / 80ms |
| UE presentation work | 0.481ms | 8.011ms | 2 / 15ms |

mean 為總和/樣本數向下取整，不是p95。server計算排除排程sleep與transport send waits；client排除證據IO與呈現；IPC是message byte數與另報window throughput，沒有延遲量測。UE interval不是GPU/render-thread時間；presentation work獨立量測。門檻為這台共用開發機的回歸上限，保留可觀測jitter空間，不代表每幀16.67ms、十Bot壓力或LAN規格。

初次有server65ms、client24ms、UE53ms尖峰，不能宣布穩定60FPS。邏輯60Hz與三方hash另由遊戲報告核對。其他專案程序可能共用硬體；不終止它們以製造數據。

## 建置與產物身份

本轮正常-NoEngineChanges UBT13 actions及後續fixture4 actions成功；engine/project/兩plugin manifest BuildId `1323cea4-7408-4662-8321-6abdaf191604`，沒有手改。當前UE staged debug bridge SHA `0e5bac851dff2e8a4a3b7acdaeac6b200ab65897f174838bb6fa9be48154fe5d`；base SHA `2cb13f4dcec12e34bd1a24c0e9f0b27f2d27843c79784409088a503396dbe60f`。release base因共用build-time loader依賴重新生成，SHA `20d10f23dcb1e386b4667df1eb12bedb6841dddf1f1f410ce5cb87cf05346aef`；不能冒稱舊100場使用的c68e46…身份。compiled registry check17/17/hash d5553bbb31c459a4不變；沒有新增Lua遊戲runtime。

## 獨立驗證

`final-perf-validation-20261006-v1`全新同配置run已exit0，正式60Hz renderer重連/雙隊三方一致/所有owned程序清理成功。雙玩家performance-validation-p1/p2.json均exit1/threshold fail：server max133.894ms，client p1 max58.001ms/p2 max132.987ms超標。所有mean、IPC與UE間隔/呈現工作通過。保留原門檻，不以「遊戲run成功」等同效能合格；6.5仍未完成。

三個尖峰window同05:25:15Z，首次renderer ACK同秒，重連ACK約05:26:14Z。硬體AMD Ryzen9 9950X3D/16cores/32logical；共用Rust pool每pool32threads，多程序排程競爭可能影響，但沒有CPU sampling證據不得當已定因。需要後續通用資源預算/分段診斷，不能用反覆重試挑通過樣本或調大上限。
