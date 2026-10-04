# Lua 野區營地／Rust 權威／60Hz 增量

## 本輪計畫與結果

- [x] Lua 配置兩個單怪營地，數值與座標只接受有界整數；拒絕重複 ID、未知欄位、浮點或非法範圍。
- [x] Rust 產生 MobaJungleConst，納入 compiled map catalog 與完整內容 hash；沿原本重建／拒絕 hot reload 契約。
- [x] 正式 ECS 受擊仇恨、追擊、超距／英雄死亡回位、回位免傷、到家走正式 Heal、15 秒 active-time 重生。
- [x] first-lethal 一次性 Gold 60／XP 90，不計英雄 KDA；匿名傷害、NPC 來源、forced Death 不付款，擊殺者死亡／重生保留獎勵。
- [x] 舊 Neutral 不變；新增 HostileNeutral／安全 kind=3，filtered 重建相同陣營分類，Unreal bridge 使用通用 creep fallback。
- [x] 三 seed 60Hz 正式 PlayerInput／雙 authority 每tick replay／双隊 filtered hash，零 component repair；完整回歸與最後 build-only／stage SHA 核對。
- [x] 最後補強後實際 release DLL headless 三路 seed42，22745tick 勝利／全tick replay；獨立確認兩個本輪 headless PID 已退出。

## 決策與邊界

Lua 原始資料在 `scripts/lua_data/templates/moba_maps.lua`；生物 AI 在 shared Rust `omoba-core/src/runtime/native/moba_match/jungle.rs`，只有權威建立 MobaMatch。不新增角色專用 C++ 或 Blueprint graph。營地不占第三玩家 team／lane index；team0、Vision scope、沒有 VisionSource。私有 aggro target、回位狀態、respawn deadline 不傳客戶端；可見運動與已結算生命走現有 ordered facts。客戶端沿既有 MOBA authority-combat settlement，沒有私有野怪 AI。

回位免傷同时在 Damage 與 ScriptDirectDamage 入口檢查。到家以正式 Heal 復原 CProperty，生成 committed vitals。僅正傷害產生仇恨；死亡先 retire 舊 entity，再按 active elapsed 排重生，不把 HP0 當即時換代。重複 lethal／Death 不重付；same-tick killer death 的 XP／Gold 會在死亡快照／重生保留。

營地先採各一隻、無 Buff 的對稱基礎 Guardian（座標800,700與1600,-700、HP450／Damage30／MS260／Range130／Leash450／攻擊間隔1秒）。新增營地只改 Lua，不需改 AI 或 Unreal 程式。尚未把營地 ID 接成專用 art recipe；當前 bridge 以 practice_dummy fallback 呈現，不宣稱完整野區美術可替換管線。

## 最後測試

| 驗證 | 結果 |
|---|---|
| base_content 完整 | 最後 Lua filtered settlement 補強後105 passed，195.30秒 |
| omoba-core lib | 最後335 passed，包含Lua直接傷害不重算的回歸 |
| server lib | 156 passed／1 benchmark ignored |
| client-runtime | 61 passed／8 capture ignored；另3整合通過 |
| bridge | 53 passed／1 ignored；另2整合通過／1 ignored |
| template runtime-lua-content | 38 unit＋23 generated＋8 hero＋2 catalog通過 |
| 野區正式輸入60Hz | seed1／42／539365380 各3000tick、6000雙隊steps、230 external effects、回位／擊殺／一次重生；總18000雙隊steps零repair、每tick與fresh bootstrap hash一致 |
| 三路完整含營地60Hz | seed1／42／539365380勝利tick4955／7930／7776，total4969／7944／7790，雙隊9936／15886／15578＝41400steps一致；終局後15 frozen ticks |
| codegen --check | 11files／16Lua inputs，art content_hash仍3b296ff5bdbcd7d9；full catalog_data_hash更新888adbf86bb59730 |
| Unreal最後build-only | 最後命令exit0（restart檢查UBT status）；bridge built／staged SHA7d934b6f986d5fcf3cbf6dd3cb19c6466aa728f656990548c7ed995bbb9e209e，獨立verify通過 |
| Debug script stage | scripts/target/debug、根scripts/base_content.dll、Unreal stage SHAae61a98aa4c8c50abb8a2ae24015c41bd5e9e103b4ad3f7d926a7604afc793a9一致 |
| 最後 release DLL headless | seed42／60Hz／three_lane_training／withdraw；22745tick勝利、379.0830078125遊戲秒、48waves、四槽46／6／5／4、14791combat facts、end1、全部22745tick replay一致，己方3塔存活／敵方3塔retired |

最後 headless 由固定 Lua 入口 `scripts/run_moba_headless.lua` 執行，明確從 `scripts/target/release` 載入DLL；host是debug moba-headless，不是release host效能基準。最後 release DLL SHA `45ea50f8531d21e80abdbbe99ccf28d02e01cebf8defcc02af6715c45493bac9`；報告 `omb/target/moba-headless/jungle-three-lane-60hz-seed42.json` SHA `5daf5a0497f404890ca0346189bf773e2d3e4d13cbe406d9db469a596b2f6845`、final digest `f498812409ac96a11860c1aa34fd0f73c3492d0fa8f91df8ecf8560818681747`。最後補強後再跑與中間版 gameplay 報告相同，但 DLL SHA 已改，不混用舊 binary。PID47076／42432 都正常結束、最後 CIM 無對應程序，無須強制停止。

這輪未啟動 Editor／PIE、未使用新的 MCP Blueprint 操作，也沒有新增真實 KCP 野區 gameplay 驗證。60Hz 指 deterministic simulation profile，不是實測 Unreal 60FPS 或網路 frame-time 保證。build-only 既有 precompiled BpGeneratorUltimate dependency warning 不代表動畫匯入已驗證。

最後 review 補上 ScriptDirectDamage 與一般 Damage 相同的 filtered authority-settlement gate，避免客戶端重做匿名Lua傷害。所有受影響 core／base／server／runtime／bridge 重跑後才最後 stage；此前222306…／26c588…只是中間版本。共享 UE engine UBT Log.txt 在檢查時已被別的 OpenKoikatsu build 覆蓋，不能拿它的 Failed 當 OmGame 結果；只採本輪 om_restart 捕捉的執行 status，不操作別的專案。

## 尚未完成與下一段

OpenSpec 仍20/30；5.4 不勾選。接下來先補 Lua 地形／通用避障及三路 public layout→Unreal 地圖／小地圖，再補多層塔與大型野怪 Buff／專用美術 recipe。5.5 五位置合法視野 Bot／100場、6.x 完整 Unreal UI／LAN／效能驗收仍獨立未完成。錯誤、重送前搖與 stage 不一致的防重犯記在 `docs/plans/unreal-moba-error-register.md` E127。
