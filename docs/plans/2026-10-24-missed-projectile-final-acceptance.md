# Immutable missed ordinary projectiles COMPLETE

OpenSpec complete-missed-projectile-outcomes4/4.

Native hero and typed script ordinary shots now retain their already computed source-side miss in authority-only immutable provenance. The homing impact bypasses damage, hit callbacks and control envelopes, emits the existing target-only MISS cue once when the exact target still exists, then retires normally. Source expiry or ID reuse cannot repair the committed result. Visual pellets and hit-context children do not inherit the flag; zero-damage abilities are never classified as missed by damage amount. Legacy serialized origins default to no source miss. No public attacker/reason or ABI changes.

Fresh Rust 1907 passed/0failed/9existing ignored across eight packages; Lua11/11, backend and UE5.8 builds actual exit0. Native 92 distinct explicitly selected cases (91 Success/1 SuccessWithWarnings, 4 existing warnings), including PublicHealthTextLifecycle with required actual MISS PNG. Formal60Hz hero shared/legacy/both miss sources, real summons after source expiry, late modifier changes, primary/visual separation, exact source provenance, legitimate zero damage and target removal passed.

3472 formal sources frozen/verified;3462 unchanged/10 explicit changed versus strict index-bound avoidance-feedback parent. 525 independently rehashed files; 7 original workers retired with validated actual Windows exits, max health gap151297ms. Original edit failures and original input copies retained. All64 reviewed images are inherited with exact SHA and recorded prior coverage; fresh native raw captures retained separately, not claimed as newly viewed.

Remaining: more shared critical/hit and geometric gameplay, full combat replica, presentation/art and11external acceptance/MVP33/44. Next concrete investigation: compare shared UnitStats::crit modifiers with native DamageInstance and hero ordinary projectile launch; do not assume shared stat getters are already consumed. No true network match, quota reset, commit/push or omfx work. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.
