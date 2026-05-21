# UECoPilot Tasks

Purpose: close the remaining UE Editor / Blueprint / visual gaps that C++ automation cannot prove by itself. Keep all handmade assets under `D:\omoba\omfue\Content\RustBP` and `/Game/RustBP`. Do not introduce `omfue` or `Omfue` prefixes in asset, class, function, or event names; `Om` is the only allowed prefix when one is needed.

## Current Automated Baseline

- Command smoke already passes from repo root:
  - `D:\omoba\run_ue.bat --headless-smoke --single-player`
- C++ automation passes from repo root:
  - `D:\omoba\run_ue_tests.bat`
- C++ automation verifies:
  - `/Game/Map/Main` default map settings.
  - default runtime mode is single-player `TD_1`.
  - every `/Game/RustBP` registry Blueprint class loads and inherits its generated C++ class.
  - system/UI Blueprint parent classes load.
  - `AOmMapRouteActor` can procedurally build a spline route.
  - `Om.Playable.LinkedSinglePlayerLockstepSmoke` directly links through `om_bridge.lib`, starts a local lockstep runtime with `OM_RUNTIME_FLAG_SINGLE_PLAYER`, reads ticked frame/catalog data, confirms no network rx bytes, and submits `InputStartRound` without launching `omobab.exe`.

## UECoPilot Must Verify Or Implement

- [x] Run PIE in `D:\omoba\omfue\Content\Map\Main.umap` and confirm first viewport is playable, not only loadable:
  - `BP_GameMode`, `BP_PlayerController`, `BP_RtsCameraPawn`, `BP_WorldBridge`, `BP_MapRoute`, and `WBP_OmHudRoot` all start.
  - Visible first screen includes map route line, tower shop, Start Round, hero HUD, ability bar, buff list area, diagnostics, and selectable hero.
  - No Blueprint load failure, missing class, widget construction error, or runtime Error log.

- [x] Implement or fix `BP_PlayerController` input graph so it matches this priority order:
  - UI capture first.
  - Pending ability target, attack move, and tower placement next.
  - Tower selection next.
  - General right-click move/attack last.
  - Clicking UI buttons, scrollbar, tower cards, ability slots, or diagnostics must not also move hero or place a tower.

- [x] Implement PIE interaction path from UI to Rust bridge:
  - Start Round button calls `SubmitStartRound`.
  - Right-click ground calls `SubmitMoveFromScreenEx`.
  - Shift + right-click ground calls `SubmitMoveFromScreenEx` with queued input.
  - `A` + click ground calls `SubmitAttackMoveFromScreen`.
  - `W/E/R/T` casts ability or enters targeting mode; `Shift+W/E/R/T` calls `SubmitUpgradeAbility`.
  - Number keys or tower card click enter tower placement mode.
  - Valid left-click placement calls `SubmitPlaceTowerFromScreen`.
  - Selected tower panel calls `SubmitUpgradeTower`, `SubmitSellTower`, and `SubmitSetTowerTargetPriority`.
  - Every submit path must surface accepted/rejected status in diagnostics.

- [x] Make `WBP_OmHudRoot` a real playable HUD:
  - Owns or references `WBP_OmTowerShopPanel`, `WBP_OmSelectedTowerPanel`, `WBP_OmHeroHud`, `WBP_OmAbilityBar`, `WBP_OmBuffList`, `WBP_OmEntityOverlayLayer`, and `WBP_OmDiagnostics`.
  - Binds to `BP_WorldBridge` delegates: `TowerShopSelectionChanged`, `TowerPlacementPreviewChanged`, `SelectedTowerChanged`, `HeroHudStateChanged`, `AbilityHudStateChanged`, `BuffListStateChanged`, `EntityOverlayStateChanged`, and `RuntimeDiagnosticsChanged`.
  - Calls `SetHudInputGuard(true/false)` when pointer is over UI.

- [x] Make tower placement visibly usable:
  - `/Game/RustBP/System/BP_OmPlacementPreview` follows mouse while placement mode is active.
  - Shows tower ghost, footprint, attack range, and valid/invalid material.
  - Invalid reasons at least distinguish route blocked, occupied, not enough gold, outside map, and unknown tower.
  - Right-click or Escape cancels placement mode.
  - Ctrl held after a successful placement keeps placement mode active for continuous building.

