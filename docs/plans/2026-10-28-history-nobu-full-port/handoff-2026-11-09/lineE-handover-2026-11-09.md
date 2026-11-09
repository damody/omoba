# E 線交接（2026-11-09，使用者要求停工）

所有工作都沒有 push。停工時沒有尚未提交的檔案，omb submodule 也是乾淨的。

## 已完成的 commit

**道具上架：** 158 件來源道具全部上架。

| commit | 內容 |
|---|---|
| a2b5e61d | 月輪鏡、靈都宮燈（246–247） |
| 4bbb5203 | 洞察鏈與御魂（248–252）；v3_vocab ITEM_SCRIPT_STATS 加 vision_night |
| e7d5544e | 浪人十手、禦神刀、闇牙黃泉津、鞍切（253–256）；gen_item_v3 的 only_in_a_status 比對帶號 kv |
| 88b0a753 | 巨龍（257）；gen_item_v3 加 unit_or_point 目標 |
| 9c6ec3e3 | 真知短甲、真視赤備挂甲（258–259）；ITEM_SCRIPT_STATS 加 physical_block 與 physical_block_chance |
| c6b1807d | 忍法．土遁（260） |
| 80906d0e | 陸奧守吉行．銘刀（千里大劍，261）；item_sources_check_test 處理缺口列已全部上架的情況 |

A 線列為只差上架的另外 10 件，原作沒有 KV，依 D13 不上架：shintogo_kunimitsu、ancient_doll、ghost_gloves、gold_to_prestige、hachiman_bell、ice_book、kat_steel、ksitigarbha、magic_ring、poisonous_ring。

**信使：**

| commit | 內容 |
|---|---|
| 3f2c2c92（omb c98b1c9） | 開關 shop_range_rule，預設關：game.toml MATCH_SHOP_RANGE_RULE 要和 role plan 一致；comp::STASH_SLOTS = 6 |
| f8fecbbb | 信使生成與立即重生：moba_match/courier.rs；commit_summoned_spawn 依 courier 旗標設 UnitType::Courier，依 magic_immune 旗標套常駐魔免；含 replica soak |
| ef0b7593 | fix-candidates FC-187：信使移速 550、無碰撞 |

其他線相關的 commit：
- A 5f232ab3：courier 模板，含送貨、回家兩個技能殼。
- C：馬的模型 d88e98b4／84e8fb33；外觀與小地圖 4511686e。

## 信使規格（courier-spec-2026-11-09.md）各節進度

| 節 | 狀態 |
|---|---|
| §0 開關 | 完成，預設關。 |
| §1 信使單位 | 生成、重生、賞金、魔免、FC-187 都完成。泉水 1200 內物理免疫還沒做，歸 B。 |
| §2 stash | B 進行中，or21（第一部分、協定 v19）停工時正在 land。 |
| §3 送貨、回家 | B 要等下面的 E 未完成修補 land 後，才接在 SummonOrder operation 3 上。 |
| §4 bot | B，尚未開始。 |
| §5 UE | C 已完成模型、外觀列、小地圖；stash 介面要等 B 的欄位。 |
| 實機驗證 | 尚未開始：千里大劍點信使要中止、bot 會用信使、預設模式打開開關。 |

## 未完成：E 的修補已寫好，沒有 land（停工時在排 build 鎖，已取消）

**內容：** C 回報信使（UnitType::Courier）在幾處被當成非召喚物，修法是那幾處都改成 Summon|Courier：
- snapshot.rs:1137 與 :1648、visibility.rs:1380：render kind 4。
- summon_orders.rs:54：主人的 SummonOrder。
- filtered_specs.rs:1043：replica 的 SummonOrder。
- filtered_specs 還原 ScriptUnitTag 之後，summon 帶 courier 旗標就還原成 UnitType::Courier。
- courier.rs 加測試 the_owner_moves_its_courier_like_a_summon。

**測試結果：** courier 4、summon 126、snapshot 18、filtered_specs 41、visibility 26 全部通過。

**檔案位置：**
- 影子：D:\omoba\target\snapshots\ecourier3\tree。
- land 用的新檔與 base：C:\Users\User\AppData\Local\Temp\claude\D--omoba\99dbfc1f-e68f-4215-a95f-2a039b8382af\scratchpad\e_courier3\new 與 \base；base 是當時的 live，HEAD ef0b7593。

**影響：** 沒有這個修補，UE 對不上信使的外觀，信使也不能接受指揮，B 的 §3 會被卡住。復工時重新做 3-way land 即可（命令與 commit 訊息在 scratchpad e_courier3\msg.txt）。B 已同意這幾行由 E 修。

## 驗證結果

- **cover8／9／10／12／15：** 新道具都有買到、也有使用，headless 的 observer 都是 0。
- **千里大劍：** 買到 7 次、用了 0 次。B 的 07fa0d1e 加了 ally_anchor_unit，修正之後還沒重驗；cover17 已取消。
- **巨龍：** cover15 用了 130 次（enemy_point）。A 的 8c11d5ca 改成 enemy_line、B 的 9ceb5d9e 讓施放時轉向目標，這兩項之後還沒重驗。
- **observer：**
  - cover13 是充能 mismatch，B 的 50423af0 已修。
  - cover14 與 cover16 是買下道具時沒有同步初始充能，B 的 07fa0d1e 已修，但修正之後還沒重驗。
- **禦神刀：** 使用次數是 0，這是正常的。它的主動在文件裡是 kind passive，冷卻只用於被動觸發。

## 已知問題

- **verify_m6_batch --split 會靜默失敗：** split 子程序的輸出沒有被保存，cover11 和 cover14 都只看得到 NO AUTHORITY LOG。直接跑 run_moba_split_local 會成功，可能是建置逾時。
- **gen_item_v3 寫出 LF：** live 的 item_shop_v3／item_texts_v3 是 CRLF，目前每次都要手動轉換。之後可以讓工具沿用原檔的換行。
- **cover9 的效能：** 有一個 tick 是 58 ms，其中 towers_scripts 佔 52 ms，只出現過一次，沒有追查。
