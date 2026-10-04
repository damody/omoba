# 60Hz owner K／D／A 投影與 Unreal 共用 HUD

## 本輪決定與實作

- 先完成配置玩家的持久 K／D／A 管線，不把此增量當作完整十人計分板。權威 MobaMatch 保存玩家分數；namespace 加完整 player ID，僅向所屬隊伍發布，不輸出助攻參與 ledger、敵隊金錢或隱藏位置。
- active tick 只在 PostStep 傷害／死亡結算後發布一次；inactive tick 使用單一 PreStep。首測發現同 tick 發布兩次會讓 first-match reader 讀到死亡前舊值，已修正並列入 E111。
- runtime 將三個有效 u32 metric 投影為 optional OwnerScorePresentation；缺少、負值、超量不偽造 0/0/0。死亡不依賴存活英雄，reconnect baseline 保留分數。
- bridge 核對 score.player_id 與 HUD owner，C ABI 升至 8，拒絕舊 7。Unreal 共用 payload／WorldBridge／控制器／CommandBar 顯示 K/D/A；UPROPERTY 使用 int64 保留 u32 全範圍，unsupported 顯示 `K/D/A --`。沒有角色專屬 C++ 或 Blueprint graph。
- saved capture 驗證不假定 Move-only 沒有死亡：NPC 仍可合法擊殺英雄。render snapshot tick 是已套用 frame tick + 1，以精確減 1 對照原始 wire；不以最近值或最大值掩蓋差異。

## 驗證結果

- Rust：base_content 84、core lib 329、server 154（1 ignored）、runtime lib 56（5 ignored）＋main 3、bridge lib 51（1 ignored）＋legacy integration 2（1 ignored）通過；Fyrox `cargo check --tests` 通過。這輪未重跑 TD 1–100 integration。
- 640 tick 三英雄回歸：每步雙隊 replica／fresh bootstrap hash 一致、零 ComponentRepair；owner metric 核對擊殺、死亡、助攻及隊伍隔離。
- 真實 60Hz 三外部 runtime：`target/interactive-runs/moba-runtime-1791077095/moba-runtime-smoke-report.json`，success／cleanup true；雙隊各 9、第三玩家獨立 8 checkpoints 至 1080，safe 1094，三人實際移動。
- 上述原始 wire 與 IPC capture 精確驗證：player 1 為 1187 筆／tick 1348；player 2 為 1081 筆／tick 1343；player 3 為 1056 筆／tick 1337，共 3324 筆 score 一致。含 NPC 合法死亡，核對 manifest player/team 及所有 score metric 隊伍隔離。
- Unreal full build：18 C++ actions 通過；11 Blueprint validation gate `target/blueprint-validation-runs/compile-1791077018/report.json` 通過；同一 Editor 兩輪各 14/14 原生 automation（含 NativeOwnerScore）通過。另有既有 native art／memory PIE smoke 通過，不是 KDA 實戰畫面驗收。
- 最終 build-only、ABI 8 stage gate 與 codegen --check 通過；11 files／15 Lua inputs，content hash `de9c7fcfc98d6479`。最終 staged bridge SHA-256：`7a4d11a7297bbc562ca18c613323a2d51a6ea5483034b855660c0b61ef848a5d`。
- 所有本輪 smoke 擁有程序 51972／88396／97792／2908 與 Editor 87196 已清理並獨立核對退出；Editor 正常 SaveAll 後關閉，未清除任何使用者程序。

## 重現

```text
OMOBA_SCORE_CAPTURE_ROOT=D:/code/omoba/target/interactive-runs/moba-runtime-1791077095
cargo test --manifest-path omoba-client-runtime/Cargo.toml real_three_player_owner_score_capture -- --ignored --nocapture
tools/lua/lua.exe scripts/build_ue_moba.lua --verify-staged-only
```

capture 是未版控執行證據，測試預設 ignored；指定來源才執行，不生成或改寫 capture。

## 明確尚未完成

未驗收真實 KCP 玩家擊殺／助攻、多人 Unreal KDA 實戰畫面、完整公開十人計分板、選角至結算及完整 LAN。OpenSpec 5.3／6.2 不勾選；本增量不代表整體框架完成。下一步以正式玩家輸入驗證多人擊殺／助攻，再接公開計分板的安全資料契約。