- [x] Make selected tower actions visibly usable:
  - Left-click owned tower selects it and opens `WBP_OmSelectedTowerPanel`.
  - Empty click clears selection.
  - Selected tower range overlay is visible.
  - Upgrade path buttons show cost, level, availability, and rejection reason.
  - Sell button clears selected tower after accepted submit.
  - Target priority button and `P` cycle priority and update UI.

- [x] Verify `BP_RtsCameraPawn` controls in real PIE input:
  - Mouse at left/right/top/bottom viewport edge pans camera.
  - Mouse wheel zooms in/out unless pointer is over a scrollable UI panel.
  - Camera remains finite and clamped to map bounds.
  - UI guard suppresses edge scroll while pointer is over HUD.

- [x] Verify `BP_MapRoute` visual output:
  - Runtime route data creates a visible spline/line in Main.
  - Direction markers or node markers are visible enough for TD path reading.
  - Route material, width, and vertical offset are editable in Blueprint defaults.
  - Tower placement preview treats route as blocked.

- [x] Complete Saika hero visuals in `/Game/RustBP/Heroes/BP_SaikaMagoichi`:
  - Uses original Saika skeletal mesh/material, not capsule-only.
  - Has AnimBP with states or montages for `stand_1`, `stand_2`, `stand_3`, `walk`, `sniper_walk`, `attack`, `CriticalAttack`, `cast`, `cast2`, and `death`.
  - `OnAnimationState` updates AnimBP variables: `Locomotion`, `LocomotionVariant`, `IdleVariant`, `AnimationOverlay`, `ActionState`, `AttackPhase`, `PhaseProgress`, `ActionInstanceId`, `bCriticalAttack`, and `PlayRate`.
  - `OnAttackPhase` handles `Windup`, `Impact`, and `Recovery`; Impact spawns muzzle flash/tracer from `Weapon Ref`.
  - `HandleSaikaActionEvent` is wired and preferred when present.

- [x] Complete Saika ability and buff visuals:
  - `/Game/RustBP/Abilities/BP_SniperMode`: scope ring, weapon glow, enabled/disabled cue.
  - `/Game/RustBP/Abilities/BP_SaikaReinforcements`: target marker, summon circle, spawn markers based on `SummonCount`.
  - `/Game/RustBP/Abilities/BP_RainIronCannon`: 90-degree impact cone using `AoeRadius` and `AttackDirectionRadians`.
  - `/Game/RustBP/Abilities/BP_ThreeStageTechnique`: transform flash, aura, multi-shot cue.
  - `/Game/RustBP/Buffs/BP_SniperMode`: add/remove aura and switch walk overlay to `sniper_walk`.
  - `/Game/RustBP/Buffs/BP_ThreeStage`: weapon trail on add, cleanup on remove.
  - All buff visuals use `VisualInstanceKey` with `TrackBuffEffect` and `UntrackBuffEffect`; no lingering effects after remove/despawn.

- [x] Complete tower/projectile visuals:
  - Five tower BPs have distinct readable silhouettes and fire feedback: dart, tack, bomb, ice, cake splash.
  - All nine projectile BPs implement `OnProjectileCue` visibly: dart, spike_opult, tack, tack_blade, bomb, bomb_frag, saika_shot, ice, icicle.
  - Tower fire at `Impact` spawns correct projectile or tracer and shows recoil/flash.

- [x] Complete creep/summon visuals:
  - `/Game/RustBP/Summons/BP_SaikaGunner` has rifleman/summon visual and attack cue.
  - Creeps are visually distinguishable by family: mage, dummy, wolf, lane minion, shooter, TD balloon/basic/tough/stress.
  - HP bar/name/team color overlay works for large numbers and can be throttled/hidden.

- [x] Add UECoPilot PIE screenshot or log evidence to `D:\omoba\omfue\README.md` after verification:
  - Start Round accepted.
  - Tower placement accepted and invalid placement rejected with reason.
  - Selected tower upgrade/sell/priority accepted.
  - Right-click move and Shift move accepted.
  - `W/E/R/T` cast or target mode works.
  - Saika sniper mode changes walk visual.
  - Buff add/remove effect cleanup confirmed.
  - Projectile/tracer visible.

## Expected Completion State

The frontend is not done when assets merely load. It is done when `Main.umap` Play can run a local `TD_1` session in UE Editor and the user can start a round, move Saika, cast abilities, place/upgrade/sell towers, see route/placement/selection/HUD feedback, and observe buff/projectile/ability visuals without using omfx.
