# 權威迷霧網格核心與資料契約

## 本批計畫

- [x] 從既有WaveBReadView的正式vision sources／occluders取樣，沿用共用LOS，不查renderer或hidden actor。
- [x] 固定點grid geometry：origin／cell size／columns／rows，最多4096格，驗證正尺寸與checked coordinate span。
- [x] 每隊獨立三態：0 unseen、1 explored、2 visible；新epoch明確清exploration。
- [x] FG01版本化row-major payload含team／epoch／tick／geometry，不包含source身分或hidden entity資料。
- [x] decoder先驗wire size、geometry／states／audience再配置；同tick冪等、倒退tick拒絕。
- [x] 本功能必要測試成功確認。

## 決策與限制

取樣是純權威呈現API，沒有塞入每個60Hz gameplay tick；待接server team-safe發布時設定節奏。它不改既有entity視野scope／override／stealth判定，也不以網格中心是否可見當成輸入target能力或披露gate。區域視野使用正式來源與LOS，但entity特定stealth／delay仍由原流程決定。

只有本隊sources參與。radius<=0忽略，極端距離以u128飽和平方避免overflow；LOS維持既有fail-closed。完整grid是row-major矩形，geometry含負座標且與位置無關，不依玩家位置推導map bounds。一次取樣後的immutable view tick不重新算，epoch必須advance才重設explored。

`omoba-core/src/runtime/fog_grid.rs`是可重用core，不是已串接的端到端fog。server安全發布、bootstrap／重連、runtime保持最新grid、ABI與Unreal繪製尚未接上，VISION N/A仍保留，不勾選完整6.1／6.2。後續需選compiled map配置的geometry與明確epoch生命週期，不可為了demo沿用固定10-unit或猜邊界。

## 當前確認

固定Rust1.95.0格式化；只執行 `cargo test --manifest-path omoba-core/Cargo.toml authority_fog_grid -- --nocapture`，3 passed／0 failed。驗證公開結果wire exact round-trip、team／epoch隔離、探索保留／reset、tick冪等／拒絕倒退、shared LOS tree blocking、radius非法／極端位置、所有truncated prefixes、未知version／state／trailing bytes、geometry容量／overflow。

diff check通過；未修改Unreal或ABI，未重建UE／重跑MCP／雙UE／60Hz長測。尚未將網格加入team stream或bootstrap，完整驗收留最後。
