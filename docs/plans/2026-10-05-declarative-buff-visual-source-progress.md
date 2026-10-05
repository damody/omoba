# 宣告式效果的通用公開 Buff 映射（2026-10-05）

## 計畫與決定

1. 在 Lua Buff 資料宣告可選 ue.buff_visual.sources，不新增每技能 Rust／C++ 判斷或 Blueprint graph。正式遊戲只使用編譯 lookup，Lua 仍只在建置時執行。
2. 共用 omoba-content-model validator 驗證來源存在且非 tombstone、kind 與實際效果相符、Mana stat 精確相符；每 Buff 至多 32 來源，同一來源不得被重複／衝突映射。Rust template generator 與 Unreal codegen 使用同一驗證入口。
3. script-abi 新增共用減速 key writer 與 canonical 私有來源 parser，沒有修改 StableAbi 型別布局。Rust安全狀態只映射合法已生成來源，不公開內部 key／caster／payload。其他不合法或未宣告的動態 Buff 仍不披露。
4. 對同公開 ID 的來源以剩餘時間最大值合併，無限期優先；每次 committed 快照重算，使部分來源移除後維持剩餘效果，最後到期才清除。不修改實際減速 strongest-family 或資源規則。
5. 現有 ranger_shot 綁定 slow；append-only 新增 mana_regeneration、mana_capacity，分別綁定 ranger_patch 的 mana_regen_constant 與 vanguard_recover 的 mana_bonus。新增資料由既有 Unreal Buff 模板生成，缺非必要資產仍使用通用 fallback。

作者範例（來源必須符合該技能已宣告的實際效果）：

```lua
ue = { buff_visual = { sources = {
  { ability_id = "ranger_patch", kind = "mana_buff_self", stat = "mana_regen_constant" },
} } }
```

## 局部確認

- model1：未知來源、錯效果／stat、額外欄位、重复衝突與 tombstone 拒絕。
- ABI1：兩種實際 writer→parser，以及非法／溢位／模糊 key 拒絕。
- core7：包含新增來源合併、部分移除／全部到期、公開資料無 key 或 payload，既有 owner-team、codec、更新與版本拒絕。
- base_content 正式 compiled-content-only、正常生成 handler 60Hz 2/2：三原型資源技能的公開 ID／6秒／空payload；真實多減速來源的單一公開 ID。不是用手寫假 handler 取代正式來源。
- bridge 最終編譯後 2/2 通過：安全 component 消費與不重播 Added／toggle。共 13 個不同的局部 Rust 測試通過；OpenSpec strict 與主 repo／omfue whitespace 檢查通過。
- Unreal codegen 15 檔生成與只讀 --check 通過；固定 Lua 5.4 Buff 作者檔語法確認通過。
- 保留 -NoEngineChanges 的三 project modules scoped native 編譯12 actions成功（9.71秒），日誌 omfue/Saved/Logs/declarative-buff-source-modules-20261005.log。未執行新 native automation 或畫面；E224 BuildId 仍不相容。
- 生成 content_hash 6460930eee2ba926；完整 catalog_data_hash 29940eea6845d75b；英雄／技能 catalog_identity_hash ff3ef5e2957aa89f 不變。旧部署資料需正常重建後才可開局，不能 stage 舊 DLL 冒充新結果。
- 不改 C ABI／BVS1／IPC4／selective wire3，不維護 omfx、不跑全套或雙UE完整驗收。整體 21/31，6.1／6.2與完整來源政策仍待最後整合。

防錯紀錄：unreal-moba-error-register.md E232。
