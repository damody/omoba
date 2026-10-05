# 共用施法等級解析

## 問題與決定

- 原本 managed Mana、handler 與成功 cue 各自取技能等級。無 Mana 路徑使用 `as u8`，257 可能變成 1，導致錯誤等級仍執行，而 cue 記錄另一個值。
- 以共用 `resolve_cast_rank` 在 handler／資源變更前解析一次。正等級使用 checked conversion，拒絕超過 u8 或技能宣告 max_level；正式 MOBA 與 managed Mana 另要求已習得且存在對應 level data。
- 結果同時提供 handler level 與 presentation rank。Mana 成本／冷卻、handler execute、成功事實不再各自取值。拒絕時不執行 handler、不啟動技能冷卻、不發布施法 visual／Ability fact。
- 舊 Story／TD 無 Mana 路徑的未習得 fallback 保留：handler 使用既有 level 1，呈現 rank=0 明確未知。正但異常的等級不 fallback、不溢位。正式 MOBA 的零／負等級仍拒絕。

## 計畫調整：實際效果位置

調查現有 AbilityScript.execute 只回傳成功／失敗；效果透過 GameWorld 方法提交。它沒有回傳已解析的實際效果位置，且 dash destination 可經 collision 修改、area center 與 projectile impact 時機不同。因此目前不把 SkillTarget.Point（請求）冒充實際效果位置，也不從 renderer 目前位置推測。

後續位置契約須由效果執行端提供已解析結果、來源與發生時機，再交既有 team projector 決定可公開欄位；不可直接以可見 caster 放行任意 hidden target／impact 資訊。本批先修正已確認的等級一致性缺陷，位置契約尚未實作，不宣稱完成。

## 局部確認

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only checked_cast_rank -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only successful_cast_facts -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only cast_visual -- --nocapture
```

- 新 checked_cast_rank：1/1，內含正式／舊模式 × managed／無 Mana × 九種等級，共 36 組真實 generated handler／dispatcher 情境。
- 測試包含 -1、0、1、3、5、255、256、257、i32::MAX；檢查成功 visual／fact rank、拒絕不扣 Mana／不啟動技能冷卻，正確保留 legacy fallback。
- 相鄰成功施法事實 3/3、cast visual 2/2。僅本批指定功能確認，未跑全套／Unreal／LAN／部署或最終驗收。既有三項 build script dead_code warnings 未改。
- 初輪測試 E0609：把 FactOrderingKey.fact_kind 寫成 kind，改正後重新編譯，新矩陣與相鄰指定測試全部 exit 0；失敗 executable 不算成功證據。
- OpenSpec 仍 21/31；6.1 及完整其他項目維持待完成，不新增角色分支或 runtime Lua，不維護 omfx。

防錯細節見 E223。
