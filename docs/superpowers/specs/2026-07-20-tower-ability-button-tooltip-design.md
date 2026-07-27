# Tower Ability Button Tooltip Design

## Goal

Simplify every tower active-ability button to a large, readable ability name while moving detailed ability information and live state into a mouse-hover tooltip.

## Scope

- Apply to every tower active ability shown in the global tower ability bar, including existing and future level-four abilities.
- Change `omfx` presentation and presentation-model logic, plus the minimal `omoba-core` render snapshot field needed to expose authored total duration.
- Preserve ability discovery, slot assignment, pagination, keyboard shortcuts, clicking, cast validation, cooldown timing, and backend authority.
- Do not modify tower upgrade definitions, ability balance, cast behavior, transport protocol, or `omfue`.

## Chosen Approach

Extend the existing dedicated `AbilityBarTooltipModel` and ability-bar rendering path. This reuses the current authoritative `AbilityBarItem` data, hover hit-testing, and tooltip widgets without introducing a second HUD panel or a new popup framework.

## Button Presentation

Each occupied ability slot displays only `ability_name` as its text:

- Remove the tower label, shortcut prefix, `READY`, active/cooldown seconds, rejection reason, fallback icon label, and authored/fallback image from the button surface.
- Render the ability name as larger centered text.
- Permit long names to wrap to at most two centered lines within the existing slot.
- Preserve the slot background colors, cooldown fill overlay, active overlay, click target, pagination, and keyboard shortcut behavior.
- An empty slot remains hidden exactly as it is today.

Although the button no longer prints state text, its background and overlays continue to communicate ready, active, cooling, and temporarily disabled states at a glance.

## Tooltip Content

Hovering an occupied ability slot shows a dedicated tooltip containing:

1. Ability name as the title.
2. `Tower: <tower label>`.
3. The complete authored ability description. If it is empty, show `No ability description available.`.
4. Total cooldown formatted to one decimal place.
5. Duration formatted to one decimal place, or `Instant` when the duration is zero.
6. Live status:
   - `Ready` when the ability can be cast.
   - `Active: <seconds>s remaining` while active.
   - `Cooldown: <seconds>s remaining` while cooling down.
7. `Shortcut: <number>` for the current visible-page slot.
8. `Last cast rejected: <reason>` only while a rejection for this exact ability remains visible.

The implementation may use localized Chinese labels matching the surrounding UI. Tests assert the information and values rather than depending on incidental whitespace.

## Data Flow

`TowerActiveAbilitySnapshot` gains a `duration_total` display field populated from the existing authored active-ability definition. This is render-only snapshot data and does not change authoritative ability state or network transport. The snapshot continues to populate `AbilityBarItem`. A pure presentation helper builds:

- the button label from `ability_name` only; and
- the tooltip model from the ability item plus the currently visible rejection for the same `AbilityBarKey`.

`update_tower_ability_bar_ui` keeps its current hover hit-test. When the hovered key or any live tooltip field changes, it updates the existing tooltip title and description widgets. This is important because cooldown and active seconds change even when the cursor remains over the same button.

The tooltip stays near the cursor and remains clamped within the game window. Leaving the slot hides it.

## Error Handling

- Missing authored descriptions use the explicit fallback sentence.
- Non-finite or negative cooldown, duration, and remaining values display as zero.
- A rejection appears only for the ability key that produced it and disappears with the existing rejection lifetime.
- Missing ability icons no longer affect button readability because buttons do not render images.

## Testing

Add focused pure-model tests in `omfx/game/src/native.rs` covering:

- Button text equals only the ability name.
- Long names use the existing wrapping constraint without adding tower or state text.
- Ready tooltip content includes tower, description, total cooldown, instant/duration, status, and shortcut.
- Active and cooling tooltips report their respective remaining time.
- Empty descriptions and invalid numeric values use safe fallbacks.
- Rejection text is included only for the matching ability key.
- `None` hover still produces no tooltip.

Add an `omoba-core` snapshot test proving `duration_total` is copied from the unlocked ability definition.

Run the focused `omoba-core` snapshot test, focused `omfx` ability-bar tests, and the relevant full frontend test suite. Existing ability cast, slot reconciliation, pagination, and cooldown tests must continue to pass.

## Acceptance Criteria

- Every visible tower ability button contains only a large ability name.
- Hovering any button shows its detailed description, tower, timing, live state, shortcut, and matching rejection information.
- Cooling and active state information refreshes while the pointer remains stationary over the button.
- Clicking and numeric shortcuts still cast the same authoritative ability.
- No backend or tower balance behavior changes.
