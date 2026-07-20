# Tower Ability Bar Vertical Offset Design

**Date:** 2026-07-14

## Goal

Move the bottom-center tower ability bar upward by exactly 70 screen pixels so it has more breathing room above the bottom edge.

## Design

The ability bar currently uses an 18-pixel bottom margin. Replace that implicit margin with a named 88-pixel bottom margin. Since the slot height and window height remain unchanged, this moves the bar upward by exactly 70 pixels at every resolution.

The previous-page control, next-page control, page indicator, hover hit rectangles, and tooltip continue to derive their positions from the ability bar slot position. They therefore move with the bar without separate offsets.

No other HUD elements, tower-shop layout, hero ability controls, slot dimensions, or paging behavior change.

## Testing

Extract the vertical-position calculation into a small pure helper and test that:

- A 1080-pixel-high window places an 88-pixel-tall slot at `y = 904`.
- A 720-pixel-high window places the same slot at `y = 544`.
- Very short windows still clamp the position to zero.

Run the focused layout test, the `omfx` test suite, and an executor compile check with the development runtime-Lua feature.
