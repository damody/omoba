# 正式生成主動商品（2026-10-06）

## 計畫與決定

1. 在既有 moba_items Lua 來源追加 ID5–9，保留1–4被動商品、價格與配方；總計九件，不重排身分。
2. 五種已支援主動效果各提供一個原創訓練商品；不綁英雄／專屬C++／Blueprint graph。數值是資料驅動初版，不宣稱競技平衡完成。
3. 通用核心 registry／購買／背包／使用與原生 metadata 沿現有生成鏈取得內容；驗證不再用 from_configs 替換正式效果。

| ID／商品 | 價格 | 主動效果 | 時間 | 冷卻 |
|---|---:|---|---:|---:|
| 5／Guard Charm | 500 | 吸收100傷害 | 3秒 | 12秒 |
| 6／Stride Charm | 500 | 移速＋60 | 3秒 | 12秒 |
| 7／Mana Charm | 400 | 回魔100，容量上限內 | 即時 | 12秒 |
| 8／Ward Charm | 600 | 共用packet減傷25% | 2秒 | 15秒 |
| 9／Strike Charm | 600 | 下一次正常普攻＋60 | 下次launch消耗 | 10秒 |

Mana Charm仍需英雄有既有受管理魔力池，未開啟魔力規則時不隱式建立池。所有商品都占一格，並非消耗品；效果／刷新規則沿共用執行器，不新增假動畫或改基礎數值。

## 當前功能確認

- `generated_active_shop_items_60hz_buy_use_real_catalog_without_effect_overrides`：1個測試、5種商品情境通過。正式 match setup 的生成 registry、正式60Hz PlayerInput買入／使用、扣款／槽位、真正盾／限時Buff／回魔事件／一次性增傷、其他owner不受影響、CD拒絕且不修改CD。
- 指令：`cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only generated_active_shop_items -- --nocapture`。
- 首次測試回魔期待110，實際raw112810而期待112640，漏算自然回魔；改驗證物品權威ManaGained精確100與池容量界，不關閉／延後正式自然回魔。修正後通過，詳見E265。
- Unreal生成17檔／17輸入，九件 metadata 已生成，`--check`通過。相關模組4 actions／7.14秒 Result: Succeeded，日誌 `omfue/Saved/Logs/native-active-shop-modules-20261006.log`。

## 版本與待辦

- identity保持 `ff3ef5e2957aa89f`；data由 `26f34a124129cc46` 變 `2df5b6b1e02d95d7`；presentation由 `e8bdc0929625fdd2` 變 `3634f807282b0515`。真正來源變更必須變hash，不能偽裝與舊部署相容。
- ABI14／wire5／IPC4不變。未完整重建／stage server、client runtime、bridge或release script DLL；最後統一建置部署，舊binary錯配仍需拒絕。
- 未跑PIE、真實按鍵、完整LAN／對局或效能驗收，不把核心正式商品案例當整套前端已可玩。
- 護盾餘量安全投影／HUD、最後端到端權威使用結果與其餘MOBA框架待辦繼續保留，整體21/31。無runtime Lua，不維護omfx、不提交推送。
