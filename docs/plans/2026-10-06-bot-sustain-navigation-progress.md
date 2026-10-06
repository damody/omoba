# 五位置 Bot 撤退導航（2026-10-06）

## 問題與決策

接續 OpenSpec build-unreal-rust-moba-framework 的5.5玩法，原sustain Retreat不查公開導航，直接提交回基地MoveTo並continue；地形封住基地時可能反覆無效旅行、阻斷合法自保技能，舊攻擊也沒有停止。

- 五位置共用正式static_next_waypoint，以本人CollisionRadius與公開BlockedRegions檢查完整路徑；採既有有界搜尋，不新增另一套導航、不把partial path當到達。
- 未找到路徑時進入recovery_wait，封住進攻意圖但允許自保技能；無可用自保時正式HoldPosition取消舊命令，已Hold且無queued去重。
- 不讀隱藏敵方位置、仇恨或狀態，不直接改HP／位置／command；地形恢復下次think重新查路，以普通MoveTo撤退，不建立永久黑名單。
- 定身與回城受控的原等待語意保持，不藉此覆寫定身命令；合法旅行與基地恢復優先保持。
- 局部fixture遍歷Top／Mid／Carry／Support／Jungle，分有／無恢復技能，檢查正式60Hz技能效果、無進攻冷卻、Hold去重與普通移動恢復。不把命令accepted當同tick已移動。

## 局部確認

- `cargo test --manifest-path scripts/Cargo.toml -p base_content role_bot_sustain_navigation_60hz -- --nocapture`：1 test通過，內含五位置×有／無自保的10組正式60Hz矩陣。
- 相鄰局部案例各1通過：`role_bot_rooted_sustain_60hz`、`role_bot_sustain_recall_and_authoritative_base_recovery_at_60hz`、`role_bot_recall_control_60hz`。合計4 tests通過，未以filtered out的其他案例冒充已跑。
- 既有template build script三個dead_code warnings保持，未更改無關TD碼。
- 不跑100場、Unreal全驗收或stage，不更改ABI／生成hash，不維護omfx；本輪Rust test build不代表release DLL已重新部署。

## 錯誤與防再犯

- 本輪再次將Windows literal glob傳給rg造成os error123，改rg --files找真實檔案，再讀comp/phys.rs。不可把這種工具錯誤算成測試失敗或成功。
- 新fixture最初猜AttackTarget欄位為last_known_pos，編譯前核對實際型別並改chase_origin；先核對宣告再寫fixture，不改production契約迎合測試。
- 首次編譯fixture讀MobaMatch.routes私有欄位出E0616，改從正常Bot MoveTo取得目的地，不開放私有欄位供測試。
- 首次執行fixture假設spawn已附HeroCommandQueue，get_mut unwrap失敗；改明確insert舊攻擊及queued command作fixture，正常Hold由權威輸入清除，不改production spawn。
- 詳見防錯紀錄E294；整體21/31，完整5.5未勾選。
