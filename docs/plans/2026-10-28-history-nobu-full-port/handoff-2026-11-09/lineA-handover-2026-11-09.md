# A 線交接（2026-11-09 停工）

範圍：產生器（v3_rust.lua）、v3_bot、gen_item_v3 的上架輸出、v3_summons、hero_author，以及道具和 D22 變體英雄的產生器缺口。

## 已完成的 commit（都已 land，沒有 push）

| commit | 內容 |
|---|---|
| 9b7bf2d8 | item_instance 狀態 bool／unit 讀寫（月輪鏡、靈都宮燈） |
| 94f671d9 | select／aura types building+hero → HeroesAndBuildings（真視寶石・英雄） |
| c38bd3db | 道具一律以 v3 id 註冊（侍的徽章） |
| a9ae7133 | 道具施放內的 refund mana（魂蛇） |
| 1b8af10c | projectile.lifetime → projectile_expiring（浪人十手） |
| 38f10132 | NotPrimary、damage type event、道具 orb_slot release=unequip（不敗大劍、暗黃之牙、鞍切） |
| 9ef90d98 | 道具充能（gen_item_v3 輸出 charges），石牆 collision 80（土牆之書） |
| 73478294 | 道具前搖 cast（gen_item_v3），item_shop_v3 有 28 件帶前搖 |
| e5f6446c | courier tag、道具非英雄目標 exclude_heroes、unit range 0（千里大劍） |
| 8c11d5ca | 巨龍、扇子的 bot 改成作者指定 enemy_line；v3_rust 道具 active 可寫 bot |
| b8e4f3c0 | v3_bot 新 intent ally_anchor_unit（千里大劍），item_spotlight AIM |
| 5f232ab3 | 信使 summon courier（馬）、SummonStats.courier／magic_immune、技能殼 courier_deliver／courier_return、空程式參數 _cx |
| a1b7a6a0 | hero_author 照實發布 unit_or_point（7 個技能） |

現況（live）：
- 英雄 192 個、868 條目，產生器阻擋 0。
- 道具 170 件，沒有產生器或 B 層阻擋。
- 原作 158 件都已上架；另外 10 件依 D13 不移植。

## 未完成

1. **unit_or_point 技能補 unit_target／targets_heroes_only**
   - 等 B 的 or22：content model 讓 unit_target 也接受 unit_or_point。
   - 做法：scripts/hero_author/v3.lua 加 `TAKES_UNIT = { unit = true, unit_or_point = true }`，unit_target 和 targets_heroes_only 都改用它，再重生 v3_test_heroes。
   - 補丁在 target/lineA/uop_patch.lua（第一版，含 TAKES_UNIT）。
   - 影響 7 個技能：a06_flame_of_anger、a32_asura_blade、c10_black_dragon_wave、b08_old_fang_slash、b08_old_sky_fang、b04_old_flame_blade、a14_old_fire_phoenix。
   - unit_target 只影響 UE 目標呈現，host 敵我由產生程式的 target 檢查。
   - content hash 會變：C 正在補錄這 7 個技能，land 前要先問 C。
2. **主機端（B 線）尚未實作：**
   - ally_anchor_unit 的 bot 執行。千里大劍的 bot 在這之前不會使用它。
   - 信使的 courier_deliver／courier_return 行為，以及生成時 grant。B 和 E 對接中。

## 影子與工具

- 影子：target/lineA/host。
  - 同步用 `sh <scratchpad>/sync.sh`：會覆蓋 v3_rust.lua，也不會複製所有測試檔。
  - 影子少 docs/plans/.../source-catalog.lua，所以 hero_source_index roster --check 在影子會失敗，要在 live 跑。
- cargo target：target/lineA/cargo，指令加 CARGO_PROFILE_DEV_DEBUG=0／CARGO_PROFILE_TEST_DEBUG=0。結束時可用 cargo_target_retention 清掉。
- 常用腳本（target/lineA）：
  - item_all.lua：道具阻擋清單
  - hero_all.lua：英雄阻擋
  - bot_scan.lua：scripted 主動道具的 bot 策略主機能不能用
  - bot_cmp.lua：英雄 bot 推導比對
  - temp_list.lua：影子暫加上架列
  - crlf_file.lua：item_shop／texts 行尾要 CRLF
  - land 目錄 target/lineA/source/{new,base}、manifest.json

## 已知問題與教訓

- 送 land 前先在影子跑 land 的全部檢查：
  - generator --check（v3_test_heroes、v3_summons、gen_item_v3、live 的 roster）
  - lua_content_check
  - template-ids 測試
  - items_impl 與相關 hero_*_impl 的 --lib 測試，items 有 deny(warnings)
  - 信使那批因漏檢 land 了四次。
- gen_item_v3 直接寫出 LF，live 的 item_shop_v3／item_texts_v3 是 CRLF。
- 用 live 當 base 前要確認 live 從同步後沒變；否則重新同步再套補丁，避免把別人的變更當成刪除。
