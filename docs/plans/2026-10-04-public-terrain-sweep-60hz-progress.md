# 公開地形整段碰撞／60Hz 前置增量

## 計畫與決定

- [x] 檢查既有 BlockedRegions、hero grid planner 與 filtered metadata 路徑。
- [x] 修正「落點合法但途中穿牆」：共用 Rust 固定點 swept-circle polygon。
- [x] BFS 每條 edge、同格／超大跨度 fallback、實際移動／axis slide 使用相同檢查。
- [x] 新增薄牆／斜角／相切／反向 winding／凹多邊形／重複點／超界／零預算測試。
- [x] 60Hz 正式 MoveTo、三 seed、雙隊逐 tick hash／無 repair／重播與抵達測試。
- [x] 全套既有遊戲回歸、Unreal build-only 與最後 staged DLL SHA 核對。

不另寫角色 C++／Blueprint graph。不讓隱藏實體影響路徑；沿既有公開靜態 metadata，沒有新增私密 AI 資源披露。

## 已驗證

- `cargo test --manifest-path omoba-sim/Cargo.toml`：69 unit＋8 deterministic integration 通過。
- `cargo test --manifest-path omoba-core/Cargo.toml --lib`：338 通過。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content public_terrain_60hz -- --nocapture`：通過；三 seed 1／42／539365380 各 600 tick，共 3600 雙隊 filtered steps。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib --quiet`：106 通過，185.07 秒；包含既有三路完整 lifecycle、野區、經濟／技能／死亡重生 filtered 回歸。
- server lib：156 通過／1 benchmark ignored；client-runtime：61 通過／8 opt-in ignored＋3 integration；Unreal bridge：53 通過／1 opt-in ignored＋2 integration／1 opt-in ignored。
- codegen 絕對 content-root／out `--check`：11 files／16 Lua inputs，content_hash `3b296ff5bdbcd7d9`，exit 0；OpenSpec strict 與本輪範圍 diff whitespace check 通過。
- `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only`：exit 0，OmGameEditor Win64 Development `Result: Succeeded`／up-to-date；bridge／script DLL 已重建並 stage。没有启动 Editor／PIE。
- 隨後獨立 `--verify-staged-only`：exit 0；bridge build 與 Unreal stage SHA-256 皆 `79533b927239a6d82b222ca3a7463f4473417887eaec1bef4c8a20227b0b8bc5`。
- 獨立 Get-FileHash 核對 scripts/target/debug、scripts/base_content.dll、Unreal Binaries/Win64 三份 script DLL 皆 `c6c6c7c16c9b5b71b6b2043255fa5b08db2e815b4f7e868000a804cdaf868148`；沒拿舊 release headless DLL 當本轮证据，沒有改 legacy omb/scripts 副本。
- 地形 fixture 從實際英雄出生點設牆，正式輸入一次 MoveTo，不 teleport、不注入直接位移。必須觀察實際偏離直線且抵達、兩份權威 replay digest 一致、兩隊每 tick 對 fresh authority projection hash 相同、零 ComponentRepair、敵隊不收到自己的輸入、filtered 無 MobaMatch。

## 邊界與剩餘項目

legacy 靜態 f32 polygon 在幾何邊界量化為 raw Fixed64；判定用 i128，不靠 UE collision/navmesh。允許 raw 座標／半徑至 2^29，超界／非有限資料 fail closed，覆蓋目前 Lua map 的 ±100000 world-unit 限制。

這輪是英雄公開碰撞／導航前置，不是完整 5.4：Lua 三路地形宣告與合法出生／兵線可通行驗證、NPC／野怪共用避障、UE 三路 terrain layout、完整建築層級仍未完成。測試 metadata 是 fixture，不冒充真實 KCP 地形傳輸或 Unreal 顯示。沒有穩定 60FPS／tick wall-clock 效能驗收；既有 polygon 邊界量化在查詢時處理，後续應做靜態 cache 與熱路徑量測。

錯誤與修正見 `docs/plans/unreal-moba-error-register.md` E128。舊 E127 DLL／stage 不作為這輪最後建置證據。

本輪前置增量完成，總計仍 20/30；5.4 保留未勾選。下一段先做 Lua terrain 編譯／静態快取與合法路徑驗證，再接 NPC／野怪，而後 UE 三路公開 layout，不用反覆 Editor 重啟代替後端缺項。
