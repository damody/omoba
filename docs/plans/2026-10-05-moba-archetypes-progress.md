# 三種 Lua 英雄原型與共用 FFI include（2026-10-05）

## 計畫與決策

1. 三種原型採近戰前排、遠程物理輸出、法術型；以現有正式單體傷害／自身治療效果建立四槽完整可操作的內容 loadout，不新增英雄專屬 Rust handler、C++ 或 Blueprint graph。
2. 新增 training_vanguard 與 training_ranger，沿用 training_luminary 作法術型。新角色、八個新技能全部附加在原有 apprentice 之後；舊 HeroId 1..4／AbilityId 1..16 不變，新 HeroId 5..6。
3. 共用 templates/moba_archetypes.lua 一次宣告英雄數值、四技能逐級資料、效果、Bot 意圖與學習順序。Rust／Unreal 模板以 ctx.include 取英雄及技能；混合對局配方讀同一 builder 的政策與學習，沒有另一份技能 ID 對照表。
4. 舊單英雄配方保持不變。新 opt-in moba_archetype_match.lua 讓 Top／Jungle 使用前排、Carry 使用遠程、Mid／Support 使用法術型；moba_archetype_single_player.lua 僅把 player1 標為真人。完整計畫含十名玩家、12個施法政策、48個學習步驟，由既有嚴格 Rust compiler 驗證。
5. 新原型不宣告 rust_module，技能走既有 generated generic_effect_ffi；native_only 讓 Unreal 直接使用產生的薄類別與共用 fallback，不製造空 Blueprint graph。新美術可沿既有配方替換，未宣稱本輪已製作美術。
6. FFI 產生器補通用 include：相對路徑、父目錄／絕對路徑拒絕、Windows 大小寫一致的循環检查、最多64層；builder 共用 context。build.rs 監看完整 templates 目錄，不能漏掉新相依檔。

## 當前功能確認

- omoba-template-ids hero_abilities 的 moba_archetypes 指定測試 1/1 passed：附加數值 ID、三種屬性／HP／普攻距離差異、四槽技能與逐級冷卻存在。
- base_content 的 moba_archetypes_four_skills 指定測試 1/1 passed：真正 SingleLaneConfig、Lua manifest、SimulationDriver 60 Hz、正式 PlayerInputEnum::CastAbility、權威 outcome 結算；三英雄各四招共12次，精確傷害／治療與冷卻。位置與零護甲為明示 fixture，不是完整 match／filtered parity 或 Bot 平衡驗收。
- test_hero_registry_includes.lua 5/5 passed：同一 include 產生 FFI registry 與 ID；父目錄、絕對路徑、直接／大小寫循環均拒絕。
- 固定 Lua launcher 對新單真人混合配方 --prepare-only 成功，正式 Rust 設定解碼／catalog／控制驗證產生一真人＋九Bot、60Hz；沒有 game／IPC sockets 或 Unreal 程序。
- Unreal codegen 產生11檔、讀17個 Lua inputs；隔離輸出 --check 成功後，同一內容正式生成至 omfue/Plugins/OmRuntime/Source/OmGenerated。包含 AOmHeroTrainingVanguard／AOmHeroTrainingRanger 與沒有 Blueprint 相依的資產配方。
- codegen content_hash=d197946331c9476f，catalog_identity_hash=58136a29dddae4af，catalog_data_hash=755482ca66dd110b。root／omfue diff --check 通過，未提交或清理既有工作樹。

## 使用入口

```text
tools/lua/lua.exe scripts/run_moba_role_ue.lua --prepare-only --recipe scripts/lua_data/moba_archetype_single_player.lua
tools/lua/lua.exe scripts/run_moba_role_ue.lua --recipe scripts/lua_data/moba_archetype_single_player.lua
tools/lua/lua.exe scripts/run_moba_headless.lua --help
tools/lua/lua.exe scripts/test_hero_registry_includes.lua
```

正常 Unreal 指令會走既有建置／stage guard，不可直接拿旧 DLL 啟動新內容。此輪只生成 C++，尚未重新編譯 OmGame 或 stage 新 DLL；目前磁碟上的舊執行期產物不是本次原型的完成證據。

## 尚未完成／不能冒稱

- 位移、控制、護盾、隊友治療、區域技能未加入通用 effect；近戰恢復不是護盾，連射仍是單體物理傷害。Mana 仍是 metadata，沒有虛構扣魔規則。
- 未驗收三原型整場死亡重生／filtered／Unreal 圖像／Bot 策略平衡、100場 headless、最後全套建置與端到端。此批完成的是可操作內容與生成接線，不是5.5完整封關。
- OpenSpec 維持20/30，完整驗收集中最後；錯誤與通用修正見 E165。
