# Requirements

- A separate typed PlayerInput SHALL address the exact owner hero and exact summoned entity generations; operation and point shape SHALL be closed and bounded.
- Authority SHALL reject foreign, stale, dead, expired, non-summon, mismatched-faction or inactive recipients without mutating either hero or summon.
- Accepted movement SHALL use existing terrain/control/navigation and movement budgets. Release SHALL remove the explicit destination and stale navigation cache so existing autonomy can resume.
- An owned summon command SHALL NOT clear or cancel the owner's hero commands, spells or recovery channel.
- Renderer exposure SHALL remain explicitly unsupported until its own scoped identity/transport/UI integration is implemented.
