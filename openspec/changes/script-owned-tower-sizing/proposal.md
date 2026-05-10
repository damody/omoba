## Why

目前 tower visual size 與 placement radius 仍可能被前端或後端用 multiplier、clamp、`render.size / 2` 這類程式規則推導，導致 content 作者必須反向猜測程式內建規則，甚至可能為了視覺大小把 gameplay footprint 設成不自然的值。

Tower 的長期基準大小、放置半徑與相關視覺尺寸應該由 scripts content mod 明確宣告；前端只負責 render-time 換算與短暫動畫 scale，後端只消費 content metadata 做 authoritative placement validation。

## What Changes

- 將 tower sizing 拆成 script-owned metadata：至少包含 combat visual size 與 placement radius，不再由程式使用 `/ 2`、multiplier、clamp 或 footprint fallback 推導。
- 更新 `scripts/lua_data/templates/towers.lua`，讓每座 shipped tower 明確宣告 visual size 與 placement radius。
- 更新 `omoba-template-ids`、`scripts/script-abi`、`scripts/base_content`、`omb` 與 `omfx` 的 metadata chain，傳遞新的 explicit sizing fields。
- 更新 omfx composite tower rendering，使用 script-provided visual size 作為 base sprite/barrel/body 的長期基準尺寸；recoil/buff/attack animation 只在此基準上做短暫 scale。
- 更新前後端 tower placement validation，使用 script-provided placement radius；不影響 runtime `CollisionRadius`、attack range、projectile spawn、damage、cooldown 或 lockstep combat state。
- 移除或拒絕程式內建 sizing fallback 規則；缺值應在 content/codegen validation 階段失敗，而不是 runtime 靜默推導。

## Capabilities

### New Capabilities

- 無。

### Modified Capabilities

- `tower-composite-combat-rendering`: tower combat visual size 與 placement radius 必須由 scripts metadata 明確提供，omfx 不得用程式公式推導長期基準尺寸。
- `sim-snapshot-rendering`: `SimWorldSnapshot.tower_templates` 必須 expose explicit script-owned tower sizing fields，供 omfx render 與 placement preview 使用。

## Impact

- `scripts/lua_data/templates/towers.lua`: 新增每座 tower 的 explicit sizing metadata。
- `omoba-template-ids`: codegen model、generated const、validation 與 tests 需要更新。
- `scripts/script-abi`: `TowerRenderMetadata` 或等效 ABI-safe metadata 需要新增 sizing 欄位。
- `scripts/base_content`: metadata builder 需要傳遞 sizing 欄位。
- `omb/src/comp/tower_registry.rs`、`omb/src/state/initialization.rs`、`omb/src/state/resource_management.rs`: registry 與 placement validation 需消費 explicit placement radius。
- `omfx/game/src/sim_runner.rs`、`omfx/game/src/lib.rs`: snapshot 與 composite render cache 需消費 explicit visual size 與 placement radius。
- `scripts/base_content/assets/towers/README.md`、gen-docs 與 OpenSpec specs 需更新 sizing contract。
