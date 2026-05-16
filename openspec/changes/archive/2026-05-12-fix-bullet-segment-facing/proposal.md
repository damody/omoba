## Why

目前子彈線段特效沒有穩定對齊發射點到敵人的方向，導致射擊起始時可能偏離目標，且飛行期間有時會因持續重算朝向而旋轉。這會讓攻擊回饋看起來不可信，特別是在高速或多塔同時射擊時更明顯。

## What Changes

- 修正子彈線段產生時的朝向計算，讓線段從發射點正確面向目標或命中點。
- 將線段的飛行朝向固定為建立時的方向，避免飛行中因目標移動或追蹤更新而旋轉。
- 保留現有子彈線段的視覺生命週期、速度、命中與清理流程，不改變 gameplay 判定。
- 新增或更新驗證，覆蓋起始朝向與飛行中不旋轉的行為。

## Capabilities

### New Capabilities
- `bullet-segment-facing`: 規範子彈線段渲染物件必須以發射點到初始目標位置的方向建立，並在飛行期間維持該視覺朝向。

### Modified Capabilities

## Impact

- 影響 `omfx` 前端中子彈線段或投射物視覺節點的建立、更新與旋轉邏輯。
- 不預期修改 server gameplay API、transport protocol、script ABI 或 content data。
- 可能需要調整既有 rendering 測試、debug log 或手動驗證流程，以確認視覺方向穩定。
