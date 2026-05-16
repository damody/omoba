## Purpose

定義 omfx projectile 線段視覺的初始朝向與飛行期間穩定性，避免子彈拖尾因目標移動或 snapshot 更新而旋轉到錯誤方向。

## Requirements

### Requirement: 子彈線段使用初始發射方向

前端渲染系統 SHALL 在第一次建立 projectile 線段視覺時，以發射點到初始目標位置或第一個有效 projectile 位置的向量決定線段朝向。

#### Scenario: 子彈從塔射向敵人時正確面向目標

- **WHEN** projectile 第一次被前端渲染，且可取得發射點與目標方向
- **THEN** 線段的 `rotation` MUST 對齊發射點到初始目標位置或第一個有效 projectile 位置的方向

#### Scenario: 初始方向向量太短時使用安全 fallback

- **WHEN** projectile 第一次被前端渲染，但發射點與可用位置幾乎重疊
- **THEN** 前端 MUST 使用穩定 fallback 方向，且不得產生 NaN 或無效 transform

### Requirement: 子彈線段飛行期間不改變視覺朝向

前端渲染系統 SHALL 在 projectile 線段建立後維持其初始視覺朝向，飛行期間不得因目標移動、snapshot 更新或追蹤插值而重新旋轉線段。

#### Scenario: 目標移動時線段不旋轉

- **WHEN** projectile 已建立線段視覺，且目標在 projectile 飛行期間改變位置
- **THEN** 線段的 `rotation` MUST 保持建立時的值

#### Scenario: projectile 位置更新時只更新位置與長度

- **WHEN** projectile 的 render position 隨 snapshot 或插值更新
- **THEN** 前端 MUST 更新線段位置與可見拖尾長度，但 MUST NOT 用新的位移向量覆寫初始 `rotation`
