# 主動物品建置期作者契約（2026-10-05）

## 計畫與決定

1. 在現有 moba_items 作者模型增加可選 active 與 cooldown，不建立另一份外部 JSON 或遊戲期 Lua。
2. 五種效果採 kind 閉集與 deny_unknown_fields；有限正量／時間／比例與冷卻在生成前拒絕，限制與核心一致。
3. 生成 MobaItemActiveConst 的 Fixed64 常數與 cooldown，ItemRegistry::generated_moba 不再硬寫 active=None，而是轉接共用 native ActiveEffect。
4. 被動商品省略 active=None／cooldown=0 序列化欄位，保留舊 canonical 商品資料。現有四商品不增刪、不改平衡；尚不部署新主動商品。

## 作者方式

```lua
{ catalog_id = 5, id = "example_shield", name = "Shield", cost = 500,
  active = { kind = "shield", amount = 100, duration = 3 }, cooldown = 10 }
```

其他合法 kind 為 sprint_buff（ms_bonus／duration）、restore_mana（amount）、damage_reduce（percent／duration）、headshot_next（bonus_damage）。上例是作者說明，不是本輪已加入的商品。

- 量範圍 1/1024–1000000，移速上限10000，減傷比例上限1，時間上限60秒。
- 冷卻為0或1/1024–3600秒；0沿核心無冷卻契約。被動商品不得有非零冷卻。
- 本輪仍使用既有 native f32 ItemConfig 轉接契約，不冒稱整條物品執行鏈已改成純 Fixed64。

## 局部確認與問題

- `cargo test --manifest-path omoba-template-ids/Cargo.toml --features runtime-lua-content --lib moba_item -- --nocapture`：2/2 通過。涵蓋真正 Lua table 解析、五種效果的 Rust literal／量化、非法量／時間／冷卻／未知 kind／欄位、active hash敏感與被動序列化不變。
- 正式 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib --no-default-features --features kcp,compiled-content-only legacy_item_adapter -- --nocapture`：3/3 通過，五種 fixture 從生成 enum 轉為 native 效果，驗證實際提交與冷卻、非法准入及正式模式雙入口拒絕。
- 上述 opt-in feature 只為作者解析器單元測試；正式後端另以 compiled-content-only 編譯，不將 runtime-lua-content 加入遊戲或啟動流程。
- 首次 Cargo 從 omb workspace 選外部套件加 feature 失敗；改用套件自己的 manifest。新作者測試漏匯入 canonical_template_hash 導致2個 E0425，補上測試 import 後通過，未改 production 雜湊算法。詳見 E262。

## 待辦

- 接續主動商品設定／UE C++ 通用 metadata、冷卻與盾量 HUD；本輪未聲稱 C++ 主動物品生成或全對局驗收完成。
- 保持 Lua 建置期、Rust／Unreal native 執行方向，不維護 omfx、不部署 DLL、不跑全驗收。大項進度仍21/31。
